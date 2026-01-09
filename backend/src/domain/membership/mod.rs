//! Membership domain module.
//!
//! Handles subscription lifecycle, access control, and payment status.
//!
//! # Module Structure
//!
//! - `aggregate` - Membership aggregate entity
//! - `promo_code` - PromoCode value object for promotional discounts
//! - `status` - MembershipStatus state machine
//! - `tier` - MembershipTier subscription levels
//! - `tier_limits` - Feature limits per tier

mod aggregate;
mod promo_code;
mod status;
mod tier;
mod tier_limits;

pub use aggregate::Membership;
pub use promo_code::PromoCode;
pub use status::MembershipStatus;
pub use tier::MembershipTier;
pub use tier_limits::TierLimits;
