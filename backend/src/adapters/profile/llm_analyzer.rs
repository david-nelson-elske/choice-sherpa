//! AI-powered ProfileAnalyzer implementation

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{
    foundation::{DomainError, ErrorCode, Timestamp},
    user::{
        BlindSpot, CognitiveBiasType, CognitivePattern, DecisionHistory, DecisionProfile,
        RiskClassification, RiskProfile, SeverityLevel,
    },
};
use crate::ports::{
    AIProvider, AgentInstructions, AnalysisResult, CompletionRequest, DecisionAnalysisData,
    Message, MessageRole, ProfileAnalyzer, RequestMetadata,
};

/// AI-powered profile analyzer using LLM for pattern detection
pub struct LlmProfileAnalyzer {
    ai_provider: Arc<dyn AIProvider>,
}

impl LlmProfileAnalyzer {
    pub fn new(ai_provider: Arc<dyn AIProvider>) -> Self {
        Self { ai_provider }
    }

    /// Create dummy metadata for profile analysis (not tied to a specific conversation)
    fn create_analysis_metadata(&self, profile: &DecisionProfile) -> RequestMetadata {
        use crate::domain::foundation::{ConversationId, SessionId};

        RequestMetadata::new(
            profile.user_id().clone(),
            SessionId::new(), // Dummy session ID
            ConversationId::new(), // Dummy conversation ID
            format!("profile-analysis-{}", uuid::Uuid::new_v4()),
        )
    }

