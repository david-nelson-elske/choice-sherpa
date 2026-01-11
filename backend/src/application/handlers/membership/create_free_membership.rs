//! CreateFreeMembershipHandler - Command handler for creating free memberships via promo codes.

use std::sync::Arc;

use crate::domain::foundation::{EventId, MembershipId, SerializableDomainEvent, Timestamp, UserId};
use crate::domain::membership::{Membership, MembershipError, MembershipEvent, PromoCode};
use crate::ports::{
    EventPublisher, MembershipRepository, PromoCodeValidation, PromoCodeValidator,
};

/// Command to create a free membership using a promo code.
#[derive(Debug, Clone)]
pub struct CreateFreeMembershipCommand {
    pub user_id: UserId,
    pub promo_code: String,
}

/// Result of successful free membership creation.
#[derive(Debug, Clone)]
pub struct CreateFreeMembershipResult {
    pub membership: Membership,
    pub event: MembershipEvent,
}

/// Handler for creating free memberships via promo codes.
pub struct CreateFreeMembershipHandler {
    repository: Arc<dyn MembershipRepository>,
    promo_validator: Arc<dyn PromoCodeValidator>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl CreateFreeMembershipHandler {
    pub fn new(
        repository: Arc<dyn MembershipRepository>,
        promo_validator: Arc<dyn PromoCodeValidator>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            repository,
            promo_validator,
            event_publisher,
        }
    }

