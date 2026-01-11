//! BranchCycleHandler - Command handler for creating cycle branches.
//!
//! Branching allows users to explore "what-if" scenarios without losing
//! their existing work. A branch copies completed components up to the
//! branch point, marks the branch point for revision, and starts fresh
//! for components after the branch point.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::domain::cycle::Cycle;
use crate::domain::foundation::{
    domain_event, CommandMetadata, ComponentType, CycleId, DomainError, EventId,
    SerializableDomainEvent, SessionId, Timestamp,
};
use crate::ports::{AccessChecker, AccessResult, CycleRepository, EventPublisher};

/// Command to branch an existing cycle at a specific component.
#[derive(Debug, Clone)]
pub struct BranchCycleCommand {
    /// The cycle to branch from.
    pub parent_cycle_id: CycleId,
    /// The component where branching occurs.
    /// This component will be marked for revision in the new branch.
    pub branch_point: ComponentType,
    /// Optional label for the branch (e.g., "Remote Option", "Risk Analysis").
    pub branch_label: Option<String>,
}

/// Result of successful cycle branching.
#[derive(Debug, Clone)]
pub struct BranchCycleResult {
    /// The newly created branch cycle.
    pub branch: Cycle,
    /// The emitted event.
    pub event: CycleBranchedEvent,
}

/// Event published when a cycle is branched.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleBranchedEvent {
    /// Unique event identifier.
    pub event_id: EventId,
    /// The new branch cycle that was created.
    pub cycle_id: CycleId,
    /// The parent cycle that was branched from.
    pub parent_cycle_id: CycleId,
    /// The session this cycle belongs to.
    pub session_id: SessionId,
    /// The component where branching occurred.
    pub branch_point: ComponentType,
    /// When the branch was created.
    pub created_at: Timestamp,
}

domain_event!(
    CycleBranchedEvent,
    event_type = "cycle.branched",
    aggregate_id = cycle_id,
    aggregate_type = "Cycle",
    occurred_at = created_at,
    event_id = event_id
);

/// Error type for cycle branching.
#[derive(Debug, Clone)]
pub enum BranchCycleError {
    /// Parent cycle not found.
    CycleNotFound(CycleId),
    /// Access denied by membership check.
    AccessDenied(crate::ports::AccessDeniedReason),
    /// Domain error (e.g., invalid branch point).
    Domain(DomainError),
}

