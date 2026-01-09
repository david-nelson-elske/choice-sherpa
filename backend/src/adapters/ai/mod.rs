//! AI Provider Adapters.
//!
//! Implementations of the AIProvider port for various LLM providers.
//!
//! ## Available Adapters
//!
//! - `MockAIProvider` - Configurable mock for testing
//! - `OpenAIProvider` - OpenAI GPT models (GPT-4, GPT-3.5)
//! - `AnthropicProvider` - Anthropic Claude models (Opus, Sonnet, Haiku)
//! - `FailoverAIProvider` - Wrapper with automatic failover between providers
//! - `AIUsageHandler` - Event handler for tracking AI token usage

mod anthropic_provider;
mod failover_provider;
mod mock_provider;
mod openai_provider;
mod usage_handler;

pub use anthropic_provider::{AnthropicConfig, AnthropicProvider};
pub use failover_provider::{events as ai_events, AIEventCallback, FailoverAIProvider};
pub use mock_provider::{MockAIProvider, MockError, MockResponse};
pub use openai_provider::{OpenAIConfig, OpenAIProvider};
pub use usage_handler::AIUsageHandler;
