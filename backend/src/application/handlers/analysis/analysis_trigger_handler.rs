//! AnalysisTriggerHandler - Event handler for ComponentCompleted events.
//!
//! Listens for component completions and triggers the appropriate analysis:
//! - Consequences completion → Pugh matrix analysis → PughScoresComputed
//! - Tradeoffs completion → Tradeoff analysis → TradeoffsAnalyzed
//! - DecisionQuality completion → DQ calculation → DQScoresComputed
//!
//! This enables the dashboard to show computed analysis results in real-time.

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::domain::analysis::{
    ConsequencesTable, ConsequencesTableBuilder, DQCalculator, DQElement,
    DQElementScore, DQScoresComputed, PughAnalyzer, PughScoresComputed,
    TensionSummary, TradeoffAnalyzer, TradeoffsAnalyzed,
};
use crate::domain::foundation::{
    ComponentType, CycleId, DomainError, ErrorCode, EventEnvelope, EventId, Percentage, Rating,
    SerializableDomainEvent, SessionId, Timestamp,
};
use crate::ports::{CycleReader, EventHandler, EventPublisher};

/// External ComponentCompleted event from the Cycle module.
///
/// This is the expected payload format for `component.completed` events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentCompletedPayload {
    /// Unique event identifier.
    pub event_id: EventId,
    /// The cycle containing the component.
    pub cycle_id: CycleId,
    /// The component that was completed.
    pub component_type: ComponentType,
    /// When the component was completed.
    pub completed_at: Timestamp,
}

/// Handles ComponentCompleted events to trigger analysis computations.
///
/// When specific components complete, this handler:
/// 1. Fetches the component output from the cycle
/// 2. Runs the appropriate analysis function
/// 3. Publishes the computed analysis event
///
/// This maintains eventual consistency for analysis results.
pub struct AnalysisTriggerHandler {
    cycle_reader: Arc<dyn CycleReader>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl AnalysisTriggerHandler {
    /// Creates a new AnalysisTriggerHandler.
    pub fn new(
        cycle_reader: Arc<dyn CycleReader>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            cycle_reader,
            event_publisher,
        }
    }

    /// Handles Consequences component completion.
    ///
    /// Fetches consequences table data, runs Pugh analysis, and publishes results.
    async fn handle_consequences_completed(
        &self,
        cycle_id: CycleId,
        session_id: SessionId,
        causation_id: &str,
    ) -> Result<(), DomainError> {
        // Fetch consequences component output
        let output_view = self
            .cycle_reader
            .get_component_output(&cycle_id, ComponentType::Consequences)
            .await?
            .ok_or_else(|| {
                DomainError::new(
                    ErrorCode::ComponentNotFound,
                    "Consequences component output not found",
                )
            })?;

        // Parse consequences table from output
        let table = self.parse_consequences_table(&output_view.output)?;

        // Run Pugh analysis
        let scores = PughAnalyzer::compute_scores(&table);
        let dominated = PughAnalyzer::find_dominated(&table);
        let irrelevant = PughAnalyzer::find_irrelevant_objectives(&table);

        // Find best alternative (highest score)
        let best_alternative_id = scores
            .iter()
            .max_by_key(|(_, score)| *score)
            .map(|(id, _)| id.clone());

        // Build event
        let event = PughScoresComputed {
            event_id: EventId::new(),
            cycle_id,
            session_id,
            alternative_scores: scores,
            dominated_alternatives: dominated.iter().map(|d| d.alternative_id.clone()).collect(),
            irrelevant_objectives: irrelevant.iter().map(|i| i.objective_id.clone()).collect(),
            best_alternative_id,
            computed_at: Timestamp::now(),
        };

        let envelope = event.to_envelope().with_causation_id(causation_id);
        self.event_publisher.publish(envelope).await?;

        debug!(
            cycle_id = %cycle_id,
            "Published PughScoresComputed event"
        );

        Ok(())
    }

