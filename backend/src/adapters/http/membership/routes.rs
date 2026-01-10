//! Axum router configuration for membership endpoints.
//!
//! This module defines the route structure for membership-related API endpoints
//! and wires them to their corresponding handlers.

use axum::{
    routing::{get, post},
    Router,
};

use super::handlers::{
    cancel_membership, check_access, create_checkout, create_free_membership, get_membership,
    get_membership_stats, get_portal_url, get_tier_limits, handle_stripe_webhook,
    MembershipAppState,
};

/// Create the membership API router.
///
/// # Routes
///
/// ## User Endpoints (require authentication)
/// - `GET /` - Get current user's membership details
/// - `GET /limits` - Get tier limits for current user
/// - `GET /access` - Check if user has access
/// - `GET /portal` - Get Stripe customer portal URL
/// - `POST /free` - Create free membership with promo code
/// - `POST /checkout` - Start paid checkout flow
/// - `POST /cancel` - Cancel membership
///
/// ## Admin Endpoints (require admin role)
/// - `GET /stats` - Get membership statistics
///
/// ## Webhook Endpoints (no auth, signature verified)
/// - `POST /webhooks/stripe` - Handle Stripe webhooks
pub fn membership_routes() -> Router<MembershipAppState> {
    Router::new()
        // User endpoints
        .route("/", get(get_membership))
        .route("/limits", get(get_tier_limits))
        .route("/access", get(check_access))
        .route("/portal", get(get_portal_url))
        .route("/free", post(create_free_membership))
        .route("/checkout", post(create_checkout))
        .route("/cancel", post(cancel_membership))
        // Admin endpoints
        .route("/stats", get(get_membership_stats))
}

/// Create the Stripe webhook router.
///
/// This is separate from the main membership routes because webhooks
/// don't require user authentication (they're verified via signature).
///
/// # Routes
/// - `POST /stripe` - Handle Stripe webhooks
pub fn webhook_routes() -> Router<MembershipAppState> {
    Router::new().route("/stripe", post(handle_stripe_webhook))
}

