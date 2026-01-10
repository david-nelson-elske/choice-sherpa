//! HTTP adapter for session endpoints.
//!
//! Provides REST API access to session-related operations.

mod dto;
mod handlers;
mod routes;

pub use dto::*;
pub use handlers::SessionAppState;
pub use routes::session_router;
