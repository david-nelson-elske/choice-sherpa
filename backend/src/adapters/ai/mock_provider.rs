//! Mock AI Provider for testing.
//!
//! Provides a configurable mock implementation of the AIProvider port,
//! allowing tests to run without calling real AI APIs.
//!
//! # Features
//!
//! - Pre-configured responses
//! - Simulated delays for timeout testing
//! - Error injection for resilience testing
//! - Call tracking for verification
//!
//! # Example
//!
//! ```ignore
//! let provider = MockAIProvider::new()
//!     .with_response("Hello, I'm the assistant!")
//!     .with_delay(Duration::from_millis(100));
//!
//! let response = provider.complete(request).await?;
//! assert_eq!(response.content, "Hello, I'm the assistant!");
//! ```

use async_trait::async_trait;
use futures::stream::{self, Stream, StreamExt};
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

use crate::ports::{
    AIError, AIProvider, CompletionRequest, CompletionResponse, FinishReason, ProviderInfo,
    StreamChunk, TokenUsage,
};

/// Mock AI provider for testing.
///
/// Configurable to return specific responses, simulate delays, or inject errors.
#[derive(Debug, Clone)]
pub struct MockAIProvider {
    /// Pre-configured responses (consumed in order).
    responses: Arc<Mutex<VecDeque<MockResponse>>>,
    /// Provider info to return.
    info: ProviderInfo,
    /// Simulated latency per request.
    delay: Duration,
    /// Call history for verification.
    calls: Arc<Mutex<Vec<CompletionRequest>>>,
}

/// A configured mock response.
#[derive(Debug, Clone)]
pub enum MockResponse {
    /// Return a successful completion.
    Success {
        content: String,
        usage: TokenUsage,
        finish_reason: FinishReason,
    },
    /// Return an error.
    Error(MockError),
}

/// Mock error types for testing error handling.
#[derive(Debug, Clone)]
pub enum MockError {
    /// Simulate rate limiting.
    RateLimited { retry_after_secs: u32 },
    /// Simulate context too long.
    ContextTooLong { tokens: u32, max: u32 },
    /// Simulate content filtering.
    ContentFiltered { reason: String },
    /// Simulate provider unavailable.
    Unavailable { message: String },
    /// Simulate authentication failure.
    AuthenticationFailed,
    /// Simulate network error.
    Network { message: String },
    /// Simulate timeout.
    Timeout { timeout_secs: u32 },
}

impl From<MockError> for AIError {
    fn from(err: MockError) -> Self {
        match err {
            MockError::RateLimited { retry_after_secs } => AIError::rate_limited(retry_after_secs),
            MockError::ContextTooLong { tokens, max } => AIError::context_too_long(tokens, max),
            MockError::ContentFiltered { reason } => AIError::content_filtered(reason),
            MockError::Unavailable { message } => AIError::unavailable(message),
            MockError::AuthenticationFailed => AIError::AuthenticationFailed,
            MockError::Network { message } => AIError::network(message),
            MockError::Timeout { timeout_secs } => AIError::Timeout { timeout_secs },
        }
    }
}

impl Default for MockAIProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MockAIProvider {
    /// Creates a new mock provider with default settings.
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(VecDeque::new())),
            info: ProviderInfo::new("mock", "mock-model-1", 128000)
                .with_streaming(true)
                .with_functions(false),
            delay: Duration::ZERO,
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Adds a successful response to the queue.
    pub fn with_response(self, content: impl Into<String>) -> Self {
        self.with_response_full(
            content,
            TokenUsage::new(10, 20, 1),
            FinishReason::Stop,
        )
    }

    /// Adds a successful response with full configuration.
    pub fn with_response_full(
        self,
        content: impl Into<String>,
        usage: TokenUsage,
        finish_reason: FinishReason,
    ) -> Self {
        let mut responses = self.responses.lock().unwrap();
        responses.push_back(MockResponse::Success {
            content: content.into(),
            usage,
            finish_reason,
        });
        drop(responses);
        self
    }

    /// Adds an error response to the queue.
    pub fn with_error(self, error: MockError) -> Self {
        let mut responses = self.responses.lock().unwrap();
        responses.push_back(MockResponse::Error(error));
        drop(responses);
        self
    }

    /// Sets simulated latency per request.
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Sets the provider info.
    pub fn with_provider_info(mut self, info: ProviderInfo) -> Self {
        self.info = info;
        self
    }

    /// Returns the number of calls made to this provider.
    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    /// Returns all recorded calls.
    pub fn get_calls(&self) -> Vec<CompletionRequest> {
        self.calls.lock().unwrap().clone()
    }

    /// Clears the call history.
    pub fn clear_calls(&self) {
        self.calls.lock().unwrap().clear();
    }

    /// Gets the next response or a default.
    fn next_response(&self) -> MockResponse {
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| MockResponse::Success {
                content: "Mock response".to_string(),
                usage: TokenUsage::new(5, 10, 1),
                finish_reason: FinishReason::Stop,
            })
    }
}

