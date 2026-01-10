//! Adapters - Implementations of port interfaces.
//!
//! Adapters connect the domain to external systems:
//! - `ai` - AI/LLM provider implementations (mock, OpenAI, Anthropic)
//! - `events` - Event bus implementations (in-memory, Redis)
//! - `http` - HTTP/REST API implementations
//! - `membership` - Membership access control implementations
//! - `postgres` - PostgreSQL database implementations
//! - `storage` - State storage implementations (file, in-memory)
//! - `validation` - Schema validation implementations

pub mod ai;
pub mod events;
pub mod http;
pub mod membership;
pub mod postgres;
pub mod storage;
pub mod validation;

pub use ai::{
    ai_events, AIEventCallback, AIUsageHandler, AnthropicConfig, AnthropicProvider,
    FailoverAIProvider, InMemoryUsageTracker, MockAIProvider, MockError, MockResponse,
    OpenAIConfig, OpenAIProvider,
};
pub use events::{IdempotentHandler, InMemoryEventBus, OutboxPublisher, OutboxPublisherConfig};
pub use membership::StubAccessChecker;
pub use postgres::{PostgresCycleReader, PostgresCycleRepository};
pub use storage::{FileStateStorage, InMemoryStateStorage};
pub use validation::JsonSchemaValidator;
