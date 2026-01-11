//! CreatePaidMembershipHandler - Command handler for initiating paid membership checkout.

use std::sync::Arc;

use crate::domain::foundation::{EventId, MembershipId, SerializableDomainEvent, Timestamp, UserId};
use crate::domain::membership::{Membership, MembershipError, MembershipEvent, MembershipTier};
use crate::ports::{
    CheckoutSession, CreateCheckoutRequest, CreateCustomerRequest, EventPublisher,
    MembershipRepository, PaymentProvider,
};

/// Command to initiate a paid membership checkout.
#[derive(Debug, Clone)]
pub struct CreatePaidMembershipCommand {
    pub user_id: UserId,
    pub email: String,
    pub tier: MembershipTier,
    pub success_url: String,
    pub cancel_url: String,
    pub promo_code: Option<String>,
}

/// Result of successful checkout initiation.
#[derive(Debug, Clone)]
pub struct CreatePaidMembershipResult {
    pub membership: Membership,
    pub checkout_session: CheckoutSession,
    pub event: MembershipEvent,
}

/// Handler for initiating paid membership checkout.
///
/// This creates a pending membership and redirects the user to the payment provider's
/// checkout page. The membership is activated when the webhook confirms payment.
pub struct CreatePaidMembershipHandler {
    repository: Arc<dyn MembershipRepository>,
    payment_provider: Arc<dyn PaymentProvider>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl CreatePaidMembershipHandler {
    pub fn new(
        repository: Arc<dyn MembershipRepository>,
        payment_provider: Arc<dyn PaymentProvider>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            repository,
            payment_provider,
            event_publisher,
        }
    }

