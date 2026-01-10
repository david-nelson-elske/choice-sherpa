//! Stripe-specific types for webhook handling.
//!
//! These types represent Stripe API objects as they arrive in webhook payloads.
//! They are designed to:
//! - Parse actual Stripe JSON accurately
//! - Map to domain types for further processing
//! - Support idempotency via event IDs

use serde::{Deserialize, Serialize};

// ════════════════════════════════════════════════════════════════════════════════
// Signature Parsing
// ════════════════════════════════════════════════════════════════════════════════

/// Error parsing the Stripe-Signature header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureParseError {
    /// Header is empty or missing.
    MissingHeader,
    /// Missing timestamp component (t=...).
    MissingTimestamp,
    /// Missing v1 signature component.
    MissingV1Signature,
    /// Invalid timestamp format.
    InvalidTimestamp,
    /// Invalid signature format (not valid hex).
    InvalidSignatureFormat,
}

impl std::fmt::Display for SignatureParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingHeader => write!(f, "Missing Stripe-Signature header"),
            Self::MissingTimestamp => write!(f, "Missing timestamp (t=) in signature"),
            Self::MissingV1Signature => write!(f, "Missing v1 signature in header"),
            Self::InvalidTimestamp => write!(f, "Invalid timestamp format"),
            Self::InvalidSignatureFormat => write!(f, "Invalid signature format (not valid hex)"),
        }
    }
}

impl std::error::Error for SignatureParseError {}

/// Parsed Stripe-Signature header components.
///
/// The header format is: `t=timestamp,v1=signature[,v0=legacy_signature]`
///
/// # Example
///
/// ```ignore
/// let header = "t=1704067200,v1=abc123def456...";
/// let parsed = SignatureHeader::parse(header)?;
/// assert_eq!(parsed.timestamp, 1704067200);
/// ```
#[derive(Debug, Clone)]
pub struct SignatureHeader {
    /// Unix timestamp when Stripe generated the event.
    pub timestamp: i64,

    /// Primary v1 signature (HMAC-SHA256, hex-encoded).
    pub v1_signature: Vec<u8>,

    /// Legacy v0 signature (deprecated, may be absent).
    pub v0_signature: Option<Vec<u8>>,
}

impl SignatureHeader {
    /// Parse a Stripe-Signature header into components.
    ///
    /// # Format
    ///
    /// ```text
    /// t=<timestamp>,v1=<signature>[,v0=<legacy_signature>]
    /// ```
    pub fn parse(header: &str) -> Result<Self, SignatureParseError> {
        if header.is_empty() {
            return Err(SignatureParseError::MissingHeader);
        }

        let mut timestamp: Option<i64> = None;
        let mut v1_signature: Option<Vec<u8>> = None;
        let mut v0_signature: Option<Vec<u8>> = None;

        for part in header.split(',') {
            let (key, value) = part
                .split_once('=')
                .ok_or(SignatureParseError::MissingTimestamp)?;

            match key.trim() {
                "t" => {
                    timestamp = Some(
                        value
                            .trim()
                            .parse()
                            .map_err(|_| SignatureParseError::InvalidTimestamp)?,
                    );
                }
                "v1" => {
                    v1_signature =
                        Some(hex_decode(value.trim()).ok_or(SignatureParseError::InvalidSignatureFormat)?);
                }
                "v0" => {
                    v0_signature =
                        Some(hex_decode(value.trim()).ok_or(SignatureParseError::InvalidSignatureFormat)?);
                }
                _ => {
                    // Ignore unknown fields for forward compatibility
                }
            }
        }

        Ok(Self {
            timestamp: timestamp.ok_or(SignatureParseError::MissingTimestamp)?,
            v1_signature: v1_signature.ok_or(SignatureParseError::MissingV1Signature)?,
            v0_signature,
        })
    }
}

/// Decode a hex string to bytes.
fn hex_decode(hex: &str) -> Option<Vec<u8>> {
    let hex = hex.trim();
    if !hex.len().is_multiple_of(2) {
        return None;
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16).ok()?;
        bytes.push(byte);
    }
    Some(bytes)
}

/// Encode bytes to hex string.
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// ════════════════════════════════════════════════════════════════════════════════
// Stripe Event Types
// ════════════════════════════════════════════════════════════════════════════════

