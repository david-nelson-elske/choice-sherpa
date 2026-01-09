//! Adapters - Implementations of port interfaces.
//!
//! Adapters connect the domain to external systems:
//! - `events` - Event bus implementations (in-memory, Redis)
//! - `validation` - Schema validation implementations

pub mod events;
pub mod validation;

pub use events::{IdempotentHandler, InMemoryEventBus, OutboxPublisher, OutboxPublisherConfig};
pub use validation::JsonSchemaValidator;
