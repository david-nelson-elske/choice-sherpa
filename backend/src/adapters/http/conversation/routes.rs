//! Axum routes for conversation endpoints.
//!
//! Defines the routing table for all conversation-related HTTP endpoints.

use axum::routing::{get, post};
use axum::Router;

use super::handlers::{get_conversation, get_messages, regenerate_response, ConversationAppState};

/// Creates routes for conversation endpoints.
///
/// REST Endpoints:
/// - GET /api/components/{component_id}/conversation - Get conversation for component
/// - GET /api/conversations/{conversation_id}/messages - Get paginated messages
/// - POST /api/components/{component_id}/conversation/regenerate - Regenerate last response
///
/// WebSocket Endpoints (separate router):
/// - WS /api/components/{component_id}/stream - Real-time AI streaming
pub fn conversation_routes() -> Router<ConversationAppState> {
    Router::new()
        .route("/components/{component_id}/conversation", get(get_conversation))
        .route("/conversations/{conversation_id}/messages", get(get_messages))
        .route("/components/{component_id}/conversation/regenerate", post(regenerate_response))
}

/// Combined router with all conversation routes under /api.
pub fn conversation_router() -> Router<ConversationAppState> {
    Router::new().nest("/api", conversation_routes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversation_routes_creates_valid_router() {
        let _routes = conversation_routes();
    }

    #[test]
    fn conversation_router_creates_combined_router() {
        let _router = conversation_router();
    }
}
