//! Membership domain module.
//!
//! Handles subscription lifecycle, access control, and payment status.
//!
//! # Module Structure
//!
//! - `aggregate` - Membership aggregate entity
//! - `events` - Domain events for membership lifecycle
//! - `promo_code` - PromoCode value object for promotional discounts
//! - `status` - MembershipStatus state machine
//! - `tier` - MembershipTier subscription levels
//! - `tier_limits` - Feature limits per tier

mod aggregate;
mod errors;
mod events;
mod promo_code;
mod status;
mod tier;
mod tier_limits;

pub use aggregate::Membership;
pub use errors::MembershipError;
pub use events::{ExpiredReason, MembershipEvent};
pub use promo_code::PromoCode;
pub use status::MembershipStatus;
pub use tier::MembershipTier;
pub use tier_limits::TierLimits;
