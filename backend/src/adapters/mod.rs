//! Adapters - Implementations of port interfaces.
//!
//! Adapters connect the domain to external systems:
//! - `ai` - AI/LLM provider implementations (mock, OpenAI, Anthropic)
//! - `events` - Event bus implementations (in-memory, Redis)
//! - `membership` - Membership access control implementations
//! - `validation` - Schema validation implementations

pub mod ai;
pub mod events;
pub mod membership;
pub mod validation;

pub use ai::{
    ai_events, AIEventCallback, AIUsageHandler, AnthropicConfig, AnthropicProvider,
    FailoverAIProvider, MockAIProvider, MockError, MockResponse, OpenAIConfig, OpenAIProvider,
};
pub use events::{IdempotentHandler, InMemoryEventBus, OutboxPublisher, OutboxPublisherConfig};
pub use membership::StubAccessChecker;
pub use validation::JsonSchemaValidator;
