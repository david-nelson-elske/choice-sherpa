//! RouteIntentHandler - Determine target component from user intent

use std::sync::Arc;

use crate::domain::ai_engine::{Orchestrator, UserIntent};
use crate::domain::foundation::{ComponentType, CycleId, DomainError};
use crate::ports::{StateStorage, StateStorageError};

/// Command to route user intent
#[derive(Debug, Clone)]
pub struct RouteIntentCommand {
    pub cycle_id: CycleId,
    pub intent: UserIntent,
}

/// Result of routing intent
#[derive(Debug, Clone)]
pub struct RouteIntentResult {
    pub target_component: ComponentType,
}

/// Error type for routing intent
#[derive(Debug, Clone)]
pub enum RouteIntentError {
    /// Conversation not found
    NotFound(CycleId),
    /// Storage error
    Storage(String),
    /// Orchestrator error (invalid transition, etc.)
    Orchestrator(String),
    /// Domain error
    Domain(DomainError),
}

impl std::fmt::Display for RouteIntentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouteIntentError::NotFound(id) => {
                write!(f, "Conversation not found for cycle: {}", id)
            }
            RouteIntentError::Storage(err) => write!(f, "Storage error: {}", err),
            RouteIntentError::Orchestrator(err) => write!(f, "Orchestrator error: {}", err),
            RouteIntentError::Domain(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for RouteIntentError {}

impl From<DomainError> for RouteIntentError {
    fn from(err: DomainError) -> Self {
        RouteIntentError::Domain(err)
    }
}

impl From<StateStorageError> for RouteIntentError {
    fn from(err: StateStorageError) -> Self {
        match err {
            StateStorageError::NotFound(cycle_id) => RouteIntentError::NotFound(cycle_id),
            other => RouteIntentError::Storage(other.to_string()),
        }
    }
}

/// Handler for routing user intents
pub struct RouteIntentHandler {
    storage: Arc<dyn StateStorage>,
}

impl RouteIntentHandler {
    pub fn new(storage: Arc<dyn StateStorage>) -> Self {
        Self { storage }
    }

    pub async fn handle(
        &self,
        cmd: RouteIntentCommand,
    ) -> Result<RouteIntentResult, RouteIntentError> {
        // 1. Load conversation state
        let state = self.storage.load_state(cmd.cycle_id).await?;

        // 2. Create orchestrator from state
        let orchestrator = Orchestrator::from_state(state)
            .map_err(|e| RouteIntentError::Orchestrator(e.to_string()))?;

        // 3. Route the intent
        let target_component = orchestrator
            .route(cmd.intent)
            .map_err(|e| RouteIntentError::Orchestrator(e.to_string()))?;

        Ok(RouteIntentResult { target_component })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::InMemoryStateStorage;
    use crate::domain::ai_engine::ConversationState;
    use crate::domain::foundation::SessionId;

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
    async fn test_route_intent_continue() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let cycle_id = test_cycle_id();
        setup_conversation(storage.clone(), cycle_id).await;

        let handler = RouteIntentHandler::new(storage);

        let cmd = RouteIntentCommand {
            cycle_id,
            intent: UserIntent::Continue,
        };

        let result = handler.handle(cmd).await.unwrap();

        // Continue should stay on current step
        assert_eq!(result.target_component, ComponentType::IssueRaising);
    }

    #[tokio::test]
    async fn test_route_intent_navigate() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let cycle_id = test_cycle_id();
        let mut state = setup_conversation(storage.clone(), cycle_id).await;

        // Complete first step
        state.complete_current_step("Done".to_string(), vec![]);
        storage.save_state(cycle_id, &state).await.unwrap();

        let handler = RouteIntentHandler::new(storage);

        let cmd = RouteIntentCommand {
            cycle_id,
            intent: UserIntent::Navigate(ComponentType::ProblemFrame),
        };

        let result = handler.handle(cmd).await.unwrap();

        assert_eq!(result.target_component, ComponentType::ProblemFrame);
    }

    #[tokio::test]
    async fn test_route_intent_invalid_transition_fails() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let cycle_id = test_cycle_id();
        setup_conversation(storage.clone(), cycle_id).await;

        let handler = RouteIntentHandler::new(storage);

        // Try to skip ahead without completing previous steps
        let cmd = RouteIntentCommand {
            cycle_id,
            intent: UserIntent::Navigate(ComponentType::Consequences),
        };

        let result = handler.handle(cmd).await;

        assert!(matches!(result, Err(RouteIntentError::Orchestrator(_))));
    }

    #[tokio::test]
    async fn test_route_intent_not_found() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let handler = RouteIntentHandler::new(storage);

        let cmd = RouteIntentCommand {
            cycle_id: test_cycle_id(),
            intent: UserIntent::Continue,
        };

        let result = handler.handle(cmd).await;

        assert!(matches!(result, Err(RouteIntentError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_route_intent_complete_advances() {
        let storage = Arc::new(InMemoryStateStorage::new());
        let cycle_id = test_cycle_id();
        let mut state = setup_conversation(storage.clone(), cycle_id).await;

        // Complete the current step
        state.complete_current_step("Done".to_string(), vec![]);
        storage.save_state(cycle_id, &state).await.unwrap();

        let handler = RouteIntentHandler::new(storage);

        let cmd = RouteIntentCommand {
            cycle_id,
            intent: UserIntent::Complete,
        };

        let result = handler.handle(cmd).await.unwrap();

        // Should advance to next step
        assert_eq!(result.target_component, ComponentType::ProblemFrame);
    }
}
