# Atomic Decision Tools with Emergent Composition

**Module:** conversation
**Type:** Feature Enhancement
**Priority:** P1 (Phase 1 of Agent-Native Enrichments)
**Status:** Specification
**Version:** 1.0.0
**Created:** 2026-01-09
**Based on:** [Agent-Native Enrichments](../../docs/architecture/AGENT-NATIVE-ENRICHMENTS.md) - Suggestion 2

---

## Executive Summary

Atomic Decision Tools transform the agent from a "conversation-only" assistant into a **tool-wielding decision analyst**. Instead of generating unstructured text that must be parsed, the agent invokes precise tools that directly manipulate the decision document and component state.

### Key Benefits

| Benefit | Description |
|---------|-------------|
| **Token Efficiency** | Tools operate on specific data, eliminating redundant context |
| **Structured Output** | Bypass markdown parsing - tools produce validated data directly |
| **Emergent Behavior** | Agent composes tools based on conversation signals |
| **Auditability** | Every tool invocation is logged with reasoning |
| **Consistency** | Tools enforce domain invariants automatically |

### Design Principles

| Principle | Implementation |
|-----------|----------------|
| **Granularity** | Each tool does ONE thing well |
| **Composability** | Complex behaviors emerge from simple tool combinations |
| **Linear Flow** | Tools operate within current component; suggestions queue for later |
| **Document-First** | Tools update decision document directly |
| **Minimal Context** | Tools receive only what they need, not full conversation |

---

## Architecture Overview

### Current State

```
┌─────────────────────────────────────────────────────────────────┐
│                    CURRENT ARCHITECTURE                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   User Message ──► Agent (full context) ──► Response Text       │
│                          │                                       │
│                          ▼                                       │
│              System extracts structured data                     │
│              from response (error-prone parsing)                 │
│                                                                  │
│   Problems:                                                      │
│   • Agent regenerates entire section each turn                   │
│   • Parsing unstructured output is fragile                       │
│   • No audit trail of what agent "decided" to do                 │
│   • Full context passed every turn (token expensive)             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Agent-Native Enhancement

```
┌─────────────────────────────────────────────────────────────────┐
│                    TOOL-AUGMENTED ARCHITECTURE                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   User Message ──► Agent ──┬──► Tool Invocation(s)              │
│                            │         │                           │
│                            │         ▼                           │
│                            │    Decision Document                │
│                            │    Component State                  │
│                            │         │                           │
│                            │         ▼                           │
│                            │    Tool Result                      │
│                            │         │                           │
│                            └────◄────┘                           │
│                            │                                     │
│                            ▼                                     │
│                     Response to User                             │
│                                                                  │
│   Benefits:                                                      │
│   • Delta updates, not full regeneration                         │
│   • Structured data from tool calls (validated)                  │
│   • Complete audit trail                                         │
│   • Reduced context via focused tool parameters                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Tool Categories

### 1. Component-Specific Tools

Each PrOACT component has tools tailored to its data structures.

#### Issue Raising Tools

| Tool | Parameters | Returns | Description |
|------|------------|---------|-------------|
| `add_potential_decision` | `description: String` | `DecisionId` | Add a decision to consider |
| `add_objective_idea` | `description: String` | `ObjectiveIdeaId` | Capture an objective mentioned |
| `add_uncertainty` | `description: String, resolvable: bool` | `UncertaintyId` | Flag an uncertainty |
| `add_consideration` | `description: String` | `ConsiderationId` | Add general consideration |
| `set_focal_decision` | `decision_id: DecisionId` | `()` | Mark which decision to focus on |

```rust
/// Issue Raising: Add a potential decision to the list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPotentialDecision {
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPotentialDecisionResult {
    pub decision_id: String,
    pub current_count: usize,
    pub document_updated: bool,
}
```

#### Problem Frame Tools

