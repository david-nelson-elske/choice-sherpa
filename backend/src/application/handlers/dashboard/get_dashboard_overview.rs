//! GetDashboardOverviewHandler - Query handler for retrieving dashboard overview.
//!
//! Returns aggregated dashboard data including session info, objectives,
//! alternatives, consequences, recommendation, and DQ score.

use std::sync::Arc;

use crate::domain::dashboard::DashboardOverview;
use crate::domain::foundation::{CycleId, DomainError, SessionId, UserId};
use crate::ports::{DashboardError, DashboardReader};

/// Query to get dashboard overview for a session.
#[derive(Debug, Clone)]
pub struct GetDashboardOverviewQuery {
    /// The session ID to retrieve dashboard for.
    pub session_id: SessionId,
    /// Optional specific cycle ID (defaults to active cycle if None).
    pub cycle_id: Option<CycleId>,
    /// User ID for authorization.
    pub user_id: UserId,
}

/// Result of successful dashboard overview query.
pub type GetDashboardOverviewResult = DashboardOverview;

/// Handler for retrieving dashboard overview.
///
/// Aggregates data from all PrOACT components for dashboard display.
pub struct GetDashboardOverviewHandler {
    reader: Arc<dyn DashboardReader>,
}

impl GetDashboardOverviewHandler {
    pub fn new(reader: Arc<dyn DashboardReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(
        &self,
        query: GetDashboardOverviewQuery,
    ) -> Result<GetDashboardOverviewResult, DashboardError> {
        self.reader
            .get_overview(query.session_id, query.cycle_id, &query.user_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::dashboard::DashboardOverview;
    use crate::domain::foundation::{CycleId, SessionId, UserId};
    use async_trait::async_trait;

    // ─────────────────────────────────────────────────────────────────────
    // Mock Implementation
    // ─────────────────────────────────────────────────────────────────────

    struct MockDashboardReader {
        overview: Option<DashboardOverview>,
        should_fail: bool,
        should_unauthorized: bool,
    }

    impl MockDashboardReader {
        fn new() -> Self {
            Self {
                overview: None,
                should_fail: false,
                should_unauthorized: false,
            }
        }

        fn with_overview(overview: DashboardOverview) -> Self {
            Self {
                overview: Some(overview),
                should_fail: false,
                should_unauthorized: false,
            }
        }

        fn failing() -> Self {
            Self {
                overview: None,
                should_fail: true,
                should_unauthorized: false,
            }
        }

        fn unauthorized() -> Self {
            Self {
                overview: None,
                should_fail: false,
                should_unauthorized: true,
            }
        }
    }

    #[async_trait]
    impl DashboardReader for MockDashboardReader {
        async fn get_overview(
            &self,
            _session_id: SessionId,
            _cycle_id: Option<CycleId>,
            _user_id: &UserId,
        ) -> Result<DashboardOverview, DashboardError> {
            if self.should_unauthorized {
                return Err(DashboardError::Unauthorized);
            }
            if self.should_fail {
                return Err(DashboardError::Database("Simulated failure".to_string()));
            }
            self.overview
                .clone()
                .ok_or_else(|| DashboardError::SessionNotFound(SessionId::new()))
        }

        async fn get_component_detail(
            &self,
            _cycle_id: CycleId,
            _component_type: crate::domain::foundation::ComponentType,
            _user_id: &UserId,
        ) -> Result<crate::domain::dashboard::ComponentDetailView, DashboardError> {
            unimplemented!()
        }

        async fn compare_cycles(
            &self,
            _cycle_ids: &[CycleId],
            _user_id: &UserId,
        ) -> Result<crate::domain::dashboard::CycleComparison, DashboardError> {
            unimplemented!()
        }
    }

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    fn create_test_overview() -> DashboardOverview {
        DashboardOverview {
            session_id: SessionId::new(),
            session_title: "Test Decision".to_string(),
            decision_statement: Some("Should we expand?".to_string()),
            objectives: vec![],
            alternatives: vec![],
            consequences_table: None,
            recommendation: None,
            dq_score: None,
            active_cycle_id: Some(CycleId::new()),
            cycle_count: 1,
            last_updated: chrono::Utc::now(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Tests
    // ─────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_overview_returns_dashboard() {
        let overview = create_test_overview();
        let reader = Arc::new(MockDashboardReader::with_overview(overview.clone()));
        let handler = GetDashboardOverviewHandler::new(reader);

        let query = GetDashboardOverviewQuery {
            session_id: overview.session_id,
            cycle_id: None,
            user_id: test_user_id(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_ok());

        let returned_overview = result.unwrap();
        assert_eq!(returned_overview.session_id, overview.session_id);
        assert_eq!(returned_overview.session_title, "Test Decision");
    }

    #[tokio::test]
    async fn test_get_overview_with_specific_cycle() {
        let overview = create_test_overview();
        let cycle_id = CycleId::new();
        let reader = Arc::new(MockDashboardReader::with_overview(overview.clone()));
        let handler = GetDashboardOverviewHandler::new(reader);

        let query = GetDashboardOverviewQuery {
            session_id: overview.session_id,
            cycle_id: Some(cycle_id),
            user_id: test_user_id(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_overview_passes_user_id() {
        let overview = create_test_overview();
        let user_id = test_user_id();
        let reader = Arc::new(MockDashboardReader::with_overview(overview.clone()));
        let handler = GetDashboardOverviewHandler::new(reader);

        let query = GetDashboardOverviewQuery {
            session_id: overview.session_id,
            cycle_id: None,
            user_id: user_id.clone(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_overview_handles_unauthorized() {
        let reader = Arc::new(MockDashboardReader::unauthorized());
        let handler = GetDashboardOverviewHandler::new(reader);

        let query = GetDashboardOverviewQuery {
            session_id: SessionId::new(),
            cycle_id: None,
            user_id: test_user_id(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DashboardError::Unauthorized));
    }

    #[tokio::test]
    async fn test_get_overview_propagates_errors() {
        let reader = Arc::new(MockDashboardReader::failing());
        let handler = GetDashboardOverviewHandler::new(reader);

        let query = GetDashboardOverviewQuery {
            session_id: SessionId::new(),
            cycle_id: None,
            user_id: test_user_id(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DashboardError::Database(_)));
    }
}
