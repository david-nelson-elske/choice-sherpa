# AI Engine Module Specification

## Overview

The AI Engine module provides conversational AI capabilities for guiding users through PrOACT decision components. It is designed as a **port-based abstraction** enabling multiple AI backends (Claude Code, OpenAI API, Anthropic API, etc.) to be swapped without affecting domain logic.

---

## Module Classification

| Attribute | Value |
|-----------|-------|
| **Type** | Full Module (Ports + Adapters) |
| **Language** | Rust |
| **Responsibility** | AI-powered conversation orchestration for PrOACT steps |
| **Domain Dependencies** | foundation, proact-types, session, cycle |
| **External Dependencies** | AI providers (Claude Code CLI, OpenAI API, Anthropic API) |

---

## Architecture

### Hexagonal Structure

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            AI ENGINE MODULE                              │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                         DOMAIN LAYER                                │ │
│  │                                                                     │ │
│  │   ┌─────────────────┐  ┌─────────────────┐  ┌──────────────────┐  │ │
│  │   │ Orchestrator    │  │ StepAgent       │  │ ConversationState│  │ │
│  │   │ (flow control)  │  │ (step behavior) │  │ (context mgmt)   │  │ │
│  │   └─────────────────┘  └─────────────────┘  └──────────────────┘  │ │
│  │                                                                     │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                    │                                     │
│                                    ▼                                     │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                          PORT LAYER                                 │ │
│  │                                                                     │ │
│  │   ┌─────────────────────────────────────────────────────────────┐  │ │
│  │   │                    AIEnginePort                              │  │ │
│  │   │  - start_session(cycle_id, config) → SessionHandle          │  │ │
│  │   │  - send_message(handle, message) → Stream<Response>         │  │ │
│  │   │  - get_state(handle) → ConversationState                    │  │ │
│  │   │  - end_session(handle)                                      │  │ │
│  │   └─────────────────────────────────────────────────────────────┘  │ │
│  │                                                                     │ │
│  │   ┌─────────────────────────────────────────────────────────────┐  │ │
│  │   │                 StepAgentPort                                │  │ │
│  │   │  - get_system_prompt(component) → String                    │  │ │
│  │   │  - get_tools(component) → Vec<ToolDefinition>               │  │ │
│  │   │  - parse_output(component, response) → StructuredOutput     │  │ │
│  │   └─────────────────────────────────────────────────────────────┘  │ │
│  │                                                                     │ │
│  │   ┌─────────────────────────────────────────────────────────────┐  │ │
│  │   │               StateStoragePort                               │  │ │
│  │   │  - save_state(cycle_id, state) → Result<()>                 │  │ │
│  │   │  - load_state(cycle_id) → Result<ConversationState>         │  │ │
│  │   │  - save_step_output(cycle_id, output) → Result<()>          │  │ │
│  │   │  - load_step_output(cycle_id, component) → StructuredOutput │  │ │
│  │   └─────────────────────────────────────────────────────────────┘  │ │
│  │                                                                     │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                    │                                     │
│                                    ▼                                     │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                        ADAPTER LAYER                                │ │
│  │                                                                     │ │
│  │   ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐ │ │
│  │   │ ClaudeCode   │  │ OpenAI API   │  │ Anthropic API            │ │ │
│  │   │ Adapter      │  │ Adapter      │  │ Adapter                  │ │ │
│  │   │              │  │              │  │                          │ │ │
│  │   │ - Process    │  │ - HTTP       │  │ - HTTP                   │ │ │
│  │   │   manager    │  │ - Streaming  │  │ - Streaming              │ │ │
│  │   │ - Skills     │  │ - Functions  │  │ - Tools                  │ │ │
│  │   │ - Subagents  │  │              │  │                          │ │ │
│  │   └──────────────┘  └──────────────┘  └──────────────────────────┘ │ │
│  │                                                                     │ │
│  │   ┌──────────────────────────────────────────────────────────────┐ │ │
│  │   │ FileStateStorage │  │ PostgresStateStorage (index/cache)    │ │ │
│  │   └──────────────────────────────────────────────────────────────┘ │ │
│  │                                                                     │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Domain Layer

### Core Entities

#### Orchestrator

Manages the flow through PrOACT components within a cycle. Pure domain logic—no AI provider knowledge.

```rust
// domain/ai_engine/orchestrator.rs

use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Manages PrOACT flow within a decision cycle
pub struct Orchestrator {
    cycle_id: CycleId,
    current_step: ProactComponent,
    completed_steps: HashMap<ProactComponent, StepSummary>,
    state: ConversationState,
}

impl Orchestrator {
    /// Create a new orchestrator for a cycle
    pub fn new(cycle_id: CycleId, initial_step: ProactComponent) -> Self;

    /// Resume from persisted state
    pub fn from_state(state: ConversationState) -> Result<Self, OrchestratorError>;

    /// Route user intent to appropriate step
    pub fn route(&self, intent: UserIntent) -> Result<ProactComponent, OrchestratorError>;

    /// Check if transition to a new step is valid
    pub fn can_transition(&self, to: ProactComponent) -> bool;

    /// Transition to a new step
    pub fn transition_to(&mut self, step: ProactComponent) -> Result<(), OrchestratorError>;

    /// Record completion of current step
    pub fn record_completion(&mut self, summary: StepSummary) -> Result<(), OrchestratorError>;

    /// Get context needed for a step agent
    pub fn context_for_step(&self, step: ProactComponent) -> StepContext;

    /// Export current state for persistence
    pub fn to_state(&self) -> ConversationState;
}

#[derive(Debug, thiserror::Error)]
pub enum OrchestratorError {
    #[error("Invalid transition from {from:?} to {to:?}")]
    InvalidTransition { from: ProactComponent, to: ProactComponent },

    #[error("Step {0:?} not yet completed")]
    StepNotCompleted(ProactComponent),

    #[error("Cycle already completed")]
    CycleCompleted,
}
```

