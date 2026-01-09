# Conversation Module Specification

## Overview

The Conversation module manages AI agent behavior, conversation flow, and message handling. It implements the "thoughtful decision professional" persona across all PrOACT components, handling the interaction between users and the AI assistant.

---

## Module Classification

| Attribute | Value |
|-----------|-------|
| **Type** | Full Module (Ports + Adapters) |
| **Language** | Rust |
| **Responsibility** | AI agent behavior, message handling, conversation state |
| **Domain Dependencies** | foundation, proact-types |
| **External Dependencies** | `async-trait`, `tokio`, `tokio-stream`, `serde_json` |

---

## Architecture

### Hexagonal Structure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        CONVERSATION MODULE                                   │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         DOMAIN LAYER                                    │ │
│  │                                                                         │ │
│  │   ┌────────────────────────────────────────────────────────────────┐   │ │
│  │   │                  Conversation Entity                            │   │ │
│  │   │                                                                 │   │ │
│  │   │   - id: ConversationId                                          │   │ │
│  │   │   - component_id: ComponentId                                   │   │ │
│  │   │   - component_type: ComponentType                               │   │ │
│  │   │   - messages: Vec<Message>                                      │   │ │
│  │   │   - agent_state: AgentState                                     │   │ │
│  │   │                                                                 │   │ │
│  │   │   + add_user_message(content) -> Message                        │   │ │
│  │   │   + add_assistant_message(content) -> Message                   │   │ │
│  │   │   + update_agent_state(state)                                   │   │ │
│  │   └────────────────────────────────────────────────────────────────┘   │ │
│  │                                                                         │ │
│  │   ┌────────────────────┐  ┌────────────────────────────────────────┐   │ │
│  │   │    AgentState      │  │         AgentConfig                    │   │ │
│  │   │    (Value Object)  │  │    (Per-component behavior)            │   │ │
│  │   └────────────────────┘  └────────────────────────────────────────┘   │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
│                                      ▼                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                          PORT LAYER                                     │ │
│  │                                                                         │ │
│  │   ┌─────────────────────────────┐  ┌─────────────────────────────────┐ │ │
│  │   │      AIProvider             │  │  ConversationRepository         │ │ │
│  │   │  (Infrastructure boundary)  │  │  (Persistence)                  │ │ │
│  │   │                             │  │                                  │ │ │
│  │   │  + complete(req) -> resp    │  │  + save(conversation)            │ │ │
│  │   │  + stream(req) -> Stream    │  │  + find_by_component(id)         │ │ │
│  │   └─────────────────────────────┘  │  + append_message(id, msg)       │ │ │
│  │                                     └─────────────────────────────────┘ │ │
│  │   ┌─────────────────────────────────────────────────────────────────┐  │ │
│  │   │                  ConversationReader                              │  │ │
│  │   │   + get_by_component(id) -> ConversationView                     │  │ │
│  │   │   + get_message_count(id) -> usize                               │  │ │
│  │   └─────────────────────────────────────────────────────────────────┘  │ │
│  │   ┌─────────────────────────────────────────────────────────────────┐  │ │
│  │   │                 ConnectionRegistry                               │  │ │
│  │   │   (Multi-server WebSocket connection tracking)                   │  │ │
│  │   │   + register(user_id, server_id)                                 │  │ │
│  │   │   + unregister(user_id, server_id)                               │  │ │
│  │   │   + find_servers(user_id) -> Vec<ServerId>                       │  │ │
│  │   └─────────────────────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
│                                      ▼                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                        ADAPTER LAYER                                    │ │
│  │                                                                         │ │
│  │   ┌───────────────┐  ┌───────────────┐  ┌─────────────────────────┐    │ │
│  │   │ OpenAIAdapter │  │AnthropicAdapter│  │   MockAIAdapter         │    │ │
│  │   │   (wrapped    │  │   (wrapped    │  │                          │    │ │
│  │   │  in Resilient │  │  in Resilient │  │                          │    │ │
│  │   │  AIProvider)  │  │  AIProvider)  │  │                          │    │ │
│  │   └───────────────┘  └───────────────┘  └─────────────────────────┘    │ │
│  │                                                                         │ │
│  │   ┌───────────────────────────────────┐  ┌─────────────────────────┐   │ │
│  │   │ PostgresConversationRepository    │  │ HTTP + WebSocket        │   │ │
│  │   └───────────────────────────────────┘  │ Handlers                │   │ │
│  │                                           └─────────────────────────┘   │ │
│  │   ┌───────────────────────────────────┐  ┌─────────────────────────┐   │ │
│  │   │ RedisConnectionRegistry           │  │  ResilientAIProvider    │   │ │
│  │   │ (Multi-server WebSocket tracking) │  │  (Circuit breaker wrap) │   │ │
│  │   └───────────────────────────────────┘  └─────────────────────────┘   │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Domain Layer

### Conversation Entity

