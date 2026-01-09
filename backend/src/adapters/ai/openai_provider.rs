//! OpenAI Provider - Implementation of AIProvider for OpenAI's API.
//!
//! Supports GPT-4 and GPT-3.5 models with streaming completions via SSE.
//!
//! # Configuration
//!
//! ```ignore
//! let config = OpenAIConfig::new(api_key)
//!     .with_model("gpt-4-turbo")
//!     .with_base_url("https://api.openai.com/v1");
//!
//! let provider = OpenAIProvider::new(config);
//! ```
//!
//! # Streaming
//!
//! Uses Server-Sent Events (SSE) for streaming responses. Each chunk is parsed
//! and yielded as a `StreamChunk` until the `[DONE]` marker is received.

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

/// Configuration for the OpenAI provider.
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    /// API key for authentication.
    api_key: Secret<String>,
    /// Model to use (e.g., "gpt-4-turbo", "gpt-3.5-turbo").
    pub model: String,
    /// Base URL for the API (default: https://api.openai.com/v1).
    pub base_url: String,
    /// Request timeout.
    pub timeout: Duration,
    /// Maximum retries on transient failures.
    pub max_retries: u32,
}

impl OpenAIConfig {
    /// Creates a new configuration with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Secret::new(api_key.into()),
            model: "gpt-4-turbo".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
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

/// OpenAI API provider implementation.
pub struct OpenAIProvider {
    config: OpenAIConfig,
    client: Client,
}

impl OpenAIProvider {
    /// Creates a new OpenAI provider with the given configuration.
    pub fn new(config: OpenAIConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Builds the chat completions endpoint URL.
    fn completions_url(&self) -> String {
        format!("{}/chat/completions", self.config.base_url)
    }

    /// Converts our request to OpenAI's format.
    fn to_openai_request(&self, request: &CompletionRequest, stream: bool) -> OpenAIRequest {
        let mut messages = Vec::new();

        // Add system prompt if present
        if let Some(ref prompt) = request.system_prompt {
            messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: prompt.clone(),
            });
        }

        // Add conversation messages
        for msg in &request.messages {
            messages.push(OpenAIMessage {
                role: match msg.role {
                    crate::ports::MessageRole::System => "system",
                    crate::ports::MessageRole::User => "user",
                    crate::ports::MessageRole::Assistant => "assistant",
                }
                .to_string(),
                content: msg.content.clone(),
            });
        }

        OpenAIRequest {
            model: self.config.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            stream: Some(stream),
            stream_options: if stream {
                Some(StreamOptions {
                    include_usage: true,
                })
            } else {
                None
            },
        }
    }

