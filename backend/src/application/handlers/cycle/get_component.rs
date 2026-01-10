//! GetComponentHandler - Query handler for retrieving a component's details.
//!
//! Returns the full component data including status and output.
//! Uses CycleRepository to access the aggregate for full output data.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::domain::foundation::{ComponentStatus, ComponentType, CycleId, DomainError, ErrorCode};
use crate::ports::CycleRepository;

/// Query to get a component from a cycle.
#[derive(Debug, Clone)]
pub struct GetComponentQuery {
    /// The cycle containing the component.
    pub cycle_id: CycleId,
    /// The component type to retrieve.
    pub component_type: ComponentType,
}

/// Result of successful component query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetComponentResult {
    /// The cycle ID.
    pub cycle_id: CycleId,
    /// The component type.
    pub component_type: ComponentType,
    /// The component status.
    pub status: ComponentStatus,
    /// The component output as JSON.
    pub output: serde_json::Value,
}

/// Error type for getting a component.
#[derive(Debug, Clone)]
pub enum GetComponentError {
    /// Cycle not found.
    CycleNotFound(CycleId),
    /// Component not found.
    ComponentNotFound(CycleId, ComponentType),
    /// Infrastructure error.
    Infrastructure(String),
}

impl std::fmt::Display for GetComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetComponentError::CycleNotFound(id) => write!(f, "Cycle not found: {}", id),
            GetComponentError::ComponentNotFound(cycle_id, ct) => {
                write!(f, "Component {:?} not found in cycle: {}", ct, cycle_id)
            }
            GetComponentError::Infrastructure(msg) => write!(f, "Infrastructure error: {}", msg),
        }
    }
}

impl std::error::Error for GetComponentError {}

impl From<DomainError> for GetComponentError {
    fn from(err: DomainError) -> Self {
        match err.code {
            ErrorCode::CycleNotFound => GetComponentError::CycleNotFound(CycleId::new()),
            ErrorCode::ComponentNotFound => {
                GetComponentError::ComponentNotFound(CycleId::new(), ComponentType::IssueRaising)
            }
            _ => GetComponentError::Infrastructure(err.message),
        }
    }
}

/// Handler for retrieving component details.
///
/// Uses CycleRepository to access the aggregate for full output data.
/// This is a pragmatic approach - a dedicated ComponentReader port
/// could be added for performance optimization in the future.
pub struct GetComponentHandler {
    cycle_repository: Arc<dyn CycleRepository>,
}

impl GetComponentHandler {
    pub fn new(cycle_repository: Arc<dyn CycleRepository>) -> Self {
        Self { cycle_repository }
    }

