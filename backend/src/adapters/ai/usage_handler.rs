//! AIUsageHandler - Event handler for tracking AI token usage.
//!
//! Subscribes to `ai.tokens_used` events and records usage via the UsageTracker port.
//! This enables cost tracking, limit enforcement, and usage analytics.

use async_trait::async_trait;
use std::sync::Arc;

use crate::adapters::ai::ai_events::AITokensUsed;
use crate::domain::foundation::{DomainError, ErrorCode, EventEnvelope};
use crate::ports::{EventHandler, UsageTracker};

/// Event handler that records AI token usage for cost tracking.
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
    // Note: tracker is used in commented code awaiting AITokensUsed event enhancement
    #[allow(dead_code)]
    tracker: Arc<dyn UsageTracker>,
}

impl AIUsageHandler {
    /// Creates a new handler with the given usage tracker.
    pub fn new(tracker: Arc<dyn UsageTracker>) -> Self {
        Self { tracker }
    }

    /// Handles a tokens used event.
    async fn handle_tokens_used(&self, event: AITokensUsed) -> Result<(), DomainError> {
        // Note: The current AITokensUsed event doesn't include user_id/session_id.
        // This handler demonstrates the pattern, but requires the event to be
        // enhanced with user context for full cost attribution.
        //
        // For now, we skip recording until the event is enhanced.
        // In a production system, you'd either:
        // 1. Enhance the event with user context (preferred)
        // 2. Use a correlation store to map request_id -> user context
        // 3. Extract user context from the event envelope metadata

        // Log for debugging (replace with tracing when available)
        #[cfg(debug_assertions)]
        eprintln!(
            "AI tokens used: provider={}, model={}, prompt_tokens={}, completion_tokens={}, cost_cents={}, request_id={}",
            event.provider,
            event.model,
            event.prompt_tokens,
            event.completion_tokens,
            event.estimated_cost_cents,
            event.request_id
        );

        // Suppress unused variable warning in release builds
        let _ = &event;

        // TODO: Uncomment when AITokensUsed includes user_id and session_id
        // let record = UsageRecord::new(
        //     event.user_id,
        //     event.session_id,
        //     &event.provider,
        //     &event.model,
        //     event.prompt_tokens,
        //     event.completion_tokens,
        //     event.estimated_cost_cents,
        //     event.component_type,
        // );
        //
        // self.tracker
        //     .record_usage(record)
        //     .await
        //     .map_err(|e| DomainError::new(ErrorCode::DatabaseError, e.to_string()))?;

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

/// In-memory usage tracker for testing.
#[cfg(test)]
pub mod test_support {
    use super::*;
    use crate::domain::foundation::{SessionId, Timestamp, UserId};
    use crate::ports::{
        ProviderUsage, UsageLimitStatus, UsageRecord, UsageSummary, UsageTrackerError,
    };
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Simple in-memory tracker for tests.
    #[derive(Default)]
    pub struct InMemoryUsageTracker {
        records: Mutex<Vec<UsageRecord>>,
    }

    impl InMemoryUsageTracker {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn records(&self) -> Vec<UsageRecord> {
            self.records.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl UsageTracker for InMemoryUsageTracker {
        async fn record_usage(&self, record: UsageRecord) -> Result<(), UsageTrackerError> {
            self.records.lock().unwrap().push(record);
            Ok(())
        }

        async fn get_daily_cost(&self, user_id: &UserId) -> Result<u32, UsageTrackerError> {
            let records = self.records.lock().unwrap();
            let total = records
                .iter()
                .filter(|r| &r.user_id == user_id)
                .map(|r| r.cost_cents)
                .sum();
            Ok(total)
        }

        async fn get_session_cost(&self, session_id: SessionId) -> Result<u32, UsageTrackerError> {
            let records = self.records.lock().unwrap();
            let total = records
                .iter()
                .filter(|r| r.session_id == session_id)
                .map(|r| r.cost_cents)
                .sum();
            Ok(total)
        }

        async fn get_usage_summary(
            &self,
            user_id: &UserId,
            _from: Timestamp,
            _to: Timestamp,
        ) -> Result<UsageSummary, UsageTrackerError> {
            let records = self.records.lock().unwrap();
            let user_records: Vec<_> = records
                .iter()
                .filter(|r| &r.user_id == user_id)
                .collect();

            let mut by_provider: HashMap<String, (u32, u32, u32)> = HashMap::new();
            for record in &user_records {
                let entry = by_provider
                    .entry(record.provider.clone())
                    .or_insert((0, 0, 0));
                entry.0 += record.cost_cents;
                entry.1 += record.total_tokens();
                entry.2 += 1;
            }

            Ok(UsageSummary {
                total_cost_cents: user_records.iter().map(|r| r.cost_cents).sum(),
                total_tokens: user_records.iter().map(|r: &&UsageRecord| r.total_tokens()).sum(),
                request_count: user_records.len() as u32,
                by_provider: by_provider
                    .into_iter()
                    .map(|(provider, (cost, tokens, requests))| ProviderUsage {
                        provider,
                        cost_cents: cost,
                        tokens,
                        requests,
                    })
                    .collect(),
            })
        }

        async fn check_daily_limit(
            &self,
            user_id: &UserId,
            limit_cents: u32,
        ) -> Result<UsageLimitStatus, UsageTrackerError> {
            let current = self.get_daily_cost(user_id).await?;
            Ok(UsageLimitStatus::from_usage(current, limit_cents))
        }

        async fn check_session_limit(
            &self,
            session_id: SessionId,
            limit_cents: u32,
        ) -> Result<UsageLimitStatus, UsageTrackerError> {
            let current = self.get_session_cost(session_id).await?;
            Ok(UsageLimitStatus::from_usage(current, limit_cents))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{EventId, EventMetadata, SessionId, Timestamp, UserId};
    use crate::ports::UsageRecord;
    use test_support::InMemoryUsageTracker;

    fn make_tokens_used_envelope() -> EventEnvelope {
        let event = AITokensUsed::new("openai", "gpt-4", 100, 50, 15, "req-123");
        let payload = serde_json::to_value(&event).unwrap();

        EventEnvelope {
            event_id: EventId::new(),
            event_type: "ai.tokens_used".to_string(),
            aggregate_id: "req-123".to_string(),
            aggregate_type: "AIRequest".to_string(),
            payload,
            metadata: EventMetadata::default(),
            occurred_at: Timestamp::now(),
        }
    }

    #[tokio::test]
    async fn handler_processes_tokens_used_event() {
        let tracker = Arc::new(InMemoryUsageTracker::new());
        let handler = AIUsageHandler::new(tracker.clone());

        let envelope = make_tokens_used_envelope();
        let result = handler.handle(envelope).await;

        // Should succeed (currently just logs since event lacks user context)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn handler_ignores_other_events() {
        let tracker = Arc::new(InMemoryUsageTracker::new());
        let handler = AIUsageHandler::new(tracker);

        let envelope = EventEnvelope {
            event_id: EventId::new(),
            event_type: "session.created".to_string(),
            aggregate_id: "session-123".to_string(),
            aggregate_type: "Session".to_string(),
            payload: serde_json::json!({}),
            metadata: EventMetadata::default(),
            occurred_at: Timestamp::now(),
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

    #[tokio::test]
    async fn in_memory_tracker_records_and_queries() {
        let tracker = InMemoryUsageTracker::new();
        let user_id = UserId::new("user-1").unwrap();
        let session_id = SessionId::new();

        let record = UsageRecord::new(
            user_id.clone(),
            session_id,
            "openai",
            "gpt-4",
            100,
            50,
            15,
            None,
        );

        tracker.record_usage(record).await.unwrap();

        let daily_cost = tracker.get_daily_cost(&user_id).await.unwrap();
        assert_eq!(daily_cost, 15);

        let session_cost = tracker.get_session_cost(session_id).await.unwrap();
        assert_eq!(session_cost, 15);
    }

    #[tokio::test]
    async fn in_memory_tracker_checks_limits() {
        let tracker = InMemoryUsageTracker::new();
        let user_id = UserId::new("user-1").unwrap();
        let session_id = SessionId::new();

        // Record 80 cents of usage
        let record = UsageRecord::new(
            user_id.clone(),
            session_id,
            "openai",
            "gpt-4",
            100,
            50,
            80,
            None,
        );
        tracker.record_usage(record).await.unwrap();

        // Check against 100 cent limit - should be at warning (80%)
        let status = tracker.check_daily_limit(&user_id, 100).await.unwrap();
        assert!(status.should_warn());
        assert!(!status.is_blocked());

        // Check against 50 cent limit - should be blocked
        let status = tracker.check_daily_limit(&user_id, 50).await.unwrap();
        assert!(status.is_blocked());
    }
}