#### StepAgent

Defines behavior for each PrOACT component. Provider-agnostic specification.

```rust
// domain/ai_engine/step_agent.rs

/// Specification for a PrOACT step agent
#[derive(Debug, Clone)]
pub struct StepAgentSpec {
    pub component: ProactComponent,
    pub role: String,
    pub objectives: Vec<String>,
    pub techniques: Vec<String>,
    pub output_schema: OutputSchema,
    pub transitions: TransitionRules,
}

/// Rules for when a step can transition
#[derive(Debug, Clone)]
pub struct TransitionRules {
    pub min_turns: u32,
    pub required_outputs: Vec<String>,
    pub completion_signals: Vec<String>,
}

/// Schema for structured output
#[derive(Debug, Clone)]
pub struct OutputSchema {
    pub schema_version: String,
    pub fields: Vec<SchemaField>,
}

/// Predefined step agent specifications
pub mod agents {
    use super::*;

    pub fn issue_raising() -> StepAgentSpec;
    pub fn problem_frame() -> StepAgentSpec;
    pub fn objectives() -> StepAgentSpec;
    pub fn alternatives() -> StepAgentSpec;
    pub fn consequences() -> StepAgentSpec;
    pub fn tradeoffs() -> StepAgentSpec;
    pub fn recommendation() -> StepAgentSpec;
    pub fn decision_quality() -> StepAgentSpec;

    pub fn all() -> Vec<StepAgentSpec> {
        vec![
            issue_raising(),
            problem_frame(),
            objectives(),
            alternatives(),
            consequences(),
            tradeoffs(),
            recommendation(),
            decision_quality(),
        ]
    }
}
```

#### ConversationState

Tracks context across the session, independent of provider.

```rust
// domain/ai_engine/conversation_state.rs

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Complete state of a conversation within a cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationState {
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    pub current_step: ProactComponent,
    pub status: CycleStatus,
    pub branch_info: Option<BranchInfo>,
    pub step_states: HashMap<ProactComponent, StepState>,
    pub message_history: Vec<Message>,
    pub compressed_context: Option<CompressedContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepState {
    pub status: StepStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub turn_count: u32,
    pub summary: Option<String>,
    pub key_outputs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    NotStarted,
    InProgress,
    Completed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub step_context: ProactComponent,
    pub metadata: Option<MessageMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub parent_cycle: CycleId,
    pub branch_point: ProactComponent,
    pub branch_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedContext {
    pub summary: String,
    pub token_estimate: u32,
    pub compressed_at: DateTime<Utc>,
}
```

### Value Objects

```rust
// domain/ai_engine/values.rs

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// User's intent derived from their message
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserIntent {
    /// Continue working on current step
    Continue,
    /// Navigate to a specific step
    Navigate(ProactComponent),
    /// Create an alternate cycle branch
    Branch,
    /// Request summary of current state
    Summarize,
    /// Signal completion of current step
    Complete,
}

/// Summary of a completed step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepSummary {
    pub component: ProactComponent,
    pub summary: String,
    pub key_outputs: Vec<String>,
    pub conflicts: Vec<String>,
    pub completed_at: DateTime<Utc>,
}

/// Context passed to a step agent
#[derive(Debug, Clone)]
pub struct StepContext {
    pub component: ProactComponent,
    pub prior_summaries: Vec<StepSummary>,
    pub relevant_outputs: HashMap<ProactComponent, Box<dyn StructuredOutput>>,
}

/// Trait for structured output from any step
pub trait StructuredOutput: Send + Sync {
    fn component(&self) -> ProactComponent;
    fn validate(&self) -> Result<(), ValidationError>;
    fn to_yaml(&self) -> Result<String, SerializationError>;
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Cycle status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CycleStatus {
    Draft,
    InProgress,
    Completed,
    Abandoned,
}
```

### Domain Services

```rust
// domain/ai_engine/services.rs

use async_trait::async_trait;

/// Classifies user intent from message content
pub trait IntentClassifier: Send + Sync {
    fn classify(&self, message: &str, current_step: ProactComponent) -> UserIntent;
}

/// Compresses conversation history for token efficiency
#[async_trait]
pub trait ContextCompressor: Send + Sync {
    async fn compress(&self, messages: &[Message]) -> Result<CompressedContext, CompressionError>;
}

/// Extracts structured output from AI responses
pub trait OutputExtractor: Send + Sync {
    fn extract(
        &self,
        response: &str,
        component: ProactComponent,
    ) -> Result<Box<dyn StructuredOutput>, ExtractionError>;
}

#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
    #[error("AI service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ExtractionError {
    #[error("Invalid response format: {0}")]
    InvalidFormat(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
}
```

---

## Port Layer

### AIEnginePort (Primary Port)

The core abstraction for AI conversation management. All providers implement this trait.

