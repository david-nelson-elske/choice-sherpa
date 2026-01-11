//! Access control port for membership-gated operations.
//!
//! This port defines the contract for checking user access to platform features.
//! The Session module depends on this to gate session creation.
//!
//! # Design
//!
//! The AccessChecker follows a **fail-secure** design: on ANY error, access is denied.
//! Users without membership get zero access (no implicit free tier).
//!
//! # Example
//!
//! ```ignore
//! use choice_sherpa::ports::{AccessChecker, AccessResult};
//!
//! async fn create_session(
//!     access_checker: &dyn AccessChecker,
//!     user_id: &UserId,
//! ) -> Result<Session, DomainError> {
//!     match access_checker.can_create_session(user_id).await? {
//!         AccessResult::Allowed => { /* proceed */ }
//!         AccessResult::Denied(reason) => {
//!             return Err(DomainError::access_denied(reason));
//!         }
//!     }
//!     // ... create session
//! }
//! ```

use crate::domain::foundation::{DomainError, SessionId, UserId};
use crate::domain::membership::{MembershipTier, TierLimits};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Port for checking user access based on membership.
///
/// Implementors must handle caching appropriately, as access checks
/// may be called frequently. Cache should be invalidated on membership events.
#[async_trait]
pub trait AccessChecker: Send + Sync {
    /// Check if user can create a new session.
    ///
    /// Verifies:
    /// - User has active membership
    /// - Session limit not reached
    async fn can_create_session(&self, user_id: &UserId) -> Result<AccessResult, DomainError>;

    /// Check if user can create a new cycle in a session.
    ///
    /// Verifies:
    /// - User has active membership
    /// - Cycle limit for session not reached
    async fn can_create_cycle(
        &self,
        user_id: &UserId,
        session_id: &SessionId,
    ) -> Result<AccessResult, DomainError>;

    /// Check if user can export data (PDF, CSV).
    ///
    /// Export is only available for paid tiers (Monthly, Annual).
    async fn can_export(&self, user_id: &UserId) -> Result<AccessResult, DomainError>;

    /// Get user's current tier limits.
    ///
    /// Returns limits based on user's active membership tier.
    async fn get_tier_limits(&self, user_id: &UserId) -> Result<TierLimits, DomainError>;

    /// Get user's current usage statistics.
    ///
    /// Returns counts of active sessions, total cycles, etc.
    async fn get_usage(&self, user_id: &UserId) -> Result<UsageStats, DomainError>;
}

/// Result of an access check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessResult {
    /// Access is granted.
    Allowed,
    /// Access is denied with a specific reason.
    Denied(AccessDeniedReason),
}

impl AccessResult {
    /// Returns true if access is allowed.
    pub fn is_allowed(&self) -> bool {
        matches!(self, AccessResult::Allowed)
    }

    /// Returns true if access is denied.
    pub fn is_denied(&self) -> bool {
        matches!(self, AccessResult::Denied(_))
    }

    /// Converts the result to a Result type, with denied becoming an error.
    pub fn into_result(self) -> Result<(), AccessDeniedReason> {
        match self {
            AccessResult::Allowed => Ok(()),
            AccessResult::Denied(reason) => Err(reason),
        }
    }
}

/// Reason why access was denied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AccessDeniedReason {
    /// User has no membership record.
    NoMembership,

    /// User's membership has expired.
    MembershipExpired,

    /// User's payment is past due (outside grace period).
    MembershipPastDue,

    /// Maximum number of sessions reached for tier.
    SessionLimitReached {
        /// Current number of active sessions.
        current: u32,
        /// Maximum allowed for tier.
        max: u32,
    },

    /// Maximum number of cycles reached for this session.
    CycleLimitReached {
        /// Current number of cycles in session.
        current: u32,
        /// Maximum allowed for tier.
        max: u32,
    },

    /// Feature requires a higher tier.
    FeatureNotIncluded {
        /// Name of the feature requested.
        feature: String,
        /// Tier required for this feature.
        required_tier: MembershipTier,
    },
}

impl AccessDeniedReason {
    /// Get a user-facing message for the denial reason.
    pub fn user_message(&self) -> String {
        match self {
            AccessDeniedReason::NoMembership => {
                "A membership is required to access this feature.".to_string()
            }
            AccessDeniedReason::MembershipExpired => {
                "Your membership has expired. Please renew to continue.".to_string()
            }
            AccessDeniedReason::MembershipPastDue => {
                "Your payment is past due. Please update your payment method.".to_string()
            }
            AccessDeniedReason::SessionLimitReached { current, max } => {
                format!(
                    "You've reached the limit of {} sessions (currently have {}). Upgrade for more.",
                    max, current
                )
            }
            AccessDeniedReason::CycleLimitReached { current, max } => {
                format!(
                    "You've reached the limit of {} cycles per session (currently have {}). Upgrade for more.",
                    max, current
                )
            }
            AccessDeniedReason::FeatureNotIncluded {
                feature,
                required_tier,
            } => {
                format!(
                    "{} requires a {} membership or higher.",
                    feature,
                    required_tier.display_name()
                )
            }
        }
    }
}

