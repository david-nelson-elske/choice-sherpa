//! Error types for the domain layer.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use thiserror::Error;

/// Errors that occur during value object construction.
#[derive(Debug, Clone, Error)]
pub enum ValidationError {
    #[error("Field '{field}' cannot be empty")]
    EmptyField { field: String },

    #[error("Field '{field}' must be between {min} and {max}, got {actual}")]
    OutOfRange {
        field: String,
        min: i32,
        max: i32,
        actual: i32,
    },

    #[error("Field '{field}' has invalid format: {reason}")]
    InvalidFormat { field: String, reason: String },
}

impl ValidationError {
    /// Creates an empty field validation error.
    pub fn empty_field(field: impl Into<String>) -> Self {
        ValidationError::EmptyField { field: field.into() }
    }

    /// Creates an out of range validation error.
    pub fn out_of_range(field: impl Into<String>, min: i32, max: i32, actual: i32) -> Self {
        ValidationError::OutOfRange {
            field: field.into(),
            min,
            max,
            actual,
        }
    }

    /// Creates an invalid format validation error.
    pub fn invalid_format(field: impl Into<String>, reason: impl Into<String>) -> Self {
        ValidationError::InvalidFormat {
            field: field.into(),
            reason: reason.into(),
        }
    }
}

/// Error codes organized by category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // Validation errors
    ValidationFailed,
    EmptyField,
    OutOfRange,
    InvalidFormat,

    // Not found errors
    SessionNotFound,
    CycleNotFound,
    ComponentNotFound,
    ConversationNotFound,

    // State errors
    InvalidStateTransition,
    SessionArchived,
    CycleArchived,
    ComponentLocked,
    ComponentAlreadyStarted,
    PreviousComponentRequired,
    InvalidComponentOutput,
    CannotBranch,

    // Authorization errors
    Unauthorized,
    Forbidden,

    // AI errors
    AIProviderError,
    RateLimited,

    // Infrastructure errors
    DatabaseError,
    CacheError,
    InternalError,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ErrorCode::ValidationFailed => "VALIDATION_FAILED",
            ErrorCode::EmptyField => "EMPTY_FIELD",
            ErrorCode::OutOfRange => "OUT_OF_RANGE",
            ErrorCode::InvalidFormat => "INVALID_FORMAT",
            ErrorCode::SessionNotFound => "SESSION_NOT_FOUND",
            ErrorCode::CycleNotFound => "CYCLE_NOT_FOUND",
            ErrorCode::ComponentNotFound => "COMPONENT_NOT_FOUND",
            ErrorCode::ConversationNotFound => "CONVERSATION_NOT_FOUND",
            ErrorCode::InvalidStateTransition => "INVALID_STATE_TRANSITION",
            ErrorCode::SessionArchived => "SESSION_ARCHIVED",
            ErrorCode::CycleArchived => "CYCLE_ARCHIVED",
            ErrorCode::ComponentLocked => "COMPONENT_LOCKED",
            ErrorCode::ComponentAlreadyStarted => "COMPONENT_ALREADY_STARTED",
            ErrorCode::PreviousComponentRequired => "PREVIOUS_COMPONENT_REQUIRED",
            ErrorCode::InvalidComponentOutput => "INVALID_COMPONENT_OUTPUT",
            ErrorCode::CannotBranch => "CANNOT_BRANCH",
            ErrorCode::Unauthorized => "UNAUTHORIZED",
            ErrorCode::Forbidden => "FORBIDDEN",
            ErrorCode::AIProviderError => "AI_PROVIDER_ERROR",
            ErrorCode::RateLimited => "RATE_LIMITED",
            ErrorCode::DatabaseError => "DATABASE_ERROR",
            ErrorCode::CacheError => "CACHE_ERROR",
            ErrorCode::InternalError => "INTERNAL_ERROR",
        };
        write!(f, "{}", s)
    }
}

/// Standard domain error with code, message, and optional details.
#[derive(Debug, Clone)]
pub struct DomainError {
    pub code: ErrorCode,
    pub message: String,
    pub details: HashMap<String, String>,
}

impl DomainError {
    /// Creates a new domain error.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: HashMap::new(),
        }
    }

    /// Creates a validation error for a specific field.
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::ValidationFailed,
            message: message.into(),
            details: HashMap::new(),
        }
        .with_detail("field", field.into())
    }

    /// Adds a detail to the error.
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl Error for DomainError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_error_empty_field_displays_correctly() {
        let err = ValidationError::empty_field("username");
        assert_eq!(format!("{}", err), "Field 'username' cannot be empty");
    }

    #[test]
    fn validation_error_out_of_range_displays_correctly() {
        let err = ValidationError::out_of_range("age", 0, 100, 150);
        assert_eq!(
            format!("{}", err),
            "Field 'age' must be between 0 and 100, got 150"
        );
    }

    #[test]
    fn validation_error_invalid_format_displays_correctly() {
        let err = ValidationError::invalid_format("email", "missing @ symbol");
        assert_eq!(
            format!("{}", err),
            "Field 'email' has invalid format: missing @ symbol"
        );
    }

    #[test]
    fn domain_error_displays_code_and_message() {
        let err = DomainError::new(ErrorCode::SessionNotFound, "Session not found");
        assert_eq!(format!("{}", err), "[SESSION_NOT_FOUND] Session not found");
    }

    #[test]
    fn domain_error_with_detail_adds_detail() {
        let err = DomainError::new(ErrorCode::ValidationFailed, "Validation failed")
            .with_detail("field", "email")
            .with_detail("reason", "invalid format");

        assert_eq!(err.details.get("field"), Some(&"email".to_string()));
        assert_eq!(err.details.get("reason"), Some(&"invalid format".to_string()));
    }

    #[test]
    fn error_code_display_formats_correctly() {
        assert_eq!(format!("{}", ErrorCode::SessionNotFound), "SESSION_NOT_FOUND");
        assert_eq!(format!("{}", ErrorCode::InternalError), "INTERNAL_ERROR");
    }
}