```rust
// ports/ai_engine.rs

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

/// Primary port for AI conversation engines
///
/// This trait abstracts over different AI providers (Claude Code, OpenAI, Anthropic).
/// Each provider implements this trait, allowing the application to switch providers
/// without changing domain logic.
#[async_trait]
pub trait AIEnginePort: Send + Sync {
    /// Start a new AI session for a cycle
    async fn start_session(
        &self,
        config: SessionConfig,
    ) -> Result<SessionHandle, AIEngineError>;

    /// Resume an existing session
    async fn resume_session(
        &self,
        cycle_id: CycleId,
    ) -> Result<SessionHandle, AIEngineError>;

    /// End a session and clean up resources
    async fn end_session(
        &self,
        handle: &SessionHandle,
    ) -> Result<(), AIEngineError>;

    /// Send a message and receive streaming response
    async fn send_message(
        &self,
        handle: &SessionHandle,
        message: UserMessage,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, AIEngineError>> + Send>>, AIEngineError>;

    /// Get current conversation state
    async fn get_state(
        &self,
        handle: &SessionHandle,
    ) -> Result<ConversationState, AIEngineError>;

    /// Get structured output from a completed step
    async fn get_step_output(
        &self,
        handle: &SessionHandle,
        component: ProactComponent,
    ) -> Result<Box<dyn StructuredOutput>, AIEngineError>;
}

/// Configuration for starting a session
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    pub working_dir: PathBuf,
    pub initial_step: ProactComponent,
    pub prior_context: Option<StepContext>,
    pub provider_config: ProviderConfig,
}

/// Provider-specific configuration
#[derive(Debug, Clone)]
pub enum ProviderConfig {
    ClaudeCode(ClaudeCodeConfig),
    OpenAI(OpenAIConfig),
    Anthropic(AnthropicConfig),
}

#[derive(Debug, Clone)]
pub struct ClaudeCodeConfig {
    pub binary_path: PathBuf,
    pub skills_dir: PathBuf,
    pub agents_dir: PathBuf,
    pub timeout: Duration,
}

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

/// Allowed Anthropic models to prevent model injection attacks
const ALLOWED_ANTHROPIC_MODELS: &[&str] = &[
    "claude-sonnet-4-20250514",
    "claude-3-opus-20240229",
    "claude-3-sonnet-20240229",
    "claude-3-haiku-20240307",
];

#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub base_url: Option<String>,
}

impl OpenAIConfig {
    /// Validates the configuration for SSRF and model injection attacks.
    ///
    /// # Errors
    /// Returns error if:
    /// - base_url host is not in ALLOWED_AI_HOSTS
    /// - base_url resolves to a private/internal IP
    /// - model is not in ALLOWED_OPENAI_MODELS
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate base URL if provided
        if let Some(ref base_url) = self.base_url {
            let url = url::Url::parse(base_url)
                .map_err(|_| ConfigError::InvalidUrl)?;
            let host = url.host_str()
                .ok_or(ConfigError::InvalidUrl)?;

            // SSRF Prevention: Check host against allowlist
            if !ALLOWED_AI_HOSTS.contains(&host) {
                return Err(ConfigError::HostNotAllowed(host.to_string()));
            }

            // Block private/internal IPs
            if let Ok(ip) = host.parse::<std::net::IpAddr>() {
                if is_private_ip(&ip) {
                    return Err(ConfigError::HostNotAllowed(
                        format!("Private IP addresses not allowed: {}", ip)
                    ));
                }
            }
        }

        // Validate model against allowlist
        if !ALLOWED_OPENAI_MODELS.contains(&self.model.as_str()) {
            return Err(ConfigError::ModelNotAllowed(self.model.clone()));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub base_url: Option<String>,
}

impl AnthropicConfig {
    /// Validates the configuration for SSRF and model injection attacks.
    ///
    /// # Errors
    /// Returns error if:
    /// - base_url host is not in ALLOWED_AI_HOSTS
    /// - base_url resolves to a private/internal IP
    /// - model is not in ALLOWED_ANTHROPIC_MODELS
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate base URL if provided
        if let Some(ref base_url) = self.base_url {
            let url = url::Url::parse(base_url)
                .map_err(|_| ConfigError::InvalidUrl)?;
            let host = url.host_str()
                .ok_or(ConfigError::InvalidUrl)?;

            // SSRF Prevention: Check host against allowlist
            if !ALLOWED_AI_HOSTS.contains(&host) {
                return Err(ConfigError::HostNotAllowed(host.to_string()));
            }

            // Block private/internal IPs
            if let Ok(ip) = host.parse::<std::net::IpAddr>() {
                if is_private_ip(&ip) {
                    return Err(ConfigError::HostNotAllowed(
                        format!("Private IP addresses not allowed: {}", ip)
                    ));
                }
            }
        }

        // Validate model against allowlist
        if !ALLOWED_ANTHROPIC_MODELS.contains(&self.model.as_str()) {
            return Err(ConfigError::ModelNotAllowed(self.model.clone()));
        }

        Ok(())
    }
}

/// Configuration validation errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid URL format")]
    InvalidUrl,

    #[error("Host not allowed: {0}")]
    HostNotAllowed(String),

    #[error("Model not allowed: {0}")]
    ModelNotAllowed(String),
}

/// Check if an IP address is in a private/internal range
fn is_private_ip(ip: &std::net::IpAddr) -> bool {
    use std::net::IpAddr;
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

/// Handle to an active session
#[derive(Debug, Clone)]
pub struct SessionHandle {
    pub id: String,
    pub cycle_id: CycleId,
    pub provider: ProviderType,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    ClaudeCode,
    OpenAI,
    Anthropic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Starting,
    Ready,
    Busy,
    Idle,
    Ended,
}

/// User message to send to AI
#[derive(Debug, Clone)]
pub struct UserMessage {
    pub content: String,
    pub attachments: Vec<Attachment>,
}

/// Chunk of streaming response
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub chunk_type: ChunkType,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkType {
    Text,
    ToolCall,
    ToolResult,
    StepComplete,
    Error,
}

#[derive(Debug, thiserror::Error)]
pub enum AIEngineError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Storage error: {0}")]
    StorageError(#[from] StateStorageError),
}
```

### StepAgentPort (Secondary Port)

Provides step-specific prompts and tools to the AI engine.