impl std::fmt::Display for AccessDeniedReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

/// Current usage statistics for a user.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageStats {
    /// Number of active (non-archived) sessions.
    pub active_sessions: u32,
    /// Total number of cycles across all sessions.
    pub total_cycles: u32,
    /// Number of exports performed this month.
    pub exports_this_month: u32,
}

impl UsageStats {
    /// Create a new empty usage stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if user is at or over their session limit for the given tier.
    pub fn at_session_limit(&self, limits: &TierLimits) -> bool {
        !limits.can_create_session(self.active_sessions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // AccessResult tests

    #[test]
    fn allowed_is_allowed() {
        let result = AccessResult::Allowed;
        assert!(result.is_allowed());
        assert!(!result.is_denied());
    }

    #[test]
    fn denied_is_denied() {
        let result = AccessResult::Denied(AccessDeniedReason::NoMembership);
        assert!(result.is_denied());
        assert!(!result.is_allowed());
    }

    #[test]
    fn into_result_allowed_is_ok() {
        let result = AccessResult::Allowed;
        assert!(result.into_result().is_ok());
    }

    #[test]
    fn into_result_denied_is_err() {
        let result = AccessResult::Denied(AccessDeniedReason::MembershipExpired);
        let err = result.into_result().unwrap_err();
        assert_eq!(err, AccessDeniedReason::MembershipExpired);
    }

    // AccessDeniedReason tests

    #[test]
    fn no_membership_message() {
        let reason = AccessDeniedReason::NoMembership;
        assert!(reason.user_message().contains("membership is required"));
    }

    #[test]
    fn expired_message() {
        let reason = AccessDeniedReason::MembershipExpired;
        assert!(reason.user_message().contains("expired"));
    }

    #[test]
    fn past_due_message() {
        let reason = AccessDeniedReason::MembershipPastDue;
        assert!(reason.user_message().contains("past due"));
    }

    #[test]
    fn session_limit_message_shows_counts() {
        let reason = AccessDeniedReason::SessionLimitReached { current: 3, max: 3 };
        let msg = reason.user_message();
        assert!(msg.contains("3 sessions"));
        assert!(msg.contains("currently have 3"));
    }

    #[test]
    fn cycle_limit_message_shows_counts() {
        let reason = AccessDeniedReason::CycleLimitReached { current: 5, max: 5 };
        let msg = reason.user_message();
        assert!(msg.contains("5 cycles"));
    }

    #[test]
    fn feature_not_included_shows_tier() {
        let reason = AccessDeniedReason::FeatureNotIncluded {
            feature: "Export".to_string(),
            required_tier: MembershipTier::Monthly,
        };
        let msg = reason.user_message();
        assert!(msg.contains("Export"));
        assert!(msg.contains("Monthly"));
    }

    #[test]
    fn access_denied_reason_serializes_with_type_tag() {
        let reason = AccessDeniedReason::SessionLimitReached { current: 3, max: 3 };
        let json = serde_json::to_string(&reason).unwrap();
        assert!(json.contains("\"type\":\"session_limit_reached\""));
        assert!(json.contains("\"current\":3"));
        assert!(json.contains("\"max\":3"));
    }

    // UsageStats tests

    #[test]
    fn usage_stats_default_is_zero() {
        let stats = UsageStats::new();
        assert_eq!(stats.active_sessions, 0);
        assert_eq!(stats.total_cycles, 0);
        assert_eq!(stats.exports_this_month, 0);
    }

    #[test]
    fn at_session_limit_true_when_reached() {
        let stats = UsageStats {
            active_sessions: 3,
            total_cycles: 0,
            exports_this_month: 0,
        };
        let limits = TierLimits::for_tier(MembershipTier::Free);
        assert!(stats.at_session_limit(&limits));
    }

    #[test]
    fn at_session_limit_false_when_under() {
        let stats = UsageStats {
            active_sessions: 2,
            total_cycles: 0,
            exports_this_month: 0,
        };
        let limits = TierLimits::for_tier(MembershipTier::Free);
        assert!(!stats.at_session_limit(&limits));
    }

    #[test]
    fn at_session_limit_false_for_unlimited_tier() {
        let stats = UsageStats {
            active_sessions: 1000,
            total_cycles: 0,
            exports_this_month: 0,
        };
        let limits = TierLimits::for_tier(MembershipTier::Annual);
        assert!(!stats.at_session_limit(&limits));
    }

    // Trait object safety test

    #[test]
    fn access_checker_is_object_safe() {
        // This test verifies the trait can be used as a trait object
        fn _accepts_dyn(_checker: &dyn AccessChecker) {}
    }
}
