//! Route definitions for AI Engine endpoints

use axum::routing::{delete, get, post};
use axum::Router;

use super::handlers::{
    end_conversation, get_conversation_state, send_message, start_conversation, AIEngineAppState,
};

/// Create AI Engine router with all endpoints
///
/// # Endpoints
///
/// - `POST /ai/conversations` - Start new conversation
/// - `POST /ai/conversations/{cycle_id}/messages` - Send message
/// - `GET /ai/conversations/{cycle_id}` - Get conversation state
/// - `DELETE /ai/conversations/{cycle_id}` - End conversation
pub fn routes() -> Router<AIEngineAppState> {
    Router::new()
        .route("/ai/conversations", post(start_conversation))
        .route(
            "/ai/conversations/:cycle_id/messages",
            post(send_message),
        )
        .route("/ai/conversations/:cycle_id", get(get_conversation_state))
        .route("/ai/conversations/:cycle_id", delete(end_conversation))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_creates_valid_router() {
        // Ensures the route configuration compiles and creates a valid router
        let _routes = routes();
    }
}
