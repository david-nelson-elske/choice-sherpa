//! AI Provider Port - Interface for LLM provider integrations.
//!
//! This port abstracts all interactions with AI/LLM providers (OpenAI, Anthropic, etc.),
//! enabling the conversation module to generate completions without coupling to specific providers.
//!
//! # Design
//!
//! - Supports both streaming and non-streaming completions
//! - Provider-agnostic message format
//! - Built-in token usage and cost tracking
//! - Error types for common failure modes (rate limits, context too long, etc.)
//!
//! # Example
//!
//! ```ignore
//! use async_trait::async_trait;
//!
//! struct MockProvider;
//!
//! #[async_trait]
//! impl AIProvider for MockProvider {
//!     async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AIError> {
//!         Ok(CompletionResponse {
//!             content: "Hello!".to_string(),
//!             usage: TokenUsage::default(),
//!             model: "mock".to_string(),
//!             finish_reason: FinishReason::Stop,
//!         })
//!     }
//!     // ... other methods
//! }
//! ```

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use crate::domain::foundation::{
    ComponentType, ConversationId, SessionId, UserId,
};

/// Port for AI/LLM provider interactions.
///
/// Implementations connect to external AI services (OpenAI, Anthropic, etc.)
/// and translate between the provider-specific API and our domain types.
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Generate a single completion (non-streaming).
    ///
    /// Use this for quick, simple completions where streaming overhead isn't needed.
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AIError>;

    /// Generate a streaming completion.
    ///
    /// Returns a stream of chunks as they arrive from the provider.
    /// The final chunk contains the complete token usage information.
    async fn stream_complete(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, AIError>> + Send>>, AIError>;

    /// Estimate token count for text (for cost estimation before API call).
    ///
    /// This should use a tokenizer appropriate for the provider's model.
    fn estimate_tokens(&self, text: &str) -> u32;

    /// Get provider information (name, model, capabilities).
    fn provider_info(&self) -> ProviderInfo;
}

/// Request for AI completion.
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    /// Conversation messages (history + current user message).
    pub messages: Vec<Message>,
    /// System prompt to guide model behavior.
    pub system_prompt: Option<String>,
    /// Maximum tokens to generate.
    pub max_tokens: Option<u32>,
    /// Temperature for response randomness (0.0 = deterministic, 1.0+ = creative).
    pub temperature: Option<f32>,
    /// Component type for prompt templating.
    pub component_type: Option<ComponentType>,
    /// Request metadata for tracing and billing.
    pub metadata: RequestMetadata,
}

impl CompletionRequest {
    /// Creates a new completion request with required metadata.
    pub fn new(metadata: RequestMetadata) -> Self {
        Self {
            messages: Vec::new(),
            system_prompt: None,
            max_tokens: None,
            temperature: None,
            component_type: None,
            metadata,
        }
    }

    /// Adds a message to the conversation.
    pub fn with_message(mut self, role: MessageRole, content: impl Into<String>) -> Self {
        self.messages.push(Message {
            role,
            content: content.into(),
        });
        self
    }

    /// Sets the system prompt.
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Sets the maximum tokens to generate.
    pub fn with_max_tokens(mut self, max: u32) -> Self {
        self.max_tokens = Some(max);
        self
    }

    /// Sets the temperature.
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    /// Sets the component type for prompt templating.
    pub fn with_component_type(mut self, component_type: ComponentType) -> Self {
        self.component_type = Some(component_type);
        self
    }
}

/// A message in the conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Who sent this message.
    pub role: MessageRole,
    /// Message content.
    pub content: String,
}

impl Message {
    /// Creates a new message.
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }

    /// Creates a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(MessageRole::System, content)
    }

    /// Creates a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(MessageRole::User, content)
    }

    /// Creates an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(MessageRole::Assistant, content)
    }
}

/// Role of the message sender.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System instructions (guides model behavior).
    System,
    /// User input.
    User,
    /// Assistant (model) response.
    Assistant,
}