    /// Handles DecisionQuality component completion.
    ///
    /// Fetches DQ element scores, computes overall, and publishes results.
    async fn handle_dq_completed(
        &self,
        cycle_id: CycleId,
        session_id: SessionId,
        causation_id: &str,
    ) -> Result<(), DomainError> {
        // Fetch DQ component output
        let output_view = self
            .cycle_reader
            .get_component_output(&cycle_id, ComponentType::DecisionQuality)
            .await?
            .ok_or_else(|| {
                DomainError::new(
                    ErrorCode::ComponentNotFound,
                    "DecisionQuality component output not found",
                )
            })?;

        // Parse DQ elements from output
        let elements = self.parse_dq_elements(&output_view.output)?;

        // Compute overall score
        let overall_score = DQCalculator::compute_overall(&elements);

        // Find weakest element
        let weakest = DQCalculator::find_weakest(&elements)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        // Get sorted elements for improvement suggestions
        let sorted = DQCalculator::sorted_by_priority(&elements);
        let improvement_suggestions: Vec<String> = sorted
            .iter()
            .filter(|e| e.score.value() < 80)
            .filter_map(|e| {
                e.improvement_path.clone().or_else(|| {
                    Some(format!("Improve {} (currently at {}%)", e.name, e.score.value()))
                })
            })
            .collect();

        // Build element scores for event
        let element_scores: Vec<DQElementScore> = elements
            .iter()
            .map(|e| DQElementScore {
                element_name: e.name.clone(),
                score: e.score,
                rationale: e.rationale.clone().unwrap_or_default(),
            })
            .collect();

        // Build event
        let event = DQScoresComputed {
            event_id: EventId::new(),
            cycle_id,
            session_id,
            element_scores,
            overall_score,
            weakest_element: weakest,
            improvement_suggestions,
            computed_at: Timestamp::now(),
        };

        let envelope = event.to_envelope().with_causation_id(causation_id);
        self.event_publisher.publish(envelope).await?;

        debug!(
            cycle_id = %cycle_id,
            overall_score = %overall_score,
            "Published DQScoresComputed event"
        );

        Ok(())
    }

    /// Handles Tradeoffs component completion.
    ///
    /// Fetches analysis data, computes tensions, and publishes results.
    async fn handle_tradeoffs_completed(
        &self,
        cycle_id: CycleId,
        session_id: SessionId,
        causation_id: &str,
    ) -> Result<(), DomainError> {
        // Fetch consequences table for tradeoff analysis
        let consequences_output = self
            .cycle_reader
            .get_component_output(&cycle_id, ComponentType::Consequences)
            .await?
            .ok_or_else(|| {
                DomainError::new(
                    ErrorCode::ComponentNotFound,
                    "Consequences component output not found for tradeoff analysis",
                )
            })?;

        let table = self.parse_consequences_table(&consequences_output.output)?;

        // Get dominated alternatives and irrelevant objectives
        let dominated = PughAnalyzer::find_dominated(&table);
        let irrelevant = PughAnalyzer::find_irrelevant_objectives(&table);

        // Analyze tensions for non-dominated alternatives
        let tensions = TradeoffAnalyzer::analyze_tensions(&table, &dominated);

        // Build tension summaries for non-dominated alternatives
        let tension_summaries: Vec<TensionSummary> = tensions
            .iter()
            .map(|t| TensionSummary {
                alternative_id: t.alternative_id.clone(),
                alternative_name: t.alternative_id.clone(), // Could be enhanced with actual names
                strengths: t.gains.clone(),
                weaknesses: t.losses.clone(),
            })
            .collect();

        // Build event
        let event = TradeoffsAnalyzed {
            event_id: EventId::new(),
            cycle_id,
            session_id,
            dominated_count: dominated.len() as i32,
            irrelevant_count: irrelevant.len() as i32,
            tension_summaries,
            analyzed_at: Timestamp::now(),
        };

        let envelope = event.to_envelope().with_causation_id(causation_id);
        self.event_publisher.publish(envelope).await?;

        debug!(
            cycle_id = %cycle_id,
            dominated_count = dominated.len(),
            "Published TradeoffsAnalyzed event"
        );

        Ok(())
    }

