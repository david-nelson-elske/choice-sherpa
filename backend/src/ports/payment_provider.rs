//! Payment provider port for external payment processing.
//!
//! Defines the contract for payment gateway integrations (e.g., Stripe).
//! Implementations handle actual payment processing, subscription management,
//! and webhook handling.
//!
//! # Design
//!
//! - **Gateway agnostic**: Interface works with any payment provider
//! - **Subscription-focused**: Optimized for recurring billing
//! - **Idempotent**: Operations can be safely retried

use crate::domain::foundation::{DomainError, UserId};
use crate::domain::membership::MembershipTier;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Port for payment provider integrations.
///
/// Handles customer management, subscription lifecycle, and payment processing.
/// Implementations must ensure idempotency for all operations.
#[async_trait]
pub trait PaymentProvider: Send + Sync {
    /// Create a customer in the payment system.
    ///
    /// Returns the provider's customer ID for future reference.
    async fn create_customer(&self, request: CreateCustomerRequest)
        -> Result<Customer, PaymentError>;

    /// Get customer by provider ID.
    async fn get_customer(&self, customer_id: &str) -> Result<Option<Customer>, PaymentError>;

    /// Create a subscription for a customer.
    ///
    /// Returns the subscription details including provider IDs.
    async fn create_subscription(
        &self,
        request: CreateSubscriptionRequest,
    ) -> Result<Subscription, PaymentError>;

    /// Get subscription by provider ID.
    async fn get_subscription(
        &self,
        subscription_id: &str,
    ) -> Result<Option<Subscription>, PaymentError>;

    /// Cancel a subscription.
    ///
    /// If `at_period_end` is true, subscription remains active until period ends.
    async fn cancel_subscription(
        &self,
        subscription_id: &str,
        at_period_end: bool,
    ) -> Result<Subscription, PaymentError>;

    /// Update subscription to a different plan.
    async fn update_subscription(
        &self,
        subscription_id: &str,
        new_tier: MembershipTier,
    ) -> Result<Subscription, PaymentError>;

    /// Create a checkout session for initial subscription.
    ///
    /// Returns a URL for the customer to complete payment.
    async fn create_checkout_session(
        &self,
        request: CreateCheckoutRequest,
    ) -> Result<CheckoutSession, PaymentError>;

    /// Create a billing portal session for subscription management.
    ///
    /// Returns a URL for the customer to manage their subscription.
    async fn create_portal_session(
        &self,
        customer_id: &str,
        return_url: &str,
    ) -> Result<PortalSession, PaymentError>;

    /// Verify a webhook signature and parse the event.
    ///
    /// Returns the parsed event if valid, error if signature invalid.
    async fn verify_webhook(
        &self,
        payload: &[u8],
        signature: &str,
    ) -> Result<WebhookEvent, PaymentError>;
}

/// Request to create a customer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCustomerRequest {
    /// Internal user ID (stored as metadata).
    pub user_id: UserId,

    /// Customer email address.
    pub email: String,

    /// Customer name (optional).
    pub name: Option<String>,

    /// Idempotency key for safe retries.
    pub idempotency_key: Option<String>,
}

/// Customer in the payment system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    /// Provider's customer ID.
    pub id: String,

    /// Customer email.
    pub email: String,

    /// Customer name.
    pub name: Option<String>,

    /// When the customer was created (provider timestamp).
    pub created_at: i64,
}

/// Request to create a subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionRequest {
    /// Provider's customer ID.
    pub customer_id: String,

    /// Tier to subscribe to.
    pub tier: MembershipTier,

    /// Optional promo/coupon code.
    pub promo_code: Option<String>,

    /// Idempotency key for safe retries.
    pub idempotency_key: Option<String>,
}

/// Subscription in the payment system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// Provider's subscription ID.
    pub id: String,

    /// Provider's customer ID.
    pub customer_id: String,

    /// Current subscription status.
    pub status: SubscriptionStatus,

    /// Current billing period start (Unix timestamp).
    pub current_period_start: i64,

    /// Current billing period end (Unix timestamp).
    pub current_period_end: i64,

    /// Whether subscription cancels at period end.
    pub cancel_at_period_end: bool,

    /// When cancellation was requested (if applicable).
    pub canceled_at: Option<i64>,
}

