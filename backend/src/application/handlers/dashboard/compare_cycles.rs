//! CompareCyclesHandler - Query handler for comparing multiple cycles.
//!
//! Returns side-by-side comparison of cycles with differences highlighted.

use std::sync::Arc;

use crate::domain::dashboard::CycleComparison;
use crate::domain::foundation::{CycleId, UserId};
use crate::ports::{DashboardError, DashboardReader};

/// Query to compare multiple cycles.
#[derive(Debug, Clone)]
pub struct CompareCyclesQuery {
    /// The cycle IDs to compare (must be at least 2).
    pub cycle_ids: Vec<CycleId>,
    /// User ID for authorization.
    pub user_id: UserId,
}

/// Result of successful cycle comparison query.
pub type CompareCyclesResult = CycleComparison;

/// Handler for comparing multiple cycles.
///
/// Validates that at least 2 cycles are provided, then delegates to reader.
pub struct CompareCyclesHandler {
    reader: Arc<dyn DashboardReader>,
}

impl CompareCyclesHandler {
    pub fn new(reader: Arc<dyn DashboardReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(
        &self,
        query: CompareCyclesQuery,
    ) -> Result<CompareCyclesResult, DashboardError> {
        // Validate that we have at least 2 cycles to compare
        if query.cycle_ids.len() < 2 {
            return Err(DashboardError::InvalidInput(
                "At least 2 cycles required for comparison".to_string(),
            ));
        }

        self.reader
            .compare_cycles(&query.cycle_ids, &query.user_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::dashboard::{
        ComponentComparisonSummary, ComparisonSummary, CycleComparison, CycleComparisonItem,
        CycleProgressSnapshot, DashboardOverview, ComponentDetailView,
    };
    use crate::domain::foundation::{ComponentType, CycleId, SessionId, UserId};
    use async_trait::async_trait;

    // ─────────────────────────────────────────────────────────────────────
    // Mock Implementation
    // ─────────────────────────────────────────────────────────────────────

    struct MockDashboardReader {
        comparison: Option<CycleComparison>,
        should_fail: bool,
        should_unauthorized: bool,
    }

    impl MockDashboardReader {
        fn with_comparison(comparison: CycleComparison) -> Self {
            Self {
                comparison: Some(comparison),
                should_fail: false,
                should_unauthorized: false,
            }
        }

        fn failing() -> Self {
            Self {
                comparison: None,
                should_fail: true,
                should_unauthorized: false,
            }
        }

        fn unauthorized() -> Self {
            Self {
                comparison: None,
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
            unimplemented!()
        }

        async fn get_component_detail(
            &self,
            _cycle_id: CycleId,
            _component_type: ComponentType,
            _user_id: &UserId,
        ) -> Result<ComponentDetailView, DashboardError> {
            unimplemented!()
        }

        async fn compare_cycles(
            &self,
            _cycle_ids: &[CycleId],
            _user_id: &UserId,
        ) -> Result<CycleComparison, DashboardError> {
            if self.should_unauthorized {
                return Err(DashboardError::Unauthorized);
            }
            if self.should_fail {
                return Err(DashboardError::Database("Simulated failure".to_string()));
            }
            self.comparison
                .clone()
                .ok_or_else(|| DashboardError::CycleNotFound(CycleId::new()))
        }
    }

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    fn create_test_comparison() -> CycleComparison {
        let cycle1 = CycleId::new();
        let cycle2 = CycleId::new();

        CycleComparison {
            cycles: vec![
                CycleComparisonItem {
                    cycle_id: cycle1,
                    branch_point: None,
                    progress: CycleProgressSnapshot {
                        completed_count: 5,
                        total_count: 9,
                        percent_complete: 55,
                        current_step: Some(ComponentType::Tradeoffs),
                    },
                    component_summaries: vec![
                        ComponentComparisonSummary {
                            component_type: ComponentType::Objectives,
                            summary: "3 objectives".to_string(),
                            differs_from_others: false,
                        },
                    ],
                },
                CycleComparisonItem {
                    cycle_id: cycle2,
                    branch_point: Some(ComponentType::Alternatives),
                    progress: CycleProgressSnapshot {
                        completed_count: 4,
                        total_count: 9,
                        percent_complete: 44,
                        current_step: Some(ComponentType::Consequences),
                    },
                    component_summaries: vec![
                        ComponentComparisonSummary {
                            component_type: ComponentType::Objectives,
                            summary: "3 objectives".to_string(),
                            differs_from_others: false,
                        },
                    ],
                },
            ],
            differences: vec![],
            summary: ComparisonSummary {
                total_cycles: 2,
                components_with_differences: 0,
                most_different_cycle: None,
                recommendation_differs: false,
            },
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Tests
    // ─────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_compare_requires_two_cycles() {
        let reader = Arc::new(MockDashboardReader::with_comparison(create_test_comparison()));
        let handler = CompareCyclesHandler::new(reader);

        // Single cycle should fail
        let query = CompareCyclesQuery {
            cycle_ids: vec![CycleId::new()],
            user_id: test_user_id(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DashboardError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn test_compare_accepts_two_cycles() {
        let comparison = create_test_comparison();
        let reader = Arc::new(MockDashboardReader::with_comparison(comparison.clone()));
        let handler = CompareCyclesHandler::new(reader);

        let query = CompareCyclesQuery {
            cycle_ids: vec![CycleId::new(), CycleId::new()],
            user_id: test_user_id(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_ok());

        let returned_comparison = result.unwrap();
        assert_eq!(returned_comparison.cycles.len(), 2);
        assert_eq!(returned_comparison.summary.total_cycles, 2);
    }

    #[tokio::test]
    async fn test_compare_accepts_multiple_cycles() {
        let comparison = create_test_comparison();
        let reader = Arc::new(MockDashboardReader::with_comparison(comparison));
        let handler = CompareCyclesHandler::new(reader);

        let query = CompareCyclesQuery {
            cycle_ids: vec![CycleId::new(), CycleId::new(), CycleId::new()],
            user_id: test_user_id(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compare_passes_user_id() {
        let comparison = create_test_comparison();
        let user_id = test_user_id();
        let reader = Arc::new(MockDashboardReader::with_comparison(comparison));
        let handler = CompareCyclesHandler::new(reader);

        let query = CompareCyclesQuery {
            cycle_ids: vec![CycleId::new(), CycleId::new()],
            user_id: user_id.clone(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compare_handles_unauthorized() {
        let reader = Arc::new(MockDashboardReader::unauthorized());
        let handler = CompareCyclesHandler::new(reader);

        let query = CompareCyclesQuery {
            cycle_ids: vec![CycleId::new(), CycleId::new()],
            user_id: test_user_id(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DashboardError::Unauthorized));
    }

    #[tokio::test]
    async fn test_compare_propagates_errors() {
        let reader = Arc::new(MockDashboardReader::failing());
        let handler = CompareCyclesHandler::new(reader);

        let query = CompareCyclesQuery {
            cycle_ids: vec![CycleId::new(), CycleId::new()],
            user_id: test_user_id(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DashboardError::Database(_)));
    }
}