/// Create the complete membership module router.
///
/// Combines user/admin routes and webhook routes into a single router
/// suitable for mounting at `/api/membership` and `/api/webhooks`.
///
/// # Example
///
/// ```ignore
/// use axum::Router;
/// use crate::adapters::http::membership::{membership_router, MembershipAppState};
///
/// let app_state = MembershipAppState { /* ... */ };
/// let app = Router::new()
///     .nest("/api/membership", membership_routes())
///     .nest("/api/webhooks", webhook_routes())
///     .with_state(app_state);
/// ```
pub fn membership_router() -> Router<MembershipAppState> {
    Router::new()
        .nest("/membership", membership_routes())
        .nest("/webhooks", webhook_routes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::domain::foundation::{DomainError, MembershipId, Timestamp, UserId};
    use crate::domain::membership::{Membership, MembershipStatus, MembershipTier, TierLimits};
    use crate::ports::{
        AccessChecker, AccessResult, CheckoutSession, CreateCheckoutRequest,
        CreateCustomerRequest, CreateSubscriptionRequest, Customer, EventPublisher,
        MembershipReader, MembershipRepository, MembershipStatistics, MembershipSummary,
        MembershipView, PaymentError, PaymentProvider, PortalSession, PromoCodeValidation,
        PromoCodeValidator, Subscription, SubscriptionStatus, UsageStats, WebhookEvent,
        WebhookEventData, WebhookEventType,
    };
    use async_trait::async_trait;
    use std::sync::Mutex;

    // ════════════════════════════════════════════════════════════════════════════
    // Mock Implementations (shared with handlers tests)
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
    }

    #[async_trait]
    impl MembershipRepository for MockMembershipRepository {
        async fn save(&self, membership: &Membership) -> Result<(), DomainError> {
            self.memberships.lock().unwrap().push(membership.clone());
            Ok(())
        }

        async fn update(&self, _membership: &Membership) -> Result<(), DomainError> {
            Ok(())
        }

        async fn find_by_id(
            &self,
            id: &MembershipId,
        ) -> Result<Option<Membership>, DomainError> {
            Ok(self
                .memberships
                .lock()
                .unwrap()
                .iter()
                .find(|m| &m.id == id)
                .cloned())
        }

        async fn find_by_user_id(
            &self,
            user_id: &UserId,
        ) -> Result<Option<Membership>, DomainError> {
            Ok(self
                .memberships
                .lock()
                .unwrap()
                .iter()
                .find(|m| &m.user_id == user_id)
                .cloned())
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
            _id: &str,
        ) -> Result<Option<Membership>, DomainError> {
            Ok(None)
        }

        async fn find_by_stripe_customer_id(
            &self,
            _id: &str,
        ) -> Result<Option<Membership>, DomainError> {
            Ok(None)
        }
    }

    struct MockMembershipReader {
        view: Option<MembershipView>,
    }

    impl MockMembershipReader {
        fn with_view(view: MembershipView) -> Self {
            Self { view: Some(view) }
        }
    }

    #[async_trait]
    impl MembershipReader for MockMembershipReader {
        async fn get_by_user(
            &self,
            _user_id: &UserId,
        ) -> Result<Option<MembershipView>, DomainError> {
            Ok(self.view.clone())
        }

        async fn check_access(&self, _user_id: &UserId) -> Result<bool, DomainError> {
            Ok(self.view.as_ref().map(|v| v.has_access).unwrap_or(false))
        }

        async fn get_tier(
            &self,
            _user_id: &UserId,
        ) -> Result<Option<MembershipTier>, DomainError> {
            Ok(self.view.as_ref().map(|v| v.tier))
        }

        async fn list_expiring(&self, _days: u32) -> Result<Vec<MembershipSummary>, DomainError> {
            Ok(vec![])
        }

        async fn get_statistics(&self) -> Result<MembershipStatistics, DomainError> {
            Ok(MembershipStatistics::default())
        }
    }

    struct MockAccessChecker;

    #[async_trait]
    impl AccessChecker for MockAccessChecker {
        async fn can_create_session(&self, _user_id: &UserId) -> Result<AccessResult, DomainError> {
            Ok(AccessResult::Allowed)
        }

        async fn can_create_cycle(
            &self,
            _user_id: &UserId,
            _session_id: &crate::domain::foundation::SessionId,
        ) -> Result<AccessResult, DomainError> {
            Ok(AccessResult::Allowed)
        }

        async fn can_export(&self, _user_id: &UserId) -> Result<AccessResult, DomainError> {
            Ok(AccessResult::Allowed)
        }

        async fn get_tier_limits(&self, _user_id: &UserId) -> Result<TierLimits, DomainError> {
            Ok(TierLimits::for_tier(MembershipTier::Annual))
        }

        async fn get_usage(&self, _user_id: &UserId) -> Result<UsageStats, DomainError> {
            Ok(UsageStats::new())
        }
    }

    struct MockPromoCodeValidator;

    #[async_trait]
    impl PromoCodeValidator for MockPromoCodeValidator {
        async fn validate(
            &self,
            _code: &crate::domain::membership::PromoCode,
        ) -> Result<PromoCodeValidation, DomainError> {
            Ok(PromoCodeValidation::valid_free(365))
        }

        async fn record_redemption(
            &self,
            _code: &crate::domain::membership::PromoCode,
        ) -> Result<(), DomainError> {
            Ok(())
        }

        async fn get_usage_count(
            &self,
            _code: &crate::domain::membership::PromoCode,
        ) -> Result<Option<u32>, DomainError> {
            Ok(Some(0))
        }
    }

    struct MockPaymentProvider;

    #[async_trait]
    impl PaymentProvider for MockPaymentProvider {
        async fn create_customer(
            &self,
            _request: CreateCustomerRequest,
        ) -> Result<Customer, PaymentError> {
            Ok(Customer {
                id: "cus_test123".to_string(),
                email: "test@example.com".to_string(),
                name: Some("Test User".to_string()),
                created_at: 1704067200,
            })
        }

        async fn get_customer(
            &self,
            customer_id: &str,
        ) -> Result<Option<Customer>, PaymentError> {
            Ok(Some(Customer {
                id: customer_id.to_string(),
                email: "test@example.com".to_string(),
                name: Some("Test User".to_string()),
                created_at: 1704067200,
            }))
        }

        async fn create_subscription(
            &self,
            _request: CreateSubscriptionRequest,
        ) -> Result<Subscription, PaymentError> {
            Ok(Subscription {
                id: "sub_test123".to_string(),
                customer_id: "cus_test123".to_string(),
                status: SubscriptionStatus::Active,
                current_period_start: 1704067200,
                current_period_end: 1735689600,
                cancel_at_period_end: false,
                canceled_at: None,
            })
        }

        async fn get_subscription(
            &self,
            subscription_id: &str,
        ) -> Result<Option<Subscription>, PaymentError> {
            Ok(Some(Subscription {
                id: subscription_id.to_string(),
                customer_id: "cus_test123".to_string(),
                status: SubscriptionStatus::Active,
                current_period_start: 1704067200,
                current_period_end: 1735689600,
                cancel_at_period_end: false,
                canceled_at: None,
            }))
        }

        async fn cancel_subscription(
            &self,
            subscription_id: &str,
            at_period_end: bool,
        ) -> Result<Subscription, PaymentError> {
            Ok(Subscription {
                id: subscription_id.to_string(),
                customer_id: "cus_test123".to_string(),
                status: if at_period_end {
                    SubscriptionStatus::Active
                } else {
                    SubscriptionStatus::Canceled
                },
                current_period_start: 1704067200,
                current_period_end: 1735689600,
                cancel_at_period_end: at_period_end,
                canceled_at: Some(1704153600),
            })
        }

        async fn update_subscription(
            &self,
            subscription_id: &str,
            _new_tier: MembershipTier,
        ) -> Result<Subscription, PaymentError> {
            Ok(Subscription {
                id: subscription_id.to_string(),
                customer_id: "cus_test123".to_string(),
                status: SubscriptionStatus::Active,
                current_period_start: 1704067200,
                current_period_end: 1735689600,
                cancel_at_period_end: false,
                canceled_at: None,
            })
        }

        async fn create_checkout_session(
            &self,
            _request: CreateCheckoutRequest,
        ) -> Result<CheckoutSession, PaymentError> {
            Ok(CheckoutSession {
                id: "cs_test123".to_string(),
                url: "https://checkout.stripe.com/test".to_string(),
                expires_at: 1704153600,
            })
        }

        async fn create_portal_session(
            &self,
            _customer_id: &str,
            _return_url: &str,
        ) -> Result<PortalSession, PaymentError> {
            Ok(PortalSession {
                id: "bps_test123".to_string(),
                url: "https://billing.stripe.com/test".to_string(),
            })
        }

        async fn verify_webhook(
            &self,
            _payload: &[u8],
            _signature: &str,
        ) -> Result<WebhookEvent, PaymentError> {
            Ok(WebhookEvent {
                id: "evt_test123".to_string(),
                event_type: WebhookEventType::CheckoutSessionCompleted,
                data: WebhookEventData::Checkout {
                    session_id: "cs_test123".to_string(),
                    customer_id: "cus_test123".to_string(),
                    subscription_id: Some("sub_test123".to_string()),
                    user_id: Some("test-user-123".to_string()),
                },
                created_at: 1704067200,
            })
        }
    }

    struct MockEventPublisher {
        events: Mutex<Vec<crate::domain::foundation::EventEnvelope>>,
    }

    impl MockEventPublisher {
        fn new() -> Self {
            Self {
                events: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl EventPublisher for MockEventPublisher {
        async fn publish(
            &self,
            event: crate::domain::foundation::EventEnvelope,
        ) -> Result<(), DomainError> {
            self.events.lock().unwrap().push(event);
            Ok(())
        }

        async fn publish_all(
            &self,
            events: Vec<crate::domain::foundation::EventEnvelope>,
        ) -> Result<(), DomainError> {
            self.events.lock().unwrap().extend(events);
            Ok(())
        }
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Test Helpers
    // ════════════════════════════════════════════════════════════════════════════

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    fn test_membership_view() -> MembershipView {
        MembershipView {
            id: MembershipId::new(),
            user_id: test_user_id(),
            tier: MembershipTier::Annual,
            status: MembershipStatus::Active,
            has_access: true,
            days_remaining: 300,
            period_end: Timestamp::now().add_days(300),
            promo_code: Some("WORKSHOP2026".to_string()),
            created_at: Timestamp::now(),
        }
    }

    fn test_state() -> MembershipAppState {
        MembershipAppState {
            membership_repository: Arc::new(MockMembershipRepository::new()),
            membership_reader: Arc::new(MockMembershipReader::with_view(test_membership_view())),
            promo_code_validator: Arc::new(MockPromoCodeValidator),
            payment_provider: Arc::new(MockPaymentProvider),
            access_checker: Arc::new(MockAccessChecker),
            event_publisher: Arc::new(MockEventPublisher::new()),
        }
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Router Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn membership_routes_creates_router() {
        let router = membership_routes();
        // Just verify it creates without panic
        let _: Router<()> = router.with_state(test_state());
    }

    #[test]
    fn webhook_routes_creates_router() {
        let router = webhook_routes();
        let _: Router<()> = router.with_state(test_state());
    }

    #[test]
    fn membership_router_creates_combined_router() {
        let router = membership_router();
        let _: Router<()> = router.with_state(test_state());
    }

    // Note: Full integration tests with HTTP requests would go in a separate
    // integration test file with proper test fixtures and auth middleware.
}
