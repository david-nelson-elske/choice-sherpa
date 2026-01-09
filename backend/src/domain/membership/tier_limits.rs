//! Tier-based feature limits configuration.
//!
//! Defines what features and limits are available for each membership tier.

use super::MembershipTier;
use serde::{Deserialize, Serialize};

/// Feature limits for a membership tier.
///
/// Defines the boundaries of what a user can do based on their subscription.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TierLimits {
    /// The tier these limits apply to.
    pub tier: MembershipTier,
    /// Maximum active sessions. None = unlimited.
    pub max_sessions: Option<u32>,
    /// Maximum cycles per session. None = unlimited.
    pub max_cycles_per_session: Option<u32>,
    /// Whether PDF/CSV export is enabled.
    pub export_enabled: bool,
    /// Whether API access is enabled.
    pub api_access: bool,
}

impl TierLimits {
    /// Get the limits for a specific tier.
    ///
    /// # Tier Configuration
    ///
    /// | Tier | Sessions | Cycles/Session | Export | API |
    /// |------|----------|----------------|--------|-----|
    /// | Free | 3 | 5 | No | No |
    /// | Monthly | 10 | 20 | Yes | No |
    /// | Annual | Unlimited | Unlimited | Yes | Yes |
    pub fn for_tier(tier: MembershipTier) -> Self {
        match tier {
            MembershipTier::Free => Self {
                tier,
                max_sessions: Some(3),
                max_cycles_per_session: Some(5),
                export_enabled: false,
                api_access: false,
            },
            MembershipTier::Monthly => Self {
                tier,
                max_sessions: Some(10),
                max_cycles_per_session: Some(20),
                export_enabled: true,
                api_access: false,
            },
            MembershipTier::Annual => Self {
                tier,
                max_sessions: None, // Unlimited
                max_cycles_per_session: None,
                export_enabled: true,
                api_access: true,
            },
        }
    }

    /// Check if the session limit has been reached.
    ///
    /// Returns false if unlimited or under limit.
    pub fn session_limit_reached(&self, current_sessions: u32) -> bool {
        self.max_sessions
            .map(|max| current_sessions >= max)
            .unwrap_or(false)
    }

    /// Check if the cycle limit has been reached for a session.
    ///
    /// Returns false if unlimited or under limit.
    pub fn cycle_limit_reached(&self, current_cycles: u32) -> bool {
        self.max_cycles_per_session
            .map(|max| current_cycles >= max)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tier Configuration Tests

    #[test]
    fn free_tier_has_3_sessions() {
        let limits = TierLimits::for_tier(MembershipTier::Free);
        assert_eq!(limits.max_sessions, Some(3));
    }

    #[test]
    fn free_tier_has_5_cycles_per_session() {
        let limits = TierLimits::for_tier(MembershipTier::Free);
        assert_eq!(limits.max_cycles_per_session, Some(5));
    }

    #[test]
    fn free_tier_has_no_export() {
        let limits = TierLimits::for_tier(MembershipTier::Free);
        assert!(!limits.export_enabled);
    }

    #[test]
    fn free_tier_has_no_api_access() {
        let limits = TierLimits::for_tier(MembershipTier::Free);
        assert!(!limits.api_access);
    }

    #[test]
    fn monthly_tier_has_10_sessions() {
        let limits = TierLimits::for_tier(MembershipTier::Monthly);
        assert_eq!(limits.max_sessions, Some(10));
    }

    #[test]
    fn monthly_tier_has_20_cycles_per_session() {
        let limits = TierLimits::for_tier(MembershipTier::Monthly);
        assert_eq!(limits.max_cycles_per_session, Some(20));
    }

    #[test]
    fn monthly_tier_has_export() {
        let limits = TierLimits::for_tier(MembershipTier::Monthly);
        assert!(limits.export_enabled);
    }

    #[test]
    fn monthly_tier_has_no_api_access() {
        let limits = TierLimits::for_tier(MembershipTier::Monthly);
        assert!(!limits.api_access);
    }

    #[test]
    fn annual_tier_has_unlimited_sessions() {
        let limits = TierLimits::for_tier(MembershipTier::Annual);
        assert_eq!(limits.max_sessions, None);
    }

    #[test]
    fn annual_tier_has_unlimited_cycles() {
        let limits = TierLimits::for_tier(MembershipTier::Annual);
        assert_eq!(limits.max_cycles_per_session, None);
    }

    #[test]
    fn annual_tier_has_export() {
        let limits = TierLimits::for_tier(MembershipTier::Annual);
        assert!(limits.export_enabled);
    }

    #[test]
    fn annual_tier_has_api_access() {
        let limits = TierLimits::for_tier(MembershipTier::Annual);
        assert!(limits.api_access);
    }

    // Limit Check Tests

    #[test]
    fn session_limit_reached_when_at_max() {
        let limits = TierLimits::for_tier(MembershipTier::Free);
        assert!(limits.session_limit_reached(3));
    }

    #[test]
    fn session_limit_reached_when_over_max() {
        let limits = TierLimits::for_tier(MembershipTier::Free);
        assert!(limits.session_limit_reached(5));
    }

    #[test]
    fn session_limit_not_reached_when_under() {
        let limits = TierLimits::for_tier(MembershipTier::Free);
        assert!(!limits.session_limit_reached(2));
    }

    #[test]
    fn session_limit_never_reached_for_unlimited() {
        let limits = TierLimits::for_tier(MembershipTier::Annual);
        assert!(!limits.session_limit_reached(1000));
    }

    #[test]
    fn cycle_limit_reached_when_at_max() {
        let limits = TierLimits::for_tier(MembershipTier::Free);
        assert!(limits.cycle_limit_reached(5));
    }

    #[test]
    fn cycle_limit_not_reached_when_under() {
        let limits = TierLimits::for_tier(MembershipTier::Monthly);
        assert!(!limits.cycle_limit_reached(15));
    }

    #[test]
    fn cycle_limit_never_reached_for_unlimited() {
        let limits = TierLimits::for_tier(MembershipTier::Annual);
        assert!(!limits.cycle_limit_reached(1000));
    }
}
