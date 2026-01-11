//! HTTP DTOs for session endpoints.
//!
//! These types decouple the HTTP API from domain types, allowing independent evolution.

use serde::{Deserialize, Serialize};

use crate::domain::foundation::{SessionStatus, Timestamp};
use crate::ports::{SessionList as DomainSessionList, SessionSummary as DomainSessionSummary, SessionView as DomainSessionView};

// ════════════════════════════════════════════════════════════════════════════
// Request DTOs
// ════════════════════════════════════════════════════════════════════════════

/// Request to create a new session.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSessionRequest {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Request to rename a session.
#[derive(Debug, Clone, Deserialize)]
pub struct RenameSessionRequest {
    pub title: String,
}

/// Request to update session description.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDescriptionRequest {
    pub description: Option<String>,
}

/// Query parameters for listing sessions.
#[derive(Debug, Clone, Deserialize)]
pub struct ListSessionsQuery {
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub per_page: Option<u32>,
    #[serde(default)]
    pub status: Option<SessionStatus>,
    #[serde(default)]
    pub include_archived: bool,
}

// ════════════════════════════════════════════════════════════════════════════
// Response DTOs
// ════════════════════════════════════════════════════════════════════════════

/// Response for session command operations.
#[derive(Debug, Clone, Serialize)]
pub struct SessionCommandResponse {
    pub session_id: String,
    pub message: String,
}

/// Detailed session view for API responses.
#[derive(Debug, Clone, Serialize)]
pub struct SessionResponse {
    pub id: String,
    pub user_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: SessionStatus,
    pub cycle_count: u32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<DomainSessionView> for SessionResponse {
    fn from(view: DomainSessionView) -> Self {
        Self {
            id: view.id.to_string(),
            user_id: view.user_id.to_string(),
            title: view.title,
            description: view.description,
            status: view.status,
            cycle_count: view.cycle_count,
            created_at: view.created_at.as_datetime().to_rfc3339(),
            updated_at: view.updated_at.as_datetime().to_rfc3339(),
        }
    }
}

/// Session summary for list responses.
#[derive(Debug, Clone, Serialize)]
pub struct SessionSummaryResponse {
    pub id: String,
    pub title: String,
    pub status: SessionStatus,
    pub cycle_count: u32,
    pub updated_at: String,
}

impl From<DomainSessionSummary> for SessionSummaryResponse {
    fn from(summary: DomainSessionSummary) -> Self {
        Self {
            id: summary.id.to_string(),
            title: summary.title,
            status: summary.status,
            cycle_count: summary.cycle_count,
            updated_at: summary.updated_at.as_datetime().to_rfc3339(),
        }
    }
}

/// Paginated list of sessions.
#[derive(Debug, Clone, Serialize)]
pub struct SessionListResponse {
    pub items: Vec<SessionSummaryResponse>,
    pub total: u64,
    pub has_more: bool,
}

impl From<DomainSessionList> for SessionListResponse {
    fn from(list: DomainSessionList) -> Self {
        Self {
            items: list.items.into_iter().map(Into::into).collect(),
            total: list.total,
            has_more: list.has_more,
        }
    }
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
    use crate::domain::foundation::{SessionId, UserId};

    #[test]
    fn create_session_request_deserializes() {
        let json = r#"{"title": "My Decision"}"#;
        let req: CreateSessionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.title, "My Decision");
        assert!(req.description.is_none());
    }

    #[test]
    fn create_session_request_with_description_deserializes() {
        let json = r#"{"title": "My Decision", "description": "Important choice"}"#;
        let req: CreateSessionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.title, "My Decision");
        assert_eq!(req.description, Some("Important choice".to_string()));
    }

    #[test]
    fn session_response_conversion() {
        let view = DomainSessionView {
            id: SessionId::new(),
            user_id: UserId::new("user-123").unwrap(),
            title: "Test Session".to_string(),
            description: Some("Test description".to_string()),
            status: SessionStatus::Active,
            cycle_count: 2,
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };

        let response: SessionResponse = view.into();
        assert_eq!(response.title, "Test Session");
        assert_eq!(response.description, Some("Test description".to_string()));
        assert_eq!(response.cycle_count, 2);
    }

    #[test]
    fn error_response_bad_request_creates_correctly() {
        let error = ErrorResponse::bad_request("Invalid input");
        assert_eq!(error.code, "BAD_REQUEST");
        assert_eq!(error.message, "Invalid input");
    }

    #[test]
    fn error_response_not_found_creates_correctly() {
        let error = ErrorResponse::not_found("Session", "abc-123");
        assert_eq!(error.code, "NOT_FOUND");
        assert!(error.message.contains("Session"));
        assert!(error.message.contains("abc-123"));
    }
}
