//! ArchiveCycleHandler - Command handler for archiving a cycle.
//!
//! Archiving a cycle transitions it to Archived status.
//! Archived cycles cannot be modified and serve as historical records.
//! Both Active and Completed cycles can be archived.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::domain::foundation::{
    domain_event, CommandMetadata, CycleId, DomainError, EventId, SerializableDomainEvent,
    Timestamp,
};
use crate::ports::{CycleRepository, EventPublisher};

/// Command to archive a cycle.
#[derive(Debug, Clone)]
pub struct ArchiveCycleCommand {
    /// The cycle to archive.
    pub cycle_id: CycleId,
}

/// Result of successful cycle archival.
#[derive(Debug)]
pub struct ArchiveCycleResult {
    /// The emitted event.
    pub event: CycleArchivedEvent,
}

/// Event published when a cycle is archived.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleArchivedEvent {
    /// Unique event identifier.
    pub event_id: EventId,
    /// The cycle that was archived.
    pub cycle_id: CycleId,
    /// When the cycle was archived.
    pub archived_at: Timestamp,
}

domain_event!(
    CycleArchivedEvent,
    event_type = "cycle.archived",
    aggregate_id = cycle_id,
    aggregate_type = "Cycle",
    occurred_at = archived_at,
    event_id = event_id
);

/// Error type for archiving a cycle.
#[derive(Debug, Clone)]
pub enum ArchiveCycleError {
    /// Cycle not found.
    CycleNotFound(CycleId),
    /// Domain error (e.g., cycle already archived).
    Domain(DomainError),
}

impl std::fmt::Display for ArchiveCycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchiveCycleError::CycleNotFound(id) => write!(f, "Cycle not found: {}", id),
            ArchiveCycleError::Domain(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for ArchiveCycleError {}

impl From<DomainError> for ArchiveCycleError {
    fn from(err: DomainError) -> Self {
        ArchiveCycleError::Domain(err)
    }
}

/// Handler for archiving cycles.
pub struct ArchiveCycleHandler {
    cycle_repository: Arc<dyn CycleRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl ArchiveCycleHandler {
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
        cmd: ArchiveCycleCommand,
        metadata: CommandMetadata,
    ) -> Result<ArchiveCycleResult, ArchiveCycleError> {
        // 1. Find the cycle
        let mut cycle = self
            .cycle_repository
            .find_by_id(&cmd.cycle_id)
            .await?
            .ok_or(ArchiveCycleError::CycleNotFound(cmd.cycle_id))?;

        // 2. Archive the cycle (domain logic handles validation)
        cycle.archive()?;

        // 3. Persist the updated cycle
        self.cycle_repository.update(&cycle).await?;

        // 4. Create and publish event
        let event = CycleArchivedEvent {
            event_id: EventId::new(),
            cycle_id: cmd.cycle_id,
            archived_at: Timestamp::now(),
        };

        let envelope = event
            .to_envelope()
            .with_correlation_id(metadata.correlation_id())
            .with_user_id(metadata.user_id.to_string());

        self.event_publisher.publish(envelope).await?;

        Ok(ArchiveCycleResult { event })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cycle::Cycle;
    use crate::domain::foundation::{
        ComponentType, CycleStatus, ErrorCode, EventEnvelope, SessionId, UserId,
    };
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

    fn create_active_cycle() -> Cycle {
        let session_id = SessionId::new();
        let mut cycle = Cycle::new(session_id);
        cycle.take_events(); // Clear creation event
        cycle
    }

    fn create_completed_cycle() -> Cycle {
        let session_id = SessionId::new();
        let mut cycle = Cycle::new(session_id);
        // Must start all prior components before DecisionQuality (domain enforces order)
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle.start_component(ComponentType::ProblemFrame).unwrap();
        cycle.start_component(ComponentType::Objectives).unwrap();
        cycle.start_component(ComponentType::Alternatives).unwrap();
        cycle.start_component(ComponentType::Consequences).unwrap();
        cycle.start_component(ComponentType::Tradeoffs).unwrap();
        cycle.start_component(ComponentType::Recommendation).unwrap();
        cycle
            .start_component(ComponentType::DecisionQuality)
            .unwrap();
        cycle
            .complete_component(ComponentType::DecisionQuality)
            .unwrap();
        cycle.complete().unwrap();
        cycle.take_events();
        cycle
    }

    fn create_handler(
        cycle_repo: Arc<dyn CycleRepository>,
        publisher: Arc<dyn EventPublisher>,
    ) -> ArchiveCycleHandler {
        ArchiveCycleHandler::new(cycle_repo, publisher)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Tests
    // ─────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn archives_active_cycle_successfully() {
        let cycle = create_active_cycle();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher);

        let cmd = ArchiveCycleCommand { cycle_id };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.event.cycle_id, cycle_id);
    }

    #[tokio::test]
    async fn archives_completed_cycle_successfully() {
        let cycle = create_completed_cycle();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher);

        let cmd = ArchiveCycleCommand { cycle_id };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn updates_cycle_status_in_repository() {
        let cycle = create_active_cycle();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo.clone(), publisher);

        let cmd = ArchiveCycleCommand { cycle_id };
        handler.handle(cmd, test_metadata()).await.unwrap();

        let updated = cycle_repo.get_cycle(&cycle_id).unwrap();
        assert_eq!(updated.status(), CycleStatus::Archived);
    }

    #[tokio::test]
    async fn publishes_cycle_archived_event() {
        let cycle = create_active_cycle();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = ArchiveCycleCommand { cycle_id };
        handler.handle(cmd, test_metadata()).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "cycle.archived");
    }

    #[tokio::test]
    async fn fails_when_cycle_not_found() {
        let cycle_repo = Arc::new(MockCycleRepository::new());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = ArchiveCycleCommand {
            cycle_id: CycleId::new(),
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(result, Err(ArchiveCycleError::CycleNotFound(_))));
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn fails_when_cycle_already_archived() {
        let mut cycle = create_active_cycle();
        let cycle_id = cycle.id();
        cycle.archive().unwrap();
        cycle.take_events();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = ArchiveCycleCommand { cycle_id };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(result, Err(ArchiveCycleError::Domain(_))));
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn includes_correlation_id_in_event() {
        let cycle = create_active_cycle();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = ArchiveCycleCommand { cycle_id };
        handler.handle(cmd, test_metadata()).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(
            events[0].metadata.correlation_id,
            Some("test-correlation".to_string())
        );
    }

    #[tokio::test]
    async fn does_not_publish_event_on_update_failure() {
        let cycle = create_active_cycle();
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::failing_update_with_cycle(cycle));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, publisher.clone());

        let cmd = ArchiveCycleCommand { cycle_id };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_err());
        assert!(publisher.published_events().is_empty());
    }
}
