//! Membership reader port (read side / CQRS queries).
//!
//! Defines the contract for membership queries and read operations.
//! Optimized for UI display and quick access checks.
//!
//! # Design
//!
//! - **Read-optimized**: Can use caching, denormalized views
//! - **Separated from write**: CQRS pattern for scalability
//! - **Quick access checks**: Cached-friendly for high-frequency access checks
//!
//! # Example
//!
//! ```ignore
//! async fn display_membership_badge(
//!     reader: &dyn MembershipReader,
//!     user_id: &UserId,
//! ) -> Option<MembershipBadge> {
//!     let view = reader.get_by_user(user_id).await.ok()??;
//!     Some(MembershipBadge {
//!         tier: view.tier,
//!         days_remaining: view.days_remaining,
//!         has_access: view.has_access,
//!     })
//! }
//! ```

use crate::domain::foundation::{DomainError, MembershipId, Timestamp, UserId};
use crate::domain::membership::{MembershipStatus, MembershipTier};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Reader port for membership queries.
///
/// Provides read-optimized views of membership data.
/// Implementations may use caching for frequently-accessed data.
#[async_trait]
pub trait MembershipReader: Send + Sync {
    /// Get detailed membership view for a user.
    ///
    /// Returns `None` if user has no membership.
    async fn get_by_user(&self, user_id: &UserId) -> Result<Option<MembershipView>, DomainError>;

    /// Quick access check for a user.
    ///
    /// Returns `true` if user has active membership with access.
    /// This is the most frequently called method and should be highly optimized.
    async fn check_access(&self, user_id: &UserId) -> Result<bool, DomainError>;

    /// Get user's current tier.
    ///
    /// Returns `None` if user has no membership.
    async fn get_tier(&self, user_id: &UserId) -> Result<Option<MembershipTier>, DomainError>;

    /// List memberships expiring within the specified days.
    ///
    /// Used for renewal reminders and admin dashboards.
    async fn list_expiring(&self, days: u32) -> Result<Vec<MembershipSummary>, DomainError>;

    /// Get membership statistics for admin dashboard.
    async fn get_statistics(&self) -> Result<MembershipStatistics, DomainError>;
}

/// Detailed view of a membership for UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipView {
    /// Membership ID.
    pub id: MembershipId,

    /// User who owns this membership.
    pub user_id: UserId,

    /// Subscription tier.
    pub tier: MembershipTier,

    /// Current status.
    pub status: MembershipStatus,

    /// Whether user currently has access.
    pub has_access: bool,

    /// Days remaining in current period.
    pub days_remaining: u32,

    /// End of current billing period.
    pub period_end: Timestamp,

    /// Promo code used (if any).
    pub promo_code: Option<String>,

    /// When the membership was created.
    pub created_at: Timestamp,
}

/// Summary view of a membership for lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipSummary {
    /// Membership ID.
    pub id: MembershipId,

    /// User who owns this membership.
    pub user_id: UserId,

    /// Subscription tier.
    pub tier: MembershipTier,

    /// Current status.
    pub status: MembershipStatus,

    /// End of current billing period.
    pub period_end: Timestamp,
}

/// Statistics about memberships for admin dashboard.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MembershipStatistics {
    /// Total number of memberships.
    pub total_count: u64,

    /// Number of active memberships (with access).
    pub active_count: u64,

    /// Count by tier.
    pub by_tier: TierCounts,

    /// Count by status.
    pub by_status: StatusCounts,

    /// Monthly recurring revenue in cents.
    /// Calculated as: (monthly_count * monthly_price) + (annual_count * annual_price / 12)
    pub monthly_recurring_revenue_cents: i64,
}

/// Count of memberships by tier.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TierCounts {
    /// Free tier memberships.
    pub free: u64,

    /// Monthly tier memberships.
    pub monthly: u64,

    /// Annual tier memberships.
    pub annual: u64,
}

/// Count of memberships by status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusCounts {
    /// Pending memberships.
    pub pending: u64,

    /// Active memberships.
    pub active: u64,

    /// Past due memberships.
    pub past_due: u64,

    /// Cancelled memberships.
    pub cancelled: u64,

    /// Expired memberships.
    pub expired: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trait object safety test
    #[test]
    fn membership_reader_is_object_safe() {
        fn _accepts_dyn(_reader: &dyn MembershipReader) {}
    }

    #[test]
    fn membership_statistics_default_is_zero() {
        let stats = MembershipStatistics::default();
        assert_eq!(stats.total_count, 0);
        assert_eq!(stats.active_count, 0);
        assert_eq!(stats.monthly_recurring_revenue_cents, 0);
    }

    #[test]
    fn tier_counts_default_is_zero() {
        let counts = TierCounts::default();
        assert_eq!(counts.free, 0);
        assert_eq!(counts.monthly, 0);
        assert_eq!(counts.annual, 0);
    }

    #[test]
    fn status_counts_default_is_zero() {
        let counts = StatusCounts::default();
        assert_eq!(counts.pending, 0);
        assert_eq!(counts.active, 0);
        assert_eq!(counts.past_due, 0);
        assert_eq!(counts.cancelled, 0);
        assert_eq!(counts.expired, 0);
    }
}
