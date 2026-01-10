//! HTTP DTOs for session endpoints.
//!
//! These types decouple the HTTP API from domain types.

use serde::{Deserialize, Serialize};

use crate::domain::foundation::{ComponentType, CycleStatus, Timestamp};
use crate::ports::{CycleSummary, CycleTreeNode};

// ════════════════════════════════════════════════════════════════════════════════
// Response DTOs
// ════════════════════════════════════════════════════════════════════════════════

/// Summary of a cycle for tree display.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Tree node representing a cycle and its branches.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Full cycle tree response with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleTreeResponse {
    /// Root cycle node with all branches.
    pub root: CycleTreeNodeResponse,
    /// Total number of cycles in the tree.
    pub total_cycles: u32,
    /// Maximum depth of branching.
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
                node.children
                    .iter()
                    .map(|c| max_depth(c, current + 1))
                    .max()
                    .unwrap_or(current)
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
    use crate::domain::foundation::CycleId;

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
    fn cycle_tree_response_handles_deep_nesting() {
        // Create a tree: root -> child1 -> child2 -> child3
        let child3 = CycleTreeNode {
            cycle: CycleSummary {
                id: CycleId::new(),
                is_branch: true,
                branch_point: Some(ComponentType::Tradeoffs),
                status: CycleStatus::Active,
                current_step: ComponentType::Recommendation,
                progress_percent: 87,
                created_at: Timestamp::now(),
            },
            children: vec![],
        };

        let child2 = CycleTreeNode {
            cycle: CycleSummary {
                id: CycleId::new(),
                is_branch: true,
                branch_point: Some(ComponentType::Consequences),
                status: CycleStatus::Active,
                current_step: ComponentType::Tradeoffs,
                progress_percent: 75,
                created_at: Timestamp::now(),
            },
            children: vec![child3],
        };

        let child1 = CycleTreeNode {
            cycle: CycleSummary {
                id: CycleId::new(),
                is_branch: true,
                branch_point: Some(ComponentType::Alternatives),
                status: CycleStatus::Active,
                current_step: ComponentType::Consequences,
                progress_percent: 50,
                created_at: Timestamp::now(),
            },
            children: vec![child2],
        };

        let root = CycleTreeNode {
            cycle: CycleSummary {
                id: CycleId::new(),
                is_branch: false,
                branch_point: None,
                status: CycleStatus::Active,
                current_step: ComponentType::Alternatives,
                progress_percent: 37,
                created_at: Timestamp::now(),
            },
            children: vec![child1],
        };

        let response = CycleTreeResponse::from(root);

        assert_eq!(response.total_cycles, 4);
        assert_eq!(response.max_depth, 3);
    }

    #[test]
    fn error_response_formats_correctly() {
        let error = ErrorResponse::not_found("Cycle tree", "abc-123");
        assert_eq!(error.code, "NOT_FOUND");
        assert!(error.message.contains("Cycle tree"));
        assert!(error.message.contains("abc-123"));
    }
}
