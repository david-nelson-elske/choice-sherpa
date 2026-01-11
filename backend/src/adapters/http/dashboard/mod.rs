//! Dashboard HTTP adapter module.
//!
//! Provides REST API endpoints for dashboard queries.

pub mod dto;
pub mod handlers;
pub mod routes;

pub use dto::ErrorResponse;
pub use handlers::DashboardAppState;
pub use routes::dashboard_routes;
