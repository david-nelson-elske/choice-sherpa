//! ListUserSessionsHandler - Query handler for listing user's sessions.

use std::sync::Arc;

use crate::domain::foundation::{SessionStatus, UserId};
use crate::domain::session::SessionError;
use crate::ports::{ListOptions, SessionList, SessionReader};

/// Query to list sessions for a user.
#[derive(Debug, Clone)]
pub struct ListUserSessionsQuery {
    pub user_id: UserId,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub status: Option<SessionStatus>,
    pub include_archived: bool,
}

impl ListUserSessionsQuery {
    /// Create a simple query for all active sessions.
    pub fn all_active(user_id: UserId) -> Self {
        Self {
            user_id,
            page: None,
            per_page: None,
            status: None,
            include_archived: false,
        }
    }

    /// Create a paginated query.
    pub fn paginated(user_id: UserId, page: u32, per_page: u32) -> Self {
        Self {
            user_id,
            page: Some(page),
            per_page: Some(per_page),
            status: None,
            include_archived: false,
        }
    }

    /// Build ListOptions from the query.
    fn to_list_options(&self) -> ListOptions {
        let mut options = match (self.page, self.per_page) {
            (Some(page), Some(per_page)) => ListOptions::paginated(page, per_page),
            _ => ListOptions::default(),
        };

        if let Some(status) = self.status {
            options = options.with_status(status);
        }

        if self.include_archived {
            options = options.with_archived();
        }

        options
    }
}

/// Handler for listing user sessions.
pub struct ListUserSessionsHandler {
    reader: Arc<dyn SessionReader>,
}

impl ListUserSessionsHandler {
    pub fn new(reader: Arc<dyn SessionReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(&self, query: ListUserSessionsQuery) -> Result<SessionList, SessionError> {
        let options = query.to_list_options();
        let list = self.reader.list_by_user(&query.user_id, &options).await?;
        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{DomainError, SessionId, Timestamp};
    use crate::ports::{SessionSummary, SessionView};
    use async_trait::async_trait;

    struct MockSessionReader {
        sessions: Vec<SessionSummary>,
    }

    impl MockSessionReader {
        fn with_sessions(sessions: Vec<SessionSummary>) -> Self {
            Self { sessions }
        }

        fn empty() -> Self {
            Self {
                sessions: Vec::new(),
            }
        }
    }

    #[async_trait]
    impl SessionReader for MockSessionReader {
        async fn get_by_id(&self, _id: &SessionId) -> Result<Option<SessionView>, DomainError> {
            Ok(None)
        }

        async fn list_by_user(
            &self,
            _user_id: &UserId,
            options: &ListOptions,
        ) -> Result<SessionList, DomainError> {
            let total = self.sessions.len() as u64;
            let limit = options.limit.unwrap_or(u32::MAX) as usize;
            let offset = options.offset.unwrap_or(0) as usize;

            let items: Vec<SessionSummary> = self
                .sessions
                .iter()
                .skip(offset)
                .take(limit)
                .filter(|s| {
                    if let Some(status) = options.status {
                        s.status == status
                    } else {
                        true
                    }
                })
                .cloned()
                .collect();

            let has_more = offset + items.len() < total as usize;

            Ok(SessionList {
                items,
                total,
                has_more,
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

    fn test_session_summary(title: &str, status: SessionStatus) -> SessionSummary {
        SessionSummary {
            id: SessionId::new(),
            title: title.to_string(),
            status,
            cycle_count: 0,
            updated_at: Timestamp::now(),
        }
    }

    #[tokio::test]
    async fn returns_all_sessions_for_user() {
        let sessions = vec![
            test_session_summary("Session 1", SessionStatus::Active),
            test_session_summary("Session 2", SessionStatus::Active),
            test_session_summary("Session 3", SessionStatus::Archived),
        ];

        let reader = Arc::new(MockSessionReader::with_sessions(sessions));
        let handler = ListUserSessionsHandler::new(reader);

        let query = ListUserSessionsQuery::all_active(test_user_id());
        let result = handler.handle(query).await.unwrap();

        assert_eq!(result.total, 3);
        assert_eq!(result.items.len(), 3);
    }

    #[tokio::test]
    async fn returns_empty_list_when_no_sessions() {
        let reader = Arc::new(MockSessionReader::empty());
        let handler = ListUserSessionsHandler::new(reader);

        let query = ListUserSessionsQuery::all_active(test_user_id());
        let result = handler.handle(query).await.unwrap();

        assert_eq!(result.total, 0);
        assert!(result.items.is_empty());
    }

    #[tokio::test]
    async fn supports_pagination() {
        let sessions = vec![
            test_session_summary("Session 1", SessionStatus::Active),
            test_session_summary("Session 2", SessionStatus::Active),
            test_session_summary("Session 3", SessionStatus::Active),
        ];

        let reader = Arc::new(MockSessionReader::with_sessions(sessions));
        let handler = ListUserSessionsHandler::new(reader);

        // Get page 1 with 2 items per page
        let query = ListUserSessionsQuery::paginated(test_user_id(), 1, 2);
        let result = handler.handle(query).await.unwrap();

        assert_eq!(result.items.len(), 2);
        assert!(result.has_more);
    }

    #[tokio::test]
    async fn list_options_conversion_handles_pagination() {
        let query = ListUserSessionsQuery::paginated(test_user_id(), 2, 10);
        let options = query.to_list_options();

        assert_eq!(options.limit, Some(10));
        assert_eq!(options.offset, Some(10));
    }

    #[tokio::test]
    async fn list_options_conversion_handles_status_filter() {
        let mut query = ListUserSessionsQuery::all_active(test_user_id());
        query.status = Some(SessionStatus::Active);

        let options = query.to_list_options();
        assert_eq!(options.status, Some(SessionStatus::Active));
    }

    #[tokio::test]
    async fn list_options_conversion_handles_archived_flag() {
        let mut query = ListUserSessionsQuery::all_active(test_user_id());
        query.include_archived = true;

        let options = query.to_list_options();
        assert!(options.include_archived);
    }
}
