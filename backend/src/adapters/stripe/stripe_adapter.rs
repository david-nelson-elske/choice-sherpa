//! Stripe payment provider adapter.
//!
//! Implements the `PaymentProvider` trait for Stripe API integration.
//! Handles customer management, subscriptions, checkout sessions, and webhook verification.
//!
//! # Security
//!
//! - HMAC-SHA256 signature verification with constant-time comparison
//! - Timestamp validation (5-minute window) for replay attack prevention
//! - Secrets handled via `secrecy::SecretString`
//!
//! # Configuration
//!
//! ```ignore
//! let config = StripeConfig::new(api_key, webhook_secret);
//! let adapter = StripePaymentAdapter::new(config);
//! ```

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::domain::membership::MembershipTier;
use crate::ports::{
    CheckoutSession, CreateCheckoutRequest, CreateCustomerRequest, CreateSubscriptionRequest,
    Customer, PaymentError, PaymentErrorCode, PaymentProvider, PortalSession, Subscription,
    SubscriptionStatus, WebhookEvent, WebhookEventData, WebhookEventType,
};

use super::webhook_types::{hex_encode, SignatureHeader, StripeCheckoutSession, StripeWebhookEvent};

type HmacSha256 = Hmac<Sha256>;

/// Maximum age for webhook events (5 minutes).
const MAX_TIMESTAMP_AGE_SECS: i64 = 300;

/// Clock skew tolerance for future timestamps (60 seconds).
const MAX_FUTURE_TOLERANCE_SECS: i64 = 60;

/// Stripe API configuration.
#[derive(Clone)]
pub struct StripeConfig {
    /// Stripe secret API key (sk_live_... or sk_test_...).
    api_key: SecretString,

    /// Webhook signing secret (whsec_...).
    webhook_secret: SecretString,

    /// Base URL for Stripe API (default: https://api.stripe.com).
    api_base_url: String,

    /// Whether to require livemode events in production.
    require_livemode: bool,
}

impl StripeConfig {
    /// Create a new Stripe configuration.
    pub fn new(api_key: impl Into<String>, webhook_secret: impl Into<String>) -> Self {
        Self {
            api_key: SecretString::new(api_key.into()),
            webhook_secret: SecretString::new(webhook_secret.into()),
            api_base_url: "https://api.stripe.com".to_string(),
            require_livemode: false,
        }
    }

    /// Create configuration from environment variables.
    ///
    /// Reads:
    /// - `STRIPE_API_KEY`
    /// - `STRIPE_WEBHOOK_SECRET`
    /// - `STRIPE_REQUIRE_LIVEMODE` (optional, defaults to false)
    pub fn from_env() -> Result<Self, std::env::VarError> {
        let api_key = std::env::var("STRIPE_API_KEY")?;
        let webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET")?;
        let require_livemode = std::env::var("STRIPE_REQUIRE_LIVEMODE")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        Ok(Self {
            api_key: SecretString::new(api_key),
            webhook_secret: SecretString::new(webhook_secret),
            api_base_url: "https://api.stripe.com".to_string(),
            require_livemode,
        })
    }

    /// Set a custom API base URL (for testing).
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.api_base_url = url.into();
        self
    }

    /// Require livemode events in production.
    pub fn with_require_livemode(mut self, require: bool) -> Self {
        self.require_livemode = require;
        self
    }
}

/// Stripe payment provider adapter.
///
/// Implements `PaymentProvider` for Stripe API integration.
pub struct StripePaymentAdapter {
    config: StripeConfig,
    http_client: reqwest::Client,
}

impl StripePaymentAdapter {
    /// Create a new Stripe adapter with the given configuration.
    pub fn new(config: StripeConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }

