//! Analysis domain events.
//!
//! Events published when analysis computations complete. These enable:
//! - Dashboard updates with computed scores
//! - WebSocket real-time notifications
//! - Audit trails for analysis computations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::foundation::{
    domain_event, CycleId, EventId, Percentage, SessionId, Timestamp,
};

/// Published when Pugh matrix scores are computed for a cycle.
///
/// This event is triggered by `ComponentCompleted` for the Consequences component.
/// It contains summarized results, not raw data, to keep payload size manageable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PughScoresComputed {
    /// Unique event identifier for deduplication.
    pub event_id: EventId,
    /// The cycle this analysis belongs to.
    pub cycle_id: CycleId,
    /// The session containing this cycle.
    pub session_id: SessionId,
    /// Map of alternative_id -> total score (sum of ratings).
    pub alternative_scores: HashMap<String, i32>,
    /// IDs of alternatives dominated by at least one other alternative.
    pub dominated_alternatives: Vec<String>,
    /// IDs of objectives that don't distinguish between alternatives.
    pub irrelevant_objectives: Vec<String>,
    /// ID of the best-scoring alternative (None if tie or empty).
    pub best_alternative_id: Option<String>,
    /// When the analysis was computed.
    pub computed_at: Timestamp,
}

domain_event!(
    PughScoresComputed,
    event_type = "analysis.pugh_scores_computed",
    schema_version = 1,
    aggregate_id = cycle_id,
    aggregate_type = "Analysis",
    occurred_at = computed_at,
    event_id = event_id
);

/// Scored element for Decision Quality assessment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DQElementScore {
    /// Name of the DQ element (e.g., "Helpful Problem Frame").
    pub element_name: String,
    /// Score for this element (0-100%).
    pub score: Percentage,
    /// Rationale for the score.
    pub rationale: String,
}

/// Published when Decision Quality scores are computed for a cycle.
///
/// This event is triggered by `ComponentCompleted` for the DecisionQuality component.
/// The overall score is the minimum of all element scores (weakest link principle).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DQScoresComputed {
    /// Unique event identifier for deduplication.
    pub event_id: EventId,
    /// The cycle this analysis belongs to.
    pub cycle_id: CycleId,
    /// The session containing this cycle.
    pub session_id: SessionId,
    /// Scores for each DQ element.
    pub element_scores: Vec<DQElementScore>,
    /// Overall DQ score (minimum of all elements).
    pub overall_score: Percentage,
    /// Name of the element with the lowest score.
    pub weakest_element: String,
    /// Suggested improvement paths for low-scoring elements.
    pub improvement_suggestions: Vec<String>,
    /// When the analysis was computed.
    pub computed_at: Timestamp,
}

domain_event!(
    DQScoresComputed,
    event_type = "analysis.dq_scores_computed",
    schema_version = 1,
    aggregate_id = cycle_id,
    aggregate_type = "Analysis",
    occurred_at = computed_at,
    event_id = event_id
);

/// Summary of tradeoff tensions for a single alternative.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TensionSummary {
    /// The alternative being analyzed.
    pub alternative_id: String,
    /// Human-readable name of the alternative.
    pub alternative_name: String,
    /// Objectives where this alternative outperforms others.
    pub strengths: Vec<String>,
    /// Objectives where this alternative underperforms others.
    pub weaknesses: Vec<String>,
}

/// Published when tradeoff analysis is completed for a cycle.
///
/// This event is triggered by `ComponentCompleted` for the Tradeoffs component.
/// It summarizes dominated alternatives and tension analysis for viable options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeoffsAnalyzed {
    /// Unique event identifier for deduplication.
    pub event_id: EventId,
    /// The cycle this analysis belongs to.
    pub cycle_id: CycleId,
    /// The session containing this cycle.
    pub session_id: SessionId,
    /// Number of dominated alternatives found.
    pub dominated_count: i32,
    /// Number of irrelevant objectives found.
    pub irrelevant_count: i32,
    /// Tension summaries for non-dominated alternatives.
    pub tension_summaries: Vec<TensionSummary>,
    /// When the analysis was completed.
    pub analyzed_at: Timestamp,
}

