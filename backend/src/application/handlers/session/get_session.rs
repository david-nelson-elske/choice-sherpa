//! GetSessionHandler - Query handler for retrieving session details.

use std::sync::Arc;

use crate::domain::foundation::{SessionId, UserId};
use crate::domain::session::SessionError;
use crate::ports::{SessionReader, SessionView};

/// Query to get a session by ID.
#[derive(Debug, Clone)]
pub struct GetSessionQuery {
    pub session_id: SessionId,
    pub user_id: UserId,
}

/// Handler for retrieving session details.
pub struct GetSessionHandler {
    reader: Arc<dyn SessionReader>,
}

impl GetSessionHandler {
    pub fn new(reader: Arc<dyn SessionReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(&self, query: GetSessionQuery) -> Result<SessionView, SessionError> {
        // Fetch the session
        let session = self
            .reader
            .get_by_id(&query.session_id)
            .await?
            .ok_or_else(|| SessionError::not_found(query.session_id))?;

        // Authorization check - ensure user owns the session
        if session.user_id != query.user_id {
            return Err(SessionError::forbidden());
        }

        Ok(session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{DomainError, ErrorCode, SessionStatus, Timestamp};
    use crate::ports::{ListOptions, SessionList, SessionSummary};
    use async_trait::async_trait;

    struct MockSessionReader {
        session: Option<SessionView>,
    }

    impl MockSessionReader {
        fn with_session(session: SessionView) -> Self {
            Self {
                session: Some(session),
            }
        }

        fn empty() -> Self {
            Self { session: None }
        }
    }

    #[async_trait]
    impl SessionReader for MockSessionReader {
        async fn get_by_id(&self, _id: &SessionId) -> Result<Option<SessionView>, DomainError> {
            Ok(self.session.clone())
        }

        async fn list_by_user(
            &self,
            _user_id: &UserId,
            _options: &ListOptions,
        ) -> Result<SessionList, DomainError> {
            Ok(SessionList {
                items: vec![],
                total: 0,
                has_more: false,
            })
        }

        async fn search(
            &self,
            _user_id: &UserId,
            _query: &str,
            _options: &ListOptions,
        ) -> Result<SessionList, DomainError> {
            Ok(SessionList {
                items: vec![],
                total: 0,
                has_more: false,
            })
        }

        async fn count_by_status(
            &self,
            _user_id: &UserId,
            _status: SessionStatus,
        ) -> Result<u64, DomainError> {
            Ok(0)
        }
    }

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    fn test_session_view(user_id: UserId) -> SessionView {
        SessionView {
            id: SessionId::new(),
            user_id,
            title: "Test Session".to_string(),
            description: None,
            status: SessionStatus::Active,
            cycle_count: 0,
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        }
    }

    #[tokio::test]
    async fn returns_session_for_owner() {
        let user_id = test_user_id();
        let session = test_session_view(user_id.clone());
        let session_id = session.id;

        let reader = Arc::new(MockSessionReader::with_session(session));
        let handler = GetSessionHandler::new(reader);

        let query = GetSessionQuery {
            session_id,
            user_id,
        };

        let result = handler.handle(query).await;
        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.title, "Test Session");
    }

    #[tokio::test]
    async fn returns_not_found_when_session_does_not_exist() {
        let reader = Arc::new(MockSessionReader::empty());
        let handler = GetSessionHandler::new(reader);

        let query = GetSessionQuery {
            session_id: SessionId::new(),
            user_id: test_user_id(),
        };

        let result = handler.handle(query).await;
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[tokio::test]
    async fn returns_forbidden_when_user_is_not_owner() {
        let owner_id = test_user_id();
        let other_user_id = UserId::new("other-user").unwrap();
        let session = test_session_view(owner_id);
        let session_id = session.id;

        let reader = Arc::new(MockSessionReader::with_session(session));
        let handler = GetSessionHandler::new(reader);

        let query = GetSessionQuery {
            session_id,
            user_id: other_user_id,
        };

        let result = handler.handle(query).await;
        assert!(matches!(result, Err(SessionError::Forbidden)));
    }
}
