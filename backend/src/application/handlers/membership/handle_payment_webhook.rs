//! HandlePaymentWebhookHandler - Command handler for processing payment provider webhooks.

use std::sync::Arc;

use crate::domain::foundation::{EventId, SerializableDomainEvent, Timestamp};
use crate::domain::membership::{ExpiredReason, MembershipError, MembershipEvent};
use crate::ports::{
    EventPublisher, MembershipRepository, PaymentProvider, WebhookEvent, WebhookEventData,
    WebhookEventType,
};

/// Command to handle a payment webhook.
#[derive(Debug, Clone)]
pub struct HandlePaymentWebhookCommand {
    /// Raw webhook payload.
    pub payload: Vec<u8>,
    /// Webhook signature header.
    pub signature: String,
}

/// Result of webhook processing.
#[derive(Debug, Clone)]
pub enum HandlePaymentWebhookResult {
    /// Checkout completed, membership activated.
    MembershipActivated {
        membership_id: String,
        user_id: String,
    },
    /// Invoice paid, membership renewed.
    MembershipRenewed {
        membership_id: String,
        user_id: String,
    },
    /// Payment failed, membership marked past due.
    PaymentFailed {
        membership_id: String,
        user_id: String,
        attempt_count: u32,
    },
    /// Subscription deleted, membership expired.
    MembershipExpired {
        membership_id: String,
        user_id: String,
    },
    /// Event acknowledged but no action taken.
    Acknowledged,
    /// Event ignored (unknown or unsupported type).
    Ignored,
}

/// Handler for processing payment provider webhooks.
///
/// Processes webhook events from Stripe and updates membership state accordingly.
/// Publishes domain events for audit logging and integration.
pub struct HandlePaymentWebhookHandler {
    repository: Arc<dyn MembershipRepository>,
    payment_provider: Arc<dyn PaymentProvider>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl HandlePaymentWebhookHandler {
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
        cmd: HandlePaymentWebhookCommand,
    ) -> Result<HandlePaymentWebhookResult, MembershipError> {
        // 1. Verify webhook signature and parse event
        let webhook_event = self
            .payment_provider
            .verify_webhook(&cmd.payload, &cmd.signature)
            .await
            .map_err(|_| MembershipError::invalid_webhook_signature())?;

        // 2. Process based on event type
        match webhook_event.event_type {
            WebhookEventType::CheckoutSessionCompleted => {
                self.handle_checkout_completed(&webhook_event).await
            }
            WebhookEventType::InvoicePaid => self.handle_invoice_paid(&webhook_event).await,
            WebhookEventType::InvoicePaymentFailed => {
                self.handle_invoice_payment_failed(&webhook_event).await
            }
            WebhookEventType::SubscriptionDeleted => {
                self.handle_subscription_deleted(&webhook_event).await
            }
            WebhookEventType::SubscriptionUpdated => {
                // Acknowledge but don't process - subscription updates are handled elsewhere
                Ok(HandlePaymentWebhookResult::Acknowledged)
            }
            WebhookEventType::SubscriptionCreated => {
                // Subscription creation is handled via checkout completion
                Ok(HandlePaymentWebhookResult::Acknowledged)
            }
            WebhookEventType::TrialWillEnd => {
                // We don't use trials, acknowledge only
                Ok(HandlePaymentWebhookResult::Acknowledged)
            }
            WebhookEventType::Unknown(_) => Ok(HandlePaymentWebhookResult::Ignored),
        }
    }