    /// Sends a request and handles the response.
    async fn send_request(&self, request: &CompletionRequest) -> Result<Response, AIError> {
        let openai_request = self.to_openai_request(request, false);

        self.client
            .post(self.completions_url())
            .header("Authorization", format!("Bearer {}", self.config.api_key()))
            .header("Content-Type", "application/json")
            .json(&openai_request)
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
        let openai_request = self.to_openai_request(request, true);

        self.client
            .post(self.completions_url())
            .header("Authorization", format!("Bearer {}", self.config.api_key()))
            .header("Content-Type", "application/json")
            .json(&openai_request)
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
                // Try to extract retry-after from error
                let retry_after = Self::parse_retry_after(&error_body);
                Err(AIError::rate_limited(retry_after))
            }
            400 => {
                // Check for context length error
                if error_body.contains("maximum context length")
                    || error_body.contains("context_length_exceeded")
                {
                    // Try to parse the numbers from the error
                    Err(AIError::context_too_long(0, 0)) // Simplified - real impl would parse
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
        // OpenAI includes retry-after in the error message sometimes
        // Default to 30 seconds if we can't parse
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(error_body) {
            if let Some(msg) = parsed.get("error").and_then(|e| e.get("message")) {
                if let Some(s) = msg.as_str() {
                    // Try to find "try again in Xs" pattern
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
        30 // Default retry after
    }

    /// Parses a non-streaming response.
    async fn parse_response(&self, response: Response) -> Result<CompletionResponse, AIError> {
        let response = self.handle_response_status(response).await?;

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| AIError::parse(format!("Failed to parse response: {}", e)))?;

        let choice = openai_response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| AIError::parse("No choices in response"))?;

        let finish_reason = match choice.finish_reason.as_deref() {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some("content_filter") => FinishReason::ContentFilter,
            _ => FinishReason::Stop,
        };

        let usage = openai_response.usage.map(|u| {
            TokenUsage::new(
                u.prompt_tokens,
                u.completion_tokens,
                self.calculate_cost(u.prompt_tokens, u.completion_tokens),
            )
        }).unwrap_or_default();

        Ok(CompletionResponse {
            content: choice.message.content,
            usage,
            model: openai_response.model,
            finish_reason,
        })
    }

    /// Calculates estimated cost in cents based on model and token counts.
    fn calculate_cost(&self, prompt_tokens: u32, completion_tokens: u32) -> u32 {
        // Prices per 1M tokens as of 2024 (in cents)
        let (prompt_price, completion_price) = match self.config.model.as_str() {
            m if m.starts_with("gpt-4-turbo") || m.starts_with("gpt-4-0125") => (1000, 3000), // $10/$30 per 1M
            m if m.starts_with("gpt-4-1106") => (1000, 3000),
            m if m.starts_with("gpt-4o") => (250, 1000), // $2.50/$10 per 1M
            m if m.starts_with("gpt-4") => (3000, 6000), // $30/$60 per 1M (base GPT-4)
            m if m.starts_with("gpt-3.5") => (50, 150),  // $0.50/$1.50 per 1M
            _ => (1000, 3000), // Default to GPT-4 turbo pricing
        };

        // Calculate cost in cents (divide by 1M for per-token rate)
        let prompt_cost = (prompt_tokens as u64 * prompt_price) / 1_000_000;
        let completion_cost = (completion_tokens as u64 * completion_price) / 1_000_000;

        (prompt_cost + completion_cost) as u32
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
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
        let prompt_price_factor = match model.as_str() {
            m if m.starts_with("gpt-4o") => 250,
            m if m.starts_with("gpt-4") => 1000,
            m if m.starts_with("gpt-3.5") => 50,
            _ => 1000,
        };
        let completion_price_factor = match model.as_str() {
            m if m.starts_with("gpt-4o") => 1000,
            m if m.starts_with("gpt-4") => 3000,
            m if m.starts_with("gpt-3.5") => 150,
            _ => 3000,
        };

        // Parse SSE stream
        let stream = bytes_stream
            .map(move |chunk_result| {
                chunk_result
                    .map_err(|e| AIError::network(format!("Stream error: {}", e)))
            })
            .map(move |chunk_result| {
                match chunk_result {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        parse_sse_chunks(&text, prompt_price_factor, completion_price_factor)
                    }
                    Err(e) => vec![Err(e)],
                }
            })
            .flat_map(stream::iter);

        Ok(Box::pin(stream))
    }

    fn estimate_tokens(&self, text: &str) -> u32 {
        // GPT models use ~4 characters per token on average
        // This is a rough estimate - for accuracy, use tiktoken
        (text.len() / 4).max(1) as u32
    }

    fn provider_info(&self) -> ProviderInfo {
        let max_context = match self.config.model.as_str() {
            m if m.contains("128k") || m.starts_with("gpt-4-turbo") || m.starts_with("gpt-4o") => {
                128000
            }
            m if m.starts_with("gpt-4-1106") => 128000,
            m if m.starts_with("gpt-4") => 8192,
            m if m.contains("16k") => 16384,
            m if m.starts_with("gpt-3.5") => 4096,
            _ => 128000,
        };

        ProviderInfo::new("openai", &self.config.model, max_context)
            .with_streaming(true)
            .with_functions(true)
    }
}

/// Parses SSE data chunks into StreamChunks.
fn parse_sse_chunks(
    text: &str,
    prompt_price_factor: u64,
    completion_price_factor: u64,
) -> Vec<Result<StreamChunk, AIError>> {
    let mut results = Vec::new();

    for line in text.lines() {
        if let Some(data) = line.strip_prefix("data: ") {

            if data == "[DONE]" {
                // Stream complete - we don't emit a chunk here as usage comes in last data chunk
                continue;
            }

            match serde_json::from_str::<StreamResponseChunk>(data) {
                Ok(chunk) => {
                    if let Some(choice) = chunk.choices.first() {
                        // Content delta
                        if let Some(ref content) = choice.delta.content {
                            if !content.is_empty() {
                                results.push(Ok(StreamChunk::content(content)));
                            }
                        }

                        // Check for finish reason
                        if let Some(ref reason) = choice.finish_reason {
                            let finish = match reason.as_str() {
                                "stop" => FinishReason::Stop,
                                "length" => FinishReason::Length,
                                "content_filter" => FinishReason::ContentFilter,
                                _ => FinishReason::Stop,
                            };

                            // Check if we have usage in this chunk
                            let usage = chunk.usage.map(|u| {
                                let prompt_cost =
                                    (u.prompt_tokens as u64 * prompt_price_factor) / 1_000_000;
                                let completion_cost =
                                    (u.completion_tokens as u64 * completion_price_factor)
                                        / 1_000_000;
                                TokenUsage::new(
                                    u.prompt_tokens,
                                    u.completion_tokens,
                                    (prompt_cost + completion_cost) as u32,
                                )
                            }).unwrap_or_default();

                            results.push(Ok(StreamChunk::final_chunk(finish, usage)));
                        }
                    }
                }
                Err(e) => {
                    // Only error on non-empty data that fails to parse
                    if !data.trim().is_empty() {
                        results.push(Err(AIError::parse(format!(
                            "Failed to parse SSE chunk: {}",
                            e
                        ))));
                    }
                }
            }
        }
    }

    results
}

// ----- OpenAI API Types -----

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<StreamOptions>,
}

