//! ComponentStatus enum for tracking progress of PrOACT components.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Progress tracking for a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ComponentStatus {
    #[default]
    NotStarted,
    InProgress,
    Complete,
    NeedsRevision,
}

impl ComponentStatus {
    /// Returns true if work has begun on this component.
    pub fn is_started(&self) -> bool {
        !matches!(self, ComponentStatus::NotStarted)
    }

    /// Returns true if the component is finished.
    pub fn is_complete(&self) -> bool {
        matches!(self, ComponentStatus::Complete)
    }

    /// Returns true if the component needs attention.
    pub fn needs_work(&self) -> bool {
        matches!(
            self,
            ComponentStatus::NotStarted | ComponentStatus::InProgress | ComponentStatus::NeedsRevision
        )
    }

    /// Validates a transition from this status to another.
    ///
    /// Valid transitions:
    /// - NotStarted -> InProgress
    /// - InProgress -> Complete
    /// - InProgress -> NeedsRevision
    /// - Complete -> NeedsRevision
    /// - NeedsRevision -> InProgress
    pub fn can_transition_to(&self, target: &ComponentStatus) -> bool {
        use ComponentStatus::*;
        matches!(
            (self, target),
            // Can start from not started
            (NotStarted, InProgress) |
            // Can complete from in progress
            (InProgress, Complete) |
            // Can mark for revision from complete or in progress
            (Complete, NeedsRevision) |
            (InProgress, NeedsRevision) |
            // Can restart work on revision
            (NeedsRevision, InProgress)
        )
    }
}

impl fmt::Display for ComponentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ComponentStatus::NotStarted => "Not Started",
            ComponentStatus::InProgress => "In Progress",
            ComponentStatus::Complete => "Complete",
            ComponentStatus::NeedsRevision => "Needs Revision",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_not_started() {
        assert_eq!(ComponentStatus::default(), ComponentStatus::NotStarted);
    }

    #[test]
    fn is_started_works_correctly() {
        assert!(!ComponentStatus::NotStarted.is_started());
        assert!(ComponentStatus::InProgress.is_started());
        assert!(ComponentStatus::Complete.is_started());
        assert!(ComponentStatus::NeedsRevision.is_started());
    }

    #[test]
    fn is_complete_works_correctly() {
        assert!(!ComponentStatus::NotStarted.is_complete());
        assert!(!ComponentStatus::InProgress.is_complete());
        assert!(ComponentStatus::Complete.is_complete());
        assert!(!ComponentStatus::NeedsRevision.is_complete());
    }

    #[test]
    fn needs_work_works_correctly() {
        assert!(ComponentStatus::NotStarted.needs_work());
        assert!(ComponentStatus::InProgress.needs_work());
        assert!(!ComponentStatus::Complete.needs_work());
        assert!(ComponentStatus::NeedsRevision.needs_work());
    }

    #[test]
    fn not_started_can_transition_to_in_progress() {
        assert!(ComponentStatus::NotStarted.can_transition_to(&ComponentStatus::InProgress));
    }

    #[test]
    fn not_started_cannot_transition_to_complete() {
        assert!(!ComponentStatus::NotStarted.can_transition_to(&ComponentStatus::Complete));
    }

    #[test]
    fn not_started_cannot_transition_to_needs_revision() {
        assert!(!ComponentStatus::NotStarted.can_transition_to(&ComponentStatus::NeedsRevision));
    }

    #[test]
    fn in_progress_can_transition_to_complete() {
        assert!(ComponentStatus::InProgress.can_transition_to(&ComponentStatus::Complete));
    }

    #[test]
    fn in_progress_can_transition_to_needs_revision() {
        assert!(ComponentStatus::InProgress.can_transition_to(&ComponentStatus::NeedsRevision));
    }

    #[test]
    fn in_progress_cannot_transition_to_not_started() {
        assert!(!ComponentStatus::InProgress.can_transition_to(&ComponentStatus::NotStarted));
    }

    #[test]
    fn complete_can_transition_to_needs_revision() {
        assert!(ComponentStatus::Complete.can_transition_to(&ComponentStatus::NeedsRevision));
    }

    #[test]
    fn complete_cannot_transition_to_not_started() {
        assert!(!ComponentStatus::Complete.can_transition_to(&ComponentStatus::NotStarted));
    }

    #[test]
    fn complete_cannot_transition_to_in_progress() {
        assert!(!ComponentStatus::Complete.can_transition_to(&ComponentStatus::InProgress));
    }

    #[test]
    fn needs_revision_can_transition_to_in_progress() {
        assert!(ComponentStatus::NeedsRevision.can_transition_to(&ComponentStatus::InProgress));
    }

    #[test]
    fn needs_revision_cannot_transition_to_complete() {
        assert!(!ComponentStatus::NeedsRevision.can_transition_to(&ComponentStatus::Complete));
    }

    #[test]
    fn display_works_correctly() {
        assert_eq!(format!("{}", ComponentStatus::NotStarted), "Not Started");
        assert_eq!(format!("{}", ComponentStatus::InProgress), "In Progress");
        assert_eq!(format!("{}", ComponentStatus::Complete), "Complete");
        assert_eq!(format!("{}", ComponentStatus::NeedsRevision), "Needs Revision");
    }

    #[test]
    fn serializes_to_snake_case_json() {
        assert_eq!(
            serde_json::to_string(&ComponentStatus::NotStarted).unwrap(),
            "\"not_started\""
        );
        assert_eq!(
            serde_json::to_string(&ComponentStatus::InProgress).unwrap(),
            "\"in_progress\""
        );
        assert_eq!(
            serde_json::to_string(&ComponentStatus::NeedsRevision).unwrap(),
            "\"needs_revision\""
        );
    }

    #[test]
    fn deserializes_from_snake_case_json() {
        let status: ComponentStatus = serde_json::from_str("\"not_started\"").unwrap();
        assert_eq!(status, ComponentStatus::NotStarted);

        let status: ComponentStatus = serde_json::from_str("\"needs_revision\"").unwrap();
        assert_eq!(status, ComponentStatus::NeedsRevision);
    }
}
