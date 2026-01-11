//! HTTP routes for conversation endpoints.

use axum::{
    routing::{get, post},
    Router,
};

use super::handlers::{
    get_conversation_by_component, send_message, ConversationHandlers,
};

/// Creates the conversation router with all endpoints.
pub fn conversation_routes(handlers: ConversationHandlers) -> Router {
    Router::new()
        // Get conversation by component ID
        .route("/component/:component_id", get(get_conversation_by_component))
        // Send message to conversation
        .route("/component/:component_id/messages", post(send_message))
        .with_state(handlers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversation_routes_compiles() {
        // This test just ensures the route definitions compile correctly
        // Actual HTTP testing would require integration tests
    }
}
