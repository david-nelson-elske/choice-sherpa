//! HTTP adapter for membership endpoints.
//!
//! Exposes the membership domain via REST API:
//! - `GET /api/membership` - Get current user's membership
//! - `GET /api/membership/limits` - Get tier limits for current user
//! - `GET /api/membership/access` - Check if user has access
//! - `POST /api/membership/free` - Create free membership with promo code
//! - `POST /api/membership/checkout` - Start paid checkout flow
//! - `POST /api/membership/cancel` - Cancel membership
//! - `GET /api/membership/portal` - Get Stripe customer portal URL
//! - `POST /api/webhooks/stripe` - Handle Stripe webhooks

pub mod dto;
// TODO: Add when implemented
// pub mod handlers;
// pub mod routes;

pub use dto::*;