```rust
// ports/step_agent.rs

use async_trait::async_trait;

/// Port for step-specific agent behavior
///
/// This trait provides the prompts, tools, and parsing logic specific to each
/// PrOACT component. The implementation can load from files or be hardcoded.
#[async_trait]
pub trait StepAgentPort: Send + Sync {
    /// Get the system prompt for a component
    fn get_system_prompt(
        &self,
        component: ProactComponent,
        context: &StepContext,
    ) -> String;

    /// Get transition prompt when moving between steps
    fn get_transition_prompt(
        &self,
        from: ProactComponent,
        to: ProactComponent,
        summary: &StepSummary,
    ) -> String;

    /// Get tool definitions for a component (provider-agnostic)
    fn get_tools(&self, component: ProactComponent) -> Vec<ToolDefinition>;

    /// Parse AI response into structured output
    fn parse_response(
        &self,
        component: ProactComponent,
        response: &str,
    ) -> Result<Box<dyn StructuredOutput>, ParseError>;

    /// Validate structured output
    fn validate_output(
        &self,
        output: &dyn StructuredOutput,
    ) -> Vec<ValidationError>;
}

/// Provider-agnostic tool definition
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON Schema
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid JSON: {0}")]
    InvalidJson(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid field value for {field}: {message}")]
    InvalidField { field: String, message: String },
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Warning,
    Error,
}
```

### StateStoragePort (Secondary Port)

Abstracts state persistence—files, database, or hybrid.

```rust
// ports/state_storage.rs

use async_trait::async_trait;

/// Port for conversation state persistence
///
/// Supports file-based storage (source of truth), database indexing,
/// or hybrid approaches. Implementations handle serialization.
#[async_trait]
pub trait StateStoragePort: Send + Sync {
    // --- Conversation State ---

    /// Save complete conversation state
    async fn save_state(
        &self,
        cycle_id: &CycleId,
        state: &ConversationState,
    ) -> Result<(), StateStorageError>;

    /// Load conversation state
    async fn load_state(
        &self,
        cycle_id: &CycleId,
    ) -> Result<ConversationState, StateStorageError>;

    /// Check if state exists
    async fn state_exists(&self, cycle_id: &CycleId) -> Result<bool, StateStorageError>;

    // --- Step Outputs ---

    /// Save structured output from a step
    async fn save_step_output(
        &self,
        cycle_id: &CycleId,
        output: &dyn StructuredOutput,
    ) -> Result<(), StateStorageError>;

    /// Load structured output for a step
    async fn load_step_output(
        &self,
        cycle_id: &CycleId,
        component: ProactComponent,
    ) -> Result<Box<dyn StructuredOutput>, StateStorageError>;

    // --- Message History ---

    /// Append a message to history
    async fn append_message(
        &self,
        cycle_id: &CycleId,
        message: &Message,
    ) -> Result<(), StateStorageError>;

    /// Get recent messages
    async fn get_messages(
        &self,
        cycle_id: &CycleId,
        limit: usize,
    ) -> Result<Vec<Message>, StateStorageError>;

    // --- Search (optional, for database implementations) ---

    /// Search across decisions (returns empty if not supported)
    async fn search_decisions(
        &self,
        query: &str,
        session_filter: Option<&SessionId>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, StateStorageError>;
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub session_id: SessionId,
    pub cycle_id: CycleId,
    pub component: ProactComponent,
    pub snippet: String,
    pub relevance_score: f32,
}

#[derive(Debug, thiserror::Error)]
pub enum StateStorageError {
    #[error("State not found for cycle: {0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}
```

---

## Adapter Layer

### Claude Code Adapter

Manages Claude Code as a child process with skills and subagents.

```rust
// adapters/claude_code/adapter.rs

use async_trait::async_trait;
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Claude Code adapter - spawns CLI processes for each session
pub struct ClaudeCodeAdapter {
    process_manager: ProcessManager,
    config: ClaudeCodeConfig,
    state_storage: Arc<dyn StateStoragePort>,
    step_agents: Arc<dyn StepAgentPort>,
}

impl ClaudeCodeAdapter {
    pub fn new(
        config: ClaudeCodeConfig,
        state_storage: Arc<dyn StateStoragePort>,
        step_agents: Arc<dyn StepAgentPort>,
    ) -> Self {
        Self {
            process_manager: ProcessManager::new(),
            config,
            state_storage,
            step_agents,
        }
    }
}

#[async_trait]
impl AIEnginePort for ClaudeCodeAdapter {
    async fn start_session(
        &self,
        config: SessionConfig,
    ) -> Result<SessionHandle, AIEngineError> {
        // 1. Prepare working directory with state files
        self.prepare_working_dir(&config).await?;

        // 2. Spawn Claude Code process with orchestrator skill
        let session = self.process_manager.spawn(SpawnConfig {
            cycle_id: config.cycle_id.clone(),
            working_dir: config.working_dir.clone(),
            skill: "decision-orchestrate".to_string(),
            context_file: Some("state.yaml".into()),
        }).await?;

        // 3. Return handle
        Ok(SessionHandle {
            id: session.id.clone(),
            cycle_id: config.cycle_id,
            provider: ProviderType::ClaudeCode,
            status: SessionStatus::Ready,
        })
    }

    async fn send_message(
        &self,
        handle: &SessionHandle,
        message: UserMessage,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, AIEngineError>> + Send>>, AIEngineError> {
        // 1. Get managed session
        let session = self.process_manager.get(&handle.id).await?;

        // 2. Write to stdin
        session.send(&message.content).await?;

        // 3. Create stream from stdout
        let stream = session.output_stream();

        // 4. Parse and transform stream
        Ok(Box::pin(stream.map(|line| {
            self.parse_output_line(line)
        })))
    }

    // ... other trait methods
}
```

#### Process Manager

