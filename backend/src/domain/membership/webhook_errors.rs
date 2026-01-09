//! Webhook error types for Stripe webhook handling.
//!
//! Defines all error conditions that can occur during webhook processing,
//! with HTTP status code mapping and retryability semantics.

use axum::http::StatusCode;
use thiserror::Error;

/// Errors that occur during webhook processing.
#[derive(Debug, Error)]
pub enum WebhookError {
    /// Webhook signature verification failed.
    #[error("Invalid signature")]
    InvalidSignature,

    /// Webhook timestamp is outside the acceptable window (5 minutes).
    #[error("Timestamp out of range")]
    TimestampOutOfRange,

    /// Event timestamp is in the future beyond clock skew tolerance.
    #[error("Invalid timestamp")]
    InvalidTimestamp,

    /// Failed to parse webhook payload or signature header.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Required metadata field missing from webhook event.
    #[error("Missing metadata: {0}")]
    MissingMetadata(&'static str),

    /// Required field missing from webhook payload.
    #[error("Missing field: {0}")]
    MissingField(&'static str),

    /// Referenced membership could not be found.
    #[error("Membership not found")]
    MembershipNotFound,

    /// Attempted state transition is not valid.
    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),

    /// Event was intentionally ignored (not an error condition).
    #[error("Event ignored: {0}")]
    Ignored(String),

    /// Database operation failed.
    #[error("Database error: {0}")]
    Database(String),

    /// Storage operation failed (Redis/cache).
    #[error("Storage error: {0}")]
    StorageError(String),
}

impl WebhookError {
    /// Returns true if Stripe should retry delivering this webhook.
    ///
    /// Retryable errors indicate temporary failures that may succeed
    /// on subsequent attempts (database issues, eventual consistency).
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            WebhookError::Database(_)
                | WebhookError::StorageError(_)
                | WebhookError::MembershipNotFound // Might be eventual consistency
        )
    }

    /// Maps the error to an appropriate HTTP status code.
    ///
    /// Status codes determine Stripe's retry behavior:
    /// - 2xx: Event acknowledged, no retry
    /// - 4xx: Client error, no retry
    /// - 5xx: Server error, will retry
    pub fn status_code(&self) -> StatusCode {
        match self {
            // Auth failures - don't retry
            WebhookError::InvalidSignature | WebhookError::TimestampOutOfRange => {
                StatusCode::UNAUTHORIZED
            }

            // Invalid timestamp (future) - don't retry
            WebhookError::InvalidTimestamp => StatusCode::BAD_REQUEST,

            // Bad request - don't retry
            WebhookError::ParseError(_)
            | WebhookError::MissingMetadata(_)
            | WebhookError::MissingField(_) => StatusCode::BAD_REQUEST,

            // Ignored events are acknowledged as success
            WebhookError::Ignored(_) => StatusCode::OK,

            // Server errors - will retry
            WebhookError::MembershipNotFound
            | WebhookError::InvalidTransition(_)
            | WebhookError::Database(_)
            | WebhookError::StorageError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ══════════════════════════════════════════════════════════════
    // Error Display Tests
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn invalid_signature_displays_correctly() {
        let err = WebhookError::InvalidSignature;
        assert_eq!(format!("{}", err), "Invalid signature");
    }

    #[test]
    fn timestamp_out_of_range_displays_correctly() {
        let err = WebhookError::TimestampOutOfRange;
        assert_eq!(format!("{}", err), "Timestamp out of range");
    }

    #[test]
    fn parse_error_displays_message() {
        let err = WebhookError::ParseError("invalid JSON".to_string());
        assert_eq!(format!("{}", err), "Parse error: invalid JSON");
    }

    #[test]
    fn missing_metadata_displays_field_name() {
        let err = WebhookError::MissingMetadata("membership_id");
        assert_eq!(format!("{}", err), "Missing metadata: membership_id");
    }

    #[test]
    fn missing_field_displays_field_name() {
        let err = WebhookError::MissingField("subscription");
        assert_eq!(format!("{}", err), "Missing field: subscription");
    }

    #[test]
    fn invalid_transition_displays_reason() {
        let err = WebhookError::InvalidTransition("cannot go from Expired to Active".to_string());
        assert_eq!(
            format!("{}", err),
            "Invalid state transition: cannot go from Expired to Active"
        );
    }

    #[test]
    fn ignored_displays_reason() {
        let err = WebhookError::Ignored("membership already active".to_string());
        assert_eq!(format!("{}", err), "Event ignored: membership already active");
    }

    // ══════════════════════════════════════════════════════════════
    // Retryability Tests
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn database_error_is_retryable() {
        let err = WebhookError::Database("connection failed".to_string());
        assert!(err.is_retryable());
    }

    #[test]
    fn storage_error_is_retryable() {
        let err = WebhookError::StorageError("redis timeout".to_string());
        assert!(err.is_retryable());
    }

    #[test]
    fn membership_not_found_is_retryable() {
        // Eventual consistency - might succeed on retry
        let err = WebhookError::MembershipNotFound;
        assert!(err.is_retryable());
    }

    #[test]
    fn invalid_signature_is_not_retryable() {
        let err = WebhookError::InvalidSignature;
        assert!(!err.is_retryable());
    }

    #[test]
    fn timestamp_out_of_range_is_not_retryable() {
        let err = WebhookError::TimestampOutOfRange;
        assert!(!err.is_retryable());
    }

    #[test]
    fn parse_error_is_not_retryable() {
        let err = WebhookError::ParseError("bad json".to_string());
        assert!(!err.is_retryable());
    }

    #[test]
    fn missing_metadata_is_not_retryable() {
        let err = WebhookError::MissingMetadata("user_id");
        assert!(!err.is_retryable());
    }

    #[test]
    fn ignored_is_not_retryable() {
        let err = WebhookError::Ignored("already processed".to_string());
        assert!(!err.is_retryable());
    }

    #[test]
    fn invalid_transition_is_not_retryable() {
        let err = WebhookError::InvalidTransition("bad state".to_string());
        assert!(!err.is_retryable());
    }

    // ══════════════════════════════════════════════════════════════
    // Status Code Tests
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn invalid_signature_returns_unauthorized() {
        let err = WebhookError::InvalidSignature;
        assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn timestamp_out_of_range_returns_unauthorized() {
        let err = WebhookError::TimestampOutOfRange;
        assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn invalid_timestamp_returns_bad_request() {
        let err = WebhookError::InvalidTimestamp;
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn parse_error_returns_bad_request() {
        let err = WebhookError::ParseError("syntax error".to_string());
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn missing_metadata_returns_bad_request() {
        let err = WebhookError::MissingMetadata("field");
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn missing_field_returns_bad_request() {
        let err = WebhookError::MissingField("data");
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn ignored_returns_ok() {
        // Ignored events should be acknowledged to prevent retries
        let err = WebhookError::Ignored("not relevant".to_string());
        assert_eq!(err.status_code(), StatusCode::OK);
    }

    #[test]
    fn membership_not_found_returns_internal_error() {
        let err = WebhookError::MembershipNotFound;
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn invalid_transition_returns_internal_error() {
        let err = WebhookError::InvalidTransition("bad".to_string());
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn database_error_returns_internal_error() {
        let err = WebhookError::Database("connection lost".to_string());
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn storage_error_returns_internal_error() {
        let err = WebhookError::StorageError("cache miss".to_string());
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
