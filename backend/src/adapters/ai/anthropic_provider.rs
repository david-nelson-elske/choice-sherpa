//! Anthropic Provider - Implementation of AIProvider for Anthropic's Claude API.
//!
//! Supports Claude 3 models (Opus, Sonnet, Haiku) with streaming completions via SSE.
//!
//! # Configuration
//!
//! ```ignore
//! let config = AnthropicConfig::new(api_key)
//!     .with_model("claude-sonnet-4-20250514")
//!     .with_base_url("https://api.anthropic.com");
//!
//! let provider = AnthropicProvider::new(config);
//! ```
//!
//! # Streaming
//!
//! Uses Server-Sent Events (SSE) with Anthropic's event format. Events include
//! `message_start`, `content_block_delta`, and `message_delta` for streaming.

use async_trait::async_trait;
use futures::stream::{self, Stream, StreamExt};
use reqwest::{Client, Response};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;
use tokio::time::sleep;

use crate::ports::{
    AIError, AIProvider, CompletionRequest, CompletionResponse, FinishReason, ProviderInfo,
    StreamChunk, TokenUsage,
};

/// Configuration for the Anthropic provider.
#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    /// API key for authentication.
    api_key: Secret<String>,
    /// Model to use (e.g., "claude-sonnet-4-20250514", "claude-3-opus-20240229").
    pub model: String,
    /// Base URL for the API (default: https://api.anthropic.com).
    pub base_url: String,
    /// Request timeout.
    pub timeout: Duration,
    /// Maximum retries on transient failures.
    pub max_retries: u32,
}

impl AnthropicConfig {
    /// Creates a new configuration with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Secret::new(api_key.into()),
            model: "claude-sonnet-4-20250514".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            timeout: Duration::from_secs(60),
            max_retries: 3,
        }
    }

    /// Sets the model to use.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Sets the base URL.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Sets the request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the maximum retry count.
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Exposes the API key (for making requests).
    fn api_key(&self) -> &str {
        self.api_key.expose_secret()
    }
}

/// Anthropic API version header value.
const ANTHROPIC_API_VERSION: &str = "2023-06-01";

/// Anthropic API provider implementation.
pub struct AnthropicProvider {
    config: AnthropicConfig,
    client: Client,
}

impl AnthropicProvider {
    /// Creates a new Anthropic provider with the given configuration.
    pub fn new(config: AnthropicConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Builds the messages endpoint URL.
    fn messages_url(&self) -> String {
        format!("{}/v1/messages", self.config.base_url)
    }

    /// Converts our request to Anthropic's format.
    fn to_anthropic_request(&self, request: &CompletionRequest, stream: bool) -> AnthropicRequest {
        let mut messages = Vec::new();

        // Convert conversation messages (Anthropic doesn't use system role in messages)
        for msg in &request.messages {
            let role = match msg.role {
                crate::ports::MessageRole::System => continue, // System handled separately
                crate::ports::MessageRole::User => "user",
                crate::ports::MessageRole::Assistant => "assistant",
            };
            messages.push(AnthropicMessage {
                role: role.to_string(),
                content: msg.content.clone(),
            });
        }

        // Ensure messages alternate user/assistant (Anthropic requirement)
        // If first message isn't from user, we may need to adjust
        if messages.is_empty() {
            messages.push(AnthropicMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            });
        }

        AnthropicRequest {
            model: self.config.model.clone(),
            messages,
            system: request.system_prompt.clone(),
            max_tokens: request.max_tokens.unwrap_or(4096),
            temperature: request.temperature,
            stream: Some(stream),
        }
    }