```rust
// adapters/claude_code/process_manager.rs

use tokio::process::{Child, Command, ChildStdin, ChildStdout};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc, RwLock};
use std::collections::HashMap;

/// Manages Claude Code child processes
pub struct ProcessManager {
    sessions: RwLock<HashMap<String, ManagedSession>>,
}

pub struct ManagedSession {
    pub id: String,
    pub cycle_id: CycleId,
    pub process: Child,
    pub stdin: ChildStdin,
    pub stdout_rx: mpsc::Receiver<String>,
    pub working_dir: PathBuf,
    pub status: SessionStatus,
    pub last_active: Instant,
}

pub struct SpawnConfig {
    pub cycle_id: CycleId,
    pub working_dir: PathBuf,
    pub skill: String,
    pub context_file: Option<PathBuf>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// Spawn a new Claude Code process
    pub async fn spawn(&self, config: SpawnConfig) -> Result<ManagedSession, ProcessError> {
        let mut cmd = Command::new("claude");
        cmd.current_dir(&config.working_dir)
            .arg("--dangerously-skip-permissions")
            .arg("--print")
            .arg("--skill")
            .arg(&config.skill)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        if let Some(ref ctx_file) = config.context_file {
            cmd.arg("--context-file").arg(ctx_file);
        }

        let mut child = cmd.spawn()?;
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        // Spawn task to read stdout into channel
        let (tx, rx) = mpsc::channel(100);
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tx.send(line).await.is_err() {
                    break;
                }
            }
        });

        let session = ManagedSession {
            id: uuid::Uuid::new_v4().to_string(),
            cycle_id: config.cycle_id,
            process: child,
            stdin,
            stdout_rx: rx,
            working_dir: config.working_dir,
            status: SessionStatus::Ready,
            last_active: Instant::now(),
        };

        let id = session.id.clone();
        self.sessions.write().await.insert(id.clone(), session);

        Ok(self.sessions.read().await.get(&id).unwrap().clone())
    }

    /// Send input to a session
    pub async fn send(&self, session_id: &str, input: &str) -> Result<(), ProcessError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| ProcessError::SessionNotFound(session_id.to_string()))?;

        session.stdin.write_all(input.as_bytes()).await?;
        session.stdin.write_all(b"\n").await?;
        session.stdin.flush().await?;
        session.last_active = Instant::now();

        Ok(())
    }

    /// Kill a session
    pub async fn kill(&self, session_id: &str) -> Result<(), ProcessError> {
        let mut sessions = self.sessions.write().await;
        if let Some(mut session) = sessions.remove(session_id) {
            session.process.kill().await?;
        }
        Ok(())
    }

    /// Clean up idle sessions
    pub async fn cleanup_idle(&self, max_idle: Duration) -> usize {
        let mut sessions = self.sessions.write().await;
        let now = Instant::now();
        let mut removed = 0;

        sessions.retain(|_, session| {
            if now.duration_since(session.last_active) > max_idle {
                let _ = session.process.start_kill();
                removed += 1;
                false
            } else {
                true
            }
        });

        removed
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Process spawn failed: {0}")]
    SpawnFailed(String),
}
```

#### Claude Code Skills Structure

```
.claude/
├── skills/
│   └── decision-orchestrate.md       # Main orchestrator
│
└── agents/
    ├── proact-issue-raising.md
    ├── proact-problem-frame.md
    ├── proact-objectives.md
    ├── proact-alternatives.md
    ├── proact-consequences.md
    ├── proact-tradeoffs.md
    ├── proact-recommendation.md
    └── proact-decision-quality.md
```

### OpenAI API Adapter

Standard API-based implementation using chat completions.

```rust
// adapters/openai/adapter.rs

use async_openai::{Client, config::OpenAIConfig as ApiConfig};
use async_openai::types::{
    ChatCompletionRequestMessage, CreateChatCompletionRequestArgs,
};
use async_trait::async_trait;

/// OpenAI adapter - uses Chat Completions API
pub struct OpenAIAdapter {
    client: Client<ApiConfig>,
    model: String,
    state_storage: Arc<dyn StateStoragePort>,
    step_agents: Arc<dyn StepAgentPort>,
    orchestrator: Orchestrator, // Orchestration logic lives here for API adapters
}

impl OpenAIAdapter {
    pub fn new(
        config: OpenAIConfig,
        state_storage: Arc<dyn StateStoragePort>,
        step_agents: Arc<dyn StepAgentPort>,
    ) -> Self {
        let api_config = ApiConfig::new().with_api_key(&config.api_key);
        Self {
            client: Client::with_config(api_config),
            model: config.model,
            state_storage,
            step_agents,
            orchestrator: Orchestrator::new(), // Will be initialized per session
        }
    }
}

#[async_trait]
impl AIEnginePort for OpenAIAdapter {
    async fn start_session(
        &self,
        config: SessionConfig,
    ) -> Result<SessionHandle, AIEngineError> {
        // 1. Load or create conversation state
        let state = match self.state_storage.load_state(&config.cycle_id).await {
            Ok(state) => state,
            Err(StateStorageError::NotFound(_)) => {
                ConversationState::new(config.cycle_id.clone(), config.session_id.clone())
            }
            Err(e) => return Err(e.into()),
        };

        // 2. Initialize orchestrator from state
        self.orchestrator = Orchestrator::from_state(state)?;

        // 3. Create session handle (stateless - state in storage)
        Ok(SessionHandle {
            id: uuid::Uuid::new_v4().to_string(),
            cycle_id: config.cycle_id,
            provider: ProviderType::OpenAI,
            status: SessionStatus::Ready,
        })
    }

    async fn send_message(
        &self,
        handle: &SessionHandle,
        message: UserMessage,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, AIEngineError>> + Send>>, AIEngineError> {
        // 1. Load conversation state
        let state = self.state_storage.load_state(&handle.cycle_id).await?;

        // 2. Classify intent and route
        let intent = self.orchestrator.classify_intent(&message.content);
        let target_step = self.orchestrator.route(intent)?;

        // 3. Build messages array
        let system_prompt = self.step_agents.get_system_prompt(
            target_step,
            &self.orchestrator.context_for_step(target_step),
        );

        let messages = self.build_messages(&state, &system_prompt, &message);

        // 4. Call chat completions with streaming
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages)
            .stream(true)
            .build()?;

        let stream = self.client.chat().create_stream(request).await?;

        // 5. Transform stream and handle tool calls
        Ok(Box::pin(self.transform_stream(stream, handle.cycle_id.clone())))
    }

    // ... other trait methods
}
```

### Anthropic API Adapter

