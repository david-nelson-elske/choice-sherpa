//! Membership domain module.
//!
//! Handles subscription lifecycle, access control, and payment status.
//!
//! # Module Structure
//!
//! - `status` - MembershipStatus state machine
//! - `webhook_errors` - Webhook processing error types

mod status;
mod webhook_errors;

pub use status::MembershipStatus;
pub use webhook_errors::WebhookError;
