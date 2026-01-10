//! Promo code validation port.
//!
//! Defines the contract for validating promo codes against external storage.
//! Promo codes grant free membership access and must be validated for:
//! - Existence (is this a real code?)
//! - Expiry (has the campaign ended?)
//! - Usage limits (has it been fully redeemed?)
//!
//! # Example
//!
//! ```ignore
//! use choice_sherpa::ports::{PromoCodeValidator, PromoCodeValidation};
//! use choice_sherpa::domain::membership::PromoCode;
//!
//! async fn apply_promo(
//!     validator: &dyn PromoCodeValidator,
//!     code: &PromoCode,
//! ) -> Result<(), DomainError> {
//!     match validator.validate(code).await? {
//!         PromoCodeValidation::Valid { duration_days, tier } => {
//!             // Apply the promo code to create membership
//!         }
//!         PromoCodeValidation::Invalid(reason) => {
//!             return Err(DomainError::promo_code_invalid(reason));
//!         }
//!     }
//!     Ok(())
//! }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::foundation::DomainError;
use crate::domain::membership::{MembershipTier, PromoCode};

/// Port for validating promo codes against external storage.
///
/// Implementations typically query a database to check:
/// - Code exists and matches stored hash
/// - Code hasn't expired
/// - Code hasn't exceeded usage limits
/// - Code grants specific tier/duration
#[async_trait]
pub trait PromoCodeValidator: Send + Sync {
    /// Validates a promo code and returns its benefits if valid.
    ///
    /// # Returns
    ///
    /// - `Ok(Valid { ... })` - Code is valid with specified benefits
    /// - `Ok(Invalid(reason))` - Code is invalid for a specific reason
    /// - `Err(DomainError)` - Infrastructure error occurred
    async fn validate(&self, code: &PromoCode) -> Result<PromoCodeValidation, DomainError>;

    /// Records that a promo code has been redeemed.
    ///
    /// Implementations should increment usage counters. This should be called
    /// after successful membership creation, not during validation.
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Redemption recorded
    /// - `Err(DomainError)` - Infrastructure error or code exhausted during race
    async fn record_redemption(&self, code: &PromoCode) -> Result<(), DomainError>;

    /// Gets the current usage count for a promo code.
    ///
    /// Returns None if the code doesn't exist.
    async fn get_usage_count(&self, code: &PromoCode) -> Result<Option<u32>, DomainError>;
}

/// Result of validating a promo code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromoCodeValidation {
    /// Code is valid and can be redeemed.
    Valid {
        /// Number of days the free membership lasts.
        duration_days: u32,
        /// Membership tier granted by this code.
        tier: MembershipTier,
        /// Optional campaign name for tracking.
        campaign: Option<String>,
    },
    /// Code is invalid for the specified reason.
    Invalid(PromoCodeInvalidReason),
}

impl PromoCodeValidation {
    /// Creates a valid result with default 30-day Free tier.
    pub fn valid_free(duration_days: u32) -> Self {
        PromoCodeValidation::Valid {
            duration_days,
            tier: MembershipTier::Free,
            campaign: None,
        }
    }

    /// Creates a valid result with specified tier.
    pub fn valid_with_tier(duration_days: u32, tier: MembershipTier) -> Self {
        PromoCodeValidation::Valid {
            duration_days,
            tier,
            campaign: None,
        }
    }

    /// Creates a valid result with campaign tracking.
    pub fn valid_with_campaign(
        duration_days: u32,
        tier: MembershipTier,
        campaign: impl Into<String>,
    ) -> Self {
        PromoCodeValidation::Valid {
            duration_days,
            tier,
            campaign: Some(campaign.into()),
        }
    }

    /// Returns true if the code is valid.
    pub fn is_valid(&self) -> bool {
        matches!(self, PromoCodeValidation::Valid { .. })
    }

    /// Returns true if the code is invalid.
    pub fn is_invalid(&self) -> bool {
        matches!(self, PromoCodeValidation::Invalid(_))
    }

    /// Converts to a Result, with invalid becoming an error.
    pub fn into_result(self) -> Result<(u32, MembershipTier, Option<String>), PromoCodeInvalidReason> {
        match self {
            PromoCodeValidation::Valid {
                duration_days,
                tier,
                campaign,
            } => Ok((duration_days, tier, campaign)),
            PromoCodeValidation::Invalid(reason) => Err(reason),
        }
    }
}

/// Reason why a promo code is invalid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PromoCodeInvalidReason {
    /// Code does not exist in the system.
    NotFound,

    /// Code has expired (campaign ended).
    Expired {
        /// When the code expired.
        expired_at: String,
    },

    /// Code has reached its maximum redemption count.
    Exhausted {
        /// How many times the code has been used.
        used: u32,
        /// Maximum allowed uses.
        max: u32,
    },

    /// Code has been revoked/disabled by admin.
    Revoked,

    /// Code is not yet active (future campaign).
    NotYetActive {
        /// When the code becomes active.
        active_at: String,
    },
}

impl PromoCodeInvalidReason {
    /// Get a user-facing message for the invalid reason.
    pub fn user_message(&self) -> String {
        match self {
            PromoCodeInvalidReason::NotFound => {
                "This promo code was not found. Please check and try again.".to_string()
            }
            PromoCodeInvalidReason::Expired { expired_at } => {
                format!("This promo code expired on {}.", expired_at)
            }
            PromoCodeInvalidReason::Exhausted { used, max } => {
                format!(
                    "This promo code has been fully redeemed ({}/{} uses).",
                    used, max
                )
            }
            PromoCodeInvalidReason::Revoked => {
                "This promo code is no longer valid.".to_string()
            }
            PromoCodeInvalidReason::NotYetActive { active_at } => {
                format!("This promo code is not yet active. It starts on {}.", active_at)
            }
        }
    }
}

