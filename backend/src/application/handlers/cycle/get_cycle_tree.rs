//! GetCycleTreeHandler - Query handler for retrieving the cycle tree.
//!
//! Returns the hierarchical tree of cycles for a session,
//! showing the root cycle and all its branches.

use std::sync::Arc;

use crate::domain::foundation::{DomainError, ErrorCode, SessionId};
use crate::ports::{CycleReader, CycleTreeNode};

/// Query to get the cycle tree for a session.
#[derive(Debug, Clone)]
pub struct GetCycleTreeQuery {
    /// The session to get the tree for.
    pub session_id: SessionId,
}

/// Result of successful cycle tree query.
pub type GetCycleTreeResult = CycleTreeNode;

/// Error type for getting a cycle tree.
#[derive(Debug, Clone)]
pub enum GetCycleTreeError {
    /// No cycles found for session.
    NoCycles(SessionId),
    /// Infrastructure error.
    Infrastructure(String),
}

impl std::fmt::Display for GetCycleTreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetCycleTreeError::NoCycles(id) => write!(f, "No cycles found for session: {}", id),
            GetCycleTreeError::Infrastructure(msg) => write!(f, "Infrastructure error: {}", msg),
        }
    }
}

impl std::error::Error for GetCycleTreeError {}

impl From<DomainError> for GetCycleTreeError {
    fn from(err: DomainError) -> Self {
        match err.code {
            ErrorCode::SessionNotFound => GetCycleTreeError::NoCycles(SessionId::new()),
            _ => GetCycleTreeError::Infrastructure(err.message),
        }
    }
}

/// Handler for retrieving the cycle tree.
///
/// Returns the full cycle hierarchy for visualization,
/// or an error if no cycles exist for the session.
pub struct GetCycleTreeHandler {
    reader: Arc<dyn CycleReader>,
}

impl GetCycleTreeHandler {
    pub fn new(reader: Arc<dyn CycleReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(
        &self,
        query: GetCycleTreeQuery,
    ) -> Result<GetCycleTreeResult, GetCycleTreeError> {
        self.reader
            .get_tree(&query.session_id)
            .await?
            .ok_or(GetCycleTreeError::NoCycles(query.session_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{ComponentType, CycleId, CycleStatus, Timestamp};
    use crate::ports::{CycleProgressView, CycleSummary, CycleTreeNode, CycleView};
    use async_trait::async_trait;
    use std::collections::HashMap;

    // ─────────────────────────────────────────────────────────────────────
    // Mock Implementation
    // ─────────────────────────────────────────────────────────────────────

    struct MockCycleReader {
        trees: HashMap<SessionId, CycleTreeNode>,
        fail_read: bool,
    }

    impl MockCycleReader {
        fn new() -> Self {
            Self {
                trees: HashMap::new(),
                fail_read: false,
            }
        }

        fn with_tree(session_id: SessionId, tree: CycleTreeNode) -> Self {
            let mut trees = HashMap::new();
            trees.insert(session_id, tree);
            Self {
                trees,
                fail_read: false,
            }
        }

        fn failing() -> Self {
            Self {
                trees: HashMap::new(),
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
                    ErrorCode::DatabaseError,
                    "Simulated read failure",
                ));
            }
            Ok(self.trees.get(session_id).cloned())
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

    fn test_cycle_summary(id: CycleId, is_branch: bool) -> CycleSummary {
        CycleSummary {
            id,
            is_branch,
            branch_point: if is_branch {
                Some(ComponentType::Alternatives)
            } else {
                None
            },
            status: CycleStatus::Active,
            current_step: ComponentType::IssueRaising,
            progress_percent: 25,
            created_at: Timestamp::now(),
        }
    }

    fn test_tree_with_branch() -> CycleTreeNode {
        let root_id = CycleId::new();
        let branch_id = CycleId::new();

        CycleTreeNode {
            cycle: test_cycle_summary(root_id, false),
            children: vec![CycleTreeNode {
                cycle: test_cycle_summary(branch_id, true),
                children: vec![],
            }],
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Tests
    // ─────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn returns_tree_when_exists() {
        let session_id = SessionId::new();
        let tree = test_tree_with_branch();
        let reader = Arc::new(MockCycleReader::with_tree(session_id.clone(), tree));

        let handler = GetCycleTreeHandler::new(reader);
        let query = GetCycleTreeQuery { session_id };

        let result = handler.handle(query).await;
        assert!(result.is_ok());

        let tree = result.unwrap();
        assert!(!tree.cycle.is_branch);
        assert_eq!(tree.children.len(), 1);
    }

    #[tokio::test]
    async fn tree_contains_branch_information() {
        let session_id = SessionId::new();
        let tree = test_tree_with_branch();
        let reader = Arc::new(MockCycleReader::with_tree(session_id.clone(), tree));

        let handler = GetCycleTreeHandler::new(reader);
        let query = GetCycleTreeQuery { session_id };

        let result = handler.handle(query).await.unwrap();
        let branch = &result.children[0];
        assert!(branch.cycle.is_branch);
        assert_eq!(
            branch.cycle.branch_point,
            Some(ComponentType::Alternatives)
        );
    }

    #[tokio::test]
    async fn returns_no_cycles_when_session_empty() {
        let reader = Arc::new(MockCycleReader::new());

        let handler = GetCycleTreeHandler::new(reader);
        let query = GetCycleTreeQuery {
            session_id: SessionId::new(),
        };

        let result = handler.handle(query).await;
        assert!(matches!(result, Err(GetCycleTreeError::NoCycles(_))));
    }

    #[tokio::test]
    async fn returns_infrastructure_error_on_read_failure() {
        let reader = Arc::new(MockCycleReader::failing());

        let handler = GetCycleTreeHandler::new(reader);
        let query = GetCycleTreeQuery {
            session_id: SessionId::new(),
        };

        let result = handler.handle(query).await;
        assert!(matches!(result, Err(GetCycleTreeError::Infrastructure(_))));
    }
}