    /// Verify webhook signature using HMAC-SHA256.
    ///
    /// # Security
    ///
    /// - Uses constant-time comparison to prevent timing attacks
    /// - Validates timestamp to prevent replay attacks
    fn verify_signature(&self, payload: &[u8], header: &SignatureHeader) -> Result<(), PaymentError> {
        // 1. Validate timestamp (prevent replay attacks)
        let now = chrono::Utc::now().timestamp();
        let age = now - header.timestamp;

        if age > MAX_TIMESTAMP_AGE_SECS {
            tracing::warn!(
                event_timestamp = header.timestamp,
                current_time = now,
                age_secs = age,
                "Webhook event too old - possible replay attack"
            );
            return Err(PaymentError::invalid_webhook(format!(
                "Event too old ({} seconds)",
                age
            )));
        }

        if age < -MAX_FUTURE_TOLERANCE_SECS {
            tracing::warn!(
                event_timestamp = header.timestamp,
                current_time = now,
                "Webhook event from future - clock skew or manipulation"
            );
            return Err(PaymentError::invalid_webhook("Event timestamp in future"));
        }

        // 2. Compute expected signature
        let signed_payload = format!(
            "{}.{}",
            header.timestamp,
            String::from_utf8_lossy(payload)
        );

        let mut mac = HmacSha256::new_from_slice(
            self.config.webhook_secret.expose_secret().as_bytes(),
        )
        .expect("HMAC can take key of any size");

        mac.update(signed_payload.as_bytes());
        let expected = mac.finalize().into_bytes();

        // 3. Constant-time comparison
        let expected_bytes: &[u8] = expected.as_slice();
        let provided_bytes: &[u8] = &header.v1_signature;

        if expected_bytes.ct_eq(provided_bytes).unwrap_u8() != 1 {
            tracing::warn!(
                expected_signature = hex_encode(expected_bytes),
                "Invalid webhook signature"
            );
            return Err(PaymentError::invalid_webhook("Invalid signature"));
        }

        Ok(())
    }

    /// Parse a Stripe event and convert to domain types.
    fn parse_event(&self, payload: &[u8]) -> Result<(StripeWebhookEvent, WebhookEvent), PaymentError> {
        let stripe_event: StripeWebhookEvent = serde_json::from_slice(payload).map_err(|e| {
            tracing::warn!(error = %e, "Failed to parse webhook payload");
            PaymentError::invalid_webhook(format!("Invalid JSON: {}", e))
        })?;

        // Check livemode if required
        if self.config.require_livemode && !stripe_event.livemode {
            tracing::warn!(
                event_id = %stripe_event.id,
                "Rejected test mode event in production"
            );
            return Err(PaymentError::invalid_webhook(
                "Test mode events not allowed in production",
            ));
        }

        // Convert event type
        let event_type = match stripe_event.event_type.as_str() {
            "checkout.session.completed" => WebhookEventType::CheckoutSessionCompleted,
            "customer.subscription.created" => WebhookEventType::SubscriptionCreated,
            "customer.subscription.updated" => WebhookEventType::SubscriptionUpdated,
            "customer.subscription.deleted" => WebhookEventType::SubscriptionDeleted,
            "invoice.paid" => WebhookEventType::InvoicePaid,
            "invoice.payment_failed" => WebhookEventType::InvoicePaymentFailed,
            "customer.subscription.trial_will_end" => WebhookEventType::TrialWillEnd,
            other => WebhookEventType::Unknown(other.to_string()),
        };

        // Convert event data based on type
        let data = self.extract_event_data(&stripe_event)?;

        let webhook_event = WebhookEvent {
            id: stripe_event.id.clone(),
            event_type,
            data,
            created_at: stripe_event.created,
        };

        Ok((stripe_event, webhook_event))
    }

