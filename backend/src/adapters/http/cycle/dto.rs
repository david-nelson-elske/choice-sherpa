//! HTTP DTOs (Data Transfer Objects) for cycle endpoints.
//!
//! These types define the JSON request/response structure for the cycle API.
//! They serve as the boundary between HTTP and the application layer.

use crate::domain::foundation::{ComponentStatus, ComponentType, CycleStatus};
use crate::ports::{ComponentStatusItem, CycleSummary, CycleTreeNode, CycleView};
use serde::{Deserialize, Serialize};

// ════════════════════════════════════════════════════════════════════════════════
// Request DTOs
// ════════════════════════════════════════════════════════════════════════════════

/// Request to create a new cycle.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCycleRequest {
    /// The session to create the cycle in.
    pub session_id: String,
}

/// Request to branch a cycle.
#[derive(Debug, Clone, Deserialize)]
pub struct BranchCycleRequest {
    /// The component to branch at.
    pub branch_point: ComponentType,
}

/// Request to start a component.
#[derive(Debug, Clone, Deserialize)]
pub struct StartComponentRequest {
    /// The component type to start.
    pub component_type: ComponentType,
}

/// Request to complete a component.
#[derive(Debug, Clone, Deserialize)]
pub struct CompleteComponentRequest {
    /// The component type to complete.
    pub component_type: ComponentType,
}

/// Request to update component output.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateComponentOutputRequest {
    /// The component type to update.
    pub component_type: ComponentType,
    /// The output data as JSON.
    pub output: serde_json::Value,
}

/// Request to navigate to a component.
#[derive(Debug, Clone, Deserialize)]
pub struct NavigateComponentRequest {
    /// The target component.
    pub target: ComponentType,
}

// ════════════════════════════════════════════════════════════════════════════════
// Response DTOs
// ════════════════════════════════════════════════════════════════════════════════

/// Response for cycle details.
#[derive(Debug, Clone, Serialize)]
pub struct CycleResponse {
    /// Cycle ID.
    pub id: String,
    /// Session this cycle belongs to.
    pub session_id: String,
    /// Parent cycle ID if this is a branch.
    pub parent_cycle_id: Option<String>,
    /// Component where branching occurred.
    pub branch_point: Option<ComponentType>,
    /// Current cycle status.
    pub status: CycleStatus,
    /// Currently active component.
    pub current_step: ComponentType,
    /// Status of each component.
    pub component_statuses: Vec<ComponentStatusResponse>,
    /// Overall progress percentage (0-100).
    pub progress_percent: u8,
    /// Whether cycle is complete.
    pub is_complete: bool,
    /// Number of child branches.
    pub branch_count: u32,
    /// When the cycle was created (ISO 8601).
    pub created_at: String,
    /// When the cycle was last updated (ISO 8601).
    pub updated_at: String,
}

impl From<CycleView> for CycleResponse {
    fn from(view: CycleView) -> Self {
        Self {
            id: view.id.to_string(),
            session_id: view.session_id.to_string(),
            parent_cycle_id: view.parent_cycle_id.map(|id| id.to_string()),
            branch_point: view.branch_point,
            status: view.status,
            current_step: view.current_step,
            component_statuses: view
                .component_statuses
                .into_iter()
                .map(ComponentStatusResponse::from)
                .collect(),
            progress_percent: view.progress_percent,
            is_complete: view.is_complete,
            branch_count: view.branch_count,
            created_at: view.created_at.to_rfc3339(),
            updated_at: view.updated_at.to_rfc3339(),
        }
    }
}

/// Component status in response.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentStatusResponse {
    /// Component type.
    pub component_type: ComponentType,
    /// Current status.
    pub status: ComponentStatus,
    /// Whether this component is the current step.
    pub is_current: bool,
}

impl From<ComponentStatusItem> for ComponentStatusResponse {
    fn from(item: ComponentStatusItem) -> Self {
        Self {
            component_type: item.component_type,
            status: item.status,
            is_current: item.is_current,
        }
    }
}

