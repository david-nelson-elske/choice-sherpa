//! SessionCycleTracker - Event handler for CycleCreated events.
//!
//! Listens for cycles created by the Cycle module and updates the Session
//! to maintain its cycle list. This enables the Session to know about its
//! cycles without direct coupling to the Cycle module.

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::foundation::{
    CycleId, DomainError, ErrorCode, EventEnvelope, EventId, SerializableDomainEvent, SessionId,
    Timestamp,
};
use crate::domain::session::CycleAddedToSession;
use crate::ports::{EventHandler, EventPublisher, SessionRepository};

/// External CycleCreated event from the Cycle module.
///
/// This is the expected payload format for `cycle.created` events.
/// The SessionCycleTracker deserializes incoming events into this struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleCreated {
    /// The cycle that was created.
    pub cycle_id: CycleId,

    /// The session this cycle belongs to.
    pub session_id: SessionId,

    /// Parent cycle if this is a branch.
    pub parent_cycle_id: Option<CycleId>,

    /// When the cycle was created.
    pub created_at: Timestamp,
}

/// Handles CycleCreated events to update session's cycle list.
///
/// When a new cycle is created, this handler:
/// 1. Loads the associated session
/// 2. Adds the cycle ID to the session's cycle list
/// 3. Publishes a CycleAddedToSession event
///
/// This maintains eventual consistency between Cycle and Session modules.
pub struct SessionCycleTracker {
    session_repo: Arc<dyn SessionRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl SessionCycleTracker {
    /// Creates a new SessionCycleTracker.
    pub fn new(
        session_repo: Arc<dyn SessionRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            session_repo,
            event_publisher,
        }
    }
}

#[async_trait]
impl EventHandler for SessionCycleTracker {
    async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError> {
        // Parse cycle created event
        let cycle_created: CycleCreated = serde_json::from_value(event.payload.clone())
            .map_err(|e| DomainError::new(ErrorCode::ValidationFailed, e.to_string()))?;

        // Load session
        let mut session = self
            .session_repo
            .find_by_id(&cycle_created.session_id)
            .await?
            .ok_or_else(|| {
                DomainError::new(
                    ErrorCode::SessionNotFound,
                    format!("Session not found: {}", cycle_created.session_id),
                )
            })?;

        // Add cycle to session (returns whether it's the root cycle)
        let is_root = session.add_cycle(cycle_created.cycle_id)?;

        // Persist
        self.session_repo.update(&session).await?;

        // Publish session event
        let session_event = CycleAddedToSession {
            event_id: EventId::new(),
            session_id: cycle_created.session_id,
            cycle_id: cycle_created.cycle_id,
            is_root_cycle: is_root,
            added_at: Timestamp::now(),
        };

        let envelope = session_event
            .to_envelope()
            .with_causation_id(event.event_id.as_str());

        self.event_publisher.publish(envelope).await?;

        Ok(())
    }

    fn name(&self) -> &'static str {
        "SessionCycleTracker"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::UserId;
    use crate::domain::session::Session;
    use serde_json::json;
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

    fn cycle_created_event(session_id: SessionId, cycle_id: CycleId) -> EventEnvelope {
        let created_at = Timestamp::now();
        EventEnvelope {
            event_id: EventId::from_string("evt-cycle-1"),
            event_type: "cycle.created".to_string(),
            aggregate_id: cycle_id.to_string(),
            aggregate_type: "Cycle".to_string(),
            occurred_at: Timestamp::now(),
            payload: json!({
                "cycle_id": cycle_id.to_string(),
                "session_id": session_id.to_string(),
                "parent_cycle_id": null,
                "created_at": serde_json::to_value(created_at).unwrap(),
            }),
            metadata: Default::default(),
        }
    }

