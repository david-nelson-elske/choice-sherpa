//! Membership domain module.
//!
//! Handles subscription lifecycle, access control, and payment status.
//!
//! # Module Structure
//!
//! - `status` - MembershipStatus state machine
//! - `webhook_errors` - Webhook processing error types
//! - `stripe_event` - Stripe webhook event types
//! - `webhook_verifier` - Stripe signature verification

mod status;
mod stripe_event;
mod webhook_errors;
mod webhook_verifier;

pub use status::MembershipStatus;
pub use stripe_event::{StripeEvent, StripeEventData, StripeEventType};
pub use webhook_errors::WebhookError;
pub use webhook_verifier::{SignatureHeader, StripeWebhookVerifier};
