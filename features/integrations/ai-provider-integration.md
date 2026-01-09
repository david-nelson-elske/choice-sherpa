# Integration: AI Provider Integration

**Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md
**Type:** External Service + Core Infrastructure
**Priority:** P0 (Required for conversation module)
**Depends On:** foundation module, conversation module ports

> Multi-provider AI integration with streaming support, cost tracking, and graceful degradation for the conversation module.

---

## Overview

The AI Provider Integration connects Choice Sherpa's conversation module to external LLM providers (OpenAI, Anthropic). This integration is the backbone of the decision support experience, enabling the AI-guided conversations that extract structured data from users.

### Key Integration Points

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        External: AI Providers                                │
│   OpenAI (GPT-4)  │  Anthropic (Claude)  │  Future Providers               │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ HTTPS (streaming SSE)
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    AIProvider Port (Adapter Layer)                           │
│   - complete()              # Single completion                              │
│   - stream_complete()       # Streaming completion                           │
│   - estimate_tokens()       # Token estimation                               │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ domain types
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                       Conversation Module                                    │
│   ConversationAgent │ SystemPrompts │ StructuredExtractor                   │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ events
                                    ▼
┌───────────────────┬───────────────────┬───────────────────┬─────────────────┐
│   Cost Tracker    │   Rate Limiter    │   Usage Logger    │   Dashboard     │
│  (per-user cost)  │  (API protection) │  (analytics)      │  (display)      │
└───────────────────┴───────────────────┴───────────────────┴─────────────────┘
```

---

## Modules Involved

| Module | Role | Changes Required |
|--------|------|------------------|
| `conversation` | Consumer | Calls AIProvider for completions |
| `adapters/openai` | Producer | OpenAI API implementation |
| `adapters/anthropic` | Producer | Anthropic API implementation |
| `membership` | Observer | Token usage counts against tier |
| `dashboard` | Consumer | Display AI cost/usage stats |

---

## Data Flow

### Streaming Completion Flow

```
User                   Conversation          AIProvider           OpenAI/Anthropic
  │                         │                     │                      │
  │── SendMessage ─────────►│                     │                      │
  │                         │                     │                      │
  │                         │── build_prompt ────►│                      │
  │                         │   (messages,        │                      │
  │                         │    system_prompt,   │                      │
  │                         │    component_type)  │                      │
  │                         │                     │                      │
  │                         │── stream_complete ─►│                      │
  │                         │                     │                      │
  │                         │                     │── POST /chat ───────►│
  │                         │                     │   (stream: true)     │
  │                         │                     │                      │
  │                         │                     │◄── SSE chunk ────────│
  │◄── StreamChunk ─────────│◄── yield chunk ────│                      │
  │                         │                     │                      │
  │                         │                     │◄── SSE chunk ────────│
  │◄── StreamChunk ─────────│◄── yield chunk ────│                      │
  │                         │                     │                      │
  │                         │                     │◄── [DONE] ───────────│
  │                         │◄── CompletionDone ──│                      │
  │                         │    (usage stats)    │                      │
  │                         │                     │                      │
  │                         │── emit AITokensUsed ►                      │
  │                         │   (tokens, cost)    │                      │
  │◄── MessageComplete ─────│                     │                      │
```

### Provider Failover Flow

```
Conversation           AIProvider           Primary (OpenAI)      Fallback (Anthropic)
  │                         │                     │                      │
  │── stream_complete ─────►│                     │                      │
  │                         │                     │                      │
  │                         │── try primary ─────►│                      │
  │                         │                     │                      │
  │                         │◄── 429 Rate Limit ──│                      │
  │                         │                     │                      │
  │                         │── try fallback ────────────────────────────►
  │                         │                     │                      │
  │◄── StreamChunk ─────────│◄── SSE chunk ───────────────────────────────│
  │                         │                     │                      │
  │                         │── emit ProviderFallback event              │
```

---

## AIProvider Port Contract

The `AIProvider` port abstracts all LLM provider interactions.

```rust
// backend/src/ports/ai_provider.rs

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

