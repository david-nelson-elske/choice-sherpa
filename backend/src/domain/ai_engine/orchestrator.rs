//! Orchestrator - PrOACT Flow Management
//!
//! Manages the flow through PrOACT components within a cycle.
//! Pure domain logic—no AI provider knowledge.

use std::collections::HashMap;

use crate::domain::foundation::{ComponentType, CycleId};

use super::{
    conversation_state::ConversationState, errors::OrchestratorError, values::{StepContext, StepSummary, UserIntent}
};

/// Manages PrOACT flow within a decision cycle
#[derive(Debug, Clone)]
pub struct Orchestrator {
    cycle_id: CycleId,
    current_step: ComponentType,
    completed_steps: HashMap<ComponentType, StepSummary>,
    state: ConversationState,
}

impl Orchestrator {
    /// Create a new orchestrator for a cycle
    pub fn new(cycle_id: CycleId, state: ConversationState) -> Self {
        let current_step = state.current_step;
        let completed_steps = HashMap::new();

        Self {
            cycle_id,
            current_step,
            completed_steps,
            state,
        }
    }

    /// Resume from persisted state
    pub fn from_state(state: ConversationState) -> Result<Self, OrchestratorError> {
        let cycle_id = state.cycle_id;
        let current_step = state.current_step;

        // Build completed_steps map from state
        let completed_steps = state
            .step_states
            .iter()
            .filter_map(|(component, step_state)| {
                if let (Some(summary_text), Some(completed_at)) =
                    (&step_state.summary, &step_state.completed_at)
                {
                    Some((
                        *component,
                        StepSummary {
                            component: *component,
                            summary: summary_text.clone(),
                            key_outputs: step_state.key_outputs.clone(),
                            conflicts: Vec::new(), // TODO: Extract from state if needed
                            completed_at: *completed_at,
                        },
                    ))
                } else {
                    None
                }
            })
            .collect();

        Ok(Self {
            cycle_id,
            current_step,
            completed_steps,
            state,
        })
    }

    /// Route user intent to appropriate step
    pub fn route(&self, intent: UserIntent) -> Result<ComponentType, OrchestratorError> {
        match intent {
            UserIntent::Continue => Ok(self.current_step),
            UserIntent::Navigate(target) => {
                if self.can_transition(target) {
                    Ok(target)
                } else {
                    Err(OrchestratorError::InvalidTransition {
                        from: self.current_step,
                        to: target,
                    })
                }
            }
            UserIntent::Branch => {
                // Branching stays on current step but in a new cycle context
                Ok(self.current_step)
            }
            UserIntent::Summarize => Ok(self.current_step),
            UserIntent::Complete => {
                // Try to move to next step
                self.next_step()
                    .ok_or(OrchestratorError::CycleCompleted)
            }
        }
    }

    /// Check if transition to a new step is valid
    pub fn can_transition(&self, to: ComponentType) -> bool {
        // Can always go backwards to revisit steps
        if self.is_before_current(to) {
            return true;
        }

        // Can go forward if all previous steps are completed
        if self.is_after_current(to) {
            return self.all_steps_before_completed(to);
        }

        // Can transition to current step
        to == self.current_step
    }

    /// Transition to a new step
    pub fn transition_to(&mut self, step: ComponentType) -> Result<(), OrchestratorError> {
        if !self.can_transition(step) {
            return Err(OrchestratorError::InvalidTransition {
                from: self.current_step,
                to: step,
            });
        }

        self.state.transition_to(step);
        self.current_step = step;

        Ok(())
    }

    /// Record completion of current step
    pub fn record_completion(&mut self, summary: StepSummary) -> Result<(), OrchestratorError> {
        if summary.component != self.current_step {
            return Err(OrchestratorError::InvalidState(format!(
                "Cannot complete {:?} while on {:?}",
                summary.component, self.current_step
            )));
        }

        self.state
            .complete_current_step(summary.summary.clone(), summary.key_outputs.clone());
        self.completed_steps.insert(summary.component, summary);

        Ok(())
    }

    /// Get context needed for a step agent
    pub fn context_for_step(&self, step: ComponentType) -> StepContext {
        let prior_summaries: Vec<StepSummary> = self
            .proact_order()
            .iter()
            .filter(|&&c| c != step && self.completed_steps.contains_key(&c))
            .filter_map(|c| self.completed_steps.get(c).cloned())
            .collect();

        let relevant_outputs = HashMap::new(); // TODO: Extract from structured outputs if needed

        StepContext {
            component: step,
            prior_summaries,
            relevant_outputs,
        }
    }

    /// Export current state for persistence
    pub fn to_state(&self) -> &ConversationState {
        &self.state
    }

    /// Get the cycle ID
    pub fn cycle_id(&self) -> CycleId {
        self.cycle_id
    }

    /// Get the current step
    pub fn current_step(&self) -> ComponentType {
        self.current_step
    }

    /// Get completed steps
    pub fn completed_steps(&self) -> &HashMap<ComponentType, StepSummary> {
        &self.completed_steps
    }

