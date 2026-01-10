//! Axum routes for cycle endpoints.
//!
//! Defines the routing table for all cycle-related HTTP endpoints.

use axum::routing::post;
use axum::Router;

use super::handlers::{branch_cycle, create_cycle, CycleAppState};

/// Creates routes for cycle endpoints.
///
/// Current endpoints:
/// - POST /api/cycles - Create a new cycle
/// - POST /api/cycles/{cycle_id}/branch - Branch an existing cycle
///
/// Future endpoints (once handlers are implemented):
/// - GET /api/cycles/{cycle_id} - Get cycle details
/// - POST /api/cycles/{cycle_id}/archive - Archive a cycle
/// - POST /api/cycles/{cycle_id}/complete - Complete a cycle
/// - GET /api/cycles/{cycle_id}/components/{type} - Get component details
/// - POST /api/cycles/{cycle_id}/components/start - Start a component
/// - POST /api/cycles/{cycle_id}/components/complete - Complete a component
/// - POST /api/cycles/{cycle_id}/components/output - Update component output
/// - GET /api/sessions/{session_id}/cycles/tree - Get cycle tree
pub fn cycle_routes() -> Router<CycleAppState> {
    Router::new()
        .route("/", post(create_cycle))
        .route("/{cycle_id}/branch", post(branch_cycle))
}

/// Combined router with all cycle routes.
pub fn cycle_router() -> Router<CycleAppState> {
    Router::new().nest("/api/cycles", cycle_routes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_routes_creates_valid_router() {
        // Ensures the route configuration compiles and creates a valid router
        let _routes = cycle_routes();
    }

    #[test]
    fn cycle_router_creates_combined_router() {
        let _router = cycle_router();
    }
}