/// Port for AI/LLM provider interactions
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Generate a single completion (non-streaming)
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AIError>;

    /// Generate a streaming completion
    async fn stream_complete(
        &self,
        request: CompletionRequest
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, AIError>> + Send>>, AIError>;

    /// Estimate token count for a prompt (for cost estimation)
    fn estimate_tokens(&self, text: &str) -> u32;

    /// Get provider info (name, model, capabilities)
    fn provider_info(&self) -> ProviderInfo;
}

/// Request for AI completion
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub system_prompt: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub component_type: Option<ComponentType>,  // For prompt templating
    pub metadata: RequestMetadata,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct RequestMetadata {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub conversation_id: ConversationId,
    pub trace_id: String,
}

/// Response from AI completion
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub content: String,
    pub usage: TokenUsage,
    pub model: String,
    pub finish_reason: FinishReason,
}

#[derive(Debug, Clone)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub estimated_cost_cents: u32,  // Cost in cents for billing
}

#[derive(Debug, Clone)]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    Error,
}

/// Streaming chunk from AI completion
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub delta: String,
    pub finish_reason: Option<FinishReason>,
    pub usage: Option<TokenUsage>,  // Only present on final chunk
}

/// Provider information
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub name: String,           // "openai", "anthropic"
    pub model: String,          // "gpt-4-turbo", "claude-3-opus"
    pub max_context_tokens: u32,
    pub supports_streaming: bool,
    pub supports_functions: bool,
}

/// AI provider errors
#[derive(Debug, thiserror::Error)]
pub enum AIError {
    #[error("rate limited: retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u32 },

    #[error("context too long: {tokens} tokens exceeds {max} limit")]
    ContextTooLong { tokens: u32, max: u32 },

    #[error("content filtered: {reason}")]
    ContentFiltered { reason: String },

    #[error("provider unavailable: {message}")]
    Unavailable { message: String },

    #[error("authentication failed")]
    AuthenticationFailed,

    #[error("network error: {0}")]
    Network(String),

    #[error("parse error: {0}")]
    Parse(String),

    // SSRF Prevention errors (A10)
    #[error("invalid URL format")]
    InvalidUrl,

    #[error("host not allowed: {0}")]
    HostNotAllowed(String),

    #[error("model not allowed: {0}")]
    ModelNotAllowed(String),
}
```

---

## Adapter Implementations

### OpenAI Adapter

```rust
// backend/src/adapters/openai/client.rs

use url::Url;
use std::net::IpAddr;

/// SSRF Prevention: Allowlist of permitted AI provider hosts (A10)
const ALLOWED_AI_HOSTS: &[&str] = &[
    "api.openai.com",
    "api.anthropic.com",
];

/// Allowed OpenAI models to prevent model injection attacks
const ALLOWED_OPENAI_MODELS: &[&str] = &[
    "gpt-4o",
    "gpt-4-turbo",
    "gpt-4",
    "gpt-3.5-turbo",
];

pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
    cost_per_1k_input: u32,   // cents
    cost_per_1k_output: u32,  // cents
}

impl OpenAIProvider {
    /// Creates a new OpenAI provider with SSRF protection.
    ///
    /// # Errors
    /// Returns `AIError::HostNotAllowed` if base_url host is not in allowlist.
    /// Returns `AIError::InvalidUrl` if the URL cannot be parsed.
    /// Returns `AIError::ModelNotAllowed` if the model is not in allowlist.
    pub fn new(config: OpenAIConfig) -> Result<Self, AIError> {
        let base_url = config.base_url.unwrap_or_else(||
            "https://api.openai.com/v1".to_string()
        );
        let model = config.model.unwrap_or_else(|| "gpt-4-turbo".to_string());

        // SSRF Prevention: Validate URL and host
        let url = Url::parse(&base_url).map_err(|_| AIError::InvalidUrl)?;
        let host = url.host_str().ok_or(AIError::InvalidUrl)?;

        // Check host against allowlist
        if !ALLOWED_AI_HOSTS.contains(&host) {
            return Err(AIError::HostNotAllowed(host.to_string()));
        }

        // Block internal/private IP ranges
        if let Ok(ip) = host.parse::<IpAddr>() {
            if Self::is_private_ip(&ip) {
                return Err(AIError::HostNotAllowed(format!(
                    "Private IP addresses are not allowed: {}",
                    ip
                )));
            }
        }

        // Validate model against allowlist
        if !ALLOWED_OPENAI_MODELS.contains(&model.as_str()) {
            return Err(AIError::ModelNotAllowed(model));
        }

        Ok(Self {
            client: reqwest::Client::new(),
            api_key: config.api_key,
            model,
            base_url,
            cost_per_1k_input: config.cost_per_1k_input.unwrap_or(10),   // $0.10
            cost_per_1k_output: config.cost_per_1k_output.unwrap_or(30), // $0.30
        })
    }

