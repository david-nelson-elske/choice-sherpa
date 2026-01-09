//! AI Provider Adapters.
//!
//! Implementations of the AIProvider port for various LLM providers.
//!
//! ## Available Adapters
//!
//! - `MockAIProvider` - Configurable mock for testing
//! - `OpenAIProvider` - OpenAI GPT models (GPT-4, GPT-3.5)
//! - `AnthropicProvider` - Anthropic Claude models (Opus, Sonnet, Haiku)

mod anthropic_provider;
mod mock_provider;
mod openai_provider;

pub use anthropic_provider::{AnthropicConfig, AnthropicProvider};
pub use mock_provider::{MockAIProvider, MockError, MockResponse};
pub use openai_provider::{OpenAIConfig, OpenAIProvider};
