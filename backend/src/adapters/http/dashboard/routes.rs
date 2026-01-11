//! HTTP routes for dashboard endpoints.

use axum::routing::get;
use axum::Router;

use super::handlers::{compare_cycles, get_component_detail, get_dashboard_overview, DashboardAppState};

/// Creates the dashboard router with all routes.
pub fn dashboard_routes(state: DashboardAppState) -> Router {
    Router::new()
        // GET /api/sessions/:session_id/dashboard
        .route("/api/sessions/:session_id/dashboard", get(get_dashboard_overview))
        // GET /api/cycles/:cycle_id/components/:component_type/detail
        .route("/api/cycles/:cycle_id/components/:component_type/detail", get(get_component_detail))
        // GET /api/sessions/:session_id/compare
        .route("/api/sessions/:session_id/compare", get(compare_cycles))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routes_compile() {
        // This test ensures routes are correctly defined
        // Actual testing requires integration tests with a running server
    }
}
