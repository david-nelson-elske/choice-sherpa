# PrOACT Types Module Specification

## Overview

The PrOACT Types module defines the 9 PrOACT component types and their structured outputs. These are domain types used by the `cycle` module for persistence and by the `conversation` module for data extraction.

**Note**: This is a shared domain library, not a full module. Components are owned and persisted by the `cycle` module.

---

## Module Classification

| Attribute | Value |
|-----------|-------|
| **Type** | Shared Domain (types only, no ports/adapters) |
| **Language** | Rust |
| **Responsibility** | PrOACT component interface and 9 concrete types |
| **Domain Dependencies** | foundation |
| **External Dependencies** | `serde`, `chrono` |

---

## Architecture

### Shared Domain Pattern

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          PROACT-TYPES MODULE                                 │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                       COMPONENT TRAIT                                   │ │
│  │                                                                         │ │
│  │   trait Component {                                                     │ │
│  │       fn id(&self) -> ComponentId                                       │ │
│  │       fn component_type(&self) -> ComponentType                         │ │
│  │       fn status(&self) -> ComponentStatus                               │ │
│  │       fn start(&mut self) -> Result<()>                                 │ │
│  │       fn complete(&mut self) -> Result<()>                              │ │
│  │   }                                                                     │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
│       ┌──────────────────────────────┼──────────────────────────────┐       │
│       │              │               │               │              │       │
│  ┌────▼────┐   ┌─────▼─────┐   ┌─────▼─────┐   ┌─────▼─────┐  ┌─────▼─────┐ │
│  │ Issue   │   │ Problem   │   │Objectives │   │Alternatives│  │Consequences│ │
│  │ Raising │   │  Frame    │   │           │   │            │  │           │ │
│  └─────────┘   └───────────┘   └───────────┘   └────────────┘  └───────────┘ │
│       │              │               │               │              │       │
│  ┌────▼────┐   ┌─────▼─────┐   ┌─────▼─────┐   ┌─────▼─────┐               │
│  │Tradeoffs│   │Recomm-    │   │ Decision  │   │Notes &    │               │
│  │         │   │endation   │   │  Quality  │   │Next Steps │               │
│  └─────────┘   └───────────┘   └───────────┘   └───────────┘               │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         MESSAGE TYPE                                    │ │
│  │   Message { id, role, content, metadata, timestamp }                    │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Domain Layer

### Component Trait

```rust
use crate::foundation::{ComponentId, ComponentType, ComponentStatus, Timestamp};

/// Trait that all PrOACT components implement
pub trait Component: Send + Sync {
    /// Returns the unique identifier
    fn id(&self) -> ComponentId;

    /// Returns the component type
    fn component_type(&self) -> ComponentType;

    /// Returns the current status
    fn status(&self) -> ComponentStatus;

    /// Returns when this component was created
    fn created_at(&self) -> Timestamp;

    /// Returns when this component was last updated
    fn updated_at(&self) -> Timestamp;

    /// Starts work on this component
    fn start(&mut self) -> Result<(), ComponentError>;

    /// Completes this component
    fn complete(&mut self) -> Result<(), ComponentError>;

    /// Marks this component for revision
    fn mark_for_revision(&mut self, reason: String) -> Result<(), ComponentError>;

    /// Returns the structured output as a type-erased value
    fn output_as_value(&self) -> serde_json::Value;

    /// Sets the structured output from a type-erased value
    fn set_output_from_value(&mut self, value: serde_json::Value) -> Result<(), ComponentError>;
}

/// Base fields shared by all components
#[derive(Debug, Clone)]
pub struct ComponentBase {
    pub id: ComponentId,
    pub component_type: ComponentType,
    pub status: ComponentStatus,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub revision_reason: Option<String>,
}

impl ComponentBase {
    pub fn new(component_type: ComponentType) -> Self {
        let now = Timestamp::now();
        Self {
            id: ComponentId::new(),
            component_type,
            status: ComponentStatus::NotStarted,
            created_at: now,
            updated_at: now,
            revision_reason: None,
        }
    }

    pub fn start(&mut self) -> Result<(), ComponentError> {
        if !self.status.can_transition_to(&ComponentStatus::InProgress) {
            return Err(ComponentError::InvalidTransition {
                from: self.status,
                to: ComponentStatus::InProgress,
            });
        }
        self.status = ComponentStatus::InProgress;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    pub fn complete(&mut self) -> Result<(), ComponentError> {
        if !self.status.can_transition_to(&ComponentStatus::Complete) {
            return Err(ComponentError::InvalidTransition {
                from: self.status,
                to: ComponentStatus::Complete,
            });
        }
        self.status = ComponentStatus::Complete;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    pub fn mark_for_revision(&mut self, reason: String) -> Result<(), ComponentError> {
        if !self.status.can_transition_to(&ComponentStatus::NeedsRevision) {
            return Err(ComponentError::InvalidTransition {
                from: self.status,
                to: ComponentStatus::NeedsRevision,
            });
        }
        self.status = ComponentStatus::NeedsRevision;
        self.revision_reason = Some(reason);
        self.updated_at = Timestamp::now();
        Ok(())
    }
}
```

