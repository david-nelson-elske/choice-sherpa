//! Membership domain module.
//!
//! Handles subscription lifecycle, access control, and payment status.
//!
//! # Module Structure
//!
//! - `status` - MembershipStatus state machine
//! - `webhook_errors` - Webhook processing error types
//! - `stripe_event` - Stripe webhook event types

mod status;
mod stripe_event;
mod webhook_errors;

pub use status::MembershipStatus;
pub use stripe_event::{StripeEvent, StripeEventData, StripeEventType};
pub use webhook_errors::WebhookError;