    /// Parses a ConsequencesTable from component output JSON.
    fn parse_consequences_table(
        &self,
        output: &serde_json::Value,
    ) -> Result<ConsequencesTable, DomainError> {
        // Try to deserialize directly if format matches
        if let Ok(table) = serde_json::from_value::<ConsequencesTable>(output.clone()) {
            return Ok(table);
        }

        // Otherwise, build from structured fields
        let alternatives = output
            .get("alternatives")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                DomainError::new(ErrorCode::ValidationFailed, "Missing alternatives in output")
            })?;

        let objectives = output
            .get("objectives")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                DomainError::new(ErrorCode::ValidationFailed, "Missing objectives in output")
            })?;

        // Collect alternative and objective IDs
        let alt_ids: Vec<String> = alternatives
            .iter()
            .filter_map(|alt| alt.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();

        let obj_ids: Vec<String> = objectives
            .iter()
            .filter_map(|obj| obj.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();

        // Build with collected IDs
        let mut builder = ConsequencesTableBuilder::new()
            .alternatives(alt_ids)
            .objectives(obj_ids);

        // Add ratings from cells array or nested structure
        if let Some(cells) = output.get("cells").and_then(|v| v.as_array()) {
            for cell in cells {
                let alt_id = cell
                    .get("alternative_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let obj_id = cell
                    .get("objective_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let rating_value = cell
                    .get("rating")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i8;

                if !alt_id.is_empty() && !obj_id.is_empty() {
                    // Clamp rating value to valid range and convert
                    let clamped = rating_value.clamp(-2, 2);
                    let rating = Rating::try_from_i8(clamped).unwrap_or_default();
                    builder = builder.cell(alt_id, obj_id, rating);
                }
            }
        }

        Ok(builder.build())
    }

    /// Parses DQ elements from component output JSON.
    fn parse_dq_elements(
        &self,
        output: &serde_json::Value,
    ) -> Result<Vec<DQElement>, DomainError> {
        // Try to deserialize from elements array
        let elements_json = output
            .get("elements")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                DomainError::new(
                    ErrorCode::ValidationFailed,
                    "Missing elements array in DQ output",
                )
            })?;

        let mut elements = Vec::new();
        for elem in elements_json {
            let name = elem
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();

            let score = elem
                .get("score")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u8;

            let rationale = elem
                .get("rationale")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let improvement_path = elem
                .get("improvement_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            elements.push(DQElement {
                name,
                score: Percentage::new(score),
                rationale,
                improvement_path,
            });
        }

        if elements.is_empty() {
            return Err(DomainError::new(
                ErrorCode::ValidationFailed,
                "No DQ elements found in output",
            ));
        }

        Ok(elements)
    }
}

