//! GetProactTreeViewHandler - Query handler for retrieving the PrOACT tree visualization.
//!
//! Returns the hierarchical tree of cycles with PrOACT letter statuses for
//! each node, optimized for the cycle tree browser UI component.

use std::sync::Arc;

use crate::domain::cycle::CycleTreeNode as PrOACTTreeNode;
use crate::domain::foundation::{DomainError, SessionId};
use crate::ports::CycleReader;

/// Query to get the PrOACT tree view for a session.
#[derive(Debug, Clone)]
pub struct GetProactTreeViewQuery {
    /// The session to get the PrOACT tree for.
    pub session_id: SessionId,
}

/// Result of successful PrOACT tree query.
pub type GetProactTreeViewResult = Option<PrOACTTreeNode>;

/// Handler for retrieving the PrOACT tree visualization.
///
/// Returns the root cycle with all branches organized hierarchically,
/// with PrOACT letter statuses (P-r-O-A-C-T) for each node.
pub struct GetProactTreeViewHandler {
    reader: Arc<dyn CycleReader>,
}

impl GetProactTreeViewHandler {
    pub fn new(reader: Arc<dyn CycleReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(&self, query: GetProactTreeViewQuery) -> Result<GetProactTreeViewResult, DomainError> {
        self.reader.get_proact_tree_view(&query.session_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cycle::{LetterStatus, PrOACTLetter, PrOACTStatus};
    use crate::domain::foundation::{ComponentType, CycleId, CycleStatus, Timestamp};
    use crate::ports::{ComponentOutputView, CycleProgressView, CycleSummary, CycleTreeNode, CycleView};
    use async_trait::async_trait;

    // ─────────────────────────────────────────────────────────────────────
    // Mock Implementation
    // ─────────────────────────────────────────────────────────────────────

    struct MockCycleReader {
        trees: Vec<(SessionId, PrOACTTreeNode)>,
        fail_read: bool,
    }

    impl MockCycleReader {
        fn new() -> Self {
            Self {
                trees: Vec::new(),
                fail_read: false,
            }
        }

        fn with_tree(session_id: SessionId, tree: PrOACTTreeNode) -> Self {
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
        ) -> Result<Option<ComponentOutputView>, DomainError> {
            Ok(None)
        }

        async fn get_proact_tree_view(
            &self,
            session_id: &SessionId,
        ) -> Result<Option<PrOACTTreeNode>, DomainError> {
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
    }

    // ─────────────────────────────────────────────────────────────────────
    // Test Helpers
    // ─────────────────────────────────────────────────────────────────────

    fn create_test_tree() -> PrOACTTreeNode {
        let child = PrOACTTreeNode {
            cycle_id: CycleId::new(),
            label: "Branch at Alternatives".to_string(),
            branch_point: Some(PrOACTLetter::O),
            letter_statuses: PrOACTStatus {
                p: LetterStatus::Completed,
                r: LetterStatus::Completed,
                o: LetterStatus::InProgress,
                a: LetterStatus::NotStarted,
                c: LetterStatus::NotStarted,
                t: LetterStatus::NotStarted,
            },
            children: vec![],
            updated_at: chrono::Utc::now(),
        };

        PrOACTTreeNode {
            cycle_id: CycleId::new(),
            label: "Primary Cycle".to_string(),
            branch_point: None,
            letter_statuses: PrOACTStatus {
                p: LetterStatus::Completed,
                r: LetterStatus::Completed,
                o: LetterStatus::Completed,
                a: LetterStatus::Completed,
                c: LetterStatus::InProgress,
                t: LetterStatus::NotStarted,
            },
            children: vec![child],
            updated_at: chrono::Utc::now(),
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
        let handler = GetProactTreeViewHandler::new(reader);

        let query = GetProactTreeViewQuery { session_id };
        let result = handler.handle(query).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_some());
        let tree = result.unwrap();
        assert!(tree.branch_point.is_none());
        assert_eq!(tree.children.len(), 1);
    }

    #[tokio::test]
    async fn returns_none_when_no_cycles() {
        let reader = Arc::new(MockCycleReader::new());
        let handler = GetProactTreeViewHandler::new(reader);

        let query = GetProactTreeViewQuery {
            session_id: SessionId::new(),
        };
        let result = handler.handle(query).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn returns_error_on_read_failure() {
        let reader = Arc::new(MockCycleReader::failing());
        let handler = GetProactTreeViewHandler::new(reader);

        let query = GetProactTreeViewQuery {
            session_id: SessionId::new(),
        };
        let result = handler.handle(query).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn tree_contains_proact_status() {
        let session_id = SessionId::new();
        let tree = create_test_tree();

        let reader = Arc::new(MockCycleReader::with_tree(session_id, tree));
        let handler = GetProactTreeViewHandler::new(reader);

        let query = GetProactTreeViewQuery { session_id };
        let result = handler.handle(query).await.unwrap().unwrap();

        // Check root has letter statuses
        assert_eq!(result.letter_statuses.p, LetterStatus::Completed);
        assert_eq!(result.letter_statuses.r, LetterStatus::Completed);
        assert_eq!(result.letter_statuses.o, LetterStatus::Completed);
        assert_eq!(result.letter_statuses.c, LetterStatus::InProgress);

        // Check child branch
        assert_eq!(result.children.len(), 1);
        assert_eq!(result.children[0].branch_point, Some(PrOACTLetter::O));
        assert_eq!(result.children[0].letter_statuses.o, LetterStatus::InProgress);
    }
}
