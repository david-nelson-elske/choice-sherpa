# Feature: Stripe Webhook Handling

**Module:** membership
**Type:** Integration Specification
**Priority:** P0
**Status:** Specification Complete

> Complete specification for Stripe webhook signature verification, event processing, idempotency handling, and error recovery.

---

## Overview

Stripe webhooks are the primary mechanism for payment event notification. This specification defines:
1. Signature verification requirements
2. Event types and handlers
3. Idempotency guarantees
4. Error handling and retry behavior
5. Security considerations

---

## Webhook Endpoint

### Configuration

```yaml
endpoint: POST /api/webhooks/stripe
content_type: application/json
signature_header: Stripe-Signature
timeout: 30 seconds (Stripe expectation)
```

### Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `STRIPE_WEBHOOK_SECRET` | Signing secret from Stripe dashboard | `whsec_...` |
| `STRIPE_API_KEY` | API key for verification calls | `sk_live_...` |

---

## Signature Verification

### Algorithm

Stripe uses HMAC-SHA256 for webhook signatures. The `Stripe-Signature` header contains:

```
t=<timestamp>,v1=<signature>[,v0=<legacy_signature>]
```

### Verification Steps

```rust
/// Port for webhook signature verification
#[async_trait]
pub trait WebhookVerifier: Send + Sync {
    /// Verifies webhook signature and returns parsed event
    async fn verify_and_parse(
        &self,
        payload: &[u8],
        signature_header: &str,
    ) -> Result<StripeEvent, WebhookError>;
}

/// Verification implementation
impl WebhookVerifier for StripeWebhookVerifier {
    async fn verify_and_parse(
        &self,
        payload: &[u8],
        signature_header: &str,
    ) -> Result<StripeEvent, WebhookError> {
        // 1. Parse signature header
        let parts = parse_signature_header(signature_header)?;

        // 2. Check timestamp tolerance (5 minutes)
        let timestamp = parts.timestamp;
        let now = Utc::now().timestamp();
        if (now - timestamp).abs() > 300 {
            return Err(WebhookError::TimestampOutOfRange);
        }

        // 3. Compute expected signature
        let signed_payload = format!("{}.{}", timestamp, String::from_utf8_lossy(payload));
        let expected = hmac_sha256(&self.webhook_secret, signed_payload.as_bytes());

        // 4. Compare signatures (constant-time)
        if !constant_time_compare(&expected, &parts.v1_signature) {
            return Err(WebhookError::InvalidSignature);
        }

        // 5. Parse event
        let event: StripeEvent = serde_json::from_slice(payload)
            .map_err(|e| WebhookError::ParseError(e.to_string()))?;

        Ok(event)
    }
}
```

### Security Requirements

| Requirement | Implementation |
|-------------|----------------|
| **Constant-time comparison** | Use `subtle::ConstantTimeEq` or equivalent |
| **Timestamp validation** | Reject events older than 5 minutes |
| **Secret rotation** | Support multiple secrets during rotation |
| **TLS only** | Webhook endpoint MUST use HTTPS |

---

## Event Types

### Subscribed Events

| Event | Priority | Handler |
|-------|----------|---------|
| `checkout.session.completed` | P0 | `HandleCheckoutComplete` |
| `invoice.payment_succeeded` | P0 | `HandlePaymentSuccess` |
| `invoice.payment_failed` | P0 | `HandlePaymentFailed` |
| `customer.subscription.updated` | P1 | `HandleSubscriptionUpdated` |
| `customer.subscription.deleted` | P1 | `HandleSubscriptionDeleted` |
| `customer.subscription.paused` | P2 | `HandleSubscriptionPaused` |
| `customer.subscription.resumed` | P2 | `HandleSubscriptionResumed` |

### Event Structure

```rust
/// Stripe webhook event (simplified)
#[derive(Debug, Deserialize)]
pub struct StripeEvent {
    pub id: String,                    // evt_xxx
    #[serde(rename = "type")]
    pub event_type: String,
    pub created: i64,                  // Unix timestamp
    pub data: StripeEventData,
    pub livemode: bool,
    pub api_version: String,
}

#[derive(Debug, Deserialize)]
pub struct StripeEventData {
    pub object: serde_json::Value,     // Polymorphic based on event type
    pub previous_attributes: Option<serde_json::Value>,
}
```

---

## Idempotency

