//! Decision history tracking for outcome and pattern analysis

use crate::domain::foundation::{CycleId, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Decision domain categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionDomain {
    Career,
    Financial,
    Family,
    Health,
    Relationship,
    Education,
    Housing,
    Lifestyle,
    Business,
    Other,
}

impl std::fmt::Display for DecisionDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Career => write!(f, "Career"),
            Self::Financial => write!(f, "Financial"),
            Self::Family => write!(f, "Family"),
            Self::Health => write!(f, "Health"),
            Self::Relationship => write!(f, "Relationship"),
            Self::Education => write!(f, "Education"),
            Self::Housing => write!(f, "Housing"),
            Self::Lifestyle => write!(f, "Lifestyle"),
            Self::Business => write!(f, "Business"),
            Self::Other => write!(f, "Other"),
        }
    }
}

/// Satisfaction level with decision outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SatisfactionLevel {
    VeryDissatisfied,
    Dissatisfied,
    Neutral,
    Satisfied,
    VerySatisfied,
}

impl SatisfactionLevel {
    pub fn to_score(&self) -> u8 {
        match self {
            Self::VeryDissatisfied => 1,
            Self::Dissatisfied => 2,
            Self::Neutral => 3,
            Self::Satisfied => 4,
            Self::VerySatisfied => 5,
        }
    }
}

impl std::fmt::Display for SatisfactionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VeryDissatisfied => write!(f, "Very Dissatisfied"),
            Self::Dissatisfied => write!(f, "Dissatisfied"),
            Self::Neutral => write!(f, "Neutral"),
            Self::Satisfied => write!(f, "Satisfied"),
            Self::VerySatisfied => write!(f, "Very Satisfied"),
        }
    }
}

/// Actual outcome of a decision
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutcomeRecord {
    pub recorded_at: Timestamp,
    pub satisfaction: SatisfactionLevel,
    pub actual_consequences: String,
    pub surprises: Vec<String>,
    pub would_decide_same: bool,
}

impl OutcomeRecord {
    pub fn new(
        recorded_at: Timestamp,
        satisfaction: SatisfactionLevel,
        actual_consequences: String,
        surprises: Vec<String>,
        would_decide_same: bool,
    ) -> Result<Self, String> {
        if actual_consequences.trim().is_empty() {
            return Err("Actual consequences cannot be empty".to_string());
        }

        Ok(Self {
            recorded_at,
            satisfaction,
            actual_consequences,
            surprises,
            would_decide_same,
        })
    }
}

/// Record of a single decision
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub cycle_id: CycleId,
    pub date: Timestamp,
    pub title: String,
    pub domain: DecisionDomain,
    pub dq_score: Option<u8>,
    pub key_tradeoff: String,
    pub chosen_alternative: String,
    pub outcome: Option<OutcomeRecord>,
}

impl DecisionRecord {
    pub fn new(
        cycle_id: CycleId,
        date: Timestamp,
        title: String,
        domain: DecisionDomain,
        dq_score: Option<u8>,
        key_tradeoff: String,
        chosen_alternative: String,
    ) -> Result<Self, String> {
        if title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }
        if key_tradeoff.trim().is_empty() {
            return Err("Key tradeoff cannot be empty".to_string());
        }
        if chosen_alternative.trim().is_empty() {
            return Err("Chosen alternative cannot be empty".to_string());
        }
        if let Some(score) = dq_score {
            if score > 100 {
                return Err("DQ score must be 0-100".to_string());
            }
        }

        Ok(Self {
            cycle_id,
            date,
            title,
            domain,
            dq_score,
            key_tradeoff,
            chosen_alternative,
            outcome: None,
        })
    }

    /// Record outcome for this decision
    pub fn record_outcome(&mut self, outcome: OutcomeRecord) {
        self.outcome = Some(outcome);
    }

    /// Check if outcome has been recorded
    pub fn has_outcome(&self) -> bool {
        self.outcome.is_some()
    }
}

/// Statistics for a decision domain
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DomainStats {
    pub decision_count: u32,
    pub average_dq: f32,
    pub success_rate: f32,
    pub notes: Option<String>,
}

impl DomainStats {
    pub fn new(
        decision_count: u32,
        average_dq: f32,
        success_rate: f32,
        notes: Option<String>,
    ) -> Result<Self, String> {
        if !(0.0..=100.0).contains(&average_dq) {
            return Err("Average DQ must be 0-100".to_string());
        }
        if !(0.0..=1.0).contains(&success_rate) {
            return Err("Success rate must be 0.0-1.0".to_string());
        }

        Ok(Self {
            decision_count,
            average_dq,
            success_rate,
            notes,
        })
    }
}

/// Prediction accuracy metrics
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PredictionAccuracy {
    pub consequence_accuracy: f32,
    pub satisfaction_accuracy: f32,
    pub timeline_accuracy: f32,
    pub sample_size: u32,
}

