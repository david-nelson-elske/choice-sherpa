//! SessionStatus enum for tracking lifecycle of decision sessions.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Lifecycle status of a decision session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    #[default]
    Active,
    Archived,
}

impl SessionStatus {
    /// Returns true if the session can be modified.
    pub fn is_mutable(&self) -> bool {
        matches!(self, SessionStatus::Active)
    }

    /// Validates a transition from this status to another.
    ///
    /// Valid transitions:
    /// - Active -> Archived
    pub fn can_transition_to(&self, target: &SessionStatus) -> bool {
        use SessionStatus::*;
        matches!((self, target), (Active, Archived))
    }
}

impl fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SessionStatus::Active => "Active",
            SessionStatus::Archived => "Archived",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_active() {
        assert_eq!(SessionStatus::default(), SessionStatus::Active);
    }

    #[test]
    fn is_mutable_works_correctly() {
        assert!(SessionStatus::Active.is_mutable());
        assert!(!SessionStatus::Archived.is_mutable());
    }

    #[test]
    fn active_can_transition_to_archived() {
        assert!(SessionStatus::Active.can_transition_to(&SessionStatus::Archived));
    }

    #[test]
    fn active_cannot_transition_to_active() {
        assert!(!SessionStatus::Active.can_transition_to(&SessionStatus::Active));
    }

    #[test]
    fn archived_cannot_transition_to_active() {
        assert!(!SessionStatus::Archived.can_transition_to(&SessionStatus::Active));
    }

    #[test]
    fn archived_cannot_transition_to_archived() {
        assert!(!SessionStatus::Archived.can_transition_to(&SessionStatus::Archived));
    }

    #[test]
    fn display_works_correctly() {
        assert_eq!(format!("{}", SessionStatus::Active), "Active");
        assert_eq!(format!("{}", SessionStatus::Archived), "Archived");
    }

    #[test]
    fn serializes_to_snake_case_json() {
        assert_eq!(
            serde_json::to_string(&SessionStatus::Active).unwrap(),
            "\"active\""
        );
        assert_eq!(
            serde_json::to_string(&SessionStatus::Archived).unwrap(),
            "\"archived\""
        );
    }

    #[test]
    fn deserializes_from_snake_case_json() {
        let status: SessionStatus = serde_json::from_str("\"active\"").unwrap();
        assert_eq!(status, SessionStatus::Active);

        let status: SessionStatus = serde_json::from_str("\"archived\"").unwrap();
        assert_eq!(status, SessionStatus::Archived);
    }
}
