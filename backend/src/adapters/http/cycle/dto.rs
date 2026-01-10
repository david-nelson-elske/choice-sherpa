//! HTTP DTOs for cycle endpoints.
//!
//! These types decouple the HTTP API from domain types, allowing independent evolution.

use serde::{Deserialize, Serialize};

use crate::domain::foundation::{ComponentStatus, ComponentType, CycleStatus, Timestamp};
use crate::ports::{ComponentStatusItem, CycleSummary, CycleTreeNode, CycleView};

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
}

/// Request to start a component.
#[derive(Debug, Clone, Deserialize)]
pub struct StartComponentRequest {
    pub component_type: ComponentType,
}

/// Request to complete a component.
#[derive(Debug, Clone, Deserialize)]
pub struct CompleteComponentRequest {
    pub component_type: ComponentType,
}

/// Request to update component output.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateComponentOutputRequest {
    pub component_type: ComponentType,
    pub output: serde_json::Value,
}

/// Request to navigate to a component.
#[derive(Debug, Clone, Deserialize)]
pub struct NavigateComponentRequest {
    pub target: ComponentType,
}

// ════════════════════════════════════════════════════════════════════════════════
// Response DTOs
// ════════════════════════════════════════════════════════════════════════════════

/// Detailed cycle response.
#[derive(Debug, Clone, Serialize)]
pub struct CycleResponse {
    pub id: String,
    pub session_id: String,
    pub parent_cycle_id: Option<String>,
    pub branch_point: Option<ComponentType>,
    pub status: CycleStatus,
    pub current_step: ComponentType,
    pub component_statuses: Vec<ComponentStatusResponse>,
    pub progress_percent: u8,
    pub is_complete: bool,
    pub branch_count: u32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
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
            component_statuses: view.component_statuses.into_iter().map(Into::into).collect(),
            progress_percent: view.progress_percent,
            is_complete: view.is_complete,
            branch_count: view.branch_count,
            created_at: view.created_at,
            updated_at: view.updated_at,
        }
    }
}

/// Component status in a cycle response.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentStatusResponse {
    pub component_type: ComponentType,
    pub status: ComponentStatus,
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

/// Summary cycle response for lists.
#[derive(Debug, Clone, Serialize)]
pub struct CycleSummaryResponse {
    pub id: String,
    pub is_branch: bool,
    pub branch_point: Option<ComponentType>,
    pub status: CycleStatus,
    pub current_step: ComponentType,
    pub progress_percent: u8,
    pub created_at: Timestamp,
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
            created_at: summary.created_at,
        }
    }
}

/// Cycle tree node response.
#[derive(Debug, Clone, Serialize)]
pub struct CycleTreeNodeResponse {
    pub cycle: CycleSummaryResponse,
    pub children: Vec<CycleTreeNodeResponse>,
}

impl From<CycleTreeNode> for CycleTreeNodeResponse {
    fn from(node: CycleTreeNode) -> Self {
        Self {
            cycle: node.cycle.into(),
            children: node.children.into_iter().map(Into::into).collect(),
        }
    }
}

/// Full tree response wrapper.
#[derive(Debug, Clone, Serialize)]
pub struct CycleTreeResponse {
    pub root: CycleTreeNodeResponse,
    pub total_cycles: u32,
    pub max_depth: u32,
}

impl From<CycleTreeNode> for CycleTreeResponse {
    fn from(root: CycleTreeNode) -> Self {
        fn count_nodes(node: &CycleTreeNode) -> u32 {
            1 + node.children.iter().map(count_nodes).sum::<u32>()
        }

        fn max_depth(node: &CycleTreeNode, current: u32) -> u32 {
            if node.children.is_empty() {
                current
            } else {
                node.children.iter().map(|c| max_depth(c, current + 1)).max().unwrap_or(current)
            }
        }

        let total_cycles = count_nodes(&root);
        let max_depth_val = max_depth(&root, 0);

        Self {
            root: root.into(),
            total_cycles,
            max_depth: max_depth_val,
        }
    }
}

/// Component detail response.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentResponse {
    pub cycle_id: String,
    pub component_type: ComponentType,
    pub status: ComponentStatus,
    pub output: Option<serde_json::Value>,
}

/// Response for cycle command operations.
#[derive(Debug, Clone, Serialize)]
pub struct CycleCommandResponse {
    pub cycle_id: String,
    pub message: String,
}

/// Response for component command operations.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentCommandResponse {
    pub cycle_id: String,
    pub component_type: ComponentType,
    pub message: String,
}

// ════════════════════════════════════════════════════════════════════════════════
// Document DTOs
// ════════════════════════════════════════════════════════════════════════════════

/// Query parameters for document generation.
#[derive(Debug, Clone, Deserialize)]
pub struct GetDocumentQuery {
    /// Document format: "full", "summary", or "export".
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "full".to_string()
}

/// Response containing generated document content.
#[derive(Debug, Clone, Serialize)]
pub struct DocumentResponse {
    /// The generated markdown content.
    pub content: String,
    /// The cycle ID.
    pub cycle_id: String,
    /// The session ID.
    pub session_id: String,
    /// The format used.
    pub format: String,
}

// ════════════════════════════════════════════════════════════════════════════════
// Error DTOs
// ════════════════════════════════════════════════════════════════════════════════

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
    use crate::domain::foundation::{CycleId, SessionId};

    #[test]
    fn cycle_response_converts_from_view() {
        let view = CycleView {
            id: CycleId::new(),
            session_id: SessionId::new(),
            parent_cycle_id: None,
            branch_point: None,
            status: CycleStatus::Active,
            current_step: ComponentType::IssueRaising,
            component_statuses: vec![ComponentStatusItem {
                component_type: ComponentType::IssueRaising,
                status: ComponentStatus::NotStarted,
                is_current: true,
            }],
            progress_percent: 0,
            is_complete: false,
            branch_count: 0,
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };

        let response = CycleResponse::from(view.clone());

        assert_eq!(response.id, view.id.to_string());
        assert_eq!(response.session_id, view.session_id.to_string());
        assert_eq!(response.status, CycleStatus::Active);
        assert_eq!(response.component_statuses.len(), 1);
    }

    #[test]
    fn cycle_tree_response_calculates_totals() {
        let child = CycleTreeNode {
            cycle: CycleSummary {
                id: CycleId::new(),
                is_branch: true,
                branch_point: Some(ComponentType::IssueRaising),
                status: CycleStatus::Active,
                current_step: ComponentType::ProblemFrame,
                progress_percent: 12,
                created_at: Timestamp::now(),
            },
            children: vec![],
        };

        let root = CycleTreeNode {
            cycle: CycleSummary {
                id: CycleId::new(),
                is_branch: false,
                branch_point: None,
                status: CycleStatus::Active,
                current_step: ComponentType::IssueRaising,
                progress_percent: 12,
                created_at: Timestamp::now(),
            },
            children: vec![child],
        };

        let response = CycleTreeResponse::from(root);

        assert_eq!(response.total_cycles, 2);
        assert_eq!(response.max_depth, 1);
    }

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
}
