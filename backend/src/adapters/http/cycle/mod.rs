//! HTTP adapter for the cycle module.
//!
//! This module exposes cycle operations via REST endpoints.
//!
//! # Current Endpoints
//!
//! - `POST /api/cycles` - Create a new cycle within a session
//! - `POST /api/cycles/{id}/branch` - Branch an existing cycle at a component
//!
//! # Future Endpoints
//!
//! Additional endpoints will be added as the corresponding application layer
//! handlers are implemented (archive, complete, component operations, queries).

pub mod dto;
pub mod handlers;
pub mod routes;

// Re-export commonly used types
pub use handlers::CycleAppState;
pub use routes::cycle_router;
