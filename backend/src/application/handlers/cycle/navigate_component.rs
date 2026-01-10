//! NavigateComponentHandler - Command handler for navigating to a component.
//!
//! Navigation allows users to return to previously started components
//! or advance to the next component if prerequisites are met.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::domain::foundation::{
    domain_event, CommandMetadata, ComponentType, CycleId, DomainError, EventId,
    SerializableDomainEvent, Timestamp,
};
use crate::ports::{CycleRepository, EventPublisher};

/// Command to navigate to a component within a cycle.
#[derive(Debug, Clone)]
pub struct NavigateComponentCommand {
    /// The cycle containing the component.
    pub cycle_id: CycleId,
    /// The component type to navigate to.
    pub target: ComponentType,
}

/// Result of successful navigation.
#[derive(Debug)]
pub struct NavigateComponentResult {
    /// The emitted event.
    pub event: NavigatedToComponentEvent,
}

/// Event published when navigation to a component occurs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigatedToComponentEvent {
    /// Unique event identifier.
    pub event_id: EventId,
    /// The cycle containing the component.
    pub cycle_id: CycleId,
    /// The component that was navigated to.
    pub component_type: ComponentType,
    /// When the navigation occurred.
    pub navigated_at: Timestamp,
}

domain_event!(
    NavigatedToComponentEvent,
    event_type = "component.navigated_to",
    aggregate_id = cycle_id,
    aggregate_type = "Cycle",
    occurred_at = navigated_at,
    event_id = event_id
);

/// Error type for navigating to a component.
#[derive(Debug, Clone)]
pub enum NavigateComponentError {
    /// Cycle not found.
    CycleNotFound(CycleId),
    /// Domain error (e.g., invalid navigation target).
    Domain(DomainError),
}