```rust
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::foundation::{ComponentId, ComponentType, Timestamp};
use crate::proact::{Message, MessageId, Role};

/// Unique identifier for a conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConversationId(Uuid);

impl ConversationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

/// Conversation tracks messages for a specific component
#[derive(Debug, Clone)]
pub struct Conversation {
    id: ConversationId,
    component_id: ComponentId,
    component_type: ComponentType,
    messages: Vec<Message>,
    agent_state: AgentState,
    created_at: Timestamp,
    updated_at: Timestamp,
}

impl Conversation {
    /// Creates a new conversation for a component
    pub fn new(component_id: ComponentId, component_type: ComponentType) -> Self {
        let now = Timestamp::now();
        Self {
            id: ConversationId::new(),
            component_id,
            component_type,
            messages: Vec::new(),
            agent_state: AgentState::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitutes a conversation from persistence
    pub fn reconstitute(
        id: ConversationId,
        component_id: ComponentId,
        component_type: ComponentType,
        messages: Vec<Message>,
        agent_state: AgentState,
        created_at: Timestamp,
        updated_at: Timestamp,
    ) -> Self {
        Self {
            id,
            component_id,
            component_type,
            messages,
            agent_state,
            created_at,
            updated_at,
        }
    }

    // === Accessors ===

    pub fn id(&self) -> ConversationId { self.id }
    pub fn component_id(&self) -> ComponentId { self.component_id }
    pub fn component_type(&self) -> ComponentType { self.component_type }
    pub fn messages(&self) -> &[Message] { &self.messages }
    pub fn agent_state(&self) -> &AgentState { &self.agent_state }
    pub fn created_at(&self) -> Timestamp { self.created_at }
    pub fn updated_at(&self) -> Timestamp { self.updated_at }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn last_message(&self) -> Option<&Message> {
        self.messages.last()
    }

    pub fn last_assistant_message(&self) -> Option<&Message> {
        self.messages.iter().rev().find(|m| m.role == Role::Assistant)
    }

    // === Message Management ===

    /// Adds a user message to the conversation
    pub fn add_user_message(&mut self, content: impl Into<String>) -> &Message {
        let message = Message::user(content);
        self.messages.push(message);
        self.updated_at = Timestamp::now();
        self.messages.last().unwrap()
    }

    /// Adds an assistant message to the conversation
    pub fn add_assistant_message(&mut self, content: impl Into<String>) -> &Message {
        let message = Message::assistant(content);
        self.messages.push(message);
        self.updated_at = Timestamp::now();
        self.messages.last().unwrap()
    }

    /// Adds a system message to the conversation
    pub fn add_system_message(&mut self, content: impl Into<String>) -> &Message {
        let message = Message::system(content);
        self.messages.push(message);
        self.updated_at = Timestamp::now();
        self.messages.last().unwrap()
    }

    /// Removes the last assistant message (for regeneration)
    pub fn remove_last_assistant_message(&mut self) -> Option<Message> {
        if let Some(pos) = self.messages.iter().rposition(|m| m.role == Role::Assistant) {
            self.updated_at = Timestamp::now();
            Some(self.messages.remove(pos))
        } else {
            None
        }
    }

    // === Agent State ===

    /// Updates the agent state
    pub fn update_agent_state(&mut self, state: AgentState) {
        self.agent_state = state;
        self.updated_at = Timestamp::now();
    }

    /// Transitions to the next phase
    pub fn advance_phase(&mut self, next_phase: impl Into<String>) {
        self.agent_state.current_phase = next_phase.into();
        self.updated_at = Timestamp::now();
    }

    /// Sets the awaiting confirmation flag
    pub fn set_awaiting_confirmation(&mut self, awaiting: bool) {
        self.agent_state.awaiting_confirmation = awaiting;
        self.updated_at = Timestamp::now();
    }

    // === Context Building ===

    /// Returns messages formatted for AI prompt (last N messages)
    pub fn get_context_messages(&self, max_messages: usize) -> Vec<&Message> {
        let start = self.messages.len().saturating_sub(max_messages);
        self.messages[start..].iter().collect()
    }

    /// Estimates token count for the conversation
    pub fn estimate_tokens(&self) -> u32 {
        // Rough estimate: ~4 chars per token
        let total_chars: usize = self.messages.iter()
            .map(|m| m.content.len())
            .sum();
        (total_chars / 4) as u32
    }
}
```

### AgentState Value Object

```rust
use serde::{Deserialize, Serialize};

/// Tracks conversation progress and agent behavior state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentState {
    /// Current phase of the conversation (e.g., "listening", "categorizing", "confirming")
    pub current_phase: String,

    /// Questions the agent plans to ask
    pub pending_questions: Vec<String>,

    /// Whether agent is waiting for user confirmation
    pub awaiting_confirmation: bool,

    /// Count of items extracted from conversation
    pub extracted_items: u32,

    /// Custom phase-specific data
    #[serde(default)]
    pub phase_data: serde_json::Value,
}

impl AgentState {
    /// Creates a new agent state with initial phase
    pub fn new(phase: impl Into<String>) -> Self {
        Self {
            current_phase: phase.into(),
            pending_questions: Vec::new(),
            awaiting_confirmation: false,
            extracted_items: 0,
            phase_data: serde_json::Value::Null,
        }
    }

    /// Adds a pending question
    pub fn add_question(&mut self, question: impl Into<String>) {
        self.pending_questions.push(question.into());
    }

    /// Pops the next pending question
    pub fn next_question(&mut self) -> Option<String> {
        if self.pending_questions.is_empty() {
            None
        } else {
            Some(self.pending_questions.remove(0))
        }
    }

    /// Increments extracted items count
    pub fn increment_extracted(&mut self) {
        self.extracted_items += 1;
    }
}
```

### AgentConfig (Per-Component Behavior)

```rust
use crate::foundation::ComponentType;
use serde::{Deserialize, Serialize};

/// Configuration for agent behavior in a specific component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub component_type: ComponentType,
    pub system_prompt: String,
    pub phases: Vec<AgentPhase>,
    pub extraction_rules: Vec<ExtractionRule>,
    pub max_context_messages: usize,
    pub temperature: f32,
}

impl AgentConfig {
    /// Returns the default config for a component type
    pub fn for_component(component_type: ComponentType) -> Self {
        match component_type {
            ComponentType::IssueRaising => Self::issue_raising_config(),
            ComponentType::ProblemFrame => Self::problem_frame_config(),
            ComponentType::Objectives => Self::objectives_config(),
            ComponentType::Alternatives => Self::alternatives_config(),
            ComponentType::Consequences => Self::consequences_config(),
            ComponentType::Tradeoffs => Self::tradeoffs_config(),
            ComponentType::Recommendation => Self::recommendation_config(),
            ComponentType::DecisionQuality => Self::decision_quality_config(),
            ComponentType::NotesNextSteps => Self::notes_next_steps_config(),
        }
    }

    fn issue_raising_config() -> Self {
        Self {
            component_type: ComponentType::IssueRaising,
            system_prompt: include_str!("prompts/issue_raising.txt").to_string(),
            phases: vec![
                AgentPhase {
                    name: "listening".to_string(),
                    objective: "Gather user's initial thoughts about the decision".to_string(),
                    prompt_guidance: "Ask open-ended questions to understand the situation".to_string(),
                },
                AgentPhase {
                    name: "categorizing".to_string(),
                    objective: "Categorize items into decisions, objectives, uncertainties".to_string(),
                    prompt_guidance: "Present categorization and ask for confirmation".to_string(),
                },
                AgentPhase {
                    name: "confirming".to_string(),
                    objective: "Get user confirmation of categorization".to_string(),
                    prompt_guidance: "Ask if categorization is correct, make adjustments".to_string(),
                },
            ],
            extraction_rules: vec![
                ExtractionRule {
                    field: "potential_decisions".to_string(),
                    pattern: r"decision|choose|decide|option".to_string(),
                    description: "Statements about choices to be made".to_string(),
                },
                ExtractionRule {
                    field: "objectives".to_string(),
                    pattern: r"want|need|important|goal|value".to_string(),
                    description: "Statements about what matters".to_string(),
                },
                ExtractionRule {
                    field: "uncertainties".to_string(),
                    pattern: r"unsure|don't know|uncertain|maybe|might".to_string(),
                    description: "Statements about unknowns".to_string(),
                },
            ],
            max_context_messages: 20,
            temperature: 0.7,
        }
    }

    // ... similar configs for other components
    fn problem_frame_config() -> Self { /* ... */ todo!() }
    fn objectives_config() -> Self { /* ... */ todo!() }
    fn alternatives_config() -> Self { /* ... */ todo!() }
    fn consequences_config() -> Self { /* ... */ todo!() }
    fn tradeoffs_config() -> Self { /* ... */ todo!() }
    fn recommendation_config() -> Self { /* ... */ todo!() }
    fn decision_quality_config() -> Self { /* ... */ todo!() }
    fn notes_next_steps_config() -> Self { /* ... */ todo!() }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPhase {
    pub name: String,
    pub objective: String,
    pub prompt_guidance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRule {
    pub field: String,
    pub pattern: String,
    pub description: String,
}
```

