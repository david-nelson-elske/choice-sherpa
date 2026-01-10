//! SendMessageHandler - Send user message and update conversation state

use std::sync::Arc;

use crate::domain::ai_engine::conversation_state::MessageRole;
use crate::domain::ai_engine::{step_agent, ConversationState};
use crate::domain::foundation::{ComponentType, ConversationId, CycleId, DomainError, UserId};
use crate::ports::{
    AIError, AIProvider, CompletionRequest, Message as AIMessage, MessageRole as AIMessageRole,
    RequestMetadata, StateStorage, StateStorageError,
};

/// Command to send a message in a conversation
#[derive(Debug, Clone)]
pub struct SendMessageCommand {
    pub cycle_id: CycleId,
    pub message: String,
}

/// Result of sending a message
#[derive(Debug, Clone)]
pub struct SendMessageResult {
    pub updated_state: ConversationState,
    pub ai_response: String,
}

/// Error type for sending messages
#[derive(Debug, Clone)]
pub enum SendMessageError {
    /// Conversation not found
    NotFound(CycleId),
    /// Storage error
    Storage(String),
    /// Orchestrator error
    Orchestrator(String),
    /// Domain error
    Domain(DomainError),
    /// AI Provider error
    AIProvider(String),
}

impl std::fmt::Display for SendMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SendMessageError::NotFound(id) => {
                write!(f, "Conversation not found for cycle: {}", id)
            }
            SendMessageError::Storage(err) => write!(f, "Storage error: {}", err),
            SendMessageError::Orchestrator(err) => write!(f, "Orchestrator error: {}", err),
            SendMessageError::Domain(err) => write!(f, "{}", err),
            SendMessageError::AIProvider(err) => write!(f, "AI Provider error: {}", err),
        }
    }
}

impl std::error::Error for SendMessageError {}

impl From<DomainError> for SendMessageError {
    fn from(err: DomainError) -> Self {
        SendMessageError::Domain(err)
    }
}

impl From<StateStorageError> for SendMessageError {
    fn from(err: StateStorageError) -> Self {
        match err {
            StateStorageError::NotFound(cycle_id) => SendMessageError::NotFound(cycle_id),
            other => SendMessageError::Storage(other.to_string()),
        }
    }
}

impl From<AIError> for SendMessageError {
    fn from(err: AIError) -> Self {
        SendMessageError::AIProvider(err.to_string())
    }
}

/// Handler for sending messages in conversations
pub struct SendMessageHandler<P: ?Sized + AIProvider> {
    storage: Arc<dyn StateStorage>,
    ai_provider: Arc<P>,
}

impl<P: ?Sized + AIProvider> SendMessageHandler<P> {
    pub fn new(storage: Arc<dyn StateStorage>, ai_provider: Arc<P>) -> Self {
        Self {
            storage,
            ai_provider,
        }
    }

    pub async fn handle(
        &self,
        cmd: SendMessageCommand,
    ) -> Result<SendMessageResult, SendMessageError> {
        // 1. Load existing conversation state
        let mut state = self.storage.load_state(cmd.cycle_id).await?;

        // 2. Add user message to history
        state.add_message(MessageRole::User, cmd.message.clone());

        // 3. Generate AI response using real AI provider
        let ai_response = self.generate_ai_response(&state).await?;

        // 4. Add AI response to history
        state.add_message(MessageRole::Assistant, ai_response.clone());

        // 5. Persist updated state
        self.storage.save_state(cmd.cycle_id, &state).await?;

        Ok(SendMessageResult {
            updated_state: state,
            ai_response,
        })
    }

    /// Generate AI response using the AI provider
    async fn generate_ai_response(
        &self,
        state: &ConversationState,
    ) -> Result<String, AIError> {
        // Build system prompt from step agent spec
        let system_prompt = self.build_system_prompt(state.current_step);

        // Convert conversation history to AI messages
        let messages = self.convert_messages_to_ai_format(state);

        // Build request metadata
        let metadata = RequestMetadata::new(
            UserId::new("system").unwrap(), // TODO: Get actual user_id from context
            state.session_id,
            ConversationId::new(), // TODO: Map CycleId to ConversationId
            format!("cycle-{}", state.cycle_id),
        );

        // Build completion request
        let request = CompletionRequest::new(metadata)
            .with_system_prompt(system_prompt)
            .with_max_tokens(2000)
            .with_temperature(0.7)
            .with_component_type(state.current_step);

        // Add messages
        let mut request = request;
        for msg in messages {
            request = request.with_message(msg.role, msg.content);
        }

        // Call AI provider
        let response = self.ai_provider.complete(request).await?;

        Ok(response.content)
    }

