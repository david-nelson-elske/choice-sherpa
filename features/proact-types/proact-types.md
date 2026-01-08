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