### Message Type

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::foundation::{Timestamp, ValidationError};

/// SECURITY: Maximum message content length (100KB) to prevent DoS via oversized payloads
pub const MAX_MESSAGE_LENGTH: usize = 100_000;

/// Unique identifier for a message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageId(Uuid);

impl MessageId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

/// Role of the message sender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
    System,
}

/// A single message in a component conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub role: Role,
    pub content: String,
    pub metadata: MessageMetadata,
    pub timestamp: Timestamp,
}

impl Message {
    /// Creates a user message with content validation
    /// SECURITY: Validates content length to prevent DoS via oversized payloads
    pub fn user(content: impl Into<String>) -> Result<Self, ValidationError> {
        let content = content.into();
        if content.len() > MAX_MESSAGE_LENGTH {
            return Err(ValidationError::OutOfRange {
                field: "content".into(),
                min: 0,
                max: MAX_MESSAGE_LENGTH as i64,
                actual: content.len() as i64,
            });
        }
        Ok(Self {
            id: MessageId::new(),
            role: Role::User,
            content,
            metadata: MessageMetadata::default(),
            timestamp: Timestamp::now(),
        })
    }

    /// Creates an assistant message (from AI - size already controlled by token limits)
    /// Note: AI responses have their own token limits, so we don't validate size here
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            id: MessageId::new(),
            role: Role::Assistant,
            content: content.into(),
            metadata: MessageMetadata::default(),
            timestamp: Timestamp::now(),
        }
    }

    /// Creates a system message (internal use - size controlled by application)
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            id: MessageId::new(),
            role: Role::System,
            content: content.into(),
            metadata: MessageMetadata::default(),
            timestamp: Timestamp::now(),
        }
    }
}

/// Metadata associated with a message
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Approximate token count
    #[serde(default)]
    pub token_count: Option<u32>,

    /// Data extracted from this message
    #[serde(default)]
    pub extracted_data: serde_json::Value,
}
```

---

## Component Types (9 Total)

### 1. IssueRaising

```rust
use serde::{Deserialize, Serialize};

/// Categorized outputs from user's initial brain dump
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IssueRaisingOutput {
    /// Things that need to be chosen/decided
    pub potential_decisions: Vec<String>,

    /// Things that matter to the user
    pub objectives: Vec<String>,

    /// Things that are unknown
    pub uncertainties: Vec<String>,

    /// Process constraints, facts, stakeholders
    pub considerations: Vec<String>,

    /// Whether user has validated the categorization
    pub user_confirmed: bool,
}

#[derive(Debug, Clone)]
pub struct IssueRaising {
    base: ComponentBase,
    output: IssueRaisingOutput,
}

impl IssueRaising {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(ComponentType::IssueRaising),
            output: IssueRaisingOutput::default(),
        }
    }

    pub fn output(&self) -> &IssueRaisingOutput {
        &self.output
    }

    pub fn set_output(&mut self, output: IssueRaisingOutput) {
        self.output = output;
        self.base.updated_at = Timestamp::now();
    }

    pub fn add_potential_decision(&mut self, decision: String) {
        self.output.potential_decisions.push(decision);
        self.base.updated_at = Timestamp::now();
    }

    pub fn confirm(&mut self) {
        self.output.user_confirmed = true;
        self.base.updated_at = Timestamp::now();
    }
}

