//! Stripe webhook event types.
//!
//! Defines the structures for parsing Stripe webhook payloads.
//! Only fields relevant to our processing are captured.

use serde::{Deserialize, Serialize};

/// Stripe webhook event (simplified).
///
/// Contains the essential fields needed for webhook processing.
/// Additional fields from Stripe's full event schema are ignored.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StripeEvent {
    /// Unique identifier for the event (evt_xxx format).
    pub id: String,

    /// Type of event (e.g., "checkout.session.completed").
    #[serde(rename = "type")]
    pub event_type: String,

    /// Time at which the event was created (Unix timestamp).
    pub created: i64,

    /// Object containing event-specific data.
    pub data: StripeEventData,

    /// Whether this is a live mode event (vs test mode).
    pub livemode: bool,

    /// API version used to render this event.
    pub api_version: String,
}

/// Container for event-specific data.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StripeEventData {
    /// The object that triggered the event (polymorphic based on event type).
    pub object: serde_json::Value,

    /// Previous values for updated attributes (only for update events).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_attributes: Option<serde_json::Value>,
}

impl StripeEvent {
    /// Returns true if this is a live mode event.
    pub fn is_live(&self) -> bool {
        self.livemode
    }

    /// Returns true if this is a test mode event.
    pub fn is_test(&self) -> bool {
        !self.livemode
    }

    /// Attempts to deserialize the data object as the specified type.
    pub fn deserialize_object<T: serde::de::DeserializeOwned>(
        &self,
    ) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.data.object.clone())
    }
}

/// Known Stripe event types that we handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StripeEventType {
    /// Checkout session completed successfully.
    CheckoutSessionCompleted,
    /// Invoice payment succeeded.
    InvoicePaymentSucceeded,
    /// Invoice payment failed.
    InvoicePaymentFailed,
    /// Customer subscription was updated.
    CustomerSubscriptionUpdated,
    /// Customer subscription was deleted.
    CustomerSubscriptionDeleted,
    /// Customer subscription was paused.
    CustomerSubscriptionPaused,
    /// Customer subscription was resumed.
    CustomerSubscriptionResumed,
    /// Unknown or unhandled event type.
    Unknown,
}

impl StripeEventType {
    /// Parse event type from string.
    pub fn from_str(s: &str) -> Self {
        match s {
            "checkout.session.completed" => Self::CheckoutSessionCompleted,
            "invoice.payment_succeeded" => Self::InvoicePaymentSucceeded,
            "invoice.payment_failed" => Self::InvoicePaymentFailed,
            "customer.subscription.updated" => Self::CustomerSubscriptionUpdated,
            "customer.subscription.deleted" => Self::CustomerSubscriptionDeleted,
            "customer.subscription.paused" => Self::CustomerSubscriptionPaused,
            "customer.subscription.resumed" => Self::CustomerSubscriptionResumed,
            _ => Self::Unknown,
        }
    }

    /// Convert to the Stripe event type string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CheckoutSessionCompleted => "checkout.session.completed",
            Self::InvoicePaymentSucceeded => "invoice.payment_succeeded",
            Self::InvoicePaymentFailed => "invoice.payment_failed",
            Self::CustomerSubscriptionUpdated => "customer.subscription.updated",
            Self::CustomerSubscriptionDeleted => "customer.subscription.deleted",
            Self::CustomerSubscriptionPaused => "customer.subscription.paused",
            Self::CustomerSubscriptionResumed => "customer.subscription.resumed",
            Self::Unknown => "unknown",
        }
    }
}

impl StripeEvent {
    /// Parse the event type into a known enum variant.
    pub fn parsed_type(&self) -> StripeEventType {
        StripeEventType::from_str(&self.event_type)
    }
}

/// Builder for creating test StripeEvent instances.
#[cfg(test)]
pub struct StripeEventBuilder {
    id: String,
    event_type: String,
    created: i64,
    object: serde_json::Value,
    previous_attributes: Option<serde_json::Value>,
    livemode: bool,
    api_version: String,
}

