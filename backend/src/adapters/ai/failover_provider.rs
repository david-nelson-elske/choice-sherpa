//! Failover AI Provider - Wrapper that provides automatic failover between providers.
//!
//! When the primary provider fails with a transient error (rate limit, unavailable),
//! automatically falls back to the secondary provider if configured.
//!
//! # Example
//!
//! ```ignore
//! let primary = OpenAIProvider::new(openai_config);
//! let fallback = AnthropicProvider::new(anthropic_config);
//!
//! let provider = FailoverAIProvider::new(primary)
//!     .with_fallback(fallback);
//! ```

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use std::sync::Arc;

use crate::ports::{
    AIError, AIProvider, CompletionRequest, CompletionResponse, ProviderInfo, StreamChunk,
};

/// AI domain events for cost tracking and failover monitoring.
pub mod events {
    use serde::{Deserialize, Serialize};

    use crate::domain::foundation::{domain_event, ComponentType, EventId, SessionId, Timestamp, UserId};

    /// Emitted when AI tokens are used for a completion.
    ///
    /// This event enables cost attribution by carrying user and session context
    /// alongside provider/model/token information.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AITokensUsed {
        pub event_id: EventId,
        /// User who incurred the cost.
        pub user_id: UserId,
        /// Session context for the request.
        pub session_id: SessionId,
        pub provider: String,
        pub model: String,
        pub prompt_tokens: u32,
        pub completion_tokens: u32,
        pub estimated_cost_cents: u32,
        /// PrOACT component type for analytics (optional).
        pub component_type: Option<ComponentType>,
        pub request_id: String,
        pub occurred_at: Timestamp,
    }

    impl AITokensUsed {
        /// Creates a new AITokensUsed event with full user context.
        #[allow(clippy::too_many_arguments)]
        pub fn new(
            user_id: UserId,
            session_id: SessionId,
            provider: impl Into<String>,
            model: impl Into<String>,
            prompt_tokens: u32,
            completion_tokens: u32,
            estimated_cost_cents: u32,
            component_type: Option<ComponentType>,
            request_id: impl Into<String>,
        ) -> Self {
            Self {
                event_id: EventId::new(),
                user_id,
                session_id,
                provider: provider.into(),
                model: model.into(),
                prompt_tokens,
                completion_tokens,
                estimated_cost_cents,
                component_type,
                request_id: request_id.into(),
                occurred_at: Timestamp::now(),
            }
        }

        /// Total tokens used in this request.
        pub fn total_tokens(&self) -> u32 {
            self.prompt_tokens + self.completion_tokens
        }
    }

    domain_event!(
        AITokensUsed,
        event_type = "ai.tokens_used",
        aggregate_id = request_id,
        aggregate_type = "AIRequest",
        occurred_at = occurred_at,
        event_id = event_id
    );

    /// Emitted when a provider failover occurs.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ProviderFallback {
        pub event_id: EventId,
        pub primary_provider: String,
        pub fallback_provider: String,
        pub reason: String,
        pub request_id: String,
        pub occurred_at: Timestamp,
    }

    impl ProviderFallback {
        /// Creates a new ProviderFallback event.
        pub fn new(
            primary: impl Into<String>,
            fallback: impl Into<String>,
            reason: impl Into<String>,
            request_id: impl Into<String>,
        ) -> Self {
            Self {
                event_id: EventId::new(),
                primary_provider: primary.into(),
                fallback_provider: fallback.into(),
                reason: reason.into(),
                request_id: request_id.into(),
                occurred_at: Timestamp::now(),
            }
        }
    }

    domain_event!(
        ProviderFallback,
        event_type = "ai.provider_fallback",
        aggregate_id = request_id,
        aggregate_type = "AIRequest",
        occurred_at = occurred_at,
        event_id = event_id
    );
}

/// Callback for receiving AI events (tokens used, failover).
pub trait AIEventCallback: Send + Sync {
    /// Called when tokens are used.
    fn on_tokens_used(&self, event: events::AITokensUsed);

    /// Called when a provider failover occurs.
    fn on_fallback(&self, event: events::ProviderFallback);
}

/// No-op event callback for when event tracking isn't needed.
#[derive(Debug, Clone, Copy)]
pub struct NoOpEventCallback;

impl AIEventCallback for NoOpEventCallback {
    fn on_tokens_used(&self, _event: events::AITokensUsed) {}
    fn on_fallback(&self, _event: events::ProviderFallback) {}
}

/// AI provider wrapper with automatic failover support.
///
/// Wraps a primary provider and optionally a fallback provider.
/// On transient failures (rate limiting, unavailable), automatically
/// tries the fallback provider.
pub struct FailoverAIProvider<P: AIProvider, F: AIProvider = NoFallback> {
    primary: P,
    fallback: Option<F>,
    event_callback: Arc<dyn AIEventCallback>,
}

