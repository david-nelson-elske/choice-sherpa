//! ComponentVariant - sum type for all PrOACT component types.

use crate::domain::foundation::{ComponentId, ComponentStatus, ComponentType, Timestamp};

use super::{
    Alternatives, Component, ComponentError, Consequences, DecisionQuality, IssueRaising,
    NotesNextSteps, Objectives, ProblemFrame, Recommendation, Tradeoffs,
};

/// Sum type for all component types.
#[derive(Debug, Clone)]
pub enum ComponentVariant {
    IssueRaising(IssueRaising),
    ProblemFrame(ProblemFrame),
    Objectives(Objectives),
    Alternatives(Alternatives),
    Consequences(Consequences),
    Tradeoffs(Tradeoffs),
    Recommendation(Recommendation),
    DecisionQuality(DecisionQuality),
    NotesNextSteps(NotesNextSteps),
}

impl ComponentVariant {
    /// Creates a new component of the specified type.
    pub fn new(component_type: ComponentType) -> Self {
        match component_type {
            ComponentType::IssueRaising => ComponentVariant::IssueRaising(IssueRaising::new()),
            ComponentType::ProblemFrame => ComponentVariant::ProblemFrame(ProblemFrame::new()),
            ComponentType::Objectives => ComponentVariant::Objectives(Objectives::new()),
            ComponentType::Alternatives => ComponentVariant::Alternatives(Alternatives::new()),
            ComponentType::Consequences => ComponentVariant::Consequences(Consequences::new()),
            ComponentType::Tradeoffs => ComponentVariant::Tradeoffs(Tradeoffs::new()),
            ComponentType::Recommendation => {
                ComponentVariant::Recommendation(Recommendation::new())
            }
            ComponentType::DecisionQuality => {
                ComponentVariant::DecisionQuality(DecisionQuality::new())
            }
            ComponentType::NotesNextSteps => {
                ComponentVariant::NotesNextSteps(NotesNextSteps::new())
            }
        }
    }

    /// Returns the component ID.
    pub fn id(&self) -> ComponentId {
        match self {
            ComponentVariant::IssueRaising(c) => c.id(),
            ComponentVariant::ProblemFrame(c) => c.id(),
            ComponentVariant::Objectives(c) => c.id(),
            ComponentVariant::Alternatives(c) => c.id(),
            ComponentVariant::Consequences(c) => c.id(),
            ComponentVariant::Tradeoffs(c) => c.id(),
            ComponentVariant::Recommendation(c) => c.id(),
            ComponentVariant::DecisionQuality(c) => c.id(),
            ComponentVariant::NotesNextSteps(c) => c.id(),
        }
    }

    /// Returns the component type.
    pub fn component_type(&self) -> ComponentType {
        match self {
            ComponentVariant::IssueRaising(_) => ComponentType::IssueRaising,
            ComponentVariant::ProblemFrame(_) => ComponentType::ProblemFrame,
            ComponentVariant::Objectives(_) => ComponentType::Objectives,
            ComponentVariant::Alternatives(_) => ComponentType::Alternatives,
            ComponentVariant::Consequences(_) => ComponentType::Consequences,
            ComponentVariant::Tradeoffs(_) => ComponentType::Tradeoffs,
            ComponentVariant::Recommendation(_) => ComponentType::Recommendation,
            ComponentVariant::DecisionQuality(_) => ComponentType::DecisionQuality,
            ComponentVariant::NotesNextSteps(_) => ComponentType::NotesNextSteps,
        }
    }

    /// Returns the component status.
    pub fn status(&self) -> ComponentStatus {
        match self {
            ComponentVariant::IssueRaising(c) => c.status(),
            ComponentVariant::ProblemFrame(c) => c.status(),
            ComponentVariant::Objectives(c) => c.status(),
            ComponentVariant::Alternatives(c) => c.status(),
            ComponentVariant::Consequences(c) => c.status(),
            ComponentVariant::Tradeoffs(c) => c.status(),
            ComponentVariant::Recommendation(c) => c.status(),
            ComponentVariant::DecisionQuality(c) => c.status(),
            ComponentVariant::NotesNextSteps(c) => c.status(),
        }
    }