impl std::fmt::Display for PromoCodeInvalidReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ════════════════════════════════════════════════════════════════════════════
    // PromoCodeValidation Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn valid_free_creates_correct_validation() {
        let validation = PromoCodeValidation::valid_free(30);
        match validation {
            PromoCodeValidation::Valid {
                duration_days,
                tier,
                campaign,
            } => {
                assert_eq!(duration_days, 30);
                assert_eq!(tier, MembershipTier::Free);
                assert!(campaign.is_none());
            }
            _ => panic!("Expected Valid variant"),
        }
    }

    #[test]
    fn valid_with_tier_creates_correct_validation() {
        let validation = PromoCodeValidation::valid_with_tier(90, MembershipTier::Monthly);
        match validation {
            PromoCodeValidation::Valid {
                duration_days,
                tier,
                campaign,
            } => {
                assert_eq!(duration_days, 90);
                assert_eq!(tier, MembershipTier::Monthly);
                assert!(campaign.is_none());
            }
            _ => panic!("Expected Valid variant"),
        }
    }

    #[test]
    fn valid_with_campaign_creates_correct_validation() {
        let validation = PromoCodeValidation::valid_with_campaign(
            60,
            MembershipTier::Annual,
            "WORKSHOP2026",
        );
        match validation {
            PromoCodeValidation::Valid {
                duration_days,
                tier,
                campaign,
            } => {
                assert_eq!(duration_days, 60);
                assert_eq!(tier, MembershipTier::Annual);
                assert_eq!(campaign, Some("WORKSHOP2026".to_string()));
            }
            _ => panic!("Expected Valid variant"),
        }
    }

    #[test]
    fn is_valid_returns_true_for_valid() {
        let validation = PromoCodeValidation::valid_free(30);
        assert!(validation.is_valid());
        assert!(!validation.is_invalid());
    }

    #[test]
    fn is_invalid_returns_true_for_invalid() {
        let validation = PromoCodeValidation::Invalid(PromoCodeInvalidReason::NotFound);
        assert!(validation.is_invalid());
        assert!(!validation.is_valid());
    }

    #[test]
    fn into_result_valid_returns_ok() {
        let validation = PromoCodeValidation::valid_with_campaign(30, MembershipTier::Free, "TEST");
        let result = validation.into_result();
        assert!(result.is_ok());
        let (days, tier, campaign) = result.unwrap();
        assert_eq!(days, 30);
        assert_eq!(tier, MembershipTier::Free);
        assert_eq!(campaign, Some("TEST".to_string()));
    }

    #[test]
    fn into_result_invalid_returns_err() {
        let validation = PromoCodeValidation::Invalid(PromoCodeInvalidReason::Revoked);
        let result = validation.into_result();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PromoCodeInvalidReason::Revoked);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // PromoCodeInvalidReason Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn not_found_message_is_helpful() {
        let reason = PromoCodeInvalidReason::NotFound;
        let msg = reason.user_message();
        assert!(msg.contains("not found"));
        assert!(msg.contains("check"));
    }

    #[test]
    fn expired_message_shows_date() {
        let reason = PromoCodeInvalidReason::Expired {
            expired_at: "2026-01-01".to_string(),
        };
        let msg = reason.user_message();
        assert!(msg.contains("expired"));
        assert!(msg.contains("2026-01-01"));
    }

    #[test]
    fn exhausted_message_shows_counts() {
        let reason = PromoCodeInvalidReason::Exhausted { used: 100, max: 100 };
        let msg = reason.user_message();
        assert!(msg.contains("fully redeemed"));
        assert!(msg.contains("100/100"));
    }

    #[test]
    fn revoked_message_is_generic() {
        let reason = PromoCodeInvalidReason::Revoked;
        let msg = reason.user_message();
        assert!(msg.contains("no longer valid"));
    }

    #[test]
    fn not_yet_active_message_shows_date() {
        let reason = PromoCodeInvalidReason::NotYetActive {
            active_at: "2026-02-01".to_string(),
        };
        let msg = reason.user_message();
        assert!(msg.contains("not yet active"));
        assert!(msg.contains("2026-02-01"));
    }

    #[test]
    fn display_matches_user_message() {
        let reason = PromoCodeInvalidReason::NotFound;
        assert_eq!(format!("{}", reason), reason.user_message());
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Serialization Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn invalid_reason_serializes_with_type_tag() {
        let reason = PromoCodeInvalidReason::Exhausted { used: 50, max: 100 };
        let json = serde_json::to_string(&reason).unwrap();
        assert!(json.contains("\"type\":\"exhausted\""));
        assert!(json.contains("\"used\":50"));
        assert!(json.contains("\"max\":100"));
    }

    #[test]
    fn invalid_reason_deserializes_correctly() {
        let json = r#"{"type":"not_found"}"#;
        let reason: PromoCodeInvalidReason = serde_json::from_str(json).unwrap();
        assert_eq!(reason, PromoCodeInvalidReason::NotFound);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Trait Object Safety Test
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn promo_code_validator_is_object_safe() {
        // This test verifies the trait can be used as a trait object
        fn _accepts_dyn(_validator: &dyn PromoCodeValidator) {}
    }
}
