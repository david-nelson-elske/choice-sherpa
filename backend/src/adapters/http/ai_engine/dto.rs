//! HTTP DTOs for AI Engine endpoints
//!
//! These types decouple the HTTP API from domain types, allowing independent evolution.

use serde::{Deserialize, Serialize};

use crate::domain::foundation::ComponentType;

// ════════════════════════════════════════════════════════════════════════════════
// Request DTOs
// ════════════════════════════════════════════════════════════════════════════════

/// Request to start a new AI conversation
#[derive(Debug, Clone, Deserialize)]
pub struct StartConversationRequest {
    pub session_id: String,
    pub cycle_id: String,
    #[serde(default = "default_initial_component")]
    pub initial_component: ComponentType,
}

fn default_initial_component() -> ComponentType {
    ComponentType::IssueRaising
}

/// Request to send a message in a conversation
#[derive(Debug, Clone, Deserialize)]
pub struct SendMessageRequest {
    pub message: String,
}

// ════════════════════════════════════════════════════════════════════════════════
// Response DTOs
// ════════════════════════════════════════════════════════════════════════════════

/// Response for starting a conversation
#[derive(Debug, Clone, Serialize)]
pub struct StartConversationResponse {
    pub cycle_id: String,
    pub current_step: ComponentType,
    pub status: String,
}

/// Response for sending a message
#[derive(Debug, Clone, Serialize)]
pub struct SendMessageResponse {
    pub response: String,
    pub current_step: ComponentType,
    pub turn_count: u32,
}

/// Response for getting conversation state
#[derive(Debug, Clone, Serialize)]
pub struct ConversationStateResponse {
    pub cycle_id: String,
    pub session_id: String,
    pub current_step: ComponentType,
    pub status: String,
    pub message_count: usize,
    pub completed_steps: Vec<ComponentType>,
}

/// Response for successful delete
#[derive(Debug, Clone, Serialize)]
pub struct DeleteConversationResponse {
    pub message: String,
}

/// Standard error response
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

    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            code: "CONFLICT".to_string(),
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
    fn test_start_conversation_request_default_component() {
        let json = r#"{"session_id":"sess_123","cycle_id":"cycle_456"}"#;
        let req: StartConversationRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.initial_component, ComponentType::IssueRaising);
    }

    #[test]
    fn test_start_conversation_request_custom_component() {
        let json = r#"{"session_id":"sess_123","cycle_id":"cycle_456","initial_component":"problem_frame"}"#;
        let req: StartConversationRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.initial_component, ComponentType::ProblemFrame);
    }

    #[test]
    fn test_send_message_request_deserialization() {
        let json = r#"{"message":"Hello, AI!"}"#;
        let req: SendMessageRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.message, "Hello, AI!");
    }

    #[test]
    fn test_error_response_serialization() {
        let error = ErrorResponse::not_found("Conversation", "cycle_123");
        let json = serde_json::to_string(&error).unwrap();

        assert!(json.contains("NOT_FOUND"));
        assert!(json.contains("Conversation not found"));
    }
}