impl Component for IssueRaising {
    fn id(&self) -> ComponentId { self.base.id }
    fn component_type(&self) -> ComponentType { self.base.component_type }
    fn status(&self) -> ComponentStatus { self.base.status }
    fn created_at(&self) -> Timestamp { self.base.created_at }
    fn updated_at(&self) -> Timestamp { self.base.updated_at }
    fn start(&mut self) -> Result<(), ComponentError> { self.base.start() }
    fn complete(&mut self) -> Result<(), ComponentError> { self.base.complete() }
    fn mark_for_revision(&mut self, reason: String) -> Result<(), ComponentError> {
        self.base.mark_for_revision(reason)
    }
    fn output_as_value(&self) -> serde_json::Value {
        serde_json::to_value(&self.output).unwrap_or_default()
    }
    fn set_output_from_value(&mut self, value: serde_json::Value) -> Result<(), ComponentError> {
        self.output = serde_json::from_value(value)
            .map_err(|e| ComponentError::InvalidOutput(e.to_string()))?;
        Ok(())
    }
}
```

### 2. ProblemFrame

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Structured problem framing output
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProblemFrameOutput {
    /// Who has authority to make this decision
    pub decision_maker: Option<String>,

    /// What specifically is being decided
    pub focal_decision: Option<String>,

    /// What success looks like
    pub ultimate_aim: Option<String>,

    /// When must this be decided by
    pub temporal_constraint: Option<DateTime<Utc>>,

    /// Geographic or organizational scope
    pub spatial_scope: Option<String>,

    /// Future choices that depend on this decision
    pub linked_decisions: Vec<LinkedDecision>,

    /// Legal, financial, political, or technical constraints
    pub constraints: Vec<Constraint>,

    /// Stakeholders affected by this decision
    pub affected_parties: Vec<Party>,

    /// Experts who should be consulted
    pub expert_sources: Vec<String>,

    /// Hierarchical organization of decisions
    pub decision_hierarchy: Option<DecisionHierarchy>,

    /// Synthesized decision statement
    pub decision_statement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedDecision {
    pub description: String,
    /// Relationship type: "enables", "constrains", "depends_on"
    pub relationship: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    /// Type: "legal", "financial", "political", "technical"
    pub constraint_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Party {
    pub id: String,
    pub name: String,
    pub role: String,
    /// What this party cares about
    pub objectives: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionHierarchy {
    /// Decisions already made
    pub already_made: Vec<String>,
    /// The focal decisions being analyzed
    pub focal_decisions: Vec<String>,
    /// Decisions to be deferred
    pub deferred: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProblemFrame {
    base: ComponentBase,
    output: ProblemFrameOutput,
}

impl ProblemFrame {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(ComponentType::ProblemFrame),
            output: ProblemFrameOutput::default(),
        }
    }

    pub fn output(&self) -> &ProblemFrameOutput {
        &self.output
    }

    pub fn set_output(&mut self, output: ProblemFrameOutput) {
        self.output = output;
        self.base.updated_at = Timestamp::now();
    }

    pub fn set_decision_statement(&mut self, statement: String) {
        self.output.decision_statement = Some(statement);
        self.base.updated_at = Timestamp::now();
    }

    pub fn add_party(&mut self, party: Party) {
        self.output.affected_parties.push(party);
        self.base.updated_at = Timestamp::now();
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.output.constraints.push(constraint);
        self.base.updated_at = Timestamp::now();
    }
}
```

### 3. Objectives

```rust
use serde::{Deserialize, Serialize};

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
    /// Links to Party.id from ProblemFrame
    pub affected_party_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeansObjective {
    pub id: String,
    pub description: String,
    /// Which fundamental objective this supports
    pub contributes_to_objective_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMeasure {
    pub description: String,
    pub is_quantitative: bool,
    /// Unit of measurement (e.g., "dollars", "days")
    pub unit: Option<String>,
    /// Direction: "higher_is_better" or "lower_is_better"
    pub direction: String,
}

#[derive(Debug, Clone)]
pub struct Objectives {
    base: ComponentBase,
    output: ObjectivesOutput,
}

impl Objectives {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(ComponentType::Objectives),
            output: ObjectivesOutput::default(),
        }
    }

    pub fn output(&self) -> &ObjectivesOutput {
        &self.output
    }

    pub fn add_fundamental(&mut self, objective: FundamentalObjective) {
        self.output.fundamental_objectives.push(objective);
        self.base.updated_at = Timestamp::now();
    }

    pub fn add_means(&mut self, objective: MeansObjective) {
        self.output.means_objectives.push(objective);
        self.base.updated_at = Timestamp::now();
    }

    pub fn fundamental_count(&self) -> usize {
        self.output.fundamental_objectives.len()
    }
}
```