/// Summary response for cycle lists.
#[derive(Debug, Clone, Serialize)]
pub struct CycleSummaryResponse {
    /// Cycle ID.
    pub id: String,
    /// Whether this is a branch.
    pub is_branch: bool,
    /// Branch point component (if a branch).
    pub branch_point: Option<ComponentType>,
    /// Current cycle status.
    pub status: CycleStatus,
    /// Currently active component.
    pub current_step: ComponentType,
    /// Overall progress percentage.
    pub progress_percent: u8,
    /// When the cycle was created (ISO 8601).
    pub created_at: String,
}

impl From<CycleSummary> for CycleSummaryResponse {
    fn from(summary: CycleSummary) -> Self {
        Self {
            id: summary.id.to_string(),
            is_branch: summary.is_branch,
            branch_point: summary.branch_point,
            status: summary.status,
            current_step: summary.current_step,
            progress_percent: summary.progress_percent,
            created_at: summary.created_at.to_rfc3339(),
        }
    }
}

/// Tree node response for cycle hierarchy.
#[derive(Debug, Clone, Serialize)]
pub struct CycleTreeResponse {
    /// Summary of this cycle.
    pub cycle: CycleSummaryResponse,
    /// Child branches.
    pub children: Vec<CycleTreeResponse>,
}

impl From<CycleTreeNode> for CycleTreeResponse {
    fn from(node: CycleTreeNode) -> Self {
        Self {
            cycle: CycleSummaryResponse::from(node.cycle),
            children: node.children.into_iter().map(CycleTreeResponse::from).collect(),
        }
    }
}

/// Response for component details.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentResponse {
    /// The cycle ID.
    pub cycle_id: String,
    /// The component type.
    pub component_type: ComponentType,
    /// The component status.
    pub status: ComponentStatus,
    /// The component output as JSON.
    pub output: serde_json::Value,
}

/// Response for command operations that create/modify cycles.
#[derive(Debug, Clone, Serialize)]
pub struct CycleCommandResponse {
    /// The cycle ID.
    pub cycle_id: String,
    /// Success message.
    pub message: String,
}

/// Response for component command operations.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentCommandResponse {
    /// The cycle ID.
    pub cycle_id: String,
    /// The component type.
    pub component_type: ComponentType,
    /// Success message.
    pub message: String,
}

/// Error response for API errors.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    /// Error code.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Additional error details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    pub fn not_found(resource: &str, id: &str) -> Self {
        Self {
            code: "NOT_FOUND".to_string(),
            message: format!("{} not found: {}", resource, id),
            details: None,
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            code: "BAD_REQUEST".to_string(),
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

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            code: "FORBIDDEN".to_string(),
            message: message.into(),
            details: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{CycleId, SessionId, Timestamp};

    #[test]
    fn cycle_response_serializes_to_json() {
        let view = CycleView {
            id: CycleId::new(),
            session_id: SessionId::new(),
            parent_cycle_id: None,
            branch_point: None,
            status: CycleStatus::Active,
            current_step: ComponentType::IssueRaising,
            component_statuses: vec![],
            progress_percent: 25,
            is_complete: false,
            branch_count: 0,
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };

        let response = CycleResponse::from(view);
        let json = serde_json::to_string(&response).expect("serialization failed");

        assert!(json.contains("\"status\":\"active\""));
        assert!(json.contains("\"progress_percent\":25"));
    }

    #[test]
    fn error_response_not_found_formats_correctly() {
        let err = ErrorResponse::not_found("Cycle", "abc-123");
        assert_eq!(err.code, "NOT_FOUND");
        assert!(err.message.contains("Cycle"));
        assert!(err.message.contains("abc-123"));
    }

    #[test]
    fn create_cycle_request_deserializes() {
        let json = r#"{"session_id": "test-session"}"#;
        let req: CreateCycleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.session_id, "test-session");
    }

    #[test]
    fn update_output_request_deserializes_with_json_output() {
        let json = r#"{
            "component_type": "issue_raising",
            "output": {"potential_decisions": ["Option A"], "objectives": []}
        }"#;
        let req: UpdateComponentOutputRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.component_type, ComponentType::IssueRaising);
        assert!(req.output["potential_decisions"].is_array());
    }
}
