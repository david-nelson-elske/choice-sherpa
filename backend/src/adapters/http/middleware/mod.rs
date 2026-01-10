//! HTTP middleware for axum.
//!
//! This module contains middleware layers for cross-cutting concerns:
//!
//! - `auth` - Authentication middleware and extractors

pub mod auth;

pub use auth::{auth_middleware, AuthRejection, AuthState, OptionalAuth, RequireAuth};
