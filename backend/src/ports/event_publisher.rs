//! EventPublisher port - Interface for publishing domain events.
//!
//! This port defines how the domain publishes events without knowing
//! about the underlying transport mechanism (in-memory, Redis, etc.).

use async_trait::async_trait;

use crate::domain::foundation::{DomainError, EventEnvelope};

/// Port for publishing domain events.
///
/// Implementations must ensure:
/// - Events are delivered at-least-once (handlers may receive duplicates)
/// - `publish_all` is atomic where supported by the adapter
/// - Errors are propagated to the caller
///
/// # Example
///
/// ```ignore
/// let event = EventEnvelope::new("session.created", session_id, "Session", payload);
/// publisher.publish(event).await?;
/// ```
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a single event.
    ///
    /// The event is wrapped in an `EventEnvelope` containing:
    /// - Event ID for deduplication
    /// - Event type for routing
    /// - Aggregate context for correlation
    /// - Metadata for tracing
    async fn publish(&self, event: EventEnvelope) -> Result<(), DomainError>;

    /// Publish multiple events atomically.
    ///
    /// All events are published or none are (where supported by adapter).
    /// For adapters that don't support atomic publishing, events are
    /// published sequentially with best-effort delivery.
    async fn publish_all(&self, events: Vec<EventEnvelope>) -> Result<(), DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;

    // Compile-time check that trait is object-safe
    #[allow(dead_code)]
    fn assert_object_safe(_: &dyn EventPublisher) {}

    // Compile-time check that trait is Send + Sync
    #[allow(dead_code)]
    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn event_publisher_is_send_sync() {
        // This will fail to compile if EventPublisher is not Send + Sync
        fn check<T: EventPublisher>() {
            assert_send_sync::<T>();
        }
        // We just need the function to exist to prove the constraint
    }
}
