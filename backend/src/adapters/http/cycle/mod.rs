//! HTTP adapter for cycle endpoints.
//!
//! Exposes cycle operations via REST API:
//! - `GET /api/cycles/:id/document` - Generate decision document
//! - `GET /api/cycles/:id/document?format=summary` - Generate summary document
//! - `GET /api/cycles/:id/document?format=export` - Generate export document

pub mod dto;
pub mod handlers;
pub mod routes;

pub use dto::*;
pub use handlers::{get_document, CycleAppState};
pub use routes::cycle_router;