impl PredictionAccuracy {
    pub fn new(
        consequence_accuracy: f32,
        satisfaction_accuracy: f32,
        timeline_accuracy: f32,
        sample_size: u32,
    ) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&consequence_accuracy) {
            return Err("Consequence accuracy must be 0.0-1.0".to_string());
        }
        if !(0.0..=1.0).contains(&satisfaction_accuracy) {
            return Err("Satisfaction accuracy must be 0.0-1.0".to_string());
        }
        if !(0.0..=1.0).contains(&timeline_accuracy) {
            return Err("Timeline accuracy must be 0.0-1.0".to_string());
        }

        Ok(Self {
            consequence_accuracy,
            satisfaction_accuracy,
            timeline_accuracy,
            sample_size,
        })
    }
}

impl Default for PredictionAccuracy {
    fn default() -> Self {
        Self {
            consequence_accuracy: 0.0,
            satisfaction_accuracy: 0.0,
            timeline_accuracy: 0.0,
            sample_size: 0,
        }
    }
}

/// Complete decision history
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionHistory {
    /// Individual decision records
    pub decisions: Vec<DecisionRecord>,
    /// Aggregated patterns by domain
    pub domain_patterns: HashMap<DecisionDomain, DomainStats>,
    /// Outcome tracking accuracy
    pub prediction_accuracy: PredictionAccuracy,
}

impl DecisionHistory {
    pub fn new(
        decisions: Vec<DecisionRecord>,
        domain_patterns: HashMap<DecisionDomain, DomainStats>,
        prediction_accuracy: PredictionAccuracy,
    ) -> Self {
        Self {
            decisions,
            domain_patterns,
            prediction_accuracy,
        }
    }

    /// Get total decision count
    pub fn decision_count(&self) -> usize {
        self.decisions.len()
    }

    /// Get decisions with outcomes
    pub fn decisions_with_outcomes(&self) -> Vec<&DecisionRecord> {
        self.decisions
            .iter()
            .filter(|d| d.has_outcome())
            .collect()
    }

    /// Get decisions by domain
    pub fn decisions_by_domain(&self, domain: DecisionDomain) -> Vec<&DecisionRecord> {
        self.decisions.iter().filter(|d| d.domain == domain).collect()
    }

    /// Calculate average DQ score
    pub fn average_dq(&self) -> Option<f32> {
        let scores: Vec<u8> = self
            .decisions
            .iter()
            .filter_map(|d| d.dq_score)
            .collect();

        if scores.is_empty() {
            None
        } else {
            let sum: u32 = scores.iter().map(|&s| s as u32).sum();
            Some(sum as f32 / scores.len() as f32)
        }
    }
}