    pub async fn handle(
        &self,
        query: GetComponentQuery,
    ) -> Result<GetComponentResult, GetComponentError> {
        // Get the cycle
        let cycle = self
            .cycle_repository
            .find_by_id(&query.cycle_id)
            .await?
            .ok_or(GetComponentError::CycleNotFound(query.cycle_id))?;

        // Get the component
        let component = cycle.component(query.component_type).ok_or(
            GetComponentError::ComponentNotFound(query.cycle_id, query.component_type),
        )?;

        Ok(GetComponentResult {
            cycle_id: query.cycle_id,
            component_type: query.component_type,
            status: component.status(),
            output: component.output_as_value(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cycle::Cycle;
    use crate::domain::foundation::SessionId;
    use async_trait::async_trait;
    use std::sync::Mutex;

    // ─────────────────────────────────────────────────────────────────────
    // Mock Implementation
    // ─────────────────────────────────────────────────────────────────────

    struct MockCycleRepository {
        cycles: Mutex<Vec<Cycle>>,
        fail_read: bool,
    }

    impl MockCycleRepository {
        fn new() -> Self {
            Self {
                cycles: Mutex::new(Vec::new()),
                fail_read: false,
            }
        }

        fn with_cycle(cycle: Cycle) -> Self {
            Self {
                cycles: Mutex::new(vec![cycle]),
                fail_read: false,
            }
        }

        fn failing() -> Self {
            Self {
                cycles: Mutex::new(Vec::new()),
                fail_read: true,
            }
        }
    }

    #[async_trait]
    impl CycleRepository for MockCycleRepository {
        async fn save(&self, cycle: &Cycle) -> Result<(), DomainError> {
            self.cycles.lock().unwrap().push(cycle.clone());
            Ok(())
        }

        async fn update(&self, _cycle: &Cycle) -> Result<(), DomainError> {
            Ok(())
        }

        async fn find_by_id(&self, id: &CycleId) -> Result<Option<Cycle>, DomainError> {
            if self.fail_read {
                return Err(DomainError::new(
                    ErrorCode::DatabaseError,
                    "Simulated read failure",
                ));
            }
            Ok(self
                .cycles
                .lock()
                .unwrap()
                .iter()
                .find(|c| c.id() == *id)
                .cloned())
        }

        async fn exists(&self, _id: &CycleId) -> Result<bool, DomainError> {
            Ok(false)
        }

        async fn find_by_session_id(
            &self,
            _session_id: &SessionId,
        ) -> Result<Vec<Cycle>, DomainError> {
            Ok(vec![])
        }

        async fn find_primary_by_session_id(
            &self,
            _session_id: &SessionId,
        ) -> Result<Option<Cycle>, DomainError> {
            Ok(None)
        }

        async fn find_branches(&self, _parent_id: &CycleId) -> Result<Vec<Cycle>, DomainError> {
            Ok(vec![])
        }

        async fn count_by_session_id(&self, _session_id: &SessionId) -> Result<u32, DomainError> {
            Ok(0)
        }

        async fn delete(&self, _id: &CycleId) -> Result<(), DomainError> {
            Ok(())
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Test Helpers
    // ─────────────────────────────────────────────────────────────────────

    fn create_cycle_with_started_component() -> Cycle {
        let session_id = SessionId::new();
        let mut cycle = Cycle::new(session_id);
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle.take_events();
        cycle
    }

    fn create_fresh_cycle() -> Cycle {
        let session_id = SessionId::new();
        let mut cycle = Cycle::new(session_id);
        cycle.take_events();
        cycle
    }

    // ─────────────────────────────────────────────────────────────────────
    // Tests
    // ─────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn returns_component_when_exists() {
        let cycle = create_cycle_with_started_component();
        let cycle_id = cycle.id();
        let repo = Arc::new(MockCycleRepository::with_cycle(cycle));

        let handler = GetComponentHandler::new(repo);
        let query = GetComponentQuery {
            cycle_id,
            component_type: ComponentType::IssueRaising,
        };

        let result = handler.handle(query).await;
        assert!(result.is_ok());

        let component = result.unwrap();
        assert_eq!(component.cycle_id, cycle_id);
        assert_eq!(component.component_type, ComponentType::IssueRaising);
        assert_eq!(component.status, ComponentStatus::InProgress);
    }

    #[tokio::test]
    async fn returns_component_output_as_json() {
        let cycle = create_cycle_with_started_component();
        let cycle_id = cycle.id();
        let repo = Arc::new(MockCycleRepository::with_cycle(cycle));

        let handler = GetComponentHandler::new(repo);
        let query = GetComponentQuery {
            cycle_id,
            component_type: ComponentType::IssueRaising,
        };

        let result = handler.handle(query).await.unwrap();
        assert!(result.output.is_object());
        // IssueRaising output has potential_decisions field
        assert!(result.output.get("potential_decisions").is_some());
    }

    #[tokio::test]
    async fn returns_not_started_component() {
        let cycle = create_fresh_cycle();
        let cycle_id = cycle.id();
        let repo = Arc::new(MockCycleRepository::with_cycle(cycle));

        let handler = GetComponentHandler::new(repo);
        let query = GetComponentQuery {
            cycle_id,
            component_type: ComponentType::IssueRaising,
        };

        let result = handler.handle(query).await;
        assert!(result.is_ok());

        let component = result.unwrap();
        assert_eq!(component.status, ComponentStatus::NotStarted);
    }

    #[tokio::test]
    async fn returns_cycle_not_found_when_missing() {
        let repo = Arc::new(MockCycleRepository::new());

        let handler = GetComponentHandler::new(repo);
        let query = GetComponentQuery {
            cycle_id: CycleId::new(),
            component_type: ComponentType::IssueRaising,
        };

        let result = handler.handle(query).await;
        assert!(matches!(result, Err(GetComponentError::CycleNotFound(_))));
    }

    #[tokio::test]
    async fn returns_infrastructure_error_on_read_failure() {
        let repo = Arc::new(MockCycleRepository::failing());

        let handler = GetComponentHandler::new(repo);
        let query = GetComponentQuery {
            cycle_id: CycleId::new(),
            component_type: ComponentType::IssueRaising,
        };

        let result = handler.handle(query).await;
        assert!(matches!(result, Err(GetComponentError::Infrastructure(_))));
    }
}
