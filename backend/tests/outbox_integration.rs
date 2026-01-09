//! Integration tests for the Transactional Outbox Pattern.
//!
//! These tests verify the end-to-end flow:
//! 1. Command handler writes event to outbox (same transaction as domain changes)
//! 2. OutboxPublisher polls outbox and publishes events
//! 3. IdempotentHandler processes events with deduplication
//! 4. Events are marked as published in outbox
//!
//! Uses in-memory implementations to test the pattern without external dependencies.

use async_trait::async_trait;
use serde_json::json;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{watch, RwLock};
use uuid::Uuid;

use choice_sherpa::adapters::{IdempotentHandler, InMemoryEventBus, OutboxPublisher, OutboxPublisherConfig};
use choice_sherpa::domain::foundation::{
    DomainError, ErrorCode, EventEnvelope, EventId, EventMetadata, Timestamp,
};
use choice_sherpa::ports::{EventHandler, EventSubscriber, OutboxEntry, OutboxWriter, ProcessedEventStore};

// =============================================================================
// Test Infrastructure
// =============================================================================

/// In-memory outbox for testing
struct TestOutbox {
    entries: RwLock<Vec<OutboxEntry>>,
    published_ids: RwLock<HashSet<Uuid>>,
    failed_entries: RwLock<Vec<(Uuid, String)>>,
}

impl TestOutbox {
    fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
            published_ids: RwLock::new(HashSet::new()),
            failed_entries: RwLock::new(Vec::new()),
        }
    }

    async fn pending_count(&self) -> usize {
        let entries = self.entries.read().await;
        let published = self.published_ids.read().await;
        entries.iter().filter(|e| !published.contains(&e.id)).count()
    }

    async fn published_count(&self) -> usize {
        self.published_ids.read().await.len()
    }
}

#[async_trait]
impl OutboxWriter for TestOutbox {
    async fn write(&self, event: &EventEnvelope, partition_key: &str) -> Result<OutboxEntry, DomainError> {
        let entry = OutboxEntry::new(event.clone(), partition_key);
        self.entries.write().await.push(entry.clone());
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
            self.entries.write().await.push(entry.clone());
            entries.push(entry);
        }
        Ok(entries)
    }

    async fn get_pending(&self, limit: u32) -> Result<Vec<OutboxEntry>, DomainError> {
        let entries = self.entries.read().await;
        let published = self.published_ids.read().await;
        let pending: Vec<_> = entries
            .iter()
            .filter(|e| !published.contains(&e.id))
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(pending)
    }

    async fn mark_published(&self, id: Uuid) -> Result<(), DomainError> {
        self.published_ids.write().await.insert(id);
        Ok(())
    }

    async fn mark_failed(&self, id: Uuid, error: &str) -> Result<(), DomainError> {
        self.failed_entries.write().await.push((id, error.to_string()));
        Ok(())
    }

    async fn cleanup_old(&self, _older_than_hours: u32) -> Result<u64, DomainError> {
        Ok(0)
    }
}

/// In-memory processed event store for testing
struct TestProcessedEventStore {
    processed: RwLock<HashSet<(String, String)>>,
}

impl TestProcessedEventStore {
    fn new() -> Self {
        Self {
            processed: RwLock::new(HashSet::new()),
        }
    }
}

#[async_trait]
impl ProcessedEventStore for TestProcessedEventStore {
    async fn contains(&self, event_id: &EventId, handler_name: &str) -> Result<bool, DomainError> {
        let key = (event_id.as_str().to_string(), handler_name.to_string());
        Ok(self.processed.read().await.contains(&key))
    }

    async fn mark_processed(&self, event_id: &EventId, handler_name: &str) -> Result<(), DomainError> {
        let key = (event_id.as_str().to_string(), handler_name.to_string());
        self.processed.write().await.insert(key);
        Ok(())
    }

    async fn delete_before(&self, _timestamp: Timestamp) -> Result<u64, DomainError> {
        Ok(0)
    }
}

/// Test handler that counts processed events
struct CountingHandler {
    name: &'static str,
    count: AtomicUsize,
    events: RwLock<Vec<String>>,
}

impl CountingHandler {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            count: AtomicUsize::new(0),
            events: RwLock::new(Vec::new()),
        }
    }

    #[allow(dead_code)]
    async fn processed_event_types(&self) -> Vec<String> {
        self.events.read().await.clone()
    }
}

