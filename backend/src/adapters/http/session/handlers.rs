//! HTTP handlers for session endpoints.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::adapters::http::middleware::RequireAuth;
use crate::application::handlers::session::{
    ArchiveSessionCommand, ArchiveSessionHandler, CreateSessionCommand, CreateSessionHandler,
    GetSessionHandler, GetSessionQuery, ListUserSessionsHandler, ListUserSessionsQuery,
    RenameSessionCommand, RenameSessionHandler,
};
use crate::domain::foundation::{CommandMetadata, SessionId};
use crate::domain::session::SessionError;

use super::dto::{
    CreateSessionRequest, ErrorResponse, ListSessionsQuery, RenameSessionRequest,
    SessionCommandResponse, SessionListResponse, SessionResponse,
};

// ════════════════════════════════════════════════════════════════════════════
// Handler state
// ════════════════════════════════════════════════════════════════════════════

#[derive(Clone)]
pub struct SessionHandlers {
    create_handler: Arc<CreateSessionHandler>,
    rename_handler: Arc<RenameSessionHandler>,
    archive_handler: Arc<ArchiveSessionHandler>,
    get_handler: Arc<GetSessionHandler>,
    list_handler: Arc<ListUserSessionsHandler>,
}

impl SessionHandlers {
    pub fn new(
        create_handler: Arc<CreateSessionHandler>,
        rename_handler: Arc<RenameSessionHandler>,
        archive_handler: Arc<ArchiveSessionHandler>,
        get_handler: Arc<GetSessionHandler>,
        list_handler: Arc<ListUserSessionsHandler>,
    ) -> Self {
        Self {
            create_handler,
            rename_handler,
            archive_handler,
            get_handler,
            list_handler,
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════
// HTTP handlers
// ════════════════════════════════════════════════════════════════════════════

/// POST /api/sessions - Create a new session
pub async fn create_session(
    State(handlers): State<SessionHandlers>,
    RequireAuth(user): RequireAuth,
    Json(req): Json<CreateSessionRequest>,
) -> Response {
    let cmd = CreateSessionCommand {
        user_id: user.id.clone(),
        title: req.title,
        description: req.description,
    };

    let metadata = CommandMetadata::new(user.id).with_correlation_id("http-request");

    match handlers.create_handler.handle(cmd, metadata).await {
        Ok(result) => {
            let response = SessionCommandResponse {
                session_id: result.session.id().to_string(),
                message: "Session created successfully".to_string(),
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => handle_session_error(e),
    }
}

/// GET /api/sessions/:id - Get session details
pub async fn get_session(
    State(handlers): State<SessionHandlers>,
    RequireAuth(user): RequireAuth,
    Path(session_id): Path<String>,
) -> Response {
    let session_id = match session_id.parse::<SessionId>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request("Invalid session ID")),
            )
                .into_response()
        }
    };

    let query = GetSessionQuery {
        session_id,
        user_id: user.id,
    };

    match handlers.get_handler.handle(query).await {
        Ok(view) => {
            let response: SessionResponse = view.into();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => handle_session_error(e),
    }
}

/// GET /api/sessions - List user's sessions
pub async fn list_sessions(
    State(handlers): State<SessionHandlers>,
    RequireAuth(user): RequireAuth,
    Query(query_params): Query<ListSessionsQuery>,
) -> Response {
    let query = ListUserSessionsQuery {
        user_id: user.id,
        page: query_params.page,
        per_page: query_params.per_page,
        status: query_params.status,
        include_archived: query_params.include_archived,
    };

    match handlers.list_handler.handle(query).await {
        Ok(list) => {
            let response: SessionListResponse = list.into();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => handle_session_error(e),
    }
}

/// PATCH /api/sessions/:id/rename - Rename a session
pub async fn rename_session(
    State(handlers): State<SessionHandlers>,
    RequireAuth(user): RequireAuth,
    Path(session_id): Path<String>,
    Json(req): Json<RenameSessionRequest>,
) -> Response {
    let session_id = match session_id.parse::<SessionId>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request("Invalid session ID")),
            )
                .into_response()
        }
    };

    let cmd = RenameSessionCommand {
        session_id,
        user_id: user.id.clone(),
        new_title: req.title,
    };

    let metadata = CommandMetadata::new(user.id).with_correlation_id("http-request");

    match handlers.rename_handler.handle(cmd, metadata).await {
        Ok(_) => {
            let response = SessionCommandResponse {
                session_id: session_id.to_string(),
                message: "Session renamed successfully".to_string(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => handle_session_error(e),
    }
}

/// POST /api/sessions/:id/archive - Archive a session
pub async fn archive_session(
    State(handlers): State<SessionHandlers>,
    RequireAuth(user): RequireAuth,
    Path(session_id): Path<String>,
) -> Response {
    let session_id = match session_id.parse::<SessionId>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request("Invalid session ID")),
            )
                .into_response()
        }
    };

    let cmd = ArchiveSessionCommand {
        session_id,
        user_id: user.id.clone(),
    };

    let metadata = CommandMetadata::new(user.id).with_correlation_id("http-request");

    match handlers.archive_handler.handle(cmd, metadata).await {
        Ok(_) => {
            let response = SessionCommandResponse {
                session_id: session_id.to_string(),
                message: "Session archived successfully".to_string(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => handle_session_error(e),
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Error handling
// ════════════════════════════════════════════════════════════════════════════

fn handle_session_error(error: SessionError) -> Response {
    match error {
        SessionError::NotFound(id) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::not_found("Session", &id.to_string())),
        )
            .into_response(),
        SessionError::Forbidden => (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::forbidden("Permission denied")),
        )
            .into_response(),
        SessionError::ValidationFailed { field, message } => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::bad_request(format!(
                "Validation failed for {}: {}",
                field, message
            ))),
        )
            .into_response(),
        SessionError::AccessDenied(reason) => (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::forbidden(format!("Access denied: {:?}", reason))),
        )
            .into_response(),
        SessionError::Infrastructure(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::internal(msg)),
        )
            .into_response(),
        SessionError::AlreadyArchived => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::bad_request(
                "Cannot modify an archived session",
            )),
        )
            .into_response(),
        SessionError::InvalidState(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::bad_request(msg)),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_error_not_found_maps_to_404() {
        use crate::domain::foundation::SessionId;
        let error = SessionError::NotFound(SessionId::new());
        let response = handle_session_error(error);
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn session_error_forbidden_maps_to_403() {
        let error = SessionError::Forbidden;
        let response = handle_session_error(error);
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn session_error_validation_failed_maps_to_400() {
        let error = SessionError::ValidationFailed {
            field: "title".to_string(),
            message: "Too long".to_string(),
        };
        let response = handle_session_error(error);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
