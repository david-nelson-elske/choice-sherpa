//! HTTP DTOs for cycle endpoints.
//!
//! These types decouple the HTTP API from domain types, allowing independent evolution.

use serde::{Deserialize, Serialize};

use crate::domain::foundation::ComponentType;

// ════════════════════════════════════════════════════════════════════════════════
// Request DTOs
// ════════════════════════════════════════════════════════════════════════════════

/// Request to create a new cycle.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCycleRequest {
    pub session_id: String,
}

/// Request to branch a cycle.
#[derive(Debug, Clone, Deserialize)]
pub struct BranchCycleRequest {
    pub branch_point: ComponentType,
    #[serde(default)]
    pub branch_label: Option<String>,
}

// ════════════════════════════════════════════════════════════════════════════════
// Response DTOs
// ════════════════════════════════════════════════════════════════════════════════

/// Response for cycle command operations.
#[derive(Debug, Clone, Serialize)]
pub struct CycleCommandResponse {
    pub cycle_id: String,
    pub message: String,
}

/// Standard error response.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            code: "BAD_REQUEST".to_string(),
            message: message.into(),
            details: None,
        }
    }

    pub fn not_found(resource_type: &str, id: &str) -> Self {
        Self {
            code: "NOT_FOUND".to_string(),
            message: format!("{} not found: {}", resource_type, id),
            details: None,
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            code: "FORBIDDEN".to_string(),
            message: message.into(),
            details: None,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: "INTERNAL_ERROR".to_string(),
            message: message.into(),
            details: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_response_bad_request_creates_correctly() {
        let error = ErrorResponse::bad_request("Invalid input");
        assert_eq!(error.code, "BAD_REQUEST");
        assert_eq!(error.message, "Invalid input");
    }

    #[test]
    fn error_response_not_found_creates_correctly() {
        let error = ErrorResponse::not_found("Cycle", "abc-123");
        assert_eq!(error.code, "NOT_FOUND");
        assert!(error.message.contains("Cycle"));
        assert!(error.message.contains("abc-123"));
    }

    #[test]
    fn create_cycle_request_deserializes() {
        let json = r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000"}"#;
        let request: CreateCycleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.session_id, "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn cycle_command_response_serializes() {
        let response = CycleCommandResponse {
            cycle_id: "abc-123".to_string(),
            message: "Created".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("abc-123"));
        assert!(json.contains("Created"));
    }
}
