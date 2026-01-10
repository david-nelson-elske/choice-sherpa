//! Mock payment provider for testing.
//!
//! Provides a configurable mock implementation of `PaymentProvider` for unit
//! and integration tests. Supports:
//! - Pre-configured responses
//! - Error injection
//! - Call tracking
//! - Webhook event simulation

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::domain::membership::MembershipTier;
use crate::ports::{
    CheckoutSession, CreateCheckoutRequest, CreateCustomerRequest, CreateSubscriptionRequest,
    Customer, PaymentError, PaymentProvider, PortalSession, Subscription, SubscriptionStatus,
    WebhookEvent, WebhookEventData, WebhookEventType,
};

/// Mock payment provider for testing.
///
/// # Example
///
/// ```ignore
/// let mock = MockPaymentProvider::new();
///
/// // Configure responses
/// mock.set_customer(Customer { id: "cus_123".into(), ... });
///
/// // Inject errors
/// mock.set_error(PaymentError::card_declined("Test decline"));
///
/// // Use in tests
/// let result = mock.create_customer(request).await;
/// ```
#[derive(Default)]
pub struct MockPaymentProvider {
    /// Inner state (thread-safe for async tests).
    inner: Arc<Mutex<MockState>>,
}

/// Internal mutable state.
#[derive(Default)]
struct MockState {
    /// Pre-configured customers by ID.
    customers: HashMap<String, Customer>,

    /// Pre-configured subscriptions by ID.
    subscriptions: HashMap<String, Subscription>,

    /// Next customer ID to return.
    next_customer: Option<Customer>,

    /// Next subscription to return.
    next_subscription: Option<Subscription>,

    /// Next checkout session to return.
    next_checkout: Option<CheckoutSession>,

    /// Next portal session to return.
    next_portal: Option<PortalSession>,

    /// Next webhook event to return.
    next_webhook_event: Option<WebhookEvent>,

    /// Error to return on next call.
    next_error: Option<PaymentError>,

    /// Specific errors by method name.
    method_errors: HashMap<String, PaymentError>,

    /// Track method calls for assertions.
    call_log: Vec<MethodCall>,

    /// Webhook verification behavior.
    webhook_verify_mode: WebhookVerifyMode,
}

/// Recorded method call for assertions.
#[derive(Debug, Clone)]
pub struct MethodCall {
    pub method: String,
    pub args: Vec<String>,
}

/// How to handle webhook verification.
#[derive(Default, Clone)]
enum WebhookVerifyMode {
    /// Accept any payload and return configured event.
    #[default]
    AcceptAll,

    /// Require specific signature (reserved for future use).
    #[allow(dead_code)]
    RequireSignature(String),

    /// Always fail verification.
    AlwaysFail,
}