    /// Build system prompt from step agent specification
    fn build_system_prompt(&self, component: ComponentType) -> String {
        let spec = step_agent::agents::get(component)
            .expect("All component types should have agent specs");

        format!(
            "You are a thoughtful decision professional helping users work through the {} phase of their decision-making process.\n\n\
            Role: {}\n\n\
            Objectives:\n{}\n\n\
            Techniques:\n{}\n\n\
            Guide the user through this phase with probing questions and thoughtful reflection. \
            Do not make decisions for them - help them think clearly about their situation.",
            spec.component.to_string().to_lowercase().replace('_', " "),
            spec.role,
            spec.objectives
                .iter()
                .map(|o| format!("- {}", o))
                .collect::<Vec<_>>()
                .join("\n"),
            spec.techniques
                .iter()
                .map(|t| format!("- {}", t))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// Convert conversation history to AI provider message format
    fn convert_messages_to_ai_format(&self, state: &ConversationState) -> Vec<AIMessage> {
        state
            .message_history
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    MessageRole::System => AIMessageRole::System,
                    MessageRole::User => AIMessageRole::User,
                    MessageRole::Assistant => AIMessageRole::Assistant,
                };
                AIMessage::new(role, msg.content.clone())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::{InMemoryStateStorage, MockAIProvider};
    use crate::domain::foundation::{ComponentType, SessionId};

    fn test_cycle_id() -> CycleId {
        CycleId::new()
    }

    fn test_session_id() -> SessionId {
        SessionId::new()
    }

    async fn setup_conversation(
        storage: Arc<InMemoryStateStorage>,
        cycle_id: CycleId,
    ) -> ConversationState {
        let state = ConversationState::new(cycle_id, test_session_id(), ComponentType::IssueRaising);
        storage.save_state(cycle_id, &state).await.unwrap();
        state
    }

    #[tokio::test]
    async fn test_send_message_adds_to_history() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let cycle_id = test_cycle_id();
        setup_conversation(storage.clone(), cycle_id).await;

        let mock_provider = Arc::new(
            MockAIProvider::new()
                .with_response("I can help you think through your software architecture decision.")
        );
        let handler = SendMessageHandler::new(storage.clone(), mock_provider);

        let cmd = SendMessageCommand {
            cycle_id,
            message: "I need to decide on a new software architecture".to_string(),
        };

        let result = handler.handle(cmd).await.unwrap();

        // Should have both user and assistant messages
        assert_eq!(result.updated_state.message_history.len(), 2);
        assert_eq!(result.updated_state.message_history[0].role, MessageRole::User);
        assert_eq!(
            result.updated_state.message_history[1].role,
            MessageRole::Assistant
        );
    }

    #[tokio::test]
    async fn test_send_message_persists_state() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let cycle_id = test_cycle_id();
        setup_conversation(storage.clone(), cycle_id).await;

        let mock_provider = Arc::new(MockAIProvider::new().with_response("Test response"));
        let handler = SendMessageHandler::new(storage.clone(), mock_provider);

        let cmd = SendMessageCommand {
            cycle_id,
            message: "Hello".to_string(),
        };

        handler.handle(cmd).await.unwrap();

        // Verify state was persisted
        let loaded = storage.load_state(cycle_id).await.unwrap();
        assert_eq!(loaded.message_history.len(), 2);
    }

    #[tokio::test]
    async fn test_send_message_fails_if_not_found() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let mock_provider = Arc::new(MockAIProvider::new().with_response("Test"));
        let handler = SendMessageHandler::new(storage, mock_provider);

        let cmd = SendMessageCommand {
            cycle_id: test_cycle_id(),
            message: "Hello".to_string(),
        };

        let result = handler.handle(cmd).await;

        assert!(matches!(result, Err(SendMessageError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_send_multiple_messages() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let cycle_id = test_cycle_id();
        setup_conversation(storage.clone(), cycle_id).await;

        let mock_provider = Arc::new(
            MockAIProvider::new()
                .with_response("First")
                .with_response("Second")
                .with_response("Third")
        );
        let handler = SendMessageHandler::new(storage, mock_provider);

        let messages = vec!["First message", "Second message", "Third message"];

        for (i, msg) in messages.iter().enumerate() {
            let cmd = SendMessageCommand {
                cycle_id,
                message: msg.to_string(),
            };

            let result = handler.handle(cmd).await.unwrap();

            // Each exchange adds 2 messages (user + assistant)
            assert_eq!(result.updated_state.message_history.len(), (i + 1) * 2);
        }
    }

    #[tokio::test]
    async fn test_send_message_returns_ai_response() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let cycle_id = test_cycle_id();
        setup_conversation(storage.clone(), cycle_id).await;

        let expected_response = "Let me help you with issue raising";
        let mock_provider = Arc::new(MockAIProvider::new().with_response(expected_response));
        let handler = SendMessageHandler::new(storage, mock_provider);

        let cmd = SendMessageCommand {
            cycle_id,
            message: "Test message".to_string(),
        };

        let result = handler.handle(cmd).await.unwrap();

        assert!(!result.ai_response.is_empty());
        assert_eq!(result.ai_response, expected_response);
    }
}