    /// Check if a step is completed
    pub fn is_step_completed(&self, component: ComponentType) -> bool {
        self.completed_steps.contains_key(&component)
    }

    /// Get the next step in the PrOACT sequence
    fn next_step(&self) -> Option<ComponentType> {
        let order = self.proact_order();
        let current_index = order.iter().position(|&c| c == self.current_step)?;

        if current_index + 1 < order.len() {
            Some(order[current_index + 1])
        } else {
            None // Already at the end
        }
    }

    /// Check if a step comes before the current step
    fn is_before_current(&self, step: ComponentType) -> bool {
        let order = self.proact_order();
        let current_pos = order.iter().position(|&c| c == self.current_step).unwrap();
        let step_pos = order.iter().position(|&c| c == step).unwrap();

        step_pos < current_pos
    }

    /// Check if a step comes after the current step
    fn is_after_current(&self, step: ComponentType) -> bool {
        let order = self.proact_order();
        let current_pos = order.iter().position(|&c| c == self.current_step).unwrap();
        let step_pos = order.iter().position(|&c| c == step).unwrap();

        step_pos > current_pos
    }

    /// Check if all steps before a given step are completed
    fn all_steps_before_completed(&self, step: ComponentType) -> bool {
        let order = self.proact_order();
        let step_pos = order.iter().position(|&c| c == step).unwrap();

        order[..step_pos]
            .iter()
            .all(|c| self.completed_steps.contains_key(c))
    }

