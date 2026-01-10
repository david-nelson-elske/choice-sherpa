//! CheckAccessHandler - Query handler for checking user access.

use std::sync::Arc;

use crate::domain::foundation::UserId;
use crate::domain::membership::MembershipError;
use crate::ports::MembershipReader;

/// Query to check if a user has access.
#[derive(Debug, Clone)]
pub struct CheckAccessQuery {
    pub user_id: UserId,
}

/// Result of access check.
#[derive(Debug, Clone)]
pub struct CheckAccessResult {
    /// Whether the user has access.
    pub has_access: bool,
}

/// Handler for checking user access.
///
/// This is the most frequently called query and should be highly optimized.
/// The reader implementation may use caching for performance.
pub struct CheckAccessHandler {
    reader: Arc<dyn MembershipReader>,
}

impl CheckAccessHandler {
    pub fn new(reader: Arc<dyn MembershipReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(
        &self,
        query: CheckAccessQuery,
    ) -> Result<CheckAccessResult, MembershipError> {
        let has_access = self
            .reader
            .check_access(&query.user_id)
            .await
            .map_err(|e| MembershipError::infrastructure(e.to_string()))?;

        Ok(CheckAccessResult { has_access })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{DomainError, ErrorCode, MembershipId, Timestamp};
    use crate::domain::membership::{MembershipStatus, MembershipTier};
    use crate::ports::{MembershipStatistics, MembershipSummary, MembershipView};
    use async_trait::async_trait;

    // ════════════════════════════════════════════════════════════════════════════
    // Mock Implementation
    // ════════════════════════════════════════════════════════════════════════════

    struct MockMembershipReader {
        access_map: std::collections::HashMap<String, bool>,
        fail_read: bool,
    }

    impl MockMembershipReader {
        fn with_access(user_id: &UserId, has_access: bool) -> Self {
            let mut access_map = std::collections::HashMap::new();
            access_map.insert(user_id.to_string(), has_access);
            Self {
                access_map,
                fail_read: false,
            }
        }

        fn no_membership() -> Self {
            Self {
                access_map: std::collections::HashMap::new(),
                fail_read: false,
            }
        }

        fn failing() -> Self {
            Self {
                access_map: std::collections::HashMap::new(),
                fail_read: true,
            }
        }
    }

    #[async_trait]
    impl MembershipReader for MockMembershipReader {
        async fn get_by_user(&self, _user_id: &UserId) -> Result<Option<MembershipView>, DomainError> {
            Ok(None)
        }

        async fn check_access(&self, user_id: &UserId) -> Result<bool, DomainError> {
            if self.fail_read {
                return Err(DomainError::new(ErrorCode::DatabaseError, "Simulated read failure"));
            }
            Ok(self.access_map.get(&user_id.to_string()).copied().unwrap_or(false))
        }

        async fn get_tier(&self, _user_id: &UserId) -> Result<Option<MembershipTier>, DomainError> {
            Ok(None)
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

    // ════════════════════════════════════════════════════════════════════════════
    // Success Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn returns_true_when_user_has_access() {
        let user_id = test_user_id();
        let reader = Arc::new(MockMembershipReader::with_access(&user_id, true));

        let handler = CheckAccessHandler::new(reader);
        let query = CheckAccessQuery { user_id };

        let result = handler.handle(query).await;
        assert!(result.is_ok());
        assert!(result.unwrap().has_access);
    }

    #[tokio::test]
    async fn returns_false_when_user_has_no_access() {
        let user_id = test_user_id();
        let reader = Arc::new(MockMembershipReader::with_access(&user_id, false));

        let handler = CheckAccessHandler::new(reader);
        let query = CheckAccessQuery { user_id };

        let result = handler.handle(query).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().has_access);
    }

    #[tokio::test]
    async fn returns_false_when_no_membership() {
        let reader = Arc::new(MockMembershipReader::no_membership());

        let handler = CheckAccessHandler::new(reader);
        let query = CheckAccessQuery {
            user_id: test_user_id(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().has_access);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Failure Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn fails_when_reader_fails() {
        let reader = Arc::new(MockMembershipReader::failing());

        let handler = CheckAccessHandler::new(reader);
        let query = CheckAccessQuery {
            user_id: test_user_id(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_err());
    }
}
