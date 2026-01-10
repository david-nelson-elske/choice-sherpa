//! Axum router configuration for tools endpoints.
//!
//! This module defines the route structure for tool-related API endpoints.

use axum::{
    routing::{get, post},
    Router,
};

use super::handlers::{
    dismiss_revisit, get_confirmations, get_invocation_history, get_revisit_suggestions,
    invoke_tool, list_tools, respond_to_confirmation, ToolsAppState,
};

/// Create the tools API router.
///
/// # Routes
///
/// ## Tool Discovery
/// - `GET /` - List available tools for a component (query: component, format)
///
/// ## Tool Invocation
/// - `POST /invoke` - Invoke a tool
/// - `GET /invocations/:cycle_id` - Get invocation history for a cycle
///
/// ## Revisit Suggestions
/// - `GET /revisits/:cycle_id` - Get pending revisit suggestions for a cycle
/// - `POST /revisits/:id/dismiss` - Dismiss a suggestion
///
/// ## Confirmation Requests
/// - `GET /confirmations/:cycle_id` - Get pending confirmations for a cycle
/// - `POST /confirmations/:id/respond` - Respond to a confirmation
pub fn tools_routes() -> Router<ToolsAppState> {
    Router::new()
        // Tool discovery
        .route("/", get(list_tools))
        // Tool invocation
        .route("/invoke", post(invoke_tool))
        .route("/invocations/{cycle_id}", get(get_invocation_history))
        // Revisit suggestions
        .route("/revisits/{cycle_id}", get(get_revisit_suggestions))
        .route("/revisits/{id}/dismiss", post(dismiss_revisit))
        // Confirmations
        .route("/confirmations/{cycle_id}", get(get_confirmations))
        .route("/confirmations/{id}/respond", post(respond_to_confirmation))
}

/// Create the complete tools module router.
///
/// Suitable for mounting at `/api/tools`.
///
/// # Example
///
/// ```ignore
/// use axum::Router;
/// use crate::adapters::http::tools::{tools_router, ToolsAppState};
///
/// let app_state = ToolsAppState { /* ... */ };
/// let app = Router::new()
///     .nest("/api/tools", tools_router())
///     .with_state(app_state);
/// ```
pub fn tools_router() -> Router<ToolsAppState> {
    tools_routes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_are_defined() {
        // This just verifies the router can be constructed
        // Actual route testing requires integration tests
        let _router = tools_routes();
    }
}
