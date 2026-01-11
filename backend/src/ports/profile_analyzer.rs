//! ProfileAnalyzer port for decision analysis and profile updates

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::{
    foundation::DomainError,
    user::{
        BlindSpot, CognitivePattern, DecisionHistory, DecisionProfile, GrowthObservation,
        RiskProfile,
    },
};

use super::profile_reader::AgentInstructions;

/// Summary of conversation for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub component_name: String,
    pub message_count: usize,
    pub key_phrases: Vec<String>,
    pub questions_asked: u32,
    pub decision_time_seconds: u64,
}

/// Result of analyzing a decision
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Whether risk profile classification changed
    pub risk_profile_changed: bool,
    /// New patterns detected
    pub new_patterns_detected: Vec<String>,
    /// Blind spots identified
    pub blind_spots_identified: Vec<BlindSpot>,
    /// Growth observed
    pub growth_observed: Vec<GrowthObservation>,
}

/// Analyzer for extracting patterns from decisions
#[async_trait]
pub trait ProfileAnalyzer: Send + Sync {
    /// Analyze a completed decision and update profile
    ///
    /// This is the main entry point for profile learning. It examines
    /// the decision data and conversation history to extract patterns.
    async fn analyze_decision(
        &self,
        profile: &mut DecisionProfile,
        decision_data: &DecisionAnalysisData,
    ) -> Result<AnalysisResult, DomainError>;

    /// Recalculate risk profile from decision history
    ///
    /// Uses choice analysis (40%), language patterns (25%),
    /// consequence ratings (20%), and information seeking (15%)
    fn recalculate_risk_profile(&self, history: &DecisionHistory) -> Result<RiskProfile, DomainError>;

    /// Detect cognitive patterns from decision history
    fn detect_cognitive_patterns(
        &self,
        history: &DecisionHistory,
        conversations: &[ConversationSummary],
    ) -> Result<Vec<CognitivePattern>, DomainError>;

    /// Identify blind spots in decision-making
    fn identify_blind_spots(&self, profile: &DecisionProfile) -> Result<Vec<BlindSpot>, DomainError>;

    /// Generate agent instructions from profile
    fn generate_agent_instructions(
        &self,
        profile: &DecisionProfile,
    ) -> Result<AgentInstructions, DomainError>;
}

/// Data needed for decision analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionAnalysisData {
    /// Decision title
    pub title: String,
    /// Decision domain
    pub domain: crate::domain::user::DecisionDomain,
    /// DQ score if available
    pub dq_score: Option<u8>,
    /// Key tradeoff identified
    pub key_tradeoff: String,
    /// Alternative that was chosen
    pub chosen_alternative: String,
    /// Objectives from the decision
    pub objectives: Vec<String>,
    /// Alternatives considered
    pub alternatives: Vec<String>,
    /// Conversation summaries
    pub conversations: Vec<ConversationSummary>,
    /// Risk indicators observed
    pub risk_indicators: Vec<RiskIndicator>,
}

/// Risk indicator extracted from behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskIndicator {
    pub indicator_type: String,
    pub description: String,
    pub weight: f32,
}