---

## Ports

### AIProvider Port

```rust
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use crate::proact::Message;

/// Port for AI completion providers (OpenAI, Anthropic, etc.)
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Synchronous completion (waits for full response)
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AIError>;

    /// Streaming completion (returns chunks as they arrive)
    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<CompletionChunk, AIError>> + Send>>, AIError>;
}

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    /// System prompt defining agent behavior
    pub system_prompt: String,

    /// Conversation history
    pub messages: Vec<Message>,

    /// Maximum tokens in response
    pub max_tokens: u32,

    /// Temperature (0.0 - 2.0)
    pub temperature: f32,

    /// Optional model override
    pub model: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CompletionResponse {
    /// Generated text
    pub content: String,

    /// Tokens used (prompt + completion)
    pub tokens_used: u32,

    /// Why generation stopped
    pub finish_reason: FinishReason,
}

#[derive(Debug, Clone)]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    ToolCalls,
}

#[derive(Debug, Clone)]
pub struct CompletionChunk {
    /// Partial content
    pub content: String,

    /// Whether this is the final chunk
    pub done: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum AIError {
    #[error("Rate limited: retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },

    #[error("Model unavailable: {0}")]
    ModelUnavailable(String),

    #[error("Content filtered: {0}")]
    ContentFiltered(String),

    #[error("Token limit exceeded: {used} > {limit}")]
    TokenLimitExceeded { used: u32, limit: u32 },

    #[error("Network error: {0}")]
    Network(String),

    #[error("Provider error: {0}")]
    Provider(String),

    // SSRF Prevention errors (A10)
    #[error("Invalid URL format")]
    InvalidUrl,

    #[error("Host not allowed: {0}")]
    HostNotAllowed(String),

    #[error("Model not allowed: {0}")]
    ModelNotAllowed(String),
}
```

### ConversationRepository (Write)

```rust
use async_trait::async_trait;
use crate::foundation::ComponentId;
use crate::proact::Message;
use super::{Conversation, ConversationId};

#[async_trait]
pub trait ConversationRepository: Send + Sync {
    /// Persists a new conversation
    async fn save(&self, conversation: &Conversation) -> Result<(), RepositoryError>;

    /// Updates an existing conversation
    async fn update(&self, conversation: &Conversation) -> Result<(), RepositoryError>;

    /// Finds a conversation by component ID
    async fn find_by_component(
        &self,
        component_id: ComponentId,
    ) -> Result<Option<Conversation>, RepositoryError>;

    /// Appends a message to a conversation (optimized for append-only)
    async fn append_message(
        &self,
        component_id: ComponentId,
        message: &Message,
    ) -> Result<(), RepositoryError>;
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Conversation not found for component: {0}")]
    NotFound(ComponentId),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
```

### ConversationReader (Query)

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::foundation::{ComponentId, ComponentType};
use crate::proact::Message;
use super::{AgentState, ConversationId};

#[async_trait]
pub trait ConversationReader: Send + Sync {
    /// Gets a conversation view by component
    async fn get_by_component(
        &self,
        component_id: ComponentId,
    ) -> Result<Option<ConversationView>, ReaderError>;

    /// Gets message count for a component
    async fn get_message_count(&self, component_id: ComponentId) -> Result<usize, ReaderError>;