    /// Extract event data from Stripe event into domain format.
    fn extract_event_data(&self, event: &StripeWebhookEvent) -> Result<WebhookEventData, PaymentError> {
        match event.event_type.as_str() {
            "checkout.session.completed" => {
                let session: StripeCheckoutSession =
                    serde_json::from_value(event.data.object.clone()).map_err(|e| {
                        PaymentError::invalid_webhook(format!("Invalid checkout session: {}", e))
                    })?;

                Ok(WebhookEventData::Checkout {
                    session_id: session.id,
                    customer_id: session.customer.unwrap_or_default(),
                    subscription_id: session.subscription,
                    user_id: session.metadata.get("user_id").cloned(),
                })
            }

            s if s.starts_with("customer.subscription.") => {
                let sub: super::webhook_types::StripeSubscription =
                    serde_json::from_value(event.data.object.clone()).map_err(|e| {
                        PaymentError::invalid_webhook(format!("Invalid subscription: {}", e))
                    })?;

                let status = match sub.status.as_str() {
                    "active" => SubscriptionStatus::Active,
                    "past_due" => SubscriptionStatus::PastDue,
                    "canceled" => SubscriptionStatus::Canceled,
                    "unpaid" | "incomplete_expired" => SubscriptionStatus::IncompleteExpired,
                    "incomplete" => SubscriptionStatus::Incomplete,
                    "trialing" => SubscriptionStatus::Trialing,
                    "paused" => SubscriptionStatus::Paused,
                    _ => SubscriptionStatus::Unknown,
                };

                Ok(WebhookEventData::Subscription {
                    subscription_id: sub.id,
                    customer_id: sub.customer,
                    status,
                    current_period_end: sub.current_period_end,
                })
            }

            s if s.starts_with("invoice.") => {
                let invoice: super::webhook_types::StripeInvoice =
                    serde_json::from_value(event.data.object.clone()).map_err(|e| {
                        PaymentError::invalid_webhook(format!("Invalid invoice: {}", e))
                    })?;

                Ok(WebhookEventData::Invoice {
                    invoice_id: invoice.id,
                    customer_id: invoice.customer,
                    subscription_id: invoice.subscription,
                    amount_paid: invoice.amount_paid,
                    currency: invoice.currency,
                })
            }

            _ => {
                // Return raw JSON for unknown event types
                Ok(WebhookEventData::Raw {
                    json: serde_json::to_string(&event.data.object).unwrap_or_default(),
                })
            }
        }
    }

    /// Get price ID for a membership tier.
    fn get_price_id(&self, tier: MembershipTier) -> Result<&'static str, PaymentError> {
        match tier {
            MembershipTier::Monthly => Ok("price_monthly_cad_1999"), // TODO: Configure via env
            MembershipTier::Annual => Ok("price_annual_cad_14999"),
            MembershipTier::Free => Err(PaymentError::new(
                PaymentErrorCode::ProviderError,
                "Free tier does not have a Stripe price",
            )),
        }
    }
}

#[async_trait]
impl PaymentProvider for StripePaymentAdapter {
    async fn create_customer(
        &self,
        request: CreateCustomerRequest,
    ) -> Result<Customer, PaymentError> {
        let url = format!("{}/v1/customers", self.config.api_base_url);

        let mut params = vec![
            ("email", request.email.clone()),
            ("metadata[user_id]", request.user_id.to_string()),
        ];

        if let Some(name) = &request.name {
            params.push(("name", name.clone()));
        }

        let response = self
            .http_client
            .post(&url)
            .basic_auth(self.config.api_key.expose_secret(), Option::<&str>::None)
            .form(&params)
            .send()
            .await
            .map_err(|e| PaymentError::network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!(error = %error_text, "Stripe create_customer failed");
            return Err(PaymentError::new(
                PaymentErrorCode::ProviderError,
                format!("Stripe API error: {}", error_text),
            ));
        }

        let stripe_customer: super::webhook_types::StripeCustomer =
            response.json().await.map_err(|e| {
                PaymentError::new(
                    PaymentErrorCode::ProviderError,
                    format!("Failed to parse Stripe response: {}", e),
                )
            })?;

        Ok(Customer {
            id: stripe_customer.id,
            email: stripe_customer.email.unwrap_or(request.email),
            name: stripe_customer.name.or(request.name),
            created_at: stripe_customer.created,
        })
    }

