# Feature: Analysis Domain Events

**Module:** analysis
**Type:** Event Publishing + Event Handling
**Priority:** P1
**Phase:** 5 of Full PrOACT Journey Integration
**Depends On:** features/cycle/cycle-events.md

> Analysis module publishes computed scores and subscribes to component completion events to trigger automatic calculations.

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Required |
| Authorization Model | User must own the parent session; analysis results tied to cycle ownership |
| Sensitive Data | Analysis results derived from user decisions (Confidential) |
| Rate Limiting | Not Required - event-driven, not user-initiated |
| Audit Logging | Analysis computation events, score changes |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| Alternative scores | Confidential | Derived from user decisions, encrypt at rest |
| DQ element scores | Confidential | User self-assessment data, encrypt at rest |
| Dominated alternatives list | Confidential | Analysis of user's options |
| Tension summaries | Confidential | Contains alternative names from user input |
| cycle_id, session_id | Internal | Safe to log |
| Computed timestamps | Internal | Safe to log |
| Score counts (dominated_count, item_count) | Internal | Safe to aggregate for analytics |

### Security Events to Log

- `analysis.pugh_scores_computed` - Log cycle_id, alternative count, best_alternative_id presence (no scores)
- `analysis.dq_scores_computed` - Log cycle_id, overall_score, weakest_element name
- `analysis.tradeoffs_analyzed` - Log cycle_id, dominated_count, tension_count
- Analysis computation failures - Log cycle_id, error type (no raw data)

### Input Validation (at API Boundary)

Analysis functions are pure computation, but inputs must be validated at the API/event handler boundary:
1. Validate consequences table structure before passing to PughAnalyzer
2. Validate DQ element scores are within 0-100 range
3. Validate alternative and objective IDs are valid UUIDs
4. Reject malformed event payloads before processing

---

## Problem Statement

The analysis module provides pure computation functions but has no integration with the event system:
- Pugh matrix scores aren't calculated automatically when Consequences completes
- DQ scores aren't calculated automatically when DecisionQuality completes
- Dashboard must manually request score calculations
- No visibility into when computations complete

### Current State

- Manual invocation of analysis functions
- No automatic triggers
- Scores stored without notification

### Desired State

- Analysis triggers automatically on relevant component completions
- Computed scores published as events
- Dashboard receives real-time score updates

---

## Domain Events

### PughScoresComputed

Published when Pugh matrix analysis completes for a cycle.

```rust
// backend/src/domain/analysis/events.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Published when Pugh matrix scores are computed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PughScoresComputed {
    pub event_id: EventId,
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    /// Map of alternative_id -> total score
    pub alternative_scores: HashMap<String, i32>,
    /// IDs of dominated alternatives
    pub dominated_alternatives: Vec<String>,
    /// IDs of irrelevant objectives (same score across all alternatives)
    pub irrelevant_objectives: Vec<String>,
    /// ID of the best-scoring alternative (if clear winner)
    pub best_alternative_id: Option<String>,
    pub computed_at: Timestamp,
}

impl DomainEvent for PughScoresComputed {
    fn event_type(&self) -> &'static str {
        "analysis.pugh_scores_computed"
    }

    fn aggregate_id(&self) -> String {
        self.cycle_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.computed_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

**Subscribers:**
- `DashboardUpdateHandler` - Update consequences table with scores
- `WebSocketEventBridge` - Push scores to connected clients

---

### DQScoresComputed

Published when Decision Quality assessment is computed.

```rust
/// Published when Decision Quality scores are computed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DQScoresComputed {
    pub event_id: EventId,
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    /// Scores for each DQ element
    pub element_scores: Vec<DQElementScore>,
    /// Overall DQ score (minimum of all elements)
    pub overall_score: Percentage,
    /// Element with lowest score (weakest link)
    pub weakest_element: String,
    /// Suggested improvement paths
    pub improvement_suggestions: Vec<String>,
    pub computed_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DQElementScore {
    pub element_name: String,
    pub score: Percentage,
    pub rationale: String,
}