### Event Processing Record

```sql
CREATE TABLE stripe_webhook_events (
    event_id VARCHAR(255) PRIMARY KEY,  -- Stripe event ID (evt_xxx)
    event_type VARCHAR(100) NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    result VARCHAR(20) NOT NULL,         -- 'success', 'ignored', 'failed'
    error_message TEXT,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_webhook_events_type ON stripe_webhook_events(event_type);
CREATE INDEX idx_webhook_events_processed ON stripe_webhook_events(processed_at);
```

### Idempotency Check

```rust
/// Ensures each webhook event is processed exactly once
pub async fn process_webhook_idempotently(
    event: StripeEvent,
    repo: &dyn WebhookEventRepository,
    handler: &dyn WebhookHandler,
) -> Result<WebhookResult, WebhookError> {
    // 1. Check if already processed
    if let Some(existing) = repo.find_by_event_id(&event.id).await? {
        log::info!("Webhook {} already processed at {}", event.id, existing.processed_at);
        return Ok(WebhookResult::AlreadyProcessed);
    }

    // 2. Process the event
    let result = handler.handle(&event).await;

    // 3. Record the result
    let record = WebhookEventRecord {
        event_id: event.id.clone(),
        event_type: event.event_type.clone(),
        processed_at: Utc::now(),
        result: match &result {
            Ok(_) => "success".to_string(),
            Err(WebhookError::Ignored(_)) => "ignored".to_string(),
            Err(_) => "failed".to_string(),
        },
        error_message: result.as_ref().err().map(|e| e.to_string()),
        payload: serde_json::to_value(&event)?,
    };

    repo.save(record).await?;

    result
}
```

### Race Condition Handling

```rust
// Use database constraint to prevent duplicate processing
// If INSERT fails with unique violation, treat as already processed

pub async fn save_with_conflict_handling(
    &self,
    record: WebhookEventRecord,
) -> Result<SaveResult, DbError> {
    match sqlx::query!(
        r#"
        INSERT INTO stripe_webhook_events (event_id, event_type, processed_at, result, error_message, payload)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (event_id) DO NOTHING
        RETURNING event_id
        "#,
        record.event_id,
        record.event_type,
        record.processed_at,
        record.result,
        record.error_message,
        record.payload,
    )
    .fetch_optional(&self.pool)
    .await?
    {
        Some(_) => Ok(SaveResult::Inserted),
        None => Ok(SaveResult::AlreadyExists),
    }
}
```

---

## Event Handlers

### checkout.session.completed

Activates a pending membership after successful checkout.

```rust
pub struct HandleCheckoutComplete {
    membership_repo: Arc<dyn MembershipRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl WebhookEventHandler for HandleCheckoutComplete {
    async fn handle(&self, event: &StripeEvent) -> Result<(), WebhookError> {
        let session: CheckoutSession = serde_json::from_value(event.data.object.clone())?;

        // Extract membership ID from metadata
        let membership_id = session.metadata
            .get("membership_id")
            .ok_or(WebhookError::MissingMetadata("membership_id"))?;

        // Find pending membership
        let mut membership = self.membership_repo
            .find_by_id(&MembershipId::parse(membership_id)?)
            .await?
            .ok_or(WebhookError::MembershipNotFound)?;

        // Validate state
        if membership.status != MembershipStatus::Pending {
            return Err(WebhookError::Ignored(format!(
                "Membership {} not in Pending state (was {})",
                membership_id, membership.status
            )));
        }

        // Activate membership
        membership.activate(
            StripeSubscriptionId::new(session.subscription),
            session.customer,
            PeriodStart::new(Utc::now()),
            PeriodEnd::new(Utc::now() + Duration::days(30)), // Stripe provides actual dates
        )?;

        // Persist
        self.membership_repo.save(&membership).await?;

        // Publish domain event
        self.event_publisher.publish(MembershipActivated {
            membership_id: membership.id.clone(),
            user_id: membership.user_id.clone(),
            tier: membership.tier.clone(),
            activated_at: Utc::now(),
        }).await?;

        Ok(())
    }
}
```

### invoice.payment_failed

Transitions membership to PAST_DUE state.

