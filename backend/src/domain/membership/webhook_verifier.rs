//! Stripe webhook signature verification.
//!
//! Implements secure verification of Stripe webhook signatures using HMAC-SHA256.
//! Includes timestamp validation to prevent replay attacks.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use super::stripe_event::StripeEvent;
use super::webhook_errors::WebhookError;

/// Maximum allowed age for webhook events (5 minutes).
const MAX_EVENT_AGE_SECS: i64 = 300;

/// Maximum allowed clock skew for future events (1 minute).
const MAX_CLOCK_SKEW_SECS: i64 = 60;

/// Parsed components from the Stripe-Signature header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureHeader {
    /// Unix timestamp when the signature was generated.
    pub timestamp: i64,
    /// v1 signature (HMAC-SHA256).
    pub v1_signature: Vec<u8>,
    /// Optional v0 legacy signature.
    pub v0_signature: Option<Vec<u8>>,
}

impl SignatureHeader {
    /// Parses a Stripe-Signature header string.
    ///
    /// Format: `t=<timestamp>,v1=<signature>[,v0=<legacy>]`
    ///
    /// # Errors
    ///
    /// Returns `WebhookError::ParseError` if the header format is invalid.
    pub fn parse(header: &str) -> Result<Self, WebhookError> {
        let mut timestamp: Option<i64> = None;
        let mut v1_signature: Option<Vec<u8>> = None;
        let mut v0_signature: Option<Vec<u8>> = None;

        for part in header.split(',') {
            let (key, value) = part
                .split_once('=')
                .ok_or_else(|| WebhookError::ParseError("invalid header format".to_string()))?;

            match key {
                "t" => {
                    timestamp = Some(value.parse().map_err(|_| {
                        WebhookError::ParseError("invalid timestamp".to_string())
                    })?);
                }
                "v1" => {
                    v1_signature = Some(hex::decode(value).map_err(|_| {
                        WebhookError::ParseError("invalid v1 signature hex".to_string())
                    })?);
                }
                "v0" => {
                    v0_signature = Some(hex::decode(value).map_err(|_| {
                        WebhookError::ParseError("invalid v0 signature hex".to_string())
                    })?);
                }
                _ => {
                    // Ignore unknown fields for forward compatibility
                }
            }
        }

        let timestamp =
            timestamp.ok_or_else(|| WebhookError::ParseError("missing timestamp".to_string()))?;
        let v1_signature = v1_signature
            .ok_or_else(|| WebhookError::ParseError("missing v1 signature".to_string()))?;

        Ok(SignatureHeader {
            timestamp,
            v1_signature,
            v0_signature,
        })
    }
}

/// Verifier for Stripe webhook signatures.
pub struct StripeWebhookVerifier {
    /// The webhook signing secret from Stripe dashboard.
    secret: String,
}

impl StripeWebhookVerifier {
    /// Creates a new verifier with the given webhook secret.
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    /// Verifies the webhook signature and parses the event.
    ///
    /// # Verification Steps
    ///
    /// 1. Parse the signature header
    /// 2. Validate timestamp is within acceptable range
    /// 3. Compute expected signature using HMAC-SHA256
    /// 4. Compare signatures using constant-time comparison
    /// 5. Parse the JSON payload into a StripeEvent
    ///
    /// # Errors
    ///
    /// - `InvalidSignature` - Signature verification failed
    /// - `TimestampOutOfRange` - Event is older than 5 minutes
    /// - `InvalidTimestamp` - Event timestamp is in the future
    /// - `ParseError` - Failed to parse header or JSON payload
    pub fn verify_and_parse(
        &self,
        payload: &[u8],
        signature_header: &str,
    ) -> Result<StripeEvent, WebhookError> {
        // 1. Parse signature header
        let header = SignatureHeader::parse(signature_header)?;

        // 2. Validate timestamp
        self.validate_timestamp(header.timestamp)?;

        // 3. Compute expected signature
        let expected_signature = self.compute_signature(header.timestamp, payload);

        // 4. Compare signatures (constant-time)
        if !constant_time_compare(&expected_signature, &header.v1_signature) {
            return Err(WebhookError::InvalidSignature);
        }

        // 5. Parse event
        let event: StripeEvent = serde_json::from_slice(payload)
            .map_err(|e| WebhookError::ParseError(e.to_string()))?;

        Ok(event)
    }

