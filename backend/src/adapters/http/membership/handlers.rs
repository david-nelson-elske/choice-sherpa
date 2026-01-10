//! HTTP handlers for membership endpoints.
//!
//! These handlers connect Axum routes to application layer command/query handlers.

use std::sync::Arc;

use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::application::handlers::membership::{
    CancelMembershipCommand, CancelMembershipHandler, CheckAccessHandler, CheckAccessQuery,
    CreateFreeMembershipCommand, CreateFreeMembershipHandler, CreatePaidMembershipCommand,
    CreatePaidMembershipHandler, GetMembershipHandler, GetMembershipQuery,
    GetMembershipStatsHandler, GetMembershipStatsQuery, HandlePaymentWebhookCommand,
    HandlePaymentWebhookHandler,
};
use crate::domain::foundation::UserId;
use crate::domain::membership::MembershipError;
use crate::ports::{
    AccessChecker, EventPublisher, MembershipReader, MembershipRepository, PaymentProvider,
    PromoCodeValidator,
};

use super::dto::{
    AccessCheckResponse, CancelMembershipRequest, CheckoutResponse, CreateFreeMembershipRequest,
    CreatePaidMembershipRequest, ErrorResponse, MembershipResponse, MembershipStatsResponse,
    MembershipViewResponse, PortalResponse, TierLimitsResponse,
};

// ════════════════════════════════════════════════════════════════════════════════
// Application State
// ════════════════════════════════════════════════════════════════════════════════

/// Shared application state containing all dependencies.
///
/// This struct is cloned for each request and contains Arc-wrapped dependencies
/// for efficient sharing across handlers.
#[derive(Clone)]
pub struct MembershipAppState {
    pub membership_repository: Arc<dyn MembershipRepository>,
    pub membership_reader: Arc<dyn MembershipReader>,
    pub promo_code_validator: Arc<dyn PromoCodeValidator>,
    pub payment_provider: Arc<dyn PaymentProvider>,
    pub access_checker: Arc<dyn AccessChecker>,
    pub event_publisher: Arc<dyn EventPublisher>,
}

impl MembershipAppState {
    /// Create handlers on demand from the shared state.
    pub fn get_membership_handler(&self) -> GetMembershipHandler {
        GetMembershipHandler::new(self.membership_reader.clone())
    }

    pub fn check_access_handler(&self) -> CheckAccessHandler {
        CheckAccessHandler::new(self.membership_reader.clone())
    }

    pub fn create_free_membership_handler(&self) -> CreateFreeMembershipHandler {
        CreateFreeMembershipHandler::new(
            self.membership_repository.clone(),
            self.promo_code_validator.clone(),
            self.event_publisher.clone(),
        )
    }

    pub fn create_paid_membership_handler(&self) -> CreatePaidMembershipHandler {
        CreatePaidMembershipHandler::new(
            self.membership_repository.clone(),
            self.payment_provider.clone(),
            self.event_publisher.clone(),
        )
    }

    pub fn cancel_membership_handler(&self) -> CancelMembershipHandler {
        CancelMembershipHandler::new(
            self.membership_repository.clone(),
            self.event_publisher.clone(),
        )
    }

    pub fn webhook_handler(&self) -> HandlePaymentWebhookHandler {
        HandlePaymentWebhookHandler::new(
            self.membership_repository.clone(),
            self.payment_provider.clone(),
            self.event_publisher.clone(),
        )
    }