    async fn handle_checkout_completed(
        &self,
        webhook_event: &WebhookEvent,
    ) -> Result<HandlePaymentWebhookResult, MembershipError> {
        let (customer_id, subscription_id) = match &webhook_event.data {
            WebhookEventData::Checkout {
                customer_id,
                subscription_id,
                ..
            } => (customer_id.clone(), subscription_id.clone()),
            _ => {
                return Err(MembershipError::infrastructure(
                    "Unexpected webhook data type for checkout.session.completed",
                ))
            }
        };

        // Find membership by customer ID
        let mut membership = self
            .repository
            .find_by_stripe_customer_id(&customer_id)
            .await?
            .ok_or_else(|| {
                MembershipError::infrastructure(format!(
                    "No membership found for customer {}",
                    customer_id
                ))
            })?;

        // Activate the membership
        let now = Timestamp::now();
        let period_end = now.add_days(if membership.tier.is_annual() { 365 } else { 30 });

        membership
            .activate(now, period_end, subscription_id)
            .map_err(|e| {
                MembershipError::invalid_state(format!("{:?}", membership.status), e.to_string())
            })?;

        // Update in repository
        self.repository.update(&membership).await?;

        // Publish event
        let event = MembershipEvent::Activated {
            event_id: EventId::new(),
            membership_id: membership.id,
            user_id: membership.user_id.clone(),
            tier: membership.tier,
            period_start: now,
            period_end,
            occurred_at: now,
        };

        let envelope = event.to_envelope();
        self.event_publisher.publish(envelope).await?;

        Ok(HandlePaymentWebhookResult::MembershipActivated {
            membership_id: membership.id.to_string(),
            user_id: membership.user_id.to_string(),
        })
    }

    async fn handle_invoice_paid(
        &self,
        webhook_event: &WebhookEvent,
    ) -> Result<HandlePaymentWebhookResult, MembershipError> {
        let subscription_id = match &webhook_event.data {
            WebhookEventData::Invoice {
                subscription_id, ..
            } => subscription_id.clone(),
            _ => {
                return Err(MembershipError::infrastructure(
                    "Unexpected webhook data type for invoice.paid",
                ))
            }
        };

        let subscription_id = subscription_id.ok_or_else(|| {
            MembershipError::infrastructure("Invoice has no subscription ID")
        })?;

        // Find membership by subscription ID
        let mut membership = self
            .repository
            .find_by_stripe_subscription_id(&subscription_id)
            .await?
            .ok_or_else(|| {
                MembershipError::infrastructure(format!(
                    "No membership found for subscription {}",
                    subscription_id
                ))
            })?;

        // Renew or recover the membership
        let now = Timestamp::now();
        let period_end = now.add_days(if membership.tier.is_annual() { 365 } else { 30 });

        let was_past_due = membership.status == crate::domain::membership::MembershipStatus::PastDue;

        if was_past_due {
            membership.recover_payment(period_end).map_err(|e| {
                MembershipError::invalid_state(format!("{:?}", membership.status), e.to_string())
            })?;

            // Update and publish recovery event
            self.repository.update(&membership).await?;

            let event = MembershipEvent::PaymentRecovered {
                event_id: EventId::new(),
                membership_id: membership.id,
                user_id: membership.user_id.clone(),
                occurred_at: now,
            };
            let envelope = event.to_envelope();
            self.event_publisher.publish(envelope).await?;
        } else {
            // Standard renewal
            membership.renew(now, period_end).map_err(|e| {
                MembershipError::invalid_state(format!("{:?}", membership.status), e.to_string())
            })?;

            // Update and publish renewal event
            self.repository.update(&membership).await?;

            let event = MembershipEvent::Renewed {
                event_id: EventId::new(),
                membership_id: membership.id,
                user_id: membership.user_id.clone(),
                new_period_start: now,
                new_period_end: period_end,
                occurred_at: now,
            };
            let envelope = event.to_envelope();
            self.event_publisher.publish(envelope).await?;
        }

        Ok(HandlePaymentWebhookResult::MembershipRenewed {
            membership_id: membership.id.to_string(),
            user_id: membership.user_id.to_string(),
        })
    }

