//! GetMembershipHandler - Query handler for retrieving membership details.

use std::sync::Arc;

use crate::domain::foundation::UserId;
use crate::domain::membership::MembershipError;
use crate::ports::{MembershipReader, MembershipView};

/// Query to get a user's membership.
#[derive(Debug, Clone)]
pub struct GetMembershipQuery {
    pub user_id: UserId,
}

/// Result of successful membership query.
pub type GetMembershipResult = Option<MembershipView>;

/// Handler for retrieving membership details.
///
/// Returns the full membership view for UI display,
/// or `None` if the user has no membership.
pub struct GetMembershipHandler {
    reader: Arc<dyn MembershipReader>,
}

impl GetMembershipHandler {
    pub fn new(reader: Arc<dyn MembershipReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(
        &self,
        query: GetMembershipQuery,
    ) -> Result<GetMembershipResult, MembershipError> {
        self.reader
            .get_by_user(&query.user_id)
            .await
            .map_err(|e| MembershipError::infrastructure(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{DomainError, ErrorCode, MembershipId, Timestamp};
    use crate::domain::membership::{MembershipStatus, MembershipTier};
    use crate::ports::{MembershipStatistics, MembershipSummary};
    use async_trait::async_trait;

    // ════════════════════════════════════════════════════════════════════════════
    // Mock Implementation
    // ════════════════════════════════════════════════════════════════════════════

    struct MockMembershipReader {
        views: Vec<MembershipView>,
        fail_read: bool,
    }

    impl MockMembershipReader {
        fn new() -> Self {
            Self {
                views: Vec::new(),
                fail_read: false,
            }
        }

        fn with_membership(view: MembershipView) -> Self {
            Self {
                views: vec![view],
                fail_read: false,
            }
        }

        fn failing() -> Self {
            Self {
                views: Vec::new(),
                fail_read: true,
            }
        }
    }

    #[async_trait]
    impl MembershipReader for MockMembershipReader {
        async fn get_by_user(&self, user_id: &UserId) -> Result<Option<MembershipView>, DomainError> {
            if self.fail_read {
                return Err(DomainError::new(ErrorCode::DatabaseError, "Simulated read failure"));
            }
            Ok(self.views.iter().find(|v| &v.user_id == user_id).cloned())
        }

        async fn check_access(&self, user_id: &UserId) -> Result<bool, DomainError> {
            if self.fail_read {
                return Err(DomainError::new(ErrorCode::DatabaseError, "Simulated read failure"));
            }
            Ok(self.views.iter().any(|v| &v.user_id == user_id && v.has_access))
        }

        async fn get_tier(&self, user_id: &UserId) -> Result<Option<MembershipTier>, DomainError> {
            if self.fail_read {
                return Err(DomainError::new(ErrorCode::DatabaseError, "Simulated read failure"));
            }
            Ok(self.views.iter().find(|v| &v.user_id == user_id).map(|v| v.tier))
        }

        async fn list_expiring(&self, _days: u32) -> Result<Vec<MembershipSummary>, DomainError> {
            Ok(vec![])
        }

        async fn get_statistics(&self) -> Result<MembershipStatistics, DomainError> {
            Ok(MembershipStatistics::default())
        }
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Test Helpers
    // ════════════════════════════════════════════════════════════════════════════

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    fn test_membership_view(user_id: UserId) -> MembershipView {
        MembershipView {
            id: MembershipId::new(),
            user_id,
            tier: MembershipTier::Annual,
            status: MembershipStatus::Active,
            has_access: true,
            days_remaining: 300,
            period_end: Timestamp::now().add_days(300),
            promo_code: Some("WORKSHOP2026-A7K9M3".to_string()),
            created_at: Timestamp::now(),
        }
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Success Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn returns_membership_when_exists() {
        let user_id = test_user_id();
        let view = test_membership_view(user_id.clone());
        let reader = Arc::new(MockMembershipReader::with_membership(view.clone()));

        let handler = GetMembershipHandler::new(reader);
        let query = GetMembershipQuery { user_id };

        let result = handler.handle(query).await;
        assert!(result.is_ok());

        let membership = result.unwrap();
        assert!(membership.is_some());

        let membership = membership.unwrap();
        assert_eq!(membership.tier, MembershipTier::Annual);
        assert!(membership.has_access);
    }

    #[tokio::test]
    async fn returns_none_when_no_membership() {
        let reader = Arc::new(MockMembershipReader::new());

        let handler = GetMembershipHandler::new(reader);
        let query = GetMembershipQuery {
            user_id: test_user_id(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Failure Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn fails_when_reader_fails() {
        let reader = Arc::new(MockMembershipReader::failing());

        let handler = GetMembershipHandler::new(reader);
        let query = GetMembershipQuery {
            user_id: test_user_id(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_err());
    }
}
