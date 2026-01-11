//! UpdateProfileFromDecision - Command handler for updating profile after decision completion.

use std::sync::Arc;

use crate::domain::foundation::{CommandMetadata, DomainError, ErrorCode, Timestamp, UserId};
use crate::ports::{
    AnalysisResult, DecisionAnalysisData, ProfileAnalyzer, ProfileRepository,
};

/// Command to update profile based on completed decision.
#[derive(Debug, Clone)]
pub struct UpdateProfileFromDecisionCommand {
    pub user_id: UserId,
    pub analysis_data: DecisionAnalysisData,
}

/// Result of profile update from decision.
#[derive(Debug, Clone)]
pub struct UpdateProfileFromDecisionResult {
    pub analysis_result: AnalysisResult,
}

/// Handler for updating profiles from decision data.
pub struct UpdateProfileFromDecisionHandler {
    repository: Arc<dyn ProfileRepository>,
    analyzer: Arc<dyn ProfileAnalyzer>,
}

impl UpdateProfileFromDecisionHandler {
    pub fn new(
        repository: Arc<dyn ProfileRepository>,
        analyzer: Arc<dyn ProfileAnalyzer>,
    ) -> Self {
        Self {
            repository,
            analyzer,
        }
    }

    pub async fn handle(
        &self,
        cmd: UpdateProfileFromDecisionCommand,
        metadata: CommandMetadata,
    ) -> Result<UpdateProfileFromDecisionResult, DomainError> {
        // 1. Load profile (must exist and have consent)
        let mut profile = self
            .repository
            .find_by_user(&cmd.user_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::NotFound, "Profile not found"))?;

        if !profile.consent().analysis_enabled {
            return Err(DomainError::new(
                ErrorCode::Forbidden,
                "Analysis consent not granted",
            ));
        }

        // 2. Analyze decision and update profile
        let result = self
            .analyzer
            .analyze_decision(&mut profile, &cmd.analysis_data)
            .await?;

        // 3. Save updated profile
        self.repository.update(&profile).await?;

        // TODO: Publish domain events when event infrastructure is ready

