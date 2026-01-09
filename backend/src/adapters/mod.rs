//! Adapters - Implementations of port interfaces.
//!
//! Adapters connect the domain to external systems:
//! - `ai` - AI/LLM provider implementations (mock, OpenAI, Anthropic)
//! - `events` - Event bus implementations (in-memory, Redis)
//! - `validation` - Schema validation implementations

pub mod ai;
pub mod events;
pub mod validation;

pub use ai::{
    ai_events, AIEventCallback, AnthropicConfig, AnthropicProvider, FailoverAIProvider,
    MockAIProvider, MockError, MockResponse, OpenAIConfig, OpenAIProvider,
};
pub use events::{IdempotentHandler, InMemoryEventBus, OutboxPublisher, OutboxPublisherConfig};
pub use validation::JsonSchemaValidator;