    /// Sends a request and handles the response.
    async fn send_request(&self, request: &CompletionRequest) -> Result<Response, AIError> {
        let anthropic_request = self.to_anthropic_request(request, false);

        self.client
            .post(self.messages_url())
            .header("x-api-key", self.config.api_key())
            .header("anthropic-version", ANTHROPIC_API_VERSION)
            .header("Content-Type", "application/json")
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIError::Timeout {
                        timeout_secs: self.config.timeout.as_secs() as u32,
                    }
                } else if e.is_connect() {
                    AIError::network(format!("Connection failed: {}", e))
                } else {
                    AIError::network(e.to_string())
                }
            })
    }

    /// Sends a streaming request.
    async fn send_streaming_request(
        &self,
        request: &CompletionRequest,
    ) -> Result<Response, AIError> {
        let anthropic_request = self.to_anthropic_request(request, true);

        self.client
            .post(self.messages_url())
            .header("x-api-key", self.config.api_key())
            .header("anthropic-version", ANTHROPIC_API_VERSION)
            .header("Content-Type", "application/json")
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIError::Timeout {
                        timeout_secs: self.config.timeout.as_secs() as u32,
                    }
                } else if e.is_connect() {
                    AIError::network(format!("Connection failed: {}", e))
                } else {
                    AIError::network(e.to_string())
                }
            })
    }

    /// Parses the API response status and handles errors.
    async fn handle_response_status(&self, response: Response) -> Result<Response, AIError> {
        let status = response.status();

        if status.is_success() {
            return Ok(response);
        }

        // Try to parse error body
        let error_body = response.text().await.unwrap_or_default();

        match status.as_u16() {
            401 => Err(AIError::AuthenticationFailed),
            429 => {
                // Anthropic includes retry-after header
                let retry_after = Self::parse_retry_after(&error_body);
                Err(AIError::rate_limited(retry_after))
            }
            400 => {
                // Check for context length error
                if error_body.contains("prompt is too long") || error_body.contains("max_tokens") {
                    Err(AIError::context_too_long(0, 0))
                } else {
                    Err(AIError::InvalidRequest(error_body))
                }
            }
            500..=599 => Err(AIError::unavailable(format!(
                "Server error {}: {}",
                status, error_body
            ))),
            _ => Err(AIError::network(format!(
                "Unexpected status {}: {}",
                status, error_body
            ))),
        }
    }

    /// Parses retry-after from error response.
    fn parse_retry_after(error_body: &str) -> u32 {
        // Anthropic typically includes retry info in error message
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(error_body) {
            if let Some(msg) = parsed.get("error").and_then(|e| e.get("message")) {
                if let Some(s) = msg.as_str() {
                    // Try to find "try again in Xs" or similar patterns
                    if let Some(idx) = s.find("try again in ") {
                        let rest = &s[idx + 13..];
                        if let Some(num_end) = rest.find(|c: char| !c.is_ascii_digit()) {
                            if let Ok(secs) = rest[..num_end].parse::<u32>() {
                                return secs;
                            }
                        }
                    }
                }
            }
        }
        60 // Default retry after (Anthropic tends to have longer rate limit windows)
    }

    /// Parses a non-streaming response.
    async fn parse_response(&self, response: Response) -> Result<CompletionResponse, AIError> {
        let response = self.handle_response_status(response).await?;

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| AIError::parse(format!("Failed to parse response: {}", e)))?;

        let content = anthropic_response
            .content
            .into_iter()
            .filter_map(|block| {
                if block.block_type == "text" {
                    block.text
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");

        let finish_reason = match anthropic_response.stop_reason.as_deref() {
            Some("end_turn") | Some("stop_sequence") => FinishReason::Stop,
            Some("max_tokens") => FinishReason::Length,
            _ => FinishReason::Stop,
        };

        let usage = TokenUsage::new(
            anthropic_response.usage.input_tokens,
            anthropic_response.usage.output_tokens,
            self.calculate_cost(
                anthropic_response.usage.input_tokens,
                anthropic_response.usage.output_tokens,
            ),
        );

        Ok(CompletionResponse {
            content,
            usage,
            model: anthropic_response.model,
            finish_reason,
        })
    }

    /// Calculates estimated cost in cents based on model and token counts.
    fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> u32 {
        // Prices per 1M tokens as of 2024 (in cents)
        let (input_price, output_price) = match self.config.model.as_str() {
            m if m.contains("opus") => (1500, 7500), // $15/$75 per 1M
            m if m.contains("sonnet") => (300, 1500), // $3/$15 per 1M
            m if m.contains("haiku") => (25, 125),   // $0.25/$1.25 per 1M
            _ => (300, 1500),                        // Default to Sonnet pricing
        };

        // Calculate cost in cents (divide by 1M for per-token rate)
        let input_cost = (input_tokens as u64 * input_price) / 1_000_000;
        let output_cost = (output_tokens as u64 * output_price) / 1_000_000;

        (input_cost + output_cost) as u32
    }
}

#[async_trait]
impl AIProvider for AnthropicProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AIError> {
        let mut last_error = AIError::network("No attempts made");
        let mut retry_count = 0;

        while retry_count <= self.config.max_retries {
            match self.send_request(&request).await {
                Ok(response) => {
                    match self.parse_response(response).await {
                        Ok(completion) => return Ok(completion),
                        Err(err) => {
                            if !err.is_retryable() || retry_count >= self.config.max_retries {
                                return Err(err);
                            }
                            last_error = err;
                        }
                    }
                }
                Err(err) => {
                    if !err.is_retryable() || retry_count >= self.config.max_retries {
                        return Err(err);
                    }
                    last_error = err;
                }
            }

            // Exponential backoff: 1s, 2s, 4s, ...
            let delay = Duration::from_secs(1 << retry_count);
            sleep(delay).await;
            retry_count += 1;
        }

        Err(last_error)
    }

    async fn stream_complete(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, AIError>> + Send>>, AIError> {
        let response = self.send_streaming_request(&request).await?;
        let response = self.handle_response_status(response).await?;

        // Get the byte stream and parse SSE
        let bytes_stream = response.bytes_stream();
        let model = self.config.model.clone();

        // Pricing factors per 1M tokens
        let input_price_factor = match model.as_str() {
            m if m.contains("opus") => 1500,
            m if m.contains("sonnet") => 300,
            m if m.contains("haiku") => 25,
            _ => 300,
        };
        let output_price_factor = match model.as_str() {
            m if m.contains("opus") => 7500,
            m if m.contains("sonnet") => 1500,
            m if m.contains("haiku") => 125,
            _ => 1500,
        };

        // Parse Anthropic SSE stream
        let stream = bytes_stream
            .map(move |chunk_result| {
                chunk_result.map_err(|e| AIError::network(format!("Stream error: {}", e)))
            })
            .map(move |chunk_result| match chunk_result {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    parse_anthropic_sse(&text, input_price_factor, output_price_factor)
                }
                Err(e) => vec![Err(e)],
            })
            .flat_map(stream::iter);

        Ok(Box::pin(stream))
    }

    fn estimate_tokens(&self, text: &str) -> u32 {
        // Claude models use ~4 characters per token on average
        // This is a rough estimate
        (text.len() / 4).max(1) as u32
    }

    fn provider_info(&self) -> ProviderInfo {
        let max_context = match self.config.model.as_str() {
            m if m.contains("opus") => 200_000,
            m if m.contains("sonnet") => 200_000,
            m if m.contains("haiku") => 200_000,
            _ => 200_000, // All Claude 3 models have 200k context
        };

        ProviderInfo::new("anthropic", &self.config.model, max_context)
            .with_streaming(true)
            .with_functions(true)
    }
}