    /// Create a prompt for analyzing risk profile
    fn create_risk_analysis_prompt(&self, data: &DecisionAnalysisData) -> String {
        format!(
            r#"Analyze the following decision to assess risk tolerance.

Decision: {}
Domain: {:?}
Chosen Alternative: {}
Key Tradeoff: {}

Objectives: {}
Alternatives Considered: {}

Risk Indicators:
{}

Based on this information, provide a JSON response with:
{{
  "risk_classification": "risk_seeking" | "risk_neutral" | "risk_averse",
  "confidence": 0.0-1.0,
  "reasoning": "brief explanation"
}}

Consider:
- Did they choose high-variance or low-variance options?
- Did they ask about downsides or upsides more?
- How much information did they seek before deciding?
- Did they focus on potential gains or potential losses?"#,
            data.title,
            data.domain,
            data.chosen_alternative,
            data.key_tradeoff,
            data.objectives.join(", "),
            data.alternatives.join(", "),
            data.risk_indicators
                .iter()
                .map(|r| format!("- {}: {}", r.indicator_type, r.description))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// Create a prompt for detecting cognitive patterns
    fn create_bias_detection_prompt(&self, data: &DecisionAnalysisData) -> String {
        format!(
            r#"Analyze this decision for cognitive biases.

Decision: {}
Conversations: {} messages across {} components

Review the decision-making process and identify any cognitive biases present.

Provide a JSON array of detected biases:
[
  {{
    "bias_type": "anchoring" | "loss_aversion" | "status_quo_bias" | "confirmation_bias" | "overconfidence_bias" | "availability_bias" | "sunk_cost_fallacy" | "planning_fallacy",
    "severity": "mild" | "moderate" | "strong",
    "evidence": "specific evidence from the decision",
    "mitigation": "suggestion for the agent to address this"
  }}
]

Common patterns to look for:
- Anchoring: Fixating on first numbers mentioned
- Loss Aversion: Weighing losses more than equivalent gains
- Status Quo Bias: Preferring current state without clear reason
- Confirmation Bias: Only seeking information that confirms existing beliefs
- Overconfidence: Excessive certainty without sufficient evidence"#,
            data.title,
            data.conversations.iter().map(|c| c.message_count).sum::<usize>(),
            data.conversations.len()
        )
    }

    /// Create a prompt for identifying blind spots
    fn create_blind_spot_prompt(&self, profile: &DecisionProfile) -> String {
        format!(
            r#"Analyze this decision profile to identify blind spots.

Profile Summary:
- Risk Profile: {:?}
- Decisions Analyzed: {}
- Primary Style: {:?}

Based on this profile's patterns, identify potential blind spots.

Provide a JSON array:
[
  {{
    "name": "short name of blind spot",
    "description": "what the user tends to miss or underweight",
    "evidence": ["evidence 1", "evidence 2"],
    "agent_behavior": "what the agent should do to address this"
  }}
]

Common blind spots to consider:
- Underweighting long-term consequences
- Overconfidence in familiar domains
- Neglecting opportunity costs
- Ignoring stakeholder impacts
- Underestimating time requirements"#,
            profile.risk_profile().classification,
            profile.decisions_analyzed(),
            profile.decision_style().primary_style
        )
    }

    /// Parse risk classification from AI response
    fn parse_risk_classification(&self, response: &str) -> Result<(RiskClassification, f32), DomainError> {
        let parsed: serde_json::Value = serde_json::from_str(response)
            .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Failed to parse AI response: {}", e)))?;

        let classification_str = parsed["risk_classification"]
            .as_str()
            .ok_or_else(|| DomainError::new(ErrorCode::InternalError, "Missing risk_classification in response"))?;

        let classification = match classification_str {
            "risk_seeking" => RiskClassification::RiskSeeking,
            "risk_neutral" => RiskClassification::RiskNeutral,
            "risk_averse" => RiskClassification::RiskAverse,
            _ => RiskClassification::RiskNeutral,
        };

        let confidence = parsed["confidence"]
            .as_f64()
            .unwrap_or(0.5) as f32;

        Ok((classification, confidence))
    }

    /// Parse cognitive biases from AI response
    fn parse_cognitive_patterns(&self, response: &str) -> Result<Vec<CognitivePattern>, DomainError> {
        let parsed: serde_json::Value = serde_json::from_str(response)
            .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Failed to parse AI response: {}", e)))?;

        let biases = parsed.as_array()
            .ok_or_else(|| DomainError::new(ErrorCode::InternalError, "Expected array of biases"))?;

        let mut patterns = Vec::new();

        for bias in biases {
            let bias_type_str = bias["bias_type"].as_str().unwrap_or("confirmation_bias");
            let severity_str = bias["severity"].as_str().unwrap_or("mild");
            let evidence = bias["evidence"].as_str().unwrap_or("").to_string();
            let mitigation = bias["mitigation"].as_str().unwrap_or("").to_string();

            let bias_type = match bias_type_str {
                "anchoring" => CognitiveBiasType::Anchoring,
                "loss_aversion" => CognitiveBiasType::LossAversion,
                "status_quo_bias" => CognitiveBiasType::StatusQuoBias,
                "confirmation_bias" => CognitiveBiasType::ConfirmationBias,
                "overconfidence_bias" => CognitiveBiasType::OverconfidenceBias,
                "availability_bias" => CognitiveBiasType::AvailabilityBias,
                "sunk_cost_fallacy" => CognitiveBiasType::SunkCostFallacy,
                "planning_fallacy" => CognitiveBiasType::PlanningFallacy,
                _ => continue,
            };

            let severity = match severity_str {
                "mild" => SeverityLevel::Mild,
                "moderate" => SeverityLevel::Moderate,
                "strong" => SeverityLevel::Strong,
                _ => SeverityLevel::Mild,
            };

            if let Ok(pattern) = CognitivePattern::new(bias_type, severity, evidence, mitigation) {
                patterns.push(pattern);
            }
        }

        Ok(patterns)
    }

    /// Parse blind spots from AI response
    fn parse_blind_spots(&self, response: &str, timestamp: Timestamp) -> Result<Vec<BlindSpot>, DomainError> {
        let parsed: serde_json::Value = serde_json::from_str(response)
            .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Failed to parse AI response: {}", e)))?;

        let spots = parsed.as_array()
            .ok_or_else(|| DomainError::new(ErrorCode::InternalError, "Expected array of blind spots"))?;

        let mut blind_spots = Vec::new();

        for spot in spots {
            let name = spot["name"].as_str().unwrap_or("").to_string();
            let description = spot["description"].as_str().unwrap_or("").to_string();
            let agent_behavior = spot["agent_behavior"].as_str().unwrap_or("").to_string();

            let evidence: Vec<String> = spot["evidence"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            if let Ok(blind_spot) = BlindSpot::new(name, description, evidence, agent_behavior, timestamp) {
                blind_spots.push(blind_spot);
            }
        }

        Ok(blind_spots)
    }
}

#[async_trait]
impl ProfileAnalyzer for LlmProfileAnalyzer {
    async fn analyze_decision(
        &self,
        profile: &mut DecisionProfile,
        decision_data: &DecisionAnalysisData,
    ) -> Result<AnalysisResult, DomainError> {
        let old_classification = profile.risk_profile().classification;

        // Analyze risk profile
        let risk_prompt = self.create_risk_analysis_prompt(decision_data);
        let metadata = self.create_analysis_metadata(profile);
        let risk_request = CompletionRequest::new(metadata.clone())
            .with_message(MessageRole::User, risk_prompt)
            .with_temperature(0.3)
            .with_max_tokens(500);

        let risk_response = self.ai_provider.complete(risk_request).await
            .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("AI provider error: {}", e)))?;

        let (new_classification, confidence) = self.parse_risk_classification(&risk_response.content)?;

        // Detect cognitive biases
        let bias_prompt = self.create_bias_detection_prompt(decision_data);
        let bias_request = CompletionRequest::new(metadata.clone())
            .with_message(MessageRole::User, bias_prompt)
            .with_temperature(0.3)
            .with_max_tokens(1000);

        let bias_response = self.ai_provider.complete(bias_request).await
            .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("AI provider error: {}", e)))?;

