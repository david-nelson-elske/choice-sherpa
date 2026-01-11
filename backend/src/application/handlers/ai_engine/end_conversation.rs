//! EndConversationHandler - Terminate active conversation and cleanup

use std::sync::Arc;

use crate::domain::foundation::{CycleId, DomainError};
use crate::ports::{StateStorage, StateStorageError};

/// Command to end a conversation
#[derive(Debug, Clone)]
pub struct EndConversationCommand {
    pub cycle_id: CycleId,
}

/// Error type for ending conversations
#[derive(Debug, Clone)]
pub enum EndConversationError {
    /// Conversation not found
    NotFound(CycleId),
    /// Storage error
    Storage(String),
    /// Domain error
    Domain(DomainError),
}

impl std::fmt::Display for EndConversationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EndConversationError::NotFound(id) => {
                write!(f, "Conversation not found for cycle: {}", id)
            }
            EndConversationError::Storage(err) => write!(f, "Storage error: {}", err),
            EndConversationError::Domain(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for EndConversationError {}

impl From<DomainError> for EndConversationError {
    fn from(err: DomainError) -> Self {
        EndConversationError::Domain(err)
    }
}

impl From<StateStorageError> for EndConversationError {
    fn from(err: StateStorageError) -> Self {
        match err {
            StateStorageError::NotFound(cycle_id) => EndConversationError::NotFound(cycle_id),
            other => EndConversationError::Storage(other.to_string()),
        }
    }
}

/// Handler for ending conversations
pub struct EndConversationHandler {
    storage: Arc<dyn StateStorage>,
}

impl EndConversationHandler {
    pub fn new(storage: Arc<dyn StateStorage>) -> Self {
        Self { storage }
    }

    pub async fn handle(&self, cmd: EndConversationCommand) -> Result<(), EndConversationError> {
        // 1. Verify conversation exists
        if !self.storage.exists(cmd.cycle_id).await? {
            return Err(EndConversationError::NotFound(cmd.cycle_id));
        }

        // 2. Delete conversation state and outputs
        self.storage.delete(cmd.cycle_id).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::InMemoryStateStorage;
    use crate::domain::ai_engine::ConversationState;
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
    async fn test_end_conversation_deletes_state() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let cycle_id = test_cycle_id();
        setup_conversation(storage.clone(), cycle_id).await;

        let handler = EndConversationHandler::new(storage.clone());

        // Verify exists before
        assert!(storage.exists(cycle_id).await.unwrap());

        // End conversation
        let cmd = EndConversationCommand { cycle_id };
        handler.handle(cmd).await.unwrap();

        // Verify deleted
        assert!(!storage.exists(cycle_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_end_conversation_fails_if_not_found() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let handler = EndConversationHandler::new(storage);

        let cmd = EndConversationCommand {
            cycle_id: test_cycle_id(),
        };

        let result = handler.handle(cmd).await;

        assert!(matches!(result, Err(EndConversationError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_end_conversation_idempotent() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let cycle_id = test_cycle_id();
        setup_conversation(storage.clone(), cycle_id).await;

        let handler = EndConversationHandler::new(storage);

        let cmd = EndConversationCommand { cycle_id };

        // First end should succeed
        handler.handle(cmd.clone()).await.unwrap();

        // Second end should fail (not found)
        let result = handler.handle(cmd).await;
        assert!(matches!(result, Err(EndConversationError::NotFound(_))));
    }
}
