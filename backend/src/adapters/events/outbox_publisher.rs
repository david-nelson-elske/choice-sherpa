//! OutboxPublisher - Background service for reliable event delivery.
//!
//! This service implements the second half of the Transactional Outbox Pattern:
//! 1. Command handlers write events to outbox (same transaction as domain changes)
//! 2. **OutboxPublisher polls outbox and publishes to message broker** ← This module
//!
//! ## Why a Background Service?
//!
//! - Decouples event persistence from delivery
//! - Retries failed publications automatically
//! - Survives application crashes (events are persisted)
//! - Can be scaled independently
//!
//! ## Configuration
//!
//! | Setting | Default | Description |
//! |---------|---------|-------------|
//! | `poll_interval` | 100ms | How often to check for unpublished events |
//! | `batch_size` | 100 | Max events to publish per poll cycle |
//!
//! ## Graceful Shutdown
//!
//! The service listens for a shutdown signal and completes the current
//! batch before stopping.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::watch;
use tokio::time;

use crate::domain::foundation::DomainError;
use crate::ports::{EventPublisher, OutboxWriter};

/// Configuration for the OutboxPublisher service.
#[derive(Debug, Clone)]
pub struct OutboxPublisherConfig {
    /// How often to poll for unpublished events.
    pub poll_interval: Duration,

    /// Maximum events to process per poll cycle.
    pub batch_size: u32,
}

impl Default for OutboxPublisherConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_millis(100),
            batch_size: 100,
        }
    }
}

impl OutboxPublisherConfig {
    /// Create config with custom poll interval.
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Create config with custom batch size.
    pub fn with_batch_size(mut self, size: u32) -> Self {
        self.batch_size = size;
        self
    }
}

/// Background service that publishes events from the outbox.
///
/// Polls the outbox for pending events and publishes them to the
/// configured EventPublisher (e.g., Redis, in-memory for tests).
pub struct OutboxPublisher {
    outbox: Arc<dyn OutboxWriter>,
    event_publisher: Arc<dyn EventPublisher>,
    config: OutboxPublisherConfig,
}

impl OutboxPublisher {
    /// Create a new OutboxPublisher with default configuration.
    pub fn new(outbox: Arc<dyn OutboxWriter>, event_publisher: Arc<dyn EventPublisher>) -> Self {
        Self {
            outbox,
            event_publisher,
            config: OutboxPublisherConfig::default(),
        }
    }

    /// Create a new OutboxPublisher with custom configuration.
    pub fn with_config(
        outbox: Arc<dyn OutboxWriter>,
        event_publisher: Arc<dyn EventPublisher>,
        config: OutboxPublisherConfig,
    ) -> Self {
        Self {
            outbox,
            event_publisher,
            config,
        }
    }

    /// Run the publisher loop until shutdown signal is received.
    ///
    /// # Arguments
    ///
    /// * `shutdown` - Watch channel that signals when to stop
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on graceful shutdown, or error if a fatal error occurs.
    pub async fn run(&self, mut shutdown: watch::Receiver<bool>) -> Result<(), DomainError> {
        let mut interval = time::interval(self.config.poll_interval);

        loop {
            tokio::select! {
                // Check for shutdown signal
                _ = shutdown.changed() => {
                    if *shutdown.borrow() {
                        // Shutdown requested - process one final batch then exit
                        self.process_batch().await?;
                        return Ok(());
                    }
                }

                // Poll interval elapsed
                _ = interval.tick() => {
                    self.process_batch().await?;
                }
            }
        }
    }