/// Marker type for when no fallback is configured.
pub struct NoFallback;

#[async_trait]
impl AIProvider for NoFallback {
    async fn complete(&self, _: CompletionRequest) -> Result<CompletionResponse, AIError> {
        unreachable!("NoFallback should never be called")
    }

    async fn stream_complete(
        &self,
        _: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, AIError>> + Send>>, AIError> {
        unreachable!("NoFallback should never be called")
    }

    fn estimate_tokens(&self, _: &str) -> u32 {
        0
    }

    fn provider_info(&self) -> ProviderInfo {
        ProviderInfo::new("none", "none", 0)
    }
}

impl<P: AIProvider> FailoverAIProvider<P, NoFallback> {
    /// Creates a new failover provider with only a primary provider.
    pub fn new(primary: P) -> Self {
        Self {
            primary,
            fallback: None,
            event_callback: Arc::new(NoOpEventCallback),
        }
    }

    /// Adds a fallback provider.
    pub fn with_fallback<F: AIProvider>(self, fallback: F) -> FailoverAIProvider<P, F> {
        FailoverAIProvider {
            primary: self.primary,
            fallback: Some(fallback),
            event_callback: self.event_callback,
        }
    }
}

impl<P: AIProvider, F: AIProvider> FailoverAIProvider<P, F> {
    /// Sets the event callback for receiving AI events.
    pub fn with_event_callback(mut self, callback: Arc<dyn AIEventCallback>) -> Self {
        self.event_callback = callback;
        self
    }

    /// Emits a tokens used event with full user context.
    fn emit_tokens_used(
        &self,
        request: &CompletionRequest,
        response: &CompletionResponse,
        request_id: &str,
    ) {
        let info = self.primary.provider_info();
        let event = events::AITokensUsed::new(
            request.metadata.user_id.clone(),
            request.metadata.session_id.clone(),
            info.name,
            &response.model,
            response.usage.prompt_tokens,
            response.usage.completion_tokens,
            response.usage.estimated_cost_cents,
            request.component_type,
            request_id,
        );
        self.event_callback.on_tokens_used(event);
    }

    /// Emits a fallback event.
    fn emit_fallback(&self, reason: &str, request_id: &str) {
        if let Some(ref fallback) = self.fallback {
            let primary_info = self.primary.provider_info();
            let fallback_info = fallback.provider_info();
            let event = events::ProviderFallback::new(
                primary_info.name,
                fallback_info.name,
                reason,
                request_id,
            );
            self.event_callback.on_fallback(event);
        }
    }
}

#[async_trait]
impl<P: AIProvider + 'static, F: AIProvider + 'static> AIProvider for FailoverAIProvider<P, F> {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AIError> {
        let request_id = uuid::Uuid::new_v4().to_string();

        // Try primary provider
        match self.primary.complete(request.clone()).await {
            Ok(response) => {
                self.emit_tokens_used(&request, &response, &request_id);
                Ok(response)
            }
            Err(err) if err.is_retryable() && self.fallback.is_some() => {
                // Emit failover event
                self.emit_fallback(&err.to_string(), &request_id);

                // Try fallback
                let fallback = self.fallback.as_ref().unwrap();
                let response = fallback.complete(request.clone()).await?;
                self.emit_tokens_used(&request, &response, &request_id);
                Ok(response)
            }
            Err(err) => Err(err),
        }
    }