#[derive(Debug, Serialize)]
struct StreamOptions {
    include_usage: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct StreamResponseChunk {
    choices: Vec<StreamChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_builder_works() {
        let config = OpenAIConfig::new("test-key")
            .with_model("gpt-4o")
            .with_base_url("https://custom.api.com")
            .with_timeout(Duration::from_secs(30))
            .with_max_retries(5);

        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.base_url, "https://custom.api.com");
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.api_key(), "test-key");
    }

    #[test]
    fn cost_calculation_gpt4_turbo() {
        let config = OpenAIConfig::new("test").with_model("gpt-4-turbo");
        let provider = OpenAIProvider::new(config);

        // 1M prompt tokens = $10 = 1000 cents
        // 1M completion tokens = $30 = 3000 cents
        // 1000 prompt + 500 completion = 1 cent + 1.5 cents = 2 cents (rounded)
        let cost = provider.calculate_cost(1000, 500);
        // With integer math: (1000 * 1000) / 1_000_000 = 1 cent for prompt
        // (500 * 3000) / 1_000_000 = 1 cent for completion
        assert_eq!(cost, 2);
    }

    #[test]
    fn cost_calculation_gpt35() {
        let config = OpenAIConfig::new("test").with_model("gpt-3.5-turbo");
        let provider = OpenAIProvider::new(config);

        // 1M prompt tokens = $0.50 = 50 cents
        // 1M completion tokens = $1.50 = 150 cents
        // 100,000 tokens = 5 cents prompt, 15 cents completion = 20 cents
        let cost = provider.calculate_cost(100_000, 100_000);
        assert_eq!(cost, 20);
    }

    #[test]
    fn provider_info_gpt4_turbo() {
        let config = OpenAIConfig::new("test").with_model("gpt-4-turbo-2024-04-09");
        let provider = OpenAIProvider::new(config);

        let info = provider.provider_info();
        assert_eq!(info.name, "openai");
        assert_eq!(info.model, "gpt-4-turbo-2024-04-09");
        assert_eq!(info.max_context_tokens, 128000);
        assert!(info.supports_streaming);
        assert!(info.supports_functions);
    }

    #[test]
    fn provider_info_gpt35() {
        let config = OpenAIConfig::new("test").with_model("gpt-3.5-turbo");
        let provider = OpenAIProvider::new(config);

        let info = provider.provider_info();
        assert_eq!(info.max_context_tokens, 4096);
    }

    #[test]
    fn estimate_tokens_approximates() {
        let config = OpenAIConfig::new("test");
        let provider = OpenAIProvider::new(config);

        // ~4 chars per token
        assert_eq!(provider.estimate_tokens("Hi"), 1);
        assert_eq!(provider.estimate_tokens("Hello, world!"), 3); // 13 chars / 4 = 3
    }

    #[test]
    fn parse_sse_content_chunk() {
        let data = r#"data: {"id":"chatcmpl-123","choices":[{"delta":{"content":"Hello"},"finish_reason":null}]}"#;
        let chunks = parse_sse_chunks(data, 1000, 3000);

        assert_eq!(chunks.len(), 1);
        let chunk = chunks[0].as_ref().unwrap();
        assert_eq!(chunk.delta, "Hello");
        assert!(!chunk.is_final());
    }

    #[test]
    fn parse_sse_final_chunk() {
        let data = r#"data: {"id":"chatcmpl-123","choices":[{"delta":{},"finish_reason":"stop"}],"usage":{"prompt_tokens":10,"completion_tokens":5}}"#;
        let chunks = parse_sse_chunks(data, 1000, 3000);

        assert_eq!(chunks.len(), 1);
        let chunk = chunks[0].as_ref().unwrap();
        assert!(chunk.is_final());
        assert_eq!(chunk.finish_reason, Some(FinishReason::Stop));
    }

    #[test]
    fn parse_sse_done_marker() {
        let data = "data: [DONE]\n";
        let chunks = parse_sse_chunks(data, 1000, 3000);

        // [DONE] doesn't produce a chunk
        assert!(chunks.is_empty());
    }

    #[test]
    fn parse_retry_after_from_message() {
        let error = r#"{"error":{"message":"Rate limit exceeded. Please try again in 30 seconds."}}"#;
        let retry = OpenAIProvider::parse_retry_after(error);
        assert_eq!(retry, 30);
    }

    #[test]
    fn parse_retry_after_default() {
        let error = r#"{"error":{"message":"Something went wrong"}}"#;
        let retry = OpenAIProvider::parse_retry_after(error);
        assert_eq!(retry, 30); // Default
    }
}