#[async_trait]
impl AIProvider for MockAIProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AIError> {
        // Record the call
        self.calls.lock().unwrap().push(request);

        // Simulate delay
        if !self.delay.is_zero() {
            sleep(self.delay).await;
        }

        // Get configured response
        match self.next_response() {
            MockResponse::Success {
                content,
                usage,
                finish_reason,
            } => Ok(CompletionResponse {
                content,
                usage,
                model: self.info.model.clone(),
                finish_reason,
            }),
            MockResponse::Error(err) => Err(err.into()),
        }
    }

    async fn stream_complete(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, AIError>> + Send>>, AIError> {
        // Record the call
        self.calls.lock().unwrap().push(request);

        // Simulate initial delay
        if !self.delay.is_zero() {
            sleep(self.delay).await;
        }

        // Get configured response
        let response = self.next_response();
        let delay = self.delay;

        match response {
            MockResponse::Success {
                content,
                usage,
                finish_reason,
            } => {
                // Split content into word chunks for streaming simulation
                let word_chunks: Vec<Result<StreamChunk, AIError>> = content
                    .split_whitespace()
                    .map(|s| Ok(StreamChunk::content(format!("{} ", s))))
                    .collect();

                let chunks = stream::iter(word_chunks);

                // Add final chunk with usage
                let final_chunk = stream::once(async move {
                    if !delay.is_zero() {
                        sleep(delay / 10).await;
                    }
                    Ok(StreamChunk::final_chunk(finish_reason, usage))
                });

                let combined = chunks.chain(final_chunk);
                Ok(Box::pin(combined))
            }
            MockResponse::Error(err) => Err(err.into()),
        }
    }

    fn estimate_tokens(&self, text: &str) -> u32 {
        // Rough approximation: ~4 characters per token
        (text.len() / 4).max(1) as u32
    }

    fn provider_info(&self) -> ProviderInfo {
        self.info.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{ConversationId, SessionId, UserId};
    use crate::ports::RequestMetadata;
    use futures::StreamExt;

    fn test_metadata() -> RequestMetadata {
        RequestMetadata::new(
            UserId::new("test-user").unwrap(),
            SessionId::new(),
            ConversationId::new(),
            "trace-123",
        )
    }

    fn test_request() -> CompletionRequest {
        CompletionRequest::new(test_metadata())
            .with_message(crate::ports::MessageRole::User, "Hello")
    }

    #[tokio::test]
    async fn mock_provider_returns_configured_response() {
        let provider = MockAIProvider::new().with_response("Hello from mock!");

        let response = provider.complete(test_request()).await.unwrap();

        assert_eq!(response.content, "Hello from mock!");
        assert_eq!(response.model, "mock-model-1");
        assert_eq!(response.finish_reason, FinishReason::Stop);
    }

    #[tokio::test]
    async fn mock_provider_returns_responses_in_order() {
        let provider = MockAIProvider::new()
            .with_response("First")
            .with_response("Second")
            .with_response("Third");

        let r1 = provider.complete(test_request()).await.unwrap();
        let r2 = provider.complete(test_request()).await.unwrap();
        let r3 = provider.complete(test_request()).await.unwrap();

        assert_eq!(r1.content, "First");
        assert_eq!(r2.content, "Second");
        assert_eq!(r3.content, "Third");
    }

    #[tokio::test]
    async fn mock_provider_returns_default_after_exhausted() {
        let provider = MockAIProvider::new().with_response("Only one");

        let r1 = provider.complete(test_request()).await.unwrap();
        let r2 = provider.complete(test_request()).await.unwrap();

        assert_eq!(r1.content, "Only one");
        assert_eq!(r2.content, "Mock response"); // Default
    }

    #[tokio::test]
    async fn mock_provider_returns_configured_error() {
        let provider = MockAIProvider::new()
            .with_error(MockError::RateLimited { retry_after_secs: 30 });

        let result = provider.complete(test_request()).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.is_retryable());
        assert!(matches!(err, AIError::RateLimited { retry_after_secs: 30 }));
    }

    #[tokio::test]
    async fn mock_provider_tracks_calls() {
        let provider = MockAIProvider::new()
            .with_response("Response 1")
            .with_response("Response 2");

        assert_eq!(provider.call_count(), 0);

        provider.complete(test_request()).await.unwrap();
        assert_eq!(provider.call_count(), 1);

        provider.complete(test_request()).await.unwrap();
        assert_eq!(provider.call_count(), 2);

        provider.clear_calls();
        assert_eq!(provider.call_count(), 0);
    }

    #[tokio::test]
    async fn mock_provider_streaming_returns_chunks() {
        let provider = MockAIProvider::new()
            .with_response("Hello world from streaming");

        let mut stream = provider.stream_complete(test_request()).await.unwrap();

        let mut content = String::new();
        let mut final_chunk = None;

        while let Some(result) = stream.next().await {
            let chunk = result.unwrap();
            if chunk.is_final() {
                final_chunk = Some(chunk);
            } else {
                content.push_str(&chunk.delta);
            }
        }

        assert!(!content.is_empty());
        assert!(final_chunk.is_some());
        assert_eq!(final_chunk.unwrap().finish_reason, Some(FinishReason::Stop));
    }

    #[tokio::test]
    async fn mock_provider_streaming_returns_error() {
        let provider = MockAIProvider::new()
            .with_error(MockError::Unavailable { message: "Service down".to_string() });

        let result = provider.stream_complete(test_request()).await;

        match result {
            Ok(_) => panic!("Expected error, got stream"),
            Err(err) => assert!(matches!(err, AIError::Unavailable { .. })),
        }
    }

    #[tokio::test]
    async fn mock_provider_estimates_tokens() {
        let provider = MockAIProvider::new();

        // ~4 chars per token
        assert_eq!(provider.estimate_tokens("Hi"), 1);
        assert_eq!(provider.estimate_tokens("Hello world"), 2); // 11 chars / 4 = 2
        assert_eq!(provider.estimate_tokens("This is a longer sentence."), 6); // 26 chars / 4 = 6
    }

    #[tokio::test]
    async fn mock_provider_returns_info() {
        let custom_info = ProviderInfo::new("custom", "custom-model", 32000)
            .with_streaming(false)
            .with_functions(true);

        let provider = MockAIProvider::new().with_provider_info(custom_info.clone());

        let info = provider.provider_info();
        assert_eq!(info.name, "custom");
        assert_eq!(info.model, "custom-model");
        assert_eq!(info.max_context_tokens, 32000);
        assert!(!info.supports_streaming);
        assert!(info.supports_functions);
    }

    #[tokio::test]
    async fn mock_provider_respects_delay() {
        let provider = MockAIProvider::new()
            .with_response("Delayed response")
            .with_delay(Duration::from_millis(50));

        let start = std::time::Instant::now();
        provider.complete(test_request()).await.unwrap();
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(50));
    }

    #[test]
    fn mock_error_converts_to_ai_error() {
        let err: AIError = MockError::RateLimited { retry_after_secs: 10 }.into();
        assert!(matches!(err, AIError::RateLimited { retry_after_secs: 10 }));

        let err: AIError = MockError::ContextTooLong { tokens: 100, max: 50 }.into();
        assert!(matches!(err, AIError::ContextTooLong { tokens: 100, max: 50 }));

        let err: AIError = MockError::AuthenticationFailed.into();
        assert!(matches!(err, AIError::AuthenticationFailed));

        let err: AIError = MockError::Timeout { timeout_secs: 30 }.into();
        assert!(matches!(err, AIError::Timeout { timeout_secs: 30 }));
    }
}