    async fn get_customer(&self, customer_id: &str) -> Result<Option<Customer>, PaymentError> {
        let url = format!("{}/v1/customers/{}", self.config.api_base_url, customer_id);

        let response = self
            .http_client
            .get(&url)
            .basic_auth(self.config.api_key.expose_secret(), Option::<&str>::None)
            .send()
            .await
            .map_err(|e| PaymentError::network(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(PaymentError::new(
                PaymentErrorCode::ProviderError,
                format!("Stripe API error: {}", error_text),
            ));
        }

        let stripe_customer: super::webhook_types::StripeCustomer =
            response.json().await.map_err(|e| {
                PaymentError::new(
                    PaymentErrorCode::ProviderError,
                    format!("Failed to parse Stripe response: {}", e),
                )
            })?;

        if stripe_customer.deleted {
            return Ok(None);
        }

        Ok(Some(Customer {
            id: stripe_customer.id,
            email: stripe_customer.email.unwrap_or_default(),
            name: stripe_customer.name,
            created_at: stripe_customer.created,
        }))
    }

    async fn create_subscription(
        &self,
        request: CreateSubscriptionRequest,
    ) -> Result<Subscription, PaymentError> {
        let url = format!("{}/v1/subscriptions", self.config.api_base_url);
        let price_id = self.get_price_id(request.tier)?;

        let mut params = vec![
            ("customer", request.customer_id.clone()),
            ("items[0][price]", price_id.to_string()),
        ];

        if let Some(idempotency_key) = &request.idempotency_key {
            params.push(("idempotency_key", idempotency_key.clone()));
        }

        let response = self
            .http_client
            .post(&url)
            .basic_auth(self.config.api_key.expose_secret(), Option::<&str>::None)
            .form(&params)
            .send()
            .await
            .map_err(|e| PaymentError::network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(PaymentError::new(
                PaymentErrorCode::ProviderError,
                format!("Stripe API error: {}", error_text),
            ));
        }

        let stripe_sub: super::webhook_types::StripeSubscription =
            response.json().await.map_err(|e| {
                PaymentError::new(
                    PaymentErrorCode::ProviderError,
                    format!("Failed to parse Stripe response: {}", e),
                )
            })?;

        Ok(Subscription {
            id: stripe_sub.id,
            customer_id: stripe_sub.customer,
            status: match stripe_sub.status.as_str() {
                "active" => SubscriptionStatus::Active,
                "past_due" => SubscriptionStatus::PastDue,
                "canceled" => SubscriptionStatus::Canceled,
                "trialing" => SubscriptionStatus::Trialing,
                _ => SubscriptionStatus::Unknown,
            },
            current_period_start: stripe_sub.current_period_start,
            current_period_end: stripe_sub.current_period_end,
            cancel_at_period_end: stripe_sub.cancel_at_period_end,
            canceled_at: stripe_sub.canceled_at,
        })
    }

    async fn get_subscription(
        &self,
        subscription_id: &str,
    ) -> Result<Option<Subscription>, PaymentError> {
        let url = format!(
            "{}/v1/subscriptions/{}",
            self.config.api_base_url, subscription_id
        );

        let response = self
            .http_client
            .get(&url)
            .basic_auth(self.config.api_key.expose_secret(), Option::<&str>::None)
            .send()
            .await
            .map_err(|e| PaymentError::network(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(PaymentError::new(
                PaymentErrorCode::ProviderError,
                format!("Stripe API error: {}", error_text),
            ));
        }

        let stripe_sub: super::webhook_types::StripeSubscription =
            response.json().await.map_err(|e| {
                PaymentError::new(
                    PaymentErrorCode::ProviderError,
                    format!("Failed to parse Stripe response: {}", e),
                )
            })?;

        Ok(Some(Subscription {
            id: stripe_sub.id,
            customer_id: stripe_sub.customer,
            status: match stripe_sub.status.as_str() {
                "active" => SubscriptionStatus::Active,
                "past_due" => SubscriptionStatus::PastDue,
                "canceled" => SubscriptionStatus::Canceled,
                "trialing" => SubscriptionStatus::Trialing,
                _ => SubscriptionStatus::Unknown,
            },
            current_period_start: stripe_sub.current_period_start,
            current_period_end: stripe_sub.current_period_end,
            cancel_at_period_end: stripe_sub.cancel_at_period_end,
            canceled_at: stripe_sub.canceled_at,
        }))
    }

    async fn cancel_subscription(
        &self,
        subscription_id: &str,
        at_period_end: bool,
    ) -> Result<Subscription, PaymentError> {
        let url = format!(
            "{}/v1/subscriptions/{}",
            self.config.api_base_url, subscription_id
        );

        let response = if at_period_end {
            // Update subscription to cancel at period end
            self.http_client
                .post(&url)
                .basic_auth(self.config.api_key.expose_secret(), Option::<&str>::None)
                .form(&[("cancel_at_period_end", "true")])
                .send()
                .await
        } else {
            // Immediately cancel
            self.http_client
                .delete(&url)
                .basic_auth(self.config.api_key.expose_secret(), Option::<&str>::None)
                .send()
                .await
        }
        .map_err(|e| PaymentError::network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(PaymentError::new(
                PaymentErrorCode::ProviderError,
                format!("Stripe API error: {}", error_text),
            ));
        }

        let stripe_sub: super::webhook_types::StripeSubscription =
            response.json().await.map_err(|e| {
                PaymentError::new(
                    PaymentErrorCode::ProviderError,
                    format!("Failed to parse Stripe response: {}", e),
                )
            })?;

        Ok(Subscription {
            id: stripe_sub.id,
            customer_id: stripe_sub.customer,
            status: match stripe_sub.status.as_str() {
                "active" => SubscriptionStatus::Active,
                "canceled" => SubscriptionStatus::Canceled,
                _ => SubscriptionStatus::Unknown,
            },
            current_period_start: stripe_sub.current_period_start,
            current_period_end: stripe_sub.current_period_end,
            cancel_at_period_end: stripe_sub.cancel_at_period_end,
            canceled_at: stripe_sub.canceled_at,
        })
    }

    async fn update_subscription(
        &self,
        subscription_id: &str,
        new_tier: MembershipTier,
    ) -> Result<Subscription, PaymentError> {
        // First, get the current subscription to find the item ID
        let current = self
            .get_subscription(subscription_id)
            .await?
            .ok_or_else(|| PaymentError::not_found("Subscription"))?;

        let url = format!(
            "{}/v1/subscriptions/{}",
            self.config.api_base_url, subscription_id
        );

        let new_price_id = self.get_price_id(new_tier)?;

        // Note: This is simplified - real implementation would need to get the item ID
        let response = self
            .http_client
            .post(&url)
            .basic_auth(self.config.api_key.expose_secret(), Option::<&str>::None)
            .form(&[("items[0][price]", new_price_id)])
            .send()
            .await
            .map_err(|e| PaymentError::network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(PaymentError::new(
                PaymentErrorCode::ProviderError,
                format!("Stripe API error: {}", error_text),
            ));
        }

        let stripe_sub: super::webhook_types::StripeSubscription =
            response.json().await.map_err(|e| {
                PaymentError::new(
                    PaymentErrorCode::ProviderError,
                    format!("Failed to parse Stripe response: {}", e),
                )
            })?;

        Ok(Subscription {
            id: stripe_sub.id,
            customer_id: stripe_sub.customer,
            status: match stripe_sub.status.as_str() {
                "active" => SubscriptionStatus::Active,
                _ => current.status,
            },
            current_period_start: stripe_sub.current_period_start,
            current_period_end: stripe_sub.current_period_end,
            cancel_at_period_end: stripe_sub.cancel_at_period_end,
            canceled_at: stripe_sub.canceled_at,
        })
    }

    async fn create_checkout_session(
        &self,
        request: CreateCheckoutRequest,
    ) -> Result<CheckoutSession, PaymentError> {
        let url = format!("{}/v1/checkout/sessions", self.config.api_base_url);
        let price_id = self.get_price_id(request.tier)?;

        let mut params = vec![
            ("mode", "subscription".to_string()),
            ("customer_email", request.email),
            ("line_items[0][price]", price_id.to_string()),
            ("line_items[0][quantity]", "1".to_string()),
            ("success_url", request.success_url),
            ("cancel_url", request.cancel_url),
            ("metadata[user_id]", request.user_id.to_string()),
        ];

        if let Some(promo) = request.promo_code {
            params.push(("discounts[0][coupon]", promo));
        }

        let response = self
            .http_client
            .post(&url)
            .basic_auth(self.config.api_key.expose_secret(), Option::<&str>::None)
            .form(&params)
            .send()
            .await
            .map_err(|e| PaymentError::network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(PaymentError::new(
                PaymentErrorCode::ProviderError,
                format!("Stripe API error: {}", error_text),
            ));
        }

        let stripe_session: StripeCheckoutSession = response.json().await.map_err(|e| {
            PaymentError::new(
                PaymentErrorCode::ProviderError,
                format!("Failed to parse Stripe response: {}", e),
            )
        })?;

        // Stripe checkout sessions expire after 24 hours by default
        let expires_at = chrono::Utc::now().timestamp() + 24 * 60 * 60;

        // Generate URL - Stripe provides the actual checkout URL via their API response
        let url = stripe_session.success_url.unwrap_or_else(|| {
            format!("https://checkout.stripe.com/c/pay/{}", &stripe_session.id)
        });

        Ok(CheckoutSession {
            id: stripe_session.id,
            url,
            expires_at,
        })
    }

    async fn create_portal_session(
        &self,
        customer_id: &str,
        return_url: &str,
    ) -> Result<PortalSession, PaymentError> {
        let url = format!(
            "{}/v1/billing_portal/sessions",
            self.config.api_base_url
        );

        let response = self
            .http_client
            .post(&url)
            .basic_auth(self.config.api_key.expose_secret(), Option::<&str>::None)
            .form(&[
                ("customer", customer_id),
                ("return_url", return_url),
            ])
            .send()
            .await
            .map_err(|e| PaymentError::network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(PaymentError::new(
                PaymentErrorCode::ProviderError,
                format!("Stripe API error: {}", error_text),
            ));
        }

        #[derive(Deserialize)]
        struct PortalSessionResponse {
            id: String,
            url: String,
        }

        let portal: PortalSessionResponse = response.json().await.map_err(|e| {
            PaymentError::new(
                PaymentErrorCode::ProviderError,
                format!("Failed to parse Stripe response: {}", e),
            )
        })?;

        Ok(PortalSession {
            id: portal.id,
            url: portal.url,
        })
    }

    async fn verify_webhook(
        &self,
        payload: &[u8],
        signature: &str,
    ) -> Result<WebhookEvent, PaymentError> {
        // 1. Parse signature header
        let header = SignatureHeader::parse(signature).map_err(|e| {
            tracing::warn!(error = %e, "Failed to parse Stripe-Signature header");
            PaymentError::invalid_webhook(e.to_string())
        })?;

        // 2. Verify signature (includes timestamp validation)
        self.verify_signature(payload, &header)?;

        // 3. Parse and convert event
        let (_stripe_event, webhook_event) = self.parse_event(payload)?;

        tracing::info!(
            event_id = %webhook_event.id,
            event_type = ?webhook_event.event_type,
            "Webhook signature verified"
        );

        Ok(webhook_event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> StripeConfig {
        StripeConfig::new("sk_test_key", "whsec_test_secret")
    }

    fn create_test_signature(secret: &str, timestamp: i64, payload: &str) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let signed_payload = format!("{}.{}", timestamp, payload);
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(signed_payload.as_bytes());
        let result = mac.finalize().into_bytes();

        format!("t={},v1={}", timestamp, hex_encode(&result))
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Configuration Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn config_new_sets_defaults() {
        let config = StripeConfig::new("api_key", "webhook_secret");
        assert_eq!(config.api_base_url, "https://api.stripe.com");
        assert!(!config.require_livemode);
    }

    #[test]
    fn config_with_base_url() {
        let config = StripeConfig::new("key", "secret").with_base_url("http://localhost:8080");
        assert_eq!(config.api_base_url, "http://localhost:8080");
    }

    #[test]
    fn config_with_require_livemode() {
        let config = StripeConfig::new("key", "secret").with_require_livemode(true);
        assert!(config.require_livemode);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Signature Verification Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn verify_signature_valid() {
        let adapter = StripePaymentAdapter::new(test_config());
        let payload = r#"{"id":"evt_test"}"#;
        let timestamp = chrono::Utc::now().timestamp();
        let signature = create_test_signature("whsec_test_secret", timestamp, payload);

        let header = SignatureHeader::parse(&signature).unwrap();
        let result = adapter.verify_signature(payload.as_bytes(), &header);

        assert!(result.is_ok());
    }

    #[test]
    fn verify_signature_invalid() {
        let adapter = StripePaymentAdapter::new(test_config());
        let payload = r#"{"id":"evt_test"}"#;
        let timestamp = chrono::Utc::now().timestamp();

        // Create signature with wrong secret
        let signature = create_test_signature("wrong_secret", timestamp, payload);

        let header = SignatureHeader::parse(&signature).unwrap();
        let result = adapter.verify_signature(payload.as_bytes(), &header);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().code,
            PaymentErrorCode::InvalidWebhook
        ));
    }

    #[test]
    fn verify_signature_expired_timestamp() {
        let adapter = StripePaymentAdapter::new(test_config());
        let payload = r#"{"id":"evt_test"}"#;
        let old_timestamp = chrono::Utc::now().timestamp() - 600; // 10 minutes ago

        let signature = create_test_signature("whsec_test_secret", old_timestamp, payload);

        let header = SignatureHeader::parse(&signature).unwrap();
        let result = adapter.verify_signature(payload.as_bytes(), &header);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("too old"));
    }

    #[test]
    fn verify_signature_future_timestamp() {
        let adapter = StripePaymentAdapter::new(test_config());
        let payload = r#"{"id":"evt_test"}"#;
        let future_timestamp = chrono::Utc::now().timestamp() + 120; // 2 minutes in future

        let signature = create_test_signature("whsec_test_secret", future_timestamp, payload);

        let header = SignatureHeader::parse(&signature).unwrap();
        let result = adapter.verify_signature(payload.as_bytes(), &header);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("future"));
    }

    #[test]
    fn verify_signature_small_future_tolerance() {
        let adapter = StripePaymentAdapter::new(test_config());
        let payload = r#"{"id":"evt_test"}"#;
        // 30 seconds in future should be tolerated
        let timestamp = chrono::Utc::now().timestamp() + 30;

        let signature = create_test_signature("whsec_test_secret", timestamp, payload);

        let header = SignatureHeader::parse(&signature).unwrap();
        let result = adapter.verify_signature(payload.as_bytes(), &header);

        assert!(result.is_ok());
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Event Parsing Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn parse_checkout_session_completed() {
        let adapter = StripePaymentAdapter::new(test_config());
        let payload = r#"{
            "id": "evt_test",
            "type": "checkout.session.completed",
            "created": 1704067200,
            "data": {
                "object": {
                    "id": "cs_test",
                    "object": "checkout.session",
                    "customer": "cus_test",
                    "subscription": "sub_test",
                    "payment_status": "paid",
                    "status": "complete",
                    "mode": "subscription",
                    "metadata": {"user_id": "usr_123"}
                }
            },
            "livemode": false,
            "pending_webhooks": 0
        }"#;

        let (_, event) = adapter.parse_event(payload.as_bytes()).unwrap();

        assert_eq!(event.id, "evt_test");
        assert_eq!(event.event_type, WebhookEventType::CheckoutSessionCompleted);
        match event.data {
            WebhookEventData::Checkout {
                customer_id,
                subscription_id,
                user_id,
                ..
            } => {
                assert_eq!(customer_id, "cus_test");
                assert_eq!(subscription_id, Some("sub_test".to_string()));
                assert_eq!(user_id, Some("usr_123".to_string()));
            }
            _ => panic!("Expected Checkout data"),
        }
    }

    #[test]
    fn parse_subscription_updated() {
        let adapter = StripePaymentAdapter::new(test_config());
        let payload = r#"{
            "id": "evt_sub",
            "type": "customer.subscription.updated",
            "created": 1704067200,
            "data": {
                "object": {
                    "id": "sub_test",
                    "object": "subscription",
                    "customer": "cus_test",
                    "status": "active",
                    "current_period_start": 1704067200,
                    "current_period_end": 1706745600
                }
            },
            "livemode": false,
            "pending_webhooks": 0
        }"#;

        let (_, event) = adapter.parse_event(payload.as_bytes()).unwrap();

        assert_eq!(event.event_type, WebhookEventType::SubscriptionUpdated);
        match event.data {
            WebhookEventData::Subscription {
                subscription_id,
                status,
                ..
            } => {
                assert_eq!(subscription_id, "sub_test");
                assert_eq!(status, SubscriptionStatus::Active);
            }
            _ => panic!("Expected Subscription data"),
        }
    }

