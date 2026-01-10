//! IssueRaising component - categorizes initial thoughts from user's brain dump.

use serde::{Deserialize, Serialize};

use crate::domain::foundation::{ComponentId, ComponentStatus, ComponentType, Timestamp};

use super::{Component, ComponentBase, ComponentError};

/// Categorized outputs from user's initial brain dump.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IssueRaisingOutput {
    /// Things that need to be chosen/decided.
    pub potential_decisions: Vec<String>,

    /// Things that matter to the user.
    pub objectives: Vec<String>,

    /// Things that are unknown.
    pub uncertainties: Vec<String>,

    /// Process constraints, facts, stakeholders.
    pub considerations: Vec<String>,

    /// Whether user has validated the categorization.
    pub user_confirmed: bool,
}

/// The IssueRaising component.
#[derive(Debug, Clone)]
pub struct IssueRaising {
    base: ComponentBase,
    output: IssueRaisingOutput,
}

impl IssueRaising {
    /// Creates a new IssueRaising component.
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(ComponentType::IssueRaising),
            output: IssueRaisingOutput::default(),
        }
    }

    /// Reconstitutes an IssueRaising component from persisted data.
    pub(crate) fn reconstitute(base: ComponentBase, output: IssueRaisingOutput) -> Self {
        Self { base, output }
    }

    /// Returns the output.
    pub fn output(&self) -> &IssueRaisingOutput {
        &self.output
    }

    /// Sets the output.
    pub fn set_output(&mut self, output: IssueRaisingOutput) {
        self.output = output;
        self.base.touch();
    }

    /// Adds a potential decision.
    pub fn add_potential_decision(&mut self, decision: String) {
        self.output.potential_decisions.push(decision);
        self.base.touch();
    }

    /// Adds an objective.
    pub fn add_objective(&mut self, objective: String) {
        self.output.objectives.push(objective);
        self.base.touch();
    }

    /// Adds an uncertainty.
    pub fn add_uncertainty(&mut self, uncertainty: String) {
        self.output.uncertainties.push(uncertainty);
        self.base.touch();
    }

    /// Adds a consideration.
    pub fn add_consideration(&mut self, consideration: String) {
        self.output.considerations.push(consideration);
        self.base.touch();
    }

    /// Marks the categorization as confirmed by user.
    pub fn confirm(&mut self) {
        self.output.user_confirmed = true;
        self.base.touch();
    }

    /// Returns true if user has confirmed the categorization.
    pub fn is_confirmed(&self) -> bool {
        self.output.user_confirmed
    }
}

impl Default for IssueRaising {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for IssueRaising {
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
    use serde_json::json;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn issue_raising_starts_as_not_started() {
        let ir = IssueRaising::new();
        assert_eq!(ir.status(), ComponentStatus::NotStarted);
    }

    #[test]
    fn issue_raising_has_correct_component_type() {
        let ir = IssueRaising::new();
        assert_eq!(ir.component_type(), ComponentType::IssueRaising);
    }

    #[test]
    fn issue_raising_output_starts_empty() {
        let ir = IssueRaising::new();
        assert!(ir.output().potential_decisions.is_empty());
        assert!(ir.output().objectives.is_empty());
        assert!(ir.output().uncertainties.is_empty());
        assert!(ir.output().considerations.is_empty());
        assert!(!ir.output().user_confirmed);
    }

    #[test]
    fn add_potential_decision_updates_output() {
        let mut ir = IssueRaising::new();
        ir.add_potential_decision("Should I change jobs?".to_string());

        assert_eq!(ir.output().potential_decisions.len(), 1);
        assert_eq!(ir.output().potential_decisions[0], "Should I change jobs?");
    }

    #[test]
    fn add_potential_decision_updates_timestamp() {
        let mut ir = IssueRaising::new();
        let initial = ir.updated_at();
        sleep(Duration::from_millis(10));
        ir.add_potential_decision("Test".to_string());

        assert!(ir.updated_at().is_after(&initial));
    }

    #[test]
    fn add_objective_updates_output() {
        let mut ir = IssueRaising::new();
        ir.add_objective("Financial stability".to_string());

        assert_eq!(ir.output().objectives.len(), 1);
    }

    #[test]
    fn add_uncertainty_updates_output() {
        let mut ir = IssueRaising::new();
        ir.add_uncertainty("How will the market change?".to_string());

        assert_eq!(ir.output().uncertainties.len(), 1);
    }

    #[test]
    fn add_consideration_updates_output() {
        let mut ir = IssueRaising::new();
        ir.add_consideration("My family depends on my income".to_string());

        assert_eq!(ir.output().considerations.len(), 1);
    }

    #[test]
    fn confirm_sets_user_confirmed_flag() {
        let mut ir = IssueRaising::new();
        assert!(!ir.is_confirmed());
        ir.confirm();
        assert!(ir.is_confirmed());
    }

    #[test]
    fn set_output_replaces_entire_output() {
        let mut ir = IssueRaising::new();
        let new_output = IssueRaisingOutput {
            potential_decisions: vec!["Decision 1".to_string()],
            objectives: vec!["Objective 1".to_string()],
            uncertainties: vec!["Uncertainty 1".to_string()],
            considerations: vec!["Consideration 1".to_string()],
            user_confirmed: true,
        };
        ir.set_output(new_output);

        assert_eq!(ir.output().potential_decisions.len(), 1);
        assert!(ir.is_confirmed());
    }

    #[test]
    fn output_as_value_returns_json() {
        let mut ir = IssueRaising::new();
        ir.add_potential_decision("Test decision".to_string());

        let value = ir.output_as_value();
        assert!(value.is_object());
        assert!(value["potential_decisions"].is_array());
    }

    #[test]
    fn set_output_from_value_parses_json() {
        let mut ir = IssueRaising::new();
        let value = json!({
            "potential_decisions": ["Test"],
            "objectives": [],
            "uncertainties": [],
            "considerations": [],
            "user_confirmed": true
        });

        assert!(ir.set_output_from_value(value).is_ok());
        assert_eq!(ir.output().potential_decisions.len(), 1);
        assert!(ir.is_confirmed());
    }

    #[test]
    fn set_output_from_invalid_value_returns_error() {
        let mut ir = IssueRaising::new();
        let value = json!({"invalid": "structure"});

        let result = ir.set_output_from_value(value);
        assert!(result.is_err());
    }

    #[test]
    fn output_roundtrips_through_json() {
        let mut ir = IssueRaising::new();
        ir.add_potential_decision("Decision 1".to_string());
        ir.add_objective("Objective 1".to_string());
        ir.confirm();

        let value = ir.output_as_value();
        let mut ir2 = IssueRaising::new();
        ir2.set_output_from_value(value).unwrap();

        assert_eq!(ir.output().potential_decisions, ir2.output().potential_decisions);
        assert_eq!(ir.output().objectives, ir2.output().objectives);
        assert_eq!(ir.output().user_confirmed, ir2.output().user_confirmed);
    }
}