    /// Check if an IP address is in a private/internal range
    fn is_private_ip(ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                ipv4.is_private()
                    || ipv4.is_loopback()
                    || ipv4.is_link_local()
                    || ipv4.is_broadcast()
                    || ipv4.is_documentation()
                    || ipv4.is_unspecified()
            }
            IpAddr::V6(ipv6) => {
                ipv6.is_loopback() || ipv6.is_unspecified()
            }
        }
    }

    fn calculate_cost(&self, usage: &TokenUsage) -> u32 {
        let input_cost = (usage.prompt_tokens as u32 * self.cost_per_1k_input) / 1000;
        let output_cost = (usage.completion_tokens as u32 * self.cost_per_1k_output) / 1000;
        input_cost + output_cost
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AIError> {
        // Implementation details...
    }

    async fn stream_complete(
        &self,
        request: CompletionRequest
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, AIError>> + Send>>, AIError> {
        // Implementation with SSE parsing...
    }

    fn estimate_tokens(&self, text: &str) -> u32 {
        // Use tiktoken or simple estimation: chars / 4
        (text.len() / 4) as u32
    }

    fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "openai".to_string(),
            model: self.model.clone(),
            max_context_tokens: 128000,
            supports_streaming: true,
            supports_functions: true,
        }
    }
}
```

### Anthropic Adapter

```rust
// backend/src/adapters/anthropic/client.rs

use url::Url;
use std::net::IpAddr;

/// Allowed Anthropic models to prevent model injection attacks
const ALLOWED_ANTHROPIC_MODELS: &[&str] = &[
    "claude-sonnet-4-20250514",
    "claude-3-opus-20240229",
    "claude-3-sonnet-20240229",
    "claude-3-haiku-20240307",
];

pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
    cost_per_1k_input: u32,
    cost_per_1k_output: u32,
}

impl AnthropicProvider {
    /// Creates a new Anthropic provider with SSRF protection.
    ///
    /// # Errors
    /// Returns `AIError::HostNotAllowed` if base_url host is not in allowlist.
    /// Returns `AIError::InvalidUrl` if the URL cannot be parsed.
    /// Returns `AIError::ModelNotAllowed` if the model is not in allowlist.
    pub fn new(config: AnthropicConfig) -> Result<Self, AIError> {
        let base_url = config.base_url.unwrap_or_else(||
            "https://api.anthropic.com/v1".to_string()
        );
        let model = config.model.unwrap_or_else(|| "claude-3-opus-20240229".to_string());

        // SSRF Prevention: Validate URL and host
        let url = Url::parse(&base_url).map_err(|_| AIError::InvalidUrl)?;
        let host = url.host_str().ok_or(AIError::InvalidUrl)?;

        // Check host against allowlist (uses same ALLOWED_AI_HOSTS constant)
        if !ALLOWED_AI_HOSTS.contains(&host) {
            return Err(AIError::HostNotAllowed(host.to_string()));
        }

        // Block internal/private IP ranges
        if let Ok(ip) = host.parse::<IpAddr>() {
            if is_private_ip(&ip) {
                return Err(AIError::HostNotAllowed(format!(
                    "Private IP addresses are not allowed: {}",
                    ip
                )));
            }
        }

        // Validate model against allowlist
        if !ALLOWED_ANTHROPIC_MODELS.contains(&model.as_str()) {
            return Err(AIError::ModelNotAllowed(model));
        }

        Ok(Self {
            client: reqwest::Client::new(),
            api_key: config.api_key,
            model,
            base_url,
            cost_per_1k_input: config.cost_per_1k_input.unwrap_or(15),   // $0.15
            cost_per_1k_output: config.cost_per_1k_output.unwrap_or(75), // $0.75
        })
    }
}

