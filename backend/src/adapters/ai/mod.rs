//! AI Provider Adapters.
//!
//! Implementations of the AIProvider port for various LLM providers.
//!
//! ## Available Adapters
//!
//! - `MockAIProvider` - Configurable mock for testing

mod mock_provider;

pub use mock_provider::{MockAIProvider, MockError, MockResponse};
