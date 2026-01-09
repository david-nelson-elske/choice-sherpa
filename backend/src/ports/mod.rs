//! Ports - Interfaces for external dependencies.
//!
//! Following hexagonal architecture, ports define the contracts between
//! the domain and the outside world. Adapters implement these ports.
//!
//! ## Event Ports
//!
//! - `EventPublisher` - Port for publishing domain events
//! - `EventSubscriber` - Port for subscribing to domain events
//! - `EventHandler` - Handler that processes incoming events
//! - `ProcessedEventStore` - Idempotency tracking for event handlers
//!
//! ## Webhook Ports
//!
//! - `WebhookEventRepository` - Stripe webhook idempotency tracking
//!
//! ## Scaling Infrastructure Ports
//!
//! - `OutboxWriter` - Transactional event persistence for guaranteed delivery
//! - `ConnectionRegistry` - Multi-server WebSocket connection tracking
//! - `CircuitBreaker` - External service resilience pattern
//!
//! See `docs/architecture/SCALING-READINESS.md` for architectural details.

mod event_publisher;
mod event_subscriber;
mod outbox_writer;
mod connection_registry;
mod circuit_breaker;
mod processed_event_store;
mod webhook_event_repository;

pub use event_publisher::EventPublisher;
pub use event_subscriber::{EventBus, EventHandler, EventSubscriber};
pub use outbox_writer::{OutboxWriter, OutboxEntry, OutboxStatus};
pub use connection_registry::{ConnectionRegistry, ConnectionRegistryError, ServerId};
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
pub use processed_event_store::ProcessedEventStore;
pub use webhook_event_repository::{WebhookEventRepository, WebhookEventRecord, WebhookResult, SaveResult};
