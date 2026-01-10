//! Stripe payment provider adapter.
//!
//! Implements the `PaymentProvider` port for Stripe integration, including:
//! - Customer management
//! - Subscription lifecycle
//! - Checkout sessions
//! - Webhook signature verification
//!
//! # Security
//!
//! - Webhook signatures use HMAC-SHA256 with constant-time comparison
//! - Timestamps are validated to prevent replay attacks (5-minute window)
//! - All secrets are handled via `secrecy::SecretString`
//!
//! # Configuration
//!
//! Required environment variables:
//! - `STRIPE_API_KEY`: Stripe secret API key
//! - `STRIPE_WEBHOOK_SECRET`: Webhook signing secret (whsec_...)

mod mock_payment_provider;
mod stripe_adapter;
mod webhook_types;

pub use mock_payment_provider::MockPaymentProvider;
pub use stripe_adapter::{StripeConfig, StripePaymentAdapter};
pub use webhook_types::{
    SignatureHeader, SignatureParseError, StripeCheckoutSession, StripeCustomer, StripeInvoice,
    StripeSubscription, StripeWebhookEvent,
};