    async fn handle_invoice_payment_failed(
        &self,
        webhook_event: &WebhookEvent,
    ) -> Result<HandlePaymentWebhookResult, MembershipError> {
        let subscription_id = match &webhook_event.data {
            WebhookEventData::Invoice {
                subscription_id, ..
            } => subscription_id.clone(),
            _ => {
                return Err(MembershipError::infrastructure(
                    "Unexpected webhook data type for invoice.payment_failed",
                ))
            }
        };

        let subscription_id = subscription_id.ok_or_else(|| {
            MembershipError::infrastructure("Invoice has no subscription ID")
        })?;

        // Find membership by subscription ID
        let mut membership = self
            .repository
            .find_by_stripe_subscription_id(&subscription_id)
            .await?
            .ok_or_else(|| {
                MembershipError::infrastructure(format!(
                    "No membership found for subscription {}",
                    subscription_id
                ))
            })?;

        // Mark as past due
        membership.mark_past_due().map_err(|e| {
            MembershipError::invalid_state(format!("{:?}", membership.status), e.to_string())
        })?;

        self.repository.update(&membership).await?;

        // Publish event
        let now = Timestamp::now();
        let event = MembershipEvent::PaymentFailed {
            event_id: EventId::new(),
            membership_id: membership.id,
            user_id: membership.user_id.clone(),
            attempt_count: 1, // Stripe provides this in the actual event
            next_retry_at: Some(now.add_days(3)), // Typical retry schedule
            occurred_at: now,
        };

        let envelope = event.to_envelope();
        self.event_publisher.publish(envelope).await?;

        Ok(HandlePaymentWebhookResult::PaymentFailed {
            membership_id: membership.id.to_string(),
            user_id: membership.user_id.to_string(),
            attempt_count: 1,
        })
    }

