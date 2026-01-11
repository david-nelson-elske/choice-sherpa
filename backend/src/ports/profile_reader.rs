//! ProfileReader port for profile query operations

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::{
    foundation::{DomainError, UserId},
    user::{
        DecisionDomain, DecisionRecord, ProfileConfidence, RiskClassification,
        StyleClassification,
    },
};

/// Lightweight profile summary for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSummary {
    /// Overall risk classification
    pub risk_classification: RiskClassification,
    /// Confidence in risk assessment
    pub risk_confidence: f32,
    /// Number of decisions analyzed
    pub decisions_analyzed: u32,
    /// Profile confidence level
    pub profile_confidence: ProfileConfidence,
    /// Top 5 core values
    pub top_values: Vec<String>,
    /// Primary decision-making style
    pub decision_style: StyleClassification,
    /// Active blind spots
    pub active_blind_spots: Vec<String>,
}

/// Agent behavior instructions derived from profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInstructions {
    /// Risk-related guidance
    pub risk_guidance: String,
    /// Prompts to address blind spots
    pub blind_spot_prompts: Vec<String>,
    /// Communication style adjustments
    pub communication_adjustments: Vec<String>,
    /// Suggested questions to ask
    pub suggested_questions: Vec<String>,
}

/// Query operations for decision profiles
#[async_trait]
pub trait ProfileReader: Send + Sync {
    /// Get profile summary for UI
    async fn get_summary(&self, user_id: &UserId) -> Result<Option<ProfileSummary>, DomainError>;

    /// Get agent instructions for personalization
    async fn get_agent_instructions(
        &self,
        user_id: &UserId,
        domain: Option<DecisionDomain>,
    ) -> Result<Option<AgentInstructions>, DomainError>;

    /// Get decision history with pagination
    async fn get_decision_history(
        &self,
        user_id: &UserId,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<DecisionRecord>, DomainError>;

    /// Get decisions by domain
    async fn get_decisions_by_domain(
        &self,
        user_id: &UserId,
        domain: DecisionDomain,
    ) -> Result<Vec<DecisionRecord>, DomainError>;
}