    pub async fn handle(
        &self,
        cmd: CreatePaidMembershipCommand,
    ) -> Result<CreatePaidMembershipResult, MembershipError> {
        // 1. Check if user already has a membership
        if let Some(_existing) = self.repository.find_by_user_id(&cmd.user_id).await? {
            return Err(MembershipError::already_exists(cmd.user_id));
        }

        // 2. Validate tier is paid
        if cmd.tier == MembershipTier::Free {
            return Err(MembershipError::invalid_tier(
                "Cannot use checkout for free tier. Use promo code instead.",
            ));
        }

        // 3. Create or get customer in payment provider
        let customer = self
            .payment_provider
            .create_customer(CreateCustomerRequest {
                user_id: cmd.user_id.clone(),
                email: cmd.email.clone(),
                name: None,
                idempotency_key: Some(format!("customer-{}", cmd.user_id)),
            })
            .await
            .map_err(|e| MembershipError::payment_failed(e.message))?;

        // 4. Create pending membership
        let membership_id = MembershipId::new();
        let membership = Membership::create_paid(
            membership_id,
            cmd.user_id.clone(),
            cmd.tier,
            customer.id.clone(),
        );

        // 5. Persist pending membership
        self.repository.save(&membership).await?;

        // 6. Create checkout session
        let checkout_session = self
            .payment_provider
            .create_checkout_session(CreateCheckoutRequest {
                user_id: cmd.user_id.clone(),
                email: cmd.email,
                tier: cmd.tier,
                success_url: cmd.success_url,
                cancel_url: cmd.cancel_url,
                promo_code: cmd.promo_code.clone(),
            })
            .await
            .map_err(|e| MembershipError::payment_failed(e.message))?;

        // 7. Create and publish event
        let event = MembershipEvent::Created {
            event_id: EventId::new(),
            membership_id,
            user_id: cmd.user_id,
            tier: cmd.tier,
            is_free: false,
            promo_code: cmd.promo_code,
            occurred_at: Timestamp::now(),
        };

        let envelope = event.to_envelope();
        self.event_publisher.publish(envelope).await?;

        Ok(CreatePaidMembershipResult {
            membership,
            checkout_session,
            event,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{DomainError, ErrorCode, EventEnvelope};
    use crate::domain::membership::MembershipStatus;
    use crate::ports::{
        CheckoutSession, CreateSubscriptionRequest, Customer, PaymentError, PaymentErrorCode,
        PortalSession, Subscription, SubscriptionStatus, WebhookEvent,
    };
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

    struct MockPaymentProvider {
        fail_create_customer: bool,
        fail_create_checkout: bool,
    }

    impl MockPaymentProvider {
        fn new() -> Self {
            Self {
                fail_create_customer: false,
                fail_create_checkout: false,
            }
        }

        fn failing_customer() -> Self {
            Self {
                fail_create_customer: true,
                fail_create_checkout: false,
            }
        }

        fn failing_checkout() -> Self {
            Self {
                fail_create_customer: false,
                fail_create_checkout: true,
            }
        }
    }

    #[async_trait]
    impl PaymentProvider for MockPaymentProvider {
        async fn create_customer(
            &self,
            request: CreateCustomerRequest,
        ) -> Result<Customer, PaymentError> {
            if self.fail_create_customer {
                return Err(PaymentError::new(
                    PaymentErrorCode::ProviderError,
                    "Customer creation failed",
                ));
            }
            Ok(Customer {
                id: format!("cus_{}", request.user_id),
                email: request.email,
                name: request.name,
                created_at: 1234567890,
            })
        }

        async fn get_customer(&self, _customer_id: &str) -> Result<Option<Customer>, PaymentError> {
            Ok(None)
        }

        async fn create_subscription(
            &self,
            _request: CreateSubscriptionRequest,
        ) -> Result<Subscription, PaymentError> {
            Ok(Subscription {
                id: "sub_123".to_string(),
                customer_id: "cus_123".to_string(),
                status: SubscriptionStatus::Active,
                current_period_start: 1234567890,
                current_period_end: 1237246290,
                cancel_at_period_end: false,
                canceled_at: None,
            })
        }

        async fn get_subscription(
            &self,
            _subscription_id: &str,
        ) -> Result<Option<Subscription>, PaymentError> {
            Ok(None)
        }

        async fn cancel_subscription(
            &self,
            _subscription_id: &str,
            _at_period_end: bool,
        ) -> Result<Subscription, PaymentError> {
            Ok(Subscription {
                id: "sub_123".to_string(),
                customer_id: "cus_123".to_string(),
                status: SubscriptionStatus::Canceled,
                current_period_start: 1234567890,
                current_period_end: 1237246290,
                cancel_at_period_end: true,
                canceled_at: Some(1234567890),
            })
        }

        async fn update_subscription(
            &self,
            _subscription_id: &str,
            _new_tier: MembershipTier,
        ) -> Result<Subscription, PaymentError> {
            Ok(Subscription {
                id: "sub_123".to_string(),
                customer_id: "cus_123".to_string(),
                status: SubscriptionStatus::Active,
                current_period_start: 1234567890,
                current_period_end: 1237246290,
                cancel_at_period_end: false,
                canceled_at: None,
            })
        }

        async fn create_checkout_session(
            &self,
            _request: CreateCheckoutRequest,
        ) -> Result<CheckoutSession, PaymentError> {
            if self.fail_create_checkout {
                return Err(PaymentError::new(
                    PaymentErrorCode::ProviderError,
                    "Checkout session creation failed",
                ));
            }
            Ok(CheckoutSession {
                id: "cs_123".to_string(),
                url: "https://checkout.stripe.com/cs_123".to_string(),
                expires_at: 1234567890 + 3600,
            })
        }

        async fn create_portal_session(
            &self,
            _customer_id: &str,
            _return_url: &str,
        ) -> Result<PortalSession, PaymentError> {
            Ok(PortalSession {
                id: "ps_123".to_string(),
                url: "https://billing.stripe.com/ps_123".to_string(),
            })
        }

        async fn verify_webhook(
            &self,
            _payload: &[u8],
            _signature: &str,
        ) -> Result<WebhookEvent, PaymentError> {
            Err(PaymentError::invalid_webhook("Not implemented in mock"))
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

    fn test_command() -> CreatePaidMembershipCommand {
        CreatePaidMembershipCommand {
            user_id: test_user_id(),
            email: "user@example.com".to_string(),
            tier: MembershipTier::Monthly,
            success_url: "https://app.example.com/success".to_string(),
            cancel_url: "https://app.example.com/cancel".to_string(),
            promo_code: None,
        }
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Success Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn creates_pending_membership_and_checkout() {
        let repo = Arc::new(MockMembershipRepository::new());
        let payment = Arc::new(MockPaymentProvider::new());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreatePaidMembershipHandler::new(repo.clone(), payment, publisher);

        let cmd = test_command();
        let result = handler.handle(cmd).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.membership.status, MembershipStatus::Pending);
        assert!(result.checkout_session.url.contains("checkout.stripe.com"));
    }

    #[tokio::test]
    async fn publishes_membership_created_event() {
        let repo = Arc::new(MockMembershipRepository::new());
        let payment = Arc::new(MockPaymentProvider::new());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreatePaidMembershipHandler::new(repo, payment, publisher.clone());

        let cmd = test_command();
        handler.handle(cmd).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "membership.created.v1");
    }

    #[tokio::test]
    async fn saves_membership_to_repository() {
        let repo = Arc::new(MockMembershipRepository::new());
        let payment = Arc::new(MockPaymentProvider::new());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreatePaidMembershipHandler::new(repo.clone(), payment, publisher);

        let cmd = test_command();
        handler.handle(cmd).await.unwrap();

        let saved = repo.saved_memberships();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].tier, MembershipTier::Monthly);
    }

    #[tokio::test]
    async fn sets_stripe_customer_id() {
        let repo = Arc::new(MockMembershipRepository::new());
        let payment = Arc::new(MockPaymentProvider::new());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreatePaidMembershipHandler::new(repo.clone(), payment, publisher);

        let cmd = test_command();
        let result = handler.handle(cmd).await.unwrap();

        assert!(result.membership.stripe_customer_id.is_some());
        assert!(result.membership.stripe_customer_id.unwrap().starts_with("cus_"));
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Failure Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn fails_when_user_already_has_membership() {
        let user_id = test_user_id();
        let repo = Arc::new(MockMembershipRepository::with_existing_user(user_id.clone()));
        let payment = Arc::new(MockPaymentProvider::new());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreatePaidMembershipHandler::new(repo.clone(), payment, publisher.clone());

        let mut cmd = test_command();
        cmd.user_id = user_id;

        let result = handler.handle(cmd).await;
        assert!(matches!(result, Err(MembershipError::AlreadyExists(_))));
        assert!(repo.saved_memberships().is_empty());
    }

    #[tokio::test]
    async fn fails_when_tier_is_free() {
        let repo = Arc::new(MockMembershipRepository::new());
        let payment = Arc::new(MockPaymentProvider::new());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreatePaidMembershipHandler::new(repo.clone(), payment, publisher);

        let mut cmd = test_command();
        cmd.tier = MembershipTier::Free;

        let result = handler.handle(cmd).await;
        assert!(matches!(result, Err(MembershipError::InvalidTier(_))));
    }

    #[tokio::test]
    async fn fails_when_customer_creation_fails() {
        let repo = Arc::new(MockMembershipRepository::new());
        let payment = Arc::new(MockPaymentProvider::failing_customer());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreatePaidMembershipHandler::new(repo.clone(), payment, publisher);

        let cmd = test_command();
        let result = handler.handle(cmd).await;

        assert!(matches!(result, Err(MembershipError::PaymentFailed { .. })));
        assert!(repo.saved_memberships().is_empty());
    }

    #[tokio::test]
    async fn fails_when_checkout_creation_fails() {
        let repo = Arc::new(MockMembershipRepository::new());
        let payment = Arc::new(MockPaymentProvider::failing_checkout());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreatePaidMembershipHandler::new(repo.clone(), payment, publisher.clone());

        let cmd = test_command();
        let result = handler.handle(cmd).await;

        assert!(matches!(result, Err(MembershipError::PaymentFailed { .. })));
        // Membership was saved before checkout failed - this is intentional
        // to track the pending state
    }

    #[tokio::test]
    async fn fails_when_repository_save_fails() {
        let repo = Arc::new(MockMembershipRepository::failing());
        let payment = Arc::new(MockPaymentProvider::new());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CreatePaidMembershipHandler::new(repo, payment, publisher.clone());

        let cmd = test_command();
        let result = handler.handle(cmd).await;

        assert!(result.is_err());
        assert!(publisher.published_events().is_empty());
    }
}