    /// Get the standard PrOACT order
    fn proact_order(&self) -> Vec<ComponentType> {
        vec![
            ComponentType::IssueRaising,
            ComponentType::ProblemFrame,
            ComponentType::Objectives,
            ComponentType::Alternatives,
            ComponentType::Consequences,
            ComponentType::Tradeoffs,
            ComponentType::Recommendation,
            ComponentType::DecisionQuality,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::SessionId;
    use chrono::Utc;
    use std::str::FromStr;

    fn test_cycle_id() -> CycleId {
        CycleId::new()
    }

    fn test_session_id() -> SessionId {
        SessionId::new()
    }

    fn new_test_orchestrator() -> Orchestrator {
        let cycle_id = test_cycle_id();
        let state =
            ConversationState::new(cycle_id, test_session_id(), ComponentType::IssueRaising);
        Orchestrator::new(cycle_id, state)
    }

    #[test]
    fn test_orchestrator_new_creates_with_initial_step() {
        let cycle_id = test_cycle_id();
        let state = ConversationState::new(cycle_id, test_session_id(), ComponentType::IssueRaising);
        let orchestrator = Orchestrator::new(cycle_id, state);

        assert_eq!(orchestrator.current_step(), ComponentType::IssueRaising);
        assert_eq!(orchestrator.cycle_id(), cycle_id);
        assert!(orchestrator.completed_steps().is_empty());
    }

    #[test]
    fn test_orchestrator_from_state_restores_correctly() {
        let mut state =
            ConversationState::new(test_cycle_id(), test_session_id(), ComponentType::IssueRaising);
        state.complete_current_step("Test summary".to_string(), vec!["output1".to_string()]);
        state.transition_to(ComponentType::ProblemFrame);

        let orchestrator = Orchestrator::from_state(state).unwrap();

        assert_eq!(orchestrator.current_step(), ComponentType::ProblemFrame);
        assert!(orchestrator.is_step_completed(ComponentType::IssueRaising));
    }

    #[test]
    fn test_orchestrator_route_continue_returns_current() {
        let orchestrator = new_test_orchestrator();

        let target = orchestrator.route(UserIntent::Continue).unwrap();

        assert_eq!(target, ComponentType::IssueRaising);
    }

    #[test]
    fn test_orchestrator_route_navigate_returns_target() {
        let mut orchestrator = new_test_orchestrator();

        // Complete first step
        orchestrator
            .record_completion(StepSummary::new(
                ComponentType::IssueRaising,
                "Done".to_string(),
                vec![],
                vec![],
            ))
            .unwrap();

        let target = orchestrator
            .route(UserIntent::Navigate(ComponentType::ProblemFrame))
            .unwrap();

        assert_eq!(target, ComponentType::ProblemFrame);
    }

    #[test]
    fn test_orchestrator_can_transition_valid_progression() {
        let mut orchestrator = new_test_orchestrator();

        // Complete first step
        orchestrator
            .record_completion(StepSummary::new(
                ComponentType::IssueRaising,
                "Done".to_string(),
                vec![],
                vec![],
            ))
            .unwrap();

        // Can transition forward
        assert!(orchestrator.can_transition(ComponentType::ProblemFrame));
    }

    #[test]
    fn test_orchestrator_can_transition_invalid_skip() {
        let orchestrator = new_test_orchestrator();

        // Cannot skip steps
        assert!(!orchestrator.can_transition(ComponentType::Alternatives));
    }

    #[test]
    fn test_orchestrator_can_transition_backward() {
        let mut orchestrator = new_test_orchestrator();

        // Move forward
        orchestrator
            .record_completion(StepSummary::new(
                ComponentType::IssueRaising,
                "Done".to_string(),
                vec![],
                vec![],
            ))
            .unwrap();
        orchestrator
            .transition_to(ComponentType::ProblemFrame)
            .unwrap();

        // Can always go backward
        assert!(orchestrator.can_transition(ComponentType::IssueRaising));
    }

    #[test]
    fn test_orchestrator_transition_to_updates_current() {
        let mut orchestrator = new_test_orchestrator();

        orchestrator
            .record_completion(StepSummary::new(
                ComponentType::IssueRaising,
                "Done".to_string(),
                vec![],
                vec![],
            ))
            .unwrap();
        orchestrator
            .transition_to(ComponentType::ProblemFrame)
            .unwrap();

        assert_eq!(orchestrator.current_step(), ComponentType::ProblemFrame);
    }

    #[test]
    fn test_orchestrator_transition_to_invalid_returns_error() {
        let mut orchestrator = new_test_orchestrator();

        let result = orchestrator.transition_to(ComponentType::Consequences);

        assert!(matches!(result, Err(OrchestratorError::InvalidTransition { .. })));
    }

    #[test]
    fn test_orchestrator_record_completion_advances_step() {
        let mut orchestrator = new_test_orchestrator();

        orchestrator
            .record_completion(StepSummary::new(
                ComponentType::IssueRaising,
                "Summary".to_string(),
                vec!["output".to_string()],
                vec![],
            ))
            .unwrap();

        assert!(orchestrator.is_step_completed(ComponentType::IssueRaising));
        assert_eq!(orchestrator.completed_steps().len(), 1);
    }

    #[test]
    fn test_orchestrator_record_completion_stores_summary() {
        let mut orchestrator = new_test_orchestrator();

        let summary = StepSummary::new(
            ComponentType::IssueRaising,
            "Test summary".to_string(),
            vec!["key output".to_string()],
            vec![],
        );

        orchestrator.record_completion(summary.clone()).unwrap();

        let stored = orchestrator
            .completed_steps()
            .get(&ComponentType::IssueRaising)
            .unwrap();
        assert_eq!(stored.summary, "Test summary");
    }

    #[test]
    fn test_orchestrator_record_completion_wrong_step_errors() {
        let mut orchestrator = new_test_orchestrator();

        // Try to complete a different step than current
        let result = orchestrator.record_completion(StepSummary::new(
            ComponentType::ProblemFrame,
            "Wrong step".to_string(),
            vec![],
            vec![],
        ));

        assert!(matches!(result, Err(OrchestratorError::InvalidState(_))));
    }

    #[test]
    fn test_orchestrator_context_for_step_includes_prior() {
        let mut orchestrator = new_test_orchestrator();

        // Complete first step
        orchestrator
            .record_completion(StepSummary::new(
                ComponentType::IssueRaising,
                "Issue summary".to_string(),
                vec![],
                vec![],
            ))
            .unwrap();

        // Complete second step
        orchestrator
            .transition_to(ComponentType::ProblemFrame)
            .unwrap();
        orchestrator
            .record_completion(StepSummary::new(
                ComponentType::ProblemFrame,
                "Frame summary".to_string(),
                vec![],
                vec![],
            ))
            .unwrap();

        // Get context for third step
        orchestrator
            .transition_to(ComponentType::Objectives)
            .unwrap();
        let context = orchestrator.context_for_step(ComponentType::Objectives);

        assert_eq!(context.component, ComponentType::Objectives);
        assert_eq!(context.prior_summaries.len(), 2);
    }

    #[test]
    fn test_orchestrator_to_state_exports_correctly() {
        let cycle_id = test_cycle_id();
        let state = ConversationState::new(cycle_id, test_session_id(), ComponentType::IssueRaising);
        let orchestrator = Orchestrator::new(cycle_id, state);

        let exported_state = orchestrator.to_state();

        assert_eq!(exported_state.cycle_id, cycle_id);
        assert_eq!(exported_state.current_step, ComponentType::IssueRaising);
    }

    #[test]
    fn test_orchestrator_route_complete_at_end_errors() {
        let mut orchestrator = new_test_orchestrator();

        // Move through all steps to the end
        let steps = vec![
            ComponentType::IssueRaising,
            ComponentType::ProblemFrame,
            ComponentType::Objectives,
            ComponentType::Alternatives,
            ComponentType::Consequences,
            ComponentType::Tradeoffs,
            ComponentType::Recommendation,
            ComponentType::DecisionQuality,
        ];

        for (i, step) in steps.iter().enumerate() {
            orchestrator
                .record_completion(StepSummary::new(*step, "Done".to_string(), vec![], vec![]))
                .unwrap();

            // Transition to next step if not at the end
            if i < steps.len() - 1 {
                orchestrator.transition_to(steps[i + 1]).unwrap();
            }
        }

        // Try to complete when already at the end
        let result = orchestrator.route(UserIntent::Complete);

        assert!(matches!(result, Err(OrchestratorError::CycleCompleted)));
    }
}