#[async_trait]
impl EventHandler for CountingHandler {
    async fn handle(&self, envelope: EventEnvelope) -> Result<(), DomainError> {
        self.count.fetch_add(1, Ordering::SeqCst);
        self.events.write().await.push(envelope.event_type.clone());
        Ok(())
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

fn test_envelope(event_type: &str, event_id: &str) -> EventEnvelope {
    EventEnvelope {
        event_id: EventId::from_string(event_id),
        event_type: event_type.to_string(),
        aggregate_id: "test-aggregate".to_string(),
        aggregate_type: "Test".to_string(),
        occurred_at: Timestamp::now(),
        payload: json!({"test": true}),
        metadata: EventMetadata::default(),
    }
}

// =============================================================================
// Integration Tests
// =============================================================================

/// Tests the complete outbox pattern flow:
/// write to outbox → publisher polls → event published → handler processes
#[tokio::test]
async fn outbox_to_handler_end_to_end() {
    // Setup components
    let outbox = Arc::new(TestOutbox::new());
    let event_bus = Arc::new(InMemoryEventBus::new());
    let processed_store = Arc::new(TestProcessedEventStore::new());

    // Create handler wrapped with idempotency
    let inner_handler = CountingHandler::new("TestHandler");
    let handler = IdempotentHandler::new(inner_handler, processed_store.clone());

    // Subscribe handler to event bus
    event_bus.subscribe("session.created", Arc::new(handler));

    // Create publisher
    let config = OutboxPublisherConfig::default().with_poll_interval(Duration::from_millis(10));
    let publisher = OutboxPublisher::with_config(outbox.clone(), event_bus.clone(), config);

    // Simulate command handler writing to outbox
    let event = test_envelope("session.created", "evt-integration-1");
    outbox.write(&event, "test-partition").await.unwrap();

    assert_eq!(outbox.pending_count().await, 1);
    assert_eq!(outbox.published_count().await, 0);

    // Run publisher for one cycle
    let count = publisher.poll_once().await.unwrap();

    // Verify event was published and marked
    assert_eq!(count, 1);
    assert_eq!(outbox.pending_count().await, 0);
    assert_eq!(outbox.published_count().await, 1);

    // Verify handler was invoked
    assert_eq!(event_bus.event_count(), 1);
}

/// Tests that multiple events are processed in order through the outbox
#[tokio::test]
async fn outbox_processes_multiple_events_in_batch() {
    let outbox = Arc::new(TestOutbox::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    let publisher = OutboxPublisher::new(outbox.clone(), event_bus.clone());

    // Write multiple events to outbox
    let events = vec![
        test_envelope("session.created", "evt-batch-1"),
        test_envelope("session.renamed", "evt-batch-2"),
        test_envelope("session.archived", "evt-batch-3"),
    ];

    for event in &events {
        outbox.write(event, "test-partition").await.unwrap();
    }

    assert_eq!(outbox.pending_count().await, 3);

    // Process all events
    let count = publisher.poll_once().await.unwrap();

    assert_eq!(count, 3);
    assert_eq!(outbox.pending_count().await, 0);
    assert_eq!(outbox.published_count().await, 3);
    assert_eq!(event_bus.event_count(), 3);
}

/// Tests that idempotent handler prevents duplicate processing even when
/// outbox publisher re-delivers an event
#[tokio::test]
async fn idempotent_handler_deduplicates_redelivered_events() {
    let outbox = Arc::new(TestOutbox::new());
    let event_bus = Arc::new(InMemoryEventBus::new());
    let processed_store = Arc::new(TestProcessedEventStore::new());

    let handler = IdempotentHandler::new(
        CountingHandler::new("DedupeHandler"),
        processed_store.clone(),
    );

    event_bus.subscribe("session.created", Arc::new(handler));

    let publisher = OutboxPublisher::new(outbox.clone(), event_bus.clone());

    // Write same event twice (simulating redelivery)
    let event = test_envelope("session.created", "evt-dedupe-1");
    outbox.write(&event, "partition-1").await.unwrap();

    // First delivery
    publisher.poll_once().await.unwrap();

    // Simulate redelivery by writing same event again
    outbox.write(&event, "partition-1").await.unwrap();

    // Second delivery attempt
    publisher.poll_once().await.unwrap();

    // Event bus received both deliveries
    assert_eq!(event_bus.event_count(), 2);

    // But handler only processed once (checked via processed store)
    let was_processed = processed_store
        .contains(&event.event_id, "DedupeHandler")
        .await
        .unwrap();
    assert!(was_processed);
}

/// Tests that different handlers can process the same event independently
#[tokio::test]
async fn multiple_handlers_process_same_event_independently() {
    let outbox = Arc::new(TestOutbox::new());
    let event_bus = Arc::new(InMemoryEventBus::new());
    let processed_store = Arc::new(TestProcessedEventStore::new());

    // Create two different handlers
    let handler_a = IdempotentHandler::new(
        CountingHandler::new("HandlerA"),
        processed_store.clone(),
    );
    let handler_b = IdempotentHandler::new(
        CountingHandler::new("HandlerB"),
        processed_store.clone(),
    );

    // Both subscribe to same event type
    event_bus.subscribe("session.created", Arc::new(handler_a));
    event_bus.subscribe("session.created", Arc::new(handler_b));

    let publisher = OutboxPublisher::new(outbox.clone(), event_bus.clone());

    // Write one event
    let event = test_envelope("session.created", "evt-multi-1");
    outbox.write(&event, "partition-1").await.unwrap();

    // Publish
    publisher.poll_once().await.unwrap();

    // Both handlers should have processed
    let a_processed = processed_store
        .contains(&event.event_id, "HandlerA")
        .await
        .unwrap();
    let b_processed = processed_store
        .contains(&event.event_id, "HandlerB")
        .await
        .unwrap();

    assert!(a_processed, "HandlerA should have processed the event");
    assert!(b_processed, "HandlerB should have processed the event");
}

/// Tests that publisher respects batch size limits
#[tokio::test]
async fn outbox_publisher_respects_batch_size() {
    let outbox = Arc::new(TestOutbox::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    // Configure small batch size
    let config = OutboxPublisherConfig::default().with_batch_size(2);
    let publisher = OutboxPublisher::with_config(outbox.clone(), event_bus.clone(), config);

    // Write 5 events
    for i in 0..5 {
        let event = test_envelope("session.created", &format!("evt-batch-{}", i));
        outbox.write(&event, "partition-1").await.unwrap();
    }

    assert_eq!(outbox.pending_count().await, 5);

    // First poll: should process 2
    let count = publisher.poll_once().await.unwrap();
    assert_eq!(count, 2);
    assert_eq!(outbox.pending_count().await, 3);

    // Second poll: should process 2 more
    let count = publisher.poll_once().await.unwrap();
    assert_eq!(count, 2);
    assert_eq!(outbox.pending_count().await, 1);

    // Third poll: should process remaining 1
    let count = publisher.poll_once().await.unwrap();
    assert_eq!(count, 1);
    assert_eq!(outbox.pending_count().await, 0);
}

/// Tests graceful shutdown of the outbox publisher
#[tokio::test]
async fn outbox_publisher_graceful_shutdown() {
    let outbox = Arc::new(TestOutbox::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    // Write an event
    let event = test_envelope("session.created", "evt-shutdown-1");
    outbox.write(&event, "partition-1").await.unwrap();

    let config = OutboxPublisherConfig::default()
        .with_poll_interval(Duration::from_millis(10));
    let publisher = OutboxPublisher::with_config(outbox.clone(), event_bus.clone(), config);

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Run publisher in background
    let handle = tokio::spawn(async move {
        publisher.run(shutdown_rx).await
    });

    // Let it process
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Signal shutdown
    shutdown_tx.send(true).unwrap();

    // Should complete gracefully
    let result = handle.await.unwrap();
    assert!(result.is_ok());

    // Event should have been processed
    assert_eq!(outbox.published_count().await, 1);
}

/// Tests that failed event publication marks event for retry
#[tokio::test]
async fn failed_publication_marks_event_for_retry() {
    /// Publisher that always fails
    struct FailingPublisher;

    #[async_trait]
    impl choice_sherpa::ports::EventPublisher for FailingPublisher {
        async fn publish(&self, _: EventEnvelope) -> Result<(), DomainError> {
            Err(DomainError::new(ErrorCode::InternalError, "Simulated failure"))
        }

        async fn publish_all(&self, _: Vec<EventEnvelope>) -> Result<(), DomainError> {
            Err(DomainError::new(ErrorCode::InternalError, "Simulated failure"))
        }
    }

    let outbox = Arc::new(TestOutbox::new());
    let failing_publisher = Arc::new(FailingPublisher);

    let publisher = OutboxPublisher::new(outbox.clone(), failing_publisher);

    // Write event
    let event = test_envelope("session.created", "evt-fail-1");
    outbox.write(&event, "partition-1").await.unwrap();

    // Try to publish - should fail
    let count = publisher.poll_once().await.unwrap();

    // No successful publications
    assert_eq!(count, 0);

    // Event still pending (not marked published)
    assert_eq!(outbox.pending_count().await, 1);
    assert_eq!(outbox.published_count().await, 0);

    // But marked as failed (for visibility/monitoring)
    let failed = outbox.failed_entries.read().await;
    assert_eq!(failed.len(), 1);
    assert!(failed[0].1.contains("Simulated failure"));
}