    /// Process a single batch of pending events.
    ///
    /// This method is also useful for testing without running the full loop.
    pub async fn process_batch(&self) -> Result<usize, DomainError> {
        let entries = self.outbox.get_pending(self.config.batch_size).await?;
        let mut published_count = 0;

        for entry in entries {
            match self.event_publisher.publish(entry.event.clone()).await {
                Ok(()) => {
                    // Mark as successfully published
                    self.outbox.mark_published(entry.id).await?;
                    published_count += 1;
                }
                Err(e) => {
                    // Mark as failed - will be retried on next poll
                    // In production, add tracing here:
                    // tracing::warn!(event_id = %entry.event.event_id, error = %e, "Failed to publish event");
                    let error_msg = format!("{}", e);
                    self.outbox.mark_failed(entry.id, &error_msg).await?;
                }
            }
        }

        Ok(published_count)
    }

    /// Run exactly one poll cycle (for testing).
    pub async fn poll_once(&self) -> Result<usize, DomainError> {
        self.process_batch().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::InMemoryEventBus;
    use crate::domain::foundation::{ErrorCode, EventEnvelope, EventId, EventMetadata, Timestamp};
    use crate::ports::OutboxEntry;
    use serde_json::json;
    use tokio::sync::RwLock;
    use uuid::Uuid;

    /// Test implementation of OutboxWriter
    struct TestOutboxWriter {
        pending: RwLock<Vec<OutboxEntry>>,
        published_ids: RwLock<Vec<Uuid>>,
        failed_ids: RwLock<Vec<(Uuid, String)>>,
    }

    impl TestOutboxWriter {
        fn new() -> Self {
            Self {
                pending: RwLock::new(Vec::new()),
                published_ids: RwLock::new(Vec::new()),
                failed_ids: RwLock::new(Vec::new()),
            }
        }

        async fn add_pending(&self, event: EventEnvelope) {
            let entry = OutboxEntry::new(event, "test-partition");
            self.pending.write().await.push(entry);
        }

        async fn published_count(&self) -> usize {
            self.published_ids.read().await.len()
        }

        async fn failed_count(&self) -> usize {
            self.failed_ids.read().await.len()
        }
    }

    #[async_trait::async_trait]
    impl OutboxWriter for TestOutboxWriter {
        async fn write(
            &self,
            event: &EventEnvelope,
            partition_key: &str,
        ) -> Result<OutboxEntry, DomainError> {
            let entry = OutboxEntry::new(event.clone(), partition_key);
            self.pending.write().await.push(entry.clone());
            Ok(entry)
        }

        async fn write_batch(
            &self,
            events: &[EventEnvelope],
            partition_key: &str,
        ) -> Result<Vec<OutboxEntry>, DomainError> {
            let mut entries = Vec::new();
            for event in events {
                let entry = OutboxEntry::new(event.clone(), partition_key);
                self.pending.write().await.push(entry.clone());
                entries.push(entry);
            }
            Ok(entries)
        }

        async fn get_pending(&self, limit: u32) -> Result<Vec<OutboxEntry>, DomainError> {
            let mut pending = self.pending.write().await;
            let to_take = std::cmp::min(limit as usize, pending.len());
            let entries: Vec<_> = pending.drain(..to_take).collect();
            Ok(entries)
        }

        async fn mark_published(&self, id: Uuid) -> Result<(), DomainError> {
            self.published_ids.write().await.push(id);
            Ok(())
        }

        async fn mark_failed(&self, id: Uuid, error: &str) -> Result<(), DomainError> {
            self.failed_ids.write().await.push((id, error.to_string()));
            Ok(())
        }

        async fn cleanup_old(&self, _older_than_hours: u32) -> Result<u64, DomainError> {
            Ok(0)
        }
    }

    fn test_envelope(id: &str) -> EventEnvelope {
        EventEnvelope {
            event_id: EventId::from_string(id),
            event_type: "test.event".to_string(),
            aggregate_id: "agg-1".to_string(),
            aggregate_type: "Test".to_string(),
            occurred_at: Timestamp::now(),
            payload: json!({}),
            metadata: EventMetadata::default(),
        }
    }

    #[tokio::test]
    async fn poll_once_publishes_pending_events() {
        let outbox = Arc::new(TestOutboxWriter::new());
        let event_bus = Arc::new(InMemoryEventBus::new());

        outbox.add_pending(test_envelope("evt-1")).await;
        outbox.add_pending(test_envelope("evt-2")).await;

        let publisher = OutboxPublisher::new(outbox.clone(), event_bus.clone());
        let count = publisher.poll_once().await.unwrap();

        assert_eq!(count, 2);
        assert_eq!(event_bus.event_count(), 2);
        assert_eq!(outbox.published_count().await, 2);
    }

    #[tokio::test]
    async fn poll_once_respects_batch_size() {
        let outbox = Arc::new(TestOutboxWriter::new());
        let event_bus = Arc::new(InMemoryEventBus::new());

        // Add 5 events but limit batch to 2
        for i in 0..5 {
            outbox.add_pending(test_envelope(&format!("evt-{}", i))).await;
        }

        let config = OutboxPublisherConfig::default().with_batch_size(2);
        let publisher = OutboxPublisher::with_config(outbox.clone(), event_bus.clone(), config);

        // First poll - should get 2
        let count = publisher.poll_once().await.unwrap();
        assert_eq!(count, 2);

        // Second poll - should get 2 more
        let count = publisher.poll_once().await.unwrap();
        assert_eq!(count, 2);

        // Third poll - should get 1 (remaining)
        let count = publisher.poll_once().await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn poll_once_with_no_pending_returns_zero() {
        let outbox = Arc::new(TestOutboxWriter::new());
        let event_bus = Arc::new(InMemoryEventBus::new());
        let publisher = OutboxPublisher::new(outbox.clone(), event_bus.clone());

        let count = publisher.poll_once().await.unwrap();

        assert_eq!(count, 0);
    }

    /// Event publisher that fails
    struct FailingPublisher;

    #[async_trait::async_trait]
    impl EventPublisher for FailingPublisher {
        async fn publish(&self, _: EventEnvelope) -> Result<(), DomainError> {
            Err(DomainError::new(ErrorCode::InternalError, "Publish failed"))
        }

        async fn publish_all(&self, _: Vec<EventEnvelope>) -> Result<(), DomainError> {
            Err(DomainError::new(ErrorCode::InternalError, "Publish failed"))
        }
    }

    #[tokio::test]
    async fn failed_publish_marks_event_as_failed() {
        let outbox = Arc::new(TestOutboxWriter::new());
        let failing_publisher = Arc::new(FailingPublisher);

        outbox.add_pending(test_envelope("evt-fail")).await;

        let publisher = OutboxPublisher::new(outbox.clone(), failing_publisher);
        let count = publisher.poll_once().await.unwrap();

        // Event was attempted but failed
        assert_eq!(count, 0);
        assert_eq!(outbox.failed_count().await, 1);
        assert_eq!(outbox.published_count().await, 0);
    }

    #[tokio::test]
    async fn run_stops_on_shutdown_signal() {
        let outbox = Arc::new(TestOutboxWriter::new());
        let event_bus = Arc::new(InMemoryEventBus::new());

        outbox.add_pending(test_envelope("evt-1")).await;

        let config = OutboxPublisherConfig::default()
            .with_poll_interval(Duration::from_millis(10));
        let publisher = OutboxPublisher::with_config(outbox.clone(), event_bus.clone(), config);

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        // Run publisher in background
        let handle = tokio::spawn(async move {
            publisher.run(shutdown_rx).await
        });

        // Give it time to process
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Signal shutdown
        shutdown_tx.send(true).unwrap();

        // Wait for graceful shutdown
        let result = handle.await.unwrap();
        assert!(result.is_ok());

        // Events should have been processed
        assert!(event_bus.event_count() >= 1);
    }

    #[tokio::test]
    async fn config_defaults_are_reasonable() {
        let config = OutboxPublisherConfig::default();

        assert_eq!(config.poll_interval, Duration::from_millis(100));
        assert_eq!(config.batch_size, 100);
    }
}
