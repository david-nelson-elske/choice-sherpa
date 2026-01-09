# Choice Sherpa: System Architecture

> **Version**: 1.3.0
> **Created**: 2026-01-07
> **Updated**: 2026-01-08 (Scaling Readiness)
> **Architecture Style**: Hexagonal (Ports & Adapters)
> **Development Method**: TDD

---

## Executive Summary

Choice Sherpa is an interactive decision support application that guides users through the PrOACT framework via conversational AI. The system is architected as a living dashboard that captures, organizes, and presents decision-relevant information through structured conversations.

### Key Architectural Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Architecture | Hexagonal | Clean separation of domain logic from infrastructure |
| State Management | Event Sourced | Preserves conversation history and enables cycle branching |
| AI Integration | Port-based | Allows swapping LLM providers without domain changes |
| Data Model | Aggregate-per-Cycle | Each cycle owns its components as child entities |
| Frontend | SvelteKit + Module-aligned | UI modules mirror backend bounded contexts |

---

## Module Classification

The system has three types of modules:

| Type | Purpose | Has Ports? | Has Adapters? | Examples |
|------|---------|------------|---------------|----------|
| **Shared Domain** | Type definitions, value objects | No | No | foundation, proact-types |
| **Full Module** | Complete bounded context | Yes | Yes | session, cycle, conversation, dashboard |
| **Domain Services** | Stateless business logic | No | Optional | analysis |

---

## Module Inventory

| Module | Type | Responsibility | Dependencies |
|--------|------|---------------|--------------|
| `foundation` | Shared Domain | Value objects, IDs, enums, errors | None |
| `proact-types` | Shared Domain | Component interface, 9 PrOACT types | foundation |
| `session` | Full Module | Session lifecycle, user ownership | foundation, membership (access check) |
| `membership` | Full Module | Subscriptions, access control, payments | foundation |
| `cycle` | Full Module | Cycles, components, branching, navigation | foundation, proact-types, session |
| `conversation` | Full Module | AI agent behavior, message handling | foundation, proact-types |
| `analysis` | Domain Services | Pugh matrix, DQ scoring, calculations | foundation, proact-types |
| `dashboard` | Full Module | Read models, views, aggregations | all modules |

### Dependency Graph

```
                         ┌─────────────┐
                         │  dashboard  │ (Phase 5)
                         └──────┬──────┘
                                │
            ┌───────────────────┼───────────────────┐
            │                   │                   │
     ┌──────▼──────┐     ┌──────▼──────┐     ┌──────▼──────┐
     │conversation │     │  analysis   │     │    cycle    │ (Phase 4)
     └──────┬──────┘     └──────┬──────┘     └──────┬──────┘
            │                   │                   │
            └───────────────────┼───────────────────┘
                                │
                         ┌──────▼──────┐
                         │ proact-types│ (Phase 3)
                         └──────┬──────┘
                                │
            ┌───────────────────┼───────────────────┐
            │                   │                   │
     ┌──────▼──────┐     ┌──────▼──────┐            │
     │   session   │────►│ membership  │            │
     └─────────────┘     └──────┬──────┘            │
       (Phase 3)           (Phase 2)                │
            │                   │                   │
            └───────────────────┴───────────────────┘
                                │
                         ┌──────▼──────┐
                         │ foundation  │ (Phase 1)
                         └─────────────┘
```

**Note:** Session depends on Membership via the `AccessChecker` port for gating session creation.
External payment provider (Stripe) is accessed via the `PaymentProvider` port in the membership module.

### Build Order

1. **Phase 1**: foundation
2. **Phase 2**: membership, proact-types (parallel)
3. **Phase 3**: session (depends on membership AccessChecker)
4. **Phase 4**: cycle, conversation, analysis (parallel)
5. **Phase 5**: dashboard

---

## Aggregate Boundaries

### Key Design Decision: Cycle Owns Components

```
┌─────────────────────────────────────────────────────────────┐
│                     Session Aggregate                        │
│  - SessionID (identity)                                     │
│  - Title, Description                                       │
│  - CycleIDs[] (references only, not embedded)               │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ references
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Cycle Aggregate                         │
│  - CycleID (identity)                                       │
│  - SessionID (parent reference)                             │
│  - ParentCycleID (for branching)                            │
│  - Components[] (EMBEDDED child entities)                   │
│      └── IssueRaising                                       │
│      └── ProblemFrame                                       │
│      └── Objectives                                         │
│      └── ... (all 9 component types)                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ referenced by (ComponentID)
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   Conversation Entity                        │
│  - ConversationID (identity)                                │
│  - ComponentID (links to specific component in cycle)       │
│  - Messages[] (owned by conversation)                       │
└─────────────────────────────────────────────────────────────┘
```

**Rationale:**
1. Components don't exist independently of cycles
2. Branching copies entire cycle state atomically
3. Deleting a cycle deletes its components
4. Simpler transaction boundaries

---

## Module: foundation (Shared Domain)

### Purpose
Shared domain primitives used across all modules. Contains value objects, base error types, and common enums.

### Domain Layer

#### Value Objects

| Value Object | Description | Validation Rules |
|--------------|-------------|------------------|
| `SessionID` | Unique session identifier | UUID format |
| `CycleID` | Unique cycle identifier | UUID format |
| `ComponentID` | Unique component identifier | UUID format |
| `UserID` | User identifier | Non-empty string |
| `Timestamp` | Immutable point in time | Valid datetime |
| `Percentage` | 0-100 scale value | 0 ≤ value ≤ 100 |
| `Rating` | Pugh matrix rating | -2 to +2 integer |

#### Enums

```rust
use serde::{Deserialize, Serialize};

/// ComponentType - The 9 PrOACT phases (including Issue Raising and Notes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    IssueRaising,
    ProblemFrame,
    Objectives,
    Alternatives,
    Consequences,
    Tradeoffs,
    Recommendation,
    DecisionQuality,
    NotesNextSteps,
}

impl ComponentType {
    /// Returns all component types in order
    pub fn all() -> &'static [ComponentType] {
        &[
            ComponentType::IssueRaising,
            ComponentType::ProblemFrame,
            ComponentType::Objectives,
            ComponentType::Alternatives,
            ComponentType::Consequences,
            ComponentType::Tradeoffs,
            ComponentType::Recommendation,
            ComponentType::DecisionQuality,
            ComponentType::NotesNextSteps,
        ]
    }
}

/// ComponentStatus - Progress tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentStatus {
    #[default]
    NotStarted,
    InProgress,
    Complete,
    NeedsRevision,
}

/// CycleStatus - Cycle lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CycleStatus {
    #[default]
    Active,
    Completed,
    Archived,
}

/// SessionStatus - Session lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    #[default]
    Active,
    Archived,
}
```

#### Error Types

```rust
use thiserror::Error;

/// Domain error categories
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid state transition: {0}")]
    InvalidState(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("validation failed: {0}")]
    Validation(String),
}
```

### File Structure

```
backend/src/domain/foundation/
├── mod.rs              # Module exports
├── ids.rs              # SessionId, CycleId, ComponentId, UserId
├── ids_test.rs
├── timestamp.rs        # Timestamp value object
├── timestamp_test.rs
├── percentage.rs       # Percentage (0-100)
├── percentage_test.rs
├── rating.rs           # Pugh Rating (-2 to +2)
├── rating_test.rs
├── component_type.rs   # ComponentType enum
├── component_status.rs # ComponentStatus enum
├── cycle_status.rs     # CycleStatus enum
├── session_status.rs   # SessionStatus enum
└── errors.rs           # Base domain errors

frontend/src/shared/domain/
├── ids.ts              # ID type definitions
├── ids.test.ts
├── enums.ts            # ComponentType, Status enums
├── enums.test.ts
└── errors.ts           # Error type definitions
```

