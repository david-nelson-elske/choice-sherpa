//! Membership-specific error types.
//!
//! Errors related to membership operations, payment processing, and access control.
//!
//! # HTTP Status Mapping
//!
//! | Error | HTTP Status |
//! |-------|-------------|
//! | NotFound | 404 |
//! | AlreadyExists | 409 |
//! | Expired | 402 |
//! | InvalidTier | 400 |
//! | InvalidPromoCode | 400 |
//! | PromoCodeExhausted | 400 |
//! | PaymentFailed | 402 |
//! | InvalidWebhookSignature | 401 |
//! | ValidationFailed | 400 |
//! | Infrastructure | 500 |

use crate::domain::foundation::{DomainError, ErrorCode, MembershipId, UserId};

/// Membership-specific errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MembershipError {
    /// Membership was not found.
    NotFound(MembershipId),

    /// No membership exists for this user.
    NotFoundForUser(UserId),

    /// User already has an active membership.
    AlreadyExists(UserId),

    /// Membership has expired.
    Expired(MembershipId),

    /// Invalid membership tier specified.
    InvalidTier(String),

    /// Invalid promo code.
    InvalidPromoCode {
        code: String,
        reason: String,
    },

    /// Promo code has reached its maximum usage count.
    PromoCodeExhausted(String),

    /// Payment processing failed.
    PaymentFailed {
        reason: String,
    },

    /// Invalid state for the requested operation.
    InvalidState {
        current: String,
        attempted: String,
    },

    /// Webhook signature verification failed.
    InvalidWebhookSignature,

    /// Validation failed.
    ValidationFailed {
        field: String,
        message: String,
    },

    /// Infrastructure error.
    Infrastructure(String),
}

impl MembershipError {
    // Constructor functions for cleaner error creation

    pub fn not_found(id: MembershipId) -> Self {
        MembershipError::NotFound(id)
    }

    pub fn not_found_for_user(user_id: UserId) -> Self {
        MembershipError::NotFoundForUser(user_id)
    }

    pub fn already_exists(user_id: UserId) -> Self {
        MembershipError::AlreadyExists(user_id)
    }

    pub fn expired(id: MembershipId) -> Self {
        MembershipError::Expired(id)
    }

    pub fn invalid_tier(tier: impl Into<String>) -> Self {
        MembershipError::InvalidTier(tier.into())
    }

    pub fn invalid_promo_code(code: impl Into<String>, reason: impl Into<String>) -> Self {
        MembershipError::InvalidPromoCode {
            code: code.into(),
            reason: reason.into(),
        }
    }

    pub fn promo_code_exhausted(code: impl Into<String>) -> Self {
        MembershipError::PromoCodeExhausted(code.into())
    }

    pub fn payment_failed(reason: impl Into<String>) -> Self {
        MembershipError::PaymentFailed {
            reason: reason.into(),
        }
    }

    pub fn invalid_state(current: impl Into<String>, attempted: impl Into<String>) -> Self {
        MembershipError::InvalidState {
            current: current.into(),
            attempted: attempted.into(),
        }
    }

    pub fn invalid_webhook_signature() -> Self {
        MembershipError::InvalidWebhookSignature
    }

    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        MembershipError::ValidationFailed {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn infrastructure(message: impl Into<String>) -> Self {
        MembershipError::Infrastructure(message.into())
    }

    /// Returns the error code for this error.
    pub fn code(&self) -> ErrorCode {
        match self {
            MembershipError::NotFound(_) | MembershipError::NotFoundForUser(_) => {
                ErrorCode::MembershipNotFound
            }
            MembershipError::AlreadyExists(_) => ErrorCode::MembershipExists,
            MembershipError::Expired(_) => ErrorCode::MembershipExpired,
            MembershipError::InvalidTier(_) => ErrorCode::InvalidTier,
            MembershipError::InvalidPromoCode { .. } => ErrorCode::InvalidPromoCode,
            MembershipError::PromoCodeExhausted(_) => ErrorCode::PromoCodeExhausted,
            MembershipError::PaymentFailed { .. } => ErrorCode::PaymentFailed,
            MembershipError::InvalidState { .. } => ErrorCode::InvalidStateTransition,
            MembershipError::InvalidWebhookSignature => ErrorCode::InvalidWebhookSignature,
            MembershipError::ValidationFailed { .. } => ErrorCode::ValidationFailed,
            MembershipError::Infrastructure(_) => ErrorCode::DatabaseError,
        }
    }

    /// Returns a user-friendly error message.
    pub fn message(&self) -> String {
        match self {
            MembershipError::NotFound(id) => format!("Membership not found: {}", id),
            MembershipError::NotFoundForUser(user_id) => {
                format!("No membership found for user: {}", user_id)
            }
            MembershipError::AlreadyExists(user_id) => {
                format!("User {} already has an active membership", user_id)
            }
            MembershipError::Expired(id) => format!("Membership {} has expired", id),
            MembershipError::InvalidTier(tier) => format!("Invalid membership tier: {}", tier),
            MembershipError::InvalidPromoCode { code, reason } => {
                format!("Promo code '{}' is invalid: {}", code, reason)
            }
            MembershipError::PromoCodeExhausted(code) => {
                format!("Promo code '{}' has been fully redeemed", code)
            }
            MembershipError::PaymentFailed { reason } => format!("Payment failed: {}", reason),
            MembershipError::InvalidState { current, attempted } => {
                format!(
                    "Cannot {} membership in {} state",
                    attempted, current
                )
            }
            MembershipError::InvalidWebhookSignature => {
                "Invalid webhook signature".to_string()
            }
            MembershipError::ValidationFailed { field, message } => {
                format!("Validation failed for '{}': {}", field, message)
            }
            MembershipError::Infrastructure(msg) => format!("Error: {}", msg),
        }
    }

    /// Returns true if this error should trigger a retry.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            MembershipError::Infrastructure(_) | MembershipError::PaymentFailed { .. }
        )
    }
}

