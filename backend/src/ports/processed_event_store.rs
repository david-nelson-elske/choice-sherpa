//! ProcessedEventStore port - Interface for tracking processed events.
//!
//! This port enables idempotent event handling by tracking which events
//! have been processed by which handlers. This prevents duplicate
//! processing when events are redelivered.
//!
//! ## Why Idempotency Matters
//!
//! Events may be delivered more than once due to:
//! - Network retries
//! - Outbox publisher restarts
//! - Consumer crashes before acknowledgment
//!
//! All event handlers MUST be idempotent. This store enables tracking
//! which events have already been processed.

use async_trait::async_trait;

use crate::domain::foundation::{DomainError, EventId, Timestamp};

/// Port for tracking which events have been processed by which handlers.
///
/// Each handler has its own processing record, allowing different handlers
/// to process the same event independently while maintaining idempotency
/// within each handler.
///
/// # Example
///
/// ```ignore
/// // Check if already processed before handling
/// if store.contains(&event_id, "DashboardHandler").await? {
///     return Ok(()); // Skip duplicate
/// }
///
/// // Process event...
///
/// // Mark as processed after successful handling
/// store.mark_processed(&event_id, "DashboardHandler").await?;
/// ```
#[async_trait]
pub trait ProcessedEventStore: Send + Sync {
    /// Check if an event has been processed by a specific handler.
    ///
    /// Returns `true` if the event has already been processed by this handler.
    async fn contains(
        &self,
        event_id: &EventId,
        handler_name: &str,
    ) -> Result<bool, DomainError>;

    /// Mark an event as processed by a specific handler.
    ///
    /// This should be called AFTER successful event handling to ensure
    /// the event is not reprocessed on retry.
    async fn mark_processed(
        &self,
        event_id: &EventId,
        handler_name: &str,
    ) -> Result<(), DomainError>;

    /// Delete old processed event entries (cleanup/retention policy).
    ///
    /// Removes entries older than the specified timestamp.
    /// Returns the number of entries deleted.
    async fn delete_before(&self, timestamp: Timestamp) -> Result<u64, DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use std::collections::HashSet;
    use tokio::sync::RwLock;

    /// In-memory implementation for testing
    struct InMemoryProcessedEventStore {
        processed: Arc<RwLock<HashSet<(String, String)>>>,
        delete_count: AtomicU64,
    }

    impl InMemoryProcessedEventStore {
        fn new() -> Self {
            Self {
                processed: Arc::new(RwLock::new(HashSet::new())),
                delete_count: AtomicU64::new(0),
            }
        }
    }

    #[async_trait]
    impl ProcessedEventStore for InMemoryProcessedEventStore {
        async fn contains(
            &self,
            event_id: &EventId,
            handler_name: &str,
        ) -> Result<bool, DomainError> {
            let key = (event_id.as_str().to_string(), handler_name.to_string());
            Ok(self.processed.read().await.contains(&key))
        }

        async fn mark_processed(
            &self,
            event_id: &EventId,
            handler_name: &str,
        ) -> Result<(), DomainError> {
            let key = (event_id.as_str().to_string(), handler_name.to_string());
            self.processed.write().await.insert(key);
            Ok(())
        }

        async fn delete_before(&self, _timestamp: Timestamp) -> Result<u64, DomainError> {
            // Simplified: just return a mock count
            let count = self.delete_count.load(Ordering::SeqCst);
            Ok(count)
        }
    }

    #[tokio::test]
    async fn contains_returns_false_for_new_event() {
        let store = InMemoryProcessedEventStore::new();
        let event_id = EventId::new();

        let result = store.contains(&event_id, "TestHandler").await.unwrap();

        assert!(!result);
    }

    #[tokio::test]
    async fn contains_returns_true_after_mark_processed() {
        let store = InMemoryProcessedEventStore::new();
        let event_id = EventId::from_string("evt-123");

        store.mark_processed(&event_id, "TestHandler").await.unwrap();
        let result = store.contains(&event_id, "TestHandler").await.unwrap();

        assert!(result);
    }

    #[tokio::test]
    async fn different_handlers_track_separately() {
        let store = InMemoryProcessedEventStore::new();
        let event_id = EventId::from_string("evt-456");

        // Mark processed by Handler A
        store.mark_processed(&event_id, "HandlerA").await.unwrap();

        // Handler A should show processed
        assert!(store.contains(&event_id, "HandlerA").await.unwrap());

        // Handler B should NOT show processed
        assert!(!store.contains(&event_id, "HandlerB").await.unwrap());
    }

    #[tokio::test]
    async fn different_events_track_separately() {
        let store = InMemoryProcessedEventStore::new();
        let event_id_1 = EventId::from_string("evt-1");
        let event_id_2 = EventId::from_string("evt-2");

        store.mark_processed(&event_id_1, "TestHandler").await.unwrap();

        // Event 1 should be marked
        assert!(store.contains(&event_id_1, "TestHandler").await.unwrap());

        // Event 2 should NOT be marked
        assert!(!store.contains(&event_id_2, "TestHandler").await.unwrap());
    }

    #[tokio::test]
    async fn mark_processed_is_idempotent() {
        let store = InMemoryProcessedEventStore::new();
        let event_id = EventId::from_string("evt-789");

        // Mark multiple times - should not error
        store.mark_processed(&event_id, "TestHandler").await.unwrap();
        store.mark_processed(&event_id, "TestHandler").await.unwrap();
        store.mark_processed(&event_id, "TestHandler").await.unwrap();

        // Should still be marked as processed
        assert!(store.contains(&event_id, "TestHandler").await.unwrap());
    }
}
