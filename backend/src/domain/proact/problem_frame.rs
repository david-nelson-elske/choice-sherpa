//! ProblemFrame component - structured problem framing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::foundation::{ComponentId, ComponentStatus, ComponentType, Timestamp};

use super::{Component, ComponentBase, ComponentError};

/// A decision linked to the focal decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedDecision {
    pub description: String,
    /// Relationship type: "enables", "constrains", "depends_on"
    pub relationship: String,
}

/// A constraint on the decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    /// Type: "legal", "financial", "political", "technical"
    pub constraint_type: String,
    pub description: String,
}

/// An affected party/stakeholder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Party {
    pub id: String,
    pub name: String,
    pub role: String,
    /// What this party cares about.
    pub objectives: Vec<String>,
}

/// Hierarchical organization of decisions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionHierarchy {
    /// Decisions already made.
    pub already_made: Vec<String>,
    /// The focal decisions being analyzed.
    pub focal_decisions: Vec<String>,
    /// Decisions to be deferred.
    pub deferred: Vec<String>,
}

/// Structured problem framing output.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProblemFrameOutput {
    /// Who has authority to make this decision.
    pub decision_maker: Option<String>,

    /// What specifically is being decided.
    pub focal_decision: Option<String>,

    /// What success looks like.
    pub ultimate_aim: Option<String>,

    /// When must this be decided by.
    pub temporal_constraint: Option<DateTime<Utc>>,

    /// Geographic or organizational scope.
    pub spatial_scope: Option<String>,

    /// Future choices that depend on this decision.
    pub linked_decisions: Vec<LinkedDecision>,

    /// Legal, financial, political, or technical constraints.
    pub constraints: Vec<Constraint>,

    /// Stakeholders affected by this decision.
    pub affected_parties: Vec<Party>,

    /// Experts who should be consulted.
    pub expert_sources: Vec<String>,

    /// Hierarchical organization of decisions.
    pub decision_hierarchy: Option<DecisionHierarchy>,

    /// Synthesized decision statement.
    pub decision_statement: Option<String>,
}

/// The ProblemFrame component.
#[derive(Debug, Clone)]
pub struct ProblemFrame {
    base: ComponentBase,
    output: ProblemFrameOutput,
}

impl ProblemFrame {
    /// Creates a new ProblemFrame component.
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(ComponentType::ProblemFrame),
            output: ProblemFrameOutput::default(),
        }
    }

    /// Reconstitutes a ProblemFrame component from persisted data.
    pub(crate) fn reconstitute(base: ComponentBase, output: ProblemFrameOutput) -> Self {
        Self { base, output }
    }

    /// Returns the output.
    pub fn output(&self) -> &ProblemFrameOutput {
        &self.output
    }

    /// Sets the output.
    pub fn set_output(&mut self, output: ProblemFrameOutput) {
        self.output = output;
        self.base.touch();
    }

    /// Sets the decision statement.
    pub fn set_decision_statement(&mut self, statement: String) {
        self.output.decision_statement = Some(statement);
        self.base.touch();
    }

    /// Sets the focal decision.
    pub fn set_focal_decision(&mut self, decision: String) {
        self.output.focal_decision = Some(decision);
        self.base.touch();
    }

    /// Sets the decision maker.
    pub fn set_decision_maker(&mut self, maker: String) {
        self.output.decision_maker = Some(maker);
        self.base.touch();
    }

    /// Adds an affected party.
    pub fn add_party(&mut self, party: Party) {
        self.output.affected_parties.push(party);
        self.base.touch();
    }

    /// Adds a constraint.
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.output.constraints.push(constraint);
        self.base.touch();
    }

    /// Adds a linked decision.
    pub fn add_linked_decision(&mut self, linked: LinkedDecision) {
        self.output.linked_decisions.push(linked);
        self.base.touch();
    }

    /// Adds an expert source.
    pub fn add_expert_source(&mut self, source: String) {
        self.output.expert_sources.push(source);
        self.base.touch();
    }
}

impl Default for ProblemFrame {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for ProblemFrame {
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
    fn problem_frame_has_correct_component_type() {
        let pf = ProblemFrame::new();
        assert_eq!(pf.component_type(), ComponentType::ProblemFrame);
    }

    #[test]
    fn set_decision_statement_updates_output() {
        let mut pf = ProblemFrame::new();
        pf.set_decision_statement("Decide whether to expand".to_string());

        assert_eq!(
            pf.output().decision_statement,
            Some("Decide whether to expand".to_string())
        );
    }

    #[test]
    fn set_focal_decision_updates_output() {
        let mut pf = ProblemFrame::new();
        pf.set_focal_decision("Should we hire more engineers?".to_string());

        assert_eq!(
            pf.output().focal_decision,
            Some("Should we hire more engineers?".to_string())
        );
    }

    #[test]
    fn add_party_adds_to_list() {
        let mut pf = ProblemFrame::new();
        let party = Party {
            id: "p1".to_string(),
            name: "Engineering Team".to_string(),
            role: "Implementer".to_string(),
            objectives: vec!["Minimize workload".to_string()],
        };
        pf.add_party(party);

        assert_eq!(pf.output().affected_parties.len(), 1);
        assert_eq!(pf.output().affected_parties[0].name, "Engineering Team");
    }

    #[test]
    fn add_constraint_adds_to_list() {
        let mut pf = ProblemFrame::new();
        let constraint = Constraint {
            constraint_type: "financial".to_string(),
            description: "Budget limited to $100k".to_string(),
        };
        pf.add_constraint(constraint);

        assert_eq!(pf.output().constraints.len(), 1);
        assert_eq!(pf.output().constraints[0].constraint_type, "financial");
    }

    #[test]
    fn add_linked_decision_adds_to_list() {
        let mut pf = ProblemFrame::new();
        let linked = LinkedDecision {
            description: "Office location".to_string(),
            relationship: "depends_on".to_string(),
        };
        pf.add_linked_decision(linked);

        assert_eq!(pf.output().linked_decisions.len(), 1);
    }

    #[test]
    fn output_roundtrips_through_json() {
        let mut pf = ProblemFrame::new();
        pf.set_decision_maker("CEO".to_string());
        pf.set_focal_decision("Market expansion".to_string());

        let value = pf.output_as_value();
        let mut pf2 = ProblemFrame::new();
        pf2.set_output_from_value(value).unwrap();

        assert_eq!(pf.output().decision_maker, pf2.output().decision_maker);
        assert_eq!(pf.output().focal_decision, pf2.output().focal_decision);
    }
}
