//! Membership handlers.
//!
//! Command and query handlers for membership lifecycle operations including:
//!
//! ## Commands
//! - Creating free memberships via promo codes
//! - Creating paid memberships via checkout
//! - Cancelling memberships
//! - Processing payment webhooks
//!
//! ## Queries
//! - Get membership details
//! - Check user access
//! - Get membership statistics (admin)

mod cancel_membership;
mod check_access;
mod create_free_membership;
mod create_paid_membership;
mod get_membership;
mod get_membership_stats;
mod handle_payment_webhook;

// Commands
pub use cancel_membership::{CancelMembershipCommand, CancelMembershipHandler, CancelMembershipResult};
pub use create_free_membership::{
    CreateFreeMembershipCommand, CreateFreeMembershipHandler, CreateFreeMembershipResult,
};
pub use create_paid_membership::{
    CreatePaidMembershipCommand, CreatePaidMembershipHandler, CreatePaidMembershipResult,
};
pub use handle_payment_webhook::{
    HandlePaymentWebhookCommand, HandlePaymentWebhookHandler, HandlePaymentWebhookResult,
};

// Queries
pub use check_access::{CheckAccessHandler, CheckAccessQuery, CheckAccessResult};
pub use get_membership::{GetMembershipHandler, GetMembershipQuery, GetMembershipResult};
pub use get_membership_stats::{GetMembershipStatsHandler, GetMembershipStatsQuery, GetMembershipStatsResult};
