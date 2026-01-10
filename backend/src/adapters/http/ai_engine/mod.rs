//! HTTP adapters for AI Engine
//!
//! Exposes REST API endpoints for AI-powered conversation management.

pub mod dto;
pub mod handlers;
pub mod routes;

pub use dto::*;
pub use handlers::AIEngineAppState;
pub use routes::routes;
