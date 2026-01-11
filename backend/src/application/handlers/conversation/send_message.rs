//! SendMessage command handler.
//!
//! Handles sending user messages to a conversation and receiving AI responses.
//! Supports streaming responses via WebSocket.

use crate::domain::conversation::{
    AgentPhase, ConversationState, PhaseTransitionEngine,
};
use crate::domain::foundation::{
    ComponentId, ComponentType, ConversationId, CycleId, DomainError, SessionId, Timestamp, UserId,
};
use crate::ports::{
    AIError, AIProvider, CompletionRequest, Message, MessageRole as AIMessageRole, RequestMetadata,
    TokenUsage,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Unique identifier for a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageId(Uuid);

impl MessageId {
    /// Creates a new random MessageId.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a MessageId from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the inner UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Command to send a message in a conversation.
#[derive(Debug, Clone)]
pub struct SendMessageCommand {
    /// The user sending the message.
    pub user_id: UserId,
    /// The component this conversation is for.
    pub component_id: ComponentId,
    /// The message content.
    pub content: String,
}

impl SendMessageCommand {
    /// Creates a new send message command.
    pub fn new(user_id: UserId, component_id: ComponentId, content: impl Into<String>) -> Self {
        Self {
            user_id,
            component_id,
            content: content.into(),
        }
    }
}

/// Errors that can occur when sending a message.
#[derive(Debug, Clone, Error)]
pub enum SendMessageError {
    /// User is not authorized to access this component.
    #[error("Forbidden: user does not own this component")]
    Forbidden,

    /// Message content is empty or whitespace only.
    #[error("Validation error: message content cannot be empty")]
    EmptyContent,

    /// Conversation is in Complete state and cannot accept messages.
    #[error("Conversation is complete and cannot accept new messages")]
    ConversationComplete,

    /// Component was not found.
    #[error("Component not found: {0}")]
    ComponentNotFound(ComponentId),

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

impl From<DomainError> for SendMessageError {
    fn from(err: DomainError) -> Self {
        SendMessageError::DomainError(err.to_string())
    }
}

impl From<AIError> for SendMessageError {
    fn from(err: AIError) -> Self {
        SendMessageError::AIProviderError(err.to_string())
    }
}

/// Result of sending a message.
#[derive(Debug, Clone)]
pub struct SendMessageResult {
    /// ID of the user message that was stored.
    pub user_message_id: MessageId,
    /// ID of the assistant response message.
    pub assistant_message_id: MessageId,
    /// New conversation phase after processing.
    pub new_phase: AgentPhase,
    /// New conversation state after processing.
    pub new_state: ConversationState,
    /// Token usage for this exchange.
    pub usage: Option<TokenUsage>,
}

/// A stored message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    /// Unique ID of this message.
    pub id: MessageId,
    /// Role of the sender.
    pub role: MessageRole,
    /// Content of the message.
    pub content: String,
    /// When the message was created.
    pub created_at: Timestamp,
    /// Token count for this message (if available).
    pub token_count: Option<u32>,
}

/// Role of a message sender.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    /// System prompt message.
    System,
    /// User message.
    User,
    /// Assistant (AI) message.
    Assistant,
}

impl StoredMessage {
    /// Creates a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            id: MessageId::new(),
            role: MessageRole::User,
            content: content.into(),
            created_at: Timestamp::now(),
            token_count: None,
        }
    }

    /// Creates a new assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            id: MessageId::new(),
            role: MessageRole::Assistant,
            content: content.into(),
            created_at: Timestamp::now(),
            token_count: None,
        }
    }

    /// Creates an assistant message with a specific ID.
    pub fn assistant_with_id(id: MessageId, content: impl Into<String>) -> Self {
        Self {
            id,
            role: MessageRole::Assistant,
            content: content.into(),
            created_at: Timestamp::now(),
            token_count: None,
        }
    }

    /// Sets the token count for this message.
    pub fn with_token_count(mut self, count: u32) -> Self {
        self.token_count = Some(count);
        self
    }

    /// Converts to an AI provider message.
    pub fn to_ai_message(&self) -> Message {
        let role = match self.role {
            MessageRole::System => AIMessageRole::System,
            MessageRole::User => AIMessageRole::User,
            MessageRole::Assistant => AIMessageRole::Assistant,
        };
        Message::new(role, &self.content)
    }
}

/// Port for verifying component ownership through the session chain.
///
/// The ownership chain is: component -> cycle -> session -> user
#[async_trait]
pub trait ComponentOwnershipChecker: Send + Sync {
    /// Checks if a user owns a component through the session chain.
    ///
    /// Returns `Ok(OwnershipInfo)` if the user owns the component,
    /// or `Err(DomainError::Forbidden)` if not.
    async fn check_ownership(
        &self,
        user_id: &UserId,
        component_id: &ComponentId,
    ) -> Result<OwnershipInfo, DomainError>;
}

