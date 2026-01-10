//! HTTP handlers for session endpoints.
//!
//! Implements Axum handlers for session-related operations.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;

use crate::domain::foundation::SessionId;
use crate::ports::CycleReader;

use super::dto::{CycleTreeResponse, ErrorResponse};

/// Application state for session endpoints.
#[derive(Clone)]
pub struct SessionAppState {
    pub cycle_reader: Arc<dyn CycleReader>,
}

impl SessionAppState {
    /// Creates new session app state.
    pub fn new(cycle_reader: Arc<dyn CycleReader>) -> Self {
        Self { cycle_reader }
    }
}

/// GET /api/sessions/:id/cycle-tree
///
/// Returns the tree of cycles for a session, including the primary cycle
/// and all its branches organized hierarchically.
///
/// # Path Parameters
///
/// - `id`: Session ID (UUID format)
///
/// # Response
///
/// - `200 OK`: Cycle tree with root node, total count, and max depth
/// - `400 Bad Request`: Invalid session ID format
/// - `404 Not Found`: Session has no cycles
pub async fn get_cycle_tree(
    State(state): State<SessionAppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    // Parse session ID
    let session_id = match session_id.parse::<uuid::Uuid>() {
        Ok(uuid) => SessionId::from_uuid(uuid),
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request("Invalid session ID format")),
            )
                .into_response();
        }
    };

    // Get cycle tree from reader
    match state.cycle_reader.get_tree(&session_id).await {
        Ok(Some(tree)) => {
            let response = CycleTreeResponse::from(tree);
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::not_found(
                "Cycle tree",
                &session_id.to_string(),
            )),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::internal(e.to_string())),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{ComponentType, CycleId, CycleStatus, DomainError, Timestamp};
    use crate::ports::{CycleProgressView, CycleSummary, CycleTreeNode, CycleView};
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    // ───────────────────────────────────────────────────────────────
    // Mock implementations
    // ───────────────────────────────────────────────────────────────

    struct MockCycleReader {
        tree: Option<CycleTreeNode>,
    }

    impl MockCycleReader {
        fn with_tree(tree: CycleTreeNode) -> Self {
            Self { tree: Some(tree) }
        }

        fn empty() -> Self {
            Self { tree: None }
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
        let child = CycleTreeNode {
            cycle: CycleSummary {
                id: CycleId::new(),
                is_branch: true,
                branch_point: Some(ComponentType::Alternatives),
                status: CycleStatus::Active,
                current_step: ComponentType::Consequences,
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

    // ───────────────────────────────────────────────────────────────
    // Tests
    // ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn get_cycle_tree_returns_tree_for_valid_session() {
        let tree = create_test_tree();
        let state = SessionAppState::new(Arc::new(MockCycleReader::with_tree(tree)));

        let app = Router::new()
            .route("/api/sessions/:id/cycle-tree", get(get_cycle_tree))
            .with_state(state);

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

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let tree: CycleTreeResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(tree.total_cycles, 2);
        assert_eq!(tree.max_depth, 1);
    }

    #[tokio::test]
    async fn get_cycle_tree_returns_not_found_when_no_cycles() {
        let state = SessionAppState::new(Arc::new(MockCycleReader::empty()));

        let app = Router::new()
            .route("/api/sessions/:id/cycle-tree", get(get_cycle_tree))
            .with_state(state);

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

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn get_cycle_tree_returns_bad_request_for_invalid_uuid() {
        let state = SessionAppState::new(Arc::new(MockCycleReader::empty()));

        let app = Router::new()
            .route("/api/sessions/:id/cycle-tree", get(get_cycle_tree))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/sessions/not-a-uuid/cycle-tree")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
