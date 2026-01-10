//! Tool execution result value object.
//!
//! Represents the outcome of a tool invocation.

use serde::{Deserialize, Serialize};

/// Outcome of a tool execution.
///
/// Represents whether the tool succeeded, and if not, what category
/// of failure occurred. This enables structured error handling and
/// retry logic based on error type.
///
/// # Examples
///
/// ```ignore
/// use choice_sherpa::domain::conversation::tools::ToolResult;
///
/// let result = ToolResult::Success;
/// assert!(result.is_success());
///
/// let error = ToolResult::ValidationError;
/// assert!(!error.is_success());
/// assert!(error.is_retryable() == false);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolResult {
    /// Tool executed successfully
    Success,

    /// Parameters failed validation (e.g., missing required field, invalid format)
    ValidationError,

    /// Referenced entity not found (e.g., unknown objective_id)
    NotFound,

    /// Operation would violate business rules (e.g., duplicate alternative name)
    Conflict,

    /// Unexpected system error (database failure, etc.)
    InternalError,
}

impl ToolResult {
    /// Returns true if the tool executed successfully.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    /// Returns true if the error is potentially retryable.
    ///
    /// Internal errors may be transient and worth retrying.
    /// Validation errors and not-found errors are not retryable without
    /// user intervention.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::InternalError)
    }

    /// Returns a human-readable description of the result.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Success => "Tool executed successfully",
            Self::ValidationError => "Tool parameters failed validation",
            Self::NotFound => "Referenced entity not found",
            Self::Conflict => "Operation would violate business rules",
            Self::InternalError => "Unexpected system error occurred",
        }
    }
}

impl std::fmt::Display for ToolResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_is_success() {
        let result = ToolResult::Success;
        assert!(result.is_success());
        assert!(!result.is_retryable());
    }

    #[test]
    fn validation_error_is_not_retryable() {
        let result = ToolResult::ValidationError;
        assert!(!result.is_success());
        assert!(!result.is_retryable());
    }

    #[test]
    fn internal_error_is_retryable() {
        let result = ToolResult::InternalError;
        assert!(!result.is_success());
        assert!(result.is_retryable());
    }

    #[test]
    fn serializes_to_snake_case() {
        let result = ToolResult::ValidationError;
        let json = serde_json::to_string(&result).unwrap();
        assert_eq!(json, "\"validation_error\"");
    }

    #[test]
    fn deserializes_from_snake_case() {
        let result: ToolResult = serde_json::from_str("\"not_found\"").unwrap();
        assert_eq!(result, ToolResult::NotFound);
    }

    #[test]
    fn description_returns_human_readable_text() {
        assert!(!ToolResult::Success.description().is_empty());
        assert!(!ToolResult::Conflict.description().is_empty());
    }

    #[test]
    fn display_uses_description() {
        let result = ToolResult::NotFound;
        assert_eq!(format!("{}", result), result.description());
    }
}
