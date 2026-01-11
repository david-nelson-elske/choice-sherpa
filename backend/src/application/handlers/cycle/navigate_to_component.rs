//! NavigateToComponentHandler - Command handler for navigating to a component.
//!
//! Navigation changes the current step within a cycle without changing
//! component status. This allows users to revisit previous components
//! or jump ahead in the workflow.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::domain::cycle::Cycle;
use crate::domain::foundation::{
    domain_event, CommandMetadata, ComponentType, CycleId, DomainError, EventId,
    SerializableDomainEvent, Timestamp,
};
use crate::ports::{CycleRepository, EventPublisher};

/// Command to navigate to a component within a cycle.
#[derive(Debug, Clone)]
pub struct NavigateToComponentCommand {
    /// The cycle to navigate within.
    pub cycle_id: CycleId,
    /// The component type to navigate to.
    pub component_type: ComponentType,
}

/// Result of successfully navigating to a component.
#[derive(Debug, Clone)]
pub struct NavigateToComponentResult {
    /// The updated cycle.
    pub cycle: Cycle,
    /// The emitted event.
    pub event: NavigatedToComponentEvent,
}

/// Event published when navigation occurs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigatedToComponentEvent {
    /// Unique event identifier.
    pub event_id: EventId,
    /// The cycle that was navigated.
    pub cycle_id: CycleId,
    /// The previous component (before navigation).
    pub from_component: ComponentType,
    /// The component navigated to.
    pub to_component: ComponentType,
    /// When the navigation occurred.
    pub navigated_at: Timestamp,
}

domain_event!(
    NavigatedToComponentEvent,
    event_type = "cycle.navigated.v1",
    schema_version = 1,
    aggregate_id = cycle_id,
    aggregate_type = "Cycle",
    occurred_at = navigated_at,
    event_id = event_id
);

/// Error type for navigating to a component.
#[derive(Debug, Clone)]
pub enum NavigateToComponentError {
    /// Cycle not found.
    CycleNotFound(CycleId),
    /// Domain error (e.g., invalid navigation).
    Domain(DomainError),
}