/// Information about component ownership.
#[derive(Debug, Clone)]
pub struct OwnershipInfo {
    /// The session that owns this component.
    pub session_id: SessionId,
    /// The cycle that contains this component.
    pub cycle_id: CycleId,
    /// The type of component.
    pub component_type: ComponentType,
}

/// Port for conversation persistence.
#[async_trait]
pub trait ConversationRepository: Send + Sync {
    /// Finds a conversation by component ID.
    async fn find_by_component(
        &self,
        component_id: &ComponentId,
    ) -> Result<Option<ConversationRecord>, DomainError>;

    /// Creates a new conversation for a component.
    async fn create(
        &self,
        component_id: &ComponentId,
        component_type: ComponentType,
        user_id: &UserId,
        system_prompt: &str,
    ) -> Result<ConversationRecord, DomainError>;

    /// Saves a conversation.
    async fn save(&self, conversation: &ConversationRecord) -> Result<(), DomainError>;

    /// Adds a message to a conversation.
    async fn add_message(
        &self,
        conversation_id: &ConversationId,
        message: StoredMessage,
    ) -> Result<(), DomainError>;

    /// Updates conversation state and phase.
    async fn update_state(
        &self,
        conversation_id: &ConversationId,
        state: ConversationState,
        phase: AgentPhase,
    ) -> Result<(), DomainError>;

    /// Finds a conversation by its ID.
    async fn find_by_id(
        &self,
        conversation_id: &ConversationId,
    ) -> Result<Option<ConversationRecord>, DomainError>;

    /// Gets paginated messages for a conversation.
    ///
    /// Returns messages ordered by creation time (oldest first).
    async fn get_messages(
        &self,
        conversation_id: &ConversationId,
        offset: u32,
        limit: u32,
    ) -> Result<(Vec<StoredMessage>, u32), DomainError>;
}

/// A conversation record from the repository.
#[derive(Debug, Clone)]
pub struct ConversationRecord {
    /// Unique ID for the conversation.
    pub id: ConversationId,
    /// The component this conversation is for.
    pub component_id: ComponentId,
    /// Type of component.
    pub component_type: ComponentType,
    /// Current state.
    pub state: ConversationState,
    /// Current phase.
    pub phase: AgentPhase,
    /// Messages in the conversation.
    pub messages: Vec<StoredMessage>,
    /// The user who owns this conversation.
    pub user_id: UserId,
    /// System prompt for this conversation.
    pub system_prompt: String,
    /// When the conversation was created.
    pub created_at: Timestamp,
    /// When the conversation was last updated.
    pub updated_at: Timestamp,
}

impl ConversationRecord {
    /// Returns the count of user messages.
    pub fn user_message_count(&self) -> usize {
        self.messages.iter().filter(|m| m.role == MessageRole::User).count()
    }

    /// Returns the last message if any.
    pub fn last_message(&self) -> Option<&StoredMessage> {
        self.messages.last()
    }

    /// Returns true if the conversation has any messages.
    pub fn has_messages(&self) -> bool {
        !self.messages.is_empty()
    }

    /// Converts messages to AI provider format.
    pub fn messages_for_ai(&self) -> Vec<Message> {
        self.messages.iter().map(|m| m.to_ai_message()).collect()
    }
}

/// Stream event for real-time updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// A chunk of the AI response.
    Chunk {
        message_id: MessageId,
        delta: String,
    },
    /// The message is complete.
    Complete {
        message_id: MessageId,
        full_content: String,
        usage: Option<TokenUsage>,
    },
    /// An error occurred.
    Error {
        message_id: MessageId,
        error: String,
    },
}

/// Handler for SendMessage commands.
pub struct SendMessageHandler<O, R, A>
where
    O: ComponentOwnershipChecker,
    R: ConversationRepository,
    A: AIProvider,
{
    ownership_checker: Arc<O>,
    conversation_repo: Arc<R>,
    ai_provider: Arc<A>,
}