/// Raw Stripe webhook event as received from the API.
///
/// This represents the full event envelope containing metadata and payload.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StripeWebhookEvent {
    /// Unique event identifier (evt_...).
    pub id: String,

    /// Event type (e.g., "checkout.session.completed").
    #[serde(rename = "type")]
    pub event_type: String,

    /// Unix timestamp when the event was created.
    pub created: i64,

    /// Event payload containing the affected object.
    pub data: StripeEventData,

    /// Whether this is a live or test event.
    pub livemode: bool,

    /// Stripe API version used for this event.
    pub api_version: Option<String>,

    /// Number of retries for this webhook delivery.
    #[serde(default)]
    pub pending_webhooks: i32,

    /// Request details for events created by API calls.
    pub request: Option<StripeEventRequest>,
}

/// Event data container.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StripeEventData {
    /// The object affected by this event.
    pub object: serde_json::Value,

    /// Previous values for updated fields (on update events).
    pub previous_attributes: Option<serde_json::Value>,
}

/// Request context for events triggered by API calls.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StripeEventRequest {
    /// Request ID from the triggering API call.
    pub id: Option<String>,

    /// Idempotency key if provided.
    pub idempotency_key: Option<String>,
}

// ════════════════════════════════════════════════════════════════════════════════
// Stripe Object Types
// ════════════════════════════════════════════════════════════════════════════════

/// Stripe Checkout Session object.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StripeCheckoutSession {
    /// Unique session identifier (cs_...).
    pub id: String,

    /// Object type (always "checkout.session").
    pub object: String,

    /// Customer ID if customer was created/attached.
    pub customer: Option<String>,

    /// Customer email used during checkout.
    pub customer_email: Option<String>,

    /// Subscription ID if checkout created a subscription.
    pub subscription: Option<String>,

    /// Session payment status.
    pub payment_status: String,

    /// Session status (open, complete, expired).
    pub status: String,

    /// Custom metadata attached to the session.
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,

    /// Payment mode (payment, setup, subscription).
    pub mode: String,

    /// Success URL for redirect after checkout.
    pub success_url: Option<String>,

    /// Cancel URL for redirect if checkout is abandoned.
    pub cancel_url: Option<String>,
}

/// Stripe Customer object.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StripeCustomer {
    /// Unique customer identifier (cus_...).
    pub id: String,

    /// Object type (always "customer").
    pub object: String,

    /// Customer email address.
    pub email: Option<String>,

    /// Customer name.
    pub name: Option<String>,

    /// Unix timestamp of creation.
    pub created: i64,

    /// Custom metadata.
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,

    /// Whether the customer has been deleted.
    #[serde(default)]
    pub deleted: bool,
}

/// Stripe Subscription object.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StripeSubscription {
    /// Unique subscription identifier (sub_...).
    pub id: String,

    /// Object type (always "subscription").
    pub object: String,

    /// Customer ID owning this subscription.
    pub customer: String,

    /// Subscription status.
    pub status: String,

    /// Current period start (Unix timestamp).
    pub current_period_start: i64,

    /// Current period end (Unix timestamp).
    pub current_period_end: i64,

    /// Whether subscription cancels at period end.
    #[serde(default)]
    pub cancel_at_period_end: bool,

    /// When cancellation was requested (Unix timestamp).
    pub canceled_at: Option<i64>,

    /// When subscription ended (Unix timestamp).
    pub ended_at: Option<i64>,

    /// Custom metadata.
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,

    /// Subscription items (price/quantity pairs).
    #[serde(default)]
    pub items: StripeSubscriptionItems,
}

/// Subscription items container.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct StripeSubscriptionItems {
    /// Object type (always "list").
    #[serde(default)]
    pub object: String,

    /// List of subscription items.
    #[serde(default)]
    pub data: Vec<StripeSubscriptionItem>,
}

/// Single subscription item.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StripeSubscriptionItem {
    /// Item ID.
    pub id: String,

    /// Price object.
    pub price: StripePrice,

    /// Item quantity.
    #[serde(default = "default_quantity")]
    pub quantity: i64,
}

fn default_quantity() -> i64 {
    1
}

/// Stripe Price object (embedded in subscription items).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StripePrice {
    /// Price ID.
    pub id: String,

    /// Product ID this price is for.
    pub product: String,

    /// Unit amount in cents.
    pub unit_amount: Option<i64>,

    /// Currency (lowercase, e.g., "cad").
    pub currency: String,

    /// Recurring interval details.
    pub recurring: Option<StripePriceRecurring>,
}

/// Price recurring configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StripePriceRecurring {
    /// Billing interval (day, week, month, year).
    pub interval: String,

    /// Number of intervals between billings.
    pub interval_count: i32,
}