        Ok(UpdateProfileFromDecisionResult {
            analysis_result: result,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{EventEnvelope, Timestamp};
    use crate::domain::user::{
        BlindSpot, CognitiveBiasType, CognitivePattern, DecisionDomain, DecisionProfile,
        DecisionProfileId, GrowthObservation, ProfileConsent, RiskClassification, SeverityLevel,
    };
    use crate::domain::foundation::{DomainError, ErrorCode};
use crate::ports::{ConversationSummary, RiskIndicator};
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct MockProfileRepository {
        profiles: Mutex<Vec<DecisionProfile>>,
    }

    impl MockProfileRepository {
        fn new() -> Self {
            Self {
                profiles: Mutex::new(Vec::new()),
            }
        }

        fn with_profile(mut self, profile: DecisionProfile) -> Self {
            self.profiles.lock().unwrap().push(profile);
            self
        }
    }

    #[async_trait]
    impl ProfileRepository for MockProfileRepository {
        async fn create(&self, _profile: &DecisionProfile) -> Result<(), DomainError> {
            unimplemented!()
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

        async fn delete(&self, _profile_id: DecisionProfileId) -> Result<(), DomainError> {
            unimplemented!()
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
            _format: crate::ports::ExportFormat,
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

    struct MockProfileAnalyzer {
        should_fail: bool,
    }

    impl MockProfileAnalyzer {
        fn new() -> Self {
            Self { should_fail: false }
        }
    }

    #[async_trait]
    impl ProfileAnalyzer for MockProfileAnalyzer {
        async fn analyze_decision(
            &self,
            _profile: &mut DecisionProfile,
            _data: &DecisionAnalysisData,
        ) -> Result<AnalysisResult, DomainError> {
            if self.should_fail {
                return Err(DomainError::new(
                    ErrorCode::InternalError,
                    "Analysis failed",
                ));
            }

            Ok(AnalysisResult {
                risk_profile_changed: true,
                new_patterns_detected: vec!["Loss aversion detected".to_string()],
                blind_spots_identified: vec![BlindSpot::new(
                    "Long-term thinking".to_string(),
                    "Tends to focus on short-term outcomes".to_string(),
                    vec!["Evidence from decision".to_string()],
                    "Prompt for 10-year view".to_string(),
                    Timestamp::now(),
                )
                .unwrap()],
                growth_observed: vec![],
            })
        }

        fn recalculate_risk_profile(
            &self,
            _history: &crate::domain::user::DecisionHistory,
        ) -> Result<crate::domain::user::RiskProfile, DomainError> {
            unimplemented!()
        }

        fn detect_cognitive_patterns(
            &self,
            _history: &crate::domain::user::DecisionHistory,
            _conversations: &[ConversationSummary],
        ) -> Result<Vec<CognitivePattern>, DomainError> {
            unimplemented!()
        }

        fn identify_blind_spots(
            &self,
            _profile: &DecisionProfile,
        ) -> Result<Vec<BlindSpot>, DomainError> {
            unimplemented!()
        }

        fn generate_agent_instructions(
            &self,
            _profile: &DecisionProfile,
        ) -> Result<crate::ports::AgentInstructions, DomainError> {
            unimplemented!()
        }
    }

    fn test_user_id() -> UserId {
        UserId::new("test@example.com".to_string()).unwrap()
    }

    fn test_consent() -> ProfileConsent {
        ProfileConsent::full(Timestamp::now())
    }

    fn test_metadata() -> CommandMetadata {
        CommandMetadata::new(test_user_id())
    }

    fn test_analysis_data() -> DecisionAnalysisData {
        DecisionAnalysisData {
            title: "Test Decision".to_string(),
            domain: DecisionDomain::Career,
            dq_score: Some(85),
            key_tradeoff: "Growth vs Stability".to_string(),
            chosen_alternative: "Option A".to_string(),
            objectives: vec!["Financial security".to_string()],
            alternatives: vec!["Option A".to_string(), "Option B".to_string()],
            conversations: vec![],
            risk_indicators: vec![],
        }
    }

    #[tokio::test]
    async fn test_update_profile_from_decision_success() {
        let profile = DecisionProfile::new(test_user_id(), test_consent(), Timestamp::now()).unwrap();
        let repo = Arc::new(MockProfileRepository::new().with_profile(profile));
        let analyzer = Arc::new(MockProfileAnalyzer::new());
        let handler = UpdateProfileFromDecisionHandler::new(repo, analyzer);

        let result = handler
            .handle(
                UpdateProfileFromDecisionCommand {
                    user_id: test_user_id(),
                    analysis_data: test_analysis_data(),
                },
                test_metadata(),
            )
            .await;

        assert!(result.is_ok());
        let update_result = result.unwrap();
        assert!(update_result.analysis_result.risk_profile_changed);
        assert_eq!(
            update_result.analysis_result.new_patterns_detected.len(),
            1
        );
    }

    #[tokio::test]
    async fn test_update_profile_not_found() {
        let repo = Arc::new(MockProfileRepository::new());
        let analyzer = Arc::new(MockProfileAnalyzer::new());
        let handler = UpdateProfileFromDecisionHandler::new(repo, analyzer);

        let result = handler
            .handle(
                UpdateProfileFromDecisionCommand {
                    user_id: test_user_id(),
                    analysis_data: test_analysis_data(),
                },
                test_metadata(),
            )
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message()
            .contains("Profile not found"));
    }

    #[tokio::test]
    async fn test_update_profile_without_analysis_consent() {
        let mut consent = test_consent();
        consent.analysis_enabled = false;
        let profile = DecisionProfile::new(test_user_id(), consent, Timestamp::now()).unwrap();
        let repo = Arc::new(MockProfileRepository::new().with_profile(profile));
        let analyzer = Arc::new(MockProfileAnalyzer::new());
        let handler = UpdateProfileFromDecisionHandler::new(repo, analyzer);

        let result = handler
            .handle(
                UpdateProfileFromDecisionCommand {
                    user_id: test_user_id(),
                    analysis_data: test_analysis_data(),
                },
                test_metadata(),
            )
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message()
            .contains("Analysis consent not granted"));
    }
}