    #[test]
    fn parse_invoice_payment_failed() {
        let adapter = StripePaymentAdapter::new(test_config());
        let payload = r#"{
            "id": "evt_inv",
            "type": "invoice.payment_failed",
            "created": 1704067200,
            "data": {
                "object": {
                    "id": "in_test",
                    "object": "invoice",
                    "customer": "cus_test",
                    "subscription": "sub_test",
                    "status": "open",
                    "amount_paid": 0,
                    "amount_due": 1999,
                    "currency": "cad"
                }
            },
            "livemode": false,
            "pending_webhooks": 0
        }"#;

        let (_, event) = adapter.parse_event(payload.as_bytes()).unwrap();

        assert_eq!(event.event_type, WebhookEventType::InvoicePaymentFailed);
        match event.data {
            WebhookEventData::Invoice {
                amount_paid,
                currency,
                ..
            } => {
                assert_eq!(amount_paid, 0);
                assert_eq!(currency, "cad");
            }
            _ => panic!("Expected Invoice data"),
        }
    }

    #[test]
    fn parse_unknown_event_type() {
        let adapter = StripePaymentAdapter::new(test_config());
        let payload = r#"{
            "id": "evt_unknown",
            "type": "some.future.event",
            "created": 1704067200,
            "data": {
                "object": {"foo": "bar"}
            },
            "livemode": false,
            "pending_webhooks": 0
        }"#;

        let (_, event) = adapter.parse_event(payload.as_bytes()).unwrap();

        assert!(matches!(
            event.event_type,
            WebhookEventType::Unknown(ref s) if s == "some.future.event"
        ));
        assert!(matches!(event.data, WebhookEventData::Raw { .. }));
    }

    #[test]
    fn parse_rejects_test_mode_in_production() {
        let config = StripeConfig::new("key", "secret").with_require_livemode(true);
        let adapter = StripePaymentAdapter::new(config);

        let payload = r#"{
            "id": "evt_test",
            "type": "checkout.session.completed",
            "created": 1704067200,
            "data": {"object": {}},
            "livemode": false,
            "pending_webhooks": 0
        }"#;

        let result = adapter.parse_event(payload.as_bytes());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Test mode"));
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Price ID Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn get_price_id_monthly() {
        let adapter = StripePaymentAdapter::new(test_config());
        let price = adapter.get_price_id(MembershipTier::Monthly);
        assert!(price.is_ok());
    }

    #[test]
    fn get_price_id_annual() {
        let adapter = StripePaymentAdapter::new(test_config());
        let price = adapter.get_price_id(MembershipTier::Annual);
        assert!(price.is_ok());
    }

    #[test]
    fn get_price_id_free_fails() {
        let adapter = StripePaymentAdapter::new(test_config());
        let result = adapter.get_price_id(MembershipTier::Free);
        assert!(result.is_err());
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Integration Tests (verify_webhook full flow)
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn verify_webhook_valid_signature_and_payload() {
        let adapter = StripePaymentAdapter::new(test_config());

        let payload = r#"{
            "id": "evt_test123",
            "type": "checkout.session.completed",
            "created": 1704067200,
            "data": {
                "object": {
                    "id": "cs_test",
                    "object": "checkout.session",
                    "customer": "cus_test",
                    "payment_status": "paid",
                    "status": "complete",
                    "mode": "subscription",
                    "metadata": {}
                }
            },
            "livemode": false,
            "pending_webhooks": 0
        }"#;

        let timestamp = chrono::Utc::now().timestamp();
        let signature = create_test_signature("whsec_test_secret", timestamp, payload);

        let result = adapter.verify_webhook(payload.as_bytes(), &signature).await;

        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.id, "evt_test123");
        assert_eq!(event.event_type, WebhookEventType::CheckoutSessionCompleted);
    }

    #[tokio::test]
    async fn verify_webhook_rejects_invalid_signature() {
        let adapter = StripePaymentAdapter::new(test_config());
        let payload = r#"{"id":"evt_test"}"#;
        let signature = "t=1704067200,v1=invalid_signature_hex";

        let result = adapter.verify_webhook(payload.as_bytes(), signature).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn verify_webhook_rejects_malformed_header() {
        let adapter = StripePaymentAdapter::new(test_config());
        let payload = r#"{"id":"evt_test"}"#;
        let signature = "malformed_header";

        let result = adapter.verify_webhook(payload.as_bytes(), signature).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn verify_webhook_rejects_invalid_json() {
        let adapter = StripePaymentAdapter::new(test_config());
        let payload = "not valid json";
        let timestamp = chrono::Utc::now().timestamp();
        let signature = create_test_signature("whsec_test_secret", timestamp, payload);

        let result = adapter.verify_webhook(payload.as_bytes(), &signature).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Invalid JSON"));
    }
}