Similar to OpenAI but using Anthropic's message format.

```rust
// adapters/anthropic/adapter.rs

use anthropic_sdk::{Client, MessageRequest};
use async_trait::async_trait;

/// Anthropic adapter - uses Messages API
pub struct AnthropicAdapter {
    client: Client,
    model: String,
    state_storage: Arc<dyn StateStoragePort>,
    step_agents: Arc<dyn StepAgentPort>,
    orchestrator: Orchestrator,
}

#[async_trait]
impl AIEnginePort for AnthropicAdapter {
    // Similar implementation to OpenAI adapter
    // but using Anthropic's Messages API format and tool_use

    async fn send_message(
        &self,
        handle: &SessionHandle,
        message: UserMessage,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, AIEngineError>> + Send>>, AIEngineError> {
        // Uses Anthropic-specific request format
        let request = MessageRequest::builder()
            .model(&self.model)
            .max_tokens(4096)
            .system(&system_prompt)
            .messages(messages)
            .stream(true)
            .build()?;

        let stream = self.client.messages().create_stream(request).await?;

        Ok(Box::pin(self.transform_stream(stream)))
    }
}
```

### State Storage Adapters

#### File Storage Adapter

```rust
// adapters/storage/file_storage.rs

use async_trait::async_trait;
use tokio::fs;

/// File-based state storage - source of truth
pub struct FileStorageAdapter {
    base_dir: PathBuf,
}

impl FileStorageAdapter {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    fn state_path(&self, cycle_id: &CycleId) -> PathBuf {
        self.base_dir
            .join(cycle_id.session_id.to_string())
            .join(cycle_id.to_string())
            .join("state.yaml")
    }

    fn step_output_path(&self, cycle_id: &CycleId, component: ProactComponent) -> PathBuf {
        let index = component.index();
        let name = component.snake_case_name();
        self.base_dir
            .join(cycle_id.session_id.to_string())
            .join(cycle_id.to_string())
            .join(format!("{}-{}.yaml", index, name))
    }
}

#[async_trait]
impl StateStoragePort for FileStorageAdapter {
    async fn save_state(
        &self,
        cycle_id: &CycleId,
        state: &ConversationState,
    ) -> Result<(), StateStorageError> {
        let path = self.state_path(cycle_id);

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Serialize and write
        let yaml = serde_yaml::to_string(state)
            .map_err(|e| StateStorageError::SerializationError(e.to_string()))?;

        fs::write(&path, yaml).await?;

        Ok(())
    }

    async fn load_state(
        &self,
        cycle_id: &CycleId,
    ) -> Result<ConversationState, StateStorageError> {
        let path = self.state_path(cycle_id);

        let yaml = fs::read_to_string(&path).await
            .map_err(|_| StateStorageError::NotFound(cycle_id.to_string()))?;

        serde_yaml::from_str(&yaml)
            .map_err(|e| StateStorageError::SerializationError(e.to_string()))
    }

    async fn save_step_output(
        &self,
        cycle_id: &CycleId,
        output: &dyn StructuredOutput,
    ) -> Result<(), StateStorageError> {
        let path = self.step_output_path(cycle_id, output.component());

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let yaml = output.to_yaml()
            .map_err(|e| StateStorageError::SerializationError(e.to_string()))?;

        fs::write(&path, yaml).await?;

        Ok(())
    }

    async fn search_decisions(
        &self,
        _query: &str,
        _session_filter: Option<&SessionId>,
        _limit: usize,
    ) -> Result<Vec<SearchResult>, StateStorageError> {
        // File storage doesn't support search - return empty
        Ok(vec![])
    }

    // ... other methods
}
```

#### Hybrid Storage Adapter

Files as source of truth, database for indexing.

```rust
// adapters/storage/hybrid_storage.rs

use async_trait::async_trait;
use sqlx::PgPool;

/// Hybrid storage - files for truth, database for indexing/search
pub struct HybridStorageAdapter {
    file_storage: FileStorageAdapter,
    pool: PgPool,
}

impl HybridStorageAdapter {
    pub async fn new(base_dir: PathBuf, database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self {
            file_storage: FileStorageAdapter::new(base_dir),
            pool,
        })
    }
}

#[async_trait]
impl StateStoragePort for HybridStorageAdapter {
    async fn save_step_output(
        &self,
        cycle_id: &CycleId,
        output: &dyn StructuredOutput,
    ) -> Result<(), StateStorageError> {
        // 1. Write to file (source of truth)
        self.file_storage.save_step_output(cycle_id, output).await?;

        // 2. Update database index
        self.index_output(cycle_id, output).await?;

        Ok(())
    }

    async fn search_decisions(
        &self,
        query: &str,
        session_filter: Option<&SessionId>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, StateStorageError> {
        let results = sqlx::query_as!(
            SearchResultRow,
            r#"
            SELECT
                session_id, cycle_id, component,
                ts_headline('english', content, plainto_tsquery($1)) as snippet,
                ts_rank(search_vector, plainto_tsquery($1)) as score
            FROM decision_search
            WHERE search_vector @@ plainto_tsquery($1)
              AND ($2::uuid IS NULL OR session_id = $2)
            ORDER BY score DESC
            LIMIT $3
            "#,
            query,
            session_filter.map(|s| s.0),
            limit as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StateStorageError::DatabaseError(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    /// Rebuild database index from files
    pub async fn sync_from_files(&self, session_dir: &Path) -> Result<usize, StateStorageError> {
        let mut count = 0;

        // Walk directory and index each file
        let mut entries = fs::read_dir(session_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension() == Some("yaml".as_ref()) {
                self.index_file(&entry.path()).await?;
                count += 1;
            }
        }

        Ok(count)
    }
}
```

---

## Application Layer

### Commands

