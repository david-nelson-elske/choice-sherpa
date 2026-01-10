//! Stub implementation of AccessChecker for development and testing.
//!
//! This adapter always grants access and returns unlimited tier limits.
//! Replace with a real implementation (e.g., PostgresAccessChecker) for production.
//!
//! # Usage
//!
//! ```ignore
//! use choice_sherpa::adapters::membership::StubAccessChecker;
//!
//! let checker = StubAccessChecker::new();
//! // Or with a specific tier:
//! let checker = StubAccessChecker::with_tier(MembershipTier::Monthly);
//! ```

use crate::domain::foundation::{DomainError, SessionId, UserId};
use crate::domain::membership::{MembershipTier, TierLimits};
use crate::ports::{AccessChecker, AccessDeniedReason, AccessResult, UsageStats};
use async_trait::async_trait;

/// Stub AccessChecker that always grants access.
///
/// For development and testing purposes only.
/// Always returns `AccessResult::Allowed` and configurable tier limits.
#[derive(Debug, Clone)]
pub struct StubAccessChecker {
    /// The tier to simulate for all users.
    tier: MembershipTier,
    /// Whether to simulate denied access for testing.
    deny_access: bool,
    /// Simulated usage stats.
    usage: UsageStats,
}

impl Default for StubAccessChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl StubAccessChecker {
    /// Create a new stub that always allows access with Annual tier.
    pub fn new() -> Self {
        Self {
            tier: MembershipTier::Annual,
            deny_access: false,
            usage: UsageStats::new(),
        }
    }

    /// Create a stub with a specific tier.
    pub fn with_tier(tier: MembershipTier) -> Self {
        Self {
            tier,
            deny_access: false,
            usage: UsageStats::new(),
        }
    }

    /// Create a stub that denies all access (for testing denial flows).
    pub fn denying() -> Self {
        Self {
            tier: MembershipTier::Free,
            deny_access: true,
            usage: UsageStats::new(),
        }
    }

    /// Set simulated usage statistics.
    pub fn with_usage(mut self, usage: UsageStats) -> Self {
        self.usage = usage;
        self
    }

    /// Set the tier for this stub.
    pub fn set_tier(&mut self, tier: MembershipTier) {
        self.tier = tier;
    }

    /// Set whether to deny access.
    pub fn set_deny_access(&mut self, deny: bool) {
        self.deny_access = deny;
    }
}

#[async_trait]
impl AccessChecker for StubAccessChecker {
    async fn can_create_session(&self, _user_id: &UserId) -> Result<AccessResult, DomainError> {
        if self.deny_access {
            return Ok(AccessResult::Denied(AccessDeniedReason::NoMembership));
        }

        let limits = TierLimits::for_tier(self.tier);
        if !limits.can_create_session(self.usage.active_sessions) {
            return Ok(AccessResult::Denied(AccessDeniedReason::SessionLimitReached {
                current: self.usage.active_sessions,
                max: limits.max_active_sessions.unwrap_or(0),
            }));
        }

        Ok(AccessResult::Allowed)
    }

    async fn can_create_cycle(
        &self,
        _user_id: &UserId,
        _session_id: &SessionId,
    ) -> Result<AccessResult, DomainError> {
        if self.deny_access {
            return Ok(AccessResult::Denied(AccessDeniedReason::NoMembership));
        }

        // For stub, always allow cycles (real impl would check per-session count)
        Ok(AccessResult::Allowed)
    }

    async fn can_export(&self, _user_id: &UserId) -> Result<AccessResult, DomainError> {
        if self.deny_access {
            return Ok(AccessResult::Denied(AccessDeniedReason::NoMembership));
        }

        let limits = TierLimits::for_tier(self.tier);
        if !limits.can_export_pdf() {
            return Ok(AccessResult::Denied(AccessDeniedReason::FeatureNotIncluded {
                feature: "Export".to_string(),
                required_tier: MembershipTier::Monthly,
            }));
        }

        Ok(AccessResult::Allowed)
    }

    async fn get_tier_limits(&self, _user_id: &UserId) -> Result<TierLimits, DomainError> {
        Ok(TierLimits::for_tier(self.tier))
    }