    async fn stream_complete(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, AIError>> + Send>>, AIError> {
        let request_id = uuid::Uuid::new_v4().to_string();

        // Try primary provider
        match self.primary.stream_complete(request.clone()).await {
            Ok(stream) => {
                // Note: For streaming, we can't easily emit tokens_used since
                // usage comes at the end of the stream. The caller should handle this.
                Ok(stream)
            }
            Err(err) if err.is_retryable() && self.fallback.is_some() => {
                // Emit failover event
                self.emit_fallback(&err.to_string(), &request_id);

                // Try fallback
                let fallback = self.fallback.as_ref().unwrap();
                fallback.stream_complete(request).await
            }
            Err(err) => Err(err),
        }
    }

    fn estimate_tokens(&self, text: &str) -> u32 {
        // Use primary provider's estimation
        self.primary.estimate_tokens(text)
    }

    fn provider_info(&self) -> ProviderInfo {
        // Report primary provider's info
        self.primary.provider_info()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::ai::MockAIProvider;
    use crate::adapters::ai::MockError;
    use crate::domain::foundation::{ConversationId, SessionId, UserId};
    use crate::ports::{CompletionRequest, MessageRole, RequestMetadata};
    use std::sync::atomic::{AtomicU32, Ordering};

    #[derive(Default)]
    struct TestEventCallback {
        tokens_used_count: AtomicU32,
        fallback_count: AtomicU32,
    }

    impl AIEventCallback for TestEventCallback {
        fn on_tokens_used(&self, _event: events::AITokensUsed) {
            self.tokens_used_count.fetch_add(1, Ordering::SeqCst);
        }

        fn on_fallback(&self, _event: events::ProviderFallback) {
            self.fallback_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn test_metadata() -> RequestMetadata {
        RequestMetadata::new(
            UserId::new("test-user").unwrap(),
            SessionId::new(),
            ConversationId::new(),
            "trace-123",
        )
    }

    fn make_request() -> CompletionRequest {
        CompletionRequest::new(test_metadata())
            .with_message(MessageRole::User, "Hello")
    }

    #[tokio::test]
    async fn primary_success_no_fallback_used() {
        let primary = MockAIProvider::new().with_response("Hi there!");
        let fallback = MockAIProvider::new().with_response("Fallback response");

        let callback = Arc::new(TestEventCallback::default());
        let provider = FailoverAIProvider::new(primary)
            .with_fallback(fallback)
            .with_event_callback(callback.clone());

        let response = provider.complete(make_request()).await.unwrap();

        assert_eq!(response.content, "Hi there!");
        assert_eq!(callback.tokens_used_count.load(Ordering::SeqCst), 1);
        assert_eq!(callback.fallback_count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn primary_rate_limited_uses_fallback() {
        let primary =
            MockAIProvider::new().with_error(MockError::RateLimited { retry_after_secs: 30 });
        let fallback = MockAIProvider::new().with_response("Fallback response");

        let callback = Arc::new(TestEventCallback::default());
        let provider = FailoverAIProvider::new(primary)
            .with_fallback(fallback)
            .with_event_callback(callback.clone());

        let response = provider.complete(make_request()).await.unwrap();

        assert_eq!(response.content, "Fallback response");
        assert_eq!(callback.tokens_used_count.load(Ordering::SeqCst), 1);
        assert_eq!(callback.fallback_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn primary_unavailable_uses_fallback() {
        let primary = MockAIProvider::new().with_error(MockError::Unavailable {
            message: "Service down".to_string(),
        });
        let fallback = MockAIProvider::new().with_response("Fallback response");

        let callback = Arc::new(TestEventCallback::default());
        let provider = FailoverAIProvider::new(primary)
            .with_fallback(fallback)
            .with_event_callback(callback.clone());

        let response = provider.complete(make_request()).await.unwrap();

        assert_eq!(response.content, "Fallback response");
        assert_eq!(callback.fallback_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn non_retryable_error_not_fallback() {
        let primary = MockAIProvider::new().with_error(MockError::AuthenticationFailed);
        let fallback = MockAIProvider::new().with_response("Fallback response");

        let callback = Arc::new(TestEventCallback::default());
        let provider = FailoverAIProvider::new(primary)
            .with_fallback(fallback)
            .with_event_callback(callback.clone());

        let result = provider.complete(make_request()).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AIError::AuthenticationFailed));
        assert_eq!(callback.fallback_count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn no_fallback_configured_returns_error() {
        let primary =
            MockAIProvider::new().with_error(MockError::RateLimited { retry_after_secs: 30 });

        let provider = FailoverAIProvider::new(primary);

        let result = provider.complete(make_request()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn fallback_also_fails_returns_fallback_error() {
        let primary =
            MockAIProvider::new().with_error(MockError::RateLimited { retry_after_secs: 30 });
        let fallback = MockAIProvider::new().with_error(MockError::AuthenticationFailed);

        let callback = Arc::new(TestEventCallback::default());
        let provider = FailoverAIProvider::new(primary)
            .with_fallback(fallback)
            .with_event_callback(callback.clone());

        let result = provider.complete(make_request()).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AIError::AuthenticationFailed));
        assert_eq!(callback.fallback_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn tokens_used_event_creates_correctly() {
        

        let user_id = UserId::new("user-test-123").unwrap();
        let session_id = SessionId::new();
        let event = events::AITokensUsed::new(
            user_id,
            session_id,
            "openai",
            "gpt-4",
            100,
            50,
            5,
            None, // component_type
            "req-123",
        );

        assert_eq!(event.provider, "openai");
        assert_eq!(event.model, "gpt-4");
        assert_eq!(event.prompt_tokens, 100);
        assert_eq!(event.completion_tokens, 50);
        assert_eq!(event.estimated_cost_cents, 5);
        assert_eq!(event.request_id, "req-123");
    }

    #[test]
    fn provider_fallback_event_creates_correctly() {
        let event = events::ProviderFallback::new("openai", "anthropic", "Rate limited", "req-456");

        assert_eq!(event.primary_provider, "openai");
        assert_eq!(event.fallback_provider, "anthropic");
        assert_eq!(event.reason, "Rate limited");
        assert_eq!(event.request_id, "req-456");
    }
}