/// Request metadata for tracing and billing.
#[derive(Debug, Clone)]
pub struct RequestMetadata {
    /// User making the request.
    pub user_id: UserId,
    /// Session containing this conversation.
    pub session_id: SessionId,
    /// Conversation within the component.
    pub conversation_id: ConversationId,
    /// Trace ID for distributed tracing.
    pub trace_id: String,
}

impl RequestMetadata {
    /// Creates new request metadata.
    pub fn new(
        user_id: UserId,
        session_id: SessionId,
        conversation_id: ConversationId,
        trace_id: impl Into<String>,
    ) -> Self {
        Self {
            user_id,
            session_id,
            conversation_id,
            trace_id: trace_id.into(),
        }
    }
}

/// Response from AI completion.
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    /// Generated content.
    pub content: String,
    /// Token usage and cost.
    pub usage: TokenUsage,
    /// Model that generated the response.
    pub model: String,
    /// Why the model stopped generating.
    pub finish_reason: FinishReason,
}

/// Token usage information for billing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Tokens in the prompt.
    pub prompt_tokens: u32,
    /// Tokens in the completion.
    pub completion_tokens: u32,
    /// Total tokens (prompt + completion).
    pub total_tokens: u32,
    /// Estimated cost in cents (for billing).
    pub estimated_cost_cents: u32,
}

impl TokenUsage {
    /// Creates new token usage.
    pub fn new(prompt_tokens: u32, completion_tokens: u32, cost_cents: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            estimated_cost_cents: cost_cents,
        }
    }

    /// Creates zero usage.
    pub fn zero() -> Self {
        Self::default()
    }
}

/// Reason the model stopped generating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Natural stop (end of response).
    Stop,
    /// Hit max_tokens limit.
    Length,
    /// Content was filtered for safety.
    ContentFilter,
    /// An error occurred.
    Error,
}

/// Streaming chunk from AI completion.
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// New content in this chunk.
    pub delta: String,
    /// If present, generation is complete.
    pub finish_reason: Option<FinishReason>,
    /// Token usage (only present on final chunk).
    pub usage: Option<TokenUsage>,
}

impl StreamChunk {
    /// Creates a content chunk.
    pub fn content(delta: impl Into<String>) -> Self {
        Self {
            delta: delta.into(),
            finish_reason: None,
            usage: None,
        }
    }

    /// Creates a final chunk with usage information.
    pub fn final_chunk(finish_reason: FinishReason, usage: TokenUsage) -> Self {
        Self {
            delta: String::new(),
            finish_reason: Some(finish_reason),
            usage: Some(usage),
        }
    }

    /// Returns true if this is the final chunk.
    pub fn is_final(&self) -> bool {
        self.finish_reason.is_some()
    }
}

/// Provider information and capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Provider name (e.g., "openai", "anthropic").
    pub name: String,
    /// Model identifier (e.g., "gpt-4-turbo", "claude-3-opus").
    pub model: String,
    /// Maximum context window size in tokens.
    pub max_context_tokens: u32,
    /// Whether streaming is supported.
    pub supports_streaming: bool,
    /// Whether function/tool calling is supported.
    pub supports_functions: bool,
}

impl ProviderInfo {
    /// Creates new provider info.
    pub fn new(
        name: impl Into<String>,
        model: impl Into<String>,
        max_context_tokens: u32,
    ) -> Self {
        Self {
            name: name.into(),
            model: model.into(),
            max_context_tokens,
            supports_streaming: true,
            supports_functions: false,
        }
    }

    /// Sets streaming support.
    pub fn with_streaming(mut self, supports: bool) -> Self {
        self.supports_streaming = supports;
        self
    }

    /// Sets function calling support.
    pub fn with_functions(mut self, supports: bool) -> Self {
        self.supports_functions = supports;
        self
    }
}