/// Check if an IP address is in a private/internal range (shared helper)
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            ipv4.is_private()
                || ipv4.is_loopback()
                || ipv4.is_link_local()
                || ipv4.is_broadcast()
                || ipv4.is_documentation()
                || ipv4.is_unspecified()
        }
        IpAddr::V6(ipv6) => {
            ipv6.is_loopback() || ipv6.is_unspecified()
        }
    }
}

#[async_trait]
impl AIProvider for AnthropicProvider {
    // Similar implementation with Anthropic's message format
    // ...
}
```

---

## System Prompts

Each PrOACT component has a tailored system prompt that guides the AI's behavior.

```rust
// backend/src/domain/conversation/prompts.rs

pub struct SystemPromptBuilder;

impl SystemPromptBuilder {
    pub fn for_component(component_type: ComponentType) -> String {
        let base = include_str!("prompts/base.txt");
        let component_specific = match component_type {
            ComponentType::IssueRaising => include_str!("prompts/issue_raising.txt"),
            ComponentType::ProblemFrame => include_str!("prompts/problem_frame.txt"),
            ComponentType::Objectives => include_str!("prompts/objectives.txt"),
            ComponentType::Alternatives => include_str!("prompts/alternatives.txt"),
            ComponentType::Consequences => include_str!("prompts/consequences.txt"),
            ComponentType::Tradeoffs => include_str!("prompts/tradeoffs.txt"),
            ComponentType::Recommendation => include_str!("prompts/recommendation.txt"),
            ComponentType::DecisionQuality => include_str!("prompts/decision_quality.txt"),
            ComponentType::NotesNextSteps => include_str!("prompts/notes.txt"),
        };

        format!("{}\n\n{}", base, component_specific)
    }
}
```

### Prompt Design Principles

| Principle | Implementation |
|-----------|----------------|
| **Role clarity** | AI is a "thoughtful decision professional" |
| **Non-directive** | Asks probing questions, never decides for user |
| **Structured extraction** | Guides toward component-specific outputs |
| **Assumption surfacing** | Explicitly asks about hidden assumptions |
| **Completeness checks** | Confirms all required fields before completion |

---

## Structured Data Extraction

The conversation module extracts structured data from AI responses for each component type.

```rust
// backend/src/domain/conversation/extractor.rs

pub struct StructuredExtractor {
    ai_provider: Arc<dyn AIProvider>,
}

impl StructuredExtractor {
    /// Extract structured data from conversation history
    pub async fn extract(
        &self,
        component_type: ComponentType,
        messages: &[Message],
    ) -> Result<ComponentOutput, ExtractionError> {
        let extraction_prompt = self.build_extraction_prompt(component_type, messages);

        let response = self.ai_provider.complete(CompletionRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: extraction_prompt,
            }],
            system_prompt: Some(include_str!("prompts/extraction_system.txt").to_string()),
            temperature: Some(0.0),  // Deterministic for extraction
            ..Default::default()
        }).await?;

        self.parse_extraction(component_type, &response.content)
    }

    fn build_extraction_prompt(&self, component_type: ComponentType, messages: &[Message]) -> String {
        let schema = self.schema_for_component(component_type);
        let conversation = messages.iter()
            .map(|m| format!("{:?}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "Extract the following structured data from this conversation.\n\n\
             Schema:\n{}\n\n\
             Conversation:\n{}\n\n\
             Respond with valid JSON matching the schema.",
            schema, conversation
        )
    }
}
```

---

## Cost Tracking Events

AI usage generates domain events for cost tracking and analytics.

```rust
// backend/src/domain/conversation/events.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AITokensUsed {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub conversation_id: ConversationId,
    pub component_type: ComponentType,
    pub provider: String,
    pub model: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub estimated_cost_cents: u32,
    pub occurred_at: Timestamp,
}