/// Subscription status from payment provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    /// Subscription is active and current.
    Active,

    /// Payment is past due, grace period active.
    PastDue,

    /// Subscription is canceled (may still be active until period end).
    Canceled,

    /// Subscription has ended.
    Ended,

    /// Subscription is in trial period.
    Trialing,

    /// Initial payment incomplete.
    Incomplete,

    /// Payment failed after retries exhausted.
    IncompleteExpired,

    /// Subscription is paused.
    Paused,

    /// Unknown status from provider.
    Unknown,
}

impl SubscriptionStatus {
    /// Check if subscription grants access.
    pub fn has_access(&self) -> bool {
        matches!(
            self,
            SubscriptionStatus::Active | SubscriptionStatus::Trialing | SubscriptionStatus::PastDue
        )
    }
}

/// Request to create a checkout session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCheckoutRequest {
    /// Internal user ID.
    pub user_id: UserId,

    /// Customer email for pre-fill.
    pub email: String,

    /// Tier to subscribe to.
    pub tier: MembershipTier,

    /// URL to redirect after successful checkout.
    pub success_url: String,

    /// URL to redirect after canceled checkout.
    pub cancel_url: String,

    /// Optional promo/coupon code.
    pub promo_code: Option<String>,
}

/// Checkout session for payment completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutSession {
    /// Provider's session ID.
    pub id: String,

    /// URL for customer to complete checkout.
    pub url: String,

    /// When the session expires (Unix timestamp).
    pub expires_at: i64,
}

/// Portal session for subscription management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalSession {
    /// Provider's session ID.
    pub id: String,

    /// URL for customer to access portal.
    pub url: String,
}

/// Webhook event from payment provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    /// Event ID from provider.
    pub id: String,

    /// Event type.
    pub event_type: WebhookEventType,

    /// Event payload (provider-specific).
    pub data: WebhookEventData,

    /// When the event occurred (Unix timestamp).
    pub created_at: i64,
}

/// Types of webhook events we handle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventType {
    /// Checkout session completed successfully.
    CheckoutSessionCompleted,

    /// Subscription created.
    SubscriptionCreated,

    /// Subscription updated (plan change, etc.).
    SubscriptionUpdated,

    /// Subscription deleted/ended.
    SubscriptionDeleted,

    /// Invoice paid successfully.
    InvoicePaid,

    /// Invoice payment failed.
    InvoicePaymentFailed,

    /// Customer subscription trial ending.
    TrialWillEnd,

    /// Unknown event type.
    Unknown(String),
}

/// Webhook event payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebhookEventData {
    /// Checkout session data.
    #[serde(rename = "checkout")]
    Checkout {
        session_id: String,
        customer_id: String,
        subscription_id: Option<String>,
        user_id: Option<String>,
    },

    /// Subscription data.
    #[serde(rename = "subscription")]
    Subscription {
        subscription_id: String,
        customer_id: String,
        status: SubscriptionStatus,
        current_period_end: i64,
    },

    /// Invoice data.
    #[serde(rename = "invoice")]
    Invoice {
        invoice_id: String,
        customer_id: String,
        subscription_id: Option<String>,
        amount_paid: i64,
        currency: String,
    },

    /// Raw/unknown event data.
    #[serde(rename = "raw")]
    Raw { json: String },
}

/// Errors from payment provider operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentError {
    /// Error code for categorization.
    pub code: PaymentErrorCode,

    /// Human-readable message.
    pub message: String,

    /// Provider's error code (if available).
    pub provider_code: Option<String>,

    /// Whether the operation can be retried.
    pub retryable: bool,
}