impl std::fmt::Display for MembershipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for MembershipError {}

impl From<DomainError> for MembershipError {
    fn from(err: DomainError) -> Self {
        match err.code {
            ErrorCode::MembershipNotFound => {
                MembershipError::Infrastructure(err.to_string())
            }
            ErrorCode::MembershipExists => {
                MembershipError::Infrastructure(err.to_string())
            }
            ErrorCode::MembershipExpired => {
                MembershipError::Infrastructure(err.to_string())
            }
            ErrorCode::InvalidTier => MembershipError::InvalidTier(err.to_string()),
            ErrorCode::InvalidPromoCode => MembershipError::InvalidPromoCode {
                code: "unknown".to_string(),
                reason: err.to_string(),
            },
            ErrorCode::PromoCodeExhausted => {
                MembershipError::PromoCodeExhausted("unknown".to_string())
            }
            ErrorCode::PaymentFailed => MembershipError::PaymentFailed {
                reason: err.to_string(),
            },
            ErrorCode::InvalidStateTransition => MembershipError::InvalidState {
                current: "unknown".to_string(),
                attempted: err.to_string(),
            },
            ErrorCode::ValidationFailed => MembershipError::ValidationFailed {
                field: "unknown".to_string(),
                message: err.to_string(),
            },
            _ => MembershipError::Infrastructure(err.to_string()),
        }
    }
}

