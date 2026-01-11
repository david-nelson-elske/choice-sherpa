//! HTTP middleware for axum.
//!
//! This module contains middleware layers for cross-cutting concerns:
//!
//! - `auth` - Authentication middleware and extractors
//! - `rate_limit` - Rate limiting middleware

pub mod auth;
pub mod rate_limit;

pub use auth::{auth_middleware, AuthRejection, AuthState, OptionalAuth, RequireAuth};
pub use rate_limit::{
    rate_limit_middleware, RateLimitCheck, RateLimitRejection, RateLimiterState,
};