### 4. Alternatives

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlternativesOutput {
    pub options: Vec<Alternative>,
    /// Strategy table for multiple focal decisions
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
    /// Maps decision_name -> chosen option
    pub choices: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Alternatives {
    base: ComponentBase,
    output: AlternativesOutput,
}

impl Alternatives {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(ComponentType::Alternatives),
            output: AlternativesOutput::default(),
        }
    }

    pub fn output(&self) -> &AlternativesOutput {
        &self.output
    }

    pub fn add_alternative(&mut self, alt: Alternative) {
        if alt.is_status_quo {
            self.output.has_status_quo = true;
        }
        self.output.options.push(alt);
        self.base.updated_at = Timestamp::now();
    }

    pub fn set_strategy_table(&mut self, table: StrategyTable) {
        self.output.strategy_table = Some(table);
        self.base.updated_at = Timestamp::now();
    }

    pub fn alternatives_count(&self) -> usize {
        self.output.options.len()
    }
}
```

### 5. Consequences

```rust
use serde::{Deserialize, Serialize};
use crate::foundation::Rating;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsequencesOutput {
    pub table: ConsequencesTable,
    pub uncertainties: Vec<Uncertainty>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsequencesTable {
    /// Column identifiers (alternative IDs)
    pub alternative_ids: Vec<String>,
    /// Row identifiers (objective IDs)
    pub objective_ids: Vec<String>,
    /// Cell data: cells[alt_id][obj_id]
    pub cells: std::collections::HashMap<String, std::collections::HashMap<String, Cell>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    /// Pugh rating: -2 to +2
    pub rating: Rating,
    /// Explanation of the rating
    pub explanation: String,
    /// Quantitative value if available
    pub quant_value: Option<f64>,
    /// Unit for quantitative value
    pub quant_unit: Option<String>,
    /// Source/citation
    pub source: Option<String>,
    /// Flag for uncertain values
    pub uncertainty: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Uncertainty {
    pub id: String,
    pub description: String,
    /// What causes this uncertainty
    pub driver: String,
    /// Is it worth spending resources to resolve?
    pub worth_resolving: bool,
    /// Can it be reduced within the decision timeframe?
    pub resolvable: bool,
}

#[derive(Debug, Clone)]
pub struct Consequences {
    base: ComponentBase,
    output: ConsequencesOutput,
}

impl Consequences {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(ComponentType::Consequences),
            output: ConsequencesOutput::default(),
        }
    }

    pub fn output(&self) -> &ConsequencesOutput {
        &self.output
    }

    pub fn set_cell(&mut self, alt_id: &str, obj_id: &str, cell: Cell) {
        self.output.table.cells
            .entry(alt_id.to_string())
            .or_default()
            .insert(obj_id.to_string(), cell);
        self.base.updated_at = Timestamp::now();
    }

    pub fn get_cell(&self, alt_id: &str, obj_id: &str) -> Option<&Cell> {
        self.output.table.cells
            .get(alt_id)
            .and_then(|row| row.get(obj_id))
    }

    pub fn add_uncertainty(&mut self, uncertainty: Uncertainty) {
        self.output.uncertainties.push(uncertainty);
        self.base.updated_at = Timestamp::now();
    }
}
```

### 6. Tradeoffs

```rust
use serde::{Deserialize, Serialize};

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
    /// Reason why this objective doesn't distinguish alternatives
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tension {
    pub alternative_id: String,
    /// Objectives where this alternative excels
    pub gains: Vec<String>,
    /// Objectives where this alternative suffers
    pub losses: Vec<String>,
    /// How uncertainty affects this tension
    pub uncertainty_impact: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Tradeoffs {
    base: ComponentBase,
    output: TradeoffsOutput,
}

impl Tradeoffs {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(ComponentType::Tradeoffs),
            output: TradeoffsOutput::default(),
        }
    }

    pub fn output(&self) -> &TradeoffsOutput {
        &self.output
    }

    pub fn add_dominated(&mut self, dominated: DominatedAlternative) {
        self.output.dominated_alternatives.push(dominated);
        self.base.updated_at = Timestamp::now();
    }

    pub fn add_irrelevant(&mut self, irrelevant: IrrelevantObjective) {
        self.output.irrelevant_objectives.push(irrelevant);
        self.base.updated_at = Timestamp::now();
    }

    pub fn add_tension(&mut self, tension: Tension) {
        self.output.tensions.push(tension);
        self.base.updated_at = Timestamp::now();
    }

    pub fn viable_alternative_count(&self, total_alternatives: usize) -> usize {
        total_alternatives - self.output.dominated_alternatives.len()
    }
}
```

### 7. Recommendation

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecommendationOutput {
    /// AlternativeID if one stands out
    pub standout_option: Option<String>,
    /// Summary of the analysis
    pub synthesis: String,
    /// Important qualifications
    pub caveats: Vec<String>,
    /// What additional information might help
    pub additional_info: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Recommendation {
    base: ComponentBase,
    output: RecommendationOutput,
}

impl Recommendation {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(ComponentType::Recommendation),
            output: RecommendationOutput::default(),
        }
    }

    pub fn output(&self) -> &RecommendationOutput {
        &self.output
    }

    pub fn set_synthesis(&mut self, synthesis: String) {
        self.output.synthesis = synthesis;
        self.base.updated_at = Timestamp::now();
    }

    pub fn set_standout(&mut self, alternative_id: String) {
        self.output.standout_option = Some(alternative_id);
        self.base.updated_at = Timestamp::now();
    }

    pub fn add_caveat(&mut self, caveat: String) {
        self.output.caveats.push(caveat);
        self.base.updated_at = Timestamp::now();
    }

    pub fn has_standout(&self) -> bool {
        self.output.standout_option.is_some()
    }
}
```

