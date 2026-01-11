//! RegenerateResponse command handler.
//!
//! Handles regenerating the last AI response in a conversation.
//! Removes the previous assistant message and generates a new one.

use crate::domain::conversation::{AgentPhase, ConversationState, PhaseTransitionEngine};
use crate::domain::foundation::{ComponentId, ConversationId, DomainError, UserId};
use crate::ports::{AIError, AIProvider, CompletionRequest, RequestMetadata, TokenUsage};
use async_trait::async_trait;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;

use super::send_message::{
    ComponentOwnershipChecker, ConversationRepository, MessageId, MessageRole, StoredMessage,
    StreamEvent,
};

/// Command to regenerate the last AI response.
#[derive(Debug, Clone)]
pub struct RegenerateResponseCommand {
    /// The user requesting regeneration.
    pub user_id: UserId,
    /// The component's conversation to regenerate in.
    pub component_id: ComponentId,
}

impl RegenerateResponseCommand {
    /// Creates a new regenerate response command.
    pub fn new(user_id: UserId, component_id: ComponentId) -> Self {
        Self {
            user_id,
            component_id,
        }
    }
}

/// Errors that can occur when regenerating a response.
#[derive(Debug, Clone, Error)]
pub enum RegenerateResponseError {
    /// User is not authorized to access this conversation.
    #[error("Forbidden: user does not own this conversation")]
    Forbidden,

    /// Conversation has no messages to regenerate.
    #[error("No messages to regenerate: conversation is empty")]
    NoMessagesToRegenerate,

    /// Last message is not from assistant (can't regenerate user message).
    #[error("Cannot regenerate: last message is not from assistant")]
    LastMessageNotAssistant,

    /// Conversation is in Complete state and cannot regenerate.
    #[error("Conversation is complete and cannot regenerate responses")]
    ConversationComplete,

    /// Conversation was not found.
    #[error("Conversation not found for component {0}")]
    ConversationNotFound(ComponentId),

    /// AI provider error during response generation.
    #[error("AI provider error: {0}")]
    AIProviderError(String),

    /// Repository error during persistence.
    #[error("Repository error: {0}")]
    RepositoryError(String),

    /// Domain error.
    #[error("Domain error: {0}")]
    DomainError(String),
}

impl From<DomainError> for RegenerateResponseError {
    fn from(err: DomainError) -> Self {
        RegenerateResponseError::DomainError(err.to_string())
    }
}

impl From<AIError> for RegenerateResponseError {
    fn from(err: AIError) -> Self {
        RegenerateResponseError::AIProviderError(err.to_string())
    }
}

/// Result of regenerating a response.
#[derive(Debug, Clone)]
pub struct RegenerateResponseResult {
    /// ID of the deleted assistant message.
    pub deleted_message_id: MessageId,
    /// ID of the new assistant response message.
    pub new_message_id: MessageId,
    /// New conversation phase after processing.
    pub new_phase: AgentPhase,
    /// Token usage for the new response.
    pub usage: Option<TokenUsage>,
}

/// Extended conversation repository with delete capability.
#[async_trait]
pub trait ConversationRepositoryExt: ConversationRepository {
    /// Deletes the last message from a conversation.
    ///
    /// Returns the ID of the deleted message, or None if no messages exist.
    async fn delete_last_message(
        &self,
        conversation_id: &ConversationId,
    ) -> Result<Option<MessageId>, DomainError>;
}

/// Handler for RegenerateResponse commands.
pub struct RegenerateResponseHandler<O, R, A>
where
    O: ComponentOwnershipChecker,
    R: ConversationRepositoryExt,
    A: AIProvider,
{
    ownership_checker: Arc<O>,
    conversation_repo: Arc<R>,
    ai_provider: Arc<A>,
}

