# Feature: PrOACT Types Module

> Shared domain library defining the 9 PrOACT component types, their structured outputs, and the Component trait. These types are used by the cycle module for persistence and by the conversation module for data extraction.

## Context

- This is a **Shared Domain** module (types only, no ports/adapters)
- Components are owned and persisted by the `cycle` module
- Depends on `foundation` module (ComponentId, ComponentType, ComponentStatus, Timestamp, Rating, Percentage)
- External dependencies: `serde`, `chrono`, `uuid`, `thiserror`
- All component types implement the Component trait
- ComponentVariant enum provides pattern matching for all 9 types

## Tasks

- [x] Create proact module structure with mod.rs
- [x] Implement ComponentError enum with thiserror
- [x] Implement MessageId value object with UUID
- [x] Implement Role enum (User, Assistant, System)
- [x] Implement MessageMetadata struct
- [x] Implement Message struct with factory methods
- [x] Implement ComponentBase struct with lifecycle methods
- [x] Implement Component trait definition
- [x] Implement IssueRaisingOutput struct
- [x] Implement IssueRaising component with Component trait
- [x] Implement ProblemFrameOutput and supporting types (LinkedDecision, Constraint, Party, DecisionHierarchy)
- [x] Implement ProblemFrame component with Component trait
- [x] Implement ObjectivesOutput and supporting types (FundamentalObjective, MeansObjective, PerformanceMeasure)
- [x] Implement Objectives component with Component trait
- [x] Implement AlternativesOutput and supporting types (Alternative, StrategyTable, DecisionColumn, Strategy)
- [x] Implement Alternatives component with Component trait
- [x] Implement ConsequencesOutput and supporting types (ConsequencesTable, Cell, Uncertainty)
- [x] Implement Consequences component with Component trait
- [x] Implement TradeoffsOutput and supporting types (DominatedAlternative, IrrelevantObjective, Tension)
- [x] Implement Tradeoffs component with Component trait
- [x] Implement RecommendationOutput struct
- [x] Implement Recommendation component with Component trait
- [x] Implement DecisionQualityOutput and DQElement types with DQ_ELEMENT_NAMES constant
- [x] Implement DecisionQuality component with Component trait and score calculation
- [x] Implement NotesNextStepsOutput and PlannedAction types
- [x] Implement NotesNextSteps component with Component trait
- [x] Implement ComponentVariant enum with factory and accessor methods

## Acceptance Criteria

- [x] All 9 component types implement the Component trait
- [x] ComponentBase handles status transitions with proper validation
- [x] Status transitions follow state machine: NotStarted -> InProgress -> Complete (with NeedsRevision from InProgress/Complete)
- [x] All output types are serializable/deserializable via serde
- [x] DecisionQuality.overall_score equals minimum of element scores
- [x] ComponentVariant::new() creates correct component for each ComponentType
- [x] All structs derive Debug and Clone
- [x] Message factory methods (user, assistant, system) set correct roles
- [x] Unit tests cover all status transitions and output mutations
- [x] All code compiles with no warnings

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Not Required (shared types, no endpoints) |
| Authorization Model | N/A - types used by authenticated modules |
| Sensitive Data | Message.content (Confidential), Component outputs (Confidential) |
| Rate Limiting | Not Required (no endpoints) |
| Audit Logging | N/A - types only, logging handled by consuming modules |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| Message.content | Confidential | User decision data - never log, encrypt at rest |
| Message.id | Internal | Opaque identifier |
| ComponentBase.output | Confidential | Structured user decision data - never log |
| All component output types | Confidential | Contains user's decision analysis |
| MessageMetadata | Internal | May contain extraction hints, do not expose |

### Validation Requirements

1. **Message Length Validation**: Implement `MAX_MESSAGE_LENGTH` to prevent DoS via oversized messages:

```rust
pub const MAX_MESSAGE_LENGTH: usize = 50_000; // 50KB max per message

impl Message {
    pub fn new(role: Role, content: impl Into<String>) -> Result<Self, ValidationError> {
        let content = content.into();
        if content.len() > MAX_MESSAGE_LENGTH {
            return Err(ValidationError::field_error(
                "content",
                format!("Message exceeds maximum length of {} bytes", MAX_MESSAGE_LENGTH),
            ));
        }
        // ... rest of construction
    }
}
```

2. **Output Size Limits**: Component outputs should have size limits to prevent storage DoS:

```rust
pub const MAX_OUTPUT_SIZE: usize = 500_000; // 500KB max per component output
```

### Security Guidelines

1. **Logging Prohibition**: Message content and component outputs MUST NOT be logged:

```rust
// CORRECT: Log message metadata only
tracing::debug!(
    message_id = %message.id,
    role = ?message.role,
    "Processing message"
);

// INCORRECT: Never log content
tracing::debug!("Message: {:?}", message); // DO NOT DO THIS if Debug includes content
```

2. **Custom Debug Implementation**: Consider implementing custom `Debug` for `Message` that redacts content:

```rust
impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Message")
            .field("id", &self.id)
            .field("role", &self.role)
            .field("content", &format!("[{} bytes]", self.content.len()))
            .field("created_at", &self.created_at)
            .finish()
    }
}
```

3. **Serialization**: When serializing for API transport, ensure Message content is only sent to authorized users (the session owner)
