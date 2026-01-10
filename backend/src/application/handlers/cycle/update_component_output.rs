//! UpdateComponentOutputHandler - Command handler for updating a component's output.
//!
//! Allows updating the output data for a component that is InProgress or NeedsRevision.
//! This is the main way AI conversations translate into structured component data.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::domain::foundation::{
    domain_event, CommandMetadata, ComponentType, CycleId, DomainError, EventId,
    SerializableDomainEvent, Timestamp,
};
use crate::ports::{CycleRepository, EventPublisher};

/// Command to update a component's output.
#[derive(Debug, Clone)]
pub struct UpdateComponentOutputCommand {
    /// The cycle containing the component.
    pub cycle_id: CycleId,
    /// The component type to update.
    pub component_type: ComponentType,
    /// The new output data as JSON.
    pub output: serde_json::Value,
}

/// Result of successful output update.
#[derive(Debug)]
pub struct UpdateComponentOutputResult {
    /// The emitted event.
    pub event: ComponentOutputUpdatedEvent,
}

/// Event published when a component's output is updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentOutputUpdatedEvent {
    /// Unique event identifier.
    pub event_id: EventId,
    /// The cycle containing the component.
    pub cycle_id: CycleId,
    /// The component type that was updated.
    pub component_type: ComponentType,
    /// When the output was updated.
    pub updated_at: Timestamp,
}

domain_event!(
    ComponentOutputUpdatedEvent,
    event_type = "component.output_updated",
    aggregate_id = cycle_id,
    aggregate_type = "Cycle",
    occurred_at = updated_at,
    event_id = event_id
);

/// Error type for updating component output.
#[derive(Debug, Clone)]
pub enum UpdateComponentOutputError {
    /// Cycle not found.
    CycleNotFound(CycleId),
    /// Domain error (e.g., component not in valid state).
    Domain(DomainError),
}

impl std::fmt::Display for UpdateComponentOutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateComponentOutputError::CycleNotFound(id) => write!(f, "Cycle not found: {}", id),
            UpdateComponentOutputError::Domain(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for UpdateComponentOutputError {}

impl From<DomainError> for UpdateComponentOutputError {
    fn from(err: DomainError) -> Self {
        UpdateComponentOutputError::Domain(err)
    }
}

/// Handler for updating component outputs.
pub struct UpdateComponentOutputHandler {
    cycle_repository: Arc<dyn CycleRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl UpdateComponentOutputHandler {
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
        cmd: UpdateComponentOutputCommand,
        metadata: CommandMetadata,
    ) -> Result<UpdateComponentOutputResult, UpdateComponentOutputError> {
        // 1. Find the cycle
        let mut cycle = self
            .cycle_repository
            .find_by_id(&cmd.cycle_id)
            .await?
            .ok_or(UpdateComponentOutputError::CycleNotFound(cmd.cycle_id))?;

        // 2. Update the component output (domain logic handles validation)
        cycle.update_component_output(cmd.component_type, cmd.output)?;

        // 3. Persist the updated cycle
        self.cycle_repository.update(&cycle).await?;

        // 4. Create and publish event
        let event = ComponentOutputUpdatedEvent {
            event_id: EventId::new(),
            cycle_id: cmd.cycle_id,
            component_type: cmd.component_type,
            updated_at: Timestamp::now(),
        };

        let envelope = event
            .to_envelope()
            .with_correlation_id(metadata.correlation_id())
            .with_user_id(metadata.user_id.to_string());

        self.event_publisher.publish(envelope).await?;

        Ok(UpdateComponentOutputResult { event })
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

    fn create_cycle_with_component_in_progress() -> Cycle {
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

    fn valid_issue_raising_output() -> serde_json::Value {
        serde_json::json!({
            "potential_decisions": ["Should I change jobs?"],
            "objectives": ["Financial stability", "Work-life balance"],
            "uncertainties": ["Market conditions"],
            "considerations": ["Family depends on income"],
            "user_confirmed": false
        })
    }

    fn create_handler(
        cycle_repo: Arc<dyn CycleRepository>,
        publisher: Arc<dyn EventPublisher>,
    ) -> UpdateComponentOutputHandler {
        UpdateComponentOutputHandler::new(cycle_repo, publisher)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Tests
    // ─────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn updates_output_for_in_progress_component() {
        let cycle = create_cycle_with_component_in_progress();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher);

        let cmd = UpdateComponentOutputCommand {
            cycle_id,
            component_type: ComponentType::IssueRaising,
            output: valid_issue_raising_output(),
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.event.cycle_id, cycle_id);
        assert_eq!(result.event.component_type, ComponentType::IssueRaising);
    }

    #[tokio::test]
    async fn persists_updated_output_in_repository() {
        let cycle = create_cycle_with_component_in_progress();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo.clone(), publisher);

        let cmd = UpdateComponentOutputCommand {
            cycle_id,
            component_type: ComponentType::IssueRaising,
            output: valid_issue_raising_output(),
        };
        handler.handle(cmd, test_metadata()).await.unwrap();

        let updated = cycle_repo.get_cycle(&cycle_id).unwrap();
        let component = updated.component(ComponentType::IssueRaising).unwrap();
        let output = component.output_as_value();
        assert!(output["potential_decisions"].is_array());
    }

    #[tokio::test]
    async fn publishes_component_output_updated_event() {
        let cycle = create_cycle_with_component_in_progress();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = UpdateComponentOutputCommand {
            cycle_id,
            component_type: ComponentType::IssueRaising,
            output: valid_issue_raising_output(),
        };
        handler.handle(cmd, test_metadata()).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "component.output_updated");
    }

    #[tokio::test]
    async fn fails_when_cycle_not_found() {
        let cycle_repo = Arc::new(MockCycleRepository::new());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = UpdateComponentOutputCommand {
            cycle_id: CycleId::new(),
            component_type: ComponentType::IssueRaising,
            output: valid_issue_raising_output(),
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(
            result,
            Err(UpdateComponentOutputError::CycleNotFound(_))
        ));
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn fails_when_component_not_started() {
        let cycle = create_fresh_cycle();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = UpdateComponentOutputCommand {
            cycle_id,
            component_type: ComponentType::IssueRaising,
            output: valid_issue_raising_output(),
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(
            result,
            Err(UpdateComponentOutputError::Domain(_))
        ));
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn fails_when_cycle_archived() {
        let mut cycle = create_cycle_with_component_in_progress();
        let cycle_id = cycle.id();
        cycle.archive().unwrap();
        cycle.take_events();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = UpdateComponentOutputCommand {
            cycle_id,
            component_type: ComponentType::IssueRaising,
            output: valid_issue_raising_output(),
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(
            result,
            Err(UpdateComponentOutputError::Domain(_))
        ));
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn includes_correlation_id_in_event() {
        let cycle = create_cycle_with_component_in_progress();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = UpdateComponentOutputCommand {
            cycle_id,
            component_type: ComponentType::IssueRaising,
            output: valid_issue_raising_output(),
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
        let cycle = create_cycle_with_component_in_progress();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::failing_update_with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = UpdateComponentOutputCommand {
            cycle_id,
            component_type: ComponentType::IssueRaising,
            output: valid_issue_raising_output(),
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_err());
        assert!(publisher.published_events().is_empty());
    }
}
