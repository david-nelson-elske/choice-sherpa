//! DeleteProfile - Command handler for deleting decision profiles.

use std::sync::Arc;

use crate::domain::foundation::{CommandMetadata, DomainError, ErrorCode, UserId};
use crate::domain::user::DecisionProfileId;
use crate::ports::ProfileRepository;

/// Command to delete a decision profile.
#[derive(Debug, Clone)]
pub struct DeleteProfileCommand {
    pub user_id: UserId,
    pub confirmation: String,
}

/// Required confirmation string for deletion.
const CONFIRMATION_STRING: &str = "DELETE MY PROFILE";

/// Result of profile deletion.
#[derive(Debug, Clone)]
pub struct DeleteProfileResult {
    pub deleted_profile_id: DecisionProfileId,
}

/// Handler for deleting profiles.
pub struct DeleteProfileHandler {
    repository: Arc<dyn ProfileRepository>,
}

impl DeleteProfileHandler {
    pub fn new(repository: Arc<dyn ProfileRepository>) -> Self {
        Self { repository }
    }

    pub async fn handle(
        &self,
        cmd: DeleteProfileCommand,
        _metadata: CommandMetadata,
    ) -> Result<DeleteProfileResult, DomainError> {
        // 1. Verify confirmation string
        if cmd.confirmation != CONFIRMATION_STRING {
            return Err(DomainError::validation(
                "confirmation",
                &format!(
                    "Confirmation string must be exactly '{}'",
                    CONFIRMATION_STRING
                ),
            ));
        }

        // 2. Load profile to get ID
        let profile = self
            .repository
            .find_by_user(&cmd.user_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::NotFound, "Profile not found"))?;

        let profile_id = profile.id();

        // 3. Delete profile (cascades to history via foreign key)
        self.repository.delete(profile_id).await?;

        // TODO: Publish domain events when event infrastructure is ready

        Ok(DeleteProfileResult {
            deleted_profile_id: profile_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::Timestamp;
    use crate::domain::user::{DecisionProfile, ProfileConsent};
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct MockProfileRepository {
        profiles: Mutex<Vec<DecisionProfile>>,
        deleted_ids: Mutex<Vec<DecisionProfileId>>,
    }

    impl MockProfileRepository {
        fn new() -> Self {
            Self {
                profiles: Mutex::new(Vec::new()),
                deleted_ids: Mutex::new(Vec::new()),
            }
        }

        fn with_profile(mut self, profile: DecisionProfile) -> Self {
            self.profiles.lock().unwrap().push(profile);
            self
        }

        fn deleted_count(&self) -> usize {
            self.deleted_ids.lock().unwrap().len()
        }
    }

    #[async_trait]
    impl ProfileRepository for MockProfileRepository {
        async fn create(&self, _profile: &DecisionProfile) -> Result<(), DomainError> {
            unimplemented!()
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

        async fn delete(&self, profile_id: DecisionProfileId) -> Result<(), DomainError> {
            let mut profiles = self.profiles.lock().unwrap();
            if let Some(pos) = profiles.iter().position(|p| p.id() == profile_id) {
                profiles.remove(pos);
                self.deleted_ids.lock().unwrap().push(profile_id);
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

    fn test_user_id() -> UserId {
        UserId::new("test@example.com".to_string()).unwrap()
    }

    fn test_consent() -> ProfileConsent {
        ProfileConsent::full(Timestamp::now())
    }

    fn test_metadata() -> CommandMetadata {
        CommandMetadata::new(test_user_id())
    }

    #[tokio::test]
    async fn test_delete_profile_success() {
        let profile = DecisionProfile::new(test_user_id(), test_consent(), Timestamp::now()).unwrap();
        let repo = Arc::new(MockProfileRepository::new().with_profile(profile));
        let handler = DeleteProfileHandler::new(repo.clone());

        let result = handler
            .handle(
                DeleteProfileCommand {
                    user_id: test_user_id(),
                    confirmation: CONFIRMATION_STRING.to_string(),
                },
                test_metadata(),
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(repo.deleted_count(), 1);
    }

    #[tokio::test]
    async fn test_delete_profile_wrong_confirmation() {
        let profile = DecisionProfile::new(test_user_id(), test_consent(), Timestamp::now()).unwrap();
        let repo = Arc::new(MockProfileRepository::new().with_profile(profile));
        let handler = DeleteProfileHandler::new(repo.clone());

        let result = handler
            .handle(
                DeleteProfileCommand {
                    user_id: test_user_id(),
                    confirmation: "WRONG CONFIRMATION".to_string(),
                },
                test_metadata(),
            )
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().message().contains("Confirmation string"));
        assert_eq!(repo.deleted_count(), 0);
    }

    #[tokio::test]
    async fn test_delete_profile_not_found() {
        let repo = Arc::new(MockProfileRepository::new());
        let handler = DeleteProfileHandler::new(repo.clone());

        let result = handler
            .handle(
                DeleteProfileCommand {
                    user_id: test_user_id(),
                    confirmation: CONFIRMATION_STRING.to_string(),
                },
                test_metadata(),
            )
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message()
            .contains("Profile not found"));
        assert_eq!(repo.deleted_count(), 0);
    }
}