/// AI provider errors.
#[derive(Debug, thiserror::Error)]
pub enum AIError {
    /// Rate limited by provider.
    #[error("rate limited: retry after {retry_after_secs}s")]
    RateLimited {
        /// Seconds until retry is allowed.
        retry_after_secs: u32,
    },

    /// Context (prompt + history) exceeds model limit.
    #[error("context too long: {tokens} tokens exceeds {max} limit")]
    ContextTooLong {
        /// Actual token count.
        tokens: u32,
        /// Maximum allowed.
        max: u32,
    },

    /// Content was filtered for safety.
    #[error("content filtered: {reason}")]
    ContentFiltered {
        /// Reason for filtering.
        reason: String,
    },

    /// Provider is unavailable.
    #[error("provider unavailable: {message}")]
    Unavailable {
        /// Error details.
        message: String,
    },

    /// API key or authentication failed.
    #[error("authentication failed")]
    AuthenticationFailed,

    /// Network error during request.
    #[error("network error: {0}")]
    Network(String),

    /// Failed to parse provider response.
    #[error("parse error: {0}")]
    Parse(String),

    /// Invalid request configuration.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// Daily or session cost limit exceeded.
    #[error("cost limit exceeded: {spent_cents} cents spent, limit is {limit_cents} cents")]
    CostLimitExceeded {
        /// Amount already spent.
        spent_cents: u32,
        /// Maximum allowed.
        limit_cents: u32,
    },

    /// Request timed out.
    #[error("request timed out after {timeout_secs}s")]
    Timeout {
        /// Configured timeout.
        timeout_secs: u32,
    },
}

impl AIError {
    /// Creates a rate limited error.
    pub fn rate_limited(retry_after_secs: u32) -> Self {
        Self::RateLimited { retry_after_secs }
    }

    /// Creates a context too long error.
    pub fn context_too_long(tokens: u32, max: u32) -> Self {
        Self::ContextTooLong { tokens, max }
    }

    /// Creates a content filtered error.
    pub fn content_filtered(reason: impl Into<String>) -> Self {
        Self::ContentFiltered {
            reason: reason.into(),
        }
    }

