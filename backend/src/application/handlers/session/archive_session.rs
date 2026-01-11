//! ArchiveSessionHandler - Command handler for archiving sessions.

use std::sync::Arc;

use crate::domain::foundation::{
    CommandMetadata, EventId, SerializableDomainEvent, SessionId, Timestamp, UserId,
};
use crate::domain::session::{Session, SessionArchived, SessionError};
use crate::ports::{EventPublisher, SessionRepository};

/// Command to archive a session.
#[derive(Debug, Clone)]
pub struct ArchiveSessionCommand {
    pub session_id: SessionId,
    pub user_id: UserId,
}

/// Result of successful session archival.
#[derive(Debug, Clone)]
pub struct ArchiveSessionResult {
    pub session: Session,
    pub event: SessionArchived,
}

/// Handler for archiving sessions.
pub struct ArchiveSessionHandler {
    repository: Arc<dyn SessionRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl ArchiveSessionHandler {
    pub fn new(
        repository: Arc<dyn SessionRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            repository,
            event_publisher,
        }
    }

    pub async fn handle(
        &self,
        cmd: ArchiveSessionCommand,
        metadata: CommandMetadata,
    ) -> Result<ArchiveSessionResult, SessionError> {
        // 1. Load session
        let mut session = self
            .repository
            .find_by_id(&cmd.session_id)
            .await?
            .ok_or_else(|| SessionError::not_found(cmd.session_id))?;

        // 2. Authorize - user must be owner
        session.authorize(&cmd.user_id)?;

        // 3. Archive
        session.archive()?;

        // 4. Persist
        self.repository.update(&session).await?;

        // 5. Publish event
        let event = SessionArchived {
            event_id: EventId::new(),
            session_id: cmd.session_id,
            user_id: cmd.user_id,
            archived_at: Timestamp::now(),
        };

        let envelope = event
            .to_envelope()
            .with_correlation_id(metadata.correlation_id())
            .with_user_id(metadata.user_id.to_string());

        self.event_publisher.publish(envelope).await?;

        Ok(ArchiveSessionResult { session, event })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{DomainError, EventEnvelope, SessionStatus};
    use async_trait::async_trait;
    use std::sync::Mutex;

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

        fn get_session(&self, id: &SessionId) -> Option<Session> {
            self.sessions
                .lock()
                .unwrap()
                .iter()
                .find(|s| s.id() == id)
                .cloned()
        }
    }

    #[async_trait]
    impl SessionRepository for MockSessionRepository {
        async fn save(&self, session: &Session) -> Result<(), DomainError> {
            self.sessions.lock().unwrap().push(session.clone());
            Ok(())
        }

        async fn update(&self, session: &Session) -> Result<(), DomainError> {
            let mut sessions = self.sessions.lock().unwrap();
            if let Some(pos) = sessions.iter().position(|s| s.id() == session.id()) {
                sessions[pos] = session.clone();
            }
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

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    fn test_session() -> Session {
        Session::new(SessionId::new(), test_user_id(), "Test Session".to_string()).unwrap()
    }

    fn test_metadata() -> CommandMetadata {
        CommandMetadata::new(test_user_id()).with_correlation_id("test-correlation")
    }

    #[tokio::test]
    async fn archives_session_successfully() {
        let session = test_session();
        let session_id = *session.id();
        let repo = Arc::new(MockSessionRepository::with_session(session));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = ArchiveSessionHandler::new(repo.clone(), publisher);

        let cmd = ArchiveSessionCommand {
            session_id,
            user_id: test_user_id(),
        };

        let result = handler.handle(cmd, test_metadata()).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.session.status(), SessionStatus::Archived);
    }

    #[tokio::test]
    async fn publishes_session_archived_event() {
        let session = test_session();
        let session_id = *session.id();
        let repo = Arc::new(MockSessionRepository::with_session(session));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = ArchiveSessionHandler::new(repo, publisher.clone());

        let cmd = ArchiveSessionCommand {
            session_id,
            user_id: test_user_id(),
        };

        handler.handle(cmd, test_metadata()).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "session.archived.v1");
        assert_eq!(events[0].aggregate_id, session_id.to_string());
    }

    #[tokio::test]
    async fn fails_when_session_not_found() {
        let repo = Arc::new(MockSessionRepository::new());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = ArchiveSessionHandler::new(repo, publisher.clone());

        let cmd = ArchiveSessionCommand {
            session_id: SessionId::new(),
            user_id: test_user_id(),
        };

        let result = handler.handle(cmd, test_metadata()).await;
        assert!(matches!(result, Err(SessionError::NotFound(_))));
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn fails_when_not_owner() {
        let session = test_session();
        let session_id = *session.id();
        let repo = Arc::new(MockSessionRepository::with_session(session));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = ArchiveSessionHandler::new(repo, publisher.clone());

        let other_user = UserId::new("other-user").unwrap();
        let cmd = ArchiveSessionCommand {
            session_id,
            user_id: other_user.clone(),
        };

        let metadata = CommandMetadata::new(other_user);
        let result = handler.handle(cmd, metadata).await;
        assert!(matches!(result, Err(SessionError::Forbidden)));
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn fails_when_already_archived() {
        let mut session = test_session();
        session.archive().unwrap();
        let session_id = *session.id();
        let repo = Arc::new(MockSessionRepository::with_session(session));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = ArchiveSessionHandler::new(repo, publisher.clone());

        let cmd = ArchiveSessionCommand {
            session_id,
            user_id: test_user_id(),
        };

        let result = handler.handle(cmd, test_metadata()).await;
        assert!(matches!(result, Err(SessionError::InvalidState(_))));
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn includes_correlation_id_in_event() {
        let session = test_session();
        let session_id = *session.id();
        let repo = Arc::new(MockSessionRepository::with_session(session));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = ArchiveSessionHandler::new(repo, publisher.clone());

        let cmd = ArchiveSessionCommand {
            session_id,
            user_id: test_user_id(),
        };

        handler.handle(cmd, test_metadata()).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(
            events[0].metadata.correlation_id,
            Some("test-correlation".to_string())
        );
    }
}