impl<O, R, A> SendMessageHandler<O, R, A>
where
    O: ComponentOwnershipChecker + 'static,
    R: ConversationRepository + 'static,
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

    /// Handles a send message command.
    ///
    /// Returns a channel receiver for streaming events plus the final result.
    pub async fn handle(
        &self,
        cmd: SendMessageCommand,
    ) -> Result<(mpsc::Receiver<StreamEvent>, SendMessageResult), SendMessageError> {
        // R3: Validate content is not empty
        let content = cmd.content.trim();
        if content.is_empty() {
            return Err(SendMessageError::EmptyContent);
        }

        // R1: Verify ownership through session chain
        let ownership = self
            .ownership_checker
            .check_ownership(&cmd.user_id, &cmd.component_id)
            .await
            .map_err(|_| SendMessageError::Forbidden)?;

        // R2: Get or create conversation
        let mut conversation = match self
            .conversation_repo
            .find_by_component(&cmd.component_id)
            .await?
        {
            Some(conv) => conv,
            None => {
                // Create new conversation
                let system_prompt = crate::domain::conversation::opening_message_for_component(
                    ownership.component_type,
                );
                self.conversation_repo
                    .create(
                        &cmd.component_id,
                        ownership.component_type,
                        &cmd.user_id,
                        system_prompt,
                    )
                    .await?
            }
        };

        // R9: Check conversation state
        if conversation.state == ConversationState::Complete {
            return Err(SendMessageError::ConversationComplete);
        }

        // R4: Create and persist user message
        let user_message = StoredMessage::user(content);
        let user_message_id = user_message.id;
        self.conversation_repo
            .add_message(&conversation.id, user_message.clone())
            .await?;
        conversation.messages.push(user_message);

        // R5: Build context and call AI provider
        let assistant_message_id = MessageId::new();
        let (tx, rx) = mpsc::channel(32);

        // Build request
        let request = CompletionRequest::new(RequestMetadata::new(
            cmd.user_id.clone(),
            ownership.session_id,
            conversation.id,
            format!("msg-{}", assistant_message_id),
        ))
        .with_system_prompt(&conversation.system_prompt)
        .with_component_type(ownership.component_type);

        // Add messages
        let mut request = request;
        for msg in conversation.messages_for_ai() {
            request = request.with_message(msg.role, &msg.content);
        }

        // R16: Stream the response
        let stream = self.ai_provider.stream_complete(request).await?;

        // Spawn task to handle streaming
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

                        // R16: Send chunk event
                        let _ = tx
                            .send(StreamEvent::Chunk {
                                message_id: assistant_message_id,
                                delta,
                            })
                            .await;

                        // R17: Check for completion
                        if is_final {
                            final_usage = usage;
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        // R18: Send error event
                        let _ = tx
                            .send(StreamEvent::Error {
                                message_id: assistant_message_id,
                                error: e.to_string(),
                            })
                            .await;
                        return Err(SendMessageError::AIProviderError(e.to_string()));
                    }
                    None => break,
                }
            }

            // R6 & R7: Store assistant message with token count
            let mut assistant_msg = StoredMessage::assistant_with_id(assistant_message_id, &full_content);
            if let Some(ref usage) = final_usage {
                assistant_msg = assistant_msg.with_token_count(usage.completion_tokens);
            }
            conversation_repo
                .add_message(&conversation_id, assistant_msg)
                .await?;

            // R17: Send complete event
            let _ = tx
                .send(StreamEvent::Complete {
                    message_id: assistant_message_id,
                    full_content: full_content.clone(),
                    usage: final_usage.clone(),
                })
                .await;

            Ok((full_content, final_usage))
        });

        // Wait for streaming to complete
        let (_full_content, usage) = handle
            .await
            .map_err(|e| SendMessageError::DomainError(e.to_string()))??;

        // R8: Update state if first message
        let new_state = if conversation.state == ConversationState::Ready {
            ConversationState::InProgress
        } else {
            conversation.state
        };

        // Determine new phase using transition engine
        let engine = PhaseTransitionEngine::for_component(ownership.component_type);
        let snapshot = crate::domain::conversation::ConversationSnapshot::new(
            conversation.user_message_count() + 1, // Include the message we just added
            Some(content.to_string()),
            ownership.component_type,
        );
        let new_phase = engine.next_phase(conversation.phase, &snapshot);

        // Update conversation state
        self.conversation_repo
            .update_state(&conversation.id, new_state, new_phase)
            .await?;

        Ok((
            rx,
            SendMessageResult {
                user_message_id,
                assistant_message_id,
                new_phase,
                new_state,
                usage,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::ErrorCode;
    use crate::ports::StreamChunk as AIStreamChunk;
    use std::sync::Mutex;
    use futures::stream;

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
                Err(DomainError::new(ErrorCode::Forbidden, "User does not own component"))
            }
        }
    }

    struct MockConversationRepo {
        conversations: Mutex<Vec<ConversationRecord>>,
        messages: Mutex<Vec<(ConversationId, StoredMessage)>>,
    }

    impl MockConversationRepo {
        fn new() -> Self {
            Self {
                conversations: Mutex::new(Vec::new()),
                messages: Mutex::new(Vec::new()),
            }
        }

        fn with_conversation(conversation: ConversationRecord) -> Self {
            Self {
                conversations: Mutex::new(vec![conversation]),
                messages: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl ConversationRepository for MockConversationRepo {
        async fn find_by_component(
            &self,
            component_id: &ComponentId,
        ) -> Result<Option<ConversationRecord>, DomainError> {
            let convs = self.conversations.lock().unwrap();
            Ok(convs.iter().find(|c| c.component_id == *component_id).cloned())
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
            self.messages.lock().unwrap().push((*conversation_id, message));
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
            if let Some(conv) = convs.iter().find(|c| c.id == *conversation_id) {
                let total = conv.messages.len() as u32;
                let messages: Vec<_> = conv.messages
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
        ) -> Result<std::pin::Pin<Box<dyn futures::Stream<Item = Result<AIStreamChunk, AIError>> + Send>>, AIError>
        {
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

    mod r1_ownership_verification {
        use super::*;

        #[tokio::test]
        async fn rejects_when_user_does_not_own_component() {
            // Given: A user who doesn't own the component
            let ownership_checker = Arc::new(MockOwnershipChecker::denying());
            let conversation_repo = Arc::new(MockConversationRepo::new());
            let ai_provider = Arc::new(MockAIProvider::with_response("Hello"));

            let handler = SendMessageHandler::new(
                ownership_checker,
                conversation_repo,
                ai_provider,
            );

            let cmd = SendMessageCommand::new(
                UserId::new("non-owner").unwrap(),
                ComponentId::new(),
                "Hello",
            );

            // When: Sending a message
            let result = handler.handle(cmd).await;

            // Then: Rejected with Forbidden error
            assert!(matches!(result, Err(SendMessageError::Forbidden)));
        }

        #[tokio::test]
        async fn allows_when_user_owns_component() {
            // Given: A user who owns the component
            let ownership_checker = Arc::new(MockOwnershipChecker::allowing());
            let conversation_repo = Arc::new(MockConversationRepo::new());
            let ai_provider = Arc::new(MockAIProvider::with_response("Hello there!"));

            let handler = SendMessageHandler::new(
                ownership_checker,
                conversation_repo,
                ai_provider,
            );

            let cmd = SendMessageCommand::new(
                UserId::new("owner").unwrap(),
                ComponentId::new(),
                "Hello",
            );

            // When: Sending a message
            let result = handler.handle(cmd).await;

            // Then: Allowed
            assert!(result.is_ok());
        }
    }

    mod r3_empty_content_validation {
        use super::*;

        #[tokio::test]
        async fn rejects_empty_content() {
            let handler = SendMessageHandler::new(
                Arc::new(MockOwnershipChecker::allowing()),
                Arc::new(MockConversationRepo::new()),
                Arc::new(MockAIProvider::with_response("Hi")),
            );

            let cmd = SendMessageCommand::new(
                UserId::new("user").unwrap(),
                ComponentId::new(),
                "",
            );

            let result = handler.handle(cmd).await;
            assert!(matches!(result, Err(SendMessageError::EmptyContent)));
        }

        #[tokio::test]
        async fn rejects_whitespace_only_content() {
            let handler = SendMessageHandler::new(
                Arc::new(MockOwnershipChecker::allowing()),
                Arc::new(MockConversationRepo::new()),
                Arc::new(MockAIProvider::with_response("Hi")),
            );

            let cmd = SendMessageCommand::new(
                UserId::new("user").unwrap(),
                ComponentId::new(),
                "   \n\t   ",
            );

            let result = handler.handle(cmd).await;
            assert!(matches!(result, Err(SendMessageError::EmptyContent)));
        }
    }

    mod r9_conversation_state_check {
        use super::*;

        #[tokio::test]
        async fn rejects_message_when_conversation_complete() {
            // Given: A conversation in Complete state
            let component_id = ComponentId::new();
            let conversation = ConversationRecord {
                id: ConversationId::new(),
                component_id,
                component_type: ComponentType::IssueRaising,
                state: ConversationState::Complete,
                phase: AgentPhase::Confirm, // Use Confirm as the last phase before Complete state
                messages: Vec::new(),
                user_id: UserId::new("user").unwrap(),
                system_prompt: "Test".to_string(),
                created_at: Timestamp::now(),
                updated_at: Timestamp::now(),
            };

            let handler = SendMessageHandler::new(
                Arc::new(MockOwnershipChecker::allowing()),
                Arc::new(MockConversationRepo::with_conversation(conversation)),
                Arc::new(MockAIProvider::with_response("Hi")),
            );

            let cmd = SendMessageCommand::new(
                UserId::new("user").unwrap(),
                component_id,
                "Hello",
            );

            // When: Sending a message
            let result = handler.handle(cmd).await;

            // Then: Rejected with ConversationComplete error
            assert!(matches!(result, Err(SendMessageError::ConversationComplete)));
        }
    }
}
