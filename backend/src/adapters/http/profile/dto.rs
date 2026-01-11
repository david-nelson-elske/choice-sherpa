//! HTTP DTOs for profile endpoints.
//!
//! These types decouple the HTTP API from domain types, allowing independent evolution.

use serde::{Deserialize, Serialize};

use crate::domain::foundation::{CycleId, Timestamp};
use crate::domain::user::{
    DecisionDomain, ProfileConfidence, RiskClassification, SatisfactionLevel, StyleClassification,
};
use crate::ports::{
    AgentInstructions as DomainAgentInstructions, AnalysisResult as DomainAnalysisResult,
    DecisionAnalysisData, ProfileSummary as DomainProfileSummary,
};

// ════════════════════════════════════════════════════════════════════════════
// Request DTOs
// ════════════════════════════════════════════════════════════════════════════

/// Request to create a profile.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateProfileRequest {
    pub collection_enabled: bool,
    pub analysis_enabled: bool,
    pub agent_access_enabled: bool,
}

/// Request to update consent.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateConsentRequest {
    pub collection_enabled: bool,
    pub analysis_enabled: bool,
    pub agent_access_enabled: bool,
}

/// Request to record decision outcome.
#[derive(Debug, Clone, Deserialize)]
pub struct RecordOutcomeRequest {
    pub cycle_id: String,
    pub satisfaction: SatisfactionLevel,
    pub actual_consequences: String,
    pub surprises: Vec<String>,
    pub would_decide_same: bool,
}

/// Request to delete profile (requires confirmation).
#[derive(Debug, Clone, Deserialize)]
pub struct DeleteProfileRequest {
    pub confirmation: String,
}

/// Request to update profile from decision.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProfileFromDecisionRequest {
    pub title: String,
    pub domain: DecisionDomain,
    pub dq_score: Option<u8>,
    pub key_tradeoff: String,
    pub chosen_alternative: String,
    pub objectives: Vec<String>,
    pub alternatives: Vec<String>,
}

// ════════════════════════════════════════════════════════════════════════════
// Response DTOs
// ════════════════════════════════════════════════════════════════════════════

/// Response for profile command operations.
#[derive(Debug, Clone, Serialize)]
pub struct ProfileCommandResponse {
    pub profile_id: Option<String>,
    pub message: String,
}

/// Profile summary for UI display.
#[derive(Debug, Clone, Serialize)]
pub struct ProfileSummaryResponse {
    pub risk_classification: RiskClassification,
    pub risk_confidence: f32,
    pub decisions_analyzed: u32,
    pub profile_confidence: ProfileConfidence,
    pub top_values: Vec<String>,
    pub decision_style: StyleClassification,
    pub active_blind_spots: Vec<String>,
}

impl From<DomainProfileSummary> for ProfileSummaryResponse {
    fn from(summary: DomainProfileSummary) -> Self {
        Self {
            risk_classification: summary.risk_classification,
            risk_confidence: summary.risk_confidence,
            decisions_analyzed: summary.decisions_analyzed,
            profile_confidence: summary.profile_confidence,
            top_values: summary.top_values,
            decision_style: summary.decision_style,
            active_blind_spots: summary.active_blind_spots,
        }
    }
}

/// Agent instructions for personalizing AI behavior.
#[derive(Debug, Clone, Serialize)]
pub struct AgentInstructionsResponse {
    pub risk_guidance: String,
    pub blind_spot_prompts: Vec<String>,
    pub communication_adjustments: Vec<String>,
    pub suggested_questions: Vec<String>,
}

impl From<DomainAgentInstructions> for AgentInstructionsResponse {
    fn from(instructions: DomainAgentInstructions) -> Self {
        Self {
            risk_guidance: instructions.risk_guidance,
            blind_spot_prompts: instructions.blind_spot_prompts,
            communication_adjustments: instructions.communication_adjustments,
            suggested_questions: instructions.suggested_questions,
        }
    }
}

/// Analysis result from profile update.
#[derive(Debug, Clone, Serialize)]
pub struct AnalysisResultResponse {
    pub risk_profile_changed: bool,
    pub new_patterns_detected: Vec<String>,
    pub blind_spots_count: usize,
    pub growth_observed_count: usize,
}

impl From<DomainAnalysisResult> for AnalysisResultResponse {
    fn from(result: DomainAnalysisResult) -> Self {
        Self {
            risk_profile_changed: result.risk_profile_changed,
            new_patterns_detected: result.new_patterns_detected,
            blind_spots_count: result.blind_spots_identified.len(),
            growth_observed_count: result.growth_observed.len(),
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

    #[test]
    fn create_profile_request_deserializes() {
        let json = r#"{"collection_enabled": true, "analysis_enabled": true, "agent_access_enabled": true}"#;
        let req: CreateProfileRequest = serde_json::from_str(json).unwrap();
        assert!(req.collection_enabled);
        assert!(req.analysis_enabled);
        assert!(req.agent_access_enabled);
    }

    #[test]
    fn delete_profile_request_deserializes() {
        let json = r#"{"confirmation": "DELETE MY PROFILE"}"#;
        let req: DeleteProfileRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.confirmation, "DELETE MY PROFILE");
    }

    #[test]
    fn error_response_bad_request_creates_correctly() {
        let error = ErrorResponse::bad_request("Invalid input");
        assert_eq!(error.code, "BAD_REQUEST");
        assert_eq!(error.message, "Invalid input");
    }

    #[test]
    fn error_response_not_found_creates_correctly() {
        let error = ErrorResponse::not_found("Profile", "user-123");
        assert_eq!(error.code, "NOT_FOUND");
        assert!(error.message.contains("Profile"));
        assert!(error.message.contains("user-123"));
    }
}