    /// Gets recent messages (for context)
    async fn get_recent_messages(
        &self,
        component_id: ComponentId,
        limit: usize,
    ) -> Result<Vec<Message>, ReaderError>;
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConversationView {
    pub id: ConversationId,
    pub component_id: ComponentId,
    pub component_type: ComponentType,
    pub messages: Vec<Message>,
    pub agent_state: AgentState,
    pub message_count: usize,
    pub last_message_at: Option<DateTime<Utc>>,
}

#[derive(Debug, thiserror::Error)]
pub enum ReaderError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

### ConnectionRegistry Port (Multi-Server WebSocket)

```rust
use async_trait::async_trait;
use crate::foundation::UserId;

/// Unique identifier for a server instance in a multi-server deployment.
/// Format: hostname:port or container ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ServerId(String);

impl ServerId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Port for tracking WebSocket connections across multiple servers.
///
/// In a multi-server deployment, a user might connect to Server A,
/// but an event affecting them originates from Server B. This port
/// enables Server B to know which server(s) hold the user's connection.
#[async_trait]
pub trait ConnectionRegistry: Send + Sync {
    /// Register a user's connection on a specific server.
    ///
    /// Called when a WebSocket connection is established.
    async fn register(
        &self,
        user_id: &UserId,
        server_id: &ServerId,
    ) -> Result<(), ConnectionRegistryError>;

    /// Unregister a user's connection from a specific server.
    ///
    /// Called when a WebSocket connection is closed.
    async fn unregister(
        &self,
        user_id: &UserId,
        server_id: &ServerId,
    ) -> Result<(), ConnectionRegistryError>;

    /// Find all servers that have connections for a user.
    ///
    /// A user may have multiple connections (different browser tabs,
    /// devices) potentially on different servers.
    async fn find_servers(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<ServerId>, ConnectionRegistryError>;

    /// Check if a user has any active connections.
    async fn is_connected(&self, user_id: &UserId) -> Result<bool, ConnectionRegistryError>;

    /// Refresh TTL for a connection (heartbeat).
    ///
    /// Called periodically to prevent stale connections from
    /// lingering if a server crashes without cleanup.
    async fn heartbeat(
        &self,
        user_id: &UserId,
        server_id: &ServerId,
    ) -> Result<(), ConnectionRegistryError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectionRegistryError {
    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Connection not found")]
    NotFound,
}
```

### CircuitBreaker Port (AI Provider Resilience)

```rust
use async_trait::async_trait;

/// Circuit breaker states for external service protection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation - requests flow through
    Closed,
    /// Too many failures - requests rejected immediately
    Open,
    /// Testing if service recovered - limited requests
    HalfOpen,
}

/// Port for circuit breaker functionality.
///
/// Protects against cascading failures when external services
/// (AI providers) become unavailable or slow.
#[async_trait]
pub trait CircuitBreaker: Send + Sync {
    /// Get the current state of the circuit.
    fn state(&self) -> CircuitState;

    /// Check if a request should be allowed through.
    fn should_allow(&self) -> bool;

    /// Record a successful request.
    fn record_success(&self);

    /// Record a failed request.
    fn record_failure(&self);

    /// Reset the circuit to closed state.
    fn reset(&self);
}

/// Configuration for circuit breaker behavior.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening circuit
    pub failure_threshold: u32,

    /// Time to wait before testing recovery (seconds)
    pub recovery_timeout_secs: u64,

    /// Number of successes in half-open state to close circuit
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout_secs: 30,
            success_threshold: 3,
        }
    }
}
```

---

## Application Layer

### Authorization

**CRITICAL**: All conversation operations require authorization. The conversation module does not own sessions directly, so it must traverse the ownership chain: `Component → Cycle → Session → User`.

See `docs/architecture/authorization-model.md` for the complete authorization model.

```rust
/// Helper trait for authorization in conversation handlers
#[async_trait]
trait ConversationAuthorization {
    /// Verifies user owns the session containing this cycle
    async fn authorize_cycle_access(
        &self,
        user_id: &UserId,
        cycle_id: CycleId,
    ) -> Result<(), CommandError>;

    /// Verifies user owns the session containing this component
    async fn authorize_component_access(
        &self,
        user_id: &UserId,
        component_id: ComponentId,
    ) -> Result<(), CommandError>;
}
```

### Commands

#### SendMessage Command

```rust
use std::sync::Arc;
use tracing::{info, warn, instrument};
use crate::foundation::{ComponentId, CycleId, UserId};
use crate::ports::{AIProvider, ConversationRepository, CycleRepository, SessionRepository};
use crate::domain::{AgentConfig, Conversation};
use crate::proact::Message;

#[derive(Debug, Clone)]
pub struct SendMessageCommand {
    pub user_id: UserId,  // REQUIRED: For authorization
    pub cycle_id: CycleId,
    pub component_type: ComponentType,
    pub content: String,
}

pub struct SendMessageHandler {
    conversation_repo: Arc<dyn ConversationRepository>,
    cycle_repo: Arc<dyn CycleRepository>,
    session_repo: Arc<dyn SessionRepository>,  // For authorization
    ai_provider: Arc<dyn AIProvider>,
}

impl SendMessageHandler {
    #[instrument(skip(self, cmd), fields(user_id = %cmd.user_id, cycle_id = %cmd.cycle_id))]
    pub async fn handle(&self, cmd: SendMessageCommand) -> Result<SendMessageResult, CommandError> {
        // 1. Load cycle
        let cycle = self.cycle_repo
            .find_by_id(cmd.cycle_id)
            .await?
            .ok_or(CommandError::CycleNotFound(cmd.cycle_id))?;

        // 2. AUTHORIZATION: Verify user owns the session
        let session = self.session_repo
            .find_by_id(cycle.session_id())
            .await?
            .ok_or(CommandError::SessionNotFound)?;

        // SECURITY: Log authorization outcomes for audit trail
        match session.authorize(&cmd.user_id) {
            Ok(_) => {
                info!(user_id = %cmd.user_id, session_id = %session.id(), "Authorization successful for send_message");
            }
            Err(e) => {
                warn!(user_id = %cmd.user_id, session_id = %session.id(), "Authorization failed for send_message");
                return Err(CommandError::Unauthorized);
            }
        }

        // 3. Get component
        let component = cycle.get_component(cmd.component_type)
            .ok_or(CommandError::ComponentNotFound)?;

        let component_id = component.id();

        // 4. Load or create conversation
        let mut conversation = self.conversation_repo
            .find_by_component(component_id)
            .await?
            .unwrap_or_else(|| Conversation::new(component_id, cmd.component_type));

        // 5. Add user message
        conversation.add_user_message(&cmd.content);

        // 6. Get agent config
        let config = AgentConfig::for_component(cmd.component_type);

        // 7. Build completion request
        let request = self.build_request(&conversation, &config, component);

        // 8. Get AI response
        let response = self.ai_provider.complete(request).await?;

        // 9. Add assistant message
        let assistant_msg = conversation.add_assistant_message(&response.content);

        // 10. Extract structured data (if any)
        let extracted = self.extract_structured_data(&response.content, &config);

        // 11. Update component output if extracted
        if let Some(output) = &extracted {
            let mut cycle = self.cycle_repo.find_by_id(cmd.cycle_id).await?.unwrap();
            cycle.update_component_output(cmd.component_type, output.clone())?;
            self.cycle_repo.update(&cycle).await?;
        }

        // 12. Persist conversation
        self.conversation_repo.save(&conversation).await?;

        Ok(SendMessageResult {
            message: assistant_msg.clone(),
            extracted_data: extracted,
            tokens_used: response.tokens_used,
        })
    }

    fn build_request(
        &self,
        conversation: &Conversation,
        config: &AgentConfig,
        component: &ComponentVariant,
    ) -> CompletionRequest {
        let context_messages = conversation.get_context_messages(config.max_context_messages);

        // Include current component state in system prompt
        let system_prompt = format!(
            "{}\n\nCurrent Component State:\n{}",
            config.system_prompt,
            serde_json::to_string_pretty(&component.output_as_value()).unwrap_or_default()
        );

        CompletionRequest {
            system_prompt,
            messages: context_messages.into_iter().cloned().collect(),
            max_tokens: 2000,
            temperature: config.temperature,
            model: None,
        }
    }

    fn extract_structured_data(
        &self,
        content: &str,
        config: &AgentConfig,
    ) -> Option<serde_json::Value> {
        // Use extraction rules to parse structured data from response
        // This is a simplified implementation
        // Real implementation would use regex or AI-based extraction
        None
    }
}

#[derive(Debug, Clone)]
pub struct SendMessageResult {
    pub message: Message,
    pub extracted_data: Option<serde_json::Value>,
    pub tokens_used: u32,
}
```

#### StreamMessage Command (WebSocket)

```rust
use futures::Stream;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub struct StreamMessageCommand {
    pub user_id: UserId,  // REQUIRED: For authorization
    pub cycle_id: CycleId,
    pub component_type: ComponentType,
    pub content: String,
}

pub struct StreamMessageHandler {
    conversation_repo: Arc<dyn ConversationRepository>,
    cycle_repo: Arc<dyn CycleRepository>,
    session_repo: Arc<dyn SessionRepository>,  // For authorization
    ai_provider: Arc<dyn AIProvider>,
}

impl StreamMessageHandler {
    #[instrument(skip(self, cmd), fields(user_id = %cmd.user_id, cycle_id = %cmd.cycle_id))]
    pub async fn handle(
        &self,
        cmd: StreamMessageCommand,
    ) -> Result<Pin<Box<dyn Stream<Item = StreamChunk> + Send>>, CommandError> {
        // 1. AUTHORIZATION: Verify user owns the session
        let cycle = self.cycle_repo.find_by_id(cmd.cycle_id).await?
            .ok_or(CommandError::CycleNotFound)?;
        let session = self.session_repo.find_by_id(cycle.session_id()).await?
            .ok_or(CommandError::SessionNotFound)?;

        // SECURITY: Log authorization outcomes for audit trail
        match session.authorize(&cmd.user_id) {
            Ok(_) => {
                info!(user_id = %cmd.user_id, session_id = %session.id(), "Authorization successful for stream_message");
            }
            Err(e) => {
                warn!(user_id = %cmd.user_id, session_id = %session.id(), "Authorization failed for stream_message");
                return Err(CommandError::Unauthorized);
            }
        }

        // 2. Add user message
        // 3. Start streaming from AI
        // 4. Accumulate chunks for final message
        // 5. Save complete message when done
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub content: String,
    pub done: bool,
    pub message_id: Option<MessageId>,
}
```

#### RegenerateResponse Command

```rust
#[derive(Debug, Clone)]
pub struct RegenerateResponseCommand {
    pub user_id: UserId,  // REQUIRED: For authorization
    pub cycle_id: CycleId,
    pub component_type: ComponentType,
}

pub struct RegenerateResponseHandler {
    conversation_repo: Arc<dyn ConversationRepository>,
    cycle_repo: Arc<dyn CycleRepository>,
    session_repo: Arc<dyn SessionRepository>,  // For authorization
    ai_provider: Arc<dyn AIProvider>,
}

impl RegenerateResponseHandler {
    #[instrument(skip(self, cmd), fields(user_id = %cmd.user_id, cycle_id = %cmd.cycle_id))]
    pub async fn handle(&self, cmd: RegenerateResponseCommand) -> Result<Message, CommandError> {
        // 1. Load cycle
        let cycle = self.cycle_repo.find_by_id(cmd.cycle_id).await?
            .ok_or(CommandError::CycleNotFound)?;

        // 2. AUTHORIZATION: Verify user owns the session
        let session = self.session_repo.find_by_id(cycle.session_id()).await?
            .ok_or(CommandError::SessionNotFound)?;

        // SECURITY: Log authorization outcomes for audit trail
        match session.authorize(&cmd.user_id) {
            Ok(_) => {
                info!(user_id = %cmd.user_id, session_id = %session.id(), "Authorization successful for regenerate_response");
            }
            Err(e) => {
                warn!(user_id = %cmd.user_id, session_id = %session.id(), "Authorization failed for regenerate_response");
                return Err(CommandError::Unauthorized);
            }
        }

        // 3. Load conversation
        let component = cycle.get_component(cmd.component_type)
            .ok_or(CommandError::ComponentNotFound)?;
        let mut conversation = self.conversation_repo
            .find_by_component(component.id())
            .await?
            .ok_or(CommandError::ConversationNotFound)?;

        // 4. Remove last assistant message
        conversation.remove_last_assistant_message();

        // 5. Re-generate response
        let config = AgentConfig::for_component(cmd.component_type);
        let request = self.build_request(&conversation, &config);
        let response = self.ai_provider.complete(request).await?;

        // 6. Add new assistant message
        let message = conversation.add_assistant_message(&response.content);

        // 7. Persist
        self.conversation_repo.update(&conversation).await?;

        Ok(message.clone())
    }
}
```

### Queries

#### GetConversation Query

```rust
use crate::ports::{ConversationReader, CycleReader, SessionReader};

#[derive(Debug, Clone)]
pub struct GetConversationQuery {
    pub user_id: UserId,  // REQUIRED: For authorization
    pub component_id: ComponentId,
}

pub struct GetConversationHandler {
    reader: Arc<dyn ConversationReader>,
    cycle_reader: Arc<dyn CycleReader>,
    session_reader: Arc<dyn SessionReader>,
}

impl GetConversationHandler {
    #[instrument(skip(self, query), fields(user_id = %query.user_id, component_id = %query.component_id))]
    pub async fn handle(&self, query: GetConversationQuery) -> Result<ConversationView, QueryError> {
        // 1. AUTHORIZATION: Find cycle containing component
        let cycle = self.cycle_reader
            .find_by_component(query.component_id)
            .await?
            .ok_or(QueryError::CycleNotFound)?;

        // 2. Verify user owns the session
        let session = self.session_reader
            .get_by_id(cycle.session_id)
            .await?
            .ok_or(QueryError::SessionNotFound)?;

        // SECURITY: Log authorization outcomes for audit trail
        match session.authorize(&query.user_id) {
            Ok(_) => {
                info!(user_id = %query.user_id, session_id = %session.id(), "Authorization successful for get_conversation");
            }
            Err(e) => {
                warn!(user_id = %query.user_id, session_id = %session.id(), "Authorization failed for get_conversation");
                return Err(QueryError::Unauthorized);
            }
        }

        // 3. Return conversation
        self.reader
            .get_by_component(query.component_id)
            .await?
            .ok_or(QueryError::NotFound(query.component_id))
    }
}
```

---

## Adapters

### HTTP Endpoints

| Method | Path | Handler |
|--------|------|---------|
| `POST` | `/api/components/:componentId/messages` | SendMessage |
| `GET` | `/api/components/:componentId/conversation` | GetConversation |
| `POST` | `/api/components/:componentId/regenerate` | RegenerateResponse |
| `WS` | `/api/components/:componentId/stream` | StreamConversation |

### AI Provider Adapters

#### OpenAI Adapter

```rust
use async_trait::async_trait;
use reqwest::Client;
use url::Url;
use std::net::IpAddr;
use crate::ports::{AIProvider, AIError, CompletionRequest, CompletionResponse};

/// SSRF Prevention: Allowlist of permitted AI provider hosts
const ALLOWED_AI_HOSTS: &[&str] = &[
    "api.openai.com",
    "api.anthropic.com",
];

/// Allowed models to prevent model injection attacks
const ALLOWED_OPENAI_MODELS: &[&str] = &[
    "gpt-4o",
    "gpt-4-turbo",
    "gpt-4",
    "gpt-3.5-turbo",
];

pub struct OpenAIAdapter {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAIAdapter {
    /// Creates a new OpenAI adapter with SSRF protection.
    ///
    /// # Errors
    /// Returns `AIError::HostNotAllowed` if the base_url host is not in the allowlist.
    /// Returns `AIError::InvalidUrl` if the URL cannot be parsed.
    /// Returns `AIError::ModelNotAllowed` if the model is not in the allowlist.
    pub fn new(api_key: String) -> Result<Self, AIError> {
        let base_url = "https://api.openai.com/v1".to_string();
        Self::with_base_url(api_key, base_url, "gpt-4o".to_string())
    }

    /// Creates adapter with custom base URL (must be in allowlist).
    pub fn with_base_url(api_key: String, base_url: String, model: String) -> Result<Self, AIError> {
        // Validate URL and host
        let url = Url::parse(&base_url).map_err(|_| AIError::InvalidUrl)?;
        let host = url.host_str().ok_or(AIError::InvalidUrl)?;

        // SSRF Prevention: Check host against allowlist
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
            client: Client::new(),
            api_key,
            model,
            base_url,
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

    pub fn with_model(mut self, model: impl Into<String>) -> Result<Self, AIError> {
        let model = model.into();
        if !ALLOWED_OPENAI_MODELS.contains(&model.as_str()) {
            return Err(AIError::ModelNotAllowed(model));
        }
        self.model = model;
        Ok(self)
    }
}

#[async_trait]
impl AIProvider for OpenAIAdapter {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AIError> {
        let model = request.model.as_deref().unwrap_or(&self.model);

        let body = serde_json::json!({
            "model": model,
            "messages": self.format_messages(&request),
            "max_tokens": request.max_tokens,
            "temperature": request.temperature,
        });

        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| AIError::Network(e.to_string()))?;

        if response.status().as_u16() == 429 {
            let retry_after = response.headers()
                .get("retry-after")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000);
            return Err(AIError::RateLimited { retry_after_ms: retry_after });
        }

        let json: serde_json::Value = response.json().await
            .map_err(|e| AIError::Provider(e.to_string()))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let tokens_used = json["usage"]["total_tokens"]
            .as_u64()
            .unwrap_or(0) as u32;

        Ok(CompletionResponse {
            content,
            tokens_used,
            finish_reason: FinishReason::Stop,
        })
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<CompletionChunk, AIError>> + Send>>, AIError> {
        // SSE streaming implementation
        todo!()
    }
}
```

#### Anthropic Adapter

```rust
use url::Url;
use std::net::IpAddr;

/// Allowed Anthropic models to prevent model injection attacks
const ALLOWED_ANTHROPIC_MODELS: &[&str] = &[
    "claude-sonnet-4-20250514",
    "claude-3-opus-20240229",
    "claude-3-sonnet-20240229",
    "claude-3-haiku-20240307",
];

pub struct AnthropicAdapter {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl AnthropicAdapter {
    /// Creates a new Anthropic adapter with SSRF protection.
    ///
    /// # Errors
    /// Returns `AIError::HostNotAllowed` if the base_url host is not in the allowlist.
    /// Returns `AIError::ModelNotAllowed` if the model is not in the allowlist.
    pub fn new(api_key: String) -> Result<Self, AIError> {
        let base_url = "https://api.anthropic.com/v1".to_string();
        Self::with_base_url(api_key, base_url, "claude-sonnet-4-20250514".to_string())
    }

    /// Creates adapter with custom base URL (must be in allowlist).
    pub fn with_base_url(api_key: String, base_url: String, model: String) -> Result<Self, AIError> {
        // Validate URL and host
        let url = Url::parse(&base_url).map_err(|_| AIError::InvalidUrl)?;
        let host = url.host_str().ok_or(AIError::InvalidUrl)?;

        // SSRF Prevention: Check host against allowlist (uses same allowlist as OpenAI)
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
            client: Client::new(),
            api_key,
            model,
            base_url,
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
impl AIProvider for AnthropicAdapter {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AIError> {
        // Anthropic API implementation
        todo!()
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<CompletionChunk, AIError>> + Send>>, AIError> {
        todo!()
    }
}
```

#### Mock Adapter (Testing)

```rust
use std::collections::VecDeque;

pub struct MockAIAdapter {
    responses: std::sync::Mutex<VecDeque<String>>,
}

impl MockAIAdapter {
    pub fn new() -> Self {
        Self {
            responses: std::sync::Mutex::new(VecDeque::new()),
        }
    }

    pub fn queue_response(&self, response: impl Into<String>) {
        self.responses.lock().unwrap().push_back(response.into());
    }
}

#[async_trait]
impl AIProvider for MockAIAdapter {
    async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse, AIError> {
        let content = self.responses
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| "Mock response".to_string());

        Ok(CompletionResponse {
            content,
            tokens_used: 100,
            finish_reason: FinishReason::Stop,
        })
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<CompletionChunk, AIError>> + Send>>, AIError> {
        // Return single chunk with full response
        let response = self.complete(request).await?;
        let stream = futures::stream::once(async move {
            Ok(CompletionChunk {
                content: response.content,
                done: true,
            })
        });
        Ok(Box::pin(stream))
    }
}
```

### Database Schema

```sql
CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id UUID NOT NULL UNIQUE,
    component_type VARCHAR(50) NOT NULL,
    agent_state JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE conversation_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL,
    content TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Maintain message order
    sequence_num SERIAL,

    CONSTRAINT messages_role_valid CHECK (role IN ('user', 'assistant', 'system'))
);

-- Indexes
CREATE INDEX idx_conversations_component ON conversations(component_id);
CREATE INDEX idx_messages_conversation ON conversation_messages(conversation_id);
CREATE INDEX idx_messages_sequence ON conversation_messages(conversation_id, sequence_num);
```

---

## File Structure

```
backend/src/domain/conversation/
├── mod.rs                  # Module exports
├── conversation.rs         # Conversation entity
├── conversation_test.rs
├── agent_state.rs          # AgentState value object
├── agent_config.rs         # AgentConfig and related types
├── agent_configs/          # Per-component configurations
│   ├── mod.rs
│   ├── issue_raising.rs
│   ├── problem_frame.rs
│   ├── objectives.rs
│   ├── alternatives.rs
│   ├── consequences.rs
│   ├── tradeoffs.rs
│   ├── recommendation.rs
│   ├── decision_quality.rs
│   └── notes_next_steps.rs
├── prompts/                # System prompts
│   ├── issue_raising.txt
│   └── ...
└── errors.rs

backend/src/ports/
├── ai_provider.rs          # AIProvider trait
├── conversation_repository.rs
├── conversation_reader.rs
├── connection_registry.rs  # Multi-server WebSocket tracking
└── circuit_breaker.rs      # External service resilience

backend/src/application/
├── commands/
│   ├── send_message.rs
│   ├── send_message_test.rs
│   ├── stream_message.rs
│   └── regenerate_response.rs
└── queries/
    └── get_conversation.rs

backend/src/adapters/
├── ai/
│   ├── mod.rs
│   ├── openai_adapter.rs
│   ├── openai_adapter_test.rs
│   ├── anthropic_adapter.rs
│   ├── mock_adapter.rs
│   └── resilient_ai_provider.rs  # Circuit breaker wrapper
├── http/conversation/
│   ├── handlers.rs
│   ├── websocket_handler.rs
│   ├── dto.rs
│   └── routes.rs
├── postgres/
│   ├── conversation_repository.rs
│   └── conversation_reader.rs
└── redis/
    ├── connection_registry.rs    # Multi-server WebSocket tracking
    ├── cross_server_messenger.rs # Redis pub/sub message delivery
    └── circuit_breaker.rs        # Circuit breaker implementation

frontend/src/modules/conversation/
├── domain/
│   ├── conversation.ts
│   └── agent-state.ts
├── api/
│   ├── conversation-api.ts
│   ├── use-conversation.ts
│   └── use-streaming.ts
├── components/
│   ├── ChatInterface.tsx
│   ├── ChatInterface.test.tsx
│   ├── MessageBubble.tsx
│   ├── TypingIndicator.tsx
│   └── InputArea.tsx
└── index.ts
```

---

## Invariants

| Invariant | Enforcement |
|-----------|-------------|
| One conversation per component | UNIQUE constraint on component_id |
| Messages ordered by time | sequence_num column |
| Messages are append-only | append_message() method |
| Valid message roles | CHECK constraint in DB |
| Agent state is always valid JSON | JSONB type |
| Connection tracking survives server crashes | Redis TTL with heartbeat |
| AI provider failures isolated | Circuit breaker pattern |

---

## Scaling Considerations

### Multi-Server WebSocket Architecture

In a multi-server deployment, WebSocket connections are inherently tied to specific servers. The conversation module uses the ConnectionRegistry port to enable cross-server message delivery.

**Problem Solved:**
```
User connects to Server A
Event occurs on Server B (AI response completed)
Server B needs to push message to User
→ Server B queries ConnectionRegistry to find User is on Server A
→ Server B publishes message to Redis channel for Server A
→ Server A receives and delivers to User's WebSocket
```

**Architecture:**
```
┌─────────────────────────────────────────────────────────────────┐
│                        Load Balancer                             │
│                    (sticky sessions by user_id)                  │
└─────────────────────────────────────────────────────────────────┘
         │                    │                    │
    ┌────▼────┐          ┌────▼────┐          ┌────▼────┐
    │Server A │          │Server B │          │Server C │
    │  WS     │          │  WS     │          │  WS     │
    │Conns:   │          │Conns:   │          │Conns:   │
    │ User1   │          │ User2   │          │ User3   │
    │ User4   │          │ User5   │          │ User6   │
    └────┬────┘          └────┬────┘          └────┬────┘
         │                    │                    │
         └────────────────────┼────────────────────┘
                              │
                    ┌─────────▼─────────┐
                    │      Redis        │
                    │ - Connection      │
                    │   Registry        │
                    │ - Pub/Sub         │
                    │   channels        │
                    └───────────────────┘
```

### RedisConnectionRegistry Adapter

```rust
use redis::aio::ConnectionManager;
use std::time::Duration;

const CONNECTION_TTL_SECS: u64 = 60;  // 1 minute TTL, refreshed by heartbeat

pub struct RedisConnectionRegistry {
    redis: ConnectionManager,
    server_id: ServerId,
}

#[async_trait]
impl ConnectionRegistry for RedisConnectionRegistry {
    async fn register(
        &self,
        user_id: &UserId,
        server_id: &ServerId,
    ) -> Result<(), ConnectionRegistryError> {
        // Store in Redis Set with TTL
        // Key: user_connections:{user_id}
        // Value: server_id
        let key = format!("user_connections:{}", user_id);

        redis::pipe()
            .atomic()
            .sadd(&key, server_id.as_str())
            .expire(&key, CONNECTION_TTL_SECS as i64)
            .query_async(&mut self.redis.clone())
            .await
            .map_err(|e| ConnectionRegistryError::Redis(e.to_string()))?;

        Ok(())
    }

    async fn heartbeat(
        &self,
        user_id: &UserId,
        server_id: &ServerId,
    ) -> Result<(), ConnectionRegistryError> {
        // Refresh TTL
        let key = format!("user_connections:{}", user_id);
        redis::cmd("EXPIRE")
            .arg(&key)
            .arg(CONNECTION_TTL_SECS)
            .query_async(&mut self.redis.clone())
            .await
            .map_err(|e| ConnectionRegistryError::Redis(e.to_string()))?;
        Ok(())
    }

    async fn find_servers(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<ServerId>, ConnectionRegistryError> {
        let key = format!("user_connections:{}", user_id);
        let servers: Vec<String> = redis::cmd("SMEMBERS")
            .arg(&key)
            .query_async(&mut self.redis.clone())
            .await
            .map_err(|e| ConnectionRegistryError::Redis(e.to_string()))?;

        Ok(servers.into_iter().map(ServerId::new).collect())
    }

    // ... other methods
}
```

### Cross-Server Message Delivery

```rust
/// Delivers messages to users across multiple servers via Redis pub/sub.
pub struct CrossServerMessenger {
    redis: ConnectionManager,
    connection_registry: Arc<dyn ConnectionRegistry>,
}

impl CrossServerMessenger {
    /// Send a message to a user, routing to the correct server(s).
    pub async fn send_to_user(
        &self,
        user_id: &UserId,
        message: &WebSocketMessage,
    ) -> Result<(), MessengerError> {
        // Find servers with user's connections
        let servers = self.connection_registry.find_servers(user_id).await?;

        if servers.is_empty() {
            // User not connected - message will be available on reconnect
            return Ok(());
        }

        // Publish to each server's channel
        let payload = serde_json::to_string(message)?;
        for server_id in servers {
            let channel = format!("ws:server:{}", server_id.as_str());
            redis::cmd("PUBLISH")
                .arg(&channel)
                .arg(&payload)
                .query_async(&mut self.redis.clone())
                .await
                .map_err(|e| MessengerError::Redis(e.to_string()))?;
        }

        Ok(())
    }
}
```

### ResilientAIProvider Wrapper

Wraps any AIProvider with circuit breaker protection and retry logic:

```rust
pub struct ResilientAIProvider {
    inner: Arc<dyn AIProvider>,
    circuit_breaker: Arc<dyn CircuitBreaker>,
    retry_policy: RetryPolicy,
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

#[async_trait]
impl AIProvider for ResilientAIProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AIError> {
        // Check circuit breaker
        if !self.circuit_breaker.should_allow() {
            return Err(AIError::ModelUnavailable(
                "Circuit breaker open - AI provider temporarily unavailable".to_string()
            ));
        }

        let mut attempt = 0;
        let mut delay = self.retry_policy.initial_delay_ms;

        loop {
            attempt += 1;

            match self.inner.complete(request.clone()).await {
                Ok(response) => {
                    self.circuit_breaker.record_success();
                    return Ok(response);
                }
                Err(e) => {
                    self.circuit_breaker.record_failure();

                    // Don't retry on certain errors
                    if matches!(e, AIError::ContentFiltered(_) | AIError::TokenLimitExceeded { .. }) {
                        return Err(e);
                    }

                    // Retry on transient errors
                    if attempt >= self.retry_policy.max_attempts {
                        return Err(e);
                    }

                    // Wait before retry with exponential backoff
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                    delay = (delay as f64 * self.retry_policy.backoff_multiplier) as u64;
                    delay = delay.min(self.retry_policy.max_delay_ms);
                }
            }
        }
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<CompletionChunk, AIError>> + Send>>, AIError> {
        // Check circuit breaker (streams don't retry)
        if !self.circuit_breaker.should_allow() {
            return Err(AIError::ModelUnavailable(
                "Circuit breaker open - AI provider temporarily unavailable".to_string()
            ));
        }

        match self.inner.stream(request).await {
            Ok(stream) => {
                self.circuit_breaker.record_success();
                Ok(stream)
            }
            Err(e) => {
                self.circuit_breaker.record_failure();
                Err(e)
            }
        }
    }
}
```

### Client Reconnection Protocol

```typescript
// frontend/src/lib/websocket/reconnect.ts

export class ReconnectingWebSocket {
    private ws: WebSocket | null = null;
    private reconnectAttempts = 0;
    private maxReconnectAttempts = 10;
    private baseDelay = 1000;  // 1 second
    private maxDelay = 30000;  // 30 seconds

    connect(url: string): void {
        this.ws = new WebSocket(url);

        this.ws.onopen = () => {
            this.reconnectAttempts = 0;
            // Optionally request any missed messages
            this.requestMissedMessages();
        };

        this.ws.onclose = (event) => {
            if (!event.wasClean && this.reconnectAttempts < this.maxReconnectAttempts) {
                this.scheduleReconnect(url);
            }
        };

        this.ws.onerror = () => {
            // Will trigger onclose
        };
    }

    private scheduleReconnect(url: string): void {
        const delay = Math.min(
            this.baseDelay * Math.pow(2, this.reconnectAttempts),
            this.maxDelay
        );

        // Add jitter (±25%)
        const jitter = delay * (0.75 + Math.random() * 0.5);

        this.reconnectAttempts++;
        setTimeout(() => this.connect(url), jitter);
    }

    private requestMissedMessages(): void {
        // Request messages since last received timestamp
        this.send({ type: 'sync', since: this.lastMessageTimestamp });
    }
}
```

See [SCALING-READINESS.md](../architecture/SCALING-READINESS.md) for full scaling architecture.

---

## Test Categories

### Unit Tests (Domain)

| Category | Example Tests |
|----------|---------------|
| Creation | `new_conversation_starts_empty` |
| Messages | `add_user_message_increments_count` |
| Messages | `last_assistant_message_finds_correct_one` |
| Context | `get_context_messages_limits_to_max` |
| Regenerate | `remove_last_assistant_removes_correct_one` |
| State | `advance_phase_updates_current_phase` |

### Integration Tests

| Category | Example Tests |
|----------|---------------|
| AI Provider | `openai_adapter_handles_rate_limit` |
| Repository | `save_persists_all_messages` |
| Streaming | `stream_accumulates_full_response` |

---

*Module Version: 1.1.0*
*Based on: SYSTEM-ARCHITECTURE.md v1.3.0*
*Language: Rust*
*Updated: 2026-01-08 (WebSocket Scaling, AI Provider Resilience)*