/// Parses Anthropic SSE format into StreamChunks.
///
/// Anthropic SSE format uses `event:` and `data:` lines:
/// ```text
/// event: content_block_delta
/// data: {"type":"content_block_delta","delta":{"text":"Hello"}}
/// ```
fn parse_anthropic_sse(
    text: &str,
    input_price_factor: u64,
    output_price_factor: u64,
) -> Vec<Result<StreamChunk, AIError>> {
    let mut results = Vec::new();
    let mut current_event = String::new();

    for line in text.lines() {
        if let Some(event_type) = line.strip_prefix("event: ") {
            current_event = event_type.to_string();
        } else if let Some(data) = line.strip_prefix("data: ") {
            match current_event.as_str() {
                "content_block_delta" => {
                    if let Ok(delta) = serde_json::from_str::<ContentBlockDelta>(data) {
                        if let Some(text) = delta.delta.text {
                            if !text.is_empty() {
                                results.push(Ok(StreamChunk::content(&text)));
                            }
                        }
                    }
                }
                "message_delta" => {
                    if let Ok(delta) = serde_json::from_str::<MessageDelta>(data) {
                        let finish_reason = match delta.delta.stop_reason.as_deref() {
                            Some("end_turn") | Some("stop_sequence") => FinishReason::Stop,
                            Some("max_tokens") => FinishReason::Length,
                            _ => FinishReason::Stop,
                        };

                        let usage = delta.usage.map(|u| {
                            let input_cost =
                                (u.input_tokens.unwrap_or(0) as u64 * input_price_factor)
                                    / 1_000_000;
                            let output_cost =
                                (u.output_tokens as u64 * output_price_factor) / 1_000_000;
                            TokenUsage::new(
                                u.input_tokens.unwrap_or(0),
                                u.output_tokens,
                                (input_cost + output_cost) as u32,
                            )
                        }).unwrap_or_default();

                        results.push(Ok(StreamChunk::final_chunk(finish_reason, usage)));
                    }
                }
                "message_stop" => {
                    // Stream complete marker - usage should have come in message_delta
                }
                "error" => {
                    if let Ok(error) = serde_json::from_str::<StreamError>(data) {
                        results.push(Err(AIError::unavailable(
                            error.error.message.unwrap_or_else(|| "Stream error".to_string()),
                        )));
                    }
                }
                _ => {
                    // Ignore other event types (message_start, content_block_start, etc.)
                }
            }
        }
    }

    results
}

// ----- Anthropic API Types -----

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    model: String,
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