impl PaymentError {
    /// Create a new payment error.
    pub fn new(code: PaymentErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            provider_code: None,
            retryable: code.is_retryable(),
        }
    }

    /// Create with provider code.
    pub fn with_provider_code(mut self, code: impl Into<String>) -> Self {
        self.provider_code = Some(code.into());
        self
    }

    /// Create a network error.
    pub fn network(message: impl Into<String>) -> Self {
        Self::new(PaymentErrorCode::NetworkError, message)
    }

    /// Create an authentication error.
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::new(PaymentErrorCode::AuthenticationError, message)
    }

    /// Create a card declined error.
    pub fn card_declined(message: impl Into<String>) -> Self {
        Self::new(PaymentErrorCode::CardDeclined, message)
    }

    /// Create a not found error.
    pub fn not_found(resource: &str) -> Self {
        Self::new(
            PaymentErrorCode::NotFound,
            format!("{} not found", resource),
        )
    }

    /// Create an invalid webhook error.
    pub fn invalid_webhook(message: impl Into<String>) -> Self {
        Self::new(PaymentErrorCode::InvalidWebhook, message)
    }
}

impl std::fmt::Display for PaymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for PaymentError {}

impl From<PaymentError> for DomainError {
    fn from(err: PaymentError) -> Self {
        use crate::domain::foundation::ErrorCode;

        let code = match err.code {
            PaymentErrorCode::CardDeclined | PaymentErrorCode::InsufficientFunds => {
                ErrorCode::PaymentRequired
            }
            PaymentErrorCode::NotFound => ErrorCode::NotFound,
            PaymentErrorCode::InvalidWebhook => ErrorCode::ValidationFailed,
            _ => ErrorCode::ExternalServiceError,
        };

        DomainError::new(code, err.message)
    }
}

/// Payment error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentErrorCode {
    /// Network connectivity issue.
    NetworkError,

    /// API authentication failed.
    AuthenticationError,

    /// Card was declined.
    CardDeclined,

    /// Insufficient funds.
    InsufficientFunds,

    /// Card expired.
    CardExpired,

    /// Invalid card details.
    InvalidCard,

    /// Resource not found.
    NotFound,

    /// Rate limit exceeded.
    RateLimitExceeded,

    /// Invalid webhook signature.
    InvalidWebhook,

    /// Provider API error.
    ProviderError,

    /// Unknown error.
    Unknown,
}

impl PaymentErrorCode {
    /// Check if this error type is typically retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            PaymentErrorCode::NetworkError | PaymentErrorCode::RateLimitExceeded
        )
    }
}

impl std::fmt::Display for PaymentErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PaymentErrorCode::NetworkError => "network_error",
            PaymentErrorCode::AuthenticationError => "authentication_error",
            PaymentErrorCode::CardDeclined => "card_declined",
            PaymentErrorCode::InsufficientFunds => "insufficient_funds",
            PaymentErrorCode::CardExpired => "card_expired",
            PaymentErrorCode::InvalidCard => "invalid_card",
            PaymentErrorCode::NotFound => "not_found",
            PaymentErrorCode::RateLimitExceeded => "rate_limit_exceeded",
            PaymentErrorCode::InvalidWebhook => "invalid_webhook",
            PaymentErrorCode::ProviderError => "provider_error",
            PaymentErrorCode::Unknown => "unknown",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trait object safety test
    #[test]
    fn payment_provider_is_object_safe() {
        fn _accepts_dyn(_provider: &dyn PaymentProvider) {}
    }

    #[test]
    fn subscription_status_access_checks() {
        assert!(SubscriptionStatus::Active.has_access());
        assert!(SubscriptionStatus::Trialing.has_access());
        assert!(SubscriptionStatus::PastDue.has_access());

        assert!(!SubscriptionStatus::Canceled.has_access());
        assert!(!SubscriptionStatus::Ended.has_access());
        assert!(!SubscriptionStatus::Incomplete.has_access());
    }

    #[test]
    fn payment_error_retryable() {
        assert!(PaymentErrorCode::NetworkError.is_retryable());
        assert!(PaymentErrorCode::RateLimitExceeded.is_retryable());

        assert!(!PaymentErrorCode::CardDeclined.is_retryable());
        assert!(!PaymentErrorCode::NotFound.is_retryable());
    }

    #[test]
    fn payment_error_display() {
        let err = PaymentError::card_declined("Your card was declined");
        assert!(err.to_string().contains("card_declined"));
        assert!(err.to_string().contains("Your card was declined"));
    }

    #[test]
    fn payment_error_converts_to_domain_error() {
        let payment_err = PaymentError::card_declined("Declined");
        let domain_err: DomainError = payment_err.into();
        assert!(domain_err.message().contains("Declined"));
    }
}
