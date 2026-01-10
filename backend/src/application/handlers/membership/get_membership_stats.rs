//! GetMembershipStatsHandler - Query handler for admin statistics.

use std::sync::Arc;

use crate::domain::membership::MembershipError;
use crate::ports::{MembershipReader, MembershipStatistics};

/// Query to get membership statistics.
///
/// This is an admin-only query for dashboard displays.
#[derive(Debug, Clone)]
pub struct GetMembershipStatsQuery;

/// Result type for statistics query.
pub type GetMembershipStatsResult = MembershipStatistics;

/// Handler for retrieving membership statistics.
///
/// Returns aggregate statistics about all memberships for admin dashboard.
pub struct GetMembershipStatsHandler {
    reader: Arc<dyn MembershipReader>,
}

impl GetMembershipStatsHandler {
    pub fn new(reader: Arc<dyn MembershipReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(
        &self,
        _query: GetMembershipStatsQuery,
    ) -> Result<GetMembershipStatsResult, MembershipError> {
        self.reader
            .get_statistics()
            .await
            .map_err(|e| MembershipError::infrastructure(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{DomainError, ErrorCode, UserId};
    use crate::domain::membership::MembershipTier;
    use crate::ports::{MembershipSummary, MembershipView, StatusCounts, TierCounts};
    use async_trait::async_trait;

    // ════════════════════════════════════════════════════════════════════════════
    // Mock Implementation
    // ════════════════════════════════════════════════════════════════════════════

    struct MockMembershipReader {
        stats: MembershipStatistics,
        fail_read: bool,
    }

    impl MockMembershipReader {
        fn with_stats(stats: MembershipStatistics) -> Self {
            Self {
                stats,
                fail_read: false,
            }
        }

        fn failing() -> Self {
            Self {
                stats: MembershipStatistics::default(),
                fail_read: true,
            }
        }
    }

    #[async_trait]
    impl MembershipReader for MockMembershipReader {
        async fn get_by_user(&self, _user_id: &UserId) -> Result<Option<MembershipView>, DomainError> {
            Ok(None)
        }

        async fn check_access(&self, _user_id: &UserId) -> Result<bool, DomainError> {
            Ok(false)
        }

        async fn get_tier(&self, _user_id: &UserId) -> Result<Option<MembershipTier>, DomainError> {
            Ok(None)
        }

        async fn list_expiring(&self, _days: u32) -> Result<Vec<MembershipSummary>, DomainError> {
            Ok(vec![])
        }

        async fn get_statistics(&self) -> Result<MembershipStatistics, DomainError> {
            if self.fail_read {
                return Err(DomainError::new(ErrorCode::DatabaseError, "Simulated read failure"));
            }
            Ok(self.stats.clone())
        }
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Test Helpers
    // ════════════════════════════════════════════════════════════════════════════

    fn sample_stats() -> MembershipStatistics {
        MembershipStatistics {
            total_count: 150,
            active_count: 120,
            by_tier: TierCounts {
                free: 50,
                monthly: 70,
                annual: 30,
            },
            by_status: StatusCounts {
                pending: 5,
                active: 120,
                past_due: 10,
                cancelled: 10,
                expired: 5,
            },
            monthly_recurring_revenue_cents: 1_500_000, // $15,000 MRR
        }
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Success Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn returns_statistics() {
        let stats = sample_stats();
        let reader = Arc::new(MockMembershipReader::with_stats(stats.clone()));

        let handler = GetMembershipStatsHandler::new(reader);
        let query = GetMembershipStatsQuery;

        let result = handler.handle(query).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.total_count, 150);
        assert_eq!(result.active_count, 120);
        assert_eq!(result.by_tier.free, 50);
        assert_eq!(result.by_tier.monthly, 70);
        assert_eq!(result.by_tier.annual, 30);
    }

    #[tokio::test]
    async fn returns_empty_stats_when_no_memberships() {
        let reader = Arc::new(MockMembershipReader::with_stats(MembershipStatistics::default()));

        let handler = GetMembershipStatsHandler::new(reader);
        let query = GetMembershipStatsQuery;

        let result = handler.handle(query).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.total_count, 0);
        assert_eq!(result.active_count, 0);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Failure Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn fails_when_reader_fails() {
        let reader = Arc::new(MockMembershipReader::failing());

        let handler = GetMembershipStatsHandler::new(reader);
        let query = GetMembershipStatsQuery;

        let result = handler.handle(query).await;
        assert!(result.is_err());
    }
}