impl DomainEvent for AITokensUsed {
    fn event_type(&self) -> &str {
        "conversation.ai_tokens_used"
    }

    fn aggregate_type(&self) -> &str {
        "Conversation"
    }

    fn aggregate_id(&self) -> String {
        self.conversation_id.to_string()
    }
}
```

---

## Provider Configuration

```rust
// backend/src/config/ai.rs

#[derive(Debug, Clone, Deserialize)]
pub struct AIConfig {
    pub primary_provider: ProviderConfig,
    pub fallback_provider: Option<ProviderConfig>,
    pub retry_policy: RetryPolicy,
    pub cost_limits: CostLimits,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: ProviderType,
    pub api_key: String,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub timeout_secs: Option<u32>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_backoff_ms: u32,
    pub max_backoff_ms: u32,
    pub retry_on_rate_limit: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CostLimits {
    pub max_tokens_per_request: u32,
    pub max_cost_per_user_daily_cents: u32,
    pub max_cost_per_session_cents: u32,
}
```

### Environment Variables

```bash
# .env
AI_PRIMARY_PROVIDER=openai
AI_PRIMARY_API_KEY=sk-xxx
AI_PRIMARY_MODEL=gpt-4-turbo

AI_FALLBACK_PROVIDER=anthropic
AI_FALLBACK_API_KEY=sk-ant-xxx
AI_FALLBACK_MODEL=claude-3-opus-20240229

AI_MAX_TOKENS_PER_REQUEST=4096
AI_MAX_COST_PER_USER_DAILY_CENTS=500
AI_MAX_COST_PER_SESSION_CENTS=100
```

---

## Failover Strategy

```rust
// backend/src/adapters/ai/failover.rs

pub struct FailoverAIProvider {
    primary: Box<dyn AIProvider>,
    fallback: Option<Box<dyn AIProvider>>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl FailoverAIProvider {
    pub async fn complete_with_failover(
        &self,
        request: CompletionRequest
    ) -> Result<CompletionResponse, AIError> {
        match self.primary.complete(request.clone()).await {
            Ok(response) => Ok(response),
            Err(AIError::RateLimited { .. }) | Err(AIError::Unavailable { .. }) => {
                if let Some(fallback) = &self.fallback {
                    // Emit failover event
                    self.event_publisher.publish(ProviderFallback {
                        primary: self.primary.provider_info().name,
                        fallback: fallback.provider_info().name,
                        reason: "primary unavailable".to_string(),
                        occurred_at: Timestamp::now(),
                    }).await?;

                    fallback.complete(request).await
                } else {
                    Err(AIError::Unavailable {
                        message: "Primary provider unavailable, no fallback configured".to_string()
                    })
                }
            }
            Err(e) => Err(e),
        }
    }
}
```

---

## Membership Integration

AI usage counts against membership tier limits.

```rust
// backend/src/application/handlers/ai_usage_handler.rs

pub struct AIUsageHandler {
    membership_repo: Arc<dyn MembershipRepository>,
    usage_tracker: Arc<dyn UsageTracker>,
}

impl EventHandler for AIUsageHandler {
    fn handles(&self) -> &[&str] {
        &["conversation.ai_tokens_used"]
    }