    pub fn stats_handler(&self) -> GetMembershipStatsHandler {
        GetMembershipStatsHandler::new(self.membership_reader.clone())
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// User Context (would come from auth middleware in production)
// ════════════════════════════════════════════════════════════════════════════════

/// Authenticated user context extracted from request.
///
/// In production, this would be extracted from JWT/session by auth middleware.
/// For now, uses a header-based extraction for development/testing.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: UserId,
}

/// Rejection type for AuthenticatedUser extraction.
pub struct AuthenticationRequired;

impl IntoResponse for AuthenticationRequired {
    fn into_response(self) -> axum::response::Response {
        let error = ErrorResponse::new("AUTHENTICATION_REQUIRED", "Authentication is required");
        (StatusCode::UNAUTHORIZED, Json(error)).into_response()
    }
}

impl<S> axum::extract::FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthenticationRequired;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut axum::http::request::Parts,
        _state: &'life1 S,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            // In production, this would validate JWT token from Authorization header
            // For development, we accept an X-User-Id header
            let user_id = parts
                .headers
                .get("X-User-Id")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| UserId::new(s).ok())
                .ok_or(AuthenticationRequired)?;

            Ok(AuthenticatedUser { user_id })
        })
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Query Handlers (GET endpoints)
// ════════════════════════════════════════════════════════════════════════════════