/// Stripe Invoice object.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StripeInvoice {
    /// Unique invoice identifier (in_...).
    pub id: String,

    /// Object type (always "invoice").
    pub object: String,

    /// Customer ID.
    pub customer: String,

    /// Associated subscription ID.
    pub subscription: Option<String>,

    /// Invoice status (draft, open, paid, void, uncollectible).
    pub status: String,

    /// Amount paid in cents.
    pub amount_paid: i64,

    /// Amount due in cents.
    pub amount_due: i64,

    /// Currency (lowercase).
    pub currency: String,

    /// Number of payment attempts made.
    #[serde(default)]
    pub attempt_count: i32,

    /// Unix timestamp of next payment attempt.
    pub next_payment_attempt: Option<i64>,

    /// Custom metadata.
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,

    /// Invoice line items.
    #[serde(default)]
    pub lines: StripeInvoiceLines,
}

/// Invoice lines container.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct StripeInvoiceLines {
    /// Object type (always "list").
    #[serde(default)]
    pub object: String,

    /// List of line items.
    #[serde(default)]
    pub data: Vec<StripeInvoiceLineItem>,
}

/// Single invoice line item.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StripeInvoiceLineItem {
    /// Line item ID.
    pub id: String,

    /// Description.
    pub description: Option<String>,

    /// Amount in cents.
    pub amount: i64,

    /// Billing period for this line.
    pub period: StripeInvoicePeriod,
}

/// Invoice line item period.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StripeInvoicePeriod {
    /// Period start (Unix timestamp).
    pub start: i64,

    /// Period end (Unix timestamp).
    pub end: i64,
}

// ════════════════════════════════════════════════════════════════════════════════
// Event Type Mapping
// ════════════════════════════════════════════════════════════════════════════════

impl StripeWebhookEvent {
    /// Extract membership_id from event metadata if present.
    ///
    /// Different event types store this in different locations.
    pub fn get_membership_id(&self) -> Option<String> {
        // Try checkout session metadata first
        if self.event_type == "checkout.session.completed" {
            if let Ok(session) = serde_json::from_value::<StripeCheckoutSession>(self.data.object.clone()) {
                return session.metadata.get("membership_id").cloned();
            }
        }

        // Try subscription metadata
        if self.event_type.starts_with("customer.subscription.") {
            if let Ok(sub) = serde_json::from_value::<StripeSubscription>(self.data.object.clone()) {
                return sub.metadata.get("membership_id").cloned();
            }
        }

        // Try invoice metadata
        if self.event_type.starts_with("invoice.") {
            if let Ok(invoice) = serde_json::from_value::<StripeInvoice>(self.data.object.clone()) {
                return invoice.metadata.get("membership_id").cloned();
            }
        }

        None
    }

    /// Extract the customer ID from the event object.
    pub fn get_customer_id(&self) -> Option<String> {
        // Most objects have a customer field at the top level
        self.data.object.get("customer").and_then(|v| v.as_str()).map(String::from)
            .or_else(|| self.data.object.get("id").and_then(|v| {
                // Customer events have the customer as the object itself
                if self.event_type.starts_with("customer.") && !self.event_type.contains("subscription") {
                    v.as_str().map(String::from)
                } else {
                    None
                }
            }))
    }