    async fn handle(&self, event: EventEnvelope) -> Result<(), HandlerError> {
        let tokens_used: AITokensUsed = serde_json::from_value(event.payload)?;

        // Track usage for billing
        self.usage_tracker.record_ai_usage(
            &tokens_used.user_id,
            tokens_used.prompt_tokens + tokens_used.completion_tokens,
            tokens_used.estimated_cost_cents,
        ).await?;

        // Check if user is approaching limits
        let membership = self.membership_repo
            .find_by_user(&tokens_used.user_id)
            .await?;

        if let Some(membership) = membership {
            let daily_usage = self.usage_tracker
                .get_daily_cost(&tokens_used.user_id)
                .await?;

            let limit = membership.tier().daily_ai_cost_limit_cents();

            if daily_usage > limit * 80 / 100 {
                // Emit warning event
            }

            if daily_usage >= limit {
                // Emit limit reached event
            }
        }

        Ok(())
    }
}
```

---

## API Endpoints

```rust
// POST /api/conversations/{id}/messages
// Streaming endpoint for conversation messages

pub async fn send_message(
    Path(conversation_id): Path<ConversationId>,
    State(state): State<AppState>,
    Json(request): Json<SendMessageRequest>,
) -> impl IntoResponse {
    // Stream response using SSE
    let stream = state.conversation_service
        .send_message_streaming(conversation_id, request)
        .await?;

    Sse::new(stream)
        .keep_alive(KeepAlive::default())
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[tokio::test]
async fn test_openai_provider_complete() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "choices": [{"message": {"content": "Hello!"}}],
                "usage": {"prompt_tokens": 10, "completion_tokens": 5}
            })))
        .mount(&mock_server)
        .await;

    let provider = OpenAIProvider::new(OpenAIConfig {
        api_key: "test".to_string(),
        base_url: Some(mock_server.uri()),
        ..Default::default()
    });

    let response = provider.complete(CompletionRequest {
        messages: vec![Message {
            role: MessageRole::User,
            content: "Hi".to_string(),
        }],
        ..Default::default()
    }).await.unwrap();

    assert_eq!(response.content, "Hello!");
    assert_eq!(response.usage.total_tokens, 15);
}
```

### Integration Tests

```rust
#[tokio::test]
#[ignore = "requires API key"]
async fn test_real_openai_streaming() {
    let api_key = std::env::var("OPENAI_API_KEY").unwrap();
    let provider = OpenAIProvider::new(OpenAIConfig {
        api_key,
        model: Some("gpt-3.5-turbo".to_string()),
        ..Default::default()
    });

    let mut stream = provider.stream_complete(CompletionRequest {
        messages: vec![Message {
            role: MessageRole::User,
            content: "Say hello".to_string(),
        }],
        max_tokens: Some(10),
        ..Default::default()
    }).await.unwrap();

    let mut full_response = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        full_response.push_str(&chunk.delta);
    }

    assert!(!full_response.is_empty());
}
```

---

## Implementation Phases

### Phase 1: Core Infrastructure (Foundation)

- [x] Define AIProvider port interface
- [x] Define domain types (CompletionRequest, StreamChunk, etc.)
- [x] Define AIError types
- [x] Create mock AIProvider for testing
- [x] Write unit tests for port contract

### Phase 2: OpenAI Adapter

- [ ] Implement OpenAIProvider
- [ ] SSE stream parsing for streaming completions
- [ ] Token counting (tiktoken)
- [ ] Cost calculation
- [ ] Rate limit handling with exponential backoff
- [ ] Integration tests with mock server

### Phase 3: Anthropic Adapter

- [ ] Implement AnthropicProvider
- [ ] Handle Anthropic message format differences
- [ ] SSE stream parsing
- [ ] Cost calculation for Claude models
- [ ] Integration tests

### Phase 4: Failover & Cost Tracking

- [ ] Implement FailoverAIProvider wrapper
- [ ] Emit AITokensUsed events
- [ ] Implement AIUsageHandler for cost tracking
- [ ] Daily/session cost limit enforcement
- [ ] Emit ProviderFallback events

### Phase 5: Conversation Integration

- [ ] SystemPromptBuilder for each component
- [ ] StructuredExtractor for data extraction
- [ ] Integration with conversation command handlers
- [ ] Streaming SSE endpoint

### Phase 6: Production Hardening

- [ ] Request timeout configuration
- [ ] Circuit breaker pattern
- [ ] Structured logging for requests/responses
- [ ] Cost alerting
- [ ] Dashboard integration for usage display

---

## Exit Criteria

1. **Streaming works end-to-end**: User messages stream back AI responses in real-time
2. **Provider failover**: Primary failure triggers fallback with event emission
3. **Cost tracking**: Every completion generates AITokensUsed event with accurate cost
4. **Limits enforced**: Users hitting daily limits receive clear error
5. **Extraction works**: Structured data extracted from conversations for all 9 components
6. **Tests pass**: Unit tests with mocks, integration tests with real APIs (CI-skipped)
