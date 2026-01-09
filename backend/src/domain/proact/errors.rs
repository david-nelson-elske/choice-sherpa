//! Error types for PrOACT components.

use thiserror::Error;

use crate::domain::foundation::ComponentStatus;

/// Errors that can occur during component operations.
#[derive(Debug, Clone, Error)]
pub enum ComponentError {
    #[error("Invalid state transition from {from} to {to}")]
    InvalidTransition {
        from: ComponentStatus,
        to: ComponentStatus,
    },

    #[error("Invalid output data: {0}")]
    InvalidOutput(String),

    #[error("Component not started")]
    NotStarted,

    #[error("Component already complete")]
    AlreadyComplete,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_transition_displays_correctly() {
        let err = ComponentError::InvalidTransition {
            from: ComponentStatus::NotStarted,
            to: ComponentStatus::Complete,
        };
        assert_eq!(
            format!("{}", err),
            "Invalid state transition from Not Started to Complete"
        );
    }

    #[test]
    fn invalid_output_displays_correctly() {
        let err = ComponentError::InvalidOutput("missing required field".to_string());
        assert_eq!(
            format!("{}", err),
            "Invalid output data: missing required field"
        );
    }

    #[test]
    fn not_started_displays_correctly() {
        let err = ComponentError::NotStarted;
        assert_eq!(format!("{}", err), "Component not started");
    }

    #[test]
    fn already_complete_displays_correctly() {
        let err = ComponentError::AlreadyComplete;
        assert_eq!(format!("{}", err), "Component already complete");
    }
}
