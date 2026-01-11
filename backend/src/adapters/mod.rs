//! Adapters - Implementations of port interfaces.
//!
//! Adapters connect the domain to external systems:
//! - `ai` - AI/LLM provider implementations (mock, OpenAI, Anthropic)
//! - `auth` - Authentication implementations (mock, Zitadel)
//! - `events` - Event bus implementations (in-memory, Redis)
//! - `http` - HTTP/REST API implementations
//! - `membership` - Membership access control implementations
//! - `postgres` - PostgreSQL database implementations
//! - `rate_limiter` - Rate limiting implementations (in-memory, Redis)
//! - `storage` - State storage implementations (file, in-memory)
//! - `stripe` - Stripe payment provider implementation
//! - `validation` - Schema validation implementations
//! - `websocket` - WebSocket real-time update implementations

pub mod ai;
pub mod auth;
pub mod events;
pub mod http;
pub mod membership;
pub mod postgres;
pub mod rate_limiter;
pub mod storage;
pub mod stripe;
pub mod validation;
pub mod websocket;

pub use ai::{
    ai_events, AIEventCallback, AIUsageHandler, AnthropicConfig, AnthropicProvider,
    FailoverAIProvider, InMemoryUsageTracker, MockAIProvider, MockError, MockResponse,
    OpenAIConfig, OpenAIProvider,
};
pub use auth::{MockAuthProvider, MockSessionValidator};
pub use events::{IdempotentHandler, InMemoryEventBus, OutboxPublisher, OutboxPublisherConfig};
pub use membership::StubAccessChecker;
pub use postgres::{
    PostgresAccessChecker, PostgresCycleReader, PostgresCycleRepository,
    PostgresMembershipReader, PostgresMembershipRepository,
};
pub use rate_limiter::{
    GlobalLimits, InMemoryRateLimiter, IpLimits, RateLimitConfig, RedisRateLimiter,
    ResourceLimits, TierAwareRateLimiter, TierRateLimits,
};
pub use storage::{FileStateStorage, InMemoryStateStorage};
pub use stripe::{MockPaymentProvider, StripeConfig, StripePaymentAdapter};
pub use validation::JsonSchemaValidator;
pub use websocket::{
    websocket_router, ClientId, DashboardUpdate, DashboardUpdateType, RoomManager, ServerMessage,
    WebSocketEventBridge, WebSocketState, DASHBOARD_EVENT_TYPES,
};