```rust
pub struct HandlePaymentFailed {
    membership_repo: Arc<dyn MembershipRepository>,
    event_publisher: Arc<dyn EventPublisher>,
    notification_service: Arc<dyn NotificationService>,
}

impl WebhookEventHandler for HandlePaymentFailed {
    async fn handle(&self, event: &StripeEvent) -> Result<(), WebhookError> {
        let invoice: Invoice = serde_json::from_value(event.data.object.clone())?;

        // Find membership by Stripe subscription ID
        let subscription_id = invoice.subscription
            .ok_or(WebhookError::MissingField("subscription"))?;

        let mut membership = self.membership_repo
            .find_by_stripe_subscription_id(&StripeSubscriptionId::new(subscription_id))
            .await?
            .ok_or(WebhookError::MembershipNotFound)?;

        // Only transition if currently Active
        if !matches!(membership.status, MembershipStatus::Active) {
            return Err(WebhookError::Ignored(format!(
                "Membership not Active (was {})", membership.status
            )));
        }

        // Transition to PastDue
        membership.mark_past_due(
            invoice.attempt_count,
            invoice.next_payment_attempt.map(|t| Utc.timestamp_opt(t, 0).unwrap()),
        )?;

        self.membership_repo.save(&membership).await?;

        // Publish domain event
        self.event_publisher.publish(PaymentFailed {
            membership_id: membership.id.clone(),
            user_id: membership.user_id.clone(),
            attempt_count: invoice.attempt_count,
            next_retry_at: membership.next_retry_at,
        }).await?;

        // Send notification email
        self.notification_service.send_payment_failed_email(
            membership.user_id.clone(),
            invoice.attempt_count,
        ).await?;

        Ok(())
    }
}
```

### invoice.payment_succeeded

Handles renewals and payment recovery.

```rust
pub struct HandlePaymentSuccess {
    membership_repo: Arc<dyn MembershipRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl WebhookEventHandler for HandlePaymentSuccess {
    async fn handle(&self, event: &StripeEvent) -> Result<(), WebhookError> {
        let invoice: Invoice = serde_json::from_value(event.data.object.clone())?;

        // Find membership
        let subscription_id = invoice.subscription
            .ok_or(WebhookError::MissingField("subscription"))?;

        let mut membership = self.membership_repo
            .find_by_stripe_subscription_id(&StripeSubscriptionId::new(subscription_id))
            .await?
            .ok_or(WebhookError::MembershipNotFound)?;

        match membership.status {
            MembershipStatus::Active => {
                // Renewal - extend period
                membership.renew(
                    PeriodEnd::new(Utc.timestamp_opt(invoice.lines.data[0].period.end, 0).unwrap()),
                )?;

                self.event_publisher.publish(MembershipRenewed {
                    membership_id: membership.id.clone(),
                    new_period_end: membership.period_end.clone(),
                }).await?;
            }
            MembershipStatus::PastDue => {
                // Recovery - return to Active
                membership.recover_from_past_due(
                    PeriodEnd::new(Utc.timestamp_opt(invoice.lines.data[0].period.end, 0).unwrap()),
                )?;

                self.event_publisher.publish(PaymentRecovered {
                    membership_id: membership.id.clone(),
                    user_id: membership.user_id.clone(),
                }).await?;
            }
            _ => {
                return Err(WebhookError::Ignored(format!(
                    "Unexpected state for payment success: {}", membership.status
                )));
            }
        }

        self.membership_repo.save(&membership).await?;

        Ok(())
    }
}
```

---

## Error Handling

### Webhook Response Codes

| HTTP Status | Meaning | Stripe Behavior |
|-------------|---------|-----------------|
| 200 | Success | Event acknowledged |
| 400 | Bad request | Won't retry |
| 401/403 | Auth failure | Won't retry |
| 404 | Not found | Won't retry |
| 500 | Server error | Will retry |
| 502/503/504 | Unavailable | Will retry |

### Retry Behavior