| Tool | Parameters | Returns | Description |
|------|------------|---------|-------------|
| `set_decision_maker` | `name: String, role: String` | `()` | Define who decides |
| `set_focal_statement` | `statement: String` | `()` | Set the focal decision statement |
| `set_scope` | `in_scope: Vec<String>, out_scope: Vec<String>` | `()` | Define boundaries |
| `add_constraint` | `type: String, description: String` | `ConstraintId` | Add a constraint |
| `add_party` | `name: String, role: PartyRole, concerns: Vec<String>` | `PartyId` | Add stakeholder |
| `set_deadline` | `deadline: String, hard: bool` | `()` | Set decision deadline |
| `add_hierarchy_decision` | `description: String, level: HierarchyLevel, status: String` | `DecisionId` | Add to decision hierarchy |

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddConstraint {
    pub constraint_type: String,  // "budget", "time", "resource", "policy", "other"
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddConstraintResult {
    pub constraint_id: String,
    pub total_constraints: usize,
}
```

#### Objectives Tools

| Tool | Parameters | Returns | Description |
|------|------------|---------|-------------|
| `add_objective` | `name: String, measure: String, direction: Direction, is_fundamental: bool` | `ObjectiveId` | Add objective |
| `link_means_to_fundamental` | `means_id: ObjectiveId, fundamental_id: ObjectiveId` | `()` | Link means → fundamental |
| `update_objective_measure` | `id: ObjectiveId, new_measure: String` | `()` | Refine measurement |
| `remove_objective` | `id: ObjectiveId, reason: String` | `()` | Remove (logged) |
| `promote_to_fundamental` | `id: ObjectiveId, reason: String` | `()` | Promote means to fundamental |

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddObjective {
    pub name: String,
    pub measure: String,
    pub direction: ObjectiveDirection,
    pub is_fundamental: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveDirection {
    Higher,  // Higher is better (maximize)
    Lower,   // Lower is better (minimize)
    Target,  // Specific target value
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddObjectiveResult {
    pub objective_id: String,
    pub objective_type: String,  // "fundamental" or "means"
    pub total_fundamental: usize,
    pub total_means: usize,
}
```

#### Alternatives Tools

| Tool | Parameters | Returns | Description |
|------|------------|---------|-------------|
| `add_alternative` | `name: String, description: String, is_status_quo: bool` | `AlternativeId` | Add option |
| `update_alternative` | `id: AlternativeId, description: String` | `()` | Update description |
| `remove_alternative` | `id: AlternativeId, reason: String` | `()` | Remove (logged) |
| `add_strategy_dimension` | `dimension: String, options: Vec<String>` | `StrategyDimensionId` | Add strategy table row |
| `set_alternative_strategy` | `alt_id: AlternativeId, dim_id: StrategyDimensionId, option: String` | `()` | Set strategy choice |

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAlternative {
    pub name: String,
    pub description: String,
    pub is_status_quo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAlternativeResult {
    pub alternative_id: String,
    pub letter: String,  // "A", "B", "C", etc.
    pub total_alternatives: usize,
    pub has_status_quo: bool,
}
```

#### Consequences Tools

| Tool | Parameters | Returns | Description |
|------|------------|---------|-------------|
| `rate_consequence` | `alt_id: AlternativeId, obj_id: ObjectiveId, rating: PughRating, reasoning: String` | `()` | Rate a cell |
| `batch_rate_consequences` | `ratings: Vec<ConsequenceRating>` | `BatchResult` | Rate multiple cells |
| `add_consequence_uncertainty` | `alt_id: AlternativeId, obj_id: ObjectiveId, uncertainty: String` | `UncertaintyId` | Flag uncertainty |
| `update_rating_reasoning` | `alt_id: AlternativeId, obj_id: ObjectiveId, reasoning: String` | `()` | Update reasoning |

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateConsequence {
    pub alternative_id: String,
    pub objective_id: String,
    pub rating: PughRating,
    pub reasoning: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PughRating {
    MuchWorse = -2,
    Worse = -1,
    Same = 0,
    Better = 1,
    MuchBetter = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateConsequenceResult {
    pub cell_updated: bool,
    pub matrix_completion: f32,  // 0.0 - 1.0
    pub cells_remaining: usize,
}
```

#### Tradeoffs Tools

| Tool | Parameters | Returns | Description |
|------|------------|---------|-------------|
| `mark_dominated` | `alt_id: AlternativeId, dominated_by: AlternativeId, reason: String` | `()` | Mark dominated |
| `mark_irrelevant_objective` | `obj_id: ObjectiveId, reason: String` | `()` | Mark non-differentiating |
| `add_tension` | `alt_id: AlternativeId, excels_at: String, sacrifices: String` | `TensionId` | Document tension |
| `clear_dominated` | `alt_id: AlternativeId, reason: String` | `()` | Un-dominate if reconsidered |

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkDominated {
    pub alternative_id: String,
    pub dominated_by: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkDominatedResult {
    pub marked: bool,
    pub remaining_alternatives: usize,
    pub analysis_note: Option<String>,  // e.g., "Only 2 non-dominated alternatives remain"
}
```

#### Recommendation Tools

| Tool | Parameters | Returns | Description |
|------|------------|---------|-------------|
| `set_synthesis` | `synthesis: String` | `()` | Set analysis synthesis |
| `set_standout` | `alt_id: Option<AlternativeId>, rationale: String` | `()` | Mark standout (or none) |
| `add_key_consideration` | `consideration: String` | `ConsiderationId` | Add pre-decision consideration |
| `add_remaining_uncertainty` | `uncertainty: String, resolution_path: String` | `UncertaintyId` | Document unresolved |

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetStandout {
    pub alternative_id: Option<String>,  // None if no clear standout
    pub rationale: String,
}
```

#### Decision Quality Tools

| Tool | Parameters | Returns | Description |
|------|------------|---------|-------------|
| `rate_dq_element` | `element: DQElement, score: u8, rationale: String` | `()` | Rate 0-100 |
| `add_quality_improvement` | `action: String, impact: String` | `ImprovementId` | Suggest improvement |
| `calculate_overall_dq` | `()` | `DQScore` | Calculate overall (min of elements) |

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateDQElement {
    pub element: DQElement,
    pub score: u8,  // 0-100
    pub rationale: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DQElement {
    HelpfulProblemFrame,
    ClearObjectives,
    CreativeAlternatives,
    ReliableConsequences,
    LogicallyCorrectReasoning,
    ClearTradeoffs,
    CommitmentToFollowThrough,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateDQElementResult {
    pub element: DQElement,
    pub score: u8,
    pub overall_dq: u8,  // Minimum of all rated elements
    pub weakest_element: DQElement,
    pub elements_rated: usize,
    pub elements_remaining: usize,
}
```

---

### 2. Cross-Cutting Tools

These tools can be invoked from any component.

#### Uncertainty Management

| Tool | Parameters | Returns | Description |
|------|------------|---------|-------------|
| `flag_uncertainty` | `description: String, component: ComponentType, resolvable: bool` | `UncertaintyId` | Flag uncertainty anywhere |
| `resolve_uncertainty` | `id: UncertaintyId, resolution: String` | `()` | Mark resolved |
| `list_uncertainties` | `filter: Option<ComponentType>` | `Vec<Uncertainty>` | Query current uncertainties |

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagUncertainty {
    pub description: String,
    pub component: ComponentType,
    pub resolvable: bool,
    pub impact: Option<String>,  // Optional: how it affects the decision
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagUncertaintyResult {
    pub uncertainty_id: String,
    pub total_open_uncertainties: usize,
    pub resolvable_count: usize,
}
```

#### Revisit Suggestions (Linear Flow Compliant)

| Tool | Parameters | Returns | Description |
|------|------------|---------|-------------|
| `suggest_revisit` | `component: ComponentType, reason: String, priority: Priority` | `SuggestionId` | Queue revisit suggestion |
| `get_pending_revisits` | `()` | `Vec<RevisitSuggestion>` | List queued suggestions |
| `dismiss_revisit` | `id: SuggestionId, reason: String` | `()` | Dismiss suggestion |

```rust
/// Suggest revisiting a component (queued, not immediate navigation)
/// Respects linear PrOACT flow - user sees suggestions after completing current component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestRevisit {
    pub component: ComponentType,
    pub reason: String,
    pub priority: RevisitPriority,
    pub trigger: String,  // What triggered this suggestion
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisitPriority {
    Low,      // Nice to have
    Medium,   // Recommended
    High,     // Important gap identified
    Critical, // Decision quality at risk
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestRevisitResult {
    pub suggestion_id: String,
    pub total_pending: usize,
    pub will_prompt_user: bool,  // If high/critical, user prompted at end of component
}
```

#### User Confirmation

| Tool | Parameters | Returns | Description |
|------|------------|---------|-------------|
| `request_confirmation` | `summary: String, options: Vec<String>` | `AwaitingConfirmation` | Pause for user input |
| `record_user_choice` | `confirmation_id: ConfirmationId, choice: String` | `()` | Record what user chose |

```rust
/// Pause conversation to get explicit user confirmation
/// Use when agent is about to make significant changes or assumptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestConfirmation {
    pub summary: String,
    pub options: Vec<ConfirmationOption>,
    pub default_option: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationOption {
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestConfirmationResult {
    pub confirmation_id: String,
    pub status: ConfirmationStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationStatus {
    Pending,
    Confirmed,
    Rejected,
    Expired,
}
```

#### Document Operations

| Tool | Parameters | Returns | Description |
|------|------------|---------|-------------|
| `get_document_section` | `component: ComponentType` | `String` | Read current section |
| `get_document_summary` | `()` | `DocumentSummary` | Get high-level state |
| `add_note` | `content: String, component: Option<ComponentType>` | `NoteId` | Add working note |

```rust
/// Get a summary of the current document state (minimal context)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDocumentSummary {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSummary {
    pub title: String,
    pub focal_decision: Option<String>,
    pub current_component: ComponentType,
    pub completion_status: HashMap<ComponentType, CompletionStatus>,
    pub objectives_count: usize,
    pub alternatives_count: usize,
    pub matrix_completion: f32,
    pub open_uncertainties: usize,
    pub pending_revisits: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionStatus {
    NotStarted,
    InProgress,
    Complete,
}
```

---

### 3. Analysis Tools

These tools perform computations on the current state.

| Tool | Parameters | Returns | Description |
|------|------------|---------|-------------|
| `compute_pugh_totals` | `()` | `PughTotals` | Calculate alternative scores |
| `find_dominated_alternatives` | `()` | `Vec<DominatedAlternative>` | Auto-detect dominated |
| `find_irrelevant_objectives` | `()` | `Vec<ObjectiveId>` | Auto-detect non-differentiating |
| `sensitivity_check` | `alt_id: AlternativeId, obj_id: ObjectiveId` | `SensitivityResult` | What-if on one cell |

```rust
/// Compute Pugh matrix totals for all alternatives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputePughTotals {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PughTotalsResult {
    pub alternatives: Vec<AlternativeScore>,
    pub baseline_id: String,
    pub leader_id: String,
    pub tied_leaders: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeScore {
    pub alternative_id: String,
    pub name: String,
    pub total_score: i32,
    pub positive_count: usize,
    pub negative_count: usize,
    pub neutral_count: usize,
}

/// Auto-detect dominated alternatives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindDominatedAlternatives {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DominatedAlternativesResult {
    pub dominated: Vec<DominatedInfo>,
    pub analysis_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DominatedInfo {
    pub alternative_id: String,
    pub alternative_name: String,
    pub dominated_by: String,
    pub dominated_by_name: String,
    pub reason: String,
}
```

---

## Token Optimization Strategies

### Strategy 1: Delta Updates

Instead of regenerating entire sections, tools update specific fields.

```
BEFORE (text generation):
┌───────────────────────────────────────────────────────────────┐
│ Agent receives: Full conversation history + full document      │
│ Agent produces: Complete regenerated Objectives section        │
│ Tokens used: ~2000 in, ~500 out                               │
└───────────────────────────────────────────────────────────────┘

AFTER (tool invocation):
┌───────────────────────────────────────────────────────────────┐
│ Agent receives: Conversation context + document summary        │
│ Agent invokes: add_objective(name, measure, direction, true)   │
│ Tokens used: ~500 in, ~50 out                                 │
└───────────────────────────────────────────────────────────────┘
```

### Strategy 2: Minimal Context Injection

Tools receive only what they need, not full conversation history.

```rust
/// Tool execution context - minimal information needed
#[derive(Debug, Clone)]
pub struct ToolExecutionContext {
    /// Current component being worked on
    pub current_component: ComponentType,

    /// Summary counts (not full data)
    pub objectives_count: usize,
    pub alternatives_count: usize,

    /// Only IDs, not full objects
    pub objective_ids: Vec<String>,
    pub alternative_ids: Vec<String>,

    /// Cycle and document IDs for persistence
    pub cycle_id: CycleId,
    pub document_id: DecisionDocumentId,
}
```

### Strategy 3: Batch Operations

When the agent has multiple updates, batch them.

```rust
/// Batch multiple consequence ratings in one call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRateConsequences {
    pub ratings: Vec<ConsequenceRating>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsequenceRating {
    pub alternative_id: String,
    pub objective_id: String,
    pub rating: PughRating,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRateResult {
    pub updated_count: usize,
    pub failed: Vec<BatchFailure>,
    pub matrix_completion: f32,
}
```

### Strategy 4: Lazy Loading

Tools that return data use pagination/summary modes.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListObjectives {
    /// If true, return full details; if false, return IDs and names only
    pub include_details: bool,

    /// Filter by type
    pub filter: Option<ObjectiveTypeFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectiveTypeFilter {
    Fundamental,
    Means,
    All,
}
```

---

## Domain Model

### ToolInvocation Entity

Every tool call is logged for audit and analysis.

```rust
#[derive(Debug, Clone)]
pub struct ToolInvocation {
    id: ToolInvocationId,
    cycle_id: CycleId,
    component: ComponentType,

    // What was called
    tool_name: String,
    parameters: serde_json::Value,

    // Result
    result: ToolResult,
    result_data: Option<serde_json::Value>,

    // Context
    conversation_turn: u32,
    triggered_by: String,  // What in the conversation triggered this

    // Timing
    invoked_at: Timestamp,
    completed_at: Timestamp,
    duration_ms: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolResult {
    Success,
    ValidationError,
    NotFound,
    Conflict,
    InternalError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolInvocationId(uuid::Uuid);
```

### RevisitSuggestion Entity

Queued suggestions for component revisits.

```rust
#[derive(Debug, Clone)]
pub struct RevisitSuggestion {
    id: RevisitSuggestionId,
    cycle_id: CycleId,

    // What to revisit
    target_component: ComponentType,
    reason: String,
    trigger: String,
    priority: RevisitPriority,

    // Status
    status: SuggestionStatus,
    created_at: Timestamp,
    resolved_at: Option<Timestamp>,
    resolution: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionStatus {
    Pending,
    Accepted,  // User chose to revisit
    Dismissed, // User chose not to revisit
    Expired,   // Decision completed without addressing
}
```

### ConfirmationRequest Entity

Tracks user confirmations requested by agent.

```rust
#[derive(Debug, Clone)]
pub struct ConfirmationRequest {
    id: ConfirmationRequestId,
    cycle_id: CycleId,
    conversation_turn: u32,

    // Request
    summary: String,
    options: Vec<ConfirmationOption>,
    default_option: Option<usize>,

    // Response
    status: ConfirmationStatus,
    chosen_option: Option<usize>,
    user_input: Option<String>,  // If they provided custom input

    // Timing
    requested_at: Timestamp,
    responded_at: Option<Timestamp>,
    expires_at: Timestamp,  // Auto-expire if not responded
}
```

---

## Domain Events

| Event | Trigger | Data |
|-------|---------|------|
| `ToolInvoked` | Any tool call | tool_name, parameters, cycle_id |
| `ToolCompleted` | Tool execution finished | tool_name, result, duration_ms |
| `ObjectiveAdded` | add_objective tool | objective_id, name, is_fundamental |
| `AlternativeAdded` | add_alternative tool | alternative_id, name |
| `ConsequenceRated` | rate_consequence tool | alt_id, obj_id, rating |
| `DominatedMarked` | mark_dominated tool | alt_id, dominated_by |
| `UncertaintyFlagged` | flag_uncertainty tool | uncertainty_id, component |
| `RevisitSuggested` | suggest_revisit tool | component, reason, priority |
| `ConfirmationRequested` | request_confirmation tool | confirmation_id, summary |
| `ConfirmationResolved` | User responds | confirmation_id, choice |

---

## Ports

### ToolExecutor Port

```rust
use async_trait::async_trait;

/// Port for executing atomic decision tools
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool and return the result
    async fn execute(
        &self,
        tool: ToolCall,
        context: ToolExecutionContext,
    ) -> Result<ToolResponse, ToolError>;

    /// Get available tools for current component
    fn available_tools(&self, component: ComponentType) -> Vec<ToolDefinition>;

    /// Validate tool parameters before execution
    fn validate(&self, tool: &ToolCall) -> Result<(), ValidationError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub document_updated: bool,
    pub suggestions: Vec<String>,  // Any agent suggestions from the tool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters_schema: serde_json::Value,  // JSON Schema
    pub returns_schema: serde_json::Value,
}
```

### ToolInvocationRepository Port

```rust
#[async_trait]
pub trait ToolInvocationRepository: Send + Sync {
    /// Save a tool invocation
    async fn save(&self, invocation: &ToolInvocation) -> Result<(), DomainError>;

    /// Find invocations for a cycle
    async fn find_by_cycle(&self, cycle_id: CycleId) -> Result<Vec<ToolInvocation>, DomainError>;

    /// Find invocations for a component within a cycle
    async fn find_by_component(
        &self,
        cycle_id: CycleId,
        component: ComponentType,
    ) -> Result<Vec<ToolInvocation>, DomainError>;

    /// Get invocation by ID
    async fn find_by_id(&self, id: ToolInvocationId) -> Result<Option<ToolInvocation>, DomainError>;
}
```

### RevisitSuggestionRepository Port

```rust
#[async_trait]
pub trait RevisitSuggestionRepository: Send + Sync {
    async fn save(&self, suggestion: &RevisitSuggestion) -> Result<(), DomainError>;
    async fn update(&self, suggestion: &RevisitSuggestion) -> Result<(), DomainError>;
    async fn find_pending(&self, cycle_id: CycleId) -> Result<Vec<RevisitSuggestion>, DomainError>;
    async fn find_by_id(&self, id: RevisitSuggestionId) -> Result<Option<RevisitSuggestion>, DomainError>;
}
```

---

## Application Layer

### ToolExecutionCommand

```rust
#[derive(Debug, Clone)]
pub struct ExecuteToolCommand {
    pub cycle_id: CycleId,
    pub user_id: UserId,
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub conversation_turn: u32,
    pub trigger_context: String,  // What triggered this tool call
}

pub struct ExecuteToolHandler {
    cycle_repo: Arc<dyn CycleRepository>,
    doc_repo: Arc<dyn DecisionDocumentRepository>,
    invocation_repo: Arc<dyn ToolInvocationRepository>,
    tool_executor: Arc<dyn ToolExecutor>,
    doc_generator: Arc<dyn DocumentGenerator>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl ExecuteToolHandler {
    pub async fn handle(&self, cmd: ExecuteToolCommand) -> Result<ToolResponse, DomainError> {
        // 1. Load cycle and verify ownership
        let mut cycle = self.cycle_repo.find_by_id(cmd.cycle_id).await?
            .ok_or_else(|| DomainError::not_found("cycle"))?;

        cycle.authorize(&cmd.user_id)?;

        // 2. Build minimal execution context
        let context = ToolExecutionContext {
            current_component: cycle.current_component(),
            objectives_count: cycle.objectives().len(),
            alternatives_count: cycle.alternatives().len(),
            objective_ids: cycle.objective_ids(),
            alternative_ids: cycle.alternative_ids(),
            cycle_id: cmd.cycle_id,
            document_id: self.doc_repo.find_by_cycle(cmd.cycle_id).await?
                .map(|d| d.id())
                .ok_or_else(|| DomainError::not_found("document"))?,
        };

        // 3. Validate and execute tool
        let tool_call = ToolCall {
            name: cmd.tool_name.clone(),
            parameters: cmd.parameters.clone(),
        };

        self.tool_executor.validate(&tool_call)?;

        let start = Instant::now();
        let response = self.tool_executor.execute(tool_call, context).await?;
        let duration_ms = start.elapsed().as_millis() as u32;

        // 4. Record invocation
        let invocation = ToolInvocation::new(
            cmd.cycle_id,
            cycle.current_component(),
            cmd.tool_name,
            cmd.parameters,
            if response.success { ToolResult::Success } else { ToolResult::ValidationError },
            response.data.clone(),
            cmd.conversation_turn,
            cmd.trigger_context,
            duration_ms,
        );

        self.invocation_repo.save(&invocation).await?;

        // 5. If tool updated component, regenerate document section
        if response.document_updated {
            // Reload cycle with updates
            let updated_cycle = self.cycle_repo.find_by_id(cmd.cycle_id).await?
                .ok_or_else(|| DomainError::not_found("cycle"))?;

            // Update just the affected section
            let section_content = self.doc_generator.generate_section(
                cycle.current_component(),
                &updated_cycle.component_output(cycle.current_component()),
            )?;

            // This is handled by the document service
            // (surgical section update, not full regeneration)
        }

        // 6. Publish events
        self.publisher.publish(vec![
            DomainEvent::ToolInvoked {
                cycle_id: cmd.cycle_id,
                tool_name: cmd.tool_name.clone(),
                component: cycle.current_component(),
            },
            DomainEvent::ToolCompleted {
                cycle_id: cmd.cycle_id,
                tool_name: cmd.tool_name,
                result: if response.success { "success" } else { "error" }.to_string(),
                duration_ms,
            },
        ]).await?;

        Ok(response)
    }
}
```

### GetAvailableToolsQuery

```rust
#[derive(Debug)]
pub struct GetAvailableToolsQuery {
    pub cycle_id: CycleId,
}

pub struct GetAvailableToolsHandler {
    cycle_reader: Arc<dyn CycleReader>,
    tool_executor: Arc<dyn ToolExecutor>,
}

impl GetAvailableToolsHandler {
    pub async fn handle(&self, query: GetAvailableToolsQuery) -> Result<Vec<ToolDefinition>, DomainError> {
        let cycle = self.cycle_reader.get_by_id(query.cycle_id).await?
            .ok_or_else(|| DomainError::not_found("cycle"))?;

        Ok(self.tool_executor.available_tools(cycle.current_component))
    }
}
```

---

## AI Provider Integration

### Tool Definitions for LLM

Tools are exposed to the AI provider using their native format.

#### OpenAI Format

```json
{
  "type": "function",
  "function": {
    "name": "add_objective",
    "description": "Add an objective to the decision analysis. Use when the user mentions something they want to achieve or avoid.",
    "parameters": {
      "type": "object",
      "properties": {
        "name": {
          "type": "string",
          "description": "Brief name for the objective (e.g., 'Maximize compensation', 'Minimize commute')"
        },
        "measure": {
          "type": "string",
          "description": "How to measure this objective (e.g., 'Total comp in $/year', 'Minutes per day')"
        },
        "direction": {
          "type": "string",
          "enum": ["higher", "lower", "target"],
          "description": "Whether higher values are better, lower are better, or a specific target"
        },
        "is_fundamental": {
          "type": "boolean",
          "description": "True if this is a fundamental objective (what really matters), false if it's a means objective (way to achieve something else)"
        }
      },
      "required": ["name", "measure", "direction", "is_fundamental"]
    }
  }
}
```

#### Anthropic Format

```json
{
  "name": "add_objective",
  "description": "Add an objective to the decision analysis. Use when the user mentions something they want to achieve or avoid.",
  "input_schema": {
    "type": "object",
    "properties": {
      "name": {
        "type": "string",
        "description": "Brief name for the objective"
      },
      "measure": {
        "type": "string",
        "description": "How to measure this objective"
      },
      "direction": {
        "type": "string",
        "enum": ["higher", "lower", "target"]
      },
      "is_fundamental": {
        "type": "boolean"
      }
    },
    "required": ["name", "measure", "direction", "is_fundamental"]
  }
}
```

### Tool Registration

```rust
/// Registry of all available tools
pub struct ToolRegistry {
    tools: HashMap<String, ToolDefinition>,
    component_tools: HashMap<ComponentType, Vec<String>>,
    cross_cutting_tools: Vec<String>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
            component_tools: HashMap::new(),
            cross_cutting_tools: Vec::new(),
        };

        // Register component-specific tools
        registry.register_issue_raising_tools();
        registry.register_problem_frame_tools();
        registry.register_objectives_tools();
        registry.register_alternatives_tools();
        registry.register_consequences_tools();
        registry.register_tradeoffs_tools();
        registry.register_recommendation_tools();
        registry.register_decision_quality_tools();

        // Register cross-cutting tools
        registry.register_cross_cutting_tools();

        registry
    }

    /// Get tools available for a component (component-specific + cross-cutting)
    pub fn tools_for_component(&self, component: ComponentType) -> Vec<&ToolDefinition> {
        let mut tools: Vec<&ToolDefinition> = Vec::new();

        // Add component-specific tools
        if let Some(tool_names) = self.component_tools.get(&component) {
            for name in tool_names {
                if let Some(tool) = self.tools.get(name) {
                    tools.push(tool);
                }
            }
        }

        // Add cross-cutting tools
        for name in &self.cross_cutting_tools {
            if let Some(tool) = self.tools.get(name) {
                tools.push(tool);
            }
        }

        tools
    }

    /// Convert to OpenAI tool format
    pub fn to_openai_tools(&self, component: ComponentType) -> Vec<serde_json::Value> {
        self.tools_for_component(component)
            .iter()
            .map(|tool| tool.to_openai_format())
            .collect()
    }

    /// Convert to Anthropic tool format
    pub fn to_anthropic_tools(&self, component: ComponentType) -> Vec<serde_json::Value> {
        self.tools_for_component(component)
            .iter()
            .map(|tool| tool.to_anthropic_format())
            .collect()
    }
}
```

---

## Database Schema

```sql
-- Tool invocation audit log
CREATE TABLE tool_invocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cycle_id UUID NOT NULL REFERENCES cycles(id) ON DELETE CASCADE,
    component VARCHAR(50) NOT NULL,

    -- Tool details
    tool_name VARCHAR(100) NOT NULL,
    parameters JSONB NOT NULL,

    -- Result
    result VARCHAR(20) NOT NULL,
    result_data JSONB,

    -- Context
    conversation_turn INTEGER NOT NULL,
    triggered_by TEXT,

    -- Timing
    invoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    duration_ms INTEGER NOT NULL,

    CONSTRAINT valid_result CHECK (result IN ('success', 'validation_error', 'not_found', 'conflict', 'internal_error'))
);

-- Revisit suggestions
CREATE TABLE revisit_suggestions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cycle_id UUID NOT NULL REFERENCES cycles(id) ON DELETE CASCADE,

    target_component VARCHAR(50) NOT NULL,
    reason TEXT NOT NULL,
    trigger TEXT NOT NULL,
    priority VARCHAR(20) NOT NULL,

    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    resolution TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,

    CONSTRAINT valid_priority CHECK (priority IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT valid_status CHECK (status IN ('pending', 'accepted', 'dismissed', 'expired'))
);

-- Confirmation requests
CREATE TABLE confirmation_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cycle_id UUID NOT NULL REFERENCES cycles(id) ON DELETE CASCADE,
    conversation_turn INTEGER NOT NULL,

    summary TEXT NOT NULL,
    options JSONB NOT NULL,
    default_option INTEGER,

    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    chosen_option INTEGER,
    user_input TEXT,

    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    responded_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,

    CONSTRAINT valid_status CHECK (status IN ('pending', 'confirmed', 'rejected', 'expired'))
);

-- Indexes
CREATE INDEX idx_tool_invocations_cycle ON tool_invocations(cycle_id);
CREATE INDEX idx_tool_invocations_component ON tool_invocations(cycle_id, component);
CREATE INDEX idx_tool_invocations_tool ON tool_invocations(tool_name);
CREATE INDEX idx_tool_invocations_time ON tool_invocations(invoked_at DESC);

CREATE INDEX idx_revisit_suggestions_cycle ON revisit_suggestions(cycle_id);
CREATE INDEX idx_revisit_suggestions_status ON revisit_suggestions(cycle_id, status);

CREATE INDEX idx_confirmation_requests_cycle ON confirmation_requests(cycle_id);
CREATE INDEX idx_confirmation_requests_pending ON confirmation_requests(cycle_id, status) WHERE status = 'pending';
```

---

## HTTP Endpoints

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| `POST` | `/api/cycles/:id/tools/:name` | ExecuteTool | Execute a tool |
| `GET` | `/api/cycles/:id/tools` | GetAvailableTools | List available tools |
| `GET` | `/api/cycles/:id/tools/invocations` | GetInvocations | Get tool history |
| `GET` | `/api/cycles/:id/revisits` | GetPendingRevisits | List pending revisits |
| `POST` | `/api/cycles/:id/revisits/:id/resolve` | ResolveRevisit | Accept/dismiss revisit |
| `GET` | `/api/cycles/:id/confirmations/pending` | GetPendingConfirmations | Get pending confirmations |
| `POST` | `/api/cycles/:id/confirmations/:id/respond` | RespondToConfirmation | Respond to confirmation |

### Request/Response DTOs

```rust
// POST /api/cycles/:id/tools/:name
#[derive(Debug, Deserialize)]
pub struct ExecuteToolRequest {
    pub parameters: serde_json::Value,
    pub conversation_turn: u32,
    pub trigger_context: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecuteToolResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub document_updated: bool,
    pub invocation_id: String,
}

// GET /api/cycles/:id/tools
#[derive(Debug, Serialize)]
pub struct AvailableToolsResponse {
    pub current_component: String,
    pub tools: Vec<ToolDefinitionDto>,
}

#[derive(Debug, Serialize)]
pub struct ToolDefinitionDto {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub returns: serde_json::Value,
}

// GET /api/cycles/:id/revisits
#[derive(Debug, Serialize)]
pub struct PendingRevisitsResponse {
    pub revisits: Vec<RevisitSuggestionDto>,
    pub has_critical: bool,
}

#[derive(Debug, Serialize)]
pub struct RevisitSuggestionDto {
    pub id: String,
    pub target_component: String,
    pub reason: String,
    pub priority: String,
    pub created_at: String,
}
```

---

## Emergent Behavior Examples

The power of atomic tools lies in how the agent composes them.

### Example 1: Detecting Repeated Themes

```
User: "I really care about not having a long commute."
User: "The commute is important because I want time with my kids."
User: "One concern with option B is the commute."

Agent recognizes pattern, invokes:
1. add_objective("Minimize commute time", "Minutes per day", "lower", true)
2. link_means_to_fundamental(<commute_id>, <family_time_id>)

Agent says: "I notice commute keeps coming up. I've added 'Minimize commute time'
as an objective. It seems like this matters because of family time - should I
link these together?"
```

### Example 2: Proactive Gap Detection

```
[After user finishes Alternatives but before moving to Consequences]

Agent invokes:
1. get_document_summary()  // Sees: 4 alternatives, 5 objectives
2. compute_pugh_totals()   // Returns: matrix is empty

Agent invokes:
3. suggest_revisit("Consequences", "Matrix is empty - alternatives exist but haven't been evaluated", "high")

Agent says: "Before we proceed, I notice we have 4 alternatives and 5 objectives
but haven't evaluated how each alternative performs. Ready to work through the
consequence matrix?"
```

### Example 3: Auto-Detecting Dominated Alternatives

```
[During Tradeoffs component]

Agent invokes:
1. find_dominated_alternatives()

Returns:
{
  "dominated": [
    {
      "alternative_id": "alt_c",
      "alternative_name": "Option C: Hybrid Role",
      "dominated_by": "alt_a",
      "dominated_by_name": "Option A: Accept VP",
      "reason": "Option A is equal or better on all objectives, and strictly better on Compensation and Growth"
    }
  ]
}

Agent invokes:
2. mark_dominated("alt_c", "alt_a", "Equal or better on all objectives, strictly better on Compensation (+2 vs +1) and Growth (+2 vs 0)")

Agent says: "Looking at the consequence matrix, Option C (Hybrid Role) appears
to be dominated by Option A (Accept VP). Option A matches or beats it on every
objective. Should we set this aside and focus on the remaining alternatives?"
```

### Example 4: Sensitivity Check

```
User: "I'm not sure about giving Accept VP a +2 on growth - maybe it should be +1"

Agent invokes:
1. sensitivity_check("alt_a", "obj_growth")

Returns:
{
  "current_rating": 2,
  "if_changed_to": {
    "1": { "total_change": -1, "ranking_change": false, "still_leads": true },
    "0": { "total_change": -2, "ranking_change": true, "new_leader": "alt_b" },
    "-1": { "total_change": -3, "ranking_change": true, "new_leader": "alt_b" }
  }
}

Agent says: "Good instinct to question that. If we change growth to +1, Option A
still leads overall. But if growth is actually neutral (0) or negative, Option B
would become the frontrunner. How confident are you in the growth potential?"
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_objective_validates_parameters() {
        let executor = ToolExecutorImpl::new();
        let tool = ToolCall {
            name: "add_objective".to_string(),
            parameters: json!({
                "name": "",  // Invalid: empty
                "measure": "$/year",
                "direction": "higher",
                "is_fundamental": true,
            }),
        };

        let result = executor.validate(&tool);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name"));
    }

    #[test]
    fn test_rate_consequence_updates_matrix() {
        let executor = ToolExecutorImpl::new();
        let mut context = create_test_context();
        context.objective_ids = vec!["obj_1".to_string()];
        context.alternative_ids = vec!["alt_a".to_string()];

        let tool = ToolCall {
            name: "rate_consequence".to_string(),
            parameters: json!({
                "alternative_id": "alt_a",
                "objective_id": "obj_1",
                "rating": 2,
                "reasoning": "Much better due to higher salary"
            }),
        };

        let result = executor.execute(tool, context).await.unwrap();
        assert!(result.success);
        assert!(result.document_updated);
    }

    #[test]
    fn test_find_dominated_detects_dominance() {
        let analyzer = PughAnalyzer::new();
        let matrix = create_test_matrix();

        // alt_c is dominated by alt_a (equal or better on all, strictly better on some)
        let dominated = analyzer.find_dominated(&matrix);

        assert_eq!(dominated.len(), 1);
        assert_eq!(dominated[0].alternative_id, "alt_c");
        assert_eq!(dominated[0].dominated_by, "alt_a");
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_tool_execution_updates_document() {
    let app = test_app().await;
    let cycle_id = create_test_cycle(&app).await;

    // Execute add_objective tool
    let response = app.post(&format!("/api/cycles/{}/tools/add_objective", cycle_id))
        .json(&json!({
            "parameters": {
                "name": "Maximize compensation",
                "measure": "Total comp ($/year)",
                "direction": "higher",
                "is_fundamental": true
            },
            "conversation_turn": 1
        }))
        .await;

    assert!(response.status().is_success());
    let body: ExecuteToolResponse = response.json().await;
    assert!(body.document_updated);

    // Verify document was updated
    let doc = get_document(&app, cycle_id).await;
    assert!(doc.content.contains("Maximize compensation"));
}

#[tokio::test]
async fn test_tool_invocation_is_logged() {
    let app = test_app().await;
    let cycle_id = create_test_cycle(&app).await;

    // Execute a tool
    execute_tool(&app, cycle_id, "add_objective", json!({
        "name": "Test objective",
        "measure": "Count",
        "direction": "higher",
        "is_fundamental": true
    })).await;

    // Check invocation log
    let invocations = get_invocations(&app, cycle_id).await;
    assert_eq!(invocations.len(), 1);
    assert_eq!(invocations[0].tool_name, "add_objective");
    assert_eq!(invocations[0].result, "success");
}
```

---

## Implementation Phases

### Phase 1: Core Infrastructure
- ToolInvocation entity and repository
- ToolExecutor port and basic implementation
- Tool registry with JSON Schema validation
- Database migrations

### Phase 2: Component Tools
- Issue Raising tools (5 tools)
- Problem Frame tools (7 tools)
- Objectives tools (5 tools)
- Alternatives tools (5 tools)

### Phase 3: Consequences & Analysis
- Consequences tools (4 tools)
- Analysis tools (compute, find dominated, sensitivity)
- Batch operations

### Phase 4: Tradeoffs & Recommendation
- Tradeoffs tools (4 tools)
- Recommendation tools (4 tools)
- Decision Quality tools (3 tools)

### Phase 5: Cross-Cutting Tools
- Uncertainty management (3 tools)
- Revisit suggestions (3 tools)
- User confirmation (2 tools)
- Document operations (3 tools)

### Phase 6: AI Provider Integration
- OpenAI tool format conversion
- Anthropic tool format conversion
- Tool result handling in conversation flow
- Context injection optimization

### Phase 7: HTTP Layer
- Tool execution endpoint
- Available tools endpoint
- Invocation history endpoint
- Revisit and confirmation endpoints

### Phase 8: Testing & Polish
- Comprehensive unit tests
- Integration tests
- Emergent behavior tests
- Performance benchmarks
- Documentation

---

## Related Documents

- [Agent-Native Enrichments Analysis](../../docs/architecture/AGENT-NATIVE-ENRICHMENTS.md)
- [Decision Document Specification](../cycle/decision-document.md)
- [Conversation Module](../../docs/modules/conversation.md)
- [PrOACT Component Types](../proact-types/component-schemas.md)

---

*Specification Version: 1.0.0*
*Created: 2026-01-09*
*Author: Claude Opus 4.5*