    /// Returns when this component was created.
    pub fn created_at(&self) -> Timestamp {
        match self {
            ComponentVariant::IssueRaising(c) => c.created_at(),
            ComponentVariant::ProblemFrame(c) => c.created_at(),
            ComponentVariant::Objectives(c) => c.created_at(),
            ComponentVariant::Alternatives(c) => c.created_at(),
            ComponentVariant::Consequences(c) => c.created_at(),
            ComponentVariant::Tradeoffs(c) => c.created_at(),
            ComponentVariant::Recommendation(c) => c.created_at(),
            ComponentVariant::DecisionQuality(c) => c.created_at(),
            ComponentVariant::NotesNextSteps(c) => c.created_at(),
        }
    }

    /// Returns when this component was last updated.
    pub fn updated_at(&self) -> Timestamp {
        match self {
            ComponentVariant::IssueRaising(c) => c.updated_at(),
            ComponentVariant::ProblemFrame(c) => c.updated_at(),
            ComponentVariant::Objectives(c) => c.updated_at(),
            ComponentVariant::Alternatives(c) => c.updated_at(),
            ComponentVariant::Consequences(c) => c.updated_at(),
            ComponentVariant::Tradeoffs(c) => c.updated_at(),
            ComponentVariant::Recommendation(c) => c.updated_at(),
            ComponentVariant::DecisionQuality(c) => c.updated_at(),
            ComponentVariant::NotesNextSteps(c) => c.updated_at(),
        }
    }

    /// Starts work on this component.
    pub fn start(&mut self) -> Result<(), ComponentError> {
        match self {
            ComponentVariant::IssueRaising(c) => c.start(),
            ComponentVariant::ProblemFrame(c) => c.start(),
            ComponentVariant::Objectives(c) => c.start(),
            ComponentVariant::Alternatives(c) => c.start(),
            ComponentVariant::Consequences(c) => c.start(),
            ComponentVariant::Tradeoffs(c) => c.start(),
            ComponentVariant::Recommendation(c) => c.start(),
            ComponentVariant::DecisionQuality(c) => c.start(),
            ComponentVariant::NotesNextSteps(c) => c.start(),
        }
    }

    /// Completes this component.
    pub fn complete(&mut self) -> Result<(), ComponentError> {
        match self {
            ComponentVariant::IssueRaising(c) => c.complete(),
            ComponentVariant::ProblemFrame(c) => c.complete(),
            ComponentVariant::Objectives(c) => c.complete(),
            ComponentVariant::Alternatives(c) => c.complete(),
            ComponentVariant::Consequences(c) => c.complete(),
            ComponentVariant::Tradeoffs(c) => c.complete(),
            ComponentVariant::Recommendation(c) => c.complete(),
            ComponentVariant::DecisionQuality(c) => c.complete(),
            ComponentVariant::NotesNextSteps(c) => c.complete(),
        }
    }

    /// Marks this component for revision.
    pub fn mark_for_revision(&mut self, reason: String) -> Result<(), ComponentError> {
        match self {
            ComponentVariant::IssueRaising(c) => c.mark_for_revision(reason),
            ComponentVariant::ProblemFrame(c) => c.mark_for_revision(reason),
            ComponentVariant::Objectives(c) => c.mark_for_revision(reason),
            ComponentVariant::Alternatives(c) => c.mark_for_revision(reason),
            ComponentVariant::Consequences(c) => c.mark_for_revision(reason),
            ComponentVariant::Tradeoffs(c) => c.mark_for_revision(reason),
            ComponentVariant::Recommendation(c) => c.mark_for_revision(reason),
            ComponentVariant::DecisionQuality(c) => c.mark_for_revision(reason),
            ComponentVariant::NotesNextSteps(c) => c.mark_for_revision(reason),
        }
    }

    /// Returns the structured output as a type-erased value.
    pub fn output_as_value(&self) -> serde_json::Value {
        match self {
            ComponentVariant::IssueRaising(c) => c.output_as_value(),
            ComponentVariant::ProblemFrame(c) => c.output_as_value(),
            ComponentVariant::Objectives(c) => c.output_as_value(),
            ComponentVariant::Alternatives(c) => c.output_as_value(),
            ComponentVariant::Consequences(c) => c.output_as_value(),
            ComponentVariant::Tradeoffs(c) => c.output_as_value(),
            ComponentVariant::Recommendation(c) => c.output_as_value(),
            ComponentVariant::DecisionQuality(c) => c.output_as_value(),
            ComponentVariant::NotesNextSteps(c) => c.output_as_value(),
        }
    }