Stripe retries failed webhooks with exponential backoff:
- 1st retry: ~1 minute
- 2nd retry: ~5 minutes
- 3rd retry: ~30 minutes
- Continues up to 3 days

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Timestamp out of range")]
    TimestampOutOfRange,

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Missing metadata: {0}")]
    MissingMetadata(&'static str),

    #[error("Missing field: {0}")]
    MissingField(&'static str),

    #[error("Membership not found")]
    MembershipNotFound,

    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),

    #[error("Event ignored: {0}")]
    Ignored(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl WebhookError {
    /// Should Stripe retry this error?
    pub fn is_retryable(&self) -> bool {
        matches!(self,
            WebhookError::Database(_) |
            WebhookError::MembershipNotFound  // Might be eventual consistency
        )
    }

    /// HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            WebhookError::InvalidSignature |
            WebhookError::TimestampOutOfRange => StatusCode::UNAUTHORIZED,

            WebhookError::ParseError(_) |
            WebhookError::MissingMetadata(_) |
            WebhookError::MissingField(_) => StatusCode::BAD_REQUEST,

            WebhookError::Ignored(_) => StatusCode::OK, // Acknowledge but ignore

            WebhookError::MembershipNotFound |
            WebhookError::InvalidTransition(_) |
            WebhookError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
```

---

## HTTP Handler

### Axum Implementation

```rust
pub async fn handle_stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, (StatusCode, String)> {
    // 1. Extract signature header
    let signature = headers
        .get("Stripe-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or((StatusCode::BAD_REQUEST, "Missing Stripe-Signature header".to_string()))?;

    // 2. Verify signature and parse event
    let event = state.webhook_verifier
        .verify_and_parse(&body, signature)
        .await
        .map_err(|e| (e.status_code(), e.to_string()))?;

    // 3. Log event receipt
    tracing::info!(
        event_id = %event.id,
        event_type = %event.event_type,
        "Received Stripe webhook"
    );

    // 4. Process with idempotency
    let result = process_webhook_idempotently(
        event,
        state.webhook_event_repo.as_ref(),
        state.webhook_handler.as_ref(),
    ).await;

    match result {
        Ok(WebhookResult::Processed) => {
            tracing::info!("Webhook processed successfully");
            Ok(StatusCode::OK)
        }
        Ok(WebhookResult::AlreadyProcessed) => {
            tracing::info!("Webhook already processed (idempotent)");
            Ok(StatusCode::OK)
        }
        Err(e) if !e.is_retryable() => {
            tracing::warn!(error = %e, "Webhook error (non-retryable)");
            Ok(StatusCode::OK) // Don't make Stripe retry
        }
        Err(e) => {
            tracing::error!(error = %e, "Webhook error (retryable)");
            Err((e.status_code(), e.to_string()))
        }
    }
}
```

---

## Security Checklist

### Configuration
- [ ] `STRIPE_WEBHOOK_SECRET` loaded from secure vault/env
- [ ] Secret rotated at least annually
- [ ] Different secrets for test vs production

### Endpoint
- [ ] HTTPS only (no HTTP)
- [ ] No authentication bypass for webhook endpoint
- [ ] Rate limiting applied (prevent DoS)
- [ ] Request size limit (16KB typical)

### Verification
- [ ] Signature verified before any processing
- [ ] Timestamp validated (5-minute window)
- [ ] Constant-time signature comparison
- [ ] Event ID validated format before processing

### Processing
- [ ] Idempotency key stored before processing
- [ ] All database operations in transaction
- [ ] Sensitive data not logged
- [ ] Livemode flag checked (reject test events in prod)

### Monitoring
- [ ] Failed verifications alerted
- [ ] Processing errors tracked
- [ ] Event volume monitored for anomalies
- [ ] Duplicate event rate tracked

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_verification_valid() {
        let secret = "whsec_test123";
        let payload = r#"{"id":"evt_test"}"#;
        let timestamp = Utc::now().timestamp();

        let signed_payload = format!("{}.{}", timestamp, payload);
        let signature = compute_hmac_sha256(secret, &signed_payload);
        let header = format!("t={},v1={}", timestamp, hex::encode(signature));

        let verifier = StripeWebhookVerifier::new(secret.to_string());
        let result = verifier.verify_and_parse(payload.as_bytes(), &header);

        assert!(result.is_ok());
    }

    #[test]
    fn test_signature_verification_invalid() {
        let verifier = StripeWebhookVerifier::new("whsec_test123".to_string());
        let result = verifier.verify_and_parse(
            b"payload",
            "t=123456789,v1=invalid_signature"
        );

        assert!(matches!(result, Err(WebhookError::InvalidSignature)));
    }

    #[test]
    fn test_timestamp_out_of_range() {
        let secret = "whsec_test123";
        let payload = r#"{"id":"evt_test"}"#;
        let old_timestamp = Utc::now().timestamp() - 600; // 10 minutes ago

        let signed_payload = format!("{}.{}", old_timestamp, payload);
        let signature = compute_hmac_sha256(secret, &signed_payload);
        let header = format!("t={},v1={}", old_timestamp, hex::encode(signature));

        let verifier = StripeWebhookVerifier::new(secret.to_string());
        let result = verifier.verify_and_parse(payload.as_bytes(), &header);

        assert!(matches!(result, Err(WebhookError::TimestampOutOfRange)));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_checkout_complete_activates_membership() {
    let (repo, publisher) = setup_test_dependencies().await;

    // Create pending membership
    let membership = Membership::create_pending(user_id, MembershipTier::Premium);
    repo.save(&membership).await.unwrap();

    // Simulate webhook
    let event = create_checkout_complete_event(membership.id.to_string());
    let handler = HandleCheckoutComplete::new(repo.clone(), publisher.clone());

    handler.handle(&event).await.unwrap();

    // Verify activation
    let updated = repo.find_by_id(&membership.id).await.unwrap().unwrap();
    assert_eq!(updated.status, MembershipStatus::Active);

    // Verify event published
    let events = publisher.published_events();
    assert!(events.iter().any(|e| matches!(e, DomainEvent::MembershipActivated(_))));
}

#[tokio::test]
async fn test_idempotent_processing() {
    let (repo, handler, event_repo) = setup_test_dependencies().await;

    let event = create_checkout_complete_event("mem_123");

    // First processing
    let result1 = process_webhook_idempotently(event.clone(), &event_repo, &handler).await;
    assert!(matches!(result1, Ok(WebhookResult::Processed)));

    // Second processing (same event)
    let result2 = process_webhook_idempotently(event, &event_repo, &handler).await;
    assert!(matches!(result2, Ok(WebhookResult::AlreadyProcessed)));
}
```

### Stripe CLI Testing

```bash
# Install Stripe CLI
brew install stripe/stripe-cli/stripe

# Login
stripe login

# Forward webhooks to local server
stripe listen --forward-to localhost:3000/api/webhooks/stripe

# Trigger test events
stripe trigger checkout.session.completed
stripe trigger invoice.payment_failed
stripe trigger customer.subscription.deleted
```

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Stripe signature verification (HMAC-SHA256) |
| Authorization Model | Webhook endpoint is public but signature-verified |
| Sensitive Data | Payment amounts, subscription details, customer IDs |
| Rate Limiting | Required: 60 requests/minute per IP |
| Audit Logging | All webhook events (success, failure, invalid signature) |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| `stripe_customer_id` | Confidential | Do not log in full; truncate to last 4 chars |
| `subscription_id` | Confidential | Do not expose in error messages |
| `invoice.amount_paid` | Confidential | Do not log |
| `card.last4` | Confidential | May be stored for display purposes |
| `webhook_secret` | Secret | Load from secure vault/env; never log |
| `event_id` | Internal | Safe to log for debugging |

### Security Controls

- **Signature Verification**: MUST use constant-time comparison (`subtle::ConstantTimeEq`)
- **Timestamp Validation**: Reject events with timestamps older than 5 minutes (replay attack prevention)
- **Replay Attack Prevention**:
  - Validate `t=` timestamp is within 5-minute window
  - Store processed `event_id` values to prevent duplicate processing
- **Rate Limiting**: Apply 60 requests/minute limit to webhook endpoint to prevent DoS
- **Request Size Limit**: Maximum 16KB payload size
- **TLS Required**: Webhook endpoint MUST only accept HTTPS connections
- **Livemode Validation**: In production, reject `livemode=false` events
- **Secret Rotation**: Support multiple webhook secrets during rotation period

### Webhook Security Checklist

- [ ] Signature verified before any processing
- [ ] Timestamp within 5-minute window
- [ ] Constant-time signature comparison
- [ ] Event ID checked for duplicates
- [ ] Rate limiting applied (60/min)
- [ ] HTTPS enforced
- [ ] Sensitive data not logged
- [ ] Livemode flag validated in production

---

## Replay Attack Prevention

### Timestamp Validation

Stripe webhooks include a timestamp that MUST be validated:

```rust
impl WebhookValidator {
    /// Maximum age for webhook events (5 minutes)
    const MAX_EVENT_AGE_SECS: i64 = 300;

    pub fn validate_timestamp(&self, signature: &str) -> Result<(), WebhookError> {
        // Parse Stripe signature header: t=timestamp,v1=signature
        let parts = self.parse_signature_header(signature)?;

        let event_time = parts.timestamp;
        let current_time = chrono::Utc::now().timestamp();
        let age = current_time - event_time;

        // Reject events that are too old
        if age > Self::MAX_EVENT_AGE_SECS {
            tracing::warn!(
                event_timestamp = event_time,
                current_time = current_time,
                age_secs = age,
                "Webhook event too old - possible replay attack"
            );
            return Err(WebhookError::EventTooOld { age_secs: age });
        }

        // Also reject events from the future (clock skew tolerance: 60s)
        if age < -60 {
            tracing::warn!(
                event_timestamp = event_time,
                current_time = current_time,
                "Webhook event from future - clock skew or manipulation"
            );
            return Err(WebhookError::InvalidTimestamp);
        }

        Ok(())
    }
}
```

### Event ID Deduplication

Prevent processing the same event twice:

```rust
impl WebhookHandler {
    pub async fn handle(&self, event: StripeEvent) -> Result<(), WebhookError> {
        // 1. Check if event already processed
        if self.is_event_processed(&event.id).await? {
            tracing::info!(
                event_id = %event.id,
                "Duplicate webhook event - already processed"
            );
            // Return success to acknowledge receipt
            return Ok(());
        }

        // 2. Process the event
        self.process_event(&event).await?;

        // 3. Mark as processed (with TTL for cleanup)
        self.mark_event_processed(&event.id).await?;

        Ok(())
    }

    async fn is_event_processed(&self, event_id: &str) -> Result<bool, WebhookError> {
        // Check Redis or database for event ID
        let key = format!("stripe:processed:{}", event_id);
        self.redis.exists(&key).await
            .map_err(|e| WebhookError::StorageError(e.to_string()))
    }

    async fn mark_event_processed(&self, event_id: &str) -> Result<(), WebhookError> {
        let key = format!("stripe:processed:{}", event_id);
        // Store with 7-day TTL (Stripe retries for up to 72 hours)
        self.redis.set_ex(&key, "1", 7 * 24 * 60 * 60).await
            .map_err(|e| WebhookError::StorageError(e.to_string()))
    }
}
```

### Database-Level Idempotency

As a backup, use database constraints:

```sql
-- Processed events table with unique constraint
CREATE TABLE stripe_processed_events (
    event_id VARCHAR(255) PRIMARY KEY,
    event_type VARCHAR(100) NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Auto-cleanup old records
    CONSTRAINT cleanup_after_30_days
        CHECK (processed_at > NOW() - INTERVAL '30 days')
);

-- Index for cleanup job
CREATE INDEX idx_processed_events_date ON stripe_processed_events(processed_at);
```

### Webhook Security Checklist (Replay Prevention)

- [ ] Signature verified with constant-time comparison
- [ ] Timestamp within 5-minute window
- [ ] Future timestamps rejected (with 60s tolerance)
- [ ] Event ID checked for duplicates before processing
- [ ] Event ID stored after successful processing
- [ ] Processed events have TTL for cleanup
- [ ] All validation failures logged with details
- [ ] Production endpoint uses HTTPS only

---

## Monitoring & Alerting

### Metrics

| Metric | Description | Alert Threshold |
|--------|-------------|-----------------|
| `webhook_received_total` | Total webhooks by type | Baseline deviation |
| `webhook_processed_total` | Successfully processed | - |
| `webhook_failed_total` | Processing failures | >5/hour |
| `webhook_signature_invalid_total` | Invalid signatures | >0 |
| `webhook_duplicate_total` | Duplicate events | >10% of total |
| `webhook_processing_duration_ms` | Handler latency | p99 >5000ms |

### Alerts

```yaml
alerts:
  - name: WebhookSignatureFailures
    condition: webhook_signature_invalid_total > 0
    severity: critical
    description: "Invalid webhook signatures detected - possible attack"

  - name: WebhookProcessingFailures
    condition: rate(webhook_failed_total[5m]) > 0.1
    severity: warning
    description: "Elevated webhook processing failure rate"

  - name: WebhookBacklog
    condition: stripe_webhook_events_pending > 100
    severity: warning
    description: "Webhook processing backlog growing"
```

---

*Version: 1.0.0*
*Created: 2026-01-08*
*Module: membership*