---

## Module: proact-types (Shared Domain)

### Purpose
Defines the 9 PrOACT component types and their structured outputs. These are domain types used by the `cycle` module.

**Note**: This is a shared domain library, not a full module. Components are owned and persisted by the `cycle` module.

### Domain Layer

#### Component Interface

```rust
use crate::foundation::{ComponentId, ComponentStatus, ComponentType, Timestamp};

/// Component is the trait all PrOACT components implement
pub trait Component {
    fn id(&self) -> ComponentId;
    fn component_type(&self) -> ComponentType;
    fn status(&self) -> ComponentStatus;

    // Lifecycle
    fn start(&mut self) -> Result<(), DomainError>;
    fn complete(&mut self) -> Result<(), DomainError>;
    fn mark_for_revision(&mut self, reason: &str) -> Result<(), DomainError>;

    // Content
    fn output_as_value(&self) -> serde_json::Value;
    fn set_output_from_value(&mut self, data: serde_json::Value) -> Result<(), DomainError>;
}

/// ComponentBase provides common fields for all components
#[derive(Debug, Clone)]
pub struct ComponentBase {
    pub id: ComponentId,
    pub component_type: ComponentType,
    pub status: ComponentStatus,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
```

#### Message Type (Shared)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Message represents a single message in component conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub role: Role,
    pub content: String,
    pub metadata: MessageMetadata,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub token_count: Option<i32>,
    pub extracted_data: HashMap<String, serde_json::Value>,
}
```

#### Component Types (9 total)

##### 1. IssueRaising

```rust
#[derive(Debug, Clone)]
pub struct IssueRaising {
    pub base: ComponentBase,
    pub output: IssueRaisingOutput,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IssueRaisingOutput {
    /// Things that need to be chosen
    pub potential_decisions: Vec<String>,
    /// Things that matter
    pub objectives: Vec<String>,
    /// Things unknown
    pub uncertainties: Vec<String>,
    /// Process constraints, facts, stakeholders
    pub considerations: Vec<String>,
    /// User validated categorization
    pub user_confirmed: bool,
}
```

##### 2. ProblemFrame

```rust
#[derive(Debug, Clone)]
pub struct ProblemFrame {
    pub base: ComponentBase,
    pub output: ProblemFrameOutput,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProblemFrameOutput {
    pub decision_maker: Option<String>,
    pub focal_decision: Option<String>,
    pub ultimate_aim: Option<String>,
    pub temporal_constraint: Option<Timestamp>,
    pub spatial_scope: Option<String>,
    pub linked_decisions: Vec<LinkedDecision>,
    pub other_constraints: Vec<Constraint>,
    pub affected_parties: Vec<Party>,
    pub expert_sources: Vec<String>,
    pub decision_hierarchy: DecisionHierarchy,
    pub decision_statement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedDecision {
    pub description: String,
    pub relationship: String, // "enables", "constrains", "depends_on"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub constraint_type: String, // "legal", "financial", "political", "technical"
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Party {
    pub id: String,
    pub name: String,
    pub role: String,
    pub objectives: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionHierarchy {
    pub already_made: Vec<String>,
    pub focal_decisions: Vec<String>,
    pub deferred: Vec<String>,
}
```

##### 3. Objectives

```rust
#[derive(Debug, Clone)]
pub struct Objectives {
    pub base: ComponentBase,
    pub output: ObjectivesOutput,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObjectivesOutput {
    pub fundamental_objectives: Vec<FundamentalObjective>,
    pub means_objectives: Vec<MeansObjective>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundamentalObjective {
    pub id: String,
    pub description: String,
    pub performance_measure: PerformanceMeasure,
    pub affected_party_id: Option<String>, // Links to Party from ProblemFrame
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeansObjective {
    pub id: String,
    pub description: String,
    pub contributes_to_objective_id: String, // Which fundamental it supports
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMeasure {
    pub description: String,
    pub is_quantitative: bool,
    pub unit: Option<String>, // e.g., "dollars", "days"
    pub direction: String,    // "higher_is_better" or "lower_is_better"
}
```

##### 4. Alternatives

```rust
#[derive(Debug, Clone)]
pub struct Alternatives {
    pub base: ComponentBase,
    pub output: AlternativesOutput,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlternativesOutput {
    pub options: Vec<Alternative>,
    pub strategy_table: Option<StrategyTable>,
    pub has_status_quo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub id: String,
    pub name: String,
    pub description: String,
    pub assumptions: Vec<String>,
    pub is_status_quo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyTable {
    pub decisions: Vec<DecisionColumn>,
    pub strategies: Vec<Strategy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionColumn {
    pub decision_name: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub id: String,
    pub name: String,
    pub choices: HashMap<String, String>, // DecisionName -> Option
}
```

##### 5. Consequences

```rust
#[derive(Debug, Clone)]
pub struct Consequences {
    pub base: ComponentBase,
    pub output: ConsequencesOutput,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsequencesOutput {
    pub table: ConsequencesTable,
    pub uncertainties: Vec<Uncertainty>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsequencesTable {
    pub alternative_ids: Vec<String>,
    pub objective_ids: Vec<String>,
    pub cells: HashMap<String, HashMap<String, Cell>>, // [alt_id][obj_id] -> Cell
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub rating: Rating,            // Pugh: -2 to +2
    pub explanation: String,
    pub quant_value: Option<f64>,
    pub quant_unit: Option<String>,
    pub source: Option<String>,    // Citation
    pub uncertainty: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Uncertainty {
    pub id: String,
    pub description: String,
    pub driver: String,           // What causes it
    pub worth_resolving: bool,    // Value of information assessment
    pub resolvable: bool,         // Can be reduced in timeframe
}
```

##### 6. Tradeoffs

```rust
#[derive(Debug, Clone)]
pub struct Tradeoffs {
    pub base: ComponentBase,
    pub output: TradeoffsOutput,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TradeoffsOutput {
    pub dominated_alternatives: Vec<DominatedAlternative>,
    pub irrelevant_objectives: Vec<IrrelevantObjective>,
    pub tensions: Vec<Tension>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DominatedAlternative {
    pub alternative_id: String,
    pub dominated_by_id: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrrelevantObjective {
    pub objective_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tension {
    pub alternative_id: String,
    pub gains: Vec<String>,   // Objectives where this alt excels
    pub losses: Vec<String>,  // Objectives where this alt suffers
    pub uncertainty_impact: Option<String>,
}
```

##### 7. PreliminaryRecommendation

```rust
#[derive(Debug, Clone)]
pub struct Recommendation {
    pub base: ComponentBase,
    pub output: RecommendationOutput,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecommendationOutput {
    pub standout_option: Option<String>, // AlternativeID if one stands out
    pub synthesis: Option<String>,       // Summary of analysis
    pub caveats: Vec<String>,            // Important qualifications
    pub additional_info: Vec<String>,    // What more might help
}
```

##### 8. DecisionQuality

```rust
#[derive(Debug, Clone)]
pub struct DecisionQuality {
    pub base: ComponentBase,
    pub output: DecisionQualityOutput,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionQualityOutput {
    pub elements: Vec<DQElement>,
    pub overall_score: Percentage,      // Min of all elements
    pub improvement_paths: Vec<String>, // What would raise lowest scores
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DQElement {
    pub name: String,          // e.g., "Helpful Problem Frame"
    pub score: Percentage,     // 0-100
    pub rationale: String,
    pub improvement: String,   // What would raise it
}

/// The 7 DQ Elements (standard)
pub const DQ_ELEMENT_NAMES: [&str; 7] = [
    "Helpful Problem Frame",
    "Clear Objectives",
    "Creative Alternatives",
    "Reliable Consequence Information",
    "Logically Correct Reasoning",
    "Clear Tradeoffs",
    "Commitment to Follow Through",
];
```

##### 9. NotesNextSteps

```rust
#[derive(Debug, Clone)]
pub struct NotesNextSteps {
    pub base: ComponentBase,
    pub output: NotesNextStepsOutput,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotesNextStepsOutput {
    pub remaining_uncertainties: Vec<String>,
    pub open_questions: Vec<String>,
    pub planned_actions: Vec<PlannedAction>,
    pub affirmation: Option<String>,          // If DQ is 100%
    pub further_analysis_paths: Vec<String>,  // If DQ < 100%
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedAction {
    pub description: String,
    pub due_date: Option<Timestamp>,
    pub owner: Option<String>,
}
```

### File Structure

```
backend/src/domain/proact/
├── mod.rs                # Module exports
├── component.rs          # Component trait + ComponentBase
├── component_test.rs
├── component_variant.rs  # ComponentVariant enum
├── message.rs            # Message type (shared)
├── message_test.rs
├── issue_raising.rs
├── issue_raising_test.rs
├── problem_frame.rs
├── problem_frame_test.rs
├── objectives.rs
├── objectives_test.rs
├── alternatives.rs
├── alternatives_test.rs
├── consequences.rs
├── consequences_test.rs
├── tradeoffs.rs
├── tradeoffs_test.rs
├── recommendation.rs
├── recommendation_test.rs
├── decision_quality.rs
├── decision_quality_test.rs
├── notes_next_steps.rs
├── notes_next_steps_test.rs
└── errors.rs             # proact-specific errors

frontend/src/shared/proact/
├── component.ts          # Component interface
├── message.ts            # Message type
├── issue-raising.ts
├── problem-frame.ts
├── objectives.ts
├── alternatives.ts
├── consequences.ts
├── tradeoffs.ts
├── recommendation.ts
├── decision-quality.ts
├── notes-next-steps.ts
└── index.ts              # Public exports
```

---

## Module: session (Full Module)

### Purpose
Manages the top-level Decision Session - the container for all cycles exploring a single decision context.

### Domain Layer

#### Aggregate: Session

```rust
pub struct Session {
    id: SessionId,
    user_id: UserId,
    title: String,
    description: Option<String>,
    created_at: Timestamp,
    updated_at: Timestamp,
    status: SessionStatus,
    cycle_ids: Vec<CycleId>,        // References only, not embedded
    domain_events: Vec<DomainEvent>,
}

impl Session {
    pub fn new(user_id: UserId, title: String) -> Result<Self, DomainError>;
    pub fn rename(&mut self, title: String) -> Result<(), DomainError>;
    pub fn update_description(&mut self, desc: Option<String>) -> Result<(), DomainError>;
    pub fn add_cycle(&mut self, cycle_id: CycleId) -> Result<(), DomainError>;
    pub fn archive(&mut self) -> Result<(), DomainError>;
    pub fn is_owner(&self, user_id: &UserId) -> bool;
    pub fn pull_domain_events(&mut self) -> Vec<DomainEvent>;
}
```

#### Invariants

1. Session must have a non-empty title
2. Only session owner can modify
3. Archived sessions are immutable
4. CycleIDs list is append-only (cycles can be archived but not removed)

#### Domain Events

| Event | Trigger | Data |
|-------|---------|------|
| `SessionCreated` | New session | sessionID, userID, title |
| `SessionRenamed` | Title change | sessionID, oldTitle, newTitle |
| `CycleAddedToSession` | New cycle linked | sessionID, cycleID |
| `SessionArchived` | Session archived | sessionID |

### Ports

```rust
use async_trait::async_trait;

/// Write operations (Command side)
#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn save(&self, session: &Session) -> Result<(), DomainError>;
    async fn update(&self, session: &Session) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: SessionId) -> Result<Option<Session>, DomainError>;
}

/// Read operations (Query side - CQRS)
#[async_trait]
pub trait SessionReader: Send + Sync {
    async fn get_by_id(&self, id: SessionId) -> Result<Option<SessionView>, DomainError>;
    async fn list_by_user(&self, user_id: &UserId, filter: SessionFilter) -> Result<Vec<SessionSummary>, DomainError>;
    async fn search(&self, user_id: &UserId, query: &str) -> Result<Vec<SessionSummary>, DomainError>;
}

/// View models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionView {
    pub id: SessionId,
    pub title: String,
    pub description: Option<String>,
    pub status: SessionStatus,
    pub cycle_count: i32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: SessionId,
    pub title: String,
    pub status: SessionStatus,
    pub cycle_count: i32,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Default)]
pub struct SessionFilter {
    pub status: Option<SessionStatus>,
    pub limit: i32,
    pub offset: i32,
    pub order_by: String, // "updated_at", "created_at", "title"
}
```

### Application Layer

#### Commands

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `CreateSession` | userID, title | SessionID | Creates session |
| `RenameSession` | sessionID, newTitle | - | Updates title |
| `ArchiveSession` | sessionID | - | Archives session |

```rust
#[derive(Debug)]
pub struct CreateSessionCommand {
    pub user_id: UserId,
    pub title: String,
}

pub struct CreateSessionHandler {
    repo: Arc<dyn SessionRepository>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl CreateSessionHandler {
    pub async fn handle(&self, cmd: CreateSessionCommand) -> Result<SessionId, DomainError> {
        let mut session = Session::new(cmd.user_id, cmd.title)?;

        self.repo.save(&session).await?;
        self.publisher.publish(session.pull_domain_events()).await?;

        Ok(session.id())
    }
}
```

#### Queries

| Query | Input | Output |
|-------|-------|--------|
| `GetSession` | sessionID | SessionView |
| `ListUserSessions` | userID, filter | []SessionSummary |
| `SearchSessions` | userID, query | []SessionSummary |

### Adapters

#### HTTP Endpoints

| Method | Path | Handler | Auth |
|--------|------|---------|------|
| `POST` | `/api/sessions` | CreateSession | Required |
| `GET` | `/api/sessions` | ListUserSessions | Required |
| `GET` | `/api/sessions/:id` | GetSession | Owner |
| `PATCH` | `/api/sessions/:id` | RenameSession | Owner |
| `DELETE` | `/api/sessions/:id` | ArchiveSession | Owner |

#### Database Schema

```sql
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT sessions_title_not_empty CHECK (title <> '')
);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_updated_at ON sessions(updated_at DESC);
```

### File Structure

```
backend/src/domain/session/
├── mod.rs              # Module exports
├── session.rs          # Session aggregate
├── session_test.rs
├── errors.rs           # Session-specific errors
└── events.rs           # Domain events

backend/src/ports/
├── mod.rs
├── session_repository.rs
└── session_reader.rs

backend/src/application/
├── mod.rs
├── commands/
│   ├── mod.rs
│   ├── create_session.rs
│   ├── create_session_test.rs
│   ├── rename_session.rs
│   ├── rename_session_test.rs
│   ├── archive_session.rs
│   └── archive_session_test.rs
└── queries/
    ├── mod.rs
    ├── get_session.rs
    ├── get_session_test.rs
    ├── list_sessions.rs
    └── list_sessions_test.rs

backend/src/adapters/
├── http/session/
│   ├── mod.rs
│   ├── handlers.rs
│   ├── handlers_test.rs
│   ├── dto.rs
│   └── routes.rs
└── postgres/
    ├── mod.rs
    ├── session_repository.rs
    ├── session_repository_test.rs
    └── session_reader.rs

frontend/src/modules/session/
├── domain/
│   ├── session.ts
│   └── session.test.ts
├── api/
│   ├── session-api.ts
│   ├── use-sessions.ts
│   └── use-session.ts
├── components/
│   ├── SessionList.svelte
│   ├── SessionList.test.ts
│   ├── SessionCard.svelte
│   ├── SessionCard.test.ts
│   └── CreateSessionDialog.svelte
└── index.ts
```

---

## Module: cycle (Full Module)

### Purpose
Manages the Cycle aggregate - a complete or partial path through PrOACT. **Owns and persists all components as child entities.**

### Domain Layer

#### Aggregate: Cycle

```rust
use std::collections::HashMap;

pub struct Cycle {
    id: CycleId,
    session_id: SessionId,
    parent_cycle_id: Option<CycleId>,   // None for root cycle
    branch_point: Option<ComponentType>, // Where it branched from parent
    created_at: Timestamp,
    status: CycleStatus,

    // Components are OWNED by cycle (not just referenced)
    components: HashMap<ComponentType, ComponentVariant>,
    current_step: ComponentType,

    domain_events: Vec<DomainEvent>,
}

impl Cycle {
    pub fn new(session_id: SessionId) -> Result<Self, DomainError>;
    pub fn branch_at(parent: &Cycle, branch_at: ComponentType) -> Result<Self, DomainError>;

    // Business methods
    pub fn get_component(&self, t: ComponentType) -> Result<&ComponentVariant, DomainError>;
    pub fn start_component(&mut self, t: ComponentType) -> Result<(), DomainError>;
    pub fn complete_component(&mut self, t: ComponentType) -> Result<(), DomainError>;
    pub fn update_component_output(&mut self, t: ComponentType, output: serde_json::Value) -> Result<(), DomainError>;
    pub fn navigate_to(&mut self, t: ComponentType) -> Result<(), DomainError>;
    pub fn get_progress(&self) -> CycleProgress;
    pub fn complete(&mut self) -> Result<(), DomainError>;
    pub fn archive(&mut self) -> Result<(), DomainError>;
    pub fn can_branch_at(&self, t: ComponentType) -> bool;
    pub fn pull_domain_events(&mut self) -> Vec<DomainEvent>;
}
```

#### Value Objects

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleProgress {
    pub total_steps: i32,
    pub completed_steps: i32,
    pub current_step: ComponentType,
    pub step_statuses: HashMap<ComponentType, ComponentStatus>,
}

impl CycleProgress {
    pub fn percent_complete(&self) -> i32 {
        if self.total_steps == 0 { return 0; }
        (self.completed_steps * 100) / self.total_steps
    }
}
```

#### Invariants

1. Components follow defined order (can skip forward, not back)
2. Only one component can be "in_progress" at a time
3. Branch point must be a started/completed component
4. Branched cycle inherits component state up to branch point
5. Cycle must belong to a session

#### Domain Events

| Event | Trigger | Data |
|-------|---------|------|
| `CycleCreated` | New cycle | cycleID, sessionID |
| `CycleBranched` | Branch created | cycleID, parentCycleID, branchPoint |
| `ComponentStarted` | Component work begins | cycleID, componentType |
| `ComponentCompleted` | Component finished | cycleID, componentType |
| `ComponentOutputUpdated` | Structured data changed | cycleID, componentType |
| `CycleCompleted` | All components done | cycleID |

### Ports

```rust
/// Write operations
#[async_trait]
pub trait CycleRepository: Send + Sync {
    async fn save(&self, cycle: &Cycle) -> Result<(), DomainError>;
    async fn update(&self, cycle: &Cycle) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: CycleId) -> Result<Option<Cycle>, DomainError>;
    async fn find_by_session(&self, session_id: SessionId) -> Result<Vec<Cycle>, DomainError>;
}

/// Read operations (CQRS)
#[async_trait]
pub trait CycleReader: Send + Sync {
    async fn get_by_id(&self, id: CycleId) -> Result<Option<CycleView>, DomainError>;
    async fn get_cycle_tree(&self, session_id: SessionId) -> Result<CycleTree, DomainError>;
    async fn get_component_view(&self, cycle_id: CycleId, comp_type: ComponentType) -> Result<Option<ComponentView>, DomainError>;
}

/// View models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleView {
    pub id: CycleId,
    pub session_id: SessionId,
    pub parent_cycle_id: Option<CycleId>,
    pub branch_point: Option<ComponentType>,
    pub status: CycleStatus,
    pub progress: CycleProgress,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleTree {
    pub root_cycles: Vec<CycleNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleNode {
    pub cycle_id: CycleId,
    pub status: CycleStatus,
    pub progress: CycleProgress,
    pub branch_point: Option<ComponentType>,
    pub children: Vec<CycleNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentView {
    pub id: ComponentId,
    pub cycle_id: CycleId,
    pub component_type: ComponentType,
    pub status: ComponentStatus,
    pub structured_output: serde_json::Value,
    pub updated_at: Timestamp,
}
```

### Application Layer

#### Commands

| Command | Description |
|---------|-------------|
| `CreateCycle` | Start new root cycle in session |
| `BranchCycle` | Create branch from existing cycle |
| `StartComponent` | Begin work on a component |
| `CompleteComponent` | Mark component as done |
| `UpdateComponentOutput` | Save structured data |
| `NavigateToComponent` | Change current step |
| `CompleteCycle` | Mark cycle as finished |

#### Queries

| Query | Returns |
|-------|---------|
| `GetCycle` | Full cycle with progress |
| `GetCycleTree` | Session's cycle hierarchy |
| `GetComponent` | Single component view |

### Adapters

#### HTTP Endpoints

| Method | Path | Handler |
|--------|------|---------|
| `POST` | `/api/sessions/:sessionId/cycles` | CreateCycle |
| `GET` | `/api/sessions/:sessionId/cycles` | GetCycleTree |
| `GET` | `/api/cycles/:id` | GetCycle |
| `POST` | `/api/cycles/:id/branch` | BranchCycle |
| `POST` | `/api/cycles/:id/navigate` | NavigateToComponent |
| `POST` | `/api/cycles/:id/complete` | CompleteCycle |
| `GET` | `/api/cycles/:id/components/:type` | GetComponent |
| `POST` | `/api/cycles/:id/components/:type/start` | StartComponent |
| `POST` | `/api/cycles/:id/components/:type/complete` | CompleteComponent |
| `PUT` | `/api/cycles/:id/components/:type` | UpdateComponentOutput |

#### Database Schema

```sql
CREATE TABLE cycles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    parent_cycle_id UUID REFERENCES cycles(id),
    branch_point VARCHAR(50),
    current_step VARCHAR(50) NOT NULL DEFAULT 'issue_raising',
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE components (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cycle_id UUID NOT NULL REFERENCES cycles(id) ON DELETE CASCADE,
    component_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'not_started',
    structured_data JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(cycle_id, component_type)
);

CREATE INDEX idx_cycles_session ON cycles(session_id);
CREATE INDEX idx_cycles_parent ON cycles(parent_cycle_id);
CREATE INDEX idx_components_cycle ON components(cycle_id);
CREATE INDEX idx_components_type ON components(component_type);
```

### File Structure

```
backend/src/domain/cycle/
├── mod.rs
├── cycle.rs            # Cycle aggregate
├── cycle_test.rs
├── progress.rs         # CycleProgress value object
├── progress_test.rs
├── errors.rs
└── events.rs

backend/src/ports/
├── mod.rs
├── cycle_repository.rs
└── cycle_reader.rs

backend/src/application/
├── mod.rs
├── commands/
│   ├── mod.rs
│   ├── create_cycle.rs
│   ├── create_cycle_test.rs
│   ├── branch_cycle.rs
│   ├── branch_cycle_test.rs
│   ├── start_component.rs
│   ├── complete_component.rs
│   ├── update_component_output.rs
│   └── navigate_component.rs
└── queries/
    ├── mod.rs
    ├── get_cycle.rs
    ├── get_cycle_tree.rs
    └── get_component.rs

backend/src/adapters/
├── mod.rs
├── http/cycle/
│   ├── mod.rs
│   ├── handlers.rs
│   ├── handlers_test.rs
│   ├── dto.rs
│   └── routes.rs
└── postgres/
    ├── mod.rs
    ├── cycle_repository.rs
    ├── cycle_repository_test.rs
    ├── cycle_reader.rs
    ├── component_mapper.rs      # Maps JSONB to component types
    └── sqlx/queries/cycles.sql

frontend/src/modules/cycle/
├── domain/
│   ├── cycle.ts
│   ├── cycle.test.ts
│   ├── progress.ts
│   └── cycle-tree.ts
├── api/
│   ├── cycle-api.ts
│   ├── use-cycle.ts
│   └── use-cycle-tree.ts
├── components/
│   ├── CycleTree.svelte
│   ├── CycleTree.test.ts
│   ├── CycleProgress.svelte
│   ├── ComponentNav.svelte
│   ├── ComponentNav.test.ts
│   └── BranchDialog.svelte
└── index.ts
```

---

## Module: conversation (Full Module)

### Purpose
Manages the AI agent behavior, conversation flow, and message handling. Implements the "thoughtful decision professional" persona across all components.

### Domain Layer

#### Entities

```rust
/// Conversation tracks messages for a specific component
pub struct Conversation {
    id: ConversationId,
    component_id: ComponentId,     // Links to component in cycle
    component_type: ComponentType,
    messages: Vec<Message>,        // Uses Message from proact-types
    agent_state: AgentState,
    created_at: Timestamp,
    updated_at: Timestamp,
}

/// AgentState tracks conversation progress
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentState {
    pub current_phase: String,       // e.g., "listening", "categorizing", "confirming"
    pub pending_questions: Vec<String>,
    pub awaiting_confirmation: bool,
    pub extracted_items: i32,        // For tracking extraction progress
}

/// AgentConfig defines per-component behavior
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub component_type: ComponentType,
    pub system_prompt: String,
    pub phases: Vec<AgentPhase>,
    pub extraction_rules: Vec<ExtractionRule>,
}

#[derive(Debug, Clone)]
pub struct AgentPhase {
    pub name: String,
    pub objective: String,
    pub questions: Vec<String>,
}
```

### Ports

```rust
use async_trait::async_trait;
use tokio::sync::mpsc;

/// AI Provider port (infrastructure boundary)
#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, DomainError>;
    async fn stream(&self, req: CompletionRequest) -> Result<mpsc::Receiver<CompletionChunk>, DomainError>;
}

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub system_prompt: String,
    pub messages: Vec<Message>,
    pub max_tokens: i32,
    pub temperature: f64,
}

#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub content: String,
    pub tokens_used: i32,
    pub finish_reason: String,
}

#[derive(Debug, Clone)]
pub struct CompletionChunk {
    pub content: String,
    pub done: bool,
    pub error: Option<String>,
}

/// Conversation persistence (write port)
#[async_trait]
pub trait ConversationRepository: Send + Sync {
    async fn save(&self, conv: &Conversation) -> Result<(), DomainError>;
    async fn find_by_component(&self, component_id: ComponentId) -> Result<Option<Conversation>, DomainError>;
    async fn append_message(&self, component_id: ComponentId, msg: Message) -> Result<(), DomainError>;
}

/// Conversation queries (read port)
#[async_trait]
pub trait ConversationReader: Send + Sync {
    async fn get_by_component(&self, component_id: ComponentId) -> Result<Option<ConversationView>, DomainError>;
    async fn get_message_count(&self, component_id: ComponentId) -> Result<i32, DomainError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationView {
    pub id: ConversationId,
    pub component_id: ComponentId,
    pub component_type: ComponentType,
    pub messages: Vec<Message>,
    pub agent_state: AgentState,
    pub message_count: i32,
}
```

### Application Layer

#### Commands

| Command | Description |
|---------|-------------|
| `SendMessage` | User sends message to agent |
| `RegenerateResponse` | Re-generate last assistant message |
| `ConfirmExtraction` | User confirms extracted data |

```rust
#[derive(Debug, Clone)]
pub struct SendMessageCommand {
    pub component_id: ComponentId,
    pub content: String,
}

pub struct SendMessageHandler {
    conv_repo: Arc<dyn ConversationRepository>,
    cycle_repo: Arc<dyn CycleRepository>,
    ai_provider: Arc<dyn AIProvider>,
    configs: HashMap<ComponentType, AgentConfig>,
}

impl SendMessageHandler {
    pub async fn handle(&self, cmd: SendMessageCommand) -> Result<Message, DomainError> {
        // 1. Load or create conversation
        let mut conv = match self.conv_repo.find_by_component(cmd.component_id).await? {
            Some(c) => c,
            None => Conversation::new(cmd.component_id),
        };

        // 2. Add user message
        let _user_msg = conv.add_user_message(&cmd.content);

        // 3. Get component context
        let cycle = self.find_cycle_by_component(cmd.component_id).await?;
        let comp = cycle.get_component(conv.component_type())?;

        // 4. Build AI prompt
        let config = self.configs.get(&conv.component_type())
            .ok_or_else(|| DomainError::new(ErrorCode::NotFound, "Agent config not found"))?;
        let prompt = self.build_prompt(config, &conv, comp);

        // 5. Call AI provider
        let response = self.ai_provider.complete(prompt).await
            .map_err(|e| DomainError::new(ErrorCode::AIProviderError, &e.to_string()))?;

        // 6. Add assistant message
        let assistant_msg = conv.add_assistant_message(&response.content);

        // 7. Extract structured data and update component
        if let Some(extracted) = self.extract_structured_data(&response.content, config) {
            let mut cycle = cycle;
            cycle.update_component_output(conv.component_type(), extracted)?;
            self.cycle_repo.update(&cycle).await?;
        }

        // 8. Persist conversation
        self.conv_repo.save(&conv).await?;

        Ok(assistant_msg)
    }
}
```

### Adapters

#### HTTP Endpoints

| Method | Path | Handler |
|--------|------|---------|
| `POST` | `/api/components/:componentId/messages` | SendMessage |
| `GET` | `/api/components/:componentId/conversation` | GetConversation |
| `POST` | `/api/components/:componentId/regenerate` | RegenerateResponse |
| `WS` | `/api/components/:componentId/stream` | StreamConversation |

#### AI Provider Adapters

```rust
/// OpenAI implementation
pub struct OpenAIAdapter {
    client: OpenAIClient,
    model: String,
}

#[async_trait]
impl AIProvider for OpenAIAdapter {
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, DomainError> {
        // Implementation details
    }
    async fn stream(&self, req: CompletionRequest) -> Result<mpsc::Receiver<CompletionChunk>, DomainError> {
        // Streaming implementation
    }
}

/// Anthropic implementation
pub struct AnthropicAdapter {
    client: AnthropicClient,
    model: String,
}

#[async_trait]
impl AIProvider for AnthropicAdapter {
    // Similar implementation
}

/// Mock for testing
pub struct MockAIAdapter {
    responses: HashMap<ComponentType, Vec<String>>,
    call_index: Mutex<HashMap<ComponentType, usize>>,
}
```

#### Database Schema

```sql
CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id UUID NOT NULL UNIQUE,  -- One conversation per component
    component_type VARCHAR(50) NOT NULL,
    agent_state JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE conversation_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL,  -- 'user', 'assistant', 'system'
    content TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Maintain order
    sequence_num SERIAL
);

CREATE INDEX idx_conversations_component ON conversations(component_id);
CREATE INDEX idx_messages_conversation ON conversation_messages(conversation_id);
CREATE INDEX idx_messages_sequence ON conversation_messages(conversation_id, sequence_num);
```

### File Structure

```
backend/src/domain/conversation/
├── mod.rs
├── conversation.rs
├── conversation_test.rs
├── agent_state.rs
├── agent_config.rs
├── agent_configs/        # Per-component configurations
│   ├── mod.rs
│   ├── issue_raising.rs
│   ├── problem_frame.rs
│   └── ...
└── errors.rs

backend/src/ports/
├── mod.rs
├── ai_provider.rs
├── conversation_repository.rs
└── conversation_reader.rs

backend/src/application/
├── mod.rs
├── commands/
│   ├── mod.rs
│   ├── send_message.rs
│   ├── send_message_test.rs
│   └── regenerate_response.rs
└── queries/
    ├── mod.rs
    └── get_conversation.rs

backend/src/adapters/
├── mod.rs
├── ai/
│   ├── mod.rs
│   ├── openai_adapter.rs
│   ├── openai_adapter_test.rs
│   ├── anthropic_adapter.rs
│   └── mock_adapter.rs
├── http/conversation/
│   ├── mod.rs
│   ├── handlers.rs
│   ├── websocket_handler.rs
│   ├── dto.rs
│   └── routes.rs
└── postgres/
    ├── mod.rs
    └── conversation_repository.rs

frontend/src/modules/conversation/
├── domain/
│   ├── conversation.ts
│   └── agent-state.ts
├── api/
│   ├── conversation-api.ts
│   ├── use-conversation.ts
│   └── use-streaming.ts
├── components/
│   ├── ChatInterface.svelte
│   ├── ChatInterface.test.ts
│   ├── MessageBubble.svelte
│   ├── TypingIndicator.svelte
│   └── InputArea.svelte
└── index.ts
```

---

## Module: analysis (Domain Services)

### Purpose
Stateless domain services for analytical computations: Pugh matrix calculations, Decision Quality scoring, and value-of-information assessments.

**Note**: This module contains pure domain logic with no persistence needs. Services are called by other modules.

### Domain Layer

#### Services

```rust
/// PughAnalyzer computes Pugh matrix results (pure functions)
pub struct PughAnalyzer;

impl PughAnalyzer {
    /// Compute total scores for each alternative
    pub fn compute_scores(table: &ConsequencesTable) -> HashMap<String, i32> {
        let mut scores = HashMap::new();
        for alt_id in &table.alternative_ids {
            let total: i32 = table.objective_ids.iter()
                .filter_map(|obj_id| {
                    table.cells.get(alt_id)
                        .and_then(|row| row.get(obj_id))
                        .map(|cell| cell.rating.value())
                })
                .sum();
            scores.insert(alt_id.clone(), total);
        }
        scores
    }

    /// Find dominated alternatives
    /// Alternative A dominates B if A >= B on all objectives and A > B on at least one
    pub fn find_dominated(table: &ConsequencesTable) -> Vec<DominatedAlternative> {
        let mut dominated = Vec::new();
        // ... implementation
        dominated
    }

    /// Find objectives where all alternatives have the same rating
    pub fn find_irrelevant_objectives(table: &ConsequencesTable) -> Vec<String> {
        let mut irrelevant = Vec::new();
        // ... implementation
        irrelevant
    }
}

/// DQCalculator computes Decision Quality scores (pure functions)
pub struct DQCalculator;

impl DQCalculator {
    /// Overall DQ score is the minimum of all element scores
    pub fn compute_overall_score(elements: &[DQElement]) -> Percentage {
        elements.iter()
            .map(|e| e.score)
            .min()
            .unwrap_or(Percentage::ZERO)
    }

    /// Identify the weakest DQ element
    pub fn identify_weakest(elements: &[DQElement]) -> Option<&DQElement> {
        elements.iter()
            .min_by_key(|e| e.score)
    }
}

/// TradeoffAnalyzer identifies tensions between alternatives (pure functions)
pub struct TradeoffAnalyzer;

impl TradeoffAnalyzer {
    /// Analyze tensions for non-dominated alternatives
    pub fn analyze_tensions(
        table: &ConsequencesTable,
        dominated: &[DominatedAlternative],
    ) -> Vec<Tension> {
        let mut tensions = Vec::new();
        // ... implementation
        tensions
    }
}
```

#### Value Objects

```rust
/// CellColor for visual display of Pugh matrix cells
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellColor {
    DarkBlue,  // +2, Best
    Blue,      // +1
    Neutral,   // 0
    Orange,    // -1
    Red,       // -2, Worst
}

impl CellColor {
    /// Convert a Rating to its display color
    pub fn from_rating(rating: Rating) -> Self {
        match rating.value() {
            2 => CellColor::DarkBlue,
            1 => CellColor::Blue,
            0 => CellColor::Neutral,
            -1 => CellColor::Orange,
            -2 => CellColor::Red,
            _ => CellColor::Neutral,
        }
    }

    /// Get CSS class name for styling
    pub fn to_css_class(&self) -> &'static str {
        match self {
            CellColor::DarkBlue => "cell-dark-blue",
            CellColor::Blue => "cell-blue",
            CellColor::Neutral => "cell-neutral",
            CellColor::Orange => "cell-orange",
            CellColor::Red => "cell-red",
        }
    }
}
```

### File Structure

```
backend/src/domain/analysis/
├── mod.rs
├── pugh_analyzer.rs
├── pugh_analyzer_test.rs
├── dq_calculator.rs
├── dq_calculator_test.rs
├── tradeoff_analyzer.rs
├── tradeoff_analyzer_test.rs
├── cell_color.rs
└── cell_color_test.rs

frontend/src/modules/analysis/
├── domain/
│   ├── pugh-matrix.ts
│   ├── pugh-matrix.test.ts
│   ├── dq-score.ts
│   ├── dq-score.test.ts
│   └── cell-color.ts
├── components/
│   ├── ConsequencesTable.svelte
│   ├── ConsequencesTable.test.ts
│   ├── ConsequencesCell.svelte
│   ├── DQGauge.svelte
│   ├── DQGauge.test.ts
│   ├── DQElementList.svelte
│   └── TradeoffsChart.svelte
└── index.ts
```

**Note**: No ports, application layer, or adapters needed - these are pure functions used by other modules.

---

## Module: dashboard (Full Module)

### Purpose
Read models and view compositions for the dashboard interface. Provides Overview and Detail views by aggregating data from other modules.

### Domain Layer

#### View Models

```rust
/// DashboardOverview - the main dashboard view model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardOverview {
    pub session_id: SessionId,
    pub session_title: String,
    pub decision_statement: String,              // From ProblemFrame
    pub objectives_summary: Vec<ObjectiveSummary>,
    pub alternatives_list: Vec<AlternativeSummary>,
    pub consequences_table: Option<CompactConsequencesTable>,
    pub recommendation: Option<RecommendationSummary>,
    pub dq_score: Option<Percentage>,
    pub active_cycle_id: CycleId,
    pub cycle_count: i32,
    pub last_updated: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveSummary {
    pub id: String,
    pub description: String,
    pub is_fundamental: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeSummary {
    pub id: String,
    pub name: String,
    pub is_status_quo: bool,
    pub pugh_score: Option<i32>,  // Computed if consequences exist
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactConsequencesTable {
    pub alternative_names: Vec<String>,
    pub objective_names: Vec<String>,
    pub cells: Vec<Vec<CellSummary>>,  // [obj][alt]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellSummary {
    pub rating: Rating,
    pub color: CellColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationSummary {
    pub has_standout: bool,
    pub standout_name: Option<String>,
    pub synthesis_short: String,  // First 200 chars
}

/// ComponentDetailView - for drilling into a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDetailView {
    pub component_type: ComponentType,
    pub status: ComponentStatus,
    pub structured_output: serde_json::Value,  // Full type-specific output
    pub conversation_count: i32,
    pub last_message_at: Option<Timestamp>,
    pub can_branch: bool,
    pub can_revise: bool,
}
```

### Ports

```rust
/// Dashboard read port - aggregates data from multiple modules
#[async_trait]
pub trait DashboardReader: Send + Sync {
    async fn get_overview(
        &self,
        session_id: SessionId,
        cycle_id: Option<CycleId>,
    ) -> Result<DashboardOverview, DomainError>;

    async fn get_component_detail(
        &self,
        cycle_id: CycleId,
        comp_type: ComponentType,
    ) -> Result<ComponentDetailView, DomainError>;

    async fn compare_cycles(
        &self,
        cycle_ids: &[CycleId],
    ) -> Result<CycleComparison, DomainError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleComparison {
    pub cycles: Vec<CycleComparisonItem>,
    pub differences: Vec<ComparisonDifference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleComparisonItem {
    pub cycle_id: CycleId,
    pub branch_point: Option<ComponentType>,
    pub progress: CycleProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonDifference {
    pub component_type: ComponentType,
    pub cycle_id: CycleId,
    pub description: String,
}
```

### Application Layer

#### Queries Only (Read-Only Module)

| Query | Returns |
|-------|---------|
| `GetDashboardOverview` | Full dashboard view |
| `GetComponentDetail` | Single component with conversation |
| `CompareCycles` | Side-by-side cycle comparison |

### Adapters

#### HTTP Endpoints

| Method | Path | Handler |
|--------|------|---------|
| `GET` | `/api/sessions/:id/dashboard` | GetDashboardOverview |
| `GET` | `/api/sessions/:id/dashboard?cycleId=:cid` | GetDashboardOverview (specific cycle) |
| `GET` | `/api/cycles/:id/components/:type/detail` | GetComponentDetail |
| `GET` | `/api/sessions/:id/compare?cycles=:id1,:id2` | CompareCycles |

### File Structure

```
backend/src/domain/dashboard/
├── mod.rs
├── overview.rs
├── component_detail.rs
└── cycle_comparison.rs

backend/src/ports/
├── mod.rs
└── dashboard_reader.rs

backend/src/application/queries/
├── mod.rs
├── get_dashboard_overview.rs
├── get_dashboard_overview_test.rs
├── get_component_detail.rs
└── compare_cycles.rs

backend/src/adapters/
├── http/dashboard/
│   ├── mod.rs
│   ├── handlers.rs
│   ├── handlers_test.rs
│   └── routes.rs
└── postgres/
    ├── mod.rs
    └── dashboard_reader.rs

frontend/src/modules/dashboard/
├── domain/
│   ├── overview.ts
│   ├── component-detail.ts
│   └── cycle-comparison.ts
├── api/
│   ├── dashboard-api.ts
│   ├── use-dashboard.ts
│   └── use-component-detail.ts
├── components/
│   ├── DashboardLayout.svelte
│   ├── DashboardLayout.test.ts
│   ├── OverviewPanel.svelte
│   ├── DecisionStatement.svelte
│   ├── ObjectivesList.svelte
│   ├── AlternativesPills.svelte
│   ├── ConsequencesMatrix.svelte
│   ├── RecommendationCard.svelte
│   ├── DQScoreBadge.svelte
│   ├── CycleTreeSidebar.svelte
│   └── ComponentDetailDrawer.svelte
├── pages/
│   ├── +page.svelte
│   └── page.test.ts
└── index.ts
```

---

## Cross-Cutting Concerns

### Error Handling Strategy

```rust
use thiserror::Error;
use std::collections::HashMap;

/// DomainError is the base for all domain errors
#[derive(Debug, Error)]
#[error("{code}: {message}")]
pub struct DomainError {
    pub code: ErrorCode,
    pub message: String,
    pub details: HashMap<String, serde_json::Value>,
}

impl DomainError {
    pub fn new(code: ErrorCode, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
            details: HashMap::new(),
        }
    }

    pub fn with_detail(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self {
        self.details.insert(key.to_string(), value.into());
        self
    }
}

/// Error codes organized by module
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Session errors
    SessionNotFound,
    SessionArchived,
    SessionUnauthorized,

    // Cycle errors
    CycleNotFound,
    InvalidNavigation,
    BranchConflict,
    CycleArchived,

    // Component errors
    ComponentLocked,
    InvalidComponentTransition,
    ComponentNotStarted,

    // Conversation errors
    AIProviderError,
    RateLimited,
    ConversationNotFound,

    // General errors
    NotFound,
    ValidationFailed,
    InternalError,
}

/// HTTP error mapping
pub fn map_domain_error_to_http(err: &DomainError) -> (axum::http::StatusCode, ErrorResponse) {
    use axum::http::StatusCode;

    let status = match err.code {
        ErrorCode::SessionNotFound | ErrorCode::CycleNotFound |
        ErrorCode::ConversationNotFound | ErrorCode::NotFound => StatusCode::NOT_FOUND,

        ErrorCode::SessionUnauthorized => StatusCode::FORBIDDEN,

        ErrorCode::InvalidNavigation | ErrorCode::InvalidComponentTransition |
        ErrorCode::BranchConflict => StatusCode::CONFLICT,

        ErrorCode::RateLimited => StatusCode::TOO_MANY_REQUESTS,

        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    let response = ErrorResponse {
        code: err.code,
        message: err.message.clone(),
    };

    (status, response)
}
```

### Scaling Readiness

The system is designed as a **modular monolith** with clear seams for future horizontal scaling. While initially deployed as a single instance, the architecture prepares for multi-instance deployment through specific patterns.

For complete details, see [SCALING-READINESS.md](./SCALING-READINESS.md).

#### Key Infrastructure Components

| Component | Port | Purpose |
|-----------|------|---------|
| **OutboxWriter** | `outbox_writer.rs` | Transactional event persistence |
| **ConnectionRegistry** | `connection_registry.rs` | WebSocket connection tracking across servers |
| **CircuitBreaker** | `circuit_breaker.rs` | External service resilience |

#### Scaling Patterns

| Pattern | Implementation | Benefit |
|---------|---------------|---------|
| **Transactional Outbox** | Events written to DB in same transaction as domain changes | Guaranteed delivery, no lost events |
| **Component-Level Versioning** | Each component has independent version counter | Reduced concurrency conflicts |
| **Connection Registry** | Redis-backed WebSocket connection tracking | Cross-server message delivery |
| **Idempotency Keys** | Client-provided keys with server-side deduplication | Safe retries |

#### Database Preparation

All domain tables include `partition_key` column for future horizontal sharding:

```sql
-- Sessions partitioned by user_id
ALTER TABLE sessions ADD COLUMN partition_key VARCHAR(255);
UPDATE sessions SET partition_key = user_id;

-- Events partitioned by aggregate owner
ALTER TABLE event_outbox ADD COLUMN partition_key VARCHAR(255);
```

#### CQRS Pool Separation

```rust
pub struct DatabasePools {
    pub writer: PgPool,  // Primary database (writes)
    pub reader: PgPool,  // Read replicas (queries)
}
```

Repositories use `writer` pool; Readers use `reader` pool.

---

### Testing Strategy

| Layer | Test Type | Tools | Coverage Target |
|-------|-----------|-------|-----------------|
| Domain | Unit | Rust #[test], proptest | 90%+ |
| Application | Unit (mocked ports) | Rust #[test], mockall | 85%+ |
| Adapters | Integration | testcontainers-rs | 80%+ |
| HTTP | API | axum-test | 75%+ |
| Frontend | Unit | Vitest, Svelte Testing Library | 80%+ |
| E2E | User journeys | Playwright | Critical paths |

---

## Frontend Architecture

### Module Structure Pattern

Every full frontend module follows the SvelteKit route-based structure:

```
frontend/src/routes/<module>/
├── +page.svelte          # Route page component
├── +page.ts              # Load function (data fetching)
├── +page.server.ts       # Server-side load (if needed)
├── [id]/                 # Dynamic routes
│   ├── +page.svelte
│   └── +page.ts
└── components/           # Module-specific components
    ├── <Component>.svelte
    └── <Component>.test.ts

frontend/src/lib/<module>/
├── types.ts              # TypeScript types mirroring backend
├── api.ts                # API client functions
└── stores.ts             # Svelte stores (if needed)
```

### State Management

```typescript
// SvelteKit data loading with +page.ts
import type { PageLoad } from './$types';
import { sessionApi } from '$lib/session/api';

export const load: PageLoad = async ({ params, fetch }) => {
  const session = await sessionApi.getById(params.id, fetch);
  return { session };
};

// Svelte stores for reactive state
import { writable, derived } from 'svelte/store';
import type { Session, Cycle } from '$lib/types';

export const currentSession = writable<Session | null>(null);
export const currentCycle = writable<Cycle | null>(null);

// Derived store example
export const sessionTitle = derived(currentSession, ($session) =>
  $session?.title ?? 'Untitled Decision'
);

// API client with SvelteKit fetch
export async function createSession(data: CreateSessionRequest, fetch: typeof globalThis.fetch) {
  const response = await fetch('/api/sessions', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to create session');
  return response.json();
}

// Global app context via Svelte context
interface AppContext {
  currentUser: User | null;
  currentSessionId: string | null;
  currentCycleId: string | null;
}
```

---

## Deployment Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Load Balancer                         │
│                    (nginx / cloud LB)                        │
└──────────────────────────┬──────────────────────────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
┌───────▼───────┐  ┌───────▼───────┐  ┌───────▼───────┐
│   Frontend    │  │   API Server  │  │  WebSocket    │
│   (Static)    │  │   (REST)      │  │   Server      │
│   CDN/S3      │  │  Rust binary  │  │  Rust binary  │
└───────────────┘  └───────┬───────┘  └───────┬───────┘
                           │                  │
                    ┌──────┴──────────────────┴──────┐
                    │                               │
              ┌─────▼─────┐                  ┌──────▼──────┐
              │PostgreSQL │                  │   Redis     │
              │ (Primary) │                  │ (Sessions,  │
              │           │                  │  PubSub)    │
              └───────────┘                  └─────────────┘
                                                   │
                                            ┌──────▼──────┐
                                            │ AI Provider │
                                            │ (OpenAI/    │
                                            │  Anthropic) │
                                            └─────────────┘
```

---

## File Structure Summary

```
backend/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── foundation/      # Shared value objects
│   │   ├── proact/          # Shared component types
│   │   ├── session/         # Session aggregate
│   │   ├── membership/      # Membership aggregate (subscriptions, access)
│   │   │   └── value_objects/  # Money (cents!), Tier, Status, etc.
│   │   ├── cycle/           # Cycle aggregate (owns components)
│   │   ├── conversation/    # Conversation entity
│   │   ├── analysis/        # Domain services
│   │   └── dashboard/       # View models
│   ├── ports/
│   │   ├── mod.rs
│   │   ├── session_repository.rs
│   │   ├── session_reader.rs
│   │   ├── membership_repository.rs
│   │   ├── membership_reader.rs
│   │   ├── access_checker.rs      # Cross-module access control
│   │   ├── payment_provider.rs    # External payment (Stripe)
│   │   ├── cycle_repository.rs
│   │   ├── cycle_reader.rs
│   │   ├── conversation_repository.rs
│   │   ├── conversation_reader.rs
│   │   ├── ai_provider.rs
│   │   ├── dashboard_reader.rs
│   │   └── domain_event_publisher.rs
│   ├── application/
│   │   ├── mod.rs
│   │   ├── commands/
│   │   └── queries/
│   └── adapters/
│       ├── mod.rs
│       ├── http/
│       │   ├── mod.rs
│       │   ├── session/
│       │   ├── membership/
│       │   ├── cycle/
│       │   ├── conversation/
│       │   └── dashboard/
│       ├── postgres/
│       ├── redis/
│       ├── ai/
│       └── stripe/          # Stripe payment adapter
├── migrations/
├── config/
└── Cargo.toml

frontend/
├── src/
│   ├── lib/
│   │   ├── domain/          # foundation types
│   │   ├── proact/          # component types
│   │   ├── components/      # Shared UI
│   │   └── utils/
│   ├── routes/
│   │   ├── session/
│   │   ├── membership/
│   │   ├── pricing/         # Pricing/plans page
│   │   ├── account/         # User account/subscription
│   │   ├── cycle/
│   │   ├── conversation/
│   │   ├── analysis/
│   │   └── dashboard/
│   ├── app.html
│   └── app.d.ts
├── static/
├── package.json
├── svelte.config.js
├── vite.config.ts
└── tsconfig.json
```

---

## Symmetry Checklist

### Vertical Symmetry (Each Full Module Has)

| Layer | session | membership | cycle | conversation | dashboard |
|-------|---------|------------|-------|--------------|-----------|
| Domain | ✅ | ✅ | ✅ | ✅ | ✅ |
| Ports (Repository) | ✅ | ✅ | ✅ | ✅ | N/A |
| Ports (Reader) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Ports (External) | N/A | ✅ (Stripe) | N/A | ✅ (AI) | N/A |
| Application/Commands | ✅ | ✅ | ✅ | ✅ | N/A |
| Application/Queries | ✅ | ✅ | ✅ | ✅ | ✅ |
| HTTP Adapters | ✅ | ✅ | ✅ | ✅ | ✅ |
| Postgres Adapters | ✅ | ✅ | ✅ | ✅ | ✅ |
| External Adapters | N/A | ✅ (Stripe) | N/A | ✅ (OpenAI/Anthropic) | N/A |
| Frontend domain/ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Frontend api/ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Frontend components/ | ✅ | ✅ | ✅ | ✅ | ✅ |

### Horizontal Symmetry (Consistent Patterns)

| Pattern | Applied Consistently |
|---------|---------------------|
| Repository interface | ✅ Save, Update, FindByID |
| Reader interface | ✅ GetByID, List* |
| Command handlers | ✅ Handle(ctx, cmd) (result, error) |
| Query handlers | ✅ Handle(ctx, query) (view, error) |
| HTTP handlers | ✅ ServeHTTP or method handlers |
| Domain events | ✅ PullDomainEvents() pattern |
| Error handling | ✅ DomainError with codes |

---

*Architecture Version: 1.3.0*
*Based on Functional Spec: functional-spec-20260107.md*
*Created: 2026-01-07*
*Updated: 2026-01-08 (Scaling Readiness)*

---

## Changelog

### v1.3.0 (2026-01-08)
- Added Scaling Readiness section with reference to SCALING-READINESS.md
- Documented new ports: OutboxWriter, ConnectionRegistry, CircuitBreaker
- Added scaling patterns overview (Transactional Outbox, Component-Level Versioning)
- Documented database preparation for future sharding (partition_key columns)
- Added CQRS pool separation pattern (writer/reader pools)

### v1.2.0 (2026-01-07)
- Added `membership` module for subscriptions, access control, and payments
- Added `AccessChecker` port for cross-module access gating
- Added `PaymentProvider` port for Stripe integration
- Updated session module to depend on membership (access check)
- Updated build order: membership now Phase 2, session now Phase 3
- Added Stripe adapter to adapters layer
- Added pricing and account pages to frontend

### v1.1.0 (2026-01-07)
- Initial symmetry review
- Established module classification (Shared Domain, Full Module, Domain Services)
