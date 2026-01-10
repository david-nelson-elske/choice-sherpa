//! GetCycleHandler - Query handler for retrieving cycle details.
//!
//! Returns a detailed view of a cycle for UI display,
//! including component statuses and progress information.

use std::sync::Arc;

use crate::domain::foundation::{CycleId, DomainError, ErrorCode};
use crate::ports::{CycleReader, CycleView};

/// Query to get a cycle by ID.
#[derive(Debug, Clone)]
pub struct GetCycleQuery {
    /// The cycle to retrieve.
    pub cycle_id: CycleId,
}

/// Result of successful cycle query.
pub type GetCycleResult = CycleView;

/// Error type for getting a cycle.
#[derive(Debug, Clone)]
pub enum GetCycleError {
    /// Cycle not found.
    NotFound(CycleId),
    /// Infrastructure error.
    Infrastructure(String),
}

impl std::fmt::Display for GetCycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetCycleError::NotFound(id) => write!(f, "Cycle not found: {}", id),
            GetCycleError::Infrastructure(msg) => write!(f, "Infrastructure error: {}", msg),
        }
    }
}

impl std::error::Error for GetCycleError {}

impl From<DomainError> for GetCycleError {
    fn from(err: DomainError) -> Self {
        match err.code {
            ErrorCode::CycleNotFound => GetCycleError::NotFound(CycleId::new()),
            _ => GetCycleError::Infrastructure(err.message),
        }
    }
}

/// Handler for retrieving cycle details.
///
/// Returns the full cycle view for UI display,
/// or an error if the cycle doesn't exist.
pub struct GetCycleHandler {
    reader: Arc<dyn CycleReader>,
}

impl GetCycleHandler {
    pub fn new(reader: Arc<dyn CycleReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(&self, query: GetCycleQuery) -> Result<GetCycleResult, GetCycleError> {
        self.reader
            .get_by_id(&query.cycle_id)
            .await?
            .ok_or(GetCycleError::NotFound(query.cycle_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{
        ComponentStatus, ComponentType, CycleStatus, SessionId, Timestamp,
    };
    use crate::ports::{
        ComponentStatusItem, CycleProgressView, CycleSummary, CycleTreeNode, CycleView,
    };
    use async_trait::async_trait;

    // ─────────────────────────────────────────────────────────────────────
    // Mock Implementation
    // ─────────────────────────────────────────────────────────────────────

    struct MockCycleReader {
        cycles: Vec<CycleView>,
        fail_read: bool,
    }

    impl MockCycleReader {
        fn new() -> Self {
            Self {
                cycles: Vec::new(),
                fail_read: false,
            }
        }

        fn with_cycle(cycle: CycleView) -> Self {
            Self {
                cycles: vec![cycle],
                fail_read: false,
            }
        }

        fn failing() -> Self {
            Self {
                cycles: Vec::new(),
                fail_read: true,
            }
        }
    }

    #[async_trait]
    impl CycleReader for MockCycleReader {
        async fn get_by_id(&self, id: &CycleId) -> Result<Option<CycleView>, DomainError> {
            if self.fail_read {
                return Err(DomainError::new(
                    ErrorCode::DatabaseError,
                    "Simulated read failure",
                ));
            }
            Ok(self.cycles.iter().find(|c| &c.id == id).cloned())
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

        async fn get_progress(
            &self,
            _id: &CycleId,
        ) -> Result<Option<CycleProgressView>, DomainError> {
            Ok(None)
        }

        async fn get_lineage(&self, _id: &CycleId) -> Result<Vec<CycleSummary>, DomainError> {
            Ok(vec![])
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Test Helpers
    // ─────────────────────────────────────────────────────────────────────

    fn test_cycle_view(id: CycleId) -> CycleView {
        CycleView {
            id,
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
            progress_percent: 0,
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
    async fn returns_cycle_when_exists() {
        let cycle_id = CycleId::new();
        let view = test_cycle_view(cycle_id);
        let reader = Arc::new(MockCycleReader::with_cycle(view.clone()));

        let handler = GetCycleHandler::new(reader);
        let query = GetCycleQuery { cycle_id };

        let result = handler.handle(query).await;
        assert!(result.is_ok());

        let cycle = result.unwrap();
        assert_eq!(cycle.id, cycle_id);
        assert_eq!(cycle.status, CycleStatus::Active);
    }

    #[tokio::test]
    async fn returns_correct_component_statuses() {
        let cycle_id = CycleId::new();
        let view = test_cycle_view(cycle_id);
        let reader = Arc::new(MockCycleReader::with_cycle(view));

        let handler = GetCycleHandler::new(reader);
        let query = GetCycleQuery { cycle_id };

        let result = handler.handle(query).await.unwrap();
        assert_eq!(result.component_statuses.len(), 1);
        assert_eq!(
            result.component_statuses[0].component_type,
            ComponentType::IssueRaising
        );
    }

    #[tokio::test]
    async fn returns_not_found_when_cycle_missing() {
        let reader = Arc::new(MockCycleReader::new());

        let handler = GetCycleHandler::new(reader);
        let query = GetCycleQuery {
            cycle_id: CycleId::new(),
        };

        let result = handler.handle(query).await;
        assert!(matches!(result, Err(GetCycleError::NotFound(_))));
    }

    #[tokio::test]
    async fn returns_infrastructure_error_on_read_failure() {
        let reader = Arc::new(MockCycleReader::failing());

        let handler = GetCycleHandler::new(reader);
        let query = GetCycleQuery {
            cycle_id: CycleId::new(),
        };

        let result = handler.handle(query).await;
        assert!(matches!(result, Err(GetCycleError::Infrastructure(_))));
    }
}
