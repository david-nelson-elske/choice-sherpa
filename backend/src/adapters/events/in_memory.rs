//! In-memory event bus implementation for testing.
//!
//! Provides synchronous, deterministic event delivery for unit tests.
//!
//! # Security Note
//!
//! This adapter is for **testing only** and should not be used in production.
//! It uses `.expect()` on lock operations which will panic if locks are poisoned.
//! Production code should use the Redis event bus adapter.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::domain::foundation::{DomainError, ErrorCode, EventEnvelope};
use crate::ports::{EventHandler, EventPublisher, EventSubscriber};

/// In-memory event bus for testing.
///
/// Features:
/// - Synchronous delivery (deterministic for tests)
/// - Event capture for assertions
/// - Handler registration and invocation
///
/// # Panics
///
/// Methods may panic if internal locks are poisoned. This is acceptable
/// for test code but this adapter should NOT be used in production.
///
/// # Example
///
/// ```ignore
/// let bus = Arc::new(InMemoryEventBus::new());
///
/// // Publish events
/// bus.publish(envelope).await?;
///
/// // Assert in tests
/// assert_eq!(bus.event_count(), 1);
/// assert!(bus.has_event("session.created"));
/// ```
pub struct InMemoryEventBus {
    handlers: RwLock<HashMap<String, Vec<Arc<dyn EventHandler>>>>,
    published: RwLock<Vec<EventEnvelope>>,
}

impl InMemoryEventBus {
    /// Creates a new empty event bus.
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
            published: RwLock::new(Vec::new()),
        }
    }

    // === Test Helpers ===

    /// Returns all published events (for test assertions).
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    pub fn published_events(&self) -> Vec<EventEnvelope> {
        self.published
            .read()
            .expect("InMemoryEventBus: published lock poisoned")
            .clone()
    }

    /// Returns events of a specific type.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    pub fn events_of_type(&self, event_type: &str) -> Vec<EventEnvelope> {
        self.published_events()
            .into_iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }

    /// Returns events for a specific aggregate.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    pub fn events_for_aggregate(&self, aggregate_id: &str) -> Vec<EventEnvelope> {
        self.published_events()
            .into_iter()
            .filter(|e| e.aggregate_id == aggregate_id)
            .collect()
    }

    /// Clears all published events (for test isolation).
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    pub fn clear(&self) {
        self.published
            .write()
            .expect("InMemoryEventBus: published write lock poisoned")
            .clear();
    }

    /// Returns count of published events.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    pub fn event_count(&self) -> usize {
        self.published
            .read()
            .expect("InMemoryEventBus: published lock poisoned")
            .len()
    }

    /// Checks if a specific event type was published.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    pub fn has_event(&self, event_type: &str) -> bool {
        self.published
            .read()
            .expect("InMemoryEventBus: published lock poisoned")
            .iter()
            .any(|e| e.event_type == event_type)
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventPublisher for InMemoryEventBus {
    async fn publish(&self, event: EventEnvelope) -> Result<(), DomainError> {
        // Store for test assertions
        self.published
            .write()
            .expect("InMemoryEventBus: published write lock poisoned")
            .push(event.clone());

        // Clone handlers to release lock before await points
        let type_handlers: Vec<Arc<dyn EventHandler>> = {
            let handlers = self
                .handlers
                .read()
                .expect("InMemoryEventBus: handlers lock poisoned");
            handlers
                .get(&event.event_type)
                .cloned()
                .unwrap_or_default()
        };

        // Invoke handlers (lock is released)
        let mut errors = Vec::new();
        for handler in type_handlers {
            if let Err(e) = handler.handle(event.clone()).await {
                errors.push(format!("{}: {}", handler.name(), e));
            }
        }

        if !errors.is_empty() {
            return Err(DomainError::new(
                ErrorCode::InternalError,
                format!("Handler errors: {}", errors.join(", ")),
            ));
        }

        Ok(())
    }

    async fn publish_all(&self, events: Vec<EventEnvelope>) -> Result<(), DomainError> {
        for event in events {
            self.publish(event).await?;
        }
        Ok(())
    }
}

impl EventSubscriber for InMemoryEventBus {
    fn subscribe(&self, event_type: &str, handler: Arc<dyn EventHandler>) {
        let mut handlers = self
            .handlers
            .write()
            .expect("InMemoryEventBus: handlers write lock poisoned");
        handlers
            .entry(event_type.to_string())
            .or_default()
            .push(handler);
    }