#[cfg(test)]
impl Default for StripeEventBuilder {
    fn default() -> Self {
        Self {
            id: "evt_test_123".to_string(),
            event_type: "checkout.session.completed".to_string(),
            created: chrono::Utc::now().timestamp(),
            object: serde_json::json!({}),
            previous_attributes: None,
            livemode: false,
            api_version: "2023-10-16".to_string(),
        }
    }
}

#[cfg(test)]
impl StripeEventBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = event_type.into();
        self
    }

    pub fn created(mut self, created: i64) -> Self {
        self.created = created;
        self
    }

    pub fn object(mut self, object: serde_json::Value) -> Self {
        self.object = object;
        self
    }

    pub fn previous_attributes(mut self, attrs: serde_json::Value) -> Self {
        self.previous_attributes = Some(attrs);
        self
    }

    pub fn livemode(mut self, livemode: bool) -> Self {
        self.livemode = livemode;
        self
    }

    pub fn api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = version.into();
        self
    }

    pub fn build(self) -> StripeEvent {
        StripeEvent {
            id: self.id,
            event_type: self.event_type,
            created: self.created,
            data: StripeEventData {
                object: self.object,
                previous_attributes: self.previous_attributes,
            },
            livemode: self.livemode,
            api_version: self.api_version,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ══════════════════════════════════════════════════════════════
    // StripeEvent Deserialization Tests
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn deserialize_minimal_event() {
        let json = r#"{
            "id": "evt_1234567890",
            "type": "checkout.session.completed",
            "created": 1704067200,
            "data": {
                "object": {}
            },
            "livemode": false,
            "api_version": "2023-10-16"
        }"#;

        let event: StripeEvent = serde_json::from_str(json).unwrap();

        assert_eq!(event.id, "evt_1234567890");
        assert_eq!(event.event_type, "checkout.session.completed");
        assert_eq!(event.created, 1704067200);
        assert!(!event.livemode);
        assert_eq!(event.api_version, "2023-10-16");
    }

    #[test]
    fn deserialize_event_with_previous_attributes() {
        let json = r#"{
            "id": "evt_update_123",
            "type": "customer.subscription.updated",
            "created": 1704067200,
            "data": {
                "object": {"status": "active"},
                "previous_attributes": {"status": "past_due"}
            },
            "livemode": true,
            "api_version": "2023-10-16"
        }"#;

        let event: StripeEvent = serde_json::from_str(json).unwrap();

        assert_eq!(event.id, "evt_update_123");
        assert!(event.livemode);
        assert!(event.data.previous_attributes.is_some());
        let prev = event.data.previous_attributes.unwrap();
        assert_eq!(prev["status"], "past_due");
    }

    #[test]
    fn serialize_event_roundtrip() {
        let event = StripeEventBuilder::new()
            .id("evt_roundtrip")
            .event_type("invoice.payment_failed")
            .livemode(true)
            .build();

        let json = serde_json::to_string(&event).unwrap();
        let parsed: StripeEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, "evt_roundtrip");
        assert_eq!(parsed.event_type, "invoice.payment_failed");
        assert!(parsed.livemode);
    }

    // ══════════════════════════════════════════════════════════════
    // StripeEvent Method Tests
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn is_live_returns_true_for_live_mode() {
        let event = StripeEventBuilder::new().livemode(true).build();
        assert!(event.is_live());
        assert!(!event.is_test());
    }

    #[test]
    fn is_test_returns_true_for_test_mode() {
        let event = StripeEventBuilder::new().livemode(false).build();
        assert!(event.is_test());
        assert!(!event.is_live());
    }

    #[test]
    fn deserialize_object_to_custom_type() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct CheckoutSession {
            id: String,
            customer: String,
        }

        let event = StripeEventBuilder::new()
            .object(json!({
                "id": "cs_test_abc123",
                "customer": "cus_xyz789"
            }))
            .build();

        let session: CheckoutSession = event.deserialize_object().unwrap();
        assert_eq!(session.id, "cs_test_abc123");
        assert_eq!(session.customer, "cus_xyz789");
    }

    #[test]
    fn deserialize_object_fails_for_wrong_type() {
        #[derive(Debug, Deserialize)]
        struct Invoice {
            amount_due: i64,
        }

        let event = StripeEventBuilder::new()
            .object(json!({
                "id": "cs_test",
                "status": "complete"
            }))
            .build();

        let result: Result<Invoice, _> = event.deserialize_object();
        assert!(result.is_err());
    }

    // ══════════════════════════════════════════════════════════════
    // StripeEventType Tests
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn event_type_from_str_checkout_completed() {
        assert_eq!(
            StripeEventType::from_str("checkout.session.completed"),
            StripeEventType::CheckoutSessionCompleted
        );
    }

    #[test]
    fn event_type_from_str_payment_succeeded() {
        assert_eq!(
            StripeEventType::from_str("invoice.payment_succeeded"),
            StripeEventType::InvoicePaymentSucceeded
        );
    }

    #[test]
    fn event_type_from_str_payment_failed() {
        assert_eq!(
            StripeEventType::from_str("invoice.payment_failed"),
            StripeEventType::InvoicePaymentFailed
        );
    }

    #[test]
    fn event_type_from_str_subscription_updated() {
        assert_eq!(
            StripeEventType::from_str("customer.subscription.updated"),
            StripeEventType::CustomerSubscriptionUpdated
        );
    }

    #[test]
    fn event_type_from_str_subscription_deleted() {
        assert_eq!(
            StripeEventType::from_str("customer.subscription.deleted"),
            StripeEventType::CustomerSubscriptionDeleted
        );
    }

    #[test]
    fn event_type_from_str_unknown() {
        assert_eq!(
            StripeEventType::from_str("some.unknown.event"),
            StripeEventType::Unknown
        );
    }

    #[test]
    fn event_type_as_str_roundtrip() {
        let types = [
            StripeEventType::CheckoutSessionCompleted,
            StripeEventType::InvoicePaymentSucceeded,
            StripeEventType::InvoicePaymentFailed,
            StripeEventType::CustomerSubscriptionUpdated,
            StripeEventType::CustomerSubscriptionDeleted,
            StripeEventType::CustomerSubscriptionPaused,
            StripeEventType::CustomerSubscriptionResumed,
        ];

        for event_type in types {
            let s = event_type.as_str();
            assert_eq!(StripeEventType::from_str(s), event_type);
        }
    }

    #[test]
    fn parsed_type_returns_correct_variant() {
        let event = StripeEventBuilder::new()
            .event_type("invoice.payment_failed")
            .build();

        assert_eq!(event.parsed_type(), StripeEventType::InvoicePaymentFailed);
    }

    // ══════════════════════════════════════════════════════════════
    // Builder Tests
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn builder_default_values() {
        let event = StripeEventBuilder::new().build();

        assert!(event.id.starts_with("evt_"));
        assert_eq!(event.event_type, "checkout.session.completed");
        assert!(!event.livemode);
        assert_eq!(event.api_version, "2023-10-16");
    }

    #[test]
    fn builder_with_custom_values() {
        let event = StripeEventBuilder::new()
            .id("evt_custom")
            .event_type("invoice.paid")
            .created(1234567890)
            .livemode(true)
            .api_version("2024-01-01")
            .object(json!({"amount": 1000}))
            .previous_attributes(json!({"amount": 500}))
            .build();

        assert_eq!(event.id, "evt_custom");
        assert_eq!(event.event_type, "invoice.paid");
        assert_eq!(event.created, 1234567890);
        assert!(event.livemode);
        assert_eq!(event.api_version, "2024-01-01");
        assert_eq!(event.data.object["amount"], 1000);
        assert!(event.data.previous_attributes.is_some());
    }
}