    /// Extract the subscription ID from the event object.
    pub fn get_subscription_id(&self) -> Option<String> {
        // Subscription events have the subscription as the object
        if self.event_type.starts_with("customer.subscription.") {
            return self.data.object.get("id").and_then(|v| v.as_str()).map(String::from);
        }

        // Invoice and checkout session events have a subscription field
        self.data.object.get("subscription").and_then(|v| v.as_str()).map(String::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ════════════════════════════════════════════════════════════════════════════
    // SignatureHeader Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn parse_signature_header_valid() {
        let header = "t=1704067200,v1=5d41402abc4b2a76b9719d911017c592";
        let parsed = SignatureHeader::parse(header).unwrap();

        assert_eq!(parsed.timestamp, 1704067200);
        assert_eq!(
            hex_encode(&parsed.v1_signature),
            "5d41402abc4b2a76b9719d911017c592"
        );
        assert!(parsed.v0_signature.is_none());
    }

    #[test]
    fn parse_signature_header_with_v0() {
        let header = "t=1704067200,v1=5d41402abc4b2a76b9719d911017c592,v0=aabbccdd";
        let parsed = SignatureHeader::parse(header).unwrap();

        assert_eq!(parsed.timestamp, 1704067200);
        assert!(parsed.v0_signature.is_some());
        assert_eq!(hex_encode(&parsed.v0_signature.unwrap()), "aabbccdd");
    }

    #[test]
    fn parse_signature_header_missing_timestamp() {
        let header = "v1=5d41402abc4b2a76b9719d911017c592";
        let result = SignatureHeader::parse(header);
        assert!(matches!(result, Err(SignatureParseError::MissingTimestamp)));
    }

    #[test]
    fn parse_signature_header_missing_v1() {
        let header = "t=1704067200,v0=aabbccdd";
        let result = SignatureHeader::parse(header);
        assert!(matches!(result, Err(SignatureParseError::MissingV1Signature)));
    }

    #[test]
    fn parse_signature_header_empty() {
        let result = SignatureHeader::parse("");
        assert!(matches!(result, Err(SignatureParseError::MissingHeader)));
    }

    #[test]
    fn parse_signature_header_invalid_timestamp() {
        let header = "t=not_a_number,v1=5d41402abc4b2a76b9719d911017c592";
        let result = SignatureHeader::parse(header);
        assert!(matches!(result, Err(SignatureParseError::InvalidTimestamp)));
    }

    #[test]
    fn parse_signature_header_invalid_hex() {
        let header = "t=1704067200,v1=not_valid_hex_xyz";
        let result = SignatureHeader::parse(header);
        assert!(matches!(result, Err(SignatureParseError::InvalidSignatureFormat)));
    }

    #[test]
    fn parse_signature_header_odd_length_hex() {
        let header = "t=1704067200,v1=abc";
        let result = SignatureHeader::parse(header);
        assert!(matches!(result, Err(SignatureParseError::InvalidSignatureFormat)));
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Hex Encoding Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn hex_encode_empty() {
        assert_eq!(hex_encode(&[]), "");
    }

    #[test]
    fn hex_encode_bytes() {
        assert_eq!(hex_encode(&[0x00, 0xff, 0x10]), "00ff10");
    }

    #[test]
    fn hex_decode_roundtrip() {
        let original = vec![0xde, 0xad, 0xbe, 0xef];
        let encoded = hex_encode(&original);
        let decoded = hex_decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Event Parsing Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn parse_checkout_session_completed_event() {
        let json = r#"{
            "id": "evt_1234567890",
            "type": "checkout.session.completed",
            "created": 1704067200,
            "data": {
                "object": {
                    "id": "cs_test_abc123",
                    "object": "checkout.session",
                    "customer": "cus_test_xyz",
                    "subscription": "sub_test_123",
                    "payment_status": "paid",
                    "status": "complete",
                    "mode": "subscription",
                    "metadata": {
                        "membership_id": "mem_abc123"
                    }
                }
            },
            "livemode": false,
            "pending_webhooks": 0
        }"#;

        let event: StripeWebhookEvent = serde_json::from_str(json).unwrap();

        assert_eq!(event.id, "evt_1234567890");
        assert_eq!(event.event_type, "checkout.session.completed");
        assert_eq!(event.created, 1704067200);
        assert!(!event.livemode);
        assert_eq!(event.get_membership_id(), Some("mem_abc123".to_string()));
        assert_eq!(event.get_customer_id(), Some("cus_test_xyz".to_string()));
        assert_eq!(event.get_subscription_id(), Some("sub_test_123".to_string()));
    }

    #[test]
    fn parse_subscription_updated_event() {
        let json = r#"{
            "id": "evt_sub_update",
            "type": "customer.subscription.updated",
            "created": 1704067200,
            "data": {
                "object": {
                    "id": "sub_test_123",
                    "object": "subscription",
                    "customer": "cus_test_xyz",
                    "status": "active",
                    "current_period_start": 1704067200,
                    "current_period_end": 1706745600,
                    "metadata": {
                        "membership_id": "mem_xyz789"
                    }
                },
                "previous_attributes": {
                    "status": "past_due"
                }
            },
            "livemode": true,
            "pending_webhooks": 1
        }"#;

        let event: StripeWebhookEvent = serde_json::from_str(json).unwrap();

        assert_eq!(event.id, "evt_sub_update");
        assert_eq!(event.event_type, "customer.subscription.updated");
        assert!(event.livemode);
        assert!(event.data.previous_attributes.is_some());
        assert_eq!(event.get_subscription_id(), Some("sub_test_123".to_string()));
        assert_eq!(event.get_customer_id(), Some("cus_test_xyz".to_string()));
    }

    #[test]
    fn parse_invoice_payment_failed_event() {
        let json = r#"{
            "id": "evt_invoice_fail",
            "type": "invoice.payment_failed",
            "created": 1704067200,
            "data": {
                "object": {
                    "id": "in_test_123",
                    "object": "invoice",
                    "customer": "cus_test_xyz",
                    "subscription": "sub_test_456",
                    "status": "open",
                    "amount_paid": 0,
                    "amount_due": 1999,
                    "currency": "cad",
                    "attempt_count": 1,
                    "next_payment_attempt": 1704153600,
                    "metadata": {}
                }
            },
            "livemode": false,
            "pending_webhooks": 0
        }"#;

        let event: StripeWebhookEvent = serde_json::from_str(json).unwrap();

        assert_eq!(event.event_type, "invoice.payment_failed");

        let invoice: StripeInvoice = serde_json::from_value(event.data.object).unwrap();
        assert_eq!(invoice.attempt_count, 1);
        assert_eq!(invoice.amount_due, 1999);
        assert_eq!(invoice.next_payment_attempt, Some(1704153600));
    }