impl<O, R, A> RegenerateResponseHandler<O, R, A>
where
    O: ComponentOwnershipChecker + 'static,
    R: ConversationRepositoryExt + 'static,
    A: AIProvider + 'static,
{
    /// Creates a new handler with the given dependencies.
    pub fn new(
        ownership_checker: Arc<O>,
        conversation_repo: Arc<R>,
        ai_provider: Arc<A>,
    ) -> Self {
        Self {
            ownership_checker,
            conversation_repo,
            ai_provider,
        }
    }

    /// Handles a regenerate response command.
    ///
    /// Returns a channel receiver for streaming events plus the final result.
    pub async fn handle(
        &self,
        cmd: RegenerateResponseCommand,
    ) -> Result<(mpsc::Receiver<StreamEvent>, RegenerateResponseResult), RegenerateResponseError>
    {
        // R10: Verify ownership through session chain
        let ownership = self
            .ownership_checker
            .check_ownership(&cmd.user_id, &cmd.component_id)
            .await
            .map_err(|_| RegenerateResponseError::Forbidden)?;

        // Get existing conversation
        let mut conversation = self
            .conversation_repo
            .find_by_component(&cmd.component_id)
            .await?
            .ok_or(RegenerateResponseError::ConversationNotFound(cmd.component_id))?;

        // R15: Check conversation state is not Complete
        if conversation.state == ConversationState::Complete {
            return Err(RegenerateResponseError::ConversationComplete);
        }

        // R11: Check conversation has messages
        if !conversation.has_messages() {
            return Err(RegenerateResponseError::NoMessagesToRegenerate);
        }

        // R14: Validate last message role is assistant
        let last_message = conversation.last_message().unwrap();
        if last_message.role != MessageRole::Assistant {
            return Err(RegenerateResponseError::LastMessageNotAssistant);
        }

        let deleted_message_id = last_message.id;

        // R12: Delete last assistant message
        self.conversation_repo
            .delete_last_message(&conversation.id)
            .await?;
        conversation.messages.pop();

        // R13: Generate new AI response with same context
        let new_message_id = MessageId::new();
        let (tx, rx) = mpsc::channel(32);

        // Build request with remaining messages
        let request = CompletionRequest::new(RequestMetadata::new(
            cmd.user_id.clone(),
            ownership.session_id,
            conversation.id,
            format!("regen-{}", new_message_id),
        ))
        .with_system_prompt(&conversation.system_prompt)
        .with_component_type(ownership.component_type);

        // Add remaining messages (without the deleted one)
        let mut request = request;
        for msg in conversation.messages_for_ai() {
            request = request.with_message(msg.role, &msg.content);
        }

        // Stream the new response
        let stream = self.ai_provider.stream_complete(request).await?;

        let conversation_id = conversation.id;
        let conversation_repo = Arc::clone(&self.conversation_repo);

        let handle = tokio::spawn(async move {
            let mut full_content = String::new();
            let mut final_usage = None;
            let mut stream = stream;

            loop {
                use futures::StreamExt;
                match stream.next().await {
                    Some(Ok(chunk)) => {
                        let delta = chunk.delta.clone();
                        let is_final = chunk.is_final();
                        let usage = chunk.usage.clone();

                        full_content.push_str(&delta);

                        let _ = tx
                            .send(StreamEvent::Chunk {
                                message_id: new_message_id,
                                delta,
                            })
                            .await;

                        if is_final {
                            final_usage = usage;
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        let _ = tx
                            .send(StreamEvent::Error {
                                message_id: new_message_id,
                                error: e.to_string(),
                            })
                            .await;
                        return Err(RegenerateResponseError::AIProviderError(e.to_string()));
                    }
                    None => break,
                }
            }

            // Store new assistant message
            let mut assistant_msg =
                StoredMessage::assistant_with_id(new_message_id, &full_content);
            if let Some(ref usage) = final_usage {
                assistant_msg = assistant_msg.with_token_count(usage.completion_tokens);
            }
            conversation_repo
                .add_message(&conversation_id, assistant_msg)
                .await?;

            // Send complete event
            let _ = tx
                .send(StreamEvent::Complete {
                    message_id: new_message_id,
                    full_content: full_content.clone(),
                    usage: final_usage.clone(),
                })
                .await;

            Ok((full_content, final_usage))
        });

        // Wait for streaming to complete
        let (_full_content, usage) = handle
            .await
            .map_err(|e| RegenerateResponseError::DomainError(e.to_string()))??;

        // Determine new phase using transition engine
        let engine = PhaseTransitionEngine::for_component(ownership.component_type);
        let latest_user_msg = conversation
            .messages
            .iter()
            .rev()
            .find(|m| m.role == MessageRole::User)
            .map(|m| m.content.clone());
        let snapshot = crate::domain::conversation::ConversationSnapshot::new(
            conversation.user_message_count(),
            latest_user_msg,
            ownership.component_type,
        );
        let new_phase = engine.next_phase(conversation.phase, &snapshot);

        // Update conversation phase
        self.conversation_repo
            .update_state(&conversation.id, conversation.state, new_phase)
            .await?;

        Ok((
            rx,
            RegenerateResponseResult {
                deleted_message_id,
                new_message_id,
                new_phase,
                usage,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::handlers::conversation::ConversationRecord;
    use crate::domain::foundation::{ComponentType, CycleId, ErrorCode, SessionId, Timestamp};
    use crate::ports::StreamChunk as AIStreamChunk;
    use super::super::send_message::OwnershipInfo;
    use futures::stream;
    use std::sync::Mutex;

    // Mock implementations for testing

    struct MockOwnershipChecker {
        should_allow: bool,
        ownership_info: Option<OwnershipInfo>,
    }

    impl MockOwnershipChecker {
        fn allowing() -> Self {
            Self {
                should_allow: true,
                ownership_info: Some(OwnershipInfo {
                    session_id: SessionId::new(),
                    cycle_id: CycleId::new(),
                    component_type: ComponentType::IssueRaising,
                }),
            }
        }

        fn denying() -> Self {
            Self {
                should_allow: false,
                ownership_info: None,
            }
        }
    }

    #[async_trait]
    impl ComponentOwnershipChecker for MockOwnershipChecker {
        async fn check_ownership(
            &self,
            _user_id: &UserId,
            _component_id: &ComponentId,
        ) -> Result<OwnershipInfo, DomainError> {
            if self.should_allow {
                Ok(self.ownership_info.clone().unwrap())
            } else {
                Err(DomainError::new(
                    ErrorCode::Forbidden,
                    "User does not own component",
                ))
            }
        }
    }

    struct MockConversationRepoExt {
        conversations: Mutex<Vec<ConversationRecord>>,
        messages: Mutex<Vec<(ConversationId, StoredMessage)>>,
        deleted_messages: Mutex<Vec<MessageId>>,
    }

    impl MockConversationRepoExt {
        fn new() -> Self {
            Self {
                conversations: Mutex::new(Vec::new()),
                messages: Mutex::new(Vec::new()),
                deleted_messages: Mutex::new(Vec::new()),
            }
        }

        fn with_conversation(conversation: ConversationRecord) -> Self {
            Self {
                conversations: Mutex::new(vec![conversation]),
                messages: Mutex::new(Vec::new()),
                deleted_messages: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl ConversationRepository for MockConversationRepoExt {
        async fn find_by_component(
            &self,
            component_id: &ComponentId,
        ) -> Result<Option<ConversationRecord>, DomainError> {
            let convs = self.conversations.lock().unwrap();
            Ok(convs
                .iter()
                .find(|c| c.component_id == *component_id)
                .cloned())
        }

        async fn create(
            &self,
            component_id: &ComponentId,
            component_type: ComponentType,
            user_id: &UserId,
            system_prompt: &str,
        ) -> Result<ConversationRecord, DomainError> {
            let conv = ConversationRecord {
                id: ConversationId::new(),
                component_id: *component_id,
                component_type,
                state: ConversationState::Ready,
                phase: AgentPhase::Intro,
                messages: Vec::new(),
                user_id: user_id.clone(),
                system_prompt: system_prompt.to_string(),
                created_at: Timestamp::now(),
                updated_at: Timestamp::now(),
            };
            self.conversations.lock().unwrap().push(conv.clone());
            Ok(conv)
        }

        async fn save(&self, conversation: &ConversationRecord) -> Result<(), DomainError> {
            let mut convs = self.conversations.lock().unwrap();
            if let Some(c) = convs.iter_mut().find(|c| c.id == conversation.id) {
                *c = conversation.clone();
            }
            Ok(())
        }

        async fn add_message(
            &self,
            conversation_id: &ConversationId,
            message: StoredMessage,
        ) -> Result<(), DomainError> {
            self.messages
                .lock()
                .unwrap()
                .push((*conversation_id, message));
            Ok(())
        }

        async fn update_state(
            &self,
            conversation_id: &ConversationId,
            state: ConversationState,
            phase: AgentPhase,
        ) -> Result<(), DomainError> {
            let mut convs = self.conversations.lock().unwrap();
            if let Some(c) = convs.iter_mut().find(|c| c.id == *conversation_id) {
                c.state = state;
                c.phase = phase;
            }
            Ok(())
        }

        async fn find_by_id(
            &self,
            conversation_id: &ConversationId,
        ) -> Result<Option<ConversationRecord>, DomainError> {
            let convs = self.conversations.lock().unwrap();
            Ok(convs.iter().find(|c| c.id == *conversation_id).cloned())
        }

        async fn get_messages(
            &self,
            conversation_id: &ConversationId,
            offset: u32,
            limit: u32,
        ) -> Result<(Vec<StoredMessage>, u32), DomainError> {
            let convs = self.conversations.lock().unwrap();
            if let Some(c) = convs.iter().find(|c| c.id == *conversation_id) {
                let total = c.messages.len() as u32;
                let messages: Vec<_> = c.messages
                    .iter()
                    .skip(offset as usize)
                    .take(limit as usize)
                    .cloned()
                    .collect();
                Ok((messages, total))
            } else {
                Ok((Vec::new(), 0))
            }
        }
    }

    #[async_trait]
    impl ConversationRepositoryExt for MockConversationRepoExt {
        async fn delete_last_message(
            &self,
            conversation_id: &ConversationId,
        ) -> Result<Option<MessageId>, DomainError> {
            let mut convs = self.conversations.lock().unwrap();
            if let Some(c) = convs.iter_mut().find(|c| c.id == *conversation_id) {
                if let Some(msg) = c.messages.pop() {
                    self.deleted_messages.lock().unwrap().push(msg.id);
                    return Ok(Some(msg.id));
                }
            }
            Ok(None)
        }
    }

    struct MockAIProvider {
        response: String,
    }

    impl MockAIProvider {
        fn with_response(response: impl Into<String>) -> Self {
            Self {
                response: response.into(),
            }
        }
    }

    #[async_trait]
    impl AIProvider for MockAIProvider {
        async fn complete(
            &self,
            _request: CompletionRequest,
        ) -> Result<crate::ports::CompletionResponse, AIError> {
            Ok(crate::ports::CompletionResponse {
                content: self.response.clone(),
                usage: TokenUsage::new(10, 20, 1),
                model: "mock".to_string(),
                finish_reason: crate::ports::FinishReason::Stop,
            })
        }

        async fn stream_complete(
            &self,
            _request: CompletionRequest,
        ) -> Result<
            std::pin::Pin<Box<dyn futures::Stream<Item = Result<AIStreamChunk, AIError>> + Send>>,
            AIError,
        > {
            let response = self.response.clone();
            let chunks = vec![
                Ok(AIStreamChunk::content(&response)),
                Ok(AIStreamChunk::final_chunk(
                    crate::ports::FinishReason::Stop,
                    TokenUsage::new(10, 20, 1),
                )),
            ];
            Ok(Box::pin(stream::iter(chunks)))
        }

        fn estimate_tokens(&self, text: &str) -> u32 {
            (text.len() / 4) as u32
        }

        fn provider_info(&self) -> crate::ports::ProviderInfo {
            crate::ports::ProviderInfo::new("mock", "mock-model", 4096)
        }
    }

    fn sample_conversation_with_messages(component_id: ComponentId) -> ConversationRecord {
        ConversationRecord {
            id: ConversationId::new(),
            component_id,
            component_type: ComponentType::IssueRaising,
            state: ConversationState::InProgress,
            phase: AgentPhase::Gather,
            messages: vec![
                StoredMessage::user("Hello"),
                StoredMessage::assistant("Hi there!"),
            ],
            user_id: UserId::new("user").unwrap(),
            system_prompt: "Test".to_string(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        }
    }

    mod r10_ownership_verification {
        use super::*;

        #[tokio::test]
        async fn rejects_when_user_does_not_own_conversation() {
            let component_id = ComponentId::new();
            let conversation = sample_conversation_with_messages(component_id);

            let handler = RegenerateResponseHandler::new(
                Arc::new(MockOwnershipChecker::denying()),
                Arc::new(MockConversationRepoExt::with_conversation(conversation)),
                Arc::new(MockAIProvider::with_response("New response")),
            );

            let cmd = RegenerateResponseCommand::new(
                UserId::new("non-owner").unwrap(),
                component_id,
            );

            let result = handler.handle(cmd).await;
            assert!(matches!(result, Err(RegenerateResponseError::Forbidden)));
        }
    }

    mod r11_has_messages {
        use super::*;

        #[tokio::test]
        async fn rejects_when_conversation_has_no_messages() {
            let component_id = ComponentId::new();
            let conversation = ConversationRecord {
                id: ConversationId::new(),
                component_id,
                component_type: ComponentType::IssueRaising,
                state: ConversationState::Ready,
                phase: AgentPhase::Intro,
                messages: Vec::new(), // No messages
                user_id: UserId::new("user").unwrap(),
                system_prompt: "Test".to_string(),
                created_at: Timestamp::now(),
                updated_at: Timestamp::now(),
            };

            let handler = RegenerateResponseHandler::new(
                Arc::new(MockOwnershipChecker::allowing()),
                Arc::new(MockConversationRepoExt::with_conversation(conversation)),
                Arc::new(MockAIProvider::with_response("Response")),
            );

            let cmd = RegenerateResponseCommand::new(
                UserId::new("user").unwrap(),
                component_id,
            );

            let result = handler.handle(cmd).await;
            assert!(matches!(
                result,
                Err(RegenerateResponseError::NoMessagesToRegenerate)
            ));
        }
    }

    mod r14_last_message_role {
        use super::*;

        #[tokio::test]
        async fn rejects_when_last_message_is_user() {
            let component_id = ComponentId::new();
            let conversation = ConversationRecord {
                id: ConversationId::new(),
                component_id,
                component_type: ComponentType::IssueRaising,
                state: ConversationState::InProgress,
                phase: AgentPhase::Gather,
                messages: vec![
                    StoredMessage::assistant("Hello!"),
                    StoredMessage::user("My response"), // Last message is user
                ],
                user_id: UserId::new("user").unwrap(),
                system_prompt: "Test".to_string(),
                created_at: Timestamp::now(),
                updated_at: Timestamp::now(),
            };

            let handler = RegenerateResponseHandler::new(
                Arc::new(MockOwnershipChecker::allowing()),
                Arc::new(MockConversationRepoExt::with_conversation(conversation)),
                Arc::new(MockAIProvider::with_response("Response")),
            );

            let cmd = RegenerateResponseCommand::new(
                UserId::new("user").unwrap(),
                component_id,
            );

            let result = handler.handle(cmd).await;
            assert!(matches!(
                result,
                Err(RegenerateResponseError::LastMessageNotAssistant)
            ));
        }
    }

    mod r15_conversation_state {
        use super::*;

        #[tokio::test]
        async fn rejects_when_conversation_complete() {
            let component_id = ComponentId::new();
            let conversation = ConversationRecord {
                id: ConversationId::new(),
                component_id,
                component_type: ComponentType::IssueRaising,
                state: ConversationState::Complete, // Complete state
                phase: AgentPhase::Confirm,
                messages: vec![
                    StoredMessage::user("Hello"),
                    StoredMessage::assistant("Hi!"),
                ],
                user_id: UserId::new("user").unwrap(),
                system_prompt: "Test".to_string(),
                created_at: Timestamp::now(),
                updated_at: Timestamp::now(),
            };

            let handler = RegenerateResponseHandler::new(
                Arc::new(MockOwnershipChecker::allowing()),
                Arc::new(MockConversationRepoExt::with_conversation(conversation)),
                Arc::new(MockAIProvider::with_response("Response")),
            );

            let cmd = RegenerateResponseCommand::new(
                UserId::new("user").unwrap(),
                component_id,
            );

            let result = handler.handle(cmd).await;
            assert!(matches!(
                result,
                Err(RegenerateResponseError::ConversationComplete)
            ));
        }
    }

    mod r12_and_r13_regeneration {
        use super::*;

        #[tokio::test]
        async fn deletes_last_message_and_generates_new_response() {
            let component_id = ComponentId::new();
            let conversation = sample_conversation_with_messages(component_id);
            let repo = Arc::new(MockConversationRepoExt::with_conversation(conversation));

            let handler = RegenerateResponseHandler::new(
                Arc::new(MockOwnershipChecker::allowing()),
                Arc::clone(&repo),
                Arc::new(MockAIProvider::with_response("New AI response")),
            );

            let cmd = RegenerateResponseCommand::new(
                UserId::new("user").unwrap(),
                component_id,
            );

            let result = handler.handle(cmd).await;
            assert!(result.is_ok());

            let (mut rx, result) = result.unwrap();

            // Verify a message was deleted
            let deleted = repo.deleted_messages.lock().unwrap();
            assert_eq!(deleted.len(), 1);

            // Verify new message was stored
            let messages = repo.messages.lock().unwrap();
            assert_eq!(messages.len(), 1);

            // Verify stream events
            let mut received_complete = false;
            while let Ok(event) = rx.try_recv() {
                if matches!(event, StreamEvent::Complete { .. }) {
                    received_complete = true;
                }
            }
            assert!(received_complete);
        }
    }
}
