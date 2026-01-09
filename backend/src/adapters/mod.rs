//! Adapters - Implementations of port interfaces.
//!
//! Adapters connect the domain to external systems:
//! - `events` - Event bus implementations (in-memory, Redis)

pub mod events;

pub use events::{IdempotentHandler, InMemoryEventBus, OutboxPublisher, OutboxPublisherConfig};
