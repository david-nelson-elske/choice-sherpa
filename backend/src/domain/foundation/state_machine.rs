//! State machine trait for status enums.
//!
//! Provides a consistent interface for validating and performing state transitions
//! across different entity lifecycle statuses (Session, Cycle, Component, etc.).

use super::ValidationError;

/// Trait for status enums that represent state machines.
///
/// Implementors define valid state transitions and get validated
/// transition methods for free.
///
/// # Example
///
/// ```ignore
/// impl StateMachine for ComponentStatus {
///     fn can_transition_to(&self, target: &Self) -> bool {
///         matches!(
///             (self, target),
///             (NotStarted, InProgress) |
///             (InProgress, Complete) |
///             // ... etc
///         )
///     }
///
///     fn valid_transitions(&self) -> Vec<Self> {
///         match self {
///             NotStarted => vec![InProgress],
///             InProgress => vec![Complete, NeedsRevision],
///             // ... etc
///         }
///     }
/// }
///
/// // Usage:
/// let new_status = current_status.transition_to(ComponentStatus::Complete)?;
/// ```
pub trait StateMachine: Sized + Copy + PartialEq + std::fmt::Debug {
    /// Returns true if transition from self to target is valid.
    fn can_transition_to(&self, target: &Self) -> bool;

    /// Returns all valid target states from current state.
    fn valid_transitions(&self) -> Vec<Self>;

    /// Performs transition with validation, returning error if invalid.
    ///
    /// This is the preferred way to change state, as it ensures
    /// the transition is valid according to the state machine rules.
    fn transition_to(&self, target: Self) -> Result<Self, ValidationError> {
        if self.can_transition_to(&target) {
            Ok(target)
        } else {
            Err(ValidationError::invalid_format(
                "state_transition",
                format!("Cannot transition from {:?} to {:?}", self, target),
            ))
        }
    }

    /// Checks if current state is terminal (no valid outgoing transitions).
    fn is_terminal(&self) -> bool {
        self.valid_transitions().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test enum for StateMachine trait
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestStatus {
        Draft,
        Active,
        Completed,
        Archived,
    }

    impl StateMachine for TestStatus {
        fn can_transition_to(&self, target: &Self) -> bool {
            use TestStatus::*;
            matches!(
                (self, target),
                (Draft, Active)
                    | (Active, Completed)
                    | (Active, Archived)
                    | (Completed, Archived)
            )
        }

        fn valid_transitions(&self) -> Vec<Self> {
            use TestStatus::*;
            match self {
                Draft => vec![Active],
                Active => vec![Completed, Archived],
                Completed => vec![Archived],
                Archived => vec![],
            }
        }
    }

    #[test]
    fn transition_to_succeeds_for_valid_transition() {
        let status = TestStatus::Draft;
        let result = status.transition_to(TestStatus::Active);
        assert_eq!(result, Ok(TestStatus::Active));
    }

    #[test]
    fn transition_to_fails_for_invalid_transition() {
        let status = TestStatus::Draft;
        let result = status.transition_to(TestStatus::Completed);
        assert!(result.is_err());
    }

    #[test]
    fn is_terminal_returns_true_for_archived() {
        assert!(TestStatus::Archived.is_terminal());
    }

    #[test]
    fn is_terminal_returns_false_for_non_terminal() {
        assert!(!TestStatus::Draft.is_terminal());
        assert!(!TestStatus::Active.is_terminal());
        assert!(!TestStatus::Completed.is_terminal());
    }

    #[test]
    fn valid_transitions_returns_correct_targets() {
        assert_eq!(TestStatus::Draft.valid_transitions(), vec![TestStatus::Active]);
        assert_eq!(
            TestStatus::Active.valid_transitions(),
            vec![TestStatus::Completed, TestStatus::Archived]
        );
        assert_eq!(TestStatus::Archived.valid_transitions(), vec![]);
    }

    #[test]
    fn can_transition_to_is_consistent_with_valid_transitions() {
        for status in [
            TestStatus::Draft,
            TestStatus::Active,
            TestStatus::Completed,
            TestStatus::Archived,
        ] {
            for valid_target in status.valid_transitions() {
                assert!(
                    status.can_transition_to(&valid_target),
                    "can_transition_to should return true for {:?} -> {:?}",
                    status,
                    valid_target
                );
            }
        }
    }
}