    #[test]
    fn parse_checkout_session_object() {
        let json = r#"{
            "id": "cs_test_abc",
            "object": "checkout.session",
            "customer": "cus_123",
            "customer_email": "test@example.com",
            "subscription": "sub_456",
            "payment_status": "paid",
            "status": "complete",
            "mode": "subscription",
            "metadata": {
                "membership_id": "mem_789",
                "user_id": "usr_abc"
            },
            "success_url": "https://example.com/success",
            "cancel_url": "https://example.com/cancel"
        }"#;

        let session: StripeCheckoutSession = serde_json::from_str(json).unwrap();

        assert_eq!(session.id, "cs_test_abc");
        assert_eq!(session.customer, Some("cus_123".to_string()));
        assert_eq!(session.customer_email, Some("test@example.com".to_string()));
        assert_eq!(session.subscription, Some("sub_456".to_string()));
        assert_eq!(session.payment_status, "paid");
        assert_eq!(session.mode, "subscription");
        assert_eq!(session.metadata.get("membership_id").unwrap(), "mem_789");
    }

    #[test]
    fn parse_subscription_object() {
        let json = r#"{
            "id": "sub_test_123",
            "object": "subscription",
            "customer": "cus_xyz",
            "status": "active",
            "current_period_start": 1704067200,
            "current_period_end": 1706745600,
            "cancel_at_period_end": false,
            "metadata": {},
            "items": {
                "object": "list",
                "data": [
                    {
                        "id": "si_abc",
                        "price": {
                            "id": "price_monthly",
                            "product": "prod_sherpa",
                            "unit_amount": 1999,
                            "currency": "cad",
                            "recurring": {
                                "interval": "month",
                                "interval_count": 1
                            }
                        },
                        "quantity": 1
                    }
                ]
            }
        }"#;

        let sub: StripeSubscription = serde_json::from_str(json).unwrap();

        assert_eq!(sub.id, "sub_test_123");
        assert_eq!(sub.status, "active");
        assert!(!sub.cancel_at_period_end);
        assert_eq!(sub.items.data.len(), 1);
        assert_eq!(sub.items.data[0].price.unit_amount, Some(1999));
        assert_eq!(sub.items.data[0].price.currency, "cad");
    }

    #[test]
    fn parse_invoice_object() {
        let json = r#"{
            "id": "in_test_123",
            "object": "invoice",
            "customer": "cus_xyz",
            "subscription": "sub_456",
            "status": "paid",
            "amount_paid": 1999,
            "amount_due": 1999,
            "currency": "cad",
            "attempt_count": 1,
            "metadata": {},
            "lines": {
                "object": "list",
                "data": [
                    {
                        "id": "il_abc",
                        "description": "Choice Sherpa Monthly",
                        "amount": 1999,
                        "period": {
                            "start": 1704067200,
                            "end": 1706745600
                        }
                    }
                ]
            }
        }"#;

        let invoice: StripeInvoice = serde_json::from_str(json).unwrap();

        assert_eq!(invoice.id, "in_test_123");
        assert_eq!(invoice.status, "paid");
        assert_eq!(invoice.amount_paid, 1999);
        assert_eq!(invoice.lines.data.len(), 1);
        assert_eq!(invoice.lines.data[0].period.end, 1706745600);
    }

    #[test]
    fn stripe_subscription_items_defaults_to_empty() {
        let json = r#"{
            "id": "sub_minimal",
            "object": "subscription",
            "customer": "cus_123",
            "status": "active",
            "current_period_start": 1704067200,
            "current_period_end": 1706745600
        }"#;

        let sub: StripeSubscription = serde_json::from_str(json).unwrap();
        assert!(sub.items.data.is_empty());
        assert!(!sub.cancel_at_period_end);
    }
}
