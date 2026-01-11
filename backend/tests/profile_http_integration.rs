//! Integration tests for profile HTTP endpoints.
//!
//! These tests verify the HTTP layer wiring for profile operations:
//! 1. Request DTOs deserialize correctly
//! 2. Response DTOs serialize correctly
//! 3. Handlers can be created and wired together

use serde_json::json;
use std::sync::Arc;

use choice_sherpa::adapters::http::profile::ProfileHandlers;
use choice_sherpa::application::handlers::user::{
    CreateProfileHandler, DeleteProfileHandler, GetAgentInstructionsHandler,
    GetProfileSummaryHandler, RecordOutcomeHandler, UpdateProfileFromDecisionHandler,
};
use choice_sherpa::domain::foundation::{DomainError, ErrorCode, Timestamp, UserId};
use choice_sherpa::domain::user::{
    DecisionDomain, DecisionProfile, DecisionProfileId, ProfileConfidence, ProfileConsent,
    RiskClassification, StyleClassification,
};
use choice_sherpa::ports::{
    AgentInstructions, AnalysisResult, ConversationSummary, DecisionAnalysisData,
    ProfileAnalyzer, ProfileReader, ProfileRepository, ProfileSummary,
};

use async_trait::async_trait;
use std::sync::Mutex;

// =============================================================================
// Test Infrastructure
// =============================================================================

/// Mock profile repository for testing
struct MockProfileRepository {
    profiles: Mutex<Vec<DecisionProfile>>,
}