    #[tokio::test]
    async fn adds_cycle_to_session() {
        let session = test_session();
        let session_id = *session.id();
        let repo = Arc::new(MockSessionRepository::with_session(session));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = SessionCycleTracker::new(repo.clone(), publisher);

        let cycle_id = CycleId::new();
        let event = cycle_created_event(session_id, cycle_id);

        let result = handler.handle(event).await;
        assert!(result.is_ok());

        // Verify session was updated
        let updated_session = repo.get_session(&session_id).unwrap();
        assert!(updated_session.cycle_ids().contains(&cycle_id));
    }

    #[tokio::test]
    async fn publishes_cycle_added_to_session_event() {
        let session = test_session();
        let session_id = *session.id();
        let repo = Arc::new(MockSessionRepository::with_session(session));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = SessionCycleTracker::new(repo, publisher.clone());

        let cycle_id = CycleId::new();
        let event = cycle_created_event(session_id, cycle_id);

        handler.handle(event).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "session.cycle_added");
        assert_eq!(events[0].aggregate_id, session_id.to_string());
    }

    #[tokio::test]
    async fn first_cycle_is_marked_as_root() {
        let session = test_session();
        let session_id = *session.id();
        let repo = Arc::new(MockSessionRepository::with_session(session));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = SessionCycleTracker::new(repo, publisher.clone());

        let cycle_id = CycleId::new();
        let event = cycle_created_event(session_id, cycle_id);

        handler.handle(event).await.unwrap();

        let events = publisher.published_events();
        let payload: CycleAddedToSession =
            serde_json::from_value(events[0].payload.clone()).unwrap();
        assert!(payload.is_root_cycle);
    }

    #[tokio::test]
    async fn second_cycle_is_not_root() {
        // Session with one existing cycle
        let mut session = test_session();
        session.add_cycle(CycleId::new()).unwrap();
        let session_id = *session.id();

        let repo = Arc::new(MockSessionRepository::with_session(session));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = SessionCycleTracker::new(repo, publisher.clone());

        let cycle_id = CycleId::new();
        let event = cycle_created_event(session_id, cycle_id);

        handler.handle(event).await.unwrap();

        let events = publisher.published_events();
        let payload: CycleAddedToSession =
            serde_json::from_value(events[0].payload.clone()).unwrap();
        assert!(!payload.is_root_cycle);
    }

    #[tokio::test]
    async fn fails_when_session_not_found() {
        let repo = Arc::new(MockSessionRepository::new());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = SessionCycleTracker::new(repo, publisher.clone());

        let event = cycle_created_event(SessionId::new(), CycleId::new());

        let result = handler.handle(event).await;
        assert!(result.is_err());
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn includes_causation_id() {
        let session = test_session();
        let session_id = *session.id();
        let repo = Arc::new(MockSessionRepository::with_session(session));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = SessionCycleTracker::new(repo, publisher.clone());

        let cycle_id = CycleId::new();
        let mut event = cycle_created_event(session_id, cycle_id);
        event.event_id = EventId::from_string("original-event-id");

        handler.handle(event).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(
            events[0].metadata.causation_id,
            Some("original-event-id".to_string())
        );
    }

    #[tokio::test]
    async fn handler_name_is_correct() {
        let repo = Arc::new(MockSessionRepository::new());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = SessionCycleTracker::new(repo, publisher);
        assert_eq!(handler.name(), "SessionCycleTracker");
    }

    #[tokio::test]
    async fn duplicate_cycle_is_handled_idempotently() {
        let session = test_session();
        let session_id = *session.id();
        let repo = Arc::new(MockSessionRepository::with_session(session));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = SessionCycleTracker::new(repo.clone(), publisher.clone());

        let cycle_id = CycleId::new();
        let event = cycle_created_event(session_id, cycle_id);

        // Handle same event twice
        handler.handle(event.clone()).await.unwrap();
        handler.handle(event).await.unwrap();

        // Session should only have cycle once
        let updated_session = repo.get_session(&session_id).unwrap();
        assert_eq!(updated_session.cycle_ids().len(), 1);

        // Two events published (second isn't root)
        let events = publisher.published_events();
        assert_eq!(events.len(), 2);
    }
}
