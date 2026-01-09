//! Component trait and ComponentBase for PrOACT components.

use crate::domain::foundation::{ComponentId, ComponentStatus, ComponentType, Timestamp};

use super::ComponentError;

/// Trait that all PrOACT components implement.
pub trait Component: Send + Sync {
    /// Returns the unique identifier.
    fn id(&self) -> ComponentId;

    /// Returns the component type.
    fn component_type(&self) -> ComponentType;

    /// Returns the current status.
    fn status(&self) -> ComponentStatus;

    /// Returns when this component was created.
    fn created_at(&self) -> Timestamp;

    /// Returns when this component was last updated.
    fn updated_at(&self) -> Timestamp;

    /// Starts work on this component.
    fn start(&mut self) -> Result<(), ComponentError>;

    /// Completes this component.
    fn complete(&mut self) -> Result<(), ComponentError>;

    /// Marks this component for revision.
    fn mark_for_revision(&mut self, reason: String) -> Result<(), ComponentError>;

    /// Returns the structured output as a type-erased value.
    fn output_as_value(&self) -> serde_json::Value;

    /// Sets the structured output from a type-erased value.
    fn set_output_from_value(&mut self, value: serde_json::Value) -> Result<(), ComponentError>;
}

/// Base fields shared by all components.
#[derive(Debug, Clone)]
pub struct ComponentBase {
    pub id: ComponentId,
    pub component_type: ComponentType,
    pub status: ComponentStatus,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub revision_reason: Option<String>,
}

impl ComponentBase {
    /// Creates a new ComponentBase for the given component type.
    pub fn new(component_type: ComponentType) -> Self {
        let now = Timestamp::now();
        Self {
            id: ComponentId::new(),
            component_type,
            status: ComponentStatus::NotStarted,
            created_at: now,
            updated_at: now,
            revision_reason: None,
        }
    }

    /// Starts work on this component.
    pub fn start(&mut self) -> Result<(), ComponentError> {
        if !self.status.can_transition_to(&ComponentStatus::InProgress) {
            return Err(ComponentError::InvalidTransition {
                from: self.status,
                to: ComponentStatus::InProgress,
            });
        }
        self.status = ComponentStatus::InProgress;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Completes this component.
    pub fn complete(&mut self) -> Result<(), ComponentError> {
        if !self.status.can_transition_to(&ComponentStatus::Complete) {
            return Err(ComponentError::InvalidTransition {
                from: self.status,
                to: ComponentStatus::Complete,
            });
        }
        self.status = ComponentStatus::Complete;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Marks this component for revision.
    pub fn mark_for_revision(&mut self, reason: String) -> Result<(), ComponentError> {
        if !self.status.can_transition_to(&ComponentStatus::NeedsRevision) {
            return Err(ComponentError::InvalidTransition {
                from: self.status,
                to: ComponentStatus::NeedsRevision,
            });
        }
        self.status = ComponentStatus::NeedsRevision;
        self.revision_reason = Some(reason);
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Updates the timestamp to now.
    pub fn touch(&mut self) {
        self.updated_at = Timestamp::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn component_base_new_creates_with_defaults() {
        let base = ComponentBase::new(ComponentType::IssueRaising);

        assert_eq!(base.component_type, ComponentType::IssueRaising);
        assert_eq!(base.status, ComponentStatus::NotStarted);
        assert!(base.revision_reason.is_none());
    }

    #[test]
    fn component_base_new_generates_unique_ids() {
        let base1 = ComponentBase::new(ComponentType::IssueRaising);
        let base2 = ComponentBase::new(ComponentType::IssueRaising);
        assert_ne!(base1.id, base2.id);
    }

    #[test]
    fn component_base_start_transitions_to_in_progress() {
        let mut base = ComponentBase::new(ComponentType::ProblemFrame);
        assert!(base.start().is_ok());
        assert_eq!(base.status, ComponentStatus::InProgress);
    }

    #[test]
    fn component_base_start_updates_timestamp() {
        let mut base = ComponentBase::new(ComponentType::ProblemFrame);
        let initial = base.updated_at;
        sleep(Duration::from_millis(10));
        base.start().unwrap();
        assert!(base.updated_at.is_after(&initial));
    }

    #[test]
    fn component_base_start_fails_if_already_complete() {
        let mut base = ComponentBase::new(ComponentType::Objectives);
        base.start().unwrap();
        base.complete().unwrap();

        let result = base.start();
        assert!(result.is_err());
        match result {
            Err(ComponentError::InvalidTransition { from, to }) => {
                assert_eq!(from, ComponentStatus::Complete);
                assert_eq!(to, ComponentStatus::InProgress);
            }
            _ => panic!("Expected InvalidTransition error"),
        }
    }

    #[test]
    fn component_base_complete_transitions_from_in_progress() {
        let mut base = ComponentBase::new(ComponentType::Alternatives);
        base.start().unwrap();
        assert!(base.complete().is_ok());
        assert_eq!(base.status, ComponentStatus::Complete);
    }

    #[test]
    fn component_base_complete_fails_if_not_started() {
        let mut base = ComponentBase::new(ComponentType::Consequences);

        let result = base.complete();
        assert!(result.is_err());
        match result {
            Err(ComponentError::InvalidTransition { from, to }) => {
                assert_eq!(from, ComponentStatus::NotStarted);
                assert_eq!(to, ComponentStatus::Complete);
            }
            _ => panic!("Expected InvalidTransition error"),
        }
    }

    #[test]
    fn component_base_mark_for_revision_from_in_progress() {
        let mut base = ComponentBase::new(ComponentType::Tradeoffs);
        base.start().unwrap();

        assert!(base.mark_for_revision("Needs more detail".to_string()).is_ok());
        assert_eq!(base.status, ComponentStatus::NeedsRevision);
        assert_eq!(base.revision_reason, Some("Needs more detail".to_string()));
    }

    #[test]
    fn component_base_mark_for_revision_from_complete() {
        let mut base = ComponentBase::new(ComponentType::Recommendation);
        base.start().unwrap();
        base.complete().unwrap();

        assert!(base.mark_for_revision("Found error".to_string()).is_ok());
        assert_eq!(base.status, ComponentStatus::NeedsRevision);
    }

    #[test]
    fn component_base_mark_for_revision_fails_if_not_started() {
        let mut base = ComponentBase::new(ComponentType::DecisionQuality);

        let result = base.mark_for_revision("Some reason".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn component_base_can_restart_after_revision() {
        let mut base = ComponentBase::new(ComponentType::NotesNextSteps);
        base.start().unwrap();
        base.mark_for_revision("Needs rework".to_string()).unwrap();

        assert!(base.start().is_ok());
        assert_eq!(base.status, ComponentStatus::InProgress);
    }

    #[test]
    fn component_base_touch_updates_timestamp() {
        let mut base = ComponentBase::new(ComponentType::IssueRaising);
        let initial = base.updated_at;
        sleep(Duration::from_millis(10));
        base.touch();
        assert!(base.updated_at.is_after(&initial));
    }
}
