//! Error types for AI Engine domain

use crate::domain::foundation::ComponentType;

/// Orchestrator domain errors
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum OrchestratorError {
    #[error("Invalid transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: ComponentType,
        to: ComponentType,
    },

    #[error("Step {0:?} not yet completed")]
    StepNotCompleted(ComponentType),

    #[error("Cycle already completed")]
    CycleCompleted,

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Step not found: {0:?}")]
    StepNotFound(ComponentType),
}

/// Compression errors
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum CompressionError {
    #[error("AI service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Compression failed: {0}")]
    CompressionFailed(String),

    #[error("Context too large: {size} exceeds limit {limit}")]
    ContextTooLarge { size: usize, limit: usize },
}

/// Extraction errors
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum ExtractionError {
    #[error("Invalid response format: {0}")]
    InvalidFormat(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Unsupported component: {0:?}")]
    UnsupportedComponent(ComponentType),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_error_invalid_transition() {
        let err = OrchestratorError::InvalidTransition {
            from: ComponentType::IssueRaising,
            to: ComponentType::Consequences,
        };

        assert!(err
            .to_string()
            .contains("Invalid transition from IssueRaising"));
    }

    #[test]
    fn test_orchestrator_error_step_not_completed() {
        let err = OrchestratorError::StepNotCompleted(ComponentType::ProblemFrame);

        assert!(err.to_string().contains("ProblemFrame not yet completed"));
    }

    #[test]
    fn test_orchestrator_error_cycle_completed() {
        let err = OrchestratorError::CycleCompleted;

        assert_eq!(err.to_string(), "Cycle already completed");
    }

    #[test]
    fn test_compression_error_service_unavailable() {
        let err = CompressionError::ServiceUnavailable("OpenAI down".to_string());

        assert!(err.to_string().contains("AI service unavailable"));
    }

    #[test]
    fn test_compression_error_context_too_large() {
        let err = CompressionError::ContextTooLarge {
            size: 100000,
            limit: 50000,
        };

        assert!(err.to_string().contains("100000 exceeds limit 50000"));
    }

    #[test]
    fn test_extraction_error_invalid_format() {
        let err = ExtractionError::InvalidFormat("Not valid JSON".to_string());

        assert!(err.to_string().contains("Invalid response format"));
    }

    #[test]
    fn test_extraction_error_missing_field() {
        let err = ExtractionError::MissingField("objectives".to_string());

        assert!(err.to_string().contains("Missing required field: objectives"));
    }
}
