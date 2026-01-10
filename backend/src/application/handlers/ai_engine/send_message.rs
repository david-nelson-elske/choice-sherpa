//! SendMessageHandler - Send user message and update conversation state

use std::sync::Arc;

use crate::domain::ai_engine::conversation_state::MessageRole;
use crate::domain::ai_engine::{ConversationState, Orchestrator};
use crate::domain::foundation::{CycleId, DomainError};
use crate::ports::{StateStorage, StateStorageError};

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

/// Handler for sending messages in conversations
pub struct SendMessageHandler {
    storage: Arc<dyn StateStorage>,
}

impl SendMessageHandler {
    pub fn new(storage: Arc<dyn StateStorage>) -> Self {
        Self { storage }
    }

    pub async fn handle(
        &self,
        cmd: SendMessageCommand,
    ) -> Result<SendMessageResult, SendMessageError> {
        // 1. Load existing conversation state
        let mut state = self.storage.load_state(cmd.cycle_id).await?;

        // 2. Add user message to history
        state.add_message(MessageRole::User, cmd.message.clone());

        // 3. Generate AI response (placeholder - will integrate with AI provider later)
        let ai_response = self.generate_ai_response(&state, &cmd.message).await;

        // 4. Add AI response to history
        state.add_message(MessageRole::Assistant, ai_response.clone());

        // 5. Persist updated state
        self.storage.save_state(cmd.cycle_id, &state).await?;

        Ok(SendMessageResult {
            updated_state: state,
            ai_response,
        })
    }

    /// Generate AI response (placeholder - will be replaced with actual AI integration)
    async fn generate_ai_response(&self, state: &ConversationState, _message: &str) -> String {
        // TODO: Integrate with actual AI provider (OpenAI/Anthropic)
        // For now, return a simple acknowledgment
        format!(
            "I understand you're working on {:?}. Let me help you with that.",
            state.current_step
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::InMemoryStateStorage;
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

        let handler = SendMessageHandler::new(storage.clone());

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

        let handler = SendMessageHandler::new(storage.clone());

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
        let handler = SendMessageHandler::new(storage);

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

        let handler = SendMessageHandler::new(storage);

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

        let handler = SendMessageHandler::new(storage);

        let cmd = SendMessageCommand {
            cycle_id,
            message: "Test message".to_string(),
        };

        let result = handler.handle(cmd).await.unwrap();

        assert!(!result.ai_response.is_empty());
        assert!(result.ai_response.contains("IssueRaising"));
    }
}
