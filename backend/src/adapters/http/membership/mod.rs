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
pub mod handlers;
pub mod routes;

pub use dto::*;
pub use handlers::{
    cancel_membership, check_access, create_checkout, create_free_membership, get_membership,
    get_membership_stats, get_portal_url, get_tier_limits, handle_stripe_webhook,
    MembershipAppState,
};
pub use routes::{membership_router, membership_routes, webhook_routes};
