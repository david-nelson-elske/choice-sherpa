//! Tradeoffs component - dominated alternatives, irrelevant objectives, tensions.

use serde::{Deserialize, Serialize};

use crate::domain::foundation::{ComponentId, ComponentStatus, ComponentType, Timestamp};

use super::{Component, ComponentBase, ComponentError};

/// An alternative that is dominated by another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DominatedAlternative {
    pub alternative_id: String,
    pub dominated_by_id: String,
    pub explanation: String,
}

/// An objective that doesn't distinguish between alternatives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrrelevantObjective {
    pub objective_id: String,
    /// Reason why this objective doesn't distinguish alternatives.
    pub reason: String,
}

/// A tension where an alternative has tradeoffs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tension {
    pub alternative_id: String,
    /// Objectives where this alternative excels.
    pub gains: Vec<String>,
    /// Objectives where this alternative suffers.
    pub losses: Vec<String>,
    /// How uncertainty affects this tension.
    pub uncertainty_impact: Option<String>,
}

/// Tradeoffs output structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TradeoffsOutput {
    pub dominated_alternatives: Vec<DominatedAlternative>,
    pub irrelevant_objectives: Vec<IrrelevantObjective>,
    pub tensions: Vec<Tension>,
}

/// The Tradeoffs component.
#[derive(Debug, Clone)]
pub struct Tradeoffs {
    base: ComponentBase,
    output: TradeoffsOutput,
}

impl Tradeoffs {
    /// Creates a new Tradeoffs component.
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(ComponentType::Tradeoffs),
            output: TradeoffsOutput::default(),
        }
    }

    /// Returns the output.
    pub fn output(&self) -> &TradeoffsOutput {
        &self.output
    }

    /// Sets the output.
    pub fn set_output(&mut self, output: TradeoffsOutput) {
        self.output = output;
        self.base.touch();
    }

    /// Adds a dominated alternative.
    pub fn add_dominated(&mut self, dominated: DominatedAlternative) {
        self.output.dominated_alternatives.push(dominated);
        self.base.touch();
    }

    /// Adds an irrelevant objective.
    pub fn add_irrelevant(&mut self, irrelevant: IrrelevantObjective) {
        self.output.irrelevant_objectives.push(irrelevant);
        self.base.touch();
    }

    /// Adds a tension.
    pub fn add_tension(&mut self, tension: Tension) {
        self.output.tensions.push(tension);
        self.base.touch();
    }

    /// Returns the count of viable alternatives (total - dominated).
    pub fn viable_alternative_count(&self, total_alternatives: usize) -> usize {
        total_alternatives.saturating_sub(self.output.dominated_alternatives.len())
    }

    /// Returns the count of relevant objectives (total - irrelevant).
    pub fn relevant_objective_count(&self, total_objectives: usize) -> usize {
        total_objectives.saturating_sub(self.output.irrelevant_objectives.len())
    }

    /// Returns true if an alternative is dominated.
    pub fn is_dominated(&self, alternative_id: &str) -> bool {
        self.output
            .dominated_alternatives
            .iter()
            .any(|d| d.alternative_id == alternative_id)
    }

    /// Returns true if an objective is irrelevant.
    pub fn is_irrelevant(&self, objective_id: &str) -> bool {
        self.output
            .irrelevant_objectives
            .iter()
            .any(|i| i.objective_id == objective_id)
    }
}

impl Default for Tradeoffs {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Tradeoffs {
    fn id(&self) -> ComponentId {
        self.base.id
    }

    fn component_type(&self) -> ComponentType {
        self.base.component_type
    }

    fn status(&self) -> ComponentStatus {
        self.base.status
    }

    fn created_at(&self) -> Timestamp {
        self.base.created_at
    }

    fn updated_at(&self) -> Timestamp {
        self.base.updated_at
    }

    fn start(&mut self) -> Result<(), ComponentError> {
        self.base.start()
    }

    fn complete(&mut self) -> Result<(), ComponentError> {
        self.base.complete()
    }

    fn mark_for_revision(&mut self, reason: String) -> Result<(), ComponentError> {
        self.base.mark_for_revision(reason)
    }

