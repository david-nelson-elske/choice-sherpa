//! CreateSessionHandler - Command handler for creating new sessions.

use std::sync::Arc;

use crate::domain::foundation::{
    CommandMetadata, EventId, SerializableDomainEvent, SessionId, UserId,
};
use crate::domain::session::{Session, SessionCreated, SessionError};
use crate::ports::{AccessChecker, AccessResult, EventPublisher, SessionRepository};

/// Command to create a new session.
#[derive(Debug, Clone)]
pub struct CreateSessionCommand {
    pub user_id: UserId,
    pub title: String,
    pub description: Option<String>,
}

/// Result of successful session creation.
#[derive(Debug, Clone)]
pub struct CreateSessionResult {
    pub session: Session,
    pub event: SessionCreated,
}

/// Handler for creating sessions.
pub struct CreateSessionHandler {
    repository: Arc<dyn SessionRepository>,
    access_checker: Arc<dyn AccessChecker>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl CreateSessionHandler {
    pub fn new(
        repository: Arc<dyn SessionRepository>,
        access_checker: Arc<dyn AccessChecker>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            repository,
            access_checker,
            event_publisher,
        }
    }

    pub async fn handle(
        &self,
        cmd: CreateSessionCommand,
        metadata: CommandMetadata,
    ) -> Result<CreateSessionResult, SessionError> {
        // 1. Check access (membership-based limits)
        match self.access_checker.can_create_session(&cmd.user_id).await? {
            AccessResult::Allowed => {}
            AccessResult::Denied(reason) => {
                return Err(SessionError::access_denied(reason));
            }
        }

        // 2. Create session aggregate
        let session_id = SessionId::new();
        let mut session = Session::new(session_id, cmd.user_id.clone(), cmd.title.clone())?;

        if let Some(description) = &cmd.description {
            session.update_description(Some(description.clone()))?;
        }

        // 3. Persist session
        self.repository.save(&session).await?;

        // 4. Create and publish event
        let event = SessionCreated {
            event_id: EventId::new(),
            session_id: *session.id(),
            user_id: cmd.user_id,
            title: cmd.title,
            description: cmd.description,
            created_at: *session.created_at(),
        };

        let envelope = event
            .to_envelope()
            .with_correlation_id(metadata.correlation_id())
            .with_user_id(metadata.user_id.to_string());

        self.event_publisher.publish(envelope).await?;

        Ok(CreateSessionResult { session, event })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{DomainError, ErrorCode, EventEnvelope};
    use crate::domain::membership::TierLimits;
    use crate::ports::{AccessDeniedReason, UsageStats};
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct MockSessionRepository {
        saved_sessions: Mutex<Vec<Session>>,
        fail_save: bool,
    }

    impl MockSessionRepository {
        fn new() -> Self {
            Self {
                saved_sessions: Mutex::new(Vec::new()),
                fail_save: false,
            }
        }

        fn failing() -> Self {
            Self {
                saved_sessions: Mutex::new(Vec::new()),
                fail_save: true,
            }
        }

        fn saved_sessions(&self) -> Vec<Session> {
            self.saved_sessions.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl SessionRepository for MockSessionRepository {
        async fn save(&self, session: &Session) -> Result<(), DomainError> {
            if self.fail_save {
                return Err(DomainError::new(
                    ErrorCode::DatabaseError,
                    "Simulated save failure",
                ));
            }
            self.saved_sessions.lock().unwrap().push(session.clone());
            Ok(())
        }

        async fn update(&self, _session: &Session) -> Result<(), DomainError> {
            Ok(())
        }

        async fn find_by_id(&self, _id: &SessionId) -> Result<Option<Session>, DomainError> {
            Ok(None)
        }

        async fn exists(&self, _id: &SessionId) -> Result<bool, DomainError> {
            Ok(false)
        }

        async fn find_by_user_id(&self, _user_id: &UserId) -> Result<Vec<Session>, DomainError> {
            Ok(vec![])
        }

        async fn count_active_by_user(&self, _user_id: &UserId) -> Result<u32, DomainError> {
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
        async fn can_create_session(&self, _user_id: &UserId) -> Result<AccessResult, DomainError> {
            Ok(self.result.clone())
        }

        async fn can_create_cycle(
            &self,
            _user_id: &UserId,
            _session_id: &SessionId,
        ) -> Result<AccessResult, DomainError> {
            Ok(AccessResult::Allowed)
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

        fn failing() -> Self {
            Self {
                published_events: Mutex::new(Vec::new()),
                fail_publish: true,
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

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    fn test_metadata() -> CommandMetadata {
        CommandMetadata::new(test_user_id()).with_correlation_id("test-correlation")
    }

    #[tokio::test]
    async fn creates_session_with_valid_input() {
        let repo = Arc::new(MockSessionRepository::new());
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreateSessionHandler::new(repo.clone(), access, publisher);

        let cmd = CreateSessionCommand {
            user_id: test_user_id(),
            title: "Test Decision".to_string(),
            description: None,
        };

        let result = handler.handle(cmd, test_metadata()).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.session.title(), "Test Decision");
    }

    #[tokio::test]
    async fn publishes_session_created_event() {
        let repo = Arc::new(MockSessionRepository::new());
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreateSessionHandler::new(repo, access, publisher.clone());

        let cmd = CreateSessionCommand {
            user_id: test_user_id(),
            title: "Event Test".to_string(),
            description: None,
        };

        let result = handler.handle(cmd, test_metadata()).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "session.created.v1");
        assert_eq!(events[0].aggregate_id, result.session.id().to_string());
    }

    #[tokio::test]
    async fn fails_when_access_denied() {
        let repo = Arc::new(MockSessionRepository::new());
        let access = Arc::new(MockAccessChecker::denied(AccessDeniedReason::NoMembership));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreateSessionHandler::new(repo.clone(), access, publisher.clone());

        let cmd = CreateSessionCommand {
            user_id: test_user_id(),
            title: "Should Fail".to_string(),
            description: None,
        };

        let result = handler.handle(cmd, test_metadata()).await;
        assert!(matches!(
            result,
            Err(SessionError::AccessDenied(AccessDeniedReason::NoMembership))
        ));
        assert!(repo.saved_sessions().is_empty());
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn fails_with_empty_title() {
        let repo = Arc::new(MockSessionRepository::new());
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreateSessionHandler::new(repo.clone(), access, publisher.clone());

        let cmd = CreateSessionCommand {
            user_id: test_user_id(),
            title: "".to_string(),
            description: None,
        };

        let result = handler.handle(cmd, test_metadata()).await;
        assert!(matches!(result, Err(SessionError::ValidationFailed { .. })));
    }

    #[tokio::test]
    async fn includes_correlation_id_in_event() {
        let repo = Arc::new(MockSessionRepository::new());
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreateSessionHandler::new(repo, access, publisher.clone());

        let cmd = CreateSessionCommand {
            user_id: test_user_id(),
            title: "Correlation Test".to_string(),
            description: None,
        };

        handler.handle(cmd, test_metadata()).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(
            events[0].metadata.correlation_id,
            Some("test-correlation".to_string())
        );
    }

    #[tokio::test]
    async fn includes_description_when_provided() {
        let repo = Arc::new(MockSessionRepository::new());
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreateSessionHandler::new(repo.clone(), access, publisher);

        let cmd = CreateSessionCommand {
            user_id: test_user_id(),
            title: "With Description".to_string(),
            description: Some("Test description".to_string()),
        };

        let result = handler.handle(cmd, test_metadata()).await.unwrap();
        assert_eq!(result.session.description(), Some("Test description"));
        assert_eq!(result.event.description, Some("Test description".to_string()));
    }

    #[tokio::test]
    async fn does_not_publish_event_on_save_failure() {
        let repo = Arc::new(MockSessionRepository::failing());
        let access = Arc::new(MockAccessChecker::allowed());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreateSessionHandler::new(repo, access, publisher.clone());

        let cmd = CreateSessionCommand {
            user_id: test_user_id(),
            title: "Should Fail Save".to_string(),
            description: None,
        };

        let result = handler.handle(cmd, test_metadata()).await;
        assert!(result.is_err());
        assert!(publisher.published_events().is_empty());
    }
}
