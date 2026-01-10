//! GetCycleHandler - Query handler for retrieving cycle details.
//!
//! Returns the full cycle view for UI display including component statuses,
//! progress, and branching information.

use std::sync::Arc;

use crate::domain::foundation::{CycleId, DomainError};
use crate::ports::{CycleReader, CycleView};

/// Query to get a cycle by ID.
#[derive(Debug, Clone)]
pub struct GetCycleQuery {
    /// The cycle ID to retrieve.
    pub cycle_id: CycleId,
}

/// Result of successful cycle query.
pub type GetCycleResult = Option<CycleView>;

/// Handler for retrieving cycle details.
///
/// Returns the full cycle view for UI display,
/// or `None` if the cycle is not found.
pub struct GetCycleHandler {
    reader: Arc<dyn CycleReader>,
}

impl GetCycleHandler {
    pub fn new(reader: Arc<dyn CycleReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(&self, query: GetCycleQuery) -> Result<GetCycleResult, DomainError> {
        self.reader.get_by_id(&query.cycle_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{ComponentStatus, ComponentType, CycleStatus, SessionId, Timestamp};
    use crate::ports::{ComponentStatusItem, CycleProgressView, CycleSummary, CycleTreeNode};
    use async_trait::async_trait;

    // ─────────────────────────────────────────────────────────────────────
    // Mock Implementation
    // ─────────────────────────────────────────────────────────────────────

    struct MockCycleReader {
        views: Vec<CycleView>,
        fail_read: bool,
    }

    impl MockCycleReader {
        fn new() -> Self {
            Self {
                views: Vec::new(),
                fail_read: false,
            }
        }

        fn with_cycle(view: CycleView) -> Self {
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
    impl CycleReader for MockCycleReader {
        async fn get_by_id(&self, id: &CycleId) -> Result<Option<CycleView>, DomainError> {
            if self.fail_read {
                return Err(DomainError::new(
                    crate::domain::foundation::ErrorCode::DatabaseError,
                    "Simulated read failure",
                ));
            }
            Ok(self.views.iter().find(|v| v.id == *id).cloned())
        }

        async fn list_by_session_id(
            &self,
            _session_id: &SessionId,
        ) -> Result<Vec<CycleSummary>, DomainError> {
            Ok(vec![])
        }

        async fn get_tree(
            &self,
            _session_id: &SessionId,
        ) -> Result<Option<CycleTreeNode>, DomainError> {
            Ok(None)
        }

        async fn get_progress(&self, _id: &CycleId) -> Result<Option<CycleProgressView>, DomainError> {
            Ok(None)
        }

        async fn get_lineage(&self, _id: &CycleId) -> Result<Vec<CycleSummary>, DomainError> {
            Ok(vec![])
        }

        async fn get_component_output(
            &self,
            _cycle_id: &CycleId,
            _component_type: ComponentType,
        ) -> Result<Option<crate::ports::ComponentOutputView>, DomainError> {
            Ok(None)
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Test Helpers
    // ─────────────────────────────────────────────────────────────────────

    fn create_test_cycle_view() -> CycleView {
        let cycle_id = CycleId::new();
        CycleView {
            id: cycle_id,
            session_id: SessionId::new(),
            parent_cycle_id: None,
            branch_point: None,
            status: CycleStatus::Active,
            current_step: ComponentType::IssueRaising,
            component_statuses: vec![ComponentStatusItem {
                component_type: ComponentType::IssueRaising,
                status: ComponentStatus::InProgress,
                is_current: true,
            }],
            progress_percent: 10,
            is_complete: false,
            branch_count: 0,
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Tests
    // ─────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn returns_cycle_when_found() {
        let view = create_test_cycle_view();
        let cycle_id = view.id;

        let reader = Arc::new(MockCycleReader::with_cycle(view.clone()));
        let handler = GetCycleHandler::new(reader);

        let query = GetCycleQuery { cycle_id };
        let result = handler.handle(query).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.id, cycle_id);
        assert_eq!(result.status, CycleStatus::Active);
    }

    #[tokio::test]
    async fn returns_none_when_not_found() {
        let reader = Arc::new(MockCycleReader::new());
        let handler = GetCycleHandler::new(reader);

        let query = GetCycleQuery {
            cycle_id: CycleId::new(),
        };
        let result = handler.handle(query).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn returns_error_on_read_failure() {
        let reader = Arc::new(MockCycleReader::failing());
        let handler = GetCycleHandler::new(reader);

        let query = GetCycleQuery {
            cycle_id: CycleId::new(),
        };
        let result = handler.handle(query).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn includes_component_statuses() {
        let view = create_test_cycle_view();
        let cycle_id = view.id;

        let reader = Arc::new(MockCycleReader::with_cycle(view));
        let handler = GetCycleHandler::new(reader);

        let query = GetCycleQuery { cycle_id };
        let result = handler.handle(query).await.unwrap().unwrap();

        assert_eq!(result.component_statuses.len(), 1);
        assert_eq!(
            result.component_statuses[0].component_type,
            ComponentType::IssueRaising
        );
    }
}