impl DomainEvent for DQScoresComputed {
    fn event_type(&self) -> &'static str {
        "analysis.dq_scores_computed"
    }

    fn aggregate_id(&self) -> String {
        self.cycle_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.computed_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

**Subscribers:**
- `DashboardUpdateHandler` - Update DQ gauge and element list
- `WebSocketEventBridge` - Push scores to connected clients
- `CycleCompletionHandler` - Include DQ score in cycle completion

---

### TradeoffsAnalyzed

Published when tradeoff analysis completes.

```rust
/// Published when tradeoff analysis is completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeoffsAnalyzed {
    pub event_id: EventId,
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    /// Number of dominated alternatives found
    pub dominated_count: i32,
    /// Number of irrelevant objectives found
    pub irrelevant_count: i32,
    /// Tension summaries
    pub tension_summaries: Vec<TensionSummary>,
    pub analyzed_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensionSummary {
    pub alternative_id: String,
    pub alternative_name: String,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
}

impl DomainEvent for TradeoffsAnalyzed {
    fn event_type(&self) -> &'static str {
        "analysis.tradeoffs_analyzed"
    }

    fn aggregate_id(&self) -> String {
        self.cycle_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.analyzed_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

---

## Event Handlers

### AnalysisTriggerHandler

Subscribes to `ComponentCompleted` to trigger appropriate analyses.

```rust
// backend/src/application/handlers/analysis_trigger.rs

/// Triggers analysis computations when relevant components complete
pub struct AnalysisTriggerHandler {
    cycle_repo: Arc<dyn CycleRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl AnalysisTriggerHandler {
    pub fn new(
        cycle_repo: Arc<dyn CycleRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self { cycle_repo, event_publisher }
    }

    async fn compute_pugh_scores(&self, cycle: &Cycle) -> Result<PughScoresComputed, DomainError> {
        // Get consequences table from cycle
        let consequences = cycle.get_component(ComponentType::Consequences)?;
        let table: ConsequencesTable = consequences.output_as_value()
            .get("table")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or_else(|| DomainError::new(ErrorCode::ValidationFailed, "No consequences table"))?;

        // Compute scores using pure analysis functions
        let scores = PughAnalyzer::compute_scores(&table);
        let dominated = PughAnalyzer::find_dominated(&table);
        let irrelevant = PughAnalyzer::find_irrelevant_objectives(&table);

        // Find best alternative
        let best_alt = scores.iter()
            .max_by_key(|(_, score)| *score)
            .map(|(id, _)| id.clone());

        Ok(PughScoresComputed {
            event_id: EventId::new(),
            cycle_id: cycle.id(),
            session_id: cycle.session_id(),
            alternative_scores: scores,
            dominated_alternatives: dominated.iter().map(|d| d.alternative_id.clone()).collect(),
            irrelevant_objectives: irrelevant,
            best_alternative_id: best_alt,
            computed_at: Timestamp::now(),
        })
    }

    async fn compute_dq_scores(&self, cycle: &Cycle) -> Result<DQScoresComputed, DomainError> {
        // Get DQ component
        let dq_component = cycle.get_component(ComponentType::DecisionQuality)?;
        let elements: Vec<DQElement> = dq_component.output_as_value()
            .get("elements")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or_else(|| DomainError::new(ErrorCode::ValidationFailed, "No DQ elements"))?;

        // Compute overall score (minimum)
        let overall = DQCalculator::compute_overall_score(&elements);

        // Find weakest element
        let weakest = DQCalculator::identify_weakest(&elements)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        // Generate improvement suggestions
        let suggestions = elements.iter()
            .filter(|e| e.score.value() < 80)
            .map(|e| e.improvement.clone())
            .collect();

        Ok(DQScoresComputed {
            event_id: EventId::new(),
            cycle_id: cycle.id(),
            session_id: cycle.session_id(),
            element_scores: elements.iter().map(|e| DQElementScore {
                element_name: e.name.clone(),
                score: e.score,
                rationale: e.rationale.clone(),
            }).collect(),
            overall_score: overall,
            weakest_element: weakest,
            improvement_suggestions: suggestions,
            computed_at: Timestamp::now(),
        })
    }

    async fn analyze_tradeoffs(&self, cycle: &Cycle) -> Result<TradeoffsAnalyzed, DomainError> {
        // Get consequences and tradeoffs data
        let consequences = cycle.get_component(ComponentType::Consequences)?;
        let table: ConsequencesTable = consequences.output_as_value()
            .get("table")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or_else(|| DomainError::new(ErrorCode::ValidationFailed, "No consequences table"))?;

        // Get alternatives for names
        let alternatives = cycle.get_component(ComponentType::Alternatives)?;
        let alt_list: Vec<Alternative> = alternatives.output_as_value()
            .get("options")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        let alt_names: HashMap<String, String> = alt_list.iter()
            .map(|a| (a.id.clone(), a.name.clone()))
            .collect();

        // Compute analysis
        let dominated = PughAnalyzer::find_dominated(&table);
        let irrelevant = PughAnalyzer::find_irrelevant_objectives(&table);
        let tensions = TradeoffAnalyzer::analyze_tensions(&table, &dominated);

        let tension_summaries = tensions.iter().map(|t| TensionSummary {
            alternative_id: t.alternative_id.clone(),
            alternative_name: alt_names.get(&t.alternative_id).cloned().unwrap_or_default(),
            strengths: t.gains.clone(),
            weaknesses: t.losses.clone(),
        }).collect();

        Ok(TradeoffsAnalyzed {
            event_id: EventId::new(),
            cycle_id: cycle.id(),
            session_id: cycle.session_id(),
            dominated_count: dominated.len() as i32,
            irrelevant_count: irrelevant.len() as i32,
            tension_summaries,
            analyzed_at: Timestamp::now(),
        })
    }
}

#[async_trait]
impl EventHandler for AnalysisTriggerHandler {
    async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError> {
        // Parse component completed event
        let component_completed: ComponentCompleted = event.payload_as()
            .map_err(|e| DomainError::new(ErrorCode::ValidationFailed, &e.to_string()))?;

        // Load cycle
        let cycle = self.cycle_repo
            .find_by_id(component_completed.cycle_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::CycleNotFound, "Cycle not found"))?;

        // Determine which analysis to run based on component type
        match component_completed.component_type {
            ComponentType::Consequences => {
                // Compute Pugh scores
                let pugh_event = self.compute_pugh_scores(&cycle).await?;
                self.event_publisher.publish(
                    EventEnvelope::from_event(&pugh_event, "Analysis")
                        .with_causation_id(event.event_id.as_str())
                ).await?;
            }

            ComponentType::Tradeoffs => {
                // Analyze tradeoffs
                let tradeoffs_event = self.analyze_tradeoffs(&cycle).await?;
                self.event_publisher.publish(
                    EventEnvelope::from_event(&tradeoffs_event, "Analysis")
                        .with_causation_id(event.event_id.as_str())
                ).await?;
            }

            ComponentType::DecisionQuality => {
                // Compute DQ scores
                let dq_event = self.compute_dq_scores(&cycle).await?;
                self.event_publisher.publish(
                    EventEnvelope::from_event(&dq_event, "Analysis")
                        .with_causation_id(event.event_id.as_str())
                ).await?;
            }

            _ => {
                // No analysis for other components
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "AnalysisTriggerHandler"
    }
}
```

---

## Acceptance Criteria

### AC1: Pugh Scores on Consequences Completion

**Given** Consequences component is completed with valid table
**When** `ComponentCompleted` event is processed
**Then** `PughScoresComputed` event is published with:
- Alternative scores (sum of ratings)
- List of dominated alternatives
- List of irrelevant objectives
- Best alternative ID (if clear)

### AC2: DQ Scores on DecisionQuality Completion

**Given** DecisionQuality component is completed with element ratings
**When** `ComponentCompleted` event is processed
**Then** `DQScoresComputed` event is published with:
- Individual element scores
- Overall score (minimum)
- Weakest element identified
- Improvement suggestions for low scores

### AC3: Tradeoffs Analysis on Tradeoffs Completion

**Given** Tradeoffs component is completed
**When** `ComponentCompleted` event is processed
**Then** `TradeoffsAnalyzed` event is published with:
- Dominated alternative count
- Irrelevant objective count
- Tension summaries per alternative

### AC4: Idempotent Analysis

**Given** analysis has already been computed for a cycle
**When** duplicate `ComponentCompleted` event is processed
**Then** analysis runs again (pure function, same result) with new event ID

### AC5: Missing Data Handling

**Given** Consequences component has incomplete table
**When** Pugh analysis is triggered
**Then** Analysis fails gracefully, no event published, error logged

### AC6: Event Causation Chain

**Given** `ComponentCompleted` triggers analysis
**When** analysis event is published
**Then** `causation_id` links back to triggering `ComponentCompleted` event

---

## Technical Design

### File Structure

```
backend/src/domain/analysis/
├── mod.rs                    # Add events export
├── pugh_analyzer.rs          # Existing pure functions
├── dq_calculator.rs          # Existing pure functions
├── tradeoff_analyzer.rs      # Existing pure functions
├── events.rs                 # NEW: Analysis events
└── events_test.rs            # NEW: Event unit tests

backend/src/application/handlers/
├── mod.rs                    # Add analysis handler
├── analysis_trigger.rs       # NEW: AnalysisTriggerHandler
└── analysis_trigger_test.rs  # NEW
```

### Event Payload Considerations

Analysis events contain **computed results**, not raw data:

```rust
// Good: Summarized results
pub struct PughScoresComputed {
    pub alternative_scores: HashMap<String, i32>,  // Just the totals
    pub dominated_alternatives: Vec<String>,       // Just the IDs
    pub best_alternative_id: Option<String>,       // Just the winner
}

// Bad: Duplicating raw data
pub struct PughScoresComputed {
    pub consequences_table: ConsequencesTable,     // Don't include raw data
    pub full_analysis: PughAnalysisResult,         // Too much detail
}
```

---

## Test Specifications

### Unit Tests: Event Types

```rust
#[test]
fn pugh_scores_computed_event_type() {
    let event = PughScoresComputed {
        event_id: EventId::new(),
        cycle_id: CycleId::new(),
        session_id: SessionId::new(),
        alternative_scores: HashMap::from([
            ("alt-1".to_string(), 5),
            ("alt-2".to_string(), -2),
        ]),
        dominated_alternatives: vec!["alt-2".to_string()],
        irrelevant_objectives: vec![],
        best_alternative_id: Some("alt-1".to_string()),
        computed_at: Timestamp::now(),
    };

    assert_eq!(event.event_type(), "analysis.pugh_scores_computed");
}

#[test]
fn dq_scores_computed_has_overall_minimum() {
    let event = DQScoresComputed {
        event_id: EventId::new(),
        cycle_id: CycleId::new(),
        session_id: SessionId::new(),
        element_scores: vec![
            DQElementScore {
                element_name: "Clear Objectives".to_string(),
                score: Percentage::new(80).unwrap(),
                rationale: "Well defined".to_string(),
            },
            DQElementScore {
                element_name: "Creative Alternatives".to_string(),
                score: Percentage::new(60).unwrap(),
                rationale: "Limited options".to_string(),
            },
        ],
        overall_score: Percentage::new(60).unwrap(), // Minimum
        weakest_element: "Creative Alternatives".to_string(),
        improvement_suggestions: vec!["Generate more alternatives".to_string()],
        computed_at: Timestamp::now(),
    };

    assert_eq!(event.overall_score.value(), 60);
    assert_eq!(event.weakest_element, "Creative Alternatives");
}
```

### Unit Tests: Event Handler

```rust
#[tokio::test]
async fn analysis_trigger_computes_pugh_on_consequences_complete() {
    let cycle_repo = Arc::new(InMemoryCycleRepository::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    // Create cycle with completed consequences
    let mut cycle = create_cycle_with_consequences();
    cycle_repo.save(&cycle).await.unwrap();

    let handler = AnalysisTriggerHandler::new(cycle_repo, event_bus.clone());

    let event = create_component_completed_event(
        cycle.id(),
        ComponentType::Consequences,
    );

    // Act
    handler.handle(event).await.unwrap();

    // Assert
    let pugh_events = event_bus.events_of_type("analysis.pugh_scores_computed");
    assert_eq!(pugh_events.len(), 1);

    let payload: PughScoresComputed = pugh_events[0].payload_as().unwrap();
    assert!(!payload.alternative_scores.is_empty());
}

#[tokio::test]
async fn analysis_trigger_computes_dq_on_decision_quality_complete() {
    let cycle_repo = Arc::new(InMemoryCycleRepository::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    let mut cycle = create_cycle_with_dq();
    cycle_repo.save(&cycle).await.unwrap();

    let handler = AnalysisTriggerHandler::new(cycle_repo, event_bus.clone());

    let event = create_component_completed_event(
        cycle.id(),
        ComponentType::DecisionQuality,
    );

    // Act
    handler.handle(event).await.unwrap();

    // Assert
    let dq_events = event_bus.events_of_type("analysis.dq_scores_computed");
    assert_eq!(dq_events.len(), 1);

    let payload: DQScoresComputed = dq_events[0].payload_as().unwrap();
    assert!(!payload.element_scores.is_empty());
    assert!(payload.overall_score.value() <= 100);
}

#[tokio::test]
async fn analysis_trigger_ignores_irrelevant_components() {
    let cycle_repo = Arc::new(InMemoryCycleRepository::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    let cycle = Cycle::new(SessionId::new()).unwrap();
    cycle_repo.save(&cycle).await.unwrap();

    let handler = AnalysisTriggerHandler::new(cycle_repo, event_bus.clone());

    // Complete IssueRaising - should not trigger analysis
    let event = create_component_completed_event(
        cycle.id(),
        ComponentType::IssueRaising,
    );

    // Act
    handler.handle(event).await.unwrap();

    // Assert - no analysis events
    assert_eq!(event_bus.event_count(), 0);
}

#[tokio::test]
async fn analysis_trigger_handles_missing_data_gracefully() {
    let cycle_repo = Arc::new(InMemoryCycleRepository::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    // Cycle with empty consequences
    let mut cycle = Cycle::new(SessionId::new()).unwrap();
    cycle.start_component(ComponentType::Consequences).unwrap();
    // No output data set
    cycle_repo.save(&cycle).await.unwrap();

    let handler = AnalysisTriggerHandler::new(cycle_repo, event_bus.clone());

    let event = create_component_completed_event(
        cycle.id(),
        ComponentType::Consequences,
    );

    // Act
    let result = handler.handle(event).await;

    // Assert - fails gracefully
    assert!(result.is_err());
    assert_eq!(event_bus.event_count(), 0);
}

#[tokio::test]
async fn analysis_event_has_causation_link() {
    let cycle_repo = Arc::new(InMemoryCycleRepository::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    let mut cycle = create_cycle_with_consequences();
    cycle_repo.save(&cycle).await.unwrap();

    let handler = AnalysisTriggerHandler::new(cycle_repo, event_bus.clone());

    let triggering_event_id = EventId::from_string("trigger-123");
    let event = EventEnvelope {
        event_id: triggering_event_id.clone(),
        event_type: "component.completed".to_string(),
        // ... other fields
    };

    // Act
    handler.handle(event).await.unwrap();

    // Assert - causation link
    let pugh_events = event_bus.events_of_type("analysis.pugh_scores_computed");
    assert_eq!(
        pugh_events[0].metadata.causation_id,
        Some("trigger-123".to_string())
    );
}

// Test helpers
fn create_cycle_with_consequences() -> Cycle {
    let mut cycle = Cycle::new(SessionId::new()).unwrap();
    // Start and complete prerequisites
    for comp in &[ComponentType::IssueRaising, ComponentType::ProblemFrame,
                  ComponentType::Objectives, ComponentType::Alternatives] {
        cycle.start_component(*comp).unwrap();
        cycle.complete_component(*comp).unwrap();
    }
    cycle.start_component(ComponentType::Consequences).unwrap();
    cycle.update_component_output(
        ComponentType::Consequences,
        json!({
            "table": {
                "alternative_ids": ["alt-1", "alt-2"],
                "objective_ids": ["obj-1", "obj-2"],
                "cells": {
                    "alt-1": { "obj-1": { "rating": 2 }, "obj-2": { "rating": 1 } },
                    "alt-2": { "obj-1": { "rating": -1 }, "obj-2": { "rating": 0 } }
                }
            }
        }),
    ).unwrap();
    cycle
}
```

---

## Event Registration

```rust
// backend/src/main.rs or setup module

fn register_analysis_handlers(event_bus: &impl EventSubscriber, deps: &Dependencies) {
    // Trigger analysis on specific component completions
    event_bus.subscribe(
        "component.completed",
        AnalysisTriggerHandler::new(
            deps.cycle_repo.clone(),
            deps.event_publisher.clone(),
        ),
    );
}
```

---

## Dependencies

### Module Dependencies

- `foundation::events` - EventId, EventEnvelope, DomainEvent
- `foundation::ids` - CycleId, SessionId
- `foundation::percentage` - Percentage
- `proact-types::consequences` - ConsequencesTable
- `proact-types::decision_quality` - DQElement
- `cycle::events` - ComponentCompleted (subscribed)
- `ports::event_publisher` - EventPublisher trait

---

## Related Documents

- **Integration Spec:** features/integrations/full-proact-journey.md
- **Phase 3:** features/cycle/cycle-events.md
- **Checklist:** REQUIREMENTS/CHECKLIST-events.md (Phase 5)
- **Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md

---

*Version: 1.0.0*
*Created: 2026-01-07*
*Phase: 5 of 8*
