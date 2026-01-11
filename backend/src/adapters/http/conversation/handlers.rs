//! HTTP handlers for conversation endpoints.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::adapters::http::middleware::RequireAuth;
use crate::application::handlers::conversation::{
    GetConversationHandler, GetConversationQuery, SendMessageCommand, SendMessageHandler,
};
use crate::domain::foundation::{ComponentId, DomainError, ErrorCode};

use super::dto::{
    ConversationResponse, SendMessageRequest, SendMessageResponse,
};

// ════════════════════════════════════════════════════════════════════════════
// Handler state
// ════════════════════════════════════════════════════════════════════════════

#[derive(Clone)]
pub struct ConversationHandlers {
    send_message_handler: Arc<SendMessageHandler>,
    get_conversation_handler: Arc<GetConversationHandler>,
}

impl ConversationHandlers {
    pub fn new(
        send_message_handler: Arc<SendMessageHandler>,
        get_conversation_handler: Arc<GetConversationHandler>,
    ) -> Self {
        Self {
            send_message_handler,
            get_conversation_handler,
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════
// HTTP handlers
// ════════════════════════════════════════════════════════════════════════════

/// POST /api/conversations/component/:component_id/messages - Send a message
pub async fn send_message(
    State(handlers): State<ConversationHandlers>,
    RequireAuth(_user): RequireAuth,
    Path(component_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Response {
    // Parse component ID
    let component_id = match component_id.parse::<ComponentId>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid component ID"
                })),
            )
                .into_response()
        }
    };

    // Execute command
    let cmd = SendMessageCommand {
        component_id,
        content: req.content,
    };

    match handlers.send_message_handler.handle(cmd).await {
        Ok(result) => {
            let response = SendMessageResponse {
                conversation_id: result.conversation_id.to_string(),
                message: (&result.message).into(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => handle_conversation_error(e),
    }
}

/// GET /api/conversations/component/:component_id - Get conversation by component ID
pub async fn get_conversation_by_component(
    State(handlers): State<ConversationHandlers>,
    RequireAuth(_user): RequireAuth,
    Path(component_id): Path<String>,
) -> Response {
    // Parse component ID
    let component_id = match component_id.parse::<ComponentId>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid component ID"
                })),
            )
                .into_response()
        }
    };

    // Execute query
    let query = GetConversationQuery { component_id };

    match handlers.get_conversation_handler.handle(query).await {
        Ok(view) => {
            let response = ConversationResponse::from(view);
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => handle_conversation_error(e),
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Error handling
// ════════════════════════════════════════════════════════════════════════════

fn handle_conversation_error(error: DomainError) -> Response {
    let (status, message) = match error.code {
        ErrorCode::ConversationNotFound => (StatusCode::NOT_FOUND, error.message),
        ErrorCode::InvalidStateTransition => (StatusCode::BAD_REQUEST, error.message),
        ErrorCode::DatabaseError => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
    };

    (
        status,
        Json(serde_json::json!({
            "error": message
        })),
    )
        .into_response()
}