    fn subscribe_all(&self, event_types: &[&str], handler: Arc<dyn EventHandler>) {
        let mut handlers = self
            .handlers
            .write()
            .expect("InMemoryEventBus: handlers write lock poisoned");
        for event_type in event_types {
            handlers
                .entry(event_type.to_string())
                .or_default()
                .push(Arc::clone(&handler));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{EventId, EventMetadata, Timestamp};
    use serde_json::json;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    fn test_envelope(event_type: &str, aggregate_id: &str) -> EventEnvelope {
        EventEnvelope {
            event_id: EventId::new(),
            event_type: event_type.to_string(),
            aggregate_id: aggregate_id.to_string(),
            aggregate_type: "Test".to_string(),
            occurred_at: Timestamp::now(),
            payload: json!({}),
            metadata: EventMetadata::default(),
        }
    }

    #[tokio::test]
    async fn publish_stores_event() {
        let bus = InMemoryEventBus::new();
        let event = test_envelope("test.event", "agg-1");

        bus.publish(event).await.unwrap();

        assert_eq!(bus.event_count(), 1);
        assert!(bus.has_event("test.event"));
    }

    #[tokio::test]
    async fn events_of_type_filters_correctly() {
        let bus = InMemoryEventBus::new();

        bus.publish(test_envelope("type.a", "1")).await.unwrap();
        bus.publish(test_envelope("type.b", "2")).await.unwrap();
        bus.publish(test_envelope("type.a", "3")).await.unwrap();

        let type_a = bus.events_of_type("type.a");
        assert_eq!(type_a.len(), 2);
    }

    #[tokio::test]
    async fn events_for_aggregate_filters_correctly() {
        let bus = InMemoryEventBus::new();

        bus.publish(test_envelope("type.a", "agg-1")).await.unwrap();
        bus.publish(test_envelope("type.b", "agg-2")).await.unwrap();
        bus.publish(test_envelope("type.c", "agg-1")).await.unwrap();

        let agg_events = bus.events_for_aggregate("agg-1");
        assert_eq!(agg_events.len(), 2);
    }

    #[tokio::test]
    async fn handler_receives_published_event() {
        let bus = Arc::new(InMemoryEventBus::new());
        let received = Arc::new(AtomicBool::new(false));

        struct TestHandler(Arc<AtomicBool>);

        #[async_trait]
        impl EventHandler for TestHandler {
            async fn handle(&self, _: EventEnvelope) -> Result<(), DomainError> {
                self.0.store(true, Ordering::SeqCst);
                Ok(())
            }
            fn name(&self) -> &'static str {
                "TestHandler"
            }
        }

        bus.subscribe("test.event", Arc::new(TestHandler(received.clone())));
        bus.publish(test_envelope("test.event", "1")).await.unwrap();

        assert!(received.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn multiple_handlers_all_invoked() {
        let bus = Arc::new(InMemoryEventBus::new());
        let counter = Arc::new(AtomicUsize::new(0));

        struct CountingHandler(Arc<AtomicUsize>);

        #[async_trait]
        impl EventHandler for CountingHandler {
            async fn handle(&self, _: EventEnvelope) -> Result<(), DomainError> {
                self.0.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
            fn name(&self) -> &'static str {
                "CountingHandler"
            }
        }

        bus.subscribe("test.event", Arc::new(CountingHandler(counter.clone())));
        bus.subscribe("test.event", Arc::new(CountingHandler(counter.clone())));
        bus.subscribe("test.event", Arc::new(CountingHandler(counter.clone())));

        bus.publish(test_envelope("test.event", "1")).await.unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn subscribe_all_registers_for_multiple_types() {
        let bus = Arc::new(InMemoryEventBus::new());
        let received = Arc::new(AtomicUsize::new(0));

        struct CountingHandler(Arc<AtomicUsize>);

        #[async_trait]
        impl EventHandler for CountingHandler {
            async fn handle(&self, _: EventEnvelope) -> Result<(), DomainError> {
                self.0.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
            fn name(&self) -> &'static str {
                "CountingHandler"
            }
        }

        bus.subscribe_all(
            &["type.a", "type.b", "type.c"],
            Arc::new(CountingHandler(received.clone())),
        );

        bus.publish(test_envelope("type.a", "1")).await.unwrap();
        bus.publish(test_envelope("type.b", "2")).await.unwrap();
        bus.publish(test_envelope("type.d", "3")).await.unwrap(); // Not subscribed

        assert_eq!(received.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn clear_removes_all_events() {
        let bus = InMemoryEventBus::new();

        bus.publish(test_envelope("test.event", "1")).await.unwrap();
        bus.publish(test_envelope("test.event", "2")).await.unwrap();

        assert_eq!(bus.event_count(), 2);

        bus.clear();

        assert_eq!(bus.event_count(), 0);
    }

    #[tokio::test]
    async fn publish_all_publishes_events() {
        let bus = InMemoryEventBus::new();

        let events = vec![
            test_envelope("type.a", "1"),
            test_envelope("type.b", "2"),
            test_envelope("type.c", "3"),
        ];

        bus.publish_all(events).await.unwrap();

        assert_eq!(bus.event_count(), 3);
    }

    #[tokio::test]
    async fn handler_error_is_propagated() {
        let bus = Arc::new(InMemoryEventBus::new());

        struct FailingHandler;

        #[async_trait]
        impl EventHandler for FailingHandler {
            async fn handle(&self, _: EventEnvelope) -> Result<(), DomainError> {
                Err(DomainError::new(ErrorCode::InternalError, "Handler failed"))
            }
            fn name(&self) -> &'static str {
                "FailingHandler"
            }
        }

        bus.subscribe("test.event", Arc::new(FailingHandler));
        let result = bus.publish(test_envelope("test.event", "1")).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("FailingHandler"));
    }
}
