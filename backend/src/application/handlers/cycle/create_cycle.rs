//! CreateCycleHandler - Command handler for creating new cycles.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::domain::cycle::Cycle;
use crate::domain::foundation::{
    domain_event, CommandMetadata, CycleId, DomainError, EventId, SerializableDomainEvent,
    SessionId, Timestamp,
};
use crate::ports::{AccessChecker, AccessResult, CycleRepository, EventPublisher, SessionRepository};

/// Command to create a new cycle.
#[derive(Debug, Clone)]
pub struct CreateCycleCommand {
    /// Session to create the cycle in.
    pub session_id: SessionId,
}

/// Result of successful cycle creation.
#[derive(Debug, Clone)]
pub struct CreateCycleResult {
    /// The created cycle.
    pub cycle: Cycle,
    /// The emitted event.
    pub event: CycleCreatedEvent,
}

/// Event published when a cycle is created.
///
/// This follows the structure expected by SessionCycleTracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleCreatedEvent {
    /// Unique event identifier.
    pub event_id: EventId,
    /// The cycle that was created.
    pub cycle_id: CycleId,
    /// The session this cycle belongs to.
    pub session_id: SessionId,
    /// Parent cycle if this is a branch.
    pub parent_cycle_id: Option<CycleId>,
    /// When the cycle was created.
    pub created_at: Timestamp,
}

domain_event!(
    CycleCreatedEvent,
    event_type = "cycle.created",
    aggregate_id = cycle_id,
    aggregate_type = "Cycle",
    occurred_at = created_at,
    event_id = event_id
);

/// Error type for cycle creation.
#[derive(Debug, Clone)]
pub enum CreateCycleError {
    /// Session not found.
    SessionNotFound(SessionId),
    /// Access denied by membership check.
    AccessDenied(crate::ports::AccessDeniedReason),
    /// Domain error.
    Domain(DomainError),
}