impl std::fmt::Display for BranchCycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BranchCycleError::CycleNotFound(id) => write!(f, "Cycle not found: {}", id),
            BranchCycleError::AccessDenied(reason) => {
                write!(f, "Access denied: {:?}", reason)
            }
            BranchCycleError::Domain(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for BranchCycleError {}

impl From<DomainError> for BranchCycleError {
    fn from(err: DomainError) -> Self {
        BranchCycleError::Domain(err)
    }
}

/// Handler for branching cycles.
pub struct BranchCycleHandler {
    cycle_repository: Arc<dyn CycleRepository>,
    access_checker: Arc<dyn AccessChecker>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl BranchCycleHandler {
    pub fn new(
        cycle_repository: Arc<dyn CycleRepository>,
        access_checker: Arc<dyn AccessChecker>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            cycle_repository,
            access_checker,
            event_publisher,
        }
    }

    pub async fn handle(
        &self,
        cmd: BranchCycleCommand,
        metadata: CommandMetadata,
    ) -> Result<BranchCycleResult, BranchCycleError> {
        // 1. Find the parent cycle
        let parent_cycle = self
            .cycle_repository
            .find_by_id(&cmd.parent_cycle_id)
            .await?
            .ok_or(BranchCycleError::CycleNotFound(cmd.parent_cycle_id))?;

        // 2. Check access (membership-based limits)
        match self
            .access_checker
            .can_create_cycle(&metadata.user_id, &parent_cycle.session_id())
            .await?
        {
            AccessResult::Allowed => {}
            AccessResult::Denied(reason) => {
                return Err(BranchCycleError::AccessDenied(reason));
            }
        }

        // 3. Branch the cycle (domain logic handles validation)
        let branch = parent_cycle.branch_at(cmd.branch_point, cmd.branch_label)?;

        // 4. Persist the new branch
        self.cycle_repository.save(&branch).await?;

        // 5. Create and publish event
        let event = CycleBranchedEvent {
            event_id: EventId::new(),
            cycle_id: branch.id(),
            parent_cycle_id: cmd.parent_cycle_id,
            session_id: parent_cycle.session_id(),
            branch_point: cmd.branch_point,
            created_at: branch.created_at(),
        };

        let envelope = event
            .to_envelope()
            .with_correlation_id(metadata.correlation_id())
            .with_user_id(metadata.user_id.to_string());

        self.event_publisher.publish(envelope).await?;

        Ok(BranchCycleResult { branch, event })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{ErrorCode, EventEnvelope, UserId};
    use crate::domain::membership::TierLimits;
    use crate::ports::{AccessDeniedReason, UsageStats};
    use async_trait::async_trait;
    use std::sync::Mutex;

    // ─────────────────────────────────────────────────────────────────────
    // Mock implementations
    // ─────────────────────────────────────────────────────────────────────

    struct MockCycleRepository {
        cycles: Mutex<Vec<Cycle>>,
        saved_cycles: Mutex<Vec<Cycle>>,
        fail_save: bool,
    }

    impl MockCycleRepository {
        fn new() -> Self {
            Self {
                cycles: Mutex::new(Vec::new()),
                saved_cycles: Mutex::new(Vec::new()),
                fail_save: false,
            }
        }

        fn with_cycle(cycle: Cycle) -> Self {
            Self {
                cycles: Mutex::new(vec![cycle]),
                saved_cycles: Mutex::new(Vec::new()),
                fail_save: false,
            }
        }

        fn failing_with_cycle(cycle: Cycle) -> Self {
            Self {
                cycles: Mutex::new(vec![cycle]),
                saved_cycles: Mutex::new(Vec::new()),
                fail_save: true,
            }
        }

        #[allow(dead_code)]
        fn add_cycle(&self, cycle: Cycle) {
            self.cycles.lock().unwrap().push(cycle);
        }

        fn saved_cycles(&self) -> Vec<Cycle> {
            self.saved_cycles.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl CycleRepository for MockCycleRepository {
        async fn save(&self, cycle: &Cycle) -> Result<(), DomainError> {
            if self.fail_save {
                return Err(DomainError::new(
                    ErrorCode::DatabaseError,
                    "Simulated save failure",
                ));
            }
            self.saved_cycles.lock().unwrap().push(cycle.clone());
            Ok(())
        }

        async fn update(&self, _cycle: &Cycle) -> Result<(), DomainError> {
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

    struct MockAccessChecker {
        result: AccessResult,
    }

    impl MockAccessChecker {
        fn allowed() -> Self {
            Self {
                result: AccessResult::Allowed,
            }
        }

        fn denied(reason: AccessDeniedReason) -> Self {
            Self {
                result: AccessResult::Denied(reason),
            }
        }
    }

    #[async_trait]
    impl AccessChecker for MockAccessChecker {
        async fn can_create_session(&self, _user_id: &UserId) -> Result<AccessResult, DomainError> {
            Ok(AccessResult::Allowed)
        }

        async fn can_create_cycle(
            &self,
            _user_id: &UserId,
            _session_id: &SessionId,
        ) -> Result<AccessResult, DomainError> {
            Ok(self.result.clone())
        }

        async fn can_export(&self, _user_id: &UserId) -> Result<AccessResult, DomainError> {
            Ok(AccessResult::Allowed)
        }

        async fn get_tier_limits(&self, _user_id: &UserId) -> Result<TierLimits, DomainError> {
            Ok(TierLimits::for_tier(
                crate::domain::membership::MembershipTier::Free,
            ))
        }

        async fn get_usage(&self, _user_id: &UserId) -> Result<UsageStats, DomainError> {
            Ok(UsageStats::new())
        }
    }

    struct MockEventPublisher {
        published_events: Mutex<Vec<EventEnvelope>>,
        fail_publish: bool,
    }

    impl MockEventPublisher {
        fn new() -> Self {
            Self {
                published_events: Mutex::new(Vec::new()),
                fail_publish: false,
            }
        }

        fn published_events(&self) -> Vec<EventEnvelope> {
            self.published_events.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl EventPublisher for MockEventPublisher {
        async fn publish(&self, event: EventEnvelope) -> Result<(), DomainError> {
            if self.fail_publish {
                return Err(DomainError::new(
                    ErrorCode::InternalError,
                    "Simulated publish failure",
                ));
            }
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

    fn create_parent_cycle_with_started_component() -> Cycle {
        let session_id = SessionId::new();
        let mut cycle = Cycle::new(session_id);
        // Start IssueRaising so we can branch at it
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle.take_events(); // Clear events from setup
        cycle
    }

    fn create_handler(
        cycle_repo: Arc<dyn CycleRepository>,
        access: Arc<dyn AccessChecker>,
        publisher: Arc<dyn EventPublisher>,
    ) -> BranchCycleHandler {
        BranchCycleHandler::new(cycle_repo, access, publisher)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Tests
    // ─────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn branches_cycle_at_valid_branch_point() {
        let parent = create_parent_cycle_with_started_component();
        let parent_id = parent.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(parent));
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, access, publisher);

        let cmd = BranchCycleCommand {
            parent_cycle_id: parent_id,
            branch_point: ComponentType::IssueRaising,
            branch_label: None,
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.branch.is_branch());
        assert_eq!(result.branch.parent_cycle_id(), Some(parent_id));
    }

    #[tokio::test]
    async fn saves_branch_to_repository() {
        let parent = create_parent_cycle_with_started_component();
        let parent_id = parent.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(parent));
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo.clone(), access, publisher);

        let cmd = BranchCycleCommand {
            parent_cycle_id: parent_id,
            branch_point: ComponentType::IssueRaising,
            branch_label: None,
        };
        handler.handle(cmd, test_metadata()).await.unwrap();

        let saved = cycle_repo.saved_cycles();
        assert_eq!(saved.len(), 1);
        assert!(saved[0].is_branch());
    }

    #[tokio::test]
    async fn publishes_cycle_branched_event() {
        let parent = create_parent_cycle_with_started_component();
        let parent_id = parent.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(parent));
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, access, publisher.clone());

        let cmd = BranchCycleCommand {
            parent_cycle_id: parent_id,
            branch_point: ComponentType::IssueRaising,
            branch_label: None,
        };
        let result = handler.handle(cmd, test_metadata()).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "cycle.branched");
        assert_eq!(events[0].aggregate_id, result.branch.id().to_string());
    }

    #[tokio::test]
    async fn fails_when_parent_cycle_not_found() {
        let cycle_repo = Arc::new(MockCycleRepository::new());
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo.clone(), access, publisher.clone());

        let cmd = BranchCycleCommand {
            parent_cycle_id: CycleId::new(),
            branch_point: ComponentType::IssueRaising,
            branch_label: None,
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(result, Err(BranchCycleError::CycleNotFound(_))));
        assert!(cycle_repo.saved_cycles().is_empty());
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn fails_when_access_denied() {
        let parent = create_parent_cycle_with_started_component();
        let parent_id = parent.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(parent));
        let access = Arc::new(MockAccessChecker::denied(AccessDeniedReason::CycleLimitReached {
            current: 10,
            max: 10,
        }));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo.clone(), access, publisher.clone());

        let cmd = BranchCycleCommand {
            parent_cycle_id: parent_id,
            branch_point: ComponentType::IssueRaising,
            branch_label: None,
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(
            result,
            Err(BranchCycleError::AccessDenied(
                AccessDeniedReason::CycleLimitReached { .. }
            ))
        ));
        assert!(cycle_repo.saved_cycles().is_empty());
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn fails_when_branch_point_not_started() {
        let session_id = SessionId::new();
        let parent = Cycle::new(session_id); // No components started
        let parent_id = parent.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(parent));
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo.clone(), access, publisher.clone());

        let cmd = BranchCycleCommand {
            parent_cycle_id: parent_id,
            branch_point: ComponentType::IssueRaising,
            branch_label: None,
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(result, Err(BranchCycleError::Domain(_))));
        assert!(cycle_repo.saved_cycles().is_empty());
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn event_contains_correct_parent_and_branch_point() {
        let parent = create_parent_cycle_with_started_component();
        let parent_id = parent.id();
        let session_id = parent.session_id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(parent));
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, access, publisher);

        let cmd = BranchCycleCommand {
            parent_cycle_id: parent_id,
            branch_point: ComponentType::IssueRaising,
            branch_label: None,
        };
        let result = handler.handle(cmd, test_metadata()).await.unwrap();

        assert_eq!(result.event.parent_cycle_id, parent_id);
        assert_eq!(result.event.session_id, session_id);
        assert_eq!(result.event.branch_point, ComponentType::IssueRaising);
    }

    #[tokio::test]
    async fn includes_correlation_id_in_event() {
        let parent = create_parent_cycle_with_started_component();
        let parent_id = parent.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(parent));
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, access, publisher.clone());

        let cmd = BranchCycleCommand {
            parent_cycle_id: parent_id,
            branch_point: ComponentType::IssueRaising,
            branch_label: None,
        };
        handler.handle(cmd, test_metadata()).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(
            events[0].metadata.correlation_id,
            Some("test-correlation".to_string())
        );
    }

    #[tokio::test]
    async fn does_not_publish_event_on_save_failure() {
        let parent = create_parent_cycle_with_started_component();
        let parent_id = parent.id();

        let cycle_repo = Arc::new(MockCycleRepository::failing_with_cycle(parent));
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, access, publisher.clone());

        let cmd = BranchCycleCommand {
            parent_cycle_id: parent_id,
            branch_point: ComponentType::IssueRaising,
            branch_label: None,
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_err());
        assert!(publisher.published_events().is_empty());
    }
}