impl From<MembershipError> for DomainError {
    fn from(err: MembershipError) -> Self {
        DomainError::new(err.code(), err.message())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_membership_id() -> MembershipId {
        MembershipId::new()
    }

    fn test_user_id() -> UserId {
        UserId::new("user-test-123").unwrap()
    }

    // ============================================================
    // Constructor Tests
    // ============================================================

    #[test]
    fn not_found_creates_correctly() {
        let id = test_membership_id();
        let err = MembershipError::not_found(id.clone());
        assert!(matches!(err, MembershipError::NotFound(ref i) if *i == id));
        assert_eq!(err.code(), ErrorCode::MembershipNotFound);
    }

    #[test]
    fn not_found_for_user_creates_correctly() {
        let user_id = test_user_id();
        let err = MembershipError::not_found_for_user(user_id.clone());
        assert!(matches!(err, MembershipError::NotFoundForUser(ref u) if *u == user_id));
        assert_eq!(err.code(), ErrorCode::MembershipNotFound);
    }

    #[test]
    fn already_exists_creates_correctly() {
        let user_id = test_user_id();
        let err = MembershipError::already_exists(user_id.clone());
        assert!(matches!(err, MembershipError::AlreadyExists(ref u) if *u == user_id));
        assert_eq!(err.code(), ErrorCode::MembershipExists);
    }

    #[test]
    fn expired_creates_correctly() {
        let id = test_membership_id();
        let err = MembershipError::expired(id.clone());
        assert!(matches!(err, MembershipError::Expired(ref i) if *i == id));
        assert_eq!(err.code(), ErrorCode::MembershipExpired);
    }

    #[test]
    fn invalid_tier_creates_correctly() {
        let err = MembershipError::invalid_tier("super_premium");
        assert!(matches!(err, MembershipError::InvalidTier(ref t) if t == "super_premium"));
        assert_eq!(err.code(), ErrorCode::InvalidTier);
    }

    #[test]
    fn invalid_promo_code_creates_correctly() {
        let err = MembershipError::invalid_promo_code("BADCODE", "expired");
        assert!(matches!(
            err,
            MembershipError::InvalidPromoCode { ref code, ref reason }
            if code == "BADCODE" && reason == "expired"
        ));
        assert_eq!(err.code(), ErrorCode::InvalidPromoCode);
    }

    #[test]
    fn promo_code_exhausted_creates_correctly() {
        let err = MembershipError::promo_code_exhausted("USED100X");
        assert!(matches!(err, MembershipError::PromoCodeExhausted(ref c) if c == "USED100X"));
        assert_eq!(err.code(), ErrorCode::PromoCodeExhausted);
    }

    #[test]
    fn payment_failed_creates_correctly() {
        let err = MembershipError::payment_failed("card declined");
        assert!(matches!(
            err,
            MembershipError::PaymentFailed { ref reason } if reason == "card declined"
        ));
        assert_eq!(err.code(), ErrorCode::PaymentFailed);
    }

    #[test]
    fn invalid_state_creates_correctly() {
        let err = MembershipError::invalid_state("Pending", "cancel");
        assert!(matches!(
            err,
            MembershipError::InvalidState { ref current, ref attempted }
            if current == "Pending" && attempted == "cancel"
        ));
        assert_eq!(err.code(), ErrorCode::InvalidStateTransition);
    }

    #[test]
    fn invalid_webhook_signature_creates_correctly() {
        let err = MembershipError::invalid_webhook_signature();
        assert!(matches!(err, MembershipError::InvalidWebhookSignature));
        assert_eq!(err.code(), ErrorCode::InvalidWebhookSignature);
    }

    #[test]
    fn validation_creates_correctly() {
        let err = MembershipError::validation("email", "invalid format");
        assert!(matches!(
            err,
            MembershipError::ValidationFailed { ref field, ref message }
            if field == "email" && message == "invalid format"
        ));
        assert_eq!(err.code(), ErrorCode::ValidationFailed);
    }

    #[test]
    fn infrastructure_creates_correctly() {
        let err = MembershipError::infrastructure("database connection lost");
        assert!(matches!(
            err,
            MembershipError::Infrastructure(ref m) if m == "database connection lost"
        ));
        assert_eq!(err.code(), ErrorCode::DatabaseError);
    }

    // ============================================================
    // Message Tests
    // ============================================================

    #[test]
    fn not_found_message_includes_id() {
        let id = test_membership_id();
        let err = MembershipError::not_found(id.clone());
        assert!(err.message().contains(&id.to_string()));
    }

    #[test]
    fn already_exists_message_includes_user() {
        let user_id = test_user_id();
        let err = MembershipError::already_exists(user_id.clone());
        assert!(err.message().contains(&user_id.to_string()));
    }

    #[test]
    fn invalid_promo_code_message_includes_code_and_reason() {
        let err = MembershipError::invalid_promo_code("TEST123", "not found");
        let msg = err.message();
        assert!(msg.contains("TEST123"));
        assert!(msg.contains("not found"));
    }

    // ============================================================
    // Retryable Tests
    // ============================================================

    #[test]
    fn infrastructure_errors_are_retryable() {
        let err = MembershipError::infrastructure("timeout");
        assert!(err.is_retryable());
    }

    #[test]
    fn payment_failed_is_retryable() {
        let err = MembershipError::payment_failed("timeout");
        assert!(err.is_retryable());
    }

    #[test]
    fn validation_errors_are_not_retryable() {
        let err = MembershipError::validation("email", "invalid");
        assert!(!err.is_retryable());
    }

    #[test]
    fn not_found_errors_are_not_retryable() {
        let err = MembershipError::not_found(test_membership_id());
        assert!(!err.is_retryable());
    }

    // ============================================================
    // Display Tests
    // ============================================================

    #[test]
    fn display_matches_message() {
        let err = MembershipError::invalid_tier("unknown");
        assert_eq!(format!("{}", err), err.message());
    }

    // ============================================================
    // Conversion Tests
    // ============================================================

    #[test]
    fn converts_to_domain_error() {
        let err = MembershipError::not_found(test_membership_id());
        let domain_err: DomainError = err.clone().into();
        assert_eq!(domain_err.code, err.code());
    }

    #[test]
    fn converts_from_domain_error() {
        let domain_err = DomainError::new(ErrorCode::PaymentFailed, "card expired");
        let membership_err: MembershipError = domain_err.into();
        assert_eq!(membership_err.code(), ErrorCode::PaymentFailed);
    }
}