impl MockPaymentProvider {
    /// Create a new mock provider with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a mock that fails all webhook verifications.
    pub fn rejecting_webhooks() -> Self {
        let mock = Self::new();
        mock.inner.lock().unwrap().webhook_verify_mode = WebhookVerifyMode::AlwaysFail;
        mock
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Configuration Methods
    // ════════════════════════════════════════════════════════════════════════════

    /// Set the customer to return on next `create_customer` call.
    pub fn set_customer(&self, customer: Customer) {
        self.inner.lock().unwrap().next_customer = Some(customer);
    }

    /// Add a customer to the "database".
    pub fn add_customer(&self, customer: Customer) {
        let id = customer.id.clone();
        self.inner.lock().unwrap().customers.insert(id, customer);
    }

    /// Set the subscription to return on next `create_subscription` call.
    pub fn set_subscription(&self, subscription: Subscription) {
        self.inner.lock().unwrap().next_subscription = Some(subscription);
    }

    /// Add a subscription to the "database".
    pub fn add_subscription(&self, subscription: Subscription) {
        let id = subscription.id.clone();
        self.inner
            .lock()
            .unwrap()
            .subscriptions
            .insert(id, subscription);
    }

    /// Set the checkout session to return.
    pub fn set_checkout_session(&self, session: CheckoutSession) {
        self.inner.lock().unwrap().next_checkout = Some(session);
    }

    /// Set the portal session to return.
    pub fn set_portal_session(&self, session: PortalSession) {
        self.inner.lock().unwrap().next_portal = Some(session);
    }

    /// Set the webhook event to return on verification.
    pub fn set_webhook_event(&self, event: WebhookEvent) {
        self.inner.lock().unwrap().next_webhook_event = Some(event);
    }

    /// Set an error to return on the next call to any method.
    pub fn set_error(&self, error: PaymentError) {
        self.inner.lock().unwrap().next_error = Some(error);
    }

    /// Set an error for a specific method.
    pub fn set_method_error(&self, method: &str, error: PaymentError) {
        self.inner
            .lock()
            .unwrap()
            .method_errors
            .insert(method.to_string(), error);
    }

    /// Clear all configured errors.
    pub fn clear_errors(&self) {
        let mut state = self.inner.lock().unwrap();
        state.next_error = None;
        state.method_errors.clear();
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Call Tracking
    // ════════════════════════════════════════════════════════════════════════════

    /// Get all recorded method calls.
    pub fn calls(&self) -> Vec<MethodCall> {
        self.inner.lock().unwrap().call_log.clone()
    }

    /// Check if a method was called.
    pub fn was_called(&self, method: &str) -> bool {
        self.inner
            .lock()
            .unwrap()
            .call_log
            .iter()
            .any(|c| c.method == method)
    }

    /// Get count of calls to a method.
    pub fn call_count(&self, method: &str) -> usize {
        self.inner
            .lock()
            .unwrap()
            .call_log
            .iter()
            .filter(|c| c.method == method)
            .count()
    }

    /// Clear the call log.
    pub fn clear_calls(&self) {
        self.inner.lock().unwrap().call_log.clear();
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Internal Helpers
    // ════════════════════════════════════════════════════════════════════════════

    fn record_call(&self, method: &str, args: Vec<String>) {
        self.inner.lock().unwrap().call_log.push(MethodCall {
            method: method.to_string(),
            args,
        });
    }

    fn check_error(&self, method: &str) -> Result<(), PaymentError> {
        let mut state = self.inner.lock().unwrap();

        // Check method-specific error first
        if let Some(error) = state.method_errors.get(method) {
            return Err(error.clone());
        }

        // Check global error (consumes it)
        if let Some(error) = state.next_error.take() {
            return Err(error);
        }

        Ok(())
    }
}

impl Clone for MockPaymentProvider {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[async_trait]
impl PaymentProvider for MockPaymentProvider {
    async fn create_customer(
        &self,
        request: CreateCustomerRequest,
    ) -> Result<Customer, PaymentError> {
        self.record_call(
            "create_customer",
            vec![request.user_id.to_string(), request.email.clone()],
        );
        self.check_error("create_customer")?;

        let mut state = self.inner.lock().unwrap();

        let customer = state.next_customer.take().unwrap_or_else(|| Customer {
            id: format!("cus_mock_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap()),
            email: request.email,
            name: request.name,
            created_at: chrono::Utc::now().timestamp(),
        });

        // Store for later retrieval
        state.customers.insert(customer.id.clone(), customer.clone());

        Ok(customer)
    }

    async fn get_customer(&self, customer_id: &str) -> Result<Option<Customer>, PaymentError> {
        self.record_call("get_customer", vec![customer_id.to_string()]);
        self.check_error("get_customer")?;

        let state = self.inner.lock().unwrap();
        Ok(state.customers.get(customer_id).cloned())
    }

    async fn create_subscription(
        &self,
        request: CreateSubscriptionRequest,
    ) -> Result<Subscription, PaymentError> {
        self.record_call(
            "create_subscription",
            vec![request.customer_id.clone(), format!("{:?}", request.tier)],
        );
        self.check_error("create_subscription")?;

        let mut state = self.inner.lock().unwrap();

        let now = chrono::Utc::now().timestamp();
        let period_end = match request.tier {
            MembershipTier::Monthly => now + 30 * 24 * 60 * 60,
            MembershipTier::Annual => now + 365 * 24 * 60 * 60,
            MembershipTier::Free => now + 365 * 24 * 60 * 60,
        };

        let subscription = state.next_subscription.take().unwrap_or_else(|| Subscription {
            id: format!("sub_mock_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap()),
            customer_id: request.customer_id,
            status: SubscriptionStatus::Active,
            current_period_start: now,
            current_period_end: period_end,
            cancel_at_period_end: false,
            canceled_at: None,
        });

        state
            .subscriptions
            .insert(subscription.id.clone(), subscription.clone());

        Ok(subscription)
    }

    async fn get_subscription(
        &self,
        subscription_id: &str,
    ) -> Result<Option<Subscription>, PaymentError> {
        self.record_call("get_subscription", vec![subscription_id.to_string()]);
        self.check_error("get_subscription")?;

        let state = self.inner.lock().unwrap();
        Ok(state.subscriptions.get(subscription_id).cloned())
    }

    async fn cancel_subscription(
        &self,
        subscription_id: &str,
        at_period_end: bool,
    ) -> Result<Subscription, PaymentError> {
        self.record_call(
            "cancel_subscription",
            vec![subscription_id.to_string(), at_period_end.to_string()],
        );
        self.check_error("cancel_subscription")?;

        let mut state = self.inner.lock().unwrap();

        let subscription = state
            .subscriptions
            .get_mut(subscription_id)
            .ok_or_else(|| PaymentError::not_found("Subscription"))?;

        subscription.cancel_at_period_end = at_period_end;
        subscription.canceled_at = Some(chrono::Utc::now().timestamp());

        if !at_period_end {
            subscription.status = SubscriptionStatus::Canceled;
        }

        Ok(subscription.clone())
    }

    async fn update_subscription(
        &self,
        subscription_id: &str,
        new_tier: MembershipTier,
    ) -> Result<Subscription, PaymentError> {
        self.record_call(
            "update_subscription",
            vec![subscription_id.to_string(), format!("{:?}", new_tier)],
        );
        self.check_error("update_subscription")?;

        let state = self.inner.lock().unwrap();

        let subscription = state
            .subscriptions
            .get(subscription_id)
            .ok_or_else(|| PaymentError::not_found("Subscription"))?;

        Ok(subscription.clone())
    }

    async fn create_checkout_session(
        &self,
        request: CreateCheckoutRequest,
    ) -> Result<CheckoutSession, PaymentError> {
        self.record_call(
            "create_checkout_session",
            vec![
                request.user_id.to_string(),
                format!("{:?}", request.tier),
                request.email,
            ],
        );
        self.check_error("create_checkout_session")?;

        let mut state = self.inner.lock().unwrap();

        let session = state.next_checkout.take().unwrap_or_else(|| {
            let id = format!("cs_mock_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
            CheckoutSession {
                id: id.clone(),
                url: format!("https://checkout.stripe.com/c/pay/{}", id),
                expires_at: chrono::Utc::now().timestamp() + 24 * 60 * 60,
            }
        });

        Ok(session)
    }

    async fn create_portal_session(
        &self,
        customer_id: &str,
        return_url: &str,
    ) -> Result<PortalSession, PaymentError> {
        self.record_call(
            "create_portal_session",
            vec![customer_id.to_string(), return_url.to_string()],
        );
        self.check_error("create_portal_session")?;

        let mut state = self.inner.lock().unwrap();

        let session = state.next_portal.take().unwrap_or_else(|| {
            let id = format!("bps_mock_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
            PortalSession {
                id: id.clone(),
                url: format!("https://billing.stripe.com/p/session/{}", id),
            }
        });

        Ok(session)
    }

    async fn verify_webhook(
        &self,
        payload: &[u8],
        signature: &str,
    ) -> Result<WebhookEvent, PaymentError> {
        self.record_call(
            "verify_webhook",
            vec![
                String::from_utf8_lossy(payload).chars().take(50).collect(),
                signature.chars().take(20).collect(),
            ],
        );
        self.check_error("verify_webhook")?;

        let state = self.inner.lock().unwrap();

        // Check verification mode
        match &state.webhook_verify_mode {
            WebhookVerifyMode::AcceptAll => {}
            WebhookVerifyMode::RequireSignature(required) => {
                if signature != required {
                    return Err(PaymentError::invalid_webhook("Invalid signature"));
                }
            }
            WebhookVerifyMode::AlwaysFail => {
                return Err(PaymentError::invalid_webhook("Verification disabled"));
            }
        }

        // Return configured event or parse from payload
        if let Some(event) = &state.next_webhook_event {
            return Ok(event.clone());
        }

        // Try to parse the payload and create a default event
        let parsed: serde_json::Value =
            serde_json::from_slice(payload).map_err(|e| PaymentError::invalid_webhook(e.to_string()))?;

        let id = parsed["id"]
            .as_str()
            .unwrap_or("evt_mock")
            .to_string();
        let event_type = parsed["type"]
            .as_str()
            .unwrap_or("unknown");
        let created = parsed["created"].as_i64().unwrap_or_else(|| {
            chrono::Utc::now().timestamp()
        });

        let webhook_event_type = match event_type {
            "checkout.session.completed" => WebhookEventType::CheckoutSessionCompleted,
            "customer.subscription.updated" => WebhookEventType::SubscriptionUpdated,
            "customer.subscription.deleted" => WebhookEventType::SubscriptionDeleted,
            "invoice.paid" => WebhookEventType::InvoicePaid,
            "invoice.payment_failed" => WebhookEventType::InvoicePaymentFailed,
            other => WebhookEventType::Unknown(other.to_string()),
        };

        Ok(WebhookEvent {
            id,
            event_type: webhook_event_type,
            data: WebhookEventData::Raw {
                json: String::from_utf8_lossy(payload).to_string(),
            },
            created_at: created,
        })
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Test Helpers
// ════════════════════════════════════════════════════════════════════════════════

impl MockPaymentProvider {
    /// Create a mock with a pre-configured active subscription.
    pub fn with_active_subscription(customer_id: &str, subscription_id: &str) -> Self {
        let mock = Self::new();

        mock.add_customer(Customer {
            id: customer_id.to_string(),
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
            created_at: chrono::Utc::now().timestamp(),
        });

        mock.add_subscription(Subscription {
            id: subscription_id.to_string(),
            customer_id: customer_id.to_string(),
            status: SubscriptionStatus::Active,
            current_period_start: chrono::Utc::now().timestamp(),
            current_period_end: chrono::Utc::now().timestamp() + 30 * 24 * 60 * 60,
            cancel_at_period_end: false,
            canceled_at: None,
        });

        mock
    }

    /// Create a checkout completed webhook event.
    pub fn checkout_completed_event(
        customer_id: &str,
        subscription_id: &str,
        user_id: &str,
    ) -> WebhookEvent {
        WebhookEvent {
            id: format!("evt_checkout_{}", uuid::Uuid::new_v4()),
            event_type: WebhookEventType::CheckoutSessionCompleted,
            data: WebhookEventData::Checkout {
                session_id: format!("cs_{}", uuid::Uuid::new_v4()),
                customer_id: customer_id.to_string(),
                subscription_id: Some(subscription_id.to_string()),
                user_id: Some(user_id.to_string()),
            },
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Create a payment failed webhook event.
    pub fn payment_failed_event(customer_id: &str, subscription_id: &str) -> WebhookEvent {
        WebhookEvent {
            id: format!("evt_fail_{}", uuid::Uuid::new_v4()),
            event_type: WebhookEventType::InvoicePaymentFailed,
            data: WebhookEventData::Invoice {
                invoice_id: format!("in_{}", uuid::Uuid::new_v4()),
                customer_id: customer_id.to_string(),
                subscription_id: Some(subscription_id.to_string()),
                amount_paid: 0,
                currency: "cad".to_string(),
            },
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Create a subscription deleted webhook event.
    pub fn subscription_deleted_event(customer_id: &str, subscription_id: &str) -> WebhookEvent {
        WebhookEvent {
            id: format!("evt_del_{}", uuid::Uuid::new_v4()),
            event_type: WebhookEventType::SubscriptionDeleted,
            data: WebhookEventData::Subscription {
                subscription_id: subscription_id.to_string(),
                customer_id: customer_id.to_string(),
                status: SubscriptionStatus::Canceled,
                current_period_end: chrono::Utc::now().timestamp(),
            },
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::UserId;
    use crate::ports::PaymentErrorCode;

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Basic Operation Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn create_customer_returns_mock_customer() {
        let mock = MockPaymentProvider::new();

        let result = mock
            .create_customer(CreateCustomerRequest {
                user_id: test_user_id(),
                email: "test@example.com".to_string(),
                name: Some("Test".to_string()),
                idempotency_key: None,
            })
            .await;

        assert!(result.is_ok());
        let customer = result.unwrap();
        assert!(customer.id.starts_with("cus_mock_"));
        assert_eq!(customer.email, "test@example.com");
    }

    #[tokio::test]
    async fn get_customer_after_create() {
        let mock = MockPaymentProvider::new();

        let created = mock
            .create_customer(CreateCustomerRequest {
                user_id: test_user_id(),
                email: "test@example.com".to_string(),
                name: None,
                idempotency_key: None,
            })
            .await
            .unwrap();

        let fetched = mock.get_customer(&created.id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().id, created.id);
    }

    #[tokio::test]
    async fn get_customer_not_found() {
        let mock = MockPaymentProvider::new();
        let result = mock.get_customer("cus_nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn create_subscription_returns_active() {
        let mock = MockPaymentProvider::new();

        let result = mock
            .create_subscription(CreateSubscriptionRequest {
                customer_id: "cus_123".to_string(),
                tier: MembershipTier::Monthly,
                promo_code: None,
                idempotency_key: None,
            })
            .await;

        assert!(result.is_ok());
        let sub = result.unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Active);
        assert!(!sub.cancel_at_period_end);
    }

    #[tokio::test]
    async fn cancel_subscription_at_period_end() {
        let mock = MockPaymentProvider::with_active_subscription("cus_123", "sub_456");

        let result = mock.cancel_subscription("sub_456", true).await;

        assert!(result.is_ok());
        let sub = result.unwrap();
        assert!(sub.cancel_at_period_end);
        assert!(sub.canceled_at.is_some());
        assert_eq!(sub.status, SubscriptionStatus::Active); // Still active until period end
    }

    #[tokio::test]
    async fn cancel_subscription_immediate() {
        let mock = MockPaymentProvider::with_active_subscription("cus_123", "sub_456");

        let result = mock.cancel_subscription("sub_456", false).await;

        assert!(result.is_ok());
        let sub = result.unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Canceled);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Configuration Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn set_customer_returns_configured() {
        let mock = MockPaymentProvider::new();
        mock.set_customer(Customer {
            id: "cus_custom".to_string(),
            email: "custom@example.com".to_string(),
            name: Some("Custom".to_string()),
            created_at: 1704067200,
        });

        let result = mock
            .create_customer(CreateCustomerRequest {
                user_id: test_user_id(),
                email: "ignored@example.com".to_string(),
                name: None,
                idempotency_key: None,
            })
            .await
            .unwrap();

        assert_eq!(result.id, "cus_custom");
        assert_eq!(result.email, "custom@example.com");
    }

    #[tokio::test]
    async fn set_checkout_session_returns_configured() {
        let mock = MockPaymentProvider::new();
        mock.set_checkout_session(CheckoutSession {
            id: "cs_custom".to_string(),
            url: "https://custom.checkout.url".to_string(),
            expires_at: 1704153600,
        });

        let result = mock
            .create_checkout_session(CreateCheckoutRequest {
                user_id: test_user_id(),
                email: "test@example.com".to_string(),
                tier: MembershipTier::Monthly,
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
                promo_code: None,
            })
            .await
            .unwrap();

        assert_eq!(result.id, "cs_custom");
        assert_eq!(result.url, "https://custom.checkout.url");
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Error Injection Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn set_error_returns_error() {
        let mock = MockPaymentProvider::new();
        mock.set_error(PaymentError::card_declined("Test decline"));

        let result = mock
            .create_customer(CreateCustomerRequest {
                user_id: test_user_id(),
                email: "test@example.com".to_string(),
                name: None,
                idempotency_key: None,
            })
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, PaymentErrorCode::CardDeclined);
    }

    #[tokio::test]
    async fn set_method_error_only_affects_method() {
        let mock = MockPaymentProvider::new();
        mock.set_method_error(
            "create_subscription",
            PaymentError::card_declined("Sub decline"),
        );

        // create_customer should work
        let customer = mock
            .create_customer(CreateCustomerRequest {
                user_id: test_user_id(),
                email: "test@example.com".to_string(),
                name: None,
                idempotency_key: None,
            })
            .await;
        assert!(customer.is_ok());

        // create_subscription should fail
        let sub = mock
            .create_subscription(CreateSubscriptionRequest {
                customer_id: "cus_123".to_string(),
                tier: MembershipTier::Monthly,
                promo_code: None,
                idempotency_key: None,
            })
            .await;
        assert!(sub.is_err());
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Call Tracking Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn tracks_method_calls() {
        let mock = MockPaymentProvider::new();

        mock.create_customer(CreateCustomerRequest {
            user_id: test_user_id(),
            email: "test@example.com".to_string(),
            name: None,
            idempotency_key: None,
        })
        .await
        .unwrap();

        assert!(mock.was_called("create_customer"));
        assert_eq!(mock.call_count("create_customer"), 1);
        assert!(!mock.was_called("create_subscription"));
    }

    #[tokio::test]
    async fn call_log_contains_arguments() {
        let mock = MockPaymentProvider::new();

        mock.create_customer(CreateCustomerRequest {
            user_id: test_user_id(),
            email: "tracked@example.com".to_string(),
            name: None,
            idempotency_key: None,
        })
        .await
        .unwrap();

        let calls = mock.calls();
        assert_eq!(calls.len(), 1);
        assert!(calls[0].args.contains(&"tracked@example.com".to_string()));
    }

    #[tokio::test]
    async fn clear_calls_resets_log() {
        let mock = MockPaymentProvider::new();

        mock.create_customer(CreateCustomerRequest {
            user_id: test_user_id(),
            email: "test@example.com".to_string(),
            name: None,
            idempotency_key: None,
        })
        .await
        .unwrap();

        assert_eq!(mock.call_count("create_customer"), 1);

        mock.clear_calls();

        assert_eq!(mock.call_count("create_customer"), 0);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Webhook Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn verify_webhook_returns_configured_event() {
        let mock = MockPaymentProvider::new();
        let event = MockPaymentProvider::checkout_completed_event("cus_123", "sub_456", "usr_789");
        mock.set_webhook_event(event.clone());

        let result = mock.verify_webhook(b"{}", "signature").await.unwrap();

        assert_eq!(result.id, event.id);
        assert_eq!(result.event_type, WebhookEventType::CheckoutSessionCompleted);
    }

    #[tokio::test]
    async fn verify_webhook_parses_payload_when_no_event_set() {
        let mock = MockPaymentProvider::new();

        let payload = r#"{"id": "evt_test", "type": "invoice.paid", "created": 1704067200}"#;
        let result = mock.verify_webhook(payload.as_bytes(), "sig").await.unwrap();

        assert_eq!(result.id, "evt_test");
        assert_eq!(result.event_type, WebhookEventType::InvoicePaid);
    }

    #[tokio::test]
    async fn rejecting_webhooks_fails_verification() {
        let mock = MockPaymentProvider::rejecting_webhooks();

        let result = mock.verify_webhook(b"{}", "signature").await;

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("disabled"));
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Helper Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn with_active_subscription_creates_correct_state() {
        let mock = MockPaymentProvider::with_active_subscription("cus_test", "sub_test");

        let state = mock.inner.lock().unwrap();
        assert!(state.customers.contains_key("cus_test"));
        assert!(state.subscriptions.contains_key("sub_test"));

        let sub = state.subscriptions.get("sub_test").unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Active);
    }

    #[test]
    fn checkout_completed_event_has_correct_structure() {
        let event = MockPaymentProvider::checkout_completed_event("cus_1", "sub_2", "usr_3");

        assert!(event.id.starts_with("evt_checkout_"));
        assert_eq!(event.event_type, WebhookEventType::CheckoutSessionCompleted);

        match event.data {
            WebhookEventData::Checkout {
                customer_id,
                subscription_id,
                user_id,
                ..
            } => {
                assert_eq!(customer_id, "cus_1");
                assert_eq!(subscription_id, Some("sub_2".to_string()));
                assert_eq!(user_id, Some("usr_3".to_string()));
            }
            _ => panic!("Expected Checkout data"),
        }
    }

    #[test]
    fn payment_failed_event_has_correct_structure() {
        let event = MockPaymentProvider::payment_failed_event("cus_1", "sub_2");

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
}
