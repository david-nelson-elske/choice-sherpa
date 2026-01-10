//! Ports - Interfaces for external dependencies.
//!
//! Following hexagonal architecture, ports define the contracts between
//! the domain and the outside world. Adapters implement these ports.
//!
//! ## Access Control Port
//!
//! - `AccessChecker` - Port for membership-based access control
//!
//! ## Event Ports
//!
//! - `EventPublisher` - Port for publishing domain events
//! - `EventSubscriber` - Port for subscribing to domain events
//! - `EventHandler` - Handler that processes incoming events
//! - `ProcessedEventStore` - Idempotency tracking for event handlers
//!
//! ## AI Provider Port
//!
//! - `AIProvider` - Port for LLM provider integrations (OpenAI, Anthropic)
//!
//! ## Scaling Infrastructure Ports
//!
//! - `OutboxWriter` - Transactional event persistence for guaranteed delivery
//! - `ConnectionRegistry` - Multi-server WebSocket connection tracking
//! - `CircuitBreaker` - External service resilience pattern
//!
//! See `docs/architecture/SCALING-READINESS.md` for architectural details.

mod access_checker;
mod ai_provider;
mod circuit_breaker;
mod connection_registry;
mod cycle_repository;
mod event_publisher;
mod event_subscriber;
mod membership_reader;
mod membership_repository;
mod outbox_writer;
mod payment_provider;
mod processed_event_store;
mod promo_code_validator;
mod schema_validator;
mod session_reader;
mod session_repository;
mod usage_tracker;

pub use access_checker::{AccessChecker, AccessDeniedReason, AccessResult, UsageStats};
pub use ai_provider::{
    AIError, AIProvider, CompletionRequest, CompletionResponse, FinishReason, Message,
    MessageRole, ProviderInfo, RequestMetadata, StreamChunk, TokenUsage,
};
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
pub use connection_registry::{ConnectionRegistry, ConnectionRegistryError, ServerId};
pub use cycle_repository::CycleRepository;
pub use event_publisher::EventPublisher;
pub use event_subscriber::{EventBus, EventHandler, EventSubscriber};
pub use membership_reader::{
    MembershipReader, MembershipStatistics, MembershipSummary, MembershipView, StatusCounts,
    TierCounts,
};
pub use membership_repository::MembershipRepository;
pub use outbox_writer::{OutboxEntry, OutboxStatus, OutboxWriter};
pub use payment_provider::{
    CheckoutSession, CreateCheckoutRequest, CreateCustomerRequest, CreateSubscriptionRequest,
    Customer, PaymentError, PaymentErrorCode, PaymentProvider, PortalSession, Subscription,
    SubscriptionStatus, WebhookEvent, WebhookEventData, WebhookEventType,
};
pub use processed_event_store::ProcessedEventStore;
pub use schema_validator::{ComponentSchemaValidator, SchemaValidationError};
pub use session_reader::{ListOptions, SessionList, SessionReader, SessionSummary, SessionView};
pub use session_repository::SessionRepository;
pub use promo_code_validator::{
    PromoCodeInvalidReason, PromoCodeValidation, PromoCodeValidator,
};
pub use usage_tracker::{
    ProviderUsage, UsageLimitStatus, UsageRecord, UsageSummary, UsageTracker, UsageTrackerError,
};
