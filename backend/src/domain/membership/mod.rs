//! Membership domain module.
//!
//! Handles subscription lifecycle, access control, and payment status.
//!
//! # Module Structure
//!
//! - `status` - MembershipStatus state machine
//! - `tier` - MembershipTier subscription levels
//! - `tier_limits` - Feature limits per tier

mod status;
mod tier;
mod tier_limits;

pub use status::MembershipStatus;
pub use tier::MembershipTier;
pub use tier_limits::TierLimits;