impl std::fmt::Display for NavigateComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NavigateComponentError::CycleNotFound(id) => write!(f, "Cycle not found: {}", id),
            NavigateComponentError::Domain(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for NavigateComponentError {}

impl From<DomainError> for NavigateComponentError {
    fn from(err: DomainError) -> Self {
        NavigateComponentError::Domain(err)
    }
}

/// Handler for navigating to components.
pub struct NavigateComponentHandler {
    cycle_repository: Arc<dyn CycleRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl NavigateComponentHandler {
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
        cmd: NavigateComponentCommand,
        metadata: CommandMetadata,
    ) -> Result<NavigateComponentResult, NavigateComponentError> {
        // 1. Find the cycle
        let mut cycle = self
            .cycle_repository
            .find_by_id(&cmd.cycle_id)
            .await?
            .ok_or(NavigateComponentError::CycleNotFound(cmd.cycle_id))?;

        // 2. Navigate to the component (domain logic handles validation)
        cycle.navigate_to(cmd.target)?;

        // 3. Persist the updated cycle
        self.cycle_repository.update(&cycle).await?;

        // 4. Create and publish event
        let event = NavigatedToComponentEvent {
            event_id: EventId::new(),
            cycle_id: cmd.cycle_id,
            component_type: cmd.target,
            navigated_at: Timestamp::now(),
        };

        let envelope = event
            .to_envelope()
            .with_correlation_id(metadata.correlation_id())
            .with_user_id(metadata.user_id.to_string());

        self.event_publisher.publish(envelope).await?;

        Ok(NavigateComponentResult { event })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cycle::Cycle;
    use crate::domain::foundation::{ErrorCode, EventEnvelope, SessionId, UserId};
    use async_trait::async_trait;
    use std::sync::Mutex;

    // ─────────────────────────────────────────────────────────────────────
    // Mock implementations
    // ─────────────────────────────────────────────────────────────────────

    struct MockCycleRepository {
        cycles: Mutex<Vec<Cycle>>,
        fail_update: bool,
    }

    impl MockCycleRepository {
        fn new() -> Self {
            Self {
                cycles: Mutex::new(Vec::new()),
                fail_update: false,
            }
        }

        fn with_cycle(cycle: Cycle) -> Self {
            Self {
                cycles: Mutex::new(vec![cycle]),
                fail_update: false,
            }
        }

        fn failing_update_with_cycle(cycle: Cycle) -> Self {
            Self {
                cycles: Mutex::new(vec![cycle]),
                fail_update: true,
            }
        }

        fn get_cycle(&self, id: &CycleId) -> Option<Cycle> {
            self.cycles
                .lock()
                .unwrap()
                .iter()
                .find(|c| c.id() == *id)
                .cloned()
        }
    }

    #[async_trait]
    impl CycleRepository for MockCycleRepository {
        async fn save(&self, cycle: &Cycle) -> Result<(), DomainError> {
            self.cycles.lock().unwrap().push(cycle.clone());
            Ok(())
        }

        async fn update(&self, cycle: &Cycle) -> Result<(), DomainError> {
            if self.fail_update {
                return Err(DomainError::new(
                    ErrorCode::DatabaseError,
                    "Simulated update failure",
                ));
            }
            let mut cycles = self.cycles.lock().unwrap();
            if let Some(pos) = cycles.iter().position(|c| c.id() == cycle.id()) {
                cycles[pos] = cycle.clone();
            }
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

    fn create_cycle_with_issue_raising_started() -> Cycle {
        let session_id = SessionId::new();
        let mut cycle = Cycle::new(session_id);
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle.take_events(); // Clear events from setup
        cycle
    }

    fn create_handler(
        cycle_repo: Arc<dyn CycleRepository>,
        publisher: Arc<dyn EventPublisher>,
    ) -> NavigateComponentHandler {
        NavigateComponentHandler::new(cycle_repo, publisher)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Tests
    // ─────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn navigates_to_started_component_successfully() {
        let cycle = create_cycle_with_issue_raising_started();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher);

        // Navigate back to IssueRaising (already started)
        let cmd = NavigateComponentCommand {
            cycle_id,
            target: ComponentType::IssueRaising,
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.event.cycle_id, cycle_id);
        assert_eq!(result.event.component_type, ComponentType::IssueRaising);
    }

    #[tokio::test]
    async fn navigates_to_next_component_when_prereq_started() {
        let cycle = create_cycle_with_issue_raising_started();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher);

        // Navigate to ProblemFrame (IssueRaising is started, so this should work)
        let cmd = NavigateComponentCommand {
            cycle_id,
            target: ComponentType::ProblemFrame,
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().event.component_type,
            ComponentType::ProblemFrame
        );
    }

    #[tokio::test]
    async fn updates_cycle_in_repository() {
        let cycle = create_cycle_with_issue_raising_started();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo.clone(), publisher);

        let cmd = NavigateComponentCommand {
            cycle_id,
            target: ComponentType::ProblemFrame,
        };
        handler.handle(cmd, test_metadata()).await.unwrap();

        let updated = cycle_repo.get_cycle(&cycle_id).unwrap();
        assert_eq!(updated.current_step(), ComponentType::ProblemFrame);
    }

    #[tokio::test]
    async fn publishes_navigated_to_event() {
        let cycle = create_cycle_with_issue_raising_started();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = NavigateComponentCommand {
            cycle_id,
            target: ComponentType::ProblemFrame,
        };
        handler.handle(cmd, test_metadata()).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "component.navigated_to");
    }

    #[tokio::test]
    async fn fails_when_cycle_not_found() {
        let cycle_repo = Arc::new(MockCycleRepository::new());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = NavigateComponentCommand {
            cycle_id: CycleId::new(),
            target: ComponentType::IssueRaising,
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(
            result,
            Err(NavigateComponentError::CycleNotFound(_))
        ));
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn fails_when_navigating_to_component_without_prereq() {
        let cycle = {
            let session_id = SessionId::new();
            let mut cycle = Cycle::new(session_id);
            cycle.take_events();
            cycle
        };
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        // Try to navigate to Alternatives without starting prior components
        let cmd = NavigateComponentCommand {
            cycle_id,
            target: ComponentType::Alternatives,
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(result, Err(NavigateComponentError::Domain(_))));
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn includes_correlation_id_in_event() {
        let cycle = create_cycle_with_issue_raising_started();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = NavigateComponentCommand {
            cycle_id,
            target: ComponentType::IssueRaising,
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
        let cycle = create_cycle_with_issue_raising_started();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::failing_update_with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = NavigateComponentCommand {
            cycle_id,
            target: ComponentType::IssueRaising,
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_err());
        assert!(publisher.published_events().is_empty());
    }
}