### 8. DecisionQuality

```rust
use serde::{Deserialize, Serialize};
use crate::foundation::Percentage;

/// The 7 standard Decision Quality elements
pub const DQ_ELEMENT_NAMES: &[&str] = &[
    "Helpful Problem Frame",
    "Clear Objectives",
    "Creative Alternatives",
    "Reliable Consequence Information",
    "Logically Correct Reasoning",
    "Clear Tradeoffs",
    "Commitment to Follow Through",
];

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionQualityOutput {
    pub elements: Vec<DQElement>,
    /// Minimum of all element scores
    pub overall_score: Percentage,
    /// What would raise the lowest scores
    pub improvement_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DQElement {
    /// One of the 7 standard element names
    pub name: String,
    /// Score from 0-100
    pub score: Percentage,
    /// Why this score was given
    pub rationale: String,
    /// What would improve this element
    pub improvement: String,
}

#[derive(Debug, Clone)]
pub struct DecisionQuality {
    base: ComponentBase,
    output: DecisionQualityOutput,
}

impl DecisionQuality {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(ComponentType::DecisionQuality),
            output: DecisionQualityOutput::default(),
        }
    }

    pub fn output(&self) -> &DecisionQualityOutput {
        &self.output
    }

    pub fn set_element(&mut self, element: DQElement) {
        // Replace if exists, otherwise add
        if let Some(existing) = self.output.elements
            .iter_mut()
            .find(|e| e.name == element.name)
        {
            *existing = element;
        } else {
            self.output.elements.push(element);
        }
        self.recalculate_overall();
        self.base.updated_at = Timestamp::now();
    }

    pub fn recalculate_overall(&mut self) {
        if self.output.elements.is_empty() {
            self.output.overall_score = Percentage::ZERO;
        } else {
            let min = self.output.elements
                .iter()
                .map(|e| e.score.value())
                .min()
                .unwrap_or(0);
            self.output.overall_score = Percentage::new(min);
        }
    }

    pub fn is_perfect(&self) -> bool {
        self.output.overall_score.value() == 100
    }

    pub fn weakest_element(&self) -> Option<&DQElement> {
        self.output.elements
            .iter()
            .min_by_key(|e| e.score.value())
    }
}
```

