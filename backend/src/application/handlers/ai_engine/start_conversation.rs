//! StartConversationHandler - Initialize AI conversation for a cycle

use std::sync::Arc;

use crate::domain::ai_engine::ConversationState;
use crate::domain::foundation::{ComponentType, CycleId, DomainError, SessionId};
use crate::ports::{StateStorage, StateStorageError};

/// Command to start an AI conversation
#[derive(Debug, Clone)]
pub struct StartConversationCommand {
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    pub initial_component: ComponentType,
}

/// Result of starting a conversation
#[derive(Debug, Clone)]
pub struct StartConversationResult {
    pub state: ConversationState,
}

/// Error type for starting conversations
#[derive(Debug, Clone)]
pub enum StartConversationError {
    /// Conversation already exists for this cycle
    AlreadyExists(CycleId),
    /// Storage error
    Storage(String),
    /// Domain error
    Domain(DomainError),
}

impl std::fmt::Display for StartConversationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartConversationError::AlreadyExists(id) => {
                write!(f, "Conversation already exists for cycle: {}", id)
            }
            StartConversationError::Storage(err) => write!(f, "Storage error: {}", err),
            StartConversationError::Domain(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for StartConversationError {}

impl From<DomainError> for StartConversationError {
    fn from(err: DomainError) -> Self {
        StartConversationError::Domain(err)
    }
}

impl From<StateStorageError> for StartConversationError {
    fn from(err: StateStorageError) -> Self {
        StartConversationError::Storage(err.to_string())
    }
}

/// Handler for starting AI conversations
pub struct StartConversationHandler {
    storage: Arc<dyn StateStorage>,
}

impl StartConversationHandler {
    pub fn new(storage: Arc<dyn StateStorage>) -> Self {
        Self { storage }
    }

    pub async fn handle(
        &self,
        cmd: StartConversationCommand,
    ) -> Result<StartConversationResult, StartConversationError> {
        // 1. Check if conversation already exists
        if self.storage.exists(cmd.cycle_id).await? {
            return Err(StartConversationError::AlreadyExists(cmd.cycle_id));
        }

        // 2. Create new conversation state
        let state = ConversationState::new(cmd.cycle_id, cmd.session_id, cmd.initial_component);

        // 3. Persist state
        self.storage.save_state(cmd.cycle_id, &state).await?;

        Ok(StartConversationResult { state })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::InMemoryStateStorage;

    fn test_cycle_id() -> CycleId {
        CycleId::new()
    }

    fn test_session_id() -> SessionId {
        SessionId::new()
    }

    #[tokio::test]
    async fn test_start_conversation_creates_new_state() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let handler = StartConversationHandler::new(storage.clone());

        let cmd = StartConversationCommand {
            cycle_id: test_cycle_id(),
            session_id: test_session_id(),
            initial_component: ComponentType::IssueRaising,
        };

        let result = handler.handle(cmd.clone()).await.unwrap();

        assert_eq!(result.state.cycle_id, cmd.cycle_id);
        assert_eq!(result.state.session_id, cmd.session_id);
        assert_eq!(result.state.current_step, ComponentType::IssueRaising);

        // Verify state was persisted
        assert!(storage.exists(cmd.cycle_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_start_conversation_fails_if_exists() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let handler = StartConversationHandler::new(storage.clone());

        let cycle_id = test_cycle_id();
        let cmd = StartConversationCommand {
            cycle_id,
            session_id: test_session_id(),
            initial_component: ComponentType::IssueRaising,
        };

        // Start first conversation
        handler.handle(cmd.clone()).await.unwrap();

        // Try to start again
        let result = handler.handle(cmd).await;

        assert!(matches!(
            result,
            Err(StartConversationError::AlreadyExists(_))
        ));
    }

    #[tokio::test]
    async fn test_start_conversation_with_different_initial_components() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let handler = StartConversationHandler::new(storage);

        let components = vec![
            ComponentType::IssueRaising,
            ComponentType::ProblemFrame,
            ComponentType::Objectives,
        ];

        for component in components {
            let cmd = StartConversationCommand {
                cycle_id: test_cycle_id(),
                session_id: test_session_id(),
                initial_component: component,
            };

            let result = handler.handle(cmd).await.unwrap();
            assert_eq!(result.state.current_step, component);
        }
    }
}
