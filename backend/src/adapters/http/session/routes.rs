//! Route configuration for session endpoints.
//!
//! Configures Axum router with session-related routes.

use axum::routing::get;
use axum::Router;

use super::handlers::{get_cycle_tree, SessionAppState};

/// Creates the session router with all endpoints.
///
/// Routes:
/// - `GET /api/sessions/:id/cycle-tree` - Get cycle tree for session
pub fn session_router() -> Router<SessionAppState> {
    Router::new().route("/api/sessions/:id/cycle-tree", get(get_cycle_tree))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{ComponentType, CycleId, CycleStatus, DomainError, SessionId, Timestamp};
    use crate::ports::{CycleProgressView, CycleSummary, CycleTreeNode, CycleView};
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::Request;
    use std::sync::Arc;
    use tower::ServiceExt;

    // ───────────────────────────────────────────────────────────────
    // Mock implementations (minimal for route testing)
    // ───────────────────────────────────────────────────────────────

    struct MockCycleReader {
        tree: Option<CycleTreeNode>,
    }

    impl MockCycleReader {
        fn with_tree(tree: CycleTreeNode) -> Self {
            Self { tree: Some(tree) }
        }
    }

    #[async_trait]
    impl crate::ports::CycleReader for MockCycleReader {
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
            Ok(self.tree.clone())
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

    fn create_test_tree() -> CycleTreeNode {
        CycleTreeNode {
            cycle: CycleSummary {
                id: CycleId::new(),
                is_branch: false,
                branch_point: None,
                status: CycleStatus::Active,
                current_step: ComponentType::IssueRaising,
                progress_percent: 0,
                created_at: Timestamp::now(),
            },
            children: vec![],
        }
    }

    // ───────────────────────────────────────────────────────────────
    // Tests
    // ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn session_router_mounts_cycle_tree_endpoint() {
        let tree = create_test_tree();
        let state = SessionAppState::new(Arc::new(MockCycleReader::with_tree(tree)));

        let app = session_router().with_state(state);

        let session_id = SessionId::new();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/sessions/{}/cycle-tree", session_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }
}