        let cognitive_patterns = self.parse_cognitive_patterns(&bias_response.content)?;

        // Identify blind spots
        let blind_spot_prompt = self.create_blind_spot_prompt(profile);
        let blind_spot_request = CompletionRequest::new(metadata)
            .with_message(MessageRole::User, blind_spot_prompt)
            .with_temperature(0.3)
            .with_max_tokens(1000);

        let blind_spot_response = self.ai_provider.complete(blind_spot_request).await
            .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("AI provider error: {}", e)))?;

        let blind_spots = self.parse_blind_spots(&blind_spot_response.content, Timestamp::now())?;

        // Update profile components
        let mut updated_style = profile.decision_style().clone();
        updated_style.cognitive_patterns.extend(cognitive_patterns);

        let mut updated_blind_spots = profile.blind_spots_growth().clone();
        updated_blind_spots.blind_spots.extend(blind_spots.clone());

        // Create updated risk profile
        let mut updated_risk = profile.risk_profile().clone();
        updated_risk.classification = new_classification;
        updated_risk.confidence = confidence;
        updated_risk.updated_at = Timestamp::now();

        profile.update_from_analysis(
            updated_risk,
            profile.values_priorities().clone(),
            updated_style,
            updated_blind_spots,
            profile.communication_prefs().clone(),
            profile.decision_history().clone(),
            Timestamp::now(),
        );

        Ok(AnalysisResult {
            risk_profile_changed: old_classification != new_classification,
            new_patterns_detected: vec![format!("Analyzed decision: {}", decision_data.title)],
            blind_spots_identified: blind_spots.clone(),
            growth_observed: vec![],
        })
    }

    fn recalculate_risk_profile(&self, _history: &DecisionHistory) -> Result<RiskProfile, DomainError> {
        // This would aggregate risk indicators across multiple decisions
        // For now, return a default profile
        Ok(RiskProfile::default())
    }

    fn detect_cognitive_patterns(
        &self,
        _history: &DecisionHistory,
        _conversations: &[crate::ports::ConversationSummary],
    ) -> Result<Vec<CognitivePattern>, DomainError> {
        // This would analyze patterns across multiple decisions
        Ok(Vec::new())
    }

    fn identify_blind_spots(&self, _profile: &DecisionProfile) -> Result<Vec<BlindSpot>, DomainError> {
        // This is handled by analyze_decision
        Ok(Vec::new())
    }

    fn generate_agent_instructions(&self, profile: &DecisionProfile) -> Result<AgentInstructions, DomainError> {
        // Generate instructions based on profile data
        let risk_guidance = match profile.risk_profile().classification {
            RiskClassification::RiskSeeking => {
                "User tends to seek high-variance options. Encourage consideration of downside scenarios.".to_string()
            }
            RiskClassification::RiskAverse => {
                "User prefers certainty and safety. Challenge risk-averse defaults when potential upside is significant.".to_string()
            }
            RiskClassification::RiskNeutral => {
                "User evaluates options on expected value. Provide balanced perspective on risk/reward.".to_string()
            }
        };

        let blind_spot_prompts: Vec<String> = profile
            .blind_spots_growth()
            .active_blind_spots()
            .iter()
            .map(|bs| bs.agent_behavior.clone())
            .collect();

        let communication_adjustments = vec![
            format!("Communication style: {:?}", profile.communication_prefs().interaction_style.challenge_style),
        ];

        let suggested_questions = vec![
            "What does this look like in 10 years?".to_string(),
            "What are you giving up by choosing this?".to_string(),
            "How would you decide if the stakes were 10x higher?".to_string(),
        ];

        Ok(AgentInstructions {
            risk_guidance,
            blind_spot_prompts,
            communication_adjustments,
            suggested_questions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::{MockAIProvider, MockResponse};
    use crate::domain::foundation::UserId;
    use crate::domain::user::{DecisionDomain, ProfileConsent};

    fn test_user_id() -> UserId {
        UserId::new("test@example.com".to_string()).unwrap()
    }

    fn test_profile() -> DecisionProfile {
        let consent = ProfileConsent::full(Timestamp::now());
        DecisionProfile::new(test_user_id(), consent, Timestamp::now()).unwrap()
    }

    fn test_decision_data() -> DecisionAnalysisData {
        DecisionAnalysisData {
            title: "Test Decision".to_string(),
            domain: DecisionDomain::Career,
            dq_score: Some(80),
            key_tradeoff: "Growth vs Stability".to_string(),
            chosen_alternative: "Take the new role".to_string(),
            objectives: vec!["Career growth".to_string(), "Work-life balance".to_string()],
            alternatives: vec!["Stay".to_string(), "Leave".to_string()],
            conversations: vec![],
            risk_indicators: vec![],
        }
    }

    #[tokio::test]
    async fn test_analyze_decision_with_mock() {
        let mock_ai = Arc::new(
            MockAIProvider::new()
                .with_response(r#"{"risk_classification": "risk_averse", "confidence": 0.75, "reasoning": "Chose stable option"}"#)
                .with_response("[]")
                .with_response("[]")
        );

        let analyzer = LlmProfileAnalyzer::new(mock_ai);
        let mut profile = test_profile();
        let data = test_decision_data();

        let result = analyzer.analyze_decision(&mut profile, &data).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_risk_classification() {
        let analyzer = LlmProfileAnalyzer::new(Arc::new(MockAIProvider::default()));

        let response = r#"{"risk_classification": "risk_averse", "confidence": 0.8}"#;
        let (classification, confidence) = analyzer.parse_risk_classification(response).unwrap();

        assert_eq!(classification, RiskClassification::RiskAverse);
        assert_eq!(confidence, 0.8);
    }

    #[test]
    fn test_parse_cognitive_patterns() {
        let analyzer = LlmProfileAnalyzer::new(Arc::new(MockAIProvider::default()));

        let response = r#"[
            {
                "bias_type": "anchoring",
                "severity": "moderate",
                "evidence": "Fixated on initial salary number",
                "mitigation": "Introduce alternative reference points"
            }
        ]"#;

        let patterns = analyzer.parse_cognitive_patterns(response).unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].bias_type, CognitiveBiasType::Anchoring);
    }

    #[test]
    fn test_generate_agent_instructions() {
        let analyzer = LlmProfileAnalyzer::new(Arc::new(MockAIProvider::default()));
        let profile = test_profile();

        let instructions = analyzer.generate_agent_instructions(&profile).unwrap();
        assert!(!instructions.risk_guidance.is_empty());
        assert!(!instructions.suggested_questions.is_empty());
    }
}
