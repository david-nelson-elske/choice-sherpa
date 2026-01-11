//! HTTP DTOs for dashboard endpoints.
//!
//! Dashboard is read-only, so we only have response DTOs.
//! The domain view models are already designed for serialization,
//! so we re-export them directly.

pub use crate::domain::dashboard::{
    AlternativeSummary, CellColor, CellSummary, CompactConsequencesTable, ComparisonDifference,
    ComparisonSummary, ComponentComparisonSummary, ComponentDetailView, CycleComparison,
    CycleComparisonItem, CycleProgressSnapshot, DashboardOverview, DifferenceSignificance,
    ObjectiveSummary, RecommendationSummary,
};

use serde::Serialize;

// ════════════════════════════════════════════════════════════════════════════════
// Response DTOs
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

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            code: "UNAUTHORIZED".to_string(),
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