domain_event!(
    TradeoffsAnalyzed,
    event_type = "analysis.tradeoffs_analyzed",
    schema_version = 1,
    aggregate_id = cycle_id,
    aggregate_type = "Analysis",
    occurred_at = analyzed_at,
    event_id = event_id
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{DomainEvent, SerializableDomainEvent};

    fn test_cycle_id() -> CycleId {
        CycleId::new()
    }

    fn test_session_id() -> SessionId {
        SessionId::new()
    }

    // ─────────────────────────────────────────────────────────────────────
    // PughScoresComputed Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn pugh_scores_computed_event_type() {
        let event = PughScoresComputed {
            event_id: EventId::new(),
            cycle_id: test_cycle_id(),
            session_id: test_session_id(),
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
    fn pugh_scores_computed_aggregate_id() {
        let cycle_id = test_cycle_id();
        let event = PughScoresComputed {
            event_id: EventId::new(),
            cycle_id,
            session_id: test_session_id(),
            alternative_scores: HashMap::new(),
            dominated_alternatives: vec![],
            irrelevant_objectives: vec![],
            best_alternative_id: None,
            computed_at: Timestamp::now(),
        };

        assert_eq!(event.aggregate_id(), cycle_id.to_string());
    }

    #[test]
    fn pugh_scores_computed_aggregate_type() {
        let event = PughScoresComputed {
            event_id: EventId::new(),
            cycle_id: test_cycle_id(),
            session_id: test_session_id(),
            alternative_scores: HashMap::new(),
            dominated_alternatives: vec![],
            irrelevant_objectives: vec![],
            best_alternative_id: None,
            computed_at: Timestamp::now(),
        };

        assert_eq!(event.aggregate_type(), "Analysis");
    }

    #[test]
    fn pugh_scores_computed_to_envelope() {
        let event = PughScoresComputed {
            event_id: EventId::from_string("evt-pugh-1"),
            cycle_id: test_cycle_id(),
            session_id: test_session_id(),
            alternative_scores: HashMap::from([("alt-1".to_string(), 3)]),
            dominated_alternatives: vec![],
            irrelevant_objectives: vec![],
            best_alternative_id: Some("alt-1".to_string()),
            computed_at: Timestamp::now(),
        };

        let envelope = event.to_envelope();

        assert_eq!(envelope.event_type, "analysis.pugh_scores_computed");
        assert_eq!(envelope.aggregate_type, "Analysis");
        assert_eq!(envelope.event_id.as_str(), "evt-pugh-1");
    }

    #[test]
    fn pugh_scores_computed_serialization_round_trip() {
        let event = PughScoresComputed {
            event_id: EventId::new(),
            cycle_id: test_cycle_id(),
            session_id: test_session_id(),
            alternative_scores: HashMap::from([
                ("alt-1".to_string(), 5),
                ("alt-2".to_string(), -2),
            ]),
            dominated_alternatives: vec!["alt-2".to_string()],
            irrelevant_objectives: vec!["obj-3".to_string()],
            best_alternative_id: Some("alt-1".to_string()),
            computed_at: Timestamp::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let restored: PughScoresComputed = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.alternative_scores, event.alternative_scores);
        assert_eq!(restored.dominated_alternatives, event.dominated_alternatives);
        assert_eq!(restored.best_alternative_id, event.best_alternative_id);
    }

    // ─────────────────────────────────────────────────────────────────────
    // DQScoresComputed Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn dq_scores_computed_event_type() {
        let event = DQScoresComputed {
            event_id: EventId::new(),
            cycle_id: test_cycle_id(),
            session_id: test_session_id(),
            element_scores: vec![
                DQElementScore {
                    element_name: "Clear Objectives".to_string(),
                    score: Percentage::new(80),
                    rationale: "Well defined".to_string(),
                },
            ],
            overall_score: Percentage::new(80),
            weakest_element: "Clear Objectives".to_string(),
            improvement_suggestions: vec![],
            computed_at: Timestamp::now(),
        };

        assert_eq!(event.event_type(), "analysis.dq_scores_computed");
    }

    #[test]
    fn dq_scores_computed_has_overall_minimum() {
        let event = DQScoresComputed {
            event_id: EventId::new(),
            cycle_id: test_cycle_id(),
            session_id: test_session_id(),
            element_scores: vec![
                DQElementScore {
                    element_name: "Clear Objectives".to_string(),
                    score: Percentage::new(80),
                    rationale: "Well defined".to_string(),
                },
                DQElementScore {
                    element_name: "Creative Alternatives".to_string(),
                    score: Percentage::new(60),
                    rationale: "Limited options".to_string(),
                },
            ],
            overall_score: Percentage::new(60), // Minimum
            weakest_element: "Creative Alternatives".to_string(),
            improvement_suggestions: vec!["Generate more alternatives".to_string()],
            computed_at: Timestamp::now(),
        };

        assert_eq!(event.overall_score.value(), 60);
        assert_eq!(event.weakest_element, "Creative Alternatives");
    }

    #[test]
    fn dq_scores_computed_aggregate_type() {
        let event = DQScoresComputed {
            event_id: EventId::new(),
            cycle_id: test_cycle_id(),
            session_id: test_session_id(),
            element_scores: vec![],
            overall_score: Percentage::ZERO,
            weakest_element: String::new(),
            improvement_suggestions: vec![],
            computed_at: Timestamp::now(),
        };

        assert_eq!(event.aggregate_type(), "Analysis");
    }

    #[test]
    fn dq_scores_computed_to_envelope() {
        let event = DQScoresComputed {
            event_id: EventId::from_string("evt-dq-1"),
            cycle_id: test_cycle_id(),
            session_id: test_session_id(),
            element_scores: vec![],
            overall_score: Percentage::new(75),
            weakest_element: "Test Element".to_string(),
            improvement_suggestions: vec![],
            computed_at: Timestamp::now(),
        };

        let envelope = event.to_envelope();

        assert_eq!(envelope.event_type, "analysis.dq_scores_computed");
        assert_eq!(envelope.event_id.as_str(), "evt-dq-1");
    }

    #[test]
    fn dq_scores_computed_serialization_round_trip() {
        let event = DQScoresComputed {
            event_id: EventId::new(),
            cycle_id: test_cycle_id(),
            session_id: test_session_id(),
            element_scores: vec![
                DQElementScore {
                    element_name: "Helpful Problem Frame".to_string(),
                    score: Percentage::new(85),
                    rationale: "Well framed".to_string(),
                },
                DQElementScore {
                    element_name: "Clear Objectives".to_string(),
                    score: Percentage::new(70),
                    rationale: "Could be clearer".to_string(),
                },
            ],
            overall_score: Percentage::new(70),
            weakest_element: "Clear Objectives".to_string(),
            improvement_suggestions: vec!["Clarify success criteria".to_string()],
            computed_at: Timestamp::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let restored: DQScoresComputed = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.overall_score.value(), 70);
        assert_eq!(restored.element_scores.len(), 2);
        assert_eq!(restored.improvement_suggestions.len(), 1);
    }

    // ─────────────────────────────────────────────────────────────────────
    // TradeoffsAnalyzed Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn tradeoffs_analyzed_event_type() {
        let event = TradeoffsAnalyzed {
            event_id: EventId::new(),
            cycle_id: test_cycle_id(),
            session_id: test_session_id(),
            dominated_count: 1,
            irrelevant_count: 0,
            tension_summaries: vec![],
            analyzed_at: Timestamp::now(),
        };

        assert_eq!(event.event_type(), "analysis.tradeoffs_analyzed");
    }

    #[test]
    fn tradeoffs_analyzed_aggregate_type() {
        let event = TradeoffsAnalyzed {
            event_id: EventId::new(),
            cycle_id: test_cycle_id(),
            session_id: test_session_id(),
            dominated_count: 0,
            irrelevant_count: 0,
            tension_summaries: vec![],
            analyzed_at: Timestamp::now(),
        };

        assert_eq!(event.aggregate_type(), "Analysis");
    }

    #[test]
    fn tradeoffs_analyzed_with_tension_summaries() {
        let event = TradeoffsAnalyzed {
            event_id: EventId::new(),
            cycle_id: test_cycle_id(),
            session_id: test_session_id(),
            dominated_count: 1,
            irrelevant_count: 2,
            tension_summaries: vec![
                TensionSummary {
                    alternative_id: "alt-1".to_string(),
                    alternative_name: "Option A".to_string(),
                    strengths: vec!["Cost".to_string(), "Speed".to_string()],
                    weaknesses: vec!["Quality".to_string()],
                },
                TensionSummary {
                    alternative_id: "alt-2".to_string(),
                    alternative_name: "Option B".to_string(),
                    strengths: vec!["Quality".to_string()],
                    weaknesses: vec!["Cost".to_string(), "Speed".to_string()],
                },
            ],
            analyzed_at: Timestamp::now(),
        };

        assert_eq!(event.tension_summaries.len(), 2);
        assert_eq!(event.tension_summaries[0].strengths.len(), 2);
        assert_eq!(event.tension_summaries[1].weaknesses.len(), 2);
    }

    #[test]
    fn tradeoffs_analyzed_serialization_round_trip() {
        let event = TradeoffsAnalyzed {
            event_id: EventId::new(),
            cycle_id: test_cycle_id(),
            session_id: test_session_id(),
            dominated_count: 2,
            irrelevant_count: 1,
            tension_summaries: vec![
                TensionSummary {
                    alternative_id: "alt-1".to_string(),
                    alternative_name: "Option A".to_string(),
                    strengths: vec!["Fast".to_string()],
                    weaknesses: vec!["Expensive".to_string()],
                },
            ],
            analyzed_at: Timestamp::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let restored: TradeoffsAnalyzed = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.dominated_count, 2);
        assert_eq!(restored.irrelevant_count, 1);
        assert_eq!(restored.tension_summaries.len(), 1);
        assert_eq!(
            restored.tension_summaries[0].alternative_name,
            "Option A"
        );
    }

    #[test]
    fn tradeoffs_analyzed_to_envelope() {
        let event = TradeoffsAnalyzed {
            event_id: EventId::from_string("evt-tradeoffs-1"),
            cycle_id: test_cycle_id(),
            session_id: test_session_id(),
            dominated_count: 0,
            irrelevant_count: 0,
            tension_summaries: vec![],
            analyzed_at: Timestamp::now(),
        };

        let envelope = event.to_envelope();

        assert_eq!(envelope.event_type, "analysis.tradeoffs_analyzed");
        assert_eq!(envelope.event_id.as_str(), "evt-tradeoffs-1");
        assert_eq!(envelope.aggregate_type, "Analysis");
    }

    // ─────────────────────────────────────────────────────────────────────
    // DQElementScore Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn dq_element_score_equality() {
        let score1 = DQElementScore {
            element_name: "Test".to_string(),
            score: Percentage::new(75),
            rationale: "Reason".to_string(),
        };
        let score2 = DQElementScore {
            element_name: "Test".to_string(),
            score: Percentage::new(75),
            rationale: "Reason".to_string(),
        };

        assert_eq!(score1, score2);
    }

    #[test]
    fn dq_element_score_serialization() {
        let score = DQElementScore {
            element_name: "Clear Objectives".to_string(),
            score: Percentage::new(85),
            rationale: "Well defined goals".to_string(),
        };

        let json = serde_json::to_string(&score).unwrap();
        assert!(json.contains("Clear Objectives"));
        assert!(json.contains("85"));

        let restored: DQElementScore = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, score);
    }

    // ─────────────────────────────────────────────────────────────────────
    // TensionSummary Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn tension_summary_equality() {
        let t1 = TensionSummary {
            alternative_id: "alt-1".to_string(),
            alternative_name: "Option A".to_string(),
            strengths: vec!["Fast".to_string()],
            weaknesses: vec!["Expensive".to_string()],
        };
        let t2 = TensionSummary {
            alternative_id: "alt-1".to_string(),
            alternative_name: "Option A".to_string(),
            strengths: vec!["Fast".to_string()],
            weaknesses: vec!["Expensive".to_string()],
        };

        assert_eq!(t1, t2);
    }

    #[test]
    fn tension_summary_serialization() {
        let tension = TensionSummary {
            alternative_id: "alt-1".to_string(),
            alternative_name: "Budget Option".to_string(),
            strengths: vec!["Cost".to_string(), "Simplicity".to_string()],
            weaknesses: vec!["Features".to_string()],
        };

        let json = serde_json::to_string(&tension).unwrap();
        assert!(json.contains("Budget Option"));
        assert!(json.contains("Cost"));

        let restored: TensionSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, tension);
    }
}