impl std::fmt::Display for CreateCycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateCycleError::SessionNotFound(id) => write!(f, "Session not found: {}", id),
            CreateCycleError::AccessDenied(reason) => {
                write!(f, "Access denied: {:?}", reason)
            }
            CreateCycleError::Domain(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for CreateCycleError {}

impl From<DomainError> for CreateCycleError {
    fn from(err: DomainError) -> Self {
        CreateCycleError::Domain(err)
    }
}

/// Handler for creating cycles.
pub struct CreateCycleHandler {
    cycle_repository: Arc<dyn CycleRepository>,
    session_repository: Arc<dyn SessionRepository>,
    access_checker: Arc<dyn AccessChecker>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl CreateCycleHandler {
    pub fn new(
        cycle_repository: Arc<dyn CycleRepository>,
        session_repository: Arc<dyn SessionRepository>,
        access_checker: Arc<dyn AccessChecker>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            cycle_repository,
            session_repository,
            access_checker,
            event_publisher,
        }
    }

    pub async fn handle(
        &self,
        cmd: CreateCycleCommand,
        metadata: CommandMetadata,
    ) -> Result<CreateCycleResult, CreateCycleError> {
        // 1. Verify session exists
        let session = self
            .session_repository
            .find_by_id(&cmd.session_id)
            .await?
            .ok_or(CreateCycleError::SessionNotFound(cmd.session_id))?;

        // 2. Check access (membership-based limits)
        match self
            .access_checker
            .can_create_cycle(&metadata.user_id, session.id())
            .await?
        {
            AccessResult::Allowed => {}
            AccessResult::Denied(reason) => {
                return Err(CreateCycleError::AccessDenied(reason));
            }
        }

        // 3. Create cycle aggregate
        let cycle = Cycle::new(cmd.session_id);

        // 4. Persist cycle
        self.cycle_repository.save(&cycle).await?;

        // 5. Create and publish event
        let event = CycleCreatedEvent {
            event_id: EventId::new(),
            cycle_id: cycle.id(),
            session_id: cmd.session_id,
            parent_cycle_id: None,
            created_at: cycle.created_at(),
        };

        let envelope = event
            .to_envelope()
            .with_correlation_id(metadata.correlation_id())
            .with_user_id(metadata.user_id.to_string());

        self.event_publisher.publish(envelope).await?;

        Ok(CreateCycleResult { cycle, event })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{ErrorCode, EventEnvelope};
    use crate::domain::membership::TierLimits;
    use crate::domain::session::Session;
    use crate::ports::{AccessDeniedReason, UsageStats};
    use async_trait::async_trait;
    use std::sync::Mutex;

    // ─────────────────────────────────────────────────────────────────────
    // Mock implementations
    // ─────────────────────────────────────────────────────────────────────

    struct MockCycleRepository {
        saved_cycles: Mutex<Vec<Cycle>>,
        fail_save: bool,
    }

    impl MockCycleRepository {
        fn new() -> Self {
            Self {
                saved_cycles: Mutex::new(Vec::new()),
                fail_save: false,
            }
        }

        fn failing() -> Self {
            Self {
                saved_cycles: Mutex::new(Vec::new()),
                fail_save: true,
            }
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

        async fn find_by_id(&self, _id: &CycleId) -> Result<Option<Cycle>, DomainError> {
            Ok(None)
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

    struct MockSessionRepository {
        sessions: Mutex<Vec<Session>>,
    }

    impl MockSessionRepository {
        fn new() -> Self {
            Self {
                sessions: Mutex::new(Vec::new()),
            }
        }

        fn with_session(session: Session) -> Self {
            Self {
                sessions: Mutex::new(vec![session]),
            }
        }
    }

    #[async_trait]
    impl SessionRepository for MockSessionRepository {
        async fn save(&self, session: &Session) -> Result<(), DomainError> {
            self.sessions.lock().unwrap().push(session.clone());
            Ok(())
        }

        async fn update(&self, _session: &Session) -> Result<(), DomainError> {
            Ok(())
        }

        async fn find_by_id(&self, id: &SessionId) -> Result<Option<Session>, DomainError> {
            Ok(self
                .sessions
                .lock()
                .unwrap()
                .iter()
                .find(|s| s.id() == id)
                .cloned())
        }

        async fn exists(&self, id: &SessionId) -> Result<bool, DomainError> {
            Ok(self.sessions.lock().unwrap().iter().any(|s| s.id() == id))
        }

        async fn find_by_user_id(
            &self,
            _user_id: &crate::domain::foundation::UserId,
        ) -> Result<Vec<Session>, DomainError> {
            Ok(vec![])
        }

        async fn count_active_by_user(
            &self,
            _user_id: &crate::domain::foundation::UserId,
        ) -> Result<u32, DomainError> {
            Ok(0)
        }

        async fn delete(&self, _id: &SessionId) -> Result<(), DomainError> {
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
        async fn can_create_session(
            &self,
            _user_id: &crate::domain::foundation::UserId,
        ) -> Result<AccessResult, DomainError> {
            Ok(AccessResult::Allowed)
        }

        async fn can_create_cycle(
            &self,
            _user_id: &crate::domain::foundation::UserId,
            _session_id: &SessionId,
        ) -> Result<AccessResult, DomainError> {
            Ok(self.result.clone())
        }

        async fn can_export(
            &self,
            _user_id: &crate::domain::foundation::UserId,
        ) -> Result<AccessResult, DomainError> {
            Ok(AccessResult::Allowed)
        }

        async fn get_tier_limits(
            &self,
            _user_id: &crate::domain::foundation::UserId,
        ) -> Result<TierLimits, DomainError> {
            Ok(TierLimits::for_tier(
                crate::domain::membership::MembershipTier::Free,
            ))
        }

        async fn get_usage(
            &self,
            _user_id: &crate::domain::foundation::UserId,
        ) -> Result<UsageStats, DomainError> {
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

    fn test_user_id() -> crate::domain::foundation::UserId {
        crate::domain::foundation::UserId::new("test-user-123").unwrap()
    }

    fn test_session() -> Session {
        Session::new(SessionId::new(), test_user_id(), "Test Session".to_string()).unwrap()
    }

    fn test_metadata() -> CommandMetadata {
        CommandMetadata::new(test_user_id()).with_correlation_id("test-correlation")
    }

    fn create_handler(
        cycle_repo: Arc<dyn CycleRepository>,
        session_repo: Arc<dyn SessionRepository>,
        access: Arc<dyn AccessChecker>,
        publisher: Arc<dyn EventPublisher>,
    ) -> CreateCycleHandler {
        CreateCycleHandler::new(cycle_repo, session_repo, access, publisher)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Tests
    // ─────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn creates_cycle_for_valid_session() {
        let session = test_session();
        let session_id = *session.id();

        let cycle_repo = Arc::new(MockCycleRepository::new());
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo.clone(), session_repo, access, publisher);

        let cmd = CreateCycleCommand { session_id };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.cycle.session_id(), session_id);
        assert!(!result.cycle.is_branch());
    }

    #[tokio::test]
    async fn saves_cycle_to_repository() {
        let session = test_session();
        let session_id = *session.id();

        let cycle_repo = Arc::new(MockCycleRepository::new());
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo.clone(), session_repo, access, publisher);

        let cmd = CreateCycleCommand { session_id };
        handler.handle(cmd, test_metadata()).await.unwrap();

        let saved = cycle_repo.saved_cycles();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].session_id(), session_id);
    }

    #[tokio::test]
    async fn publishes_cycle_created_event() {
        let session = test_session();
        let session_id = *session.id();

        let cycle_repo = Arc::new(MockCycleRepository::new());
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, session_repo, access, publisher.clone());

        let cmd = CreateCycleCommand { session_id };
        let result = handler.handle(cmd, test_metadata()).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "cycle.created");
        assert_eq!(events[0].aggregate_id, result.cycle.id().to_string());
    }

    #[tokio::test]
    async fn fails_when_session_not_found() {
        let cycle_repo = Arc::new(MockCycleRepository::new());
        let session_repo = Arc::new(MockSessionRepository::new());
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo.clone(), session_repo, access, publisher.clone());

        let cmd = CreateCycleCommand {
            session_id: SessionId::new(),
        };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(result, Err(CreateCycleError::SessionNotFound(_))));
        assert!(cycle_repo.saved_cycles().is_empty());
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn fails_when_access_denied() {
        let session = test_session();
        let session_id = *session.id();

        let cycle_repo = Arc::new(MockCycleRepository::new());
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let access = Arc::new(MockAccessChecker::denied(AccessDeniedReason::CycleLimitReached {
            current: 10,
            max: 10,
        }));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo.clone(), session_repo, access, publisher.clone());

        let cmd = CreateCycleCommand { session_id };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(
            result,
            Err(CreateCycleError::AccessDenied(
                AccessDeniedReason::CycleLimitReached { .. }
            ))
        ));
        assert!(cycle_repo.saved_cycles().is_empty());
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn does_not_publish_event_on_save_failure() {
        let session = test_session();
        let session_id = *session.id();

        let cycle_repo = Arc::new(MockCycleRepository::failing());
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, session_repo, access, publisher.clone());

        let cmd = CreateCycleCommand { session_id };
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_err());
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn includes_correlation_id_in_event() {
        let session = test_session();
        let session_id = *session.id();

        let cycle_repo = Arc::new(MockCycleRepository::new());
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, session_repo, access, publisher.clone());

        let cmd = CreateCycleCommand { session_id };
        handler.handle(cmd, test_metadata()).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(
            events[0].metadata.correlation_id,
            Some("test-correlation".to_string())
        );
    }

    #[tokio::test]
    async fn event_has_correct_session_id() {
        let session = test_session();
        let session_id = *session.id();

        let cycle_repo = Arc::new(MockCycleRepository::new());
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = create_handler(cycle_repo, session_repo, access, publisher.clone());

        let cmd = CreateCycleCommand { session_id };
        let result = handler.handle(cmd, test_metadata()).await.unwrap();

        assert_eq!(result.event.session_id, session_id);
        assert!(result.event.parent_cycle_id.is_none());
    }
}
