//! CycleStatus enum for tracking lifecycle of decision cycles.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Lifecycle status of a decision cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CycleStatus {
    #[default]
    Active,
    Completed,
    Archived,
}

impl CycleStatus {
    /// Returns true if the cycle can be modified.
    pub fn is_mutable(&self) -> bool {
        matches!(self, CycleStatus::Active)
    }

    /// Returns true if the cycle is finished (completed or archived).
    pub fn is_finished(&self) -> bool {
        matches!(self, CycleStatus::Completed | CycleStatus::Archived)
    }

    /// Validates a transition from this status to another.
    ///
    /// Valid transitions:
    /// - Active -> Completed
    /// - Active -> Archived
    /// - Completed -> Archived
    pub fn can_transition_to(&self, target: &CycleStatus) -> bool {
        use CycleStatus::*;
        matches!(
            (self, target),
            (Active, Completed) | (Active, Archived) | (Completed, Archived)
        )
    }
}

impl fmt::Display for CycleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CycleStatus::Active => "Active",
            CycleStatus::Completed => "Completed",
            CycleStatus::Archived => "Archived",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_active() {
        assert_eq!(CycleStatus::default(), CycleStatus::Active);
    }

    #[test]
    fn is_mutable_works_correctly() {
        assert!(CycleStatus::Active.is_mutable());
        assert!(!CycleStatus::Completed.is_mutable());
        assert!(!CycleStatus::Archived.is_mutable());
    }

    #[test]
    fn is_finished_works_correctly() {
        assert!(!CycleStatus::Active.is_finished());
        assert!(CycleStatus::Completed.is_finished());
        assert!(CycleStatus::Archived.is_finished());
    }

    #[test]
    fn active_can_transition_to_completed() {
        assert!(CycleStatus::Active.can_transition_to(&CycleStatus::Completed));
    }

    #[test]
    fn active_can_transition_to_archived() {
        assert!(CycleStatus::Active.can_transition_to(&CycleStatus::Archived));
    }

    #[test]
    fn completed_can_transition_to_archived() {
        assert!(CycleStatus::Completed.can_transition_to(&CycleStatus::Archived));
    }

    #[test]
    fn completed_cannot_transition_to_active() {
        assert!(!CycleStatus::Completed.can_transition_to(&CycleStatus::Active));
    }

    #[test]
    fn archived_cannot_transition_to_anything() {
        assert!(!CycleStatus::Archived.can_transition_to(&CycleStatus::Active));
        assert!(!CycleStatus::Archived.can_transition_to(&CycleStatus::Completed));
        assert!(!CycleStatus::Archived.can_transition_to(&CycleStatus::Archived));
    }

    #[test]
    fn display_works_correctly() {
        assert_eq!(format!("{}", CycleStatus::Active), "Active");
        assert_eq!(format!("{}", CycleStatus::Completed), "Completed");
        assert_eq!(format!("{}", CycleStatus::Archived), "Archived");
    }

    #[test]
    fn serializes_to_snake_case_json() {
        assert_eq!(
            serde_json::to_string(&CycleStatus::Active).unwrap(),
            "\"active\""
        );
        assert_eq!(
            serde_json::to_string(&CycleStatus::Completed).unwrap(),
            "\"completed\""
        );
    }

    #[test]
    fn deserializes_from_snake_case_json() {
        let status: CycleStatus = serde_json::from_str("\"active\"").unwrap();
        assert_eq!(status, CycleStatus::Active);

        let status: CycleStatus = serde_json::from_str("\"archived\"").unwrap();
        assert_eq!(status, CycleStatus::Archived);
    }
}
