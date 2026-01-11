//! RecordOutcome - Command handler for recording decision outcomes.

use std::sync::Arc;

use crate::domain::foundation::{CommandMetadata, CycleId, DomainError, ErrorCode, UserId};
use crate::domain::user::{OutcomeRecord, SatisfactionLevel};
use crate::ports::ProfileRepository;

/// Command to record a decision outcome.
#[derive(Debug, Clone)]
pub struct RecordOutcomeCommand {
    pub user_id: UserId,
    pub cycle_id: CycleId,
    pub satisfaction: SatisfactionLevel,
    pub actual_consequences: String,
    pub surprises: Vec<String>,
    pub would_decide_same: bool,
}

/// Result of recording outcome.
#[derive(Debug, Clone)]
pub struct RecordOutcomeResult {
    pub success: bool,
}

/// Handler for recording decision outcomes.
pub struct RecordOutcomeHandler {
    repository: Arc<dyn ProfileRepository>,
}

impl RecordOutcomeHandler {
    pub fn new(repository: Arc<dyn ProfileRepository>) -> Self {
        Self { repository }
    }

    pub async fn handle(
        &self,
        cmd: RecordOutcomeCommand,
        _metadata: CommandMetadata,
    ) -> Result<RecordOutcomeResult, DomainError> {
        // 1. Load profile
        let mut profile = self
            .repository
            .find_by_user(&cmd.user_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::NotFound, "Profile not found"))?;

        // 2. Create outcome record
        let outcome = OutcomeRecord::new(
            cmd.satisfaction,
            cmd.actual_consequences.clone(),
            cmd.surprises.clone(),
            cmd.would_decide_same,
        )?;

        // 3. Record outcome in history
        profile.record_outcome(&cmd.cycle_id, outcome)?;

        // 4. Save updated profile
        self.repository.update(&profile).await?;

        // TODO: Publish domain events when event infrastructure is ready

        Ok(RecordOutcomeResult { success: true })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{EventEnvelope, Timestamp};
    use crate::domain::user::{DecisionDomain, DecisionProfile, DecisionProfileId, ProfileConsent};
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

        async fn export(
            &self,
            _profile_id: DecisionProfileId,
            _format: crate::ports::ExportFormat,
        ) -> Result<Vec<u8>, DomainError> {
            unimplemented!()
        }
    }

    fn test_user_id() -> UserId {
        UserId::new("test@example.com".to_string()).unwrap()
    }

    fn test_consent() -> ProfileConsent {
        ProfileConsent::new(true, true, true, Timestamp::now()).unwrap()
    }

    fn test_metadata() -> CommandMetadata {
        CommandMetadata::new(test_user_id(), "test-correlation-id")
    }

    fn test_profile_with_decision() -> DecisionProfile {
        let mut profile = DecisionProfile::new(test_user_id(), test_consent()).unwrap();
        let cycle_id = CycleId::new();
        profile
            .add_decision(
                cycle_id,
                "Test Decision".to_string(),
                DecisionDomain::Career,
                Some(85),
                "Growth vs Stability".to_string(),
                "Option A".to_string(),
            )
            .unwrap();
        profile
    }

    #[tokio::test]
    async fn test_record_outcome_success() {
        let profile = test_profile_with_decision();
        let cycle_id = profile.decision_history().decisions()[0].cycle_id;
        let repo = Arc::new(MockProfileRepository::new().with_profile(profile));
        let handler = RecordOutcomeHandler::new(repo);

        let result = handler
            .handle(
                RecordOutcomeCommand {
                    user_id: test_user_id(),
                    cycle_id,
                    satisfaction: SatisfactionLevel::Satisfied,
                    actual_consequences: "Everything went as expected".to_string(),
                    surprises: vec![],
                    would_decide_same: true,
                },
                test_metadata(),
            )
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }

    #[tokio::test]
    async fn test_record_outcome_profile_not_found() {
        let repo = Arc::new(MockProfileRepository::new());
        let handler = RecordOutcomeHandler::new(repo);

        let result = handler
            .handle(
                RecordOutcomeCommand {
                    user_id: test_user_id(),
                    cycle_id: CycleId::new(),
                    satisfaction: SatisfactionLevel::Satisfied,
                    actual_consequences: "Test".to_string(),
                    surprises: vec![],
                    would_decide_same: true,
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
    async fn test_record_outcome_decision_not_found() {
        let profile = DecisionProfile::new(test_user_id(), test_consent()).unwrap();
        let repo = Arc::new(MockProfileRepository::new().with_profile(profile));
        let handler = RecordOutcomeHandler::new(repo);

        let result = handler
            .handle(
                RecordOutcomeCommand {
                    user_id: test_user_id(),
                    cycle_id: CycleId::new(), // Non-existent cycle
                    satisfaction: SatisfactionLevel::Satisfied,
                    actual_consequences: "Test".to_string(),
                    surprises: vec![],
                    would_decide_same: true,
                },
                test_metadata(),
            )
            .await;

        assert!(result.is_err());
    }
}