impl MockProfileRepository {
    fn new() -> Self {
        Self {
            profiles: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl ProfileRepository for MockProfileRepository {
    async fn create(&self, profile: &DecisionProfile) -> Result<(), DomainError> {
        self.profiles.lock().unwrap().push(profile.clone());
        Ok(())
    }

    async fn update(&self, profile: &DecisionProfile) -> Result<(), DomainError> {
        let mut profiles = self.profiles.lock().unwrap();
        if let Some(pos) = profiles.iter().position(|p| p.id() == profile.id()) {
            profiles[pos] = profile.clone();
            Ok(())
        } else {
            Err(DomainError::new(ErrorCode::NotFound, "Profile not found"))
        }
    }

    async fn find_by_user(
        &self,
        user_id: &UserId,
    ) -> Result<Option<DecisionProfile>, DomainError> {
        Ok(self
            .profiles
            .lock()
            .unwrap()
            .iter()
            .find(|p| p.user_id() == user_id)
            .cloned())
    }

    async fn delete(&self, profile_id: DecisionProfileId) -> Result<(), DomainError> {
        let mut profiles = self.profiles.lock().unwrap();
        if let Some(pos) = profiles.iter().position(|p| p.id() == profile_id) {
            profiles.remove(pos);
            Ok(())
        } else {
            Err(DomainError::new(ErrorCode::NotFound, "Profile not found"))
        }
    }

    async fn find_by_id(
        &self,
        profile_id: DecisionProfileId,
    ) -> Result<Option<DecisionProfile>, DomainError> {
        Ok(self
            .profiles
            .lock()
            .unwrap()
            .iter()
            .find(|p| p.id() == profile_id)
            .cloned())
    }

    async fn export(
        &self,
        _profile_id: DecisionProfileId,
        _format: choice_sherpa::ports::ExportFormat,
    ) -> Result<Vec<u8>, DomainError> {
        unimplemented!()
    }

    async fn exists_for_user(&self, user_id: &UserId) -> Result<bool, DomainError> {
        Ok(self
            .profiles
            .lock()
            .unwrap()
            .iter()
            .any(|p| p.user_id() == user_id))
    }
}

/// Mock profile reader for testing
struct MockProfileReader;

#[async_trait]
impl ProfileReader for MockProfileReader {
    async fn get_summary(&self, _user_id: &UserId) -> Result<Option<ProfileSummary>, DomainError> {
        Ok(None)
    }

    async fn get_agent_instructions(
        &self,
        _user_id: &UserId,
        _domain: Option<DecisionDomain>,
    ) -> Result<Option<AgentInstructions>, DomainError> {
        Ok(None)
    }

    async fn get_decision_history(
        &self,
        _user_id: &UserId,
        _limit: usize,
        _offset: usize,
    ) -> Result<Vec<choice_sherpa::domain::user::DecisionRecord>, DomainError> {
        Ok(vec![])
    }

    async fn get_decisions_by_domain(
        &self,
        _user_id: &UserId,
        _domain: DecisionDomain,
    ) -> Result<Vec<choice_sherpa::domain::user::DecisionRecord>, DomainError> {
        Ok(vec![])
    }
}

/// Mock profile analyzer for testing
struct MockProfileAnalyzer;

#[async_trait]
impl ProfileAnalyzer for MockProfileAnalyzer {
    async fn analyze_decision(
        &self,
        _profile: &mut DecisionProfile,
        _data: &DecisionAnalysisData,
    ) -> Result<AnalysisResult, DomainError> {
        Ok(AnalysisResult {
            risk_profile_changed: true,
            new_patterns_detected: vec!["Test pattern".to_string()],
            blind_spots_identified: vec![],
            growth_observed: vec![],
        })
    }

    fn recalculate_risk_profile(
        &self,
        _history: &choice_sherpa::domain::user::DecisionHistory,
    ) -> Result<choice_sherpa::domain::user::RiskProfile, DomainError> {
        unimplemented!()
    }

    fn detect_cognitive_patterns(
        &self,
        _history: &choice_sherpa::domain::user::DecisionHistory,
        _conversations: &[ConversationSummary],
    ) -> Result<Vec<choice_sherpa::domain::user::CognitivePattern>, DomainError> {
        unimplemented!()
    }

    fn identify_blind_spots(
        &self,
        _profile: &DecisionProfile,
    ) -> Result<Vec<choice_sherpa::domain::user::BlindSpot>, DomainError> {
        unimplemented!()
    }

    fn generate_agent_instructions(
        &self,
        _profile: &DecisionProfile,
    ) -> Result<AgentInstructions, DomainError> {
        unimplemented!()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[test]
fn test_handler_wiring() {
    // Verify all handlers can be created and wired together
    let repository = Arc::new(MockProfileRepository::new());
    let reader = Arc::new(MockProfileReader);
    let analyzer = Arc::new(MockProfileAnalyzer);

    let create_handler = Arc::new(CreateProfileHandler::new(repository.clone()));
    let delete_handler = Arc::new(DeleteProfileHandler::new(repository.clone()));
    let get_summary_handler = Arc::new(GetProfileSummaryHandler::new(reader.clone()));
    let get_instructions_handler = Arc::new(GetAgentInstructionsHandler::new(reader));
    let record_outcome_handler = Arc::new(RecordOutcomeHandler::new(repository.clone()));
    let update_from_decision_handler =
        Arc::new(UpdateProfileFromDecisionHandler::new(repository, analyzer));

    let _handlers = ProfileHandlers::new(
        create_handler,
        delete_handler,
        get_summary_handler,
        get_instructions_handler,
        record_outcome_handler,
        update_from_decision_handler,
    );

    // If we get here, the wiring is correct
}

#[test]
fn test_create_profile_request_deserializes() {
    // Verify request DTO deserializes correctly
    let json = json!({
        "collection_enabled": true,
        "analysis_enabled": true,
        "agent_access_enabled": true
    });

    let json_str = serde_json::to_string(&json).unwrap();
    let req: choice_sherpa::adapters::http::profile::CreateProfileRequest =
        serde_json::from_str(&json_str).unwrap();

    assert!(req.collection_enabled);
    assert!(req.analysis_enabled);
    assert!(req.agent_access_enabled);
}

#[test]
fn test_delete_profile_request_deserializes() {
    // Verify delete request DTO with confirmation string
    let json = json!({
        "confirmation": "DELETE MY PROFILE"
    });

    let json_str = serde_json::to_string(&json).unwrap();
    let req: choice_sherpa::adapters::http::profile::DeleteProfileRequest =
        serde_json::from_str(&json_str).unwrap();

    assert_eq!(req.confirmation, "DELETE MY PROFILE");
}

#[test]
fn test_record_outcome_request_deserializes() {
    use choice_sherpa::domain::user::SatisfactionLevel;

    let json = json!({
        "cycle_id": "01234567-89ab-cdef-0123-456789abcdef",
        "satisfaction": "satisfied",
        "actual_consequences": "Everything went well",
        "surprises": ["Unexpected benefit"],
        "would_decide_same": true
    });

    let json_str = serde_json::to_string(&json).unwrap();
    let req: choice_sherpa::adapters::http::profile::RecordOutcomeRequest =
        serde_json::from_str(&json_str).unwrap();

    assert_eq!(req.actual_consequences, "Everything went well");
    assert_eq!(req.surprises.len(), 1);
    assert!(req.would_decide_same);
}

#[test]
fn test_profile_summary_response_serializes() {
    // Verify response DTO serializes correctly
    let summary = ProfileSummary {
        risk_classification: RiskClassification::RiskNeutral,
        risk_confidence: 0.75,
        decisions_analyzed: 5,
        profile_confidence: ProfileConfidence::Medium,
        top_values: vec!["Growth".to_string(), "Security".to_string()],
        decision_style: StyleClassification::AnalyticalCautious,
        active_blind_spots: vec!["Short-term focus".to_string()],
    };

    let response: choice_sherpa::adapters::http::profile::ProfileSummaryResponse = summary.into();
    let json = serde_json::to_value(&response).unwrap();

    assert_eq!(json["decisions_analyzed"], 5);
    assert_eq!(json["risk_confidence"], 0.75);
    assert_eq!(json["top_values"][0], "Growth");
    assert_eq!(json["active_blind_spots"][0], "Short-term focus");
}

#[test]
fn test_agent_instructions_response_serializes() {
    let instructions = AgentInstructions {
        risk_guidance: "Test guidance".to_string(),
        blind_spot_prompts: vec!["Prompt 1".to_string(), "Prompt 2".to_string()],
        communication_adjustments: vec!["Adjustment 1".to_string()],
        suggested_questions: vec!["Question 1?".to_string()],
    };

    let response: choice_sherpa::adapters::http::profile::AgentInstructionsResponse =
        instructions.into();
    let json = serde_json::to_value(&response).unwrap();

    assert_eq!(json["risk_guidance"], "Test guidance");
    assert_eq!(json["blind_spot_prompts"].as_array().unwrap().len(), 2);
    assert_eq!(json["suggested_questions"][0], "Question 1?");
}

#[test]
fn test_analysis_result_response_serializes() {
    let result = AnalysisResult {
        risk_profile_changed: true,
        new_patterns_detected: vec!["Pattern 1".to_string(), "Pattern 2".to_string()],
        blind_spots_identified: vec![],
        growth_observed: vec![],
    };

    let response: choice_sherpa::adapters::http::profile::AnalysisResultResponse = result.into();
    let json = serde_json::to_value(&response).unwrap();

    assert_eq!(json["risk_profile_changed"], true);
    assert_eq!(json["new_patterns_detected"].as_array().unwrap().len(), 2);
    assert_eq!(json["blind_spots_count"], 0);
    assert_eq!(json["growth_observed_count"], 0);
}