#[async_trait]
impl EventHandler for AnalysisTriggerHandler {
    async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError> {
        // Parse component completed event
        let payload: ComponentCompletedPayload = serde_json::from_value(event.payload.clone())
            .map_err(|e| DomainError::new(ErrorCode::ValidationFailed, e.to_string()))?;

        // Get cycle info to find session_id
        let cycle_view = self
            .cycle_reader
            .get_by_id(&payload.cycle_id)
            .await?
            .ok_or_else(|| {
                DomainError::new(
                    ErrorCode::CycleNotFound,
                    format!("Cycle not found: {}", payload.cycle_id),
                )
            })?;

        let session_id = cycle_view.session_id;
        let causation_id = event.event_id.as_str();

        // Handle based on component type
        match payload.component_type {
            ComponentType::Consequences => {
                self.handle_consequences_completed(payload.cycle_id, session_id, causation_id)
                    .await?;
            }
            ComponentType::DecisionQuality => {
                self.handle_dq_completed(payload.cycle_id, session_id, causation_id)
                    .await?;
            }
            ComponentType::Tradeoffs => {
                self.handle_tradeoffs_completed(payload.cycle_id, session_id, causation_id)
                    .await?;
            }
            _ => {
                // Other component types don't trigger analysis
                debug!(
                    component_type = ?payload.component_type,
                    "Component completion does not trigger analysis"
                );
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "AnalysisTriggerHandler"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{ComponentStatus, CycleStatus};
    use crate::ports::{
        ComponentOutputView, CycleProgressView, CycleSummary, CycleTreeNode, CycleView,
    };
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ─────────────────────────────────────────────────────────────────────
    // Mock implementations
    // ─────────────────────────────────────────────────────────────────────

    struct MockCycleReader {
        cycle_view: Option<CycleView>,
        component_outputs: Mutex<HashMap<ComponentType, ComponentOutputView>>,
    }

    impl MockCycleReader {
        fn with_cycle_and_output(
            cycle_view: CycleView,
            component_type: ComponentType,
            output: serde_json::Value,
        ) -> Self {
            let mut outputs = HashMap::new();
            outputs.insert(
                component_type,
                ComponentOutputView {
                    cycle_id: cycle_view.id,
                    component_type,
                    status: ComponentStatus::Complete,
                    output,
                    updated_at: Timestamp::now(),
                },
            );
            Self {
                cycle_view: Some(cycle_view),
                component_outputs: Mutex::new(outputs),
            }
        }

        fn empty() -> Self {
            Self {
                cycle_view: None,
                component_outputs: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl CycleReader for MockCycleReader {
        async fn get_by_id(&self, _id: &CycleId) -> Result<Option<CycleView>, DomainError> {
            Ok(self.cycle_view.clone())
        }

        async fn list_by_session_id(
            &self,
            _session_id: &SessionId,
        ) -> Result<Vec<CycleSummary>, DomainError> {
            Ok(vec![])
        }

        async fn get_tree(
            &self,
            _session_id: &SessionId,
        ) -> Result<Option<CycleTreeNode>, DomainError> {
            Ok(None)
        }

        async fn get_progress(
            &self,
            _id: &CycleId,
        ) -> Result<Option<CycleProgressView>, DomainError> {
            Ok(None)
        }

        async fn get_lineage(&self, _id: &CycleId) -> Result<Vec<CycleSummary>, DomainError> {
            Ok(vec![])
        }

        async fn get_component_output(
            &self,
            _cycle_id: &CycleId,
            component_type: ComponentType,
        ) -> Result<Option<ComponentOutputView>, DomainError> {
            let outputs = self.component_outputs.lock().unwrap();
            Ok(outputs.get(&component_type).cloned())
        }

        async fn get_proact_tree_view(
            &self,
            _session_id: &SessionId,
        ) -> Result<Option<crate::domain::cycle::CycleTreeNode>, DomainError> {
            Ok(None)
        }
    }

    struct MockEventPublisher {
        published_events: Mutex<Vec<EventEnvelope>>,
    }

    impl MockEventPublisher {
        fn new() -> Self {
            Self {
                published_events: Mutex::new(Vec::new()),
            }
        }

        fn published_events(&self) -> Vec<EventEnvelope> {
            self.published_events.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl EventPublisher for MockEventPublisher {
        async fn publish(&self, event: EventEnvelope) -> Result<(), DomainError> {
            self.published_events.lock().unwrap().push(event);
            Ok(())
        }

        async fn publish_all(&self, events: Vec<EventEnvelope>) -> Result<(), DomainError> {
            for event in events {
                self.publish(event).await?;
            }
            Ok(())
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Test helpers
    // ─────────────────────────────────────────────────────────────────────

    fn test_cycle_view() -> CycleView {
        CycleView {
            id: CycleId::new(),
            session_id: SessionId::new(),
            parent_cycle_id: None,
            branch_point: None,
            status: CycleStatus::Active,
            current_step: ComponentType::Consequences,
            component_statuses: vec![],
            progress_percent: 50,
            is_complete: false,
            branch_count: 0,
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        }
    }

    fn component_completed_event(cycle_id: CycleId, component_type: ComponentType) -> EventEnvelope {
        EventEnvelope {
            event_id: EventId::from_string("evt-component-completed-1"),
            event_type: "component.completed".to_string(),
            aggregate_id: cycle_id.to_string(),
            aggregate_type: "Cycle".to_string(),
            occurred_at: Timestamp::now(),
            payload: json!({
                "event_id": EventId::new().to_string(),
                "cycle_id": cycle_id.to_string(),
                "component_type": component_type,
                "completed_at": serde_json::to_value(Timestamp::now()).unwrap(),
            }),
            metadata: Default::default(),
        }
    }

    fn consequences_table_output() -> serde_json::Value {
        json!({
            "alternatives": [
                {"id": "alt-1"},
                {"id": "alt-2"}
            ],
            "objectives": [
                {"id": "obj-1"},
                {"id": "obj-2"}
            ],
            "cells": [
                {"alternative_id": "alt-1", "objective_id": "obj-1", "rating": 2},
                {"alternative_id": "alt-1", "objective_id": "obj-2", "rating": 1},
                {"alternative_id": "alt-2", "objective_id": "obj-1", "rating": -1},
                {"alternative_id": "alt-2", "objective_id": "obj-2", "rating": 0}
            ]
        })
    }

    fn dq_elements_output() -> serde_json::Value {
        json!({
            "elements": [
                {"name": "Helpful Problem Frame", "score": 85, "rationale": "Clear framing"},
                {"name": "Clear Objectives", "score": 70, "rationale": "Some ambiguity"},
                {"name": "Creative Alternatives", "score": 90, "rationale": "Good options"},
                {"name": "Reliable Consequence Information", "score": 75, "rationale": "Mostly reliable"},
                {"name": "Logically Correct Reasoning", "score": 80, "rationale": "Sound logic"},
                {"name": "Clear Tradeoffs", "score": 65, "rationale": "Needs work"},
                {"name": "Commitment to Follow Through", "score": 95, "rationale": "Strong commitment"}
            ]
        })
    }

    // ─────────────────────────────────────────────────────────────────────
    // Tests
    // ─────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn handler_name_is_correct() {
        let reader = Arc::new(MockCycleReader::empty());
        let publisher = Arc::new(MockEventPublisher::new());
        let handler = AnalysisTriggerHandler::new(reader, publisher);

        assert_eq!(handler.name(), "AnalysisTriggerHandler");
    }

    #[tokio::test]
    async fn publishes_pugh_scores_on_consequences_completion() {
        let cycle_view = test_cycle_view();
        let cycle_id = cycle_view.id;

        let reader = Arc::new(MockCycleReader::with_cycle_and_output(
            cycle_view,
            ComponentType::Consequences,
            consequences_table_output(),
        ));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = AnalysisTriggerHandler::new(reader, publisher.clone());

        let event = component_completed_event(cycle_id, ComponentType::Consequences);
        let result = handler.handle(event).await;

        assert!(result.is_ok());

        let events = publisher.published_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "analysis.pugh_scores_computed");
    }

    #[tokio::test]
    async fn publishes_dq_scores_on_dq_completion() {
        let cycle_view = test_cycle_view();
        let cycle_id = cycle_view.id;

        let reader = Arc::new(MockCycleReader::with_cycle_and_output(
            cycle_view,
            ComponentType::DecisionQuality,
            dq_elements_output(),
        ));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = AnalysisTriggerHandler::new(reader, publisher.clone());

        let event = component_completed_event(cycle_id, ComponentType::DecisionQuality);
        let result = handler.handle(event).await;

        assert!(result.is_ok());

        let events = publisher.published_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "analysis.dq_scores_computed");
    }

    #[tokio::test]
    async fn publishes_tradeoffs_on_tradeoffs_completion() {
        let cycle_view = test_cycle_view();
        let cycle_id = cycle_view.id;

        let reader = Arc::new(MockCycleReader::with_cycle_and_output(
            cycle_view,
            ComponentType::Consequences, // Tradeoffs reads from Consequences
            consequences_table_output(),
        ));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = AnalysisTriggerHandler::new(reader, publisher.clone());

        let event = component_completed_event(cycle_id, ComponentType::Tradeoffs);
        let result = handler.handle(event).await;

        assert!(result.is_ok());

        let events = publisher.published_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "analysis.tradeoffs_analyzed");
    }

    #[tokio::test]
    async fn ignores_non_analysis_component_completions() {
        let cycle_view = test_cycle_view();
        let cycle_id = cycle_view.id;

        let reader = Arc::new(MockCycleReader::with_cycle_and_output(
            cycle_view,
            ComponentType::IssueRaising,
            json!({}),
        ));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = AnalysisTriggerHandler::new(reader, publisher.clone());

        let event = component_completed_event(cycle_id, ComponentType::IssueRaising);
        let result = handler.handle(event).await;

        assert!(result.is_ok());
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn fails_when_cycle_not_found() {
        let reader = Arc::new(MockCycleReader::empty());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = AnalysisTriggerHandler::new(reader, publisher.clone());

        let event = component_completed_event(CycleId::new(), ComponentType::Consequences);
        let result = handler.handle(event).await;

        assert!(result.is_err());
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn pugh_scores_computed_contains_correct_data() {
        let cycle_view = test_cycle_view();
        let cycle_id = cycle_view.id;
        let session_id = cycle_view.session_id;

        let reader = Arc::new(MockCycleReader::with_cycle_and_output(
            cycle_view,
            ComponentType::Consequences,
            consequences_table_output(),
        ));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = AnalysisTriggerHandler::new(reader, publisher.clone());

        let event = component_completed_event(cycle_id, ComponentType::Consequences);
        handler.handle(event).await.unwrap();

        let events = publisher.published_events();
        let payload: PughScoresComputed =
            serde_json::from_value(events[0].payload.clone()).unwrap();

        assert_eq!(payload.cycle_id, cycle_id);
        assert_eq!(payload.session_id, session_id);
        // alt-1: 2 + 1 = 3, alt-2: -1 + 0 = -1
        assert_eq!(*payload.alternative_scores.get("alt-1").unwrap(), 3);
        assert_eq!(*payload.alternative_scores.get("alt-2").unwrap(), -1);
        assert_eq!(payload.best_alternative_id, Some("alt-1".to_string()));
    }

    #[tokio::test]
    async fn dq_scores_computed_contains_overall_minimum() {
        let cycle_view = test_cycle_view();
        let cycle_id = cycle_view.id;

        let reader = Arc::new(MockCycleReader::with_cycle_and_output(
            cycle_view,
            ComponentType::DecisionQuality,
            dq_elements_output(),
        ));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = AnalysisTriggerHandler::new(reader, publisher.clone());

        let event = component_completed_event(cycle_id, ComponentType::DecisionQuality);
        handler.handle(event).await.unwrap();

        let events = publisher.published_events();
        let payload: DQScoresComputed =
            serde_json::from_value(events[0].payload.clone()).unwrap();

        // Overall should be the minimum (65% for "Clear Tradeoffs")
        assert_eq!(payload.overall_score.value(), 65);
        assert_eq!(payload.weakest_element, "Clear Tradeoffs");
    }

    #[tokio::test]
    async fn includes_causation_id_from_original_event() {
        let cycle_view = test_cycle_view();
        let cycle_id = cycle_view.id;

        let reader = Arc::new(MockCycleReader::with_cycle_and_output(
            cycle_view,
            ComponentType::Consequences,
            consequences_table_output(),
        ));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = AnalysisTriggerHandler::new(reader, publisher.clone());

        let mut event = component_completed_event(cycle_id, ComponentType::Consequences);
        event.event_id = EventId::from_string("original-event-123");

        handler.handle(event).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(
            events[0].metadata.causation_id,
            Some("original-event-123".to_string())
        );
    }
}