    /// Sets the structured output from a type-erased value.
    pub fn set_output_from_value(&mut self, value: serde_json::Value) -> Result<(), ComponentError> {
        match self {
            ComponentVariant::IssueRaising(c) => c.set_output_from_value(value),
            ComponentVariant::ProblemFrame(c) => c.set_output_from_value(value),
            ComponentVariant::Objectives(c) => c.set_output_from_value(value),
            ComponentVariant::Alternatives(c) => c.set_output_from_value(value),
            ComponentVariant::Consequences(c) => c.set_output_from_value(value),
            ComponentVariant::Tradeoffs(c) => c.set_output_from_value(value),
            ComponentVariant::Recommendation(c) => c.set_output_from_value(value),
            ComponentVariant::DecisionQuality(c) => c.set_output_from_value(value),
            ComponentVariant::NotesNextSteps(c) => c.set_output_from_value(value),
        }
    }

    /// Returns true if this is an IssueRaising component.
    pub fn is_issue_raising(&self) -> bool {
        matches!(self, ComponentVariant::IssueRaising(_))
    }

    /// Returns a reference to the IssueRaising component, if this is one.
    pub fn as_issue_raising(&self) -> Option<&IssueRaising> {
        match self {
            ComponentVariant::IssueRaising(c) => Some(c),
            _ => None,
        }
    }

    /// Returns a mutable reference to the IssueRaising component, if this is one.
    pub fn as_issue_raising_mut(&mut self) -> Option<&mut IssueRaising> {
        match self {
            ComponentVariant::IssueRaising(c) => Some(c),
            _ => None,
        }
    }

    /// Returns a reference to the DecisionQuality component, if this is one.
    pub fn as_decision_quality(&self) -> Option<&DecisionQuality> {
        match self {
            ComponentVariant::DecisionQuality(c) => Some(c),
            _ => None,
        }
    }

    /// Returns a mutable reference to the DecisionQuality component, if this is one.
    pub fn as_decision_quality_mut(&mut self) -> Option<&mut DecisionQuality> {
        match self {
            ComponentVariant::DecisionQuality(c) => Some(c),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_correct_component_type() {
        for ct in ComponentType::all() {
            let variant = ComponentVariant::new(*ct);
            assert_eq!(variant.component_type(), *ct);
        }
    }

    #[test]
    fn variant_returns_correct_status() {
        let variant = ComponentVariant::new(ComponentType::IssueRaising);
        assert_eq!(variant.status(), ComponentStatus::NotStarted);
    }

    #[test]
    fn variant_start_transitions_status() {
        let mut variant = ComponentVariant::new(ComponentType::ProblemFrame);
        assert!(variant.start().is_ok());
        assert_eq!(variant.status(), ComponentStatus::InProgress);
    }

    #[test]
    fn variant_complete_transitions_status() {
        let mut variant = ComponentVariant::new(ComponentType::Objectives);
        variant.start().unwrap();
        assert!(variant.complete().is_ok());
        assert_eq!(variant.status(), ComponentStatus::Complete);
    }

    #[test]
    fn is_issue_raising_returns_true_for_issue_raising() {
        let variant = ComponentVariant::new(ComponentType::IssueRaising);
        assert!(variant.is_issue_raising());
    }

    #[test]
    fn is_issue_raising_returns_false_for_other_types() {
        let variant = ComponentVariant::new(ComponentType::ProblemFrame);
        assert!(!variant.is_issue_raising());
    }

    #[test]
    fn as_issue_raising_returns_some_for_issue_raising() {
        let variant = ComponentVariant::new(ComponentType::IssueRaising);
        assert!(variant.as_issue_raising().is_some());
    }

    #[test]
    fn as_issue_raising_returns_none_for_other_types() {
        let variant = ComponentVariant::new(ComponentType::Alternatives);
        assert!(variant.as_issue_raising().is_none());
    }

    #[test]
    fn as_issue_raising_mut_allows_modification() {
        let mut variant = ComponentVariant::new(ComponentType::IssueRaising);
        if let Some(ir) = variant.as_issue_raising_mut() {
            ir.add_potential_decision("Test decision".to_string());
        }

        let ir = variant.as_issue_raising().unwrap();
        assert_eq!(ir.output().potential_decisions.len(), 1);
    }

    #[test]
    fn output_as_value_returns_json() {
        let variant = ComponentVariant::new(ComponentType::IssueRaising);
        let value = variant.output_as_value();
        assert!(value.is_object());
    }

    #[test]
    fn all_component_types_have_unique_ids() {
        let mut ids = Vec::new();
        for ct in ComponentType::all() {
            let variant = ComponentVariant::new(*ct);
            ids.push(variant.id());
        }

        // Check all IDs are unique
        let unique_count = ids.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, 9);
    }
}