/// GET /api/membership - Get current user's membership details
pub async fn get_membership(
    State(state): State<MembershipAppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, MembershipApiError> {
    let handler = state.get_membership_handler();
    let query = GetMembershipQuery {
        user_id: user.user_id,
    };

    let result = handler.handle(query).await?;

    let response = MembershipResponse {
        membership: result.map(MembershipViewResponse::from),
    };

    Ok(Json(response))
}

/// GET /api/membership/limits - Get tier limits for current user
pub async fn get_tier_limits(
    State(state): State<MembershipAppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, MembershipApiError> {
    let limits = state.access_checker.get_tier_limits(&user.user_id).await?;
    let response = TierLimitsResponse::from(limits);
    Ok(Json(response))
}

/// GET /api/membership/access - Check if user has access
pub async fn check_access(
    State(state): State<MembershipAppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, MembershipApiError> {
    let handler = state.check_access_handler();
    let query = CheckAccessQuery {
        user_id: user.user_id,
    };

    let result = handler.handle(query).await?;

    let response = AccessCheckResponse {
        has_access: result.has_access,
    };

    Ok(Json(response))
}

/// GET /api/membership/stats - Get membership statistics (admin only)
pub async fn get_membership_stats(
    State(state): State<MembershipAppState>,
    _user: AuthenticatedUser, // Would check admin role in production
) -> Result<impl IntoResponse, MembershipApiError> {
    let handler = state.stats_handler();
    let query = GetMembershipStatsQuery {};

    let result = handler.handle(query).await?;

    let response = MembershipStatsResponse::from(result);
    Ok(Json(response))
}

// ════════════════════════════════════════════════════════════════════════════════
// Command Handlers (POST endpoints)
// ════════════════════════════════════════════════════════════════════════════════

/// POST /api/membership/free - Create free membership with promo code
pub async fn create_free_membership(
    State(state): State<MembershipAppState>,
    user: AuthenticatedUser,
    Json(request): Json<CreateFreeMembershipRequest>,
) -> Result<impl IntoResponse, MembershipApiError> {
    let handler = state.create_free_membership_handler();
    let cmd = CreateFreeMembershipCommand {
        user_id: user.user_id,
        promo_code: request.promo_code,
    };

    let result = handler.handle(cmd).await?;

    // Convert to view response
    let view = crate::ports::MembershipView {
        id: result.membership.id,
        user_id: result.membership.user_id.clone(),
        tier: result.membership.tier,
        status: result.membership.status,
        has_access: result.membership.has_access(),
        days_remaining: result.membership.days_remaining(),
        period_end: result.membership.current_period_end,
        promo_code: result.membership.promo_code.clone(),
        created_at: result.membership.created_at,
    };

    let response = MembershipResponse {
        membership: Some(MembershipViewResponse::from(view)),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// POST /api/membership/checkout - Start paid checkout flow
pub async fn create_checkout(
    State(state): State<MembershipAppState>,
    user: AuthenticatedUser,
    Json(request): Json<CreatePaidMembershipRequest>,
) -> Result<impl IntoResponse, MembershipApiError> {
    let handler = state.create_paid_membership_handler();
    let cmd = CreatePaidMembershipCommand {
        user_id: user.user_id,
        email: request.email,
        tier: request.tier,
        success_url: request.success_url,
        cancel_url: request.cancel_url,
        promo_code: request.promo_code,
    };

    let result = handler.handle(cmd).await?;

    let response = CheckoutResponse {
        checkout_url: result.checkout_session.url,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// POST /api/membership/cancel - Cancel membership
///
/// Note: The `immediate` field in the request is currently ignored as cancellation
/// always takes effect at the end of the current billing period.
pub async fn cancel_membership(
    State(state): State<MembershipAppState>,
    user: AuthenticatedUser,
    Json(_request): Json<CancelMembershipRequest>,
) -> Result<impl IntoResponse, MembershipApiError> {
    let handler = state.cancel_membership_handler();
    let cmd = CancelMembershipCommand {
        user_id: user.user_id,
    };

    handler.handle(cmd).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/membership/portal - Get Stripe customer portal URL
pub async fn get_portal_url(
    State(state): State<MembershipAppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, MembershipApiError> {
    // Find membership to get Stripe customer ID
    let membership = state
        .membership_repository
        .find_by_user_id(&user.user_id)
        .await?
        .ok_or_else(|| MembershipError::not_found_for_user(user.user_id.clone()))?;

    let customer_id = membership
        .stripe_customer_id
        .clone()
        .ok_or_else(|| {
            MembershipError::validation("stripe_customer_id", "No Stripe customer associated")
        })?;

    // TODO: Get return URL from config or request
    let return_url = "/dashboard";

    let portal_session = state
        .payment_provider
        .create_portal_session(&customer_id, return_url)
        .await
        .map_err(|e| MembershipError::payment_failed(e.to_string()))?;

    let response = PortalResponse {
        portal_url: portal_session.url,
    };
    Ok(Json(response))
}

/// POST /api/webhooks/stripe - Handle Stripe webhook events
pub async fn handle_stripe_webhook(
    State(state): State<MembershipAppState>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, MembershipApiError> {
    // Extract Stripe signature header
    let signature = headers
        .get("Stripe-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            MembershipError::validation("Stripe-Signature", "Missing Stripe-Signature header")
        })?;

    let handler = state.webhook_handler();
    let cmd = HandlePaymentWebhookCommand {
        payload: body.to_vec(),
        signature: signature.to_string(),
    };

    handler.handle(cmd).await?;

    Ok(StatusCode::OK)
}

// ════════════════════════════════════════════════════════════════════════════════
// Error Handling
// ════════════════════════════════════════════════════════════════════════════════

/// API error type that converts domain errors to HTTP responses.
pub struct MembershipApiError(MembershipError);

impl From<MembershipError> for MembershipApiError {
    fn from(err: MembershipError) -> Self {
        Self(err)
    }
}

impl From<crate::domain::foundation::DomainError> for MembershipApiError {
    fn from(err: crate::domain::foundation::DomainError) -> Self {
        Self(MembershipError::infrastructure(err.to_string()))
    }
}

impl IntoResponse for MembershipApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_code) = match &self.0 {
            MembershipError::NotFound(_) | MembershipError::NotFoundForUser(_) => {
                (StatusCode::NOT_FOUND, "MEMBERSHIP_NOT_FOUND")
            }
            MembershipError::AlreadyExists(_) => (StatusCode::CONFLICT, "MEMBERSHIP_EXISTS"),
            MembershipError::Expired(_) => (StatusCode::PAYMENT_REQUIRED, "MEMBERSHIP_EXPIRED"),
            MembershipError::InvalidTier(_) => (StatusCode::BAD_REQUEST, "INVALID_TIER"),
            MembershipError::InvalidPromoCode { .. } => {
                (StatusCode::BAD_REQUEST, "INVALID_PROMO_CODE")
            }
            MembershipError::PromoCodeExhausted(_) => {
                (StatusCode::BAD_REQUEST, "PROMO_CODE_EXHAUSTED")
            }
            MembershipError::PaymentFailed { .. } => {
                (StatusCode::PAYMENT_REQUIRED, "PAYMENT_FAILED")
            }
            MembershipError::InvalidState { .. } => {
                (StatusCode::CONFLICT, "INVALID_STATE_TRANSITION")
            }
            MembershipError::InvalidWebhookSignature => {
                (StatusCode::UNAUTHORIZED, "INVALID_WEBHOOK_SIGNATURE")
            }
            MembershipError::ValidationFailed { .. } => {
                (StatusCode::BAD_REQUEST, "VALIDATION_FAILED")
            }
            MembershipError::Infrastructure(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR")
            }
        };

        // Use the error's built-in message() method for consistent messaging
        let message = self.0.message();
        let body = ErrorResponse::new(error_code, message);
        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{DomainError, MembershipId, Timestamp};
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

        #[allow(dead_code)]
        fn with_membership(membership: Membership) -> Self {
            Self {
                memberships: Mutex::new(vec![membership]),
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
        fn new() -> Self {
            Self { view: None }
        }

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

    struct MockAccessChecker {
        tier: MembershipTier,
    }

    impl MockAccessChecker {
        fn new() -> Self {
            Self {
                tier: MembershipTier::Annual,
            }
        }
    }

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
            Ok(TierLimits::for_tier(self.tier))
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

    fn test_user() -> AuthenticatedUser {
        AuthenticatedUser {
            user_id: test_user_id(),
        }
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
            access_checker: Arc::new(MockAccessChecker::new()),
            event_publisher: Arc::new(MockEventPublisher::new()),
        }
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Handler Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn get_membership_returns_view_when_exists() {
        let state = test_state();
        let user = test_user();

        let result = get_membership(State(state), user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn get_membership_returns_none_when_not_exists() {
        let state = MembershipAppState {
            membership_reader: Arc::new(MockMembershipReader::new()),
            ..test_state()
        };
        let user = test_user();

        let result = get_membership(State(state), user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn get_tier_limits_returns_limits() {
        let state = test_state();
        let user = test_user();

        let result = get_tier_limits(State(state), user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn check_access_returns_access_status() {
        let state = test_state();
        let user = test_user();

        let result = check_access(State(state), user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn get_membership_stats_returns_statistics() {
        let state = test_state();
        let user = test_user();

        let result = get_membership_stats(State(state), user).await;
        assert!(result.is_ok());
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Error Mapping Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn api_error_maps_not_found_to_404() {
        let err = MembershipApiError(MembershipError::not_found_for_user(test_user_id()));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn api_error_maps_not_found_by_id_to_404() {
        let err = MembershipApiError(MembershipError::not_found(MembershipId::new()));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn api_error_maps_already_exists_to_409() {
        let err = MembershipApiError(MembershipError::already_exists(test_user_id()));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn api_error_maps_expired_to_402() {
        let err = MembershipApiError(MembershipError::expired(MembershipId::new()));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
    }

    #[test]
    fn api_error_maps_invalid_tier_to_400() {
        let err = MembershipApiError(MembershipError::invalid_tier("super_premium"));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn api_error_maps_invalid_promo_to_400() {
        let err = MembershipApiError(MembershipError::invalid_promo_code("BAD", "Not found"));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn api_error_maps_promo_exhausted_to_400() {
        let err = MembershipApiError(MembershipError::promo_code_exhausted("USED100X"));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn api_error_maps_payment_failed_to_402() {
        let err = MembershipApiError(MembershipError::payment_failed("Card declined"));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
    }

    #[test]
    fn api_error_maps_invalid_state_to_409() {
        let err = MembershipApiError(MembershipError::invalid_state("Pending", "cancel"));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn api_error_maps_invalid_webhook_signature_to_401() {
        let err = MembershipApiError(MembershipError::invalid_webhook_signature());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn api_error_maps_validation_failed_to_400() {
        let err = MembershipApiError(MembershipError::validation("email", "invalid format"));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn api_error_maps_infrastructure_to_500() {
        let err = MembershipApiError(MembershipError::infrastructure("Database error"));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
