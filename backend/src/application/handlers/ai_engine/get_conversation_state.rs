//! GetConversationStateHandler - Query current conversation state

use std::sync::Arc;

use crate::domain::ai_engine::ConversationState;
use crate::domain::foundation::{CycleId, DomainError};
use crate::ports::{StateStorage, StateStorageError};

/// Query to get conversation state
#[derive(Debug, Clone)]
pub struct GetConversationStateQuery {
    pub cycle_id: CycleId,
}

/// Result of getting conversation state
#[derive(Debug, Clone)]
pub struct GetConversationStateResult {
    pub state: ConversationState,
}

/// Error type for getting conversation state
#[derive(Debug, Clone)]
pub enum GetConversationStateError {
    /// Conversation not found
    NotFound(CycleId),
    /// Storage error
    Storage(String),
    /// Domain error
    Domain(DomainError),
}

impl std::fmt::Display for GetConversationStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetConversationStateError::NotFound(id) => {
                write!(f, "Conversation not found for cycle: {}", id)
            }
            GetConversationStateError::Storage(err) => write!(f, "Storage error: {}", err),
            GetConversationStateError::Domain(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for GetConversationStateError {}

impl From<DomainError> for GetConversationStateError {
    fn from(err: DomainError) -> Self {
        GetConversationStateError::Domain(err)
    }
}

impl From<StateStorageError> for GetConversationStateError {
    fn from(err: StateStorageError) -> Self {
        match err {
            StateStorageError::NotFound(cycle_id) => GetConversationStateError::NotFound(cycle_id),
            other => GetConversationStateError::Storage(other.to_string()),
        }
    }
}

/// Handler for getting conversation state
pub struct GetConversationStateHandler {
    storage: Arc<dyn StateStorage>,
}

impl GetConversationStateHandler {
    pub fn new(storage: Arc<dyn StateStorage>) -> Self {
        Self { storage }
    }

    pub async fn handle(
        &self,
        query: GetConversationStateQuery,
    ) -> Result<GetConversationStateResult, GetConversationStateError> {
        // Load conversation state
        let state = self.storage.load_state(query.cycle_id).await?;

        Ok(GetConversationStateResult { state })
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
    async fn test_get_conversation_state_returns_state() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let cycle_id = test_cycle_id();
        let original_state = setup_conversation(storage.clone(), cycle_id).await;

        let handler = GetConversationStateHandler::new(storage);

        let query = GetConversationStateQuery { cycle_id };
        let result = handler.handle(query).await.unwrap();

        assert_eq!(result.state.cycle_id, original_state.cycle_id);
        assert_eq!(result.state.current_step, original_state.current_step);
        assert_eq!(result.state.status, original_state.status);
    }

    #[tokio::test]
    async fn test_get_conversation_state_fails_if_not_found() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let handler = GetConversationStateHandler::new(storage);

        let query = GetConversationStateQuery {
            cycle_id: test_cycle_id(),
        };

        let result = handler.handle(query).await;

        assert!(matches!(
            result,
            Err(GetConversationStateError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn test_get_conversation_state_with_messages() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let cycle_id = test_cycle_id();
        let mut state = setup_conversation(storage.clone(), cycle_id).await;

        // Add some messages
        use crate::domain::ai_engine::conversation_state::MessageRole;
        state.add_message(MessageRole::User, "Hello".to_string());
        state.add_message(MessageRole::Assistant, "Hi there!".to_string());

        storage.save_state(cycle_id, &state).await.unwrap();

        let handler = GetConversationStateHandler::new(storage);

        let query = GetConversationStateQuery { cycle_id };
        let result = handler.handle(query).await.unwrap();

        assert_eq!(result.state.message_history.len(), 2);
    }
}