    pub async fn handle(
        &self,
        cmd: CreateFreeMembershipCommand,
    ) -> Result<CreateFreeMembershipResult, MembershipError> {
        // 1. Check if user already has a membership
        if let Some(_existing) = self.repository.find_by_user_id(&cmd.user_id).await? {
            return Err(MembershipError::already_exists(cmd.user_id));
        }

        // 2. Parse and validate promo code
        let promo_code = PromoCode::try_new(&cmd.promo_code)
            .map_err(|e| MembershipError::invalid_promo_code(&cmd.promo_code, e.to_string()))?;

        // 3. Validate promo code with external service
        let validation = self.promo_validator.validate(&promo_code).await?;

        let (duration_days, tier, _campaign) = match validation {
            PromoCodeValidation::Valid {
                duration_days,
                tier,
                campaign,
            } => (duration_days, tier, campaign),
            PromoCodeValidation::Invalid(reason) => {
                return Err(MembershipError::invalid_promo_code(
                    &cmd.promo_code,
                    reason.user_message(),
                ));
            }
        };

        // 4. Create membership aggregate
        let membership_id = MembershipId::new();
        let now = Timestamp::now();
        let period_end = now.add_days(duration_days as i64);

        let membership = Membership::create_free(
            membership_id,
            cmd.user_id.clone(),
            tier,
            cmd.promo_code.clone(),
            now,
            period_end,
        );

        // 5. Persist membership
        self.repository.save(&membership).await?;

        // 6. Record promo code redemption
        self.promo_validator.record_redemption(&promo_code).await?;

        // 7. Create and publish event
        let event = MembershipEvent::Created {
            event_id: EventId::new(),
            membership_id,
            user_id: cmd.user_id,
            tier,
            is_free: true,
            promo_code: Some(cmd.promo_code),
            occurred_at: now,
        };

        let envelope = event.to_envelope();
        self.event_publisher.publish(envelope).await?;

        Ok(CreateFreeMembershipResult { membership, event })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{DomainError, ErrorCode, EventEnvelope};
    use crate::domain::membership::MembershipTier;
    use crate::ports::PromoCodeInvalidReason;
    use async_trait::async_trait;
    use std::sync::Mutex;

    // ════════════════════════════════════════════════════════════════════════════
    // Mock Implementations
    // ════════════════════════════════════════════════════════════════════════════

    struct MockMembershipRepository {
        saved_memberships: Mutex<Vec<Membership>>,
        existing_user_id: Mutex<Option<UserId>>,
        fail_save: bool,
    }

    impl MockMembershipRepository {
        fn new() -> Self {
            Self {
                saved_memberships: Mutex::new(Vec::new()),
                existing_user_id: Mutex::new(None),
                fail_save: false,
            }
        }

        fn with_existing_user(user_id: UserId) -> Self {
            Self {
                saved_memberships: Mutex::new(Vec::new()),
                existing_user_id: Mutex::new(Some(user_id)),
                fail_save: false,
            }
        }

        fn failing() -> Self {
            Self {
                saved_memberships: Mutex::new(Vec::new()),
                existing_user_id: Mutex::new(None),
                fail_save: true,
            }
        }

        fn saved_memberships(&self) -> Vec<Membership> {
            self.saved_memberships.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl MembershipRepository for MockMembershipRepository {
        async fn save(&self, membership: &Membership) -> Result<(), DomainError> {
            if self.fail_save {
                return Err(DomainError::new(
                    ErrorCode::DatabaseError,
                    "Simulated save failure",
                ));
            }
            self.saved_memberships
                .lock()
                .unwrap()
                .push(membership.clone());
            Ok(())
        }

        async fn update(&self, _membership: &Membership) -> Result<(), DomainError> {
            Ok(())
        }

        async fn find_by_id(
            &self,
            _id: &MembershipId,
        ) -> Result<Option<Membership>, DomainError> {
            Ok(None)
        }

        async fn find_by_user_id(
            &self,
            user_id: &UserId,
        ) -> Result<Option<Membership>, DomainError> {
            let existing = self.existing_user_id.lock().unwrap();
            if existing.as_ref() == Some(user_id) {
                // Return a dummy membership
                Ok(Some(Membership::create_free(
                    MembershipId::new(),
                    user_id.clone(),
                    MembershipTier::Free,
                    "EXISTING".to_string(),
                    Timestamp::now(),
                    Timestamp::now().add_days(30),
                )))
            } else {
                Ok(None)
            }
        }

        async fn find_expiring_within_days(
            &self,
            _days: u32,
        ) -> Result<Vec<Membership>, DomainError> {
            Ok(vec![])
        }

        async fn delete(&self, _id: &MembershipId) -> Result<(), DomainError> {
            Ok(())
        }

        async fn find_by_stripe_subscription_id(
            &self,
            _subscription_id: &str,
        ) -> Result<Option<Membership>, DomainError> {
            Ok(None)
        }

        async fn find_by_stripe_customer_id(
            &self,
            _customer_id: &str,
        ) -> Result<Option<Membership>, DomainError> {
            Ok(None)
        }
    }

    struct MockPromoCodeValidator {
        validation_result: PromoCodeValidation,
        fail_validation: bool,
        fail_redemption: bool,
    }

    impl MockPromoCodeValidator {
        fn valid() -> Self {
            Self {
                validation_result: PromoCodeValidation::valid_with_tier(365, MembershipTier::Annual),
                fail_validation: false,
                fail_redemption: false,
            }
        }

        fn invalid(reason: PromoCodeInvalidReason) -> Self {
            Self {
                validation_result: PromoCodeValidation::Invalid(reason),
                fail_validation: false,
                fail_redemption: false,
            }
        }

        fn failing_validation() -> Self {
            Self {
                validation_result: PromoCodeValidation::valid_free(30),
                fail_validation: true,
                fail_redemption: false,
            }
        }
    }

    #[async_trait]
    impl PromoCodeValidator for MockPromoCodeValidator {
        async fn validate(&self, _code: &PromoCode) -> Result<PromoCodeValidation, DomainError> {
            if self.fail_validation {
                return Err(DomainError::new(
                    ErrorCode::ExternalServiceError,
                    "Validation service unavailable",
                ));
            }
            Ok(self.validation_result.clone())
        }

        async fn record_redemption(&self, _code: &PromoCode) -> Result<(), DomainError> {
            if self.fail_redemption {
                return Err(DomainError::new(
                    ErrorCode::ExternalServiceError,
                    "Failed to record redemption",
                ));
            }
            Ok(())
        }

        async fn get_usage_count(&self, _code: &PromoCode) -> Result<Option<u32>, DomainError> {
            Ok(Some(0))
        }
    }

    struct MockEventPublisher {
        published_events: Mutex<Vec<EventEnvelope>>,
        fail_publish: bool,
    }

    impl MockEventPublisher {
        fn new() -> Self {
            Self {
                published_events: Mutex::new(Vec::new()),
                fail_publish: false,
            }
        }

        fn published_events(&self) -> Vec<EventEnvelope> {
            self.published_events.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl EventPublisher for MockEventPublisher {
        async fn publish(&self, event: EventEnvelope) -> Result<(), DomainError> {
            if self.fail_publish {
                return Err(DomainError::new(
                    ErrorCode::InternalError,
                    "Simulated publish failure",
                ));
            }
            self.published_events.lock().unwrap().push(event);
            Ok(())
        }

        async fn publish_all(&self, events: Vec<EventEnvelope>) -> Result<(), DomainError> {
            for event in events {
                self.publish(event).await?;
            }
            Ok(())
        }
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Test Helpers
    // ════════════════════════════════════════════════════════════════════════════

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Success Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn creates_free_membership_with_valid_promo_code() {
        let repo = Arc::new(MockMembershipRepository::new());
        let validator = Arc::new(MockPromoCodeValidator::valid());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreateFreeMembershipHandler::new(repo.clone(), validator, publisher);

        let cmd = CreateFreeMembershipCommand {
            user_id: test_user_id(),
            promo_code: "WORKSHOP2026-A7K9M3".to_string(),
        };

        let result = handler.handle(cmd).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.membership.tier, MembershipTier::Annual);
        assert_eq!(result.membership.promo_code, Some("WORKSHOP2026-A7K9M3".to_string()));
    }

    #[tokio::test]
    async fn publishes_membership_created_event() {
        let repo = Arc::new(MockMembershipRepository::new());
        let validator = Arc::new(MockPromoCodeValidator::valid());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreateFreeMembershipHandler::new(repo, validator, publisher.clone());

        let cmd = CreateFreeMembershipCommand {
            user_id: test_user_id(),
            promo_code: "WORKSHOP2026-A7K9M3".to_string(),
        };

        handler.handle(cmd).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "membership.created.v1");
    }

    #[tokio::test]
    async fn saves_membership_to_repository() {
        let repo = Arc::new(MockMembershipRepository::new());
        let validator = Arc::new(MockPromoCodeValidator::valid());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreateFreeMembershipHandler::new(repo.clone(), validator, publisher);

        let cmd = CreateFreeMembershipCommand {
            user_id: test_user_id(),
            promo_code: "WORKSHOP2026-A7K9M3".to_string(),
        };

        handler.handle(cmd).await.unwrap();

        let saved = repo.saved_memberships();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].user_id, test_user_id());
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Failure Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn fails_when_user_already_has_membership() {
        let user_id = test_user_id();
        let repo = Arc::new(MockMembershipRepository::with_existing_user(user_id.clone()));
        let validator = Arc::new(MockPromoCodeValidator::valid());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreateFreeMembershipHandler::new(repo.clone(), validator, publisher.clone());

        let cmd = CreateFreeMembershipCommand {
            user_id,
            promo_code: "WORKSHOP2026-A7K9M3".to_string(),
        };

        let result = handler.handle(cmd).await;
        assert!(matches!(result, Err(MembershipError::AlreadyExists(_))));
        assert!(repo.saved_memberships().is_empty());
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn fails_with_invalid_promo_code_format() {
        let repo = Arc::new(MockMembershipRepository::new());
        let validator = Arc::new(MockPromoCodeValidator::valid());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreateFreeMembershipHandler::new(repo.clone(), validator, publisher);

        let cmd = CreateFreeMembershipCommand {
            user_id: test_user_id(),
            promo_code: "ab".to_string(), // Too short
        };

        let result = handler.handle(cmd).await;
        assert!(matches!(
            result,
            Err(MembershipError::InvalidPromoCode { .. })
        ));
    }

    #[tokio::test]
    async fn fails_when_promo_code_not_found() {
        let repo = Arc::new(MockMembershipRepository::new());
        let validator = Arc::new(MockPromoCodeValidator::invalid(PromoCodeInvalidReason::NotFound));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreateFreeMembershipHandler::new(repo.clone(), validator, publisher);

        let cmd = CreateFreeMembershipCommand {
            user_id: test_user_id(),
            promo_code: "BADCODE123".to_string(),
        };

        let result = handler.handle(cmd).await;
        assert!(matches!(
            result,
            Err(MembershipError::InvalidPromoCode { .. })
        ));
    }

    #[tokio::test]
    async fn fails_when_promo_code_exhausted() {
        let repo = Arc::new(MockMembershipRepository::new());
        let validator = Arc::new(MockPromoCodeValidator::invalid(
            PromoCodeInvalidReason::Exhausted { used: 100, max: 100 },
        ));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreateFreeMembershipHandler::new(repo, validator, publisher);

        let cmd = CreateFreeMembershipCommand {
            user_id: test_user_id(),
            promo_code: "EXHAUSTED1".to_string(),
        };

        let result = handler.handle(cmd).await;
        assert!(matches!(
            result,
            Err(MembershipError::InvalidPromoCode { .. })
        ));
    }

    #[tokio::test]
    async fn fails_when_promo_code_expired() {
        let repo = Arc::new(MockMembershipRepository::new());
        let validator = Arc::new(MockPromoCodeValidator::invalid(
            PromoCodeInvalidReason::Expired { expired_at: "2025-01-01".to_string() },
        ));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreateFreeMembershipHandler::new(repo, validator, publisher);

        let cmd = CreateFreeMembershipCommand {
            user_id: test_user_id(),
            promo_code: "EXPIRED123".to_string(),
        };

        let result = handler.handle(cmd).await;
        assert!(matches!(
            result,
            Err(MembershipError::InvalidPromoCode { .. })
        ));
    }

    #[tokio::test]
    async fn fails_when_repository_save_fails() {
        let repo = Arc::new(MockMembershipRepository::failing());
        let validator = Arc::new(MockPromoCodeValidator::valid());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreateFreeMembershipHandler::new(repo, validator, publisher.clone());

        let cmd = CreateFreeMembershipCommand {
            user_id: test_user_id(),
            promo_code: "WORKSHOP2026-A7K9M3".to_string(),
        };

        let result = handler.handle(cmd).await;
        assert!(result.is_err());
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn fails_when_validation_service_unavailable() {
        let repo = Arc::new(MockMembershipRepository::new());
        let validator = Arc::new(MockPromoCodeValidator::failing_validation());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreateFreeMembershipHandler::new(repo, validator, publisher);

        let cmd = CreateFreeMembershipCommand {
            user_id: test_user_id(),
            promo_code: "WORKSHOP2026-A7K9M3".to_string(),
        };

        let result = handler.handle(cmd).await;
        assert!(result.is_err());
    }
}
