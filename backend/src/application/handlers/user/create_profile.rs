//! CreateProfile - Command handler for creating decision profiles.

use std::sync::Arc;

use crate::domain::foundation::{CommandMetadata, DomainError, ErrorCode, UserId};
use crate::domain::user::{DecisionProfile, DecisionProfileId, ProfileConsent};
use crate::ports::{ProfileRepository};

/// Command to create a new decision profile.
#[derive(Debug, Clone)]
pub struct CreateProfileCommand {
    pub user_id: UserId,
    pub consent: ProfileConsent,
}

/// Result of successful profile creation.
#[derive(Debug, Clone)]
pub struct CreateProfileResult {
    pub profile_id: DecisionProfileId,
}

/// Handler for creating profiles.
pub struct CreateProfileHandler {
    repository: Arc<dyn ProfileRepository>,
}

impl CreateProfileHandler {
    pub fn new(
        repository: Arc<dyn ProfileRepository>,
    ) -> Self {
        Self {
            repository,
        }
    }

    pub async fn handle(
        &self,
        cmd: CreateProfileCommand,
        metadata: CommandMetadata,
    ) -> Result<CreateProfileResult, DomainError> {
        // 1. Verify consent is valid
        if !cmd.consent.collection_enabled {
            return Err(DomainError::validation(
                "consent",
                "Consent required for profile creation",
            ));
        }

        // 2. Check if profile already exists
        if let Some(_existing) = self.repository.find_by_user(&cmd.user_id).await? {
            return Err(DomainError::new(
                ErrorCode::Conflict,
                "Profile already exists for this user",
            ));
        }

        // 3. Create empty profile
        let profile = DecisionProfile::new(cmd.user_id.clone(), cmd.consent)?;
        let profile_id = *profile.id();

        // 4. Persist profile
        self.repository.create(&profile).await?;

        // TODO: Publish domain events from profile when event infrastructure is ready

        Ok(CreateProfileResult { profile_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::Timestamp;
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct MockProfileRepository {
        profiles: Mutex<Vec<DecisionProfile>>,
        should_fail: bool,
    }

    impl MockProfileRepository {
        fn new() -> Self {
            Self {
                profiles: Mutex::new(Vec::new()),
                should_fail: false,
            }
        }

        fn with_existing_profile(mut self, profile: DecisionProfile) -> Self {
            self.profiles.lock().unwrap().push(profile);
            self
        }
    }

    #[async_trait]
    impl ProfileRepository for MockProfileRepository {
        async fn create(&self, profile: &DecisionProfile) -> Result<(), DomainError> {
            if self.should_fail {
                return Err(DomainError::new(
                    ErrorCode::InternalError,
                    "Repository error",
                ));
            }
            self.profiles.lock().unwrap().push(profile.clone());
            Ok(())
        }

        async fn update(&self, _profile: &DecisionProfile) -> Result<(), DomainError> {
            unimplemented!()
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

    #[tokio::test]
    async fn test_create_profile_success() {
        let repo = Arc::new(MockProfileRepository::new());
        let handler = CreateProfileHandler::new(repo.clone());

        let result = handler
            .handle(
                CreateProfileCommand {
                    user_id: test_user_id(),
                    consent: test_consent(),
                },
                test_metadata(),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_profile_without_consent() {
        let repo = Arc::new(MockProfileRepository::new());
        let handler = CreateProfileHandler::new(repo);

        let mut consent = test_consent();
        consent.collection_enabled = false;

        let result = handler
            .handle(
                CreateProfileCommand {
                    user_id: test_user_id(),
                    consent,
                },
                test_metadata(),
            )
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message()
            .contains("Consent required"));
    }

    #[tokio::test]
    async fn test_create_profile_already_exists() {
        let existing_profile =
            DecisionProfile::new(test_user_id(), test_consent()).unwrap();
        let repo = Arc::new(MockProfileRepository::new().with_existing_profile(existing_profile));
        let handler = CreateProfileHandler::new(repo);

        let result = handler
            .handle(
                CreateProfileCommand {
                    user_id: test_user_id(),
                    consent: test_consent(),
                },
                test_metadata(),
            )
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().message().contains("already exists"));
    }
}
