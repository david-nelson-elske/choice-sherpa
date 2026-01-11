//! HTTP handlers for dashboard endpoints.
//!
//! These handlers connect Axum routes to application layer query handlers.

use std::sync::Arc;

use axum::extract::{Json, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::application::handlers::{
    CompareCyclesHandler, CompareCyclesQuery, GetComponentDetailHandler, GetComponentDetailQuery,
    GetDashboardOverviewHandler, GetDashboardOverviewQuery,
};
use crate::domain::foundation::{ComponentType, CycleId, SessionId, UserId};
use crate::ports::{DashboardError, DashboardReader};

use super::dto::{ComponentDetailView, CycleComparison, DashboardOverview, ErrorResponse};

// ════════════════════════════════════════════════════════════════════════════════
// Error Type
// ════════════════════════════════════════════════════════════════════════════════

/// Dashboard API error that implements IntoResponse.
pub enum DashboardApiError {
    BadRequest(String),
    NotFound(String),
    Unauthorized(String),
    Internal(String),
}

impl IntoResponse for DashboardApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error) = match self {
            DashboardApiError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, ErrorResponse::bad_request(msg))
            }
            DashboardApiError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, ErrorResponse::not_found("Resource", &msg))
            }
            DashboardApiError::Unauthorized(msg) => {
                (StatusCode::FORBIDDEN, ErrorResponse::unauthorized(msg))
            }
            DashboardApiError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ErrorResponse::internal(msg))
            }
        };
        (status, Json(error)).into_response()
    }
}

impl From<DashboardError> for DashboardApiError {
    fn from(error: DashboardError) -> Self {
        match error {
            DashboardError::SessionNotFound(id) => {
                DashboardApiError::NotFound(format!("Session {} not found", id))
            }
            DashboardError::CycleNotFound(id) => {
                DashboardApiError::NotFound(format!("Cycle {} not found", id))
            }
            DashboardError::ComponentNotFound(component_type) => {
                DashboardApiError::NotFound(format!("Component {:?} not found", component_type))
            }
            DashboardError::Unauthorized => {
                DashboardApiError::Unauthorized("You do not have access to this resource".to_string())
            }
            DashboardError::InvalidInput(msg) => {
                DashboardApiError::BadRequest(msg)
            }
            DashboardError::Database(msg) => {
                DashboardApiError::Internal(format!("Database error: {}", msg))
            }
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Application State
// ════════════════════════════════════════════════════════════════════════════════

/// Shared application state containing dashboard dependencies.
#[derive(Clone)]
pub struct DashboardAppState {
    pub dashboard_reader: Arc<dyn DashboardReader>,
}

impl DashboardAppState {
    pub fn get_overview_handler(&self) -> GetDashboardOverviewHandler {
        GetDashboardOverviewHandler::new(self.dashboard_reader.clone())
    }

    pub fn get_component_detail_handler(&self) -> GetComponentDetailHandler {
        GetComponentDetailHandler::new(self.dashboard_reader.clone())
    }

    pub fn compare_cycles_handler(&self) -> CompareCyclesHandler {
        CompareCyclesHandler::new(self.dashboard_reader.clone())
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// User Context
// ════════════════════════════════════════════════════════════════════════════════

/// Authenticated user context extracted from request.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: UserId,
}

/// Rejection type for AuthenticatedUser extraction.
pub struct AuthenticationRequired;

impl IntoResponse for AuthenticationRequired {
    fn into_response(self) -> axum::response::Response {
        let error = ErrorResponse::unauthorized("Authentication is required");
        (StatusCode::UNAUTHORIZED, Json(error)).into_response()
    }
}

impl<S> axum::extract::FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthenticationRequired;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut axum::http::request::Parts,
        _state: &'life1 S,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let user_id = parts
                .headers
                .get("x-user-id")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| UserId::new(s).ok())
                .ok_or(AuthenticationRequired)?;

            Ok(AuthenticatedUser { user_id })
        })
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Query Parameters
// ════════════════════════════════════════════════════════════════════════════════

/// Query parameters for dashboard overview endpoint.
#[derive(Debug, Deserialize)]
pub struct DashboardOverviewParams {
    /// Optional cycle ID to view specific cycle.
    pub cycle_id: Option<String>,
}

/// Query parameters for cycle comparison endpoint.
#[derive(Debug, Deserialize)]
pub struct CompareCyclesParams {
    /// Comma-separated list of cycle IDs.
    pub cycles: String,
}

// ════════════════════════════════════════════════════════════════════════════════
// Handlers
// ════════════════════════════════════════════════════════════════════════════════

/// GET /api/sessions/:session_id/dashboard
///
/// Returns the main dashboard overview for a session.
pub async fn get_dashboard_overview(
    State(state): State<DashboardAppState>,
    Path(session_id_str): Path<String>,
    Query(params): Query<DashboardOverviewParams>,
    user: AuthenticatedUser,
) -> Result<Json<DashboardOverview>, DashboardApiError> {
    // Parse session_id
    let session_id: SessionId = session_id_str
        .parse()
        .map_err(|_| DashboardApiError::BadRequest("Invalid session ID format".to_string()))?;

    // Parse optional cycle_id
    let cycle_id = if let Some(ref cid_str) = params.cycle_id {
        Some(
            cid_str
                .parse::<CycleId>()
                .map_err(|_| DashboardApiError::BadRequest("Invalid cycle ID format".to_string()))?,
        )
    } else {
        None
    };

    // Execute query
    let query = GetDashboardOverviewQuery {
        session_id,
        cycle_id,
        user_id: user.user_id,
    };

    let handler = state.get_overview_handler();
    let overview = handler.handle(query).await?;

    Ok(Json(overview))
}

/// GET /api/cycles/:cycle_id/components/:component_type/detail
///
/// Returns detailed view of a specific component.
pub async fn get_component_detail(
    State(state): State<DashboardAppState>,
    Path((cycle_id_str, component_type)): Path<(String, ComponentType)>,
    user: AuthenticatedUser,
) -> Result<Json<ComponentDetailView>, DashboardApiError> {
    // Parse cycle_id
    let cycle_id: CycleId = cycle_id_str
        .parse()
        .map_err(|_| DashboardApiError::BadRequest("Invalid cycle ID format".to_string()))?;

    // Execute query
    let query = GetComponentDetailQuery {
        cycle_id,
        component_type,
        user_id: user.user_id,
    };

    let handler = state.get_component_detail_handler();
    let detail = handler.handle(query).await?;

    Ok(Json(detail))
}

/// GET /api/sessions/:session_id/compare?cycles=id1,id2
///
/// Returns comparison of multiple cycles.
pub async fn compare_cycles(
    State(state): State<DashboardAppState>,
    Path(_session_id_str): Path<String>,
    Query(params): Query<CompareCyclesParams>,
    user: AuthenticatedUser,
) -> Result<Json<CycleComparison>, DashboardApiError> {
    // Parse cycle IDs from comma-separated string
    let cycle_ids: Result<Vec<CycleId>, _> = params
        .cycles
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse())
        .collect();

    let cycle_ids = cycle_ids.map_err(|_| {
        DashboardApiError::BadRequest("Invalid cycle ID format in cycles parameter".to_string())
    })?;

    if cycle_ids.is_empty() {
        return Err(DashboardApiError::BadRequest(
            "At least one cycle ID required".to_string(),
        ));
    }

    // Execute query
    let query = CompareCyclesQuery {
        cycle_ids,
        user_id: user.user_id,
    };

    let handler = state.compare_cycles_handler();
    let comparison = handler.handle(query).await?;

    Ok(Json(comparison))
}