impl std::fmt::Display for NavigateToComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NavigateToComponentError::CycleNotFound(id) => write!(f, "Cycle not found: {}", id),
            NavigateToComponentError::Domain(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for NavigateToComponentError {}

impl From<DomainError> for NavigateToComponentError {
    fn from(err: DomainError) -> Self {
        NavigateToComponentError::Domain(err)
    }
}

/// Handler for navigating to components.
pub struct NavigateToComponentHandler {
    cycle_repository: Arc<dyn CycleRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl NavigateToComponentHandler {
    pub fn new(
        cycle_repository: Arc<dyn CycleRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            cycle_repository,
            event_publisher,
        }
    }

    pub async fn handle(
        &self,
        cmd: NavigateToComponentCommand,
        metadata: CommandMetadata,
    ) -> Result<NavigateToComponentResult, NavigateToComponentError> {
        // 1. Find the cycle
        let mut cycle = self
            .cycle_repository
            .find_by_id(&cmd.cycle_id)
            .await?
            .ok_or(NavigateToComponentError::CycleNotFound(cmd.cycle_id))?;

        // 2. Capture previous step for event
        let from_component = cycle.current_step();

        // 3. Navigate to the component (domain logic handles validation)
        cycle.navigate_to(cmd.component_type)?;

        // 4. Persist the updated cycle
        self.cycle_repository.update(&cycle).await?;

        // 5. Create and publish event
        let event = NavigatedToComponentEvent {
            event_id: EventId::new(),
            cycle_id: cmd.cycle_id,
            from_component,
            to_component: cmd.component_type,
            navigated_at: Timestamp::now(),
        };

        let envelope = event
            .to_envelope()
            .with_correlation_id(metadata.correlation_id())
            .with_user_id(metadata.user_id.to_string());

        self.event_publisher.publish(envelope).await?;

        Ok(NavigateToComponentResult { cycle, event })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{ErrorCode, EventEnvelope, SessionId, UserId};
    use async_trait::async_trait;
    use std::sync::Mutex;

    // ─────────────────────────────────────────────────────────────────────
    // Mock implementations
    // ─────────────────────────────────────────────────────────────────────

    struct MockCycleRepository {
        cycles: Mutex<Vec<Cycle>>,
        updated_cycles: Mutex<Vec<Cycle>>,
        fail_update: bool,
    }

    impl MockCycleRepository {
        fn with_cycle(cycle: Cycle) -> Self {
            Self {
                cycles: Mutex::new(vec![cycle]),
                updated_cycles: Mutex::new(Vec::new()),
                fail_update: false,
            }
        }

        fn failing_with_cycle(cycle: Cycle) -> Self {
            Self {
                cycles: Mutex::new(vec![cycle]),
                updated_cycles: Mutex::new(Vec::new()),
                fail_update: true,
            }
        }

        fn updated_cycles(&self) -> Vec<Cycle> {
            self.updated_cycles.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl CycleRepository for MockCycleRepository {
        async fn save(&self, _cycle: &Cycle) -> Result<(), DomainError> {
            Ok(())
        }

        async fn update(&self, cycle: &Cycle) -> Result<(), DomainError> {
            if self.fail_update {
                return Err(DomainError::new(
                    ErrorCode::DatabaseError,
                    "Simulated update failure",
                ));
            }
            self.updated_cycles.lock().unwrap().push(cycle.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: &CycleId) -> Result<Option<Cycle>, DomainError> {
            Ok(self
                .cycles
                .lock()
                .unwrap()
                .iter()
                .find(|c| c.id() == *id)
                .cloned())
        }

        async fn exists(&self, id: &CycleId) -> Result<bool, DomainError> {
            Ok(self.cycles.lock().unwrap().iter().any(|c| c.id() == *id))
        }

        async fn find_by_session_id(&self, _: &SessionId) -> Result<Vec<Cycle>, DomainError> {
            Ok(vec![])
        }

        async fn find_primary_by_session_id(&self, _: &SessionId) -> Result<Option<Cycle>, DomainError> {
            Ok(None)
        }

        async fn find_branches(&self, _: &CycleId) -> Result<Vec<Cycle>, DomainError> {
            Ok(vec![])
        }

        async fn count_by_session_id(&self, _: &SessionId) -> Result<u32, DomainError> {
            Ok(0)
        }

        async fn delete(&self, _: &CycleId) -> Result<(), DomainError> {
            Ok(())
        }
    }

    struct MockEventPublisher {
        published_events: Mutex<Vec<EventEnvelope>>,
    }

    impl MockEventPublisher {
        fn new() -> Self {
            Self {
                published_events: Mutex::new(Vec::new()),
            }
        }

        fn published_events(&self) -> Vec<EventEnvelope> {
            self.published_events.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl EventPublisher for MockEventPublisher {
        async fn publish(&self, event: EventEnvelope) -> Result<(), DomainError> {
            self.published_events.lock().unwrap().push(event);
            Ok(())
        }

        async fn publish_all(&self, events: Vec<EventEnvelope>) -> Result<(), DomainError> {
            for event in events {
                self.publish(event).await?;
            }
            Ok(())
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Test helpers
    // ─────────────────────────────────────────────────────────────────────

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    fn test_metadata() -> CommandMetadata {
        CommandMetadata::new(test_user_id()).with_correlation_id("test-correlation")
    }

    fn create_cycle_with_multiple_components() -> Cycle {
        let mut cycle = Cycle::new(SessionId::new());
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle.complete_component(ComponentType::IssueRaising).unwrap();
        cycle.start_component(ComponentType::ProblemFrame).unwrap();
        cycle.take_events(); // Clear setup events
        cycle
    }

    fn create_handler(
        cycle_repo: Arc<dyn CycleRepository>,
        publisher: Arc<dyn EventPublisher>,
    ) -> NavigateToComponentHandler {
        NavigateToComponentHandler::new(cycle_repo, publisher)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Tests
    // ─────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn navigates_to_previous_component() {
        let cycle = create_cycle_with_multiple_components();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher);

        let cmd = NavigateToComponentCommand {
            cycle_id,
            component_type: ComponentType::IssueRaising,
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.cycle.current_step(), ComponentType::IssueRaising);
    }

    #[tokio::test]
    async fn saves_updated_cycle_to_repository() {
        let cycle = create_cycle_with_multiple_components();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo.clone(), publisher);

        let cmd = NavigateToComponentCommand {
            cycle_id,
            component_type: ComponentType::IssueRaising,
        };
        handler.handle(cmd, test_metadata()).await.unwrap();

        let updated = cycle_repo.updated_cycles();
        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].current_step(), ComponentType::IssueRaising);
    }

    #[tokio::test]
    async fn publishes_navigated_event() {
        let cycle = create_cycle_with_multiple_components();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = NavigateToComponentCommand {
            cycle_id,
            component_type: ComponentType::IssueRaising,
        };
        handler.handle(cmd, test_metadata()).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "cycle.navigated.v1");
        assert_eq!(events[0].aggregate_id, cycle_id.to_string());
    }

    #[tokio::test]
    async fn event_includes_from_and_to_components() {
        let cycle = create_cycle_with_multiple_components();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = NavigateToComponentCommand {
            cycle_id,
            component_type: ComponentType::IssueRaising,
        };
        let result = handler.handle(cmd, test_metadata()).await.unwrap();

        assert_eq!(result.event.from_component, ComponentType::ProblemFrame);
        assert_eq!(result.event.to_component, ComponentType::IssueRaising);
    }

    #[tokio::test]
    async fn fails_when_cycle_not_found() {
        let cycle = create_cycle_with_multiple_components();
        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = NavigateToComponentCommand {
            cycle_id: CycleId::new(),
            component_type: ComponentType::IssueRaising,
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(result, Err(NavigateToComponentError::CycleNotFound(_))));
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn includes_correlation_id_in_event() {
        let cycle = create_cycle_with_multiple_components();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = NavigateToComponentCommand {
            cycle_id,
            component_type: ComponentType::IssueRaising,
        };
        handler.handle(cmd, test_metadata()).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(
            events[0].metadata.correlation_id,
            Some("test-correlation".to_string())
        );
    }

    #[tokio::test]
    async fn does_not_publish_event_on_update_failure() {
        let cycle = create_cycle_with_multiple_components();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::failing_with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = NavigateToComponentCommand {
            cycle_id,
            component_type: ComponentType::IssueRaising,
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_err());
        assert!(publisher.published_events().is_empty());
    }
}