// Streaming response types
#[derive(Debug, Deserialize)]
struct ContentBlockDelta {
    delta: TextDelta,
}

#[derive(Debug, Deserialize)]
struct TextDelta {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    delta_type: Option<String>,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MessageDelta {
    delta: MessageDeltaContent,
    usage: Option<StreamUsage>,
}

#[derive(Debug, Deserialize)]
struct MessageDeltaContent {
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamUsage {
    input_tokens: Option<u32>,
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct StreamError {
    error: StreamErrorContent,
}

#[derive(Debug, Deserialize)]
struct StreamErrorContent {
    message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_builder_works() {
        let config = AnthropicConfig::new("test-key")
            .with_model("claude-3-opus-20240229")
            .with_base_url("https://custom.api.com")
            .with_timeout(Duration::from_secs(30))
            .with_max_retries(5);

        assert_eq!(config.model, "claude-3-opus-20240229");
        assert_eq!(config.base_url, "https://custom.api.com");
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.api_key(), "test-key");
    }

    #[test]
    fn cost_calculation_opus() {
        let config = AnthropicConfig::new("test").with_model("claude-3-opus-20240229");
        let provider = AnthropicProvider::new(config);

        // 1M input tokens = $15 = 1500 cents
        // 1M output tokens = $75 = 7500 cents
        // 100,000 tokens = 150 cents input + 750 cents output = 900 cents
        let cost = provider.calculate_cost(100_000, 100_000);
        assert_eq!(cost, 900);
    }

    #[test]
    fn cost_calculation_sonnet() {
        let config = AnthropicConfig::new("test").with_model("claude-sonnet-4-20250514");
        let provider = AnthropicProvider::new(config);

        // 1M input tokens = $3 = 300 cents
        // 1M output tokens = $15 = 1500 cents
        // 100,000 tokens = 30 cents input + 150 cents output = 180 cents
        let cost = provider.calculate_cost(100_000, 100_000);
        assert_eq!(cost, 180);
    }

    #[test]
    fn cost_calculation_haiku() {
        let config = AnthropicConfig::new("test").with_model("claude-3-haiku-20240307");
        let provider = AnthropicProvider::new(config);

        // 1M input tokens = $0.25 = 25 cents
        // 1M output tokens = $1.25 = 125 cents
        // 1M tokens each = 25 + 125 = 150 cents
        let cost = provider.calculate_cost(1_000_000, 1_000_000);
        assert_eq!(cost, 150);
    }

    #[test]
    fn provider_info_opus() {
        let config = AnthropicConfig::new("test").with_model("claude-3-opus-20240229");
        let provider = AnthropicProvider::new(config);

        let info = provider.provider_info();
        assert_eq!(info.name, "anthropic");
        assert_eq!(info.model, "claude-3-opus-20240229");
        assert_eq!(info.max_context_tokens, 200_000);
        assert!(info.supports_streaming);
        assert!(info.supports_functions);
    }

    #[test]
    fn estimate_tokens_approximates() {
        let config = AnthropicConfig::new("test");
        let provider = AnthropicProvider::new(config);

        // ~4 chars per token
        assert_eq!(provider.estimate_tokens("Hi"), 1);
        assert_eq!(provider.estimate_tokens("Hello, world!"), 3); // 13 chars / 4 = 3
    }

    #[test]
    fn parse_sse_content_delta() {
        let data = "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}";
        let chunks = parse_anthropic_sse(data, 300, 1500);

        assert_eq!(chunks.len(), 1);
        let chunk = chunks[0].as_ref().unwrap();
        assert_eq!(chunk.delta, "Hello");
        assert!(!chunk.is_final());
    }

    #[test]
    fn parse_sse_message_delta_with_stop() {
        let data = "event: message_delta\ndata: {\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":50}}";
        let chunks = parse_anthropic_sse(data, 300, 1500);

        assert_eq!(chunks.len(), 1);
        let chunk = chunks[0].as_ref().unwrap();
        assert!(chunk.is_final());
        assert_eq!(chunk.finish_reason, Some(FinishReason::Stop));
    }

    #[test]
    fn parse_sse_multiple_events() {
        let data = "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"Hi\"}}\n\nevent: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\" there\"}}";
        let chunks = parse_anthropic_sse(data, 300, 1500);

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].as_ref().unwrap().delta, "Hi");
        assert_eq!(chunks[1].as_ref().unwrap().delta, " there");
    }

    #[test]
    fn parse_retry_after_default() {
        let error = r#"{"error":{"message":"Rate limit exceeded"}}"#;
        let retry = AnthropicProvider::parse_retry_after(error);
        assert_eq!(retry, 60); // Default for Anthropic
    }
}
