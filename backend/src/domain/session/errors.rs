//! Session-specific error types.

use crate::domain::foundation::{DomainError, ErrorCode, SessionId};
use crate::ports::AccessDeniedReason;

/// Session-specific errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionError {
    /// Session was not found.
    NotFound(SessionId),
    /// User is not authorized.
    Forbidden,
    /// Access denied due to membership restrictions.
    AccessDenied(AccessDeniedReason),
    /// Invalid state for operation.
    InvalidState(String),
    /// Session is archived.
    AlreadyArchived,
    /// Validation failed.
    ValidationFailed { field: String, message: String },
    /// Infrastructure error.
    Infrastructure(String),
}

impl SessionError {
    pub fn not_found(id: SessionId) -> Self {
        SessionError::NotFound(id)
    }
    pub fn forbidden() -> Self {
        SessionError::Forbidden
    }
    pub fn access_denied(reason: AccessDeniedReason) -> Self {
        SessionError::AccessDenied(reason)
    }
    pub fn invalid_state(message: impl Into<String>) -> Self {
        SessionError::InvalidState(message.into())
    }
    pub fn already_archived() -> Self {
        SessionError::AlreadyArchived
    }
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        SessionError::ValidationFailed {
            field: field.into(),
            message: message.into(),
        }
    }
    pub fn infrastructure(message: impl Into<String>) -> Self {
        SessionError::Infrastructure(message.into())
    }
    pub fn code(&self) -> ErrorCode {
        match self {
            SessionError::NotFound(_) => ErrorCode::SessionNotFound,
            SessionError::Forbidden => ErrorCode::Forbidden,
            SessionError::AccessDenied(_) => ErrorCode::Forbidden,
            SessionError::InvalidState(_) => ErrorCode::InvalidStateTransition,
            SessionError::AlreadyArchived => ErrorCode::SessionArchived,
            SessionError::ValidationFailed { .. } => ErrorCode::ValidationFailed,
            SessionError::Infrastructure(_) => ErrorCode::DatabaseError,
        }
    }
    pub fn message(&self) -> String {
        match self {
            SessionError::NotFound(id) => format!("Session not found: {}", id),
            SessionError::Forbidden => "Permission denied".to_string(),
            SessionError::AccessDenied(reason) => reason.user_message(),
            SessionError::InvalidState(msg) => format!("Invalid state: {}", msg),
            SessionError::AlreadyArchived => "Cannot modify archived session".to_string(),
            SessionError::ValidationFailed { field, message } => {
                format!("Validation failed for '{}': {}", field, message)
            }
            SessionError::Infrastructure(msg) => format!("Error: {}", msg),
        }
    }
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for SessionError {}

impl From<DomainError> for SessionError {
    fn from(err: DomainError) -> Self {
        match err.code {
            ErrorCode::SessionNotFound => SessionError::Forbidden,
            ErrorCode::Forbidden => SessionError::Forbidden,
            ErrorCode::SessionArchived => SessionError::AlreadyArchived,
            ErrorCode::InvalidStateTransition => SessionError::InvalidState(err.to_string()),
            ErrorCode::ValidationFailed => SessionError::ValidationFailed {
                field: "unknown".to_string(),
                message: err.to_string(),
            },
            _ => SessionError::Infrastructure(err.to_string()),
        }
    }
}
