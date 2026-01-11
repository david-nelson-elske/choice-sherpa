//! AIUsageHandler - Event handler for tracking AI token usage.
//!
//! Subscribes to `ai.tokens_used` events and records usage via the UsageTracker port.
//! This enables cost tracking, limit enforcement, and usage analytics.

use async_trait::async_trait;
use std::sync::Arc;

use crate::adapters::ai::ai_events::AITokensUsed;
use crate::domain::foundation::{DomainError, ErrorCode, EventEnvelope};
#[allow(unused_imports)] // Used in handle_tokens_used via async trait
use crate::ports::{EventHandler, UsageRecord, UsageTracker};

/// Event handler that records AI token usage for cost tracking.
///
/// Subscribes to `ai.tokens_used` events and persists usage records via the
/// UsageTracker port. This enables:
/// - Cost attribution per user/session
/// - Daily and session cost limit enforcement
/// - Usage analytics by provider/model/component
///
/// # Example
///
/// ```ignore
/// let tracker: Arc<dyn UsageTracker> = /* ... */;
/// let handler = AIUsageHandler::new(tracker);
///
/// // Subscribe to AI token events
/// event_bus.subscribe("ai.tokens_used", Arc::new(handler));
/// ```
pub struct AIUsageHandler {
    #[allow(dead_code)] // Used in handle_tokens_used via async trait
    tracker: Arc<dyn UsageTracker>,
}

impl AIUsageHandler {
    /// Creates a new handler with the given usage tracker.
    pub fn new(tracker: Arc<dyn UsageTracker>) -> Self {
        Self { tracker }
    }

    /// Handles a tokens used event by recording usage for cost tracking.
    async fn handle_tokens_used(&self, event: AITokensUsed) -> Result<(), DomainError> {
        // Create usage record from event (now includes user context)
        let record = UsageRecord::new(
            event.user_id,
            event.session_id,
            &event.provider,
            &event.model,
            event.prompt_tokens,
            event.completion_tokens,
            event.estimated_cost_cents,
            event.component_type,
        );

        // Record to tracker for cost attribution and limit enforcement
        self.tracker
            .record_usage(record)
            .await
            .map_err(|e| DomainError::new(ErrorCode::DatabaseError, e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl EventHandler for AIUsageHandler {
    async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError> {
        if event.event_type != "ai.tokens_used" {
            return Ok(());
        }

        let tokens_used: AITokensUsed = event.payload_as().map_err(|e| {
            DomainError::new(
                ErrorCode::InvalidFormat,
                format!("Failed to deserialize AITokensUsed event: {}", e),
            )
        })?;

        self.handle_tokens_used(tokens_used).await
    }

    fn name(&self) -> &'static str {
        "AIUsageHandler"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::ai::InMemoryUsageTracker;
    use crate::domain::foundation::{EventId, EventMetadata, SessionId, Timestamp, UserId};

    fn make_tokens_used_envelope() -> EventEnvelope {
        let user_id = UserId::new("user-test-123").unwrap();
        let session_id = SessionId::new();
        let event = AITokensUsed::new(
            user_id,
            session_id,
            "openai",
            "gpt-4",
            100,  // prompt_tokens
            50,   // completion_tokens
            15,   // estimated_cost_cents
            None, // component_type
            "req-123",
        );
        let payload = serde_json::to_value(&event).unwrap();

        EventEnvelope {
            event_id: EventId::new(),
            event_type: "ai.tokens_used".to_string(),
            schema_version: 1,
            aggregate_id: "req-123".to_string(),
            aggregate_type: "AIRequest".to_string(),
            occurred_at: Timestamp::now(),
            payload,
            metadata: EventMetadata::default(),
        }
    }

    #[tokio::test]
    async fn handler_processes_tokens_used_event_and_records_usage() {
        let tracker = Arc::new(InMemoryUsageTracker::new());
        let handler = AIUsageHandler::new(tracker.clone());

        let envelope = make_tokens_used_envelope();
        let result = handler.handle(envelope).await;

        // Handler should succeed
        assert!(result.is_ok());

        // Verify usage was recorded
        let records = tracker.records();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].provider, "openai");
        assert_eq!(records[0].model, "gpt-4");
        assert_eq!(records[0].prompt_tokens, 100);
        assert_eq!(records[0].completion_tokens, 50);
        assert_eq!(records[0].cost_cents, 15);
    }

    #[tokio::test]
    async fn handler_ignores_other_events() {
        let tracker = Arc::new(InMemoryUsageTracker::new());
        let handler = AIUsageHandler::new(tracker);

        let envelope = EventEnvelope {
            event_id: EventId::new(),
            event_type: "session.created.v1".to_string(),
            schema_version: 1,
            aggregate_id: "session-123".to_string(),
            aggregate_type: "Session".to_string(),
            occurred_at: Timestamp::now(),
            payload: serde_json::json!({}),
            metadata: EventMetadata::default(),
        };

        let result = handler.handle(envelope).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn handler_name_is_correct() {
        let tracker = Arc::new(InMemoryUsageTracker::new());
        let handler = AIUsageHandler::new(tracker);

        assert_eq!(handler.name(), "AIUsageHandler");
    }
}