    /// Validates that the timestamp is within acceptable bounds.
    fn validate_timestamp(&self, timestamp: i64) -> Result<(), WebhookError> {
        let now = chrono::Utc::now().timestamp();
        let age = now - timestamp;

        // Reject events that are too old
        if age > MAX_EVENT_AGE_SECS {
            return Err(WebhookError::TimestampOutOfRange);
        }

        // Reject events from the future (with clock skew tolerance)
        if age < -MAX_CLOCK_SKEW_SECS {
            return Err(WebhookError::InvalidTimestamp);
        }

        Ok(())
    }

    /// Computes the HMAC-SHA256 signature for the given timestamp and payload.
    fn compute_signature(&self, timestamp: i64, payload: &[u8]) -> Vec<u8> {
        let signed_payload = format!(
            "{}.{}",
            timestamp,
            String::from_utf8_lossy(payload)
        );

        let mut mac =
            Hmac::<Sha256>::new_from_slice(self.secret.as_bytes()).expect("HMAC accepts any key");
        mac.update(signed_payload.as_bytes());
        mac.finalize().into_bytes().to_vec()
    }
}

/// Performs constant-time comparison of two byte slices.
///
/// This prevents timing attacks that could leak information about the expected signature.
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

/// Computes HMAC-SHA256 for use in test fixtures.
#[cfg(test)]
pub fn compute_test_signature(secret: &str, timestamp: i64, payload: &str) -> String {
    let signed_payload = format!("{}.{}", timestamp, payload);
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key");
    mac.update(signed_payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "whsec_test_secret_12345";

    // ══════════════════════════════════════════════════════════════
    // SignatureHeader Parsing Tests
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn parse_header_with_v1_only() {
        let signature = "a".repeat(64); // Valid hex
        let header_str = format!("t=1234567890,v1={}", signature);

        let header = SignatureHeader::parse(&header_str).unwrap();

        assert_eq!(header.timestamp, 1234567890);
        assert_eq!(header.v1_signature.len(), 32); // 64 hex chars = 32 bytes
        assert!(header.v0_signature.is_none());
    }

    #[test]
    fn parse_header_with_v0_and_v1() {
        let v1_sig = "a".repeat(64);
        let v0_sig = "b".repeat(64);
        let header_str = format!("t=1234567890,v1={},v0={}", v1_sig, v0_sig);

        let header = SignatureHeader::parse(&header_str).unwrap();

        assert_eq!(header.timestamp, 1234567890);
        assert!(header.v0_signature.is_some());
        assert_eq!(header.v0_signature.unwrap().len(), 32);
    }

    #[test]
    fn parse_header_ignores_unknown_fields() {
        let signature = "a".repeat(64);
        let header_str = format!("t=1234567890,v1={},v2=future,scheme=hmac", signature);

        let header = SignatureHeader::parse(&header_str).unwrap();

        assert_eq!(header.timestamp, 1234567890);
        assert_eq!(header.v1_signature.len(), 32);
    }

    #[test]
    fn parse_header_missing_timestamp_fails() {
        let signature = "a".repeat(64);
        let header_str = format!("v1={}", signature);

        let result = SignatureHeader::parse(&header_str);

        assert!(matches!(result, Err(WebhookError::ParseError(_))));
    }

    #[test]
    fn parse_header_missing_v1_fails() {
        let header_str = "t=1234567890";

        let result = SignatureHeader::parse(header_str);

        assert!(matches!(result, Err(WebhookError::ParseError(_))));
    }

    #[test]
    fn parse_header_invalid_timestamp_fails() {
        let signature = "a".repeat(64);
        let header_str = format!("t=not_a_number,v1={}", signature);

        let result = SignatureHeader::parse(&header_str);

        assert!(matches!(result, Err(WebhookError::ParseError(_))));
    }

    #[test]
    fn parse_header_invalid_hex_fails() {
        let header_str = "t=1234567890,v1=not_valid_hex";

        let result = SignatureHeader::parse(header_str);

        assert!(matches!(result, Err(WebhookError::ParseError(_))));
    }

    #[test]
    fn parse_header_no_equals_fails() {
        let header_str = "t1234567890";

        let result = SignatureHeader::parse(header_str);

        assert!(matches!(result, Err(WebhookError::ParseError(_))));
    }

    // ══════════════════════════════════════════════════════════════
    // Signature Verification Tests
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn verify_valid_signature() {
        let verifier = StripeWebhookVerifier::new(TEST_SECRET);
        let payload = r#"{"id":"evt_test123","type":"checkout.session.completed","created":1704067200,"data":{"object":{}},"livemode":false,"api_version":"2023-10-16"}"#;
        let timestamp = chrono::Utc::now().timestamp();
        let signature = compute_test_signature(TEST_SECRET, timestamp, payload);
        let header = format!("t={},v1={}", timestamp, signature);

        let result = verifier.verify_and_parse(payload.as_bytes(), &header);

        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.id, "evt_test123");
    }

    #[test]
    fn verify_invalid_signature_fails() {
        let verifier = StripeWebhookVerifier::new(TEST_SECRET);
        let payload = r#"{"id":"evt_test"}"#;
        let timestamp = chrono::Utc::now().timestamp();
        let header = format!("t={},v1={}", timestamp, "a".repeat(64));

        let result = verifier.verify_and_parse(payload.as_bytes(), &header);

        assert!(matches!(result, Err(WebhookError::InvalidSignature)));
    }

    #[test]
    fn verify_wrong_secret_fails() {
        let verifier = StripeWebhookVerifier::new("wrong_secret");
        let payload = r#"{"id":"evt_test"}"#;
        let timestamp = chrono::Utc::now().timestamp();
        let signature = compute_test_signature(TEST_SECRET, timestamp, payload);
        let header = format!("t={},v1={}", timestamp, signature);

        let result = verifier.verify_and_parse(payload.as_bytes(), &header);

        assert!(matches!(result, Err(WebhookError::InvalidSignature)));
    }

    #[test]
    fn verify_tampered_payload_fails() {
        let verifier = StripeWebhookVerifier::new(TEST_SECRET);
        let original_payload = r#"{"id":"evt_test"}"#;
        let tampered_payload = r#"{"id":"evt_hacked"}"#;
        let timestamp = chrono::Utc::now().timestamp();
        let signature = compute_test_signature(TEST_SECRET, timestamp, original_payload);
        let header = format!("t={},v1={}", timestamp, signature);

        let result = verifier.verify_and_parse(tampered_payload.as_bytes(), &header);

        assert!(matches!(result, Err(WebhookError::InvalidSignature)));
    }

    // ══════════════════════════════════════════════════════════════
    // Timestamp Validation Tests
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn verify_timestamp_within_range_succeeds() {
        let verifier = StripeWebhookVerifier::new(TEST_SECRET);
        // 2 minutes ago - within 5 minute window
        let timestamp = chrono::Utc::now().timestamp() - 120;

        let result = verifier.validate_timestamp(timestamp);

        assert!(result.is_ok());
    }

    #[test]
    fn verify_timestamp_too_old_fails() {
        let verifier = StripeWebhookVerifier::new(TEST_SECRET);
        // 10 minutes ago - outside 5 minute window
        let timestamp = chrono::Utc::now().timestamp() - 600;

        let result = verifier.validate_timestamp(timestamp);

        assert!(matches!(result, Err(WebhookError::TimestampOutOfRange)));
    }

    #[test]
    fn verify_timestamp_at_boundary_succeeds() {
        let verifier = StripeWebhookVerifier::new(TEST_SECRET);
        // Exactly 5 minutes ago
        let timestamp = chrono::Utc::now().timestamp() - 300;

        let result = verifier.validate_timestamp(timestamp);

        assert!(result.is_ok());
    }

    #[test]
    fn verify_timestamp_just_past_boundary_fails() {
        let verifier = StripeWebhookVerifier::new(TEST_SECRET);
        // 5 minutes and 1 second ago
        let timestamp = chrono::Utc::now().timestamp() - 301;

        let result = verifier.validate_timestamp(timestamp);

        assert!(matches!(result, Err(WebhookError::TimestampOutOfRange)));
    }

    #[test]
    fn verify_timestamp_from_future_with_skew_succeeds() {
        let verifier = StripeWebhookVerifier::new(TEST_SECRET);
        // 30 seconds in the future - within 60s clock skew tolerance
        let timestamp = chrono::Utc::now().timestamp() + 30;

        let result = verifier.validate_timestamp(timestamp);

        assert!(result.is_ok());
    }

    #[test]
    fn verify_timestamp_from_future_beyond_skew_fails() {
        let verifier = StripeWebhookVerifier::new(TEST_SECRET);
        // 2 minutes in the future - beyond clock skew tolerance
        let timestamp = chrono::Utc::now().timestamp() + 120;

        let result = verifier.validate_timestamp(timestamp);

        assert!(matches!(result, Err(WebhookError::InvalidTimestamp)));
    }

    // ══════════════════════════════════════════════════════════════
    // JSON Parsing Tests
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn verify_invalid_json_fails() {
        let verifier = StripeWebhookVerifier::new(TEST_SECRET);
        let payload = "not valid json";
        let timestamp = chrono::Utc::now().timestamp();
        let signature = compute_test_signature(TEST_SECRET, timestamp, payload);
        let header = format!("t={},v1={}", timestamp, signature);

        let result = verifier.verify_and_parse(payload.as_bytes(), &header);

        assert!(matches!(result, Err(WebhookError::ParseError(_))));
    }

    // ══════════════════════════════════════════════════════════════
    // Constant Time Comparison Tests
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn constant_time_compare_equal_values() {
        let a = vec![1, 2, 3, 4, 5];
        let b = vec![1, 2, 3, 4, 5];
        assert!(constant_time_compare(&a, &b));
    }

    #[test]
    fn constant_time_compare_different_values() {
        let a = vec![1, 2, 3, 4, 5];
        let b = vec![1, 2, 3, 4, 6];
        assert!(!constant_time_compare(&a, &b));
    }

    #[test]
    fn constant_time_compare_different_lengths() {
        let a = vec![1, 2, 3];
        let b = vec![1, 2, 3, 4];
        assert!(!constant_time_compare(&a, &b));
    }

    #[test]
    fn constant_time_compare_empty_slices() {
        let a: Vec<u8> = vec![];
        let b: Vec<u8> = vec![];
        assert!(constant_time_compare(&a, &b));
    }

    // ══════════════════════════════════════════════════════════════
    // Integration Test
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn full_verification_flow() {
        let secret = "whsec_full_test_secret";
        let verifier = StripeWebhookVerifier::new(secret);

        let payload = serde_json::json!({
            "id": "evt_full_test",
            "type": "invoice.payment_succeeded",
            "created": 1704067200,
            "data": {
                "object": {
                    "id": "in_123",
                    "amount_paid": 1000
                }
            },
            "livemode": true,
            "api_version": "2023-10-16"
        });
        let payload_str = serde_json::to_string(&payload).unwrap();

        let timestamp = chrono::Utc::now().timestamp();
        let signature = compute_test_signature(secret, timestamp, &payload_str);
        let header = format!("t={},v1={}", timestamp, signature);

        let result = verifier.verify_and_parse(payload_str.as_bytes(), &header);

        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.id, "evt_full_test");
        assert_eq!(event.event_type, "invoice.payment_succeeded");
        assert!(event.is_live());
    }
}