```rust
// application/commands/ai_engine.rs

use cqrs_es::Command;

/// Start a conversation session for a cycle
#[derive(Debug, Clone)]
pub struct StartConversation {
    pub cycle_id: CycleId,
    pub provider: ProviderType,
    pub initial_step: ProactComponent,
}

pub struct StartConversationHandler {
    engines: HashMap<ProviderType, Arc<dyn AIEnginePort>>,
    cycle_repo: Arc<dyn CycleRepository>,
    state_storage: Arc<dyn StateStoragePort>,
}

impl StartConversationHandler {
    pub async fn handle(&self, cmd: StartConversation) -> Result<SessionHandle, CommandError> {
        // 1. Verify cycle exists and user has access
        let cycle = self.cycle_repo.get(&cmd.cycle_id).await?;

        // 2. Get appropriate engine
        let engine = self.engines.get(&cmd.provider)
            .ok_or(CommandError::ProviderNotAvailable(cmd.provider))?;

        // 3. Build config
        let config = SessionConfig {
            cycle_id: cmd.cycle_id,
            session_id: cycle.session_id,
            working_dir: self.state_storage.working_dir(&cmd.cycle_id),
            initial_step: cmd.initial_step,
            prior_context: None,
            provider_config: self.provider_config(cmd.provider),
        };

        // 4. Start session
        engine.start_session(config).await
            .map_err(CommandError::from)
    }
}

/// Send a message in an active session
#[derive(Debug, Clone)]
pub struct SendMessage {
    pub session_handle: SessionHandle,
    pub content: String,
}

/// Transition to a different PrOACT step
#[derive(Debug, Clone)]
pub struct TransitionStep {
    pub session_handle: SessionHandle,
    pub target_step: ProactComponent,
}

/// Complete the current step
#[derive(Debug, Clone)]
pub struct CompleteStep {
    pub session_handle: SessionHandle,
    pub summary: Option<String>,
}

/// Branch the cycle at current point
#[derive(Debug, Clone)]
pub struct BranchCycle {
    pub session_handle: SessionHandle,
    pub branch_name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Cycle not found: {0}")]
    CycleNotFound(CycleId),

    #[error("Provider not available: {0:?}")]
    ProviderNotAvailable(ProviderType),

    #[error("AI engine error: {0}")]
    AIEngineError(#[from] AIEngineError),

    #[error("Access denied")]
    AccessDenied,
}
```

### Queries

```rust
// application/queries/ai_engine.rs

/// Get current conversation state
#[derive(Debug, Clone)]
pub struct GetConversationState {
    pub cycle_id: CycleId,
}

pub struct GetConversationStateHandler {
    state_storage: Arc<dyn StateStoragePort>,
}

impl GetConversationStateHandler {
    pub async fn handle(&self, query: GetConversationState) -> Result<ConversationState, QueryError> {
        self.state_storage.load_state(&query.cycle_id).await
            .map_err(QueryError::from)
    }
}

/// Get structured output from a completed step
#[derive(Debug, Clone)]
pub struct GetStepOutput {
    pub cycle_id: CycleId,
    pub component: ProactComponent,
}

/// Search across all decisions
#[derive(Debug, Clone)]
pub struct SearchDecisions {
    pub query: String,
    pub session_id: Option<SessionId>,
    pub limit: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("State not found: {0}")]
    StateNotFound(String),

    #[error("Storage error: {0}")]
    StorageError(#[from] StateStorageError),
}
```

---

## File Schemas

### State File (state.yaml)

```yaml
# decisions/<session-id>/<cycle-id>/state.yaml

cycle_id: "cycle-001"
session_id: "session-abc-123"
current_step: "objectives"
status: "in_progress"

branch_info:
  parent_cycle: null          # or "cycle-001" if branched
  branch_point: null          # or "objectives" if branched
  branch_name: null           # or "higher-budget-scenario"

completed_steps:
  issue-raising:
    status: "completed"
    completed_at: "2026-01-07T10:30:00Z"
    turn_count: 8
    summary: |
      Identified 3 key decisions: office relocation, team structure,
      and technology stack. Flagged 2 major uncertainties around
      market conditions and hiring timeline.
    key_outputs:
      - "3 decisions categorized"
      - "2 uncertainties flagged"
      - "5 objectives surfaced"

  problem-frame:
    status: "completed"
    completed_at: "2026-01-07T11:15:00Z"
    turn_count: 12
    summary: |
      Framed as location choice decision with $500K budget constraint.
      Key stakeholders: leadership team, employees, clients.
      Time horizon: 18 months.
    key_outputs:
      - "Decision architecture defined"
      - "4 stakeholders mapped"
      - "3 hard constraints identified"

in_progress_steps:
  objectives:
    status: "in_progress"
    started_at: "2026-01-07T11:20:00Z"
    turn_count: 5
    partial_summary: |
      Identified 3 fundamental objectives so far. Working on
      means-ends chains.

context:
  compressed_history: |
    [AI-generated summary of conversation for context efficiency]

  token_estimate: 2500
  last_compression: "2026-01-07T11:00:00Z"
```

### Step Output Files

See original specification for complete YAML schemas for:
- Issue Raising (1-issue-raising.yaml)
- Objectives (3-objectives.yaml)
- Consequences (5-consequences.yaml)

---

## Provider Comparison Matrix

| Capability | Claude Code | OpenAI API | Anthropic API |
|------------|-------------|------------|---------------|
| **Orchestration** | Native (skills/subagents) | Application layer | Application layer |
| **State management** | File-based (built-in) | External storage | External storage |
| **Tool use** | Rich (file I/O, bash, etc.) | Function calling | Tool use |
| **Streaming** | stdout | SSE | SSE |
| **Context window** | Managed by CC | Manual | Manual |
| **Cost model** | Per-session | Per-token | Per-token |
| **Deployment** | CLI process | HTTP API | HTTP API |
| **Multi-user** | Process per user | Shared service | Shared service |

---

## Configuration