    /// Creates an unavailable error.
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::Unavailable {
            message: message.into(),
        }
    }

    /// Creates a network error.
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network(message.into())
    }

    /// Creates a parse error.
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse(message.into())
    }

    /// Creates a cost limit exceeded error.
    pub fn cost_limit_exceeded(spent_cents: u32, limit_cents: u32) -> Self {
        Self::CostLimitExceeded {
            spent_cents,
            limit_cents,
        }
    }

    /// Returns true if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AIError::RateLimited { .. }
                | AIError::Unavailable { .. }
                | AIError::Network(_)
                | AIError::Timeout { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_metadata() -> RequestMetadata {
        RequestMetadata::new(
            UserId::new("test-user").unwrap(),
            SessionId::new(),
            ConversationId::new(),
            "trace-123",
        )
    }

    #[test]
    fn completion_request_builder_works() {
        let request = CompletionRequest::new(test_metadata())
            .with_message(MessageRole::User, "Hello")
            .with_system_prompt("Be helpful")
            .with_max_tokens(100)
            .with_temperature(0.7)
            .with_component_type(ComponentType::IssueRaising);

        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, MessageRole::User);
        assert_eq!(request.messages[0].content, "Hello");
        assert_eq!(request.system_prompt, Some("Be helpful".to_string()));
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.temperature, Some(0.7));
        assert_eq!(request.component_type, Some(ComponentType::IssueRaising));
    }

    #[test]
    fn message_constructors_work() {
        let system = Message::system("You are helpful");
        let user = Message::user("Hello");
        let assistant = Message::assistant("Hi there");

        assert_eq!(system.role, MessageRole::System);
        assert_eq!(user.role, MessageRole::User);
        assert_eq!(assistant.role, MessageRole::Assistant);
    }

    #[test]
    fn token_usage_calculates_total() {
        let usage = TokenUsage::new(100, 50, 15);
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
        assert_eq!(usage.estimated_cost_cents, 15);
    }

    #[test]
    fn token_usage_zero_is_empty() {
        let usage = TokenUsage::zero();
        assert_eq!(usage.total_tokens, 0);
        assert_eq!(usage.estimated_cost_cents, 0);
    }

    #[test]
    fn stream_chunk_content_is_not_final() {
        let chunk = StreamChunk::content("Hello");
        assert!(!chunk.is_final());
        assert_eq!(chunk.delta, "Hello");
        assert!(chunk.finish_reason.is_none());
        assert!(chunk.usage.is_none());
    }

    #[test]
    fn stream_chunk_final_has_usage() {
        let usage = TokenUsage::new(10, 5, 1);
        let chunk = StreamChunk::final_chunk(FinishReason::Stop, usage.clone());

        assert!(chunk.is_final());
        assert_eq!(chunk.delta, "");
        assert_eq!(chunk.finish_reason, Some(FinishReason::Stop));
        assert_eq!(chunk.usage, Some(usage));
    }

    #[test]
    fn provider_info_builder_works() {
        let info = ProviderInfo::new("openai", "gpt-4-turbo", 128000)
            .with_streaming(true)
            .with_functions(true);

        assert_eq!(info.name, "openai");
        assert_eq!(info.model, "gpt-4-turbo");
        assert_eq!(info.max_context_tokens, 128000);
        assert!(info.supports_streaming);
        assert!(info.supports_functions);
    }

    #[test]
    fn ai_error_constructors_work() {
        let rate_limited = AIError::rate_limited(30);
        assert!(matches!(rate_limited, AIError::RateLimited { retry_after_secs: 30 }));

        let context_error = AIError::context_too_long(200000, 128000);
        assert!(matches!(
            context_error,
            AIError::ContextTooLong { tokens: 200000, max: 128000 }
        ));

        let filtered = AIError::content_filtered("unsafe content");
        assert!(matches!(filtered, AIError::ContentFiltered { .. }));
    }

    #[test]
    fn ai_error_retryable_classification() {
        assert!(AIError::rate_limited(30).is_retryable());
        assert!(AIError::unavailable("down").is_retryable());
        assert!(AIError::network("timeout").is_retryable());
        assert!(AIError::Timeout { timeout_secs: 30 }.is_retryable());

        assert!(!AIError::AuthenticationFailed.is_retryable());
        assert!(!AIError::context_too_long(100, 50).is_retryable());
        assert!(!AIError::content_filtered("bad").is_retryable());
        assert!(!AIError::cost_limit_exceeded(100, 50).is_retryable());
    }

    #[test]
    fn message_role_serializes_lowercase() {
        let json = serde_json::to_string(&MessageRole::User).unwrap();
        assert_eq!(json, "\"user\"");

        let json = serde_json::to_string(&MessageRole::Assistant).unwrap();
        assert_eq!(json, "\"assistant\"");

        let json = serde_json::to_string(&MessageRole::System).unwrap();
        assert_eq!(json, "\"system\"");
    }

    #[test]
    fn finish_reason_serializes_snake_case() {
        let json = serde_json::to_string(&FinishReason::Stop).unwrap();
        assert_eq!(json, "\"stop\"");

        let json = serde_json::to_string(&FinishReason::ContentFilter).unwrap();
        assert_eq!(json, "\"content_filter\"");
    }

    #[test]
    fn ai_error_displays_correctly() {
        let err = AIError::rate_limited(30);
        assert_eq!(err.to_string(), "rate limited: retry after 30s");

        let err = AIError::context_too_long(200000, 128000);
        assert_eq!(
            err.to_string(),
            "context too long: 200000 tokens exceeds 128000 limit"
        );

        let err = AIError::cost_limit_exceeded(1000, 500);
        assert_eq!(
            err.to_string(),
            "cost limit exceeded: 1000 cents spent, limit is 500 cents"
        );
    }
}
