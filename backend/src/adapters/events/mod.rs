//! Event bus adapters.
//!
//! Adapters implement the event publishing and subscribing ports
//! for different environments:
//!
//! - `InMemoryEventBus` - Synchronous, in-process bus for testing
//! - `IdempotentHandler` - Wrapper for at-most-once event processing
//! - `OutboxPublisher` - Background service for reliable event delivery

mod in_memory;
mod idempotent_handler;
mod outbox_publisher;

pub use in_memory::InMemoryEventBus;
pub use idempotent_handler::IdempotentHandler;
pub use outbox_publisher::{OutboxPublisher, OutboxPublisherConfig};
