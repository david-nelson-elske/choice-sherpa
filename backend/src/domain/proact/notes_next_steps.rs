//! NotesNextSteps component - remaining uncertainties, actions, and wrap-up.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::foundation::{ComponentId, ComponentStatus, ComponentType, Timestamp};

use super::{Component, ComponentBase, ComponentError};

/// A planned action to take.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedAction {
    pub description: String,
    pub due_date: Option<DateTime<Utc>>,
    pub owner: Option<String>,
}

/// NotesNextSteps output structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotesNextStepsOutput {
    pub remaining_uncertainties: Vec<String>,
    pub open_questions: Vec<String>,
    pub planned_actions: Vec<PlannedAction>,
    /// Affirmation if DQ is 100%.
    pub affirmation: Option<String>,
    /// Further analysis paths if DQ < 100%.
    pub further_analysis_paths: Vec<String>,
}

/// The NotesNextSteps component.
#[derive(Debug, Clone)]
pub struct NotesNextSteps {
    base: ComponentBase,
    output: NotesNextStepsOutput,
}

impl NotesNextSteps {
    /// Creates a new NotesNextSteps component.
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(ComponentType::NotesNextSteps),
            output: NotesNextStepsOutput::default(),
        }
    }

    /// Returns the output.
    pub fn output(&self) -> &NotesNextStepsOutput {
        &self.output
    }

    /// Sets the output.
    pub fn set_output(&mut self, output: NotesNextStepsOutput) {
        self.output = output;
        self.base.touch();
    }

    /// Adds a planned action.
    pub fn add_action(&mut self, action: PlannedAction) {
        self.output.planned_actions.push(action);
        self.base.touch();
    }

    /// Adds an open question.
    pub fn add_open_question(&mut self, question: String) {
        self.output.open_questions.push(question);
        self.base.touch();
    }

    /// Adds a remaining uncertainty.
    pub fn add_remaining_uncertainty(&mut self, uncertainty: String) {
        self.output.remaining_uncertainties.push(uncertainty);
        self.base.touch();
    }

    /// Adds a further analysis path.
    pub fn add_further_analysis_path(&mut self, path: String) {
        self.output.further_analysis_paths.push(path);
        self.base.touch();
    }

    /// Sets the affirmation.
    pub fn set_affirmation(&mut self, affirmation: String) {
        self.output.affirmation = Some(affirmation);
        self.base.touch();
    }

    /// Returns the count of planned actions.
    pub fn action_count(&self) -> usize {
        self.output.planned_actions.len()
    }

    /// Returns true if there's an affirmation (DQ was 100%).
    pub fn has_affirmation(&self) -> bool {
        self.output.affirmation.is_some()
    }

    /// Returns actions with due dates.
    pub fn actions_with_due_dates(&self) -> Vec<&PlannedAction> {
        self.output
            .planned_actions
            .iter()
            .filter(|a| a.due_date.is_some())
            .collect()
    }
}

impl Default for NotesNextSteps {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for NotesNextSteps {
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
    fn notes_next_steps_has_correct_component_type() {
        let nns = NotesNextSteps::new();
        assert_eq!(nns.component_type(), ComponentType::NotesNextSteps);
    }

    #[test]
    fn add_action_increases_count() {
        let mut nns = NotesNextSteps::new();
        assert_eq!(nns.action_count(), 0);

        let action = PlannedAction {
            description: "Schedule follow-up meeting".to_string(),
            due_date: None,
            owner: Some("John".to_string()),
        };
        nns.add_action(action);

        assert_eq!(nns.action_count(), 1);
    }

    #[test]
    fn add_open_question_adds_to_list() {
        let mut nns = NotesNextSteps::new();
        nns.add_open_question("What is the timeline?".to_string());

        assert_eq!(nns.output().open_questions.len(), 1);
    }

    #[test]
    fn add_remaining_uncertainty_adds_to_list() {
        let mut nns = NotesNextSteps::new();
        nns.add_remaining_uncertainty("Market conditions".to_string());

        assert_eq!(nns.output().remaining_uncertainties.len(), 1);
    }

    #[test]
    fn set_affirmation_updates_output() {
        let mut nns = NotesNextSteps::new();
        assert!(!nns.has_affirmation());

        nns.set_affirmation("This is a well-made decision.".to_string());

        assert!(nns.has_affirmation());
        assert!(nns.output().affirmation.is_some());
    }

    #[test]
    fn actions_with_due_dates_filters_correctly() {
        let mut nns = NotesNextSteps::new();
        nns.add_action(PlannedAction {
            description: "Action 1".to_string(),
            due_date: Some(Utc::now()),
            owner: None,
        });
        nns.add_action(PlannedAction {
            description: "Action 2".to_string(),
            due_date: None,
            owner: None,
        });
        nns.add_action(PlannedAction {
            description: "Action 3".to_string(),
            due_date: Some(Utc::now()),
            owner: None,
        });

        let with_dates = nns.actions_with_due_dates();
        assert_eq!(with_dates.len(), 2);
    }

    #[test]
    fn output_roundtrips_through_json() {
        let mut nns = NotesNextSteps::new();
        nns.add_action(PlannedAction {
            description: "Test action".to_string(),
            due_date: None,
            owner: Some("Test".to_string()),
        });
        nns.add_open_question("Test question".to_string());
        nns.set_affirmation("Good decision".to_string());

        let value = nns.output_as_value();
        let mut nns2 = NotesNextSteps::new();
        nns2.set_output_from_value(value).unwrap();

        assert_eq!(nns.action_count(), nns2.action_count());
        assert_eq!(nns.has_affirmation(), nns2.has_affirmation());
    }
}