### 9. NotesNextSteps

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotesNextStepsOutput {
    pub remaining_uncertainties: Vec<String>,
    pub open_questions: Vec<String>,
    pub planned_actions: Vec<PlannedAction>,
    /// Affirmation if DQ is 100%
    pub affirmation: Option<String>,
    /// Further analysis paths if DQ < 100%
    pub further_analysis_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedAction {
    pub description: String,
    pub due_date: Option<DateTime<Utc>>,
    pub owner: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NotesNextSteps {
    base: ComponentBase,
    output: NotesNextStepsOutput,
}

impl NotesNextSteps {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(ComponentType::NotesNextSteps),
            output: NotesNextStepsOutput::default(),
        }
    }

    pub fn output(&self) -> &NotesNextStepsOutput {
        &self.output
    }

    pub fn add_action(&mut self, action: PlannedAction) {
        self.output.planned_actions.push(action);
        self.base.updated_at = Timestamp::now();
    }

    pub fn add_open_question(&mut self, question: String) {
        self.output.open_questions.push(question);
        self.base.updated_at = Timestamp::now();
    }

    pub fn set_affirmation(&mut self, affirmation: String) {
        self.output.affirmation = Some(affirmation);
        self.base.updated_at = Timestamp::now();
    }

    pub fn action_count(&self) -> usize {
        self.output.planned_actions.len()
    }
}
```

---

## Component Enum

For pattern matching and aggregate handling:

```rust
/// Sum type for all component types
#[derive(Debug, Clone)]
pub enum ComponentVariant {
    IssueRaising(IssueRaising),
    ProblemFrame(ProblemFrame),
    Objectives(Objectives),
    Alternatives(Alternatives),
    Consequences(Consequences),
    Tradeoffs(Tradeoffs),
    Recommendation(Recommendation),
    DecisionQuality(DecisionQuality),
    NotesNextSteps(NotesNextSteps),
}

impl ComponentVariant {
    /// Creates a new component of the specified type
    pub fn new(component_type: ComponentType) -> Self {
        match component_type {
            ComponentType::IssueRaising => ComponentVariant::IssueRaising(IssueRaising::new()),
            ComponentType::ProblemFrame => ComponentVariant::ProblemFrame(ProblemFrame::new()),
            ComponentType::Objectives => ComponentVariant::Objectives(Objectives::new()),
            ComponentType::Alternatives => ComponentVariant::Alternatives(Alternatives::new()),
            ComponentType::Consequences => ComponentVariant::Consequences(Consequences::new()),
            ComponentType::Tradeoffs => ComponentVariant::Tradeoffs(Tradeoffs::new()),
            ComponentType::Recommendation => ComponentVariant::Recommendation(Recommendation::new()),
            ComponentType::DecisionQuality => ComponentVariant::DecisionQuality(DecisionQuality::new()),
            ComponentType::NotesNextSteps => ComponentVariant::NotesNextSteps(NotesNextSteps::new()),
        }
    }

    /// Returns the component type
    pub fn component_type(&self) -> ComponentType {
        match self {
            ComponentVariant::IssueRaising(_) => ComponentType::IssueRaising,
            ComponentVariant::ProblemFrame(_) => ComponentType::ProblemFrame,
            ComponentVariant::Objectives(_) => ComponentType::Objectives,
            ComponentVariant::Alternatives(_) => ComponentType::Alternatives,
            ComponentVariant::Consequences(_) => ComponentType::Consequences,
            ComponentVariant::Tradeoffs(_) => ComponentType::Tradeoffs,
            ComponentVariant::Recommendation(_) => ComponentType::Recommendation,
            ComponentVariant::DecisionQuality(_) => ComponentType::DecisionQuality,
            ComponentVariant::NotesNextSteps(_) => ComponentType::NotesNextSteps,
        }
    }

