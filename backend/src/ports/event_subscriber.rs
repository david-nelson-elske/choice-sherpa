//! EventSubscriber port - Interface for subscribing to domain events.
//!
//! This port defines how handlers register interest in domain events
//! without knowing about the underlying transport mechanism.

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::foundation::{DomainError, EventEnvelope};

/// Handler for processing domain events.
///
/// Implementations should be:
/// - **Idempotent** - Safe to call multiple times with same event
/// - **Quick** - Long operations should be queued for async processing
/// - **Isolated** - Errors don't affect other handlers
///
/// # Example
///
/// ```ignore
/// struct DashboardUpdater { /* ... */ }
///
/// #[async_trait]
/// impl EventHandler for DashboardUpdater {
///     async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError> {
///         let payload: SessionCreated = event.payload_as()?;
///         // Update dashboard read model...
///         Ok(())
///     }
///
///     fn name(&self) -> &'static str {
///         "DashboardUpdater"
///     }
/// }
/// ```
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Process an event.
    ///
    /// This method should be idempotent - calling it multiple times
    /// with the same event should produce the same result.
    async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError>;

    /// Handler name for logging and metrics.
    ///
    /// Used for:
    /// - Error messages (e.g., "DashboardUpdater: failed to update")
    /// - Metrics labels (e.g., handler_duration_seconds{name="DashboardUpdater"})
    fn name(&self) -> &'static str;
}

/// Port for subscribing to domain events.
///
/// Handlers register interest in specific event types and are invoked
/// when matching events are published.
///
/// # Example
///
/// ```ignore
/// subscriber.subscribe("session.created", dashboard_updater);
/// subscriber.subscribe_all(&["cycle.created", "cycle.completed"], cycle_tracker);
/// ```
pub trait EventSubscriber: Send + Sync {
    /// Subscribe handler to a specific event type.
    ///
    /// The handler will be invoked for every event matching the given type.
    fn subscribe(&self, event_type: &str, handler: Arc<dyn EventHandler>);

    /// Subscribe handler to multiple event types.
    ///
    /// The same handler instance is invoked for any matching event type.
    fn subscribe_all(&self, event_types: &[&str], handler: Arc<dyn EventHandler>);
}

/// Combined trait for event bus implementations.
///
/// An EventBus provides both publishing and subscribing capabilities.
pub trait EventBus: super::EventPublisher + EventSubscriber {}

// Blanket implementation - any type that implements both traits is an EventBus
impl<T: super::EventPublisher + EventSubscriber> EventBus for T {}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile-time check that traits are object-safe
    #[allow(dead_code)]
    fn assert_handler_object_safe(_: &dyn EventHandler) {}

    #[allow(dead_code)]
    fn assert_subscriber_object_safe(_: &dyn EventSubscriber) {}

    // Compile-time check that traits are Send + Sync
    #[allow(dead_code)]
    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn event_handler_is_send_sync() {
        fn check<T: EventHandler>() {
            assert_send_sync::<T>();
        }
    }

    #[test]
    fn event_subscriber_is_send_sync() {
        fn check<T: EventSubscriber>() {
            assert_send_sync::<T>();
        }
    }
}