    async fn get_usage(&self, _user_id: &UserId) -> Result<UsageStats, DomainError> {
        Ok(self.usage.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_user_id() -> UserId {
        UserId::new("test-user-123".to_string()).unwrap()
    }

    fn test_session_id() -> SessionId {
        SessionId::new()
    }

    // Construction tests

    #[test]
    fn default_uses_annual_tier() {
        let checker = StubAccessChecker::new();
        assert_eq!(checker.tier, MembershipTier::Annual);
    }

    #[test]
    fn with_tier_sets_tier() {
        let checker = StubAccessChecker::with_tier(MembershipTier::Free);
        assert_eq!(checker.tier, MembershipTier::Free);
    }

    #[test]
    fn denying_creates_deny_mode() {
        let checker = StubAccessChecker::denying();
        assert!(checker.deny_access);
    }

    // can_create_session tests

    #[tokio::test]
    async fn can_create_session_allowed_by_default() {
        let checker = StubAccessChecker::new();
        let result = checker.can_create_session(&test_user_id()).await.unwrap();
        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn can_create_session_denied_when_deny_mode() {
        let checker = StubAccessChecker::denying();
        let result = checker.can_create_session(&test_user_id()).await.unwrap();
        assert!(result.is_denied());
    }

    #[tokio::test]
    async fn can_create_session_denied_at_limit() {
        let checker = StubAccessChecker::with_tier(MembershipTier::Free)
            .with_usage(UsageStats {
                active_sessions: 3,
                total_cycles: 0,
                exports_this_month: 0,
            });

        let result = checker.can_create_session(&test_user_id()).await.unwrap();
        assert!(matches!(
            result,
            AccessResult::Denied(AccessDeniedReason::SessionLimitReached { current: 3, max: 3 })
        ));
    }

    #[tokio::test]
    async fn can_create_session_allowed_under_limit() {
        let checker = StubAccessChecker::with_tier(MembershipTier::Free)
            .with_usage(UsageStats {
                active_sessions: 2,
                total_cycles: 0,
                exports_this_month: 0,
            });

        let result = checker.can_create_session(&test_user_id()).await.unwrap();
        assert!(result.is_allowed());
    }

    // can_create_cycle tests

    #[tokio::test]
    async fn can_create_cycle_allowed_by_default() {
        let checker = StubAccessChecker::new();
        let result = checker
            .can_create_cycle(&test_user_id(), &test_session_id())
            .await
            .unwrap();
        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn can_create_cycle_denied_when_deny_mode() {
        let checker = StubAccessChecker::denying();
        let result = checker
            .can_create_cycle(&test_user_id(), &test_session_id())
            .await
            .unwrap();
        assert!(result.is_denied());
    }

    // can_export tests

    #[tokio::test]
    async fn can_export_allowed_for_annual() {
        let checker = StubAccessChecker::with_tier(MembershipTier::Annual);
        let result = checker.can_export(&test_user_id()).await.unwrap();
        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn can_export_allowed_for_monthly() {
        let checker = StubAccessChecker::with_tier(MembershipTier::Monthly);
        let result = checker.can_export(&test_user_id()).await.unwrap();
        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn can_export_denied_for_free() {
        let checker = StubAccessChecker::with_tier(MembershipTier::Free);
        let result = checker.can_export(&test_user_id()).await.unwrap();
        assert!(matches!(
            result,
            AccessResult::Denied(AccessDeniedReason::FeatureNotIncluded { .. })
        ));
    }

    // get_tier_limits tests

    #[tokio::test]
    async fn get_tier_limits_returns_tier_limits() {
        let checker = StubAccessChecker::with_tier(MembershipTier::Monthly);
        let limits = checker.get_tier_limits(&test_user_id()).await.unwrap();
        assert_eq!(limits.tier, MembershipTier::Monthly);
        assert_eq!(limits.max_active_sessions, Some(10));
    }

    // get_usage tests

    #[tokio::test]
    async fn get_usage_returns_configured_usage() {
        let expected_usage = UsageStats {
            active_sessions: 5,
            total_cycles: 15,
            exports_this_month: 2,
        };
        let checker = StubAccessChecker::new().with_usage(expected_usage.clone());

        let usage = checker.get_usage(&test_user_id()).await.unwrap();
        assert_eq!(usage, expected_usage);
    }

    // Mutability tests

    #[test]
    fn set_tier_changes_tier() {
        let mut checker = StubAccessChecker::new();
        checker.set_tier(MembershipTier::Free);
        assert_eq!(checker.tier, MembershipTier::Free);
    }

    #[test]
    fn set_deny_access_changes_mode() {
        let mut checker = StubAccessChecker::new();
        assert!(!checker.deny_access);
        checker.set_deny_access(true);
        assert!(checker.deny_access);
    }
}