    async fn handle_subscription_deleted(
        &self,
        webhook_event: &WebhookEvent,
    ) -> Result<HandlePaymentWebhookResult, MembershipError> {
        let subscription_id = match &webhook_event.data {
            WebhookEventData::Subscription {
                subscription_id, ..
            } => subscription_id.clone(),
            _ => {
                return Err(MembershipError::infrastructure(
                    "Unexpected webhook data type for subscription.deleted",
                ))
            }
        };

        // Find membership by subscription ID
        let mut membership = self
            .repository
            .find_by_stripe_subscription_id(&subscription_id)
            .await?
            .ok_or_else(|| {
                MembershipError::infrastructure(format!(
                    "No membership found for subscription {}",
                    subscription_id
                ))
            })?;

        // Expire the membership
        membership.expire().map_err(|e| {
            MembershipError::invalid_state(format!("{:?}", membership.status), e.to_string())
        })?;

        self.repository.update(&membership).await?;

        // Publish event
        let now = Timestamp::now();
        let event = MembershipEvent::Expired {
            event_id: EventId::new(),
            membership_id: membership.id,
            user_id: membership.user_id.clone(),
            reason: ExpiredReason::CancelledPeriodEnd,
            occurred_at: now,
        };

        let envelope = event.to_envelope();
        self.event_publisher.publish(envelope).await?;

        Ok(HandlePaymentWebhookResult::MembershipExpired {
            membership_id: membership.id.to_string(),
            user_id: membership.user_id.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{DomainError, EventEnvelope, MembershipId, UserId};
    use crate::domain::membership::{Membership, MembershipStatus, MembershipTier};
    use crate::ports::{
        CheckoutSession, CreateCheckoutRequest, CreateCustomerRequest, CreateSubscriptionRequest,
        Customer, PaymentError, PaymentErrorCode, PortalSession, Subscription, SubscriptionStatus,
    };
    use async_trait::async_trait;
    use std::sync::Mutex;

    // ════════════════════════════════════════════════════════════════════════════
    // Mock Implementations
    // ════════════════════════════════════════════════════════════════════════════

    struct MockMembershipRepository {
        memberships: Mutex<Vec<Membership>>,
    }

    impl MockMembershipRepository {
        fn new() -> Self {
            Self {
                memberships: Mutex::new(Vec::new()),
            }
        }

        fn with_membership(membership: Membership) -> Self {
            Self {
                memberships: Mutex::new(vec![membership]),
            }
        }

        fn get_memberships(&self) -> Vec<Membership> {
            self.memberships.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl MembershipRepository for MockMembershipRepository {
        async fn save(&self, membership: &Membership) -> Result<(), DomainError> {
            self.memberships.lock().unwrap().push(membership.clone());
            Ok(())
        }

        async fn update(&self, membership: &Membership) -> Result<(), DomainError> {
            let mut memberships = self.memberships.lock().unwrap();
            if let Some(m) = memberships.iter_mut().find(|m| m.id == membership.id) {
                *m = membership.clone();
            }
            Ok(())
        }

        async fn find_by_id(
            &self,
            id: &MembershipId,
        ) -> Result<Option<Membership>, DomainError> {
            let memberships = self.memberships.lock().unwrap();
            Ok(memberships.iter().find(|m| &m.id == id).cloned())
        }

        async fn find_by_user_id(
            &self,
            user_id: &UserId,
        ) -> Result<Option<Membership>, DomainError> {
            let memberships = self.memberships.lock().unwrap();
            Ok(memberships.iter().find(|m| &m.user_id == user_id).cloned())
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
            subscription_id: &str,
        ) -> Result<Option<Membership>, DomainError> {
            let memberships = self.memberships.lock().unwrap();
            Ok(memberships
                .iter()
                .find(|m| m.stripe_subscription_id.as_deref() == Some(subscription_id))
                .cloned())
        }

        async fn find_by_stripe_customer_id(
            &self,
            customer_id: &str,
        ) -> Result<Option<Membership>, DomainError> {
            let memberships = self.memberships.lock().unwrap();
            Ok(memberships
                .iter()
                .find(|m| m.stripe_customer_id.as_deref() == Some(customer_id))
                .cloned())
        }
    }

    struct MockPaymentProvider {
        webhook_event: Option<WebhookEvent>,
        fail_verify: bool,
    }

    impl MockPaymentProvider {
        fn with_event(event: WebhookEvent) -> Self {
            Self {
                webhook_event: Some(event),
                fail_verify: false,
            }
        }

        fn failing() -> Self {
            Self {
                webhook_event: None,
                fail_verify: true,
            }
        }
    }

    #[async_trait]
    impl PaymentProvider for MockPaymentProvider {
        async fn create_customer(
            &self,
            _request: CreateCustomerRequest,
        ) -> Result<Customer, PaymentError> {
            Ok(Customer {
                id: "cus_123".to_string(),
                email: "test@example.com".to_string(),
                name: None,
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
            if self.fail_verify {
                return Err(PaymentError::new(
                    PaymentErrorCode::InvalidWebhook,
                    "Invalid signature",
                ));
            }
            self.webhook_event
                .clone()
                .ok_or_else(|| PaymentError::new(PaymentErrorCode::InvalidWebhook, "No event"))
        }
    }

    struct MockEventPublisher {
        published_events: Mutex<Vec<EventEnvelope>>,
    }

    impl MockEventPublisher {
        fn new() -> Self {
            Self {
                published_events: Mutex::new(Vec::new()),
            }
        }

        fn published_events(&self) -> Vec<EventEnvelope> {
            self.published_events.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl EventPublisher for MockEventPublisher {
        async fn publish(&self, event: EventEnvelope) -> Result<(), DomainError> {
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

    fn pending_membership() -> Membership {
        Membership::create_paid(
            MembershipId::new(),
            test_user_id(),
            MembershipTier::Monthly,
            "cus_123".to_string(),
        )
    }

    fn active_membership() -> Membership {
        let mut m = Membership::create_paid(
            MembershipId::new(),
            test_user_id(),
            MembershipTier::Monthly,
            "cus_123".to_string(),
        );
        m.activate(
            Timestamp::now(),
            Timestamp::now().add_days(30),
            Some("sub_123".to_string()),
        )
        .unwrap();
        m
    }

    fn checkout_completed_event() -> WebhookEvent {
        WebhookEvent {
            id: "evt_123".to_string(),
            event_type: WebhookEventType::CheckoutSessionCompleted,
            data: WebhookEventData::Checkout {
                session_id: "cs_123".to_string(),
                customer_id: "cus_123".to_string(),
                subscription_id: Some("sub_123".to_string()),
                user_id: Some("test-user-123".to_string()),
            },
            created_at: 1234567890,
        }
    }

    fn invoice_paid_event() -> WebhookEvent {
        WebhookEvent {
            id: "evt_124".to_string(),
            event_type: WebhookEventType::InvoicePaid,
            data: WebhookEventData::Invoice {
                invoice_id: "in_123".to_string(),
                customer_id: "cus_123".to_string(),
                subscription_id: Some("sub_123".to_string()),
                amount_paid: 2900,
                currency: "usd".to_string(),
            },
            created_at: 1234567890,
        }
    }

    fn invoice_failed_event() -> WebhookEvent {
        WebhookEvent {
            id: "evt_125".to_string(),
            event_type: WebhookEventType::InvoicePaymentFailed,
            data: WebhookEventData::Invoice {
                invoice_id: "in_123".to_string(),
                customer_id: "cus_123".to_string(),
                subscription_id: Some("sub_123".to_string()),
                amount_paid: 0,
                currency: "usd".to_string(),
            },
            created_at: 1234567890,
        }
    }

    fn subscription_deleted_event() -> WebhookEvent {
        WebhookEvent {
            id: "evt_126".to_string(),
            event_type: WebhookEventType::SubscriptionDeleted,
            data: WebhookEventData::Subscription {
                subscription_id: "sub_123".to_string(),
                customer_id: "cus_123".to_string(),
                status: SubscriptionStatus::Canceled,
                current_period_end: 1237246290,
            },
            created_at: 1234567890,
        }
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Checkout Completed Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn checkout_completed_activates_pending_membership() {
        let membership = pending_membership();
        let repo = Arc::new(MockMembershipRepository::with_membership(membership));
        let payment = Arc::new(MockPaymentProvider::with_event(checkout_completed_event()));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = HandlePaymentWebhookHandler::new(repo.clone(), payment, publisher);

        let cmd = HandlePaymentWebhookCommand {
            payload: vec![],
            signature: "valid".to_string(),
        };

        let result = handler.handle(cmd).await.unwrap();
        assert!(matches!(
            result,
            HandlePaymentWebhookResult::MembershipActivated { .. }
        ));

        let memberships = repo.get_memberships();
        assert_eq!(memberships[0].status, MembershipStatus::Active);
    }

    #[tokio::test]
    async fn checkout_completed_publishes_activated_event() {
        let membership = pending_membership();
        let repo = Arc::new(MockMembershipRepository::with_membership(membership));
        let payment = Arc::new(MockPaymentProvider::with_event(checkout_completed_event()));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = HandlePaymentWebhookHandler::new(repo, payment, publisher.clone());

        let cmd = HandlePaymentWebhookCommand {
            payload: vec![],
            signature: "valid".to_string(),
        };

        handler.handle(cmd).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "membership.activated");
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Invoice Paid Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn invoice_paid_renews_active_membership() {
        let membership = active_membership();
        let repo = Arc::new(MockMembershipRepository::with_membership(membership));
        let payment = Arc::new(MockPaymentProvider::with_event(invoice_paid_event()));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = HandlePaymentWebhookHandler::new(repo.clone(), payment, publisher);

        let cmd = HandlePaymentWebhookCommand {
            payload: vec![],
            signature: "valid".to_string(),
        };

        let result = handler.handle(cmd).await.unwrap();
        assert!(matches!(
            result,
            HandlePaymentWebhookResult::MembershipRenewed { .. }
        ));
    }

    #[tokio::test]
    async fn invoice_paid_publishes_renewed_event() {
        let membership = active_membership();
        let repo = Arc::new(MockMembershipRepository::with_membership(membership));
        let payment = Arc::new(MockPaymentProvider::with_event(invoice_paid_event()));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = HandlePaymentWebhookHandler::new(repo, payment, publisher.clone());

        let cmd = HandlePaymentWebhookCommand {
            payload: vec![],
            signature: "valid".to_string(),
        };

        handler.handle(cmd).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "membership.renewed");
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Invoice Payment Failed Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn invoice_failed_marks_membership_past_due() {
        let membership = active_membership();
        let repo = Arc::new(MockMembershipRepository::with_membership(membership));
        let payment = Arc::new(MockPaymentProvider::with_event(invoice_failed_event()));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = HandlePaymentWebhookHandler::new(repo.clone(), payment, publisher);

        let cmd = HandlePaymentWebhookCommand {
            payload: vec![],
            signature: "valid".to_string(),
        };

        let result = handler.handle(cmd).await.unwrap();
        assert!(matches!(
            result,
            HandlePaymentWebhookResult::PaymentFailed { .. }
        ));

        let memberships = repo.get_memberships();
        assert_eq!(memberships[0].status, MembershipStatus::PastDue);
    }

    #[tokio::test]
    async fn invoice_failed_publishes_payment_failed_event() {
        let membership = active_membership();
        let repo = Arc::new(MockMembershipRepository::with_membership(membership));
        let payment = Arc::new(MockPaymentProvider::with_event(invoice_failed_event()));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = HandlePaymentWebhookHandler::new(repo, payment, publisher.clone());

        let cmd = HandlePaymentWebhookCommand {
            payload: vec![],
            signature: "valid".to_string(),
        };

        handler.handle(cmd).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "membership.payment_failed");
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Subscription Deleted Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn subscription_deleted_expires_membership() {
        let mut membership = active_membership();
        membership.cancel().unwrap(); // Must be cancelled first per state machine
        let repo = Arc::new(MockMembershipRepository::with_membership(membership));
        let payment = Arc::new(MockPaymentProvider::with_event(subscription_deleted_event()));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = HandlePaymentWebhookHandler::new(repo.clone(), payment, publisher);

        let cmd = HandlePaymentWebhookCommand {
            payload: vec![],
            signature: "valid".to_string(),
        };

        let result = handler.handle(cmd).await.unwrap();
        assert!(matches!(
            result,
            HandlePaymentWebhookResult::MembershipExpired { .. }
        ));

        let memberships = repo.get_memberships();
        assert_eq!(memberships[0].status, MembershipStatus::Expired);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Error Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn fails_with_invalid_webhook_signature() {
        let repo = Arc::new(MockMembershipRepository::new());
        let payment = Arc::new(MockPaymentProvider::failing());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = HandlePaymentWebhookHandler::new(repo, payment, publisher);

        let cmd = HandlePaymentWebhookCommand {
            payload: vec![],
            signature: "invalid".to_string(),
        };

        let result = handler.handle(cmd).await;
        assert!(matches!(
            result,
            Err(MembershipError::InvalidWebhookSignature)
        ));
    }

    #[tokio::test]
    async fn ignores_unknown_event_types() {
        let event = WebhookEvent {
            id: "evt_unknown".to_string(),
            event_type: WebhookEventType::Unknown("customer.created".to_string()),
            data: WebhookEventData::Raw {
                json: "{}".to_string(),
            },
            created_at: 1234567890,
        };

        let repo = Arc::new(MockMembershipRepository::new());
        let payment = Arc::new(MockPaymentProvider::with_event(event));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = HandlePaymentWebhookHandler::new(repo, payment, publisher.clone());

        let cmd = HandlePaymentWebhookCommand {
            payload: vec![],
            signature: "valid".to_string(),
        };

        let result = handler.handle(cmd).await.unwrap();
        assert!(matches!(result, HandlePaymentWebhookResult::Ignored));
        assert!(publisher.published_events().is_empty());
    }
}
