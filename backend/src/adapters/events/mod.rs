//! Event bus adapters.
//!
//! Adapters implement the event publishing and subscribing ports
//! for different environments:
//!
//! - `InMemoryEventBus` - Synchronous, in-process bus for testing
//! - `IdempotentHandler` - Wrapper for at-most-once event processing

mod in_memory;
mod idempotent_handler;

pub use in_memory::InMemoryEventBus;
pub use idempotent_handler::IdempotentHandler;