    /// Returns the component status
    pub fn status(&self) -> ComponentStatus {
        match self {
            ComponentVariant::IssueRaising(c) => c.status(),
            ComponentVariant::ProblemFrame(c) => c.status(),
            ComponentVariant::Objectives(c) => c.status(),
            ComponentVariant::Alternatives(c) => c.status(),
            ComponentVariant::Consequences(c) => c.status(),
            ComponentVariant::Tradeoffs(c) => c.status(),
            ComponentVariant::Recommendation(c) => c.status(),
            ComponentVariant::DecisionQuality(c) => c.status(),
            ComponentVariant::NotesNextSteps(c) => c.status(),
        }
    }
}
```

---

## Error Types

```rust
use thiserror::Error;
use crate::foundation::ComponentStatus;

#[derive(Debug, Clone, Error)]
pub enum ComponentError {
    #[error("Invalid state transition from {from} to {to}")]
    InvalidTransition {
        from: ComponentStatus,
        to: ComponentStatus,
    },

    #[error("Invalid output data: {0}")]
    InvalidOutput(String),

    #[error("Component not started")]
    NotStarted,

    #[error("Component already complete")]
    AlreadyComplete,
}
```

---

## File Structure

```
backend/src/domain/proact/
├── mod.rs                      # Module exports
├── component.rs                # Component trait + ComponentBase
├── component_test.rs           # Component trait tests
├── component_variant.rs        # ComponentVariant enum
├── message.rs                  # Message, MessageId, Role
├── message_test.rs             # Message tests
├── issue_raising.rs            # IssueRaising component
├── issue_raising_test.rs
├── problem_frame.rs            # ProblemFrame component
├── problem_frame_test.rs
├── objectives.rs               # Objectives component
├── objectives_test.rs
├── alternatives.rs             # Alternatives component
├── alternatives_test.rs
├── consequences.rs             # Consequences component
├── consequences_test.rs
├── tradeoffs.rs                # Tradeoffs component
├── tradeoffs_test.rs
├── recommendation.rs           # Recommendation component
├── recommendation_test.rs
├── decision_quality.rs         # DecisionQuality component
├── decision_quality_test.rs
├── notes_next_steps.rs         # NotesNextSteps component
├── notes_next_steps_test.rs
└── errors.rs                   # ComponentError

frontend/src/shared/proact/
├── component.ts                # Component interface
├── message.ts                  # Message type
├── issue-raising.ts            # IssueRaising types
├── problem-frame.ts            # ProblemFrame types
├── objectives.ts               # Objectives types
├── alternatives.ts             # Alternatives types
├── consequences.ts             # Consequences types
├── tradeoffs.ts                # Tradeoffs types
├── recommendation.ts           # Recommendation types
├── decision-quality.ts         # DecisionQuality types
├── notes-next-steps.ts         # NotesNextSteps types
└── index.ts                    # Public exports
```

---

## Invariants

| Invariant | Enforcement |
|-----------|-------------|
| Each component has exactly one type | Rust enum + ComponentBase |
| Status transitions are valid | `can_transition_to()` check in base |
| Only 9 component types exist | Compile-time via ComponentType enum |
| DQ overall score is minimum of elements | `recalculate_overall()` method |
| Completed components have output | Checked by complete() in subtypes |
| Messages are ordered by timestamp | Append-only in conversation |

---

## Test Categories

### Unit Tests

| Category | Example Tests |
|----------|---------------|
| Component lifecycle | `issue_raising_starts_as_not_started` |
| Status transitions | `cannot_complete_before_start` |
| Output mutation | `add_potential_decision_updates_timestamp` |
| DQ calculation | `overall_score_is_minimum` |
| Component variant | `variant_returns_correct_type` |
| Message creation | `user_message_has_user_role` |
| Serialization | `output_roundtrips_through_json` |

### Property-Based Tests

| Property | Description |
|----------|-------------|
| Status machine | Valid transitions always succeed |
| DQ minimum | Overall never exceeds any element |
| Component order | All types have unique order indices |

---

## Integration Points

### Consumed By

| Module | Usage |
|--------|-------|
| cycle | Owns ComponentVariant in aggregate |
| conversation | Extracts structured data to outputs |
| analysis | Reads Consequences table for Pugh analysis |
| dashboard | Reads all outputs for views |

### Provides To Foundation

This module depends on foundation but doesn't provide to it (unidirectional).

---

*Module Version: 1.0.0*
*Based on: SYSTEM-ARCHITECTURE.md v1.1.0*
*Language: Rust*