    fn output_as_value(&self) -> serde_json::Value {
        serde_json::to_value(&self.output).unwrap_or_default()
    }

    fn set_output_from_value(&mut self, value: serde_json::Value) -> Result<(), ComponentError> {
        self.output = serde_json::from_value(value)
            .map_err(|e| ComponentError::InvalidOutput(e.to_string()))?;
        self.base.touch();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tradeoffs_has_correct_component_type() {
        let to = Tradeoffs::new();
        assert_eq!(to.component_type(), ComponentType::Tradeoffs);
    }

    #[test]
    fn add_dominated_adds_to_list() {
        let mut to = Tradeoffs::new();
        let dominated = DominatedAlternative {
            alternative_id: "a2".to_string(),
            dominated_by_id: "a1".to_string(),
            explanation: "A1 is better on all objectives".to_string(),
        };
        to.add_dominated(dominated);

        assert_eq!(to.output().dominated_alternatives.len(), 1);
    }

    #[test]
    fn add_irrelevant_adds_to_list() {
        let mut to = Tradeoffs::new();
        let irrelevant = IrrelevantObjective {
            objective_id: "o3".to_string(),
            reason: "All alternatives score the same".to_string(),
        };
        to.add_irrelevant(irrelevant);

        assert_eq!(to.output().irrelevant_objectives.len(), 1);
    }

    #[test]
    fn add_tension_adds_to_list() {
        let mut to = Tradeoffs::new();
        let tension = Tension {
            alternative_id: "a1".to_string(),
            gains: vec!["o1".to_string(), "o2".to_string()],
            losses: vec!["o3".to_string()],
            uncertainty_impact: Some("Market changes could flip this".to_string()),
        };
        to.add_tension(tension);

        assert_eq!(to.output().tensions.len(), 1);
    }

    #[test]
    fn viable_alternative_count_subtracts_dominated() {
        let mut to = Tradeoffs::new();
        to.add_dominated(DominatedAlternative {
            alternative_id: "a2".to_string(),
            dominated_by_id: "a1".to_string(),
            explanation: "".to_string(),
        });

        assert_eq!(to.viable_alternative_count(5), 4);
    }

    #[test]
    fn relevant_objective_count_subtracts_irrelevant() {
        let mut to = Tradeoffs::new();
        to.add_irrelevant(IrrelevantObjective {
            objective_id: "o2".to_string(),
            reason: "".to_string(),
        });

        assert_eq!(to.relevant_objective_count(3), 2);
    }

    #[test]
    fn is_dominated_returns_true_for_dominated() {
        let mut to = Tradeoffs::new();
        to.add_dominated(DominatedAlternative {
            alternative_id: "a2".to_string(),
            dominated_by_id: "a1".to_string(),
            explanation: "".to_string(),
        });

        assert!(to.is_dominated("a2"));
        assert!(!to.is_dominated("a1"));
    }

    #[test]
    fn is_irrelevant_returns_true_for_irrelevant() {
        let mut to = Tradeoffs::new();
        to.add_irrelevant(IrrelevantObjective {
            objective_id: "o3".to_string(),
            reason: "".to_string(),
        });

        assert!(to.is_irrelevant("o3"));
        assert!(!to.is_irrelevant("o1"));
    }

    #[test]
    fn output_roundtrips_through_json() {
        let mut to = Tradeoffs::new();
        to.add_dominated(DominatedAlternative {
            alternative_id: "a2".to_string(),
            dominated_by_id: "a1".to_string(),
            explanation: "Test".to_string(),
        });
        to.add_tension(Tension {
            alternative_id: "a1".to_string(),
            gains: vec!["o1".to_string()],
            losses: vec!["o2".to_string()],
            uncertainty_impact: None,
        });

        let value = to.output_as_value();
        let mut to2 = Tradeoffs::new();
        to2.set_output_from_value(value).unwrap();

        assert_eq!(
            to.output().dominated_alternatives.len(),
            to2.output().dominated_alternatives.len()
        );
        assert_eq!(to.output().tensions.len(), to2.output().tensions.len());
    }
}
