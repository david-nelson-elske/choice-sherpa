//! Membership tier definitions.
//!
//! Represents the subscription tier levels available in Choice Sherpa.

use serde::{Deserialize, Serialize};

/// Membership subscription tier.
///
/// Determines feature access, usage limits, and pricing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MembershipTier {
    /// Free tier - limited features, good for evaluation.
    /// - 3 active sessions
    /// - 5 cycles per session
    /// - No export capability
    Free,

    /// Monthly subscription tier.
    /// - 10 active sessions
    /// - 20 cycles per session
    /// - Export enabled
    Monthly,

    /// Annual subscription tier - best value.
    /// - Unlimited sessions
    /// - Unlimited cycles
    /// - Export enabled
    /// - API access
    Annual,
}

impl MembershipTier {
    /// Returns true if this tier is a paid tier.
    pub fn is_paid(&self) -> bool {
        !matches!(self, MembershipTier::Free)
    }

    /// Returns the display name for this tier.
    pub fn display_name(&self) -> &'static str {
        match self {
            MembershipTier::Free => "Free",
            MembershipTier::Monthly => "Monthly",
            MembershipTier::Annual => "Annual",
        }
    }

    /// Returns the numeric rank of this tier for comparison.
    ///
    /// Higher rank = more features. Used for upgrade validation.
    pub fn rank(&self) -> u8 {
        match self {
            MembershipTier::Free => 0,
            MembershipTier::Monthly => 1,
            MembershipTier::Annual => 2,
        }
    }
}

impl std::fmt::Display for MembershipTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_tier_is_not_paid() {
        assert!(!MembershipTier::Free.is_paid());
    }

    #[test]
    fn monthly_tier_is_paid() {
        assert!(MembershipTier::Monthly.is_paid());
    }

    #[test]
    fn annual_tier_is_paid() {
        assert!(MembershipTier::Annual.is_paid());
    }

    #[test]
    fn display_names_are_correct() {
        assert_eq!(MembershipTier::Free.display_name(), "Free");
        assert_eq!(MembershipTier::Monthly.display_name(), "Monthly");
        assert_eq!(MembershipTier::Annual.display_name(), "Annual");
    }

    #[test]
    fn tier_serializes_lowercase() {
        let tier = MembershipTier::Monthly;
        let json = serde_json::to_string(&tier).unwrap();
        assert_eq!(json, "\"monthly\"");
    }

    #[test]
    fn tier_deserializes_from_lowercase() {
        let tier: MembershipTier = serde_json::from_str("\"annual\"").unwrap();
        assert_eq!(tier, MembershipTier::Annual);
    }
}