impl Default for DecisionHistory {
    fn default() -> Self {
        Self {
            decisions: Vec::new(),
            domain_patterns: HashMap::new(),
            prediction_accuracy: PredictionAccuracy::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_timestamp() -> Timestamp {
        Timestamp::from_datetime(chrono::DateTime::from_timestamp(1704326400, 0).unwrap())
    }

    #[test]
    fn test_decision_domain_display() {
        assert_eq!(format!("{}", DecisionDomain::Career), "Career");
        assert_eq!(format!("{}", DecisionDomain::Financial), "Financial");
    }

    #[test]
    fn test_satisfaction_level_to_score() {
        assert_eq!(SatisfactionLevel::VeryDissatisfied.to_score(), 1);
        assert_eq!(SatisfactionLevel::Dissatisfied.to_score(), 2);
        assert_eq!(SatisfactionLevel::Neutral.to_score(), 3);
        assert_eq!(SatisfactionLevel::Satisfied.to_score(), 4);
        assert_eq!(SatisfactionLevel::VerySatisfied.to_score(), 5);
    }

    #[test]
    fn test_outcome_record_creation() {
        let ts = test_timestamp();
        let outcome = OutcomeRecord::new(
            ts,
            SatisfactionLevel::Satisfied,
            "Met expectations".to_string(),
            vec!["Surprise 1".to_string()],
            true,
        );

        assert!(outcome.is_ok());
        let o = outcome.unwrap();
        assert_eq!(o.satisfaction, SatisfactionLevel::Satisfied);
        assert!(o.would_decide_same);
    }

    #[test]
    fn test_outcome_record_validation() {
        let ts = test_timestamp();

        // Empty consequences
        assert!(OutcomeRecord::new(
            ts,
            SatisfactionLevel::Neutral,
            "".to_string(),
            vec![],
            true
        )
        .is_err());
    }

    #[test]
    fn test_decision_record_creation() {
        let ts = test_timestamp();
        let cycle_id = CycleId::new();

        let record = DecisionRecord::new(
            cycle_id,
            ts,
            "Accept job offer".to_string(),
            DecisionDomain::Career,
            Some(85),
            "Growth vs Stability".to_string(),
            "Accept with negotiation".to_string(),
        );

        assert!(record.is_ok());
        let r = record.unwrap();
        assert_eq!(r.title, "Accept job offer");
        assert_eq!(r.domain, DecisionDomain::Career);
        assert_eq!(r.dq_score, Some(85));
        assert!(!r.has_outcome());
    }

    #[test]
    fn test_decision_record_validation() {
        let ts = test_timestamp();
        let cycle_id = CycleId::new();

        // Empty title
        assert!(DecisionRecord::new(
            cycle_id,
            ts,
            "".to_string(),
            DecisionDomain::Career,
            Some(85),
            "Tradeoff".to_string(),
            "Alternative".to_string(),
        )
        .is_err());

        // Invalid DQ score
        assert!(DecisionRecord::new(
            cycle_id,
            ts,
            "Title".to_string(),
            DecisionDomain::Career,
            Some(101),
            "Tradeoff".to_string(),
            "Alternative".to_string(),
        )
        .is_err());
    }

    #[test]
    fn test_decision_record_outcome() {
        let ts = test_timestamp();
        let cycle_id = CycleId::new();

        let mut record = DecisionRecord::new(
            cycle_id,
            ts,
            "Decision".to_string(),
            DecisionDomain::Career,
            Some(80),
            "Tradeoff".to_string(),
            "Alternative".to_string(),
        )
        .unwrap();

        assert!(!record.has_outcome());

        let outcome = OutcomeRecord::new(
            ts,
            SatisfactionLevel::Satisfied,
            "Good outcome".to_string(),
            vec![],
            true,
        )
        .unwrap();

        record.record_outcome(outcome);
        assert!(record.has_outcome());
    }

    #[test]
    fn test_domain_stats_creation() {
        let stats = DomainStats::new(10, 82.5, 0.8, Some("Strong domain".to_string()));

        assert!(stats.is_ok());
        let s = stats.unwrap();
        assert_eq!(s.decision_count, 10);
        assert_eq!(s.average_dq, 82.5);
        assert_eq!(s.success_rate, 0.8);
    }

    #[test]
    fn test_domain_stats_validation() {
        // Invalid average DQ
        assert!(DomainStats::new(5, 101.0, 0.5, None).is_err());
        assert!(DomainStats::new(5, -1.0, 0.5, None).is_err());

        // Invalid success rate
        assert!(DomainStats::new(5, 80.0, 1.1, None).is_err());
        assert!(DomainStats::new(5, 80.0, -0.1, None).is_err());
    }

    #[test]
    fn test_prediction_accuracy_creation() {
        let accuracy = PredictionAccuracy::new(0.75, 0.82, 0.68, 10);

        assert!(accuracy.is_ok());
        let a = accuracy.unwrap();
        assert_eq!(a.consequence_accuracy, 0.75);
        assert_eq!(a.sample_size, 10);
    }

    #[test]
    fn test_prediction_accuracy_validation() {
        assert!(PredictionAccuracy::new(1.1, 0.5, 0.5, 10).is_err());
        assert!(PredictionAccuracy::new(0.5, 1.1, 0.5, 10).is_err());
        assert!(PredictionAccuracy::new(0.5, 0.5, 1.1, 10).is_err());
    }

    #[test]
    fn test_decision_history_default() {
        let history = DecisionHistory::default();
        assert_eq!(history.decision_count(), 0);
        assert_eq!(history.decisions_with_outcomes().len(), 0);
    }

    #[test]
    fn test_decision_history_average_dq() {
        let ts = test_timestamp();

        let d1 = DecisionRecord::new(
            CycleId::new(),
            ts,
            "D1".to_string(),
            DecisionDomain::Career,
            Some(80),
            "T".to_string(),
            "A".to_string(),
        )
        .unwrap();

        let d2 = DecisionRecord::new(
            CycleId::new(),
            ts,
            "D2".to_string(),
            DecisionDomain::Career,
            Some(90),
            "T".to_string(),
            "A".to_string(),
        )
        .unwrap();

        let d3 = DecisionRecord::new(
            CycleId::new(),
            ts,
            "D3".to_string(),
            DecisionDomain::Career,
            None, // No DQ score
            "T".to_string(),
            "A".to_string(),
        )
        .unwrap();

        let history = DecisionHistory::new(vec![d1, d2, d3], HashMap::new(), PredictionAccuracy::default());

        // Average of 80 and 90 = 85.0
        assert_eq!(history.average_dq(), Some(85.0));
    }

    #[test]
    fn test_decision_history_decisions_by_domain() {
        let ts = test_timestamp();

        let d1 = DecisionRecord::new(
            CycleId::new(),
            ts,
            "Career decision".to_string(),
            DecisionDomain::Career,
            Some(80),
            "T".to_string(),
            "A".to_string(),
        )
        .unwrap();

        let d2 = DecisionRecord::new(
            CycleId::new(),
            ts,
            "Financial decision".to_string(),
            DecisionDomain::Financial,
            Some(75),
            "T".to_string(),
            "A".to_string(),
        )
        .unwrap();

        let d3 = DecisionRecord::new(
            CycleId::new(),
            ts,
            "Another career".to_string(),
            DecisionDomain::Career,
            Some(85),
            "T".to_string(),
            "A".to_string(),
        )
        .unwrap();

        let history = DecisionHistory::new(vec![d1, d2, d3], HashMap::new(), PredictionAccuracy::default());

        let career_decisions = history.decisions_by_domain(DecisionDomain::Career);
        assert_eq!(career_decisions.len(), 2);

        let financial_decisions = history.decisions_by_domain(DecisionDomain::Financial);
        assert_eq!(financial_decisions.len(), 1);
    }
}
