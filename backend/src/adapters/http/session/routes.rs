//! HTTP routes for session endpoints.

use axum::{
    routing::{get, patch, post},
    Router,
};

use super::handlers::{
    archive_session, create_session, get_session, list_sessions, rename_session, SessionHandlers,
};

/// Creates the session router with all endpoints.
pub fn session_routes(handlers: SessionHandlers) -> Router {
    Router::new()
        .route("/", post(create_session))
        .route("/", get(list_sessions))
        .route("/:id", get(get_session))
        .route("/:id/rename", patch(rename_session))
        .route("/:id/archive", post(archive_session))
        .with_state(handlers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_routes_compiles() {
        // This test just ensures the route definitions compile correctly
        // Actual HTTP testing would require integration tests
    }
}
