//! GetCycleTreeHandler - Query handler for retrieving the cycle tree.
//!
//! Returns the hierarchical tree of cycles for a session, showing the
//! primary cycle and all its branches organized by parent-child relationships.

use std::sync::Arc;

use crate::domain::foundation::{DomainError, SessionId};
use crate::ports::{CycleReader, CycleTreeNode};

/// Query to get the cycle tree for a session.
#[derive(Debug, Clone)]
pub struct GetCycleTreeQuery {
    /// The session to get the cycle tree for.
    pub session_id: SessionId,
}

/// Result of successful cycle tree query.
pub type GetCycleTreeResult = Option<CycleTreeNode>;

/// Handler for retrieving the cycle tree.
///
/// Returns the root cycle with all branches organized hierarchically,
/// or `None` if no cycles exist for the session.
pub struct GetCycleTreeHandler {
    reader: Arc<dyn CycleReader>,
}

impl GetCycleTreeHandler {
    pub fn new(reader: Arc<dyn CycleReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(&self, query: GetCycleTreeQuery) -> Result<GetCycleTreeResult, DomainError> {
        self.reader.get_tree(&query.session_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{ComponentType, CycleId, CycleStatus, Timestamp};
    use crate::ports::{CycleProgressView, CycleSummary, CycleView};
    use async_trait::async_trait;

    // ─────────────────────────────────────────────────────────────────────
    // Mock Implementation
    // ─────────────────────────────────────────────────────────────────────

    struct MockCycleReader {
        trees: Vec<(SessionId, CycleTreeNode)>,
        fail_read: bool,
    }

    impl MockCycleReader {
        fn new() -> Self {
            Self {
                trees: Vec::new(),
                fail_read: false,
            }
        }

        fn with_tree(session_id: SessionId, tree: CycleTreeNode) -> Self {
            Self {
                trees: vec![(session_id, tree)],
                fail_read: false,
            }
        }

        fn failing() -> Self {
            Self {
                trees: Vec::new(),
                fail_read: true,
            }
        }
    }

    #[async_trait]
    impl CycleReader for MockCycleReader {
        async fn get_by_id(&self, _id: &CycleId) -> Result<Option<CycleView>, DomainError> {
            Ok(None)
        }

        async fn list_by_session_id(
            &self,
            _session_id: &SessionId,
        ) -> Result<Vec<CycleSummary>, DomainError> {
            Ok(vec![])
        }

        async fn get_tree(
            &self,
            session_id: &SessionId,
        ) -> Result<Option<CycleTreeNode>, DomainError> {
            if self.fail_read {
                return Err(DomainError::new(
                    crate::domain::foundation::ErrorCode::DatabaseError,
                    "Simulated read failure",
                ));
            }
            Ok(self
                .trees
                .iter()
                .find(|(id, _)| id == session_id)
                .map(|(_, tree)| tree.clone()))
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

    fn create_test_tree() -> CycleTreeNode {
        let child = CycleTreeNode {
            cycle: CycleSummary {
                id: CycleId::new(),
                is_branch: true,
                branch_point: Some(ComponentType::Alternatives),
                status: CycleStatus::Active,
                current_step: ComponentType::Alternatives,
                progress_percent: 50,
                created_at: Timestamp::now(),
            },
            children: vec![],
        };

        CycleTreeNode {
            cycle: CycleSummary {
                id: CycleId::new(),
                is_branch: false,
                branch_point: None,
                status: CycleStatus::Active,
                current_step: ComponentType::Tradeoffs,
                progress_percent: 75,
                created_at: Timestamp::now(),
            },
            children: vec![child],
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Tests
    // ─────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn returns_tree_when_found() {
        let session_id = SessionId::new();
        let tree = create_test_tree();

        let reader = Arc::new(MockCycleReader::with_tree(session_id, tree));
        let handler = GetCycleTreeHandler::new(reader);

        let query = GetCycleTreeQuery { session_id };
        let result = handler.handle(query).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_some());
        let tree = result.unwrap();
        assert!(!tree.cycle.is_branch);
        assert_eq!(tree.children.len(), 1);
    }

    #[tokio::test]
    async fn returns_none_when_no_cycles() {
        let reader = Arc::new(MockCycleReader::new());
        let handler = GetCycleTreeHandler::new(reader);

        let query = GetCycleTreeQuery {
            session_id: SessionId::new(),
        };
        let result = handler.handle(query).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn returns_error_on_read_failure() {
        let reader = Arc::new(MockCycleReader::failing());
        let handler = GetCycleTreeHandler::new(reader);

        let query = GetCycleTreeQuery {
            session_id: SessionId::new(),
        };
        let result = handler.handle(query).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn tree_contains_branch_information() {
        let session_id = SessionId::new();
        let tree = create_test_tree();

        let reader = Arc::new(MockCycleReader::with_tree(session_id, tree));
        let handler = GetCycleTreeHandler::new(reader);

        let query = GetCycleTreeQuery { session_id };
        let result = handler.handle(query).await.unwrap().unwrap();

        // Check root
        assert!(!result.cycle.is_branch);
        assert!(result.cycle.branch_point.is_none());

        // Check child branch
        assert_eq!(result.children.len(), 1);
        assert!(result.children[0].cycle.is_branch);
        assert_eq!(
            result.children[0].cycle.branch_point,
            Some(ComponentType::Alternatives)
        );
    }
}