```yaml
# config/ai-engine.yaml

default_provider: "claude-code"

providers:
  claude-code:
    enabled: true
    binary_path: "/usr/local/bin/claude"
    skills_dir: ".claude/skills"
    agents_dir: ".claude/agents"
    timeout_secs: 300
    max_sessions: 10

  openai:
    enabled: true
    model: "gpt-4-turbo"
    api_key_env: "OPENAI_API_KEY"
    max_tokens: 4096
    temperature: 0.7

  anthropic:
    enabled: true
    model: "claude-sonnet-4-20250514"
    api_key_env: "ANTHROPIC_API_KEY"
    max_tokens: 4096

storage:
  type: "hybrid"  # file, database, hybrid
  file:
    base_dir: "decisions"
  database:
    connection_string_env: "DATABASE_URL"
    index_enabled: true
    search_enabled: true

orchestration:
  context_compression_threshold: 3000  # tokens
  auto_save_interval_secs: 30
  idle_session_timeout_secs: 1800
```

---

## Cargo Dependencies

```toml
# Cargo.toml for ai-engine crate

[dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
futures = "0.3"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# Error handling
thiserror = "1"
anyhow = "1"

# Database
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }

# AI SDKs (optional features)
async-openai = { version = "0.18", optional = true }
# anthropic-sdk = { version = "0.1", optional = true }  # hypothetical

# Utilities
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"

[features]
default = ["claude-code"]
claude-code = []
openai = ["async-openai"]
anthropic = []
all-providers = ["claude-code", "openai", "anthropic"]
```

---

## Testing Strategy

### Domain Layer Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orchestrator_routes_continue_intent() {
        let orch = Orchestrator::new(cycle_id(), ProactComponent::IssueRaising);
        let result = orch.route(UserIntent::Continue);
        assert_eq!(result.unwrap(), ProactComponent::IssueRaising);
    }

    #[test]
    fn orchestrator_validates_transitions() {
        let orch = Orchestrator::new(cycle_id(), ProactComponent::IssueRaising);
        assert!(orch.can_transition(ProactComponent::ProblemFrame));
        assert!(!orch.can_transition(ProactComponent::Consequences)); // too far
    }

    #[test]
    fn orchestrator_records_completion() {
        let mut orch = Orchestrator::new(cycle_id(), ProactComponent::IssueRaising);
        let summary = StepSummary { /* ... */ };
        orch.record_completion(summary).unwrap();
        assert_eq!(orch.current_step, ProactComponent::ProblemFrame);
    }
}
```

### Port Contract Tests

```rust
/// Contract test that any AIEnginePort implementation must pass
#[cfg(test)]
pub async fn ai_engine_port_contract<T: AIEnginePort>(adapter: T) {
    // Start session
    let config = test_session_config();
    let handle = adapter.start_session(config).await.unwrap();
    assert_eq!(handle.status, SessionStatus::Ready);

    // Send message and receive stream
    let msg = UserMessage { content: "Hello".into(), attachments: vec![] };
    let stream = adapter.send_message(&handle, msg).await.unwrap();
    let chunks: Vec<_> = stream.collect().await;
    assert!(!chunks.is_empty());

    // Get state
    let state = adapter.get_state(&handle).await.unwrap();
    assert_eq!(state.cycle_id, handle.cycle_id);

    // End session
    adapter.end_session(&handle).await.unwrap();
}

#[tokio::test]
async fn file_storage_satisfies_contract() {
    let adapter = FileStorageAdapter::new(temp_dir());
    state_storage_port_contract(adapter).await;
}
```

### Integration Tests

```rust
#[tokio::test]
#[ignore] // Requires Claude CLI installed
async fn claude_code_adapter_integration() {
    let adapter = ClaudeCodeAdapter::new(/* ... */);
    ai_engine_port_contract(adapter).await;
}

#[tokio::test]
#[ignore] // Requires API key
async fn openai_adapter_integration() {
    let adapter = OpenAIAdapter::new(/* ... */);
    ai_engine_port_contract(adapter).await;
}
```

---

## Implementation Sequence

### Phase 1: Domain & Ports
1. Define all domain types (Orchestrator, StepAgent, ConversationState)
2. Define port traits (AIEnginePort, StepAgentPort, StateStoragePort)
3. Implement domain services (IntentClassifier, ContextCompressor)
4. Unit test domain logic

### Phase 2: File Storage Adapter
1. Implement FileStorageAdapter
2. Define serde types for YAML schemas
3. Implement file watching for external changes
4. Integration test with filesystem

### Phase 3: Claude Code Adapter
1. Implement ProcessManager
2. Create orchestrator skill
3. Create step agent definitions
4. Implement streaming parser
5. Integration test with Claude CLI

### Phase 4: API Adapters
1. Implement OpenAI adapter
2. Implement Anthropic adapter
3. Implement adapter factory for provider selection
4. Contract tests for all adapters

### Phase 5: Hybrid Storage
1. Add database schema for indexing
2. Implement HybridStorageAdapter
3. Implement sync mechanism
4. Add search capability

### Phase 6: Application Layer
1. Implement command handlers
2. Implement query handlers
3. Wire up dependency injection
4. End-to-end tests

---

## Open Questions

1. **Context handoff between providers**: If user starts with Claude Code and switches to OpenAI mid-cycle, how do we transfer context effectively?

2. **Concurrent sessions**: Should one user be able to have multiple active AI sessions? How do we manage resource limits?

3. **Offline capability**: Should the file-based approach support fully offline operation (no AI) with manual step completion?

4. **Voice integration**: The spec mentions Whisper for voice. Should AIEnginePort include voice input handling, or is that a separate concern?

5. **Real-time collaboration**: If multiple users view the same cycle, how do we handle concurrent AI interactions?

---

## References

- [System Architecture](./architecture/SYSTEM-ARCHITECTURE.md)
- [PrOACT Types Module](./modules/proact-types.md)
- [Conversation Module](./modules/conversation.md)
- [Cycle Module](./modules/cycle.md)
