//! Membership aggregate entity.
//!
//! The Membership aggregate represents a user's subscription to the platform.
//! Each user has at most one Membership. Users without a Membership have no access.
//!
//! # Design Decisions
//!
//! - **One per user**: Unique constraint on user_id enforced at database level
//! - **Money in cents**: All monetary values stored as i64 cents (not floats)
//! - **Fail-secure**: No membership = no access (not implicit free tier)
//! - **Event-sourced transitions**: State changes emit domain events

use crate::domain::foundation::{DomainError, ErrorCode, MembershipId, Timestamp, UserId};
use serde::{Deserialize, Serialize};

use super::{MembershipStatus, MembershipTier};

/// Membership aggregate - represents a user's subscription.
///
/// # Invariants
///
/// - `id` is globally unique
/// - `user_id` is unique (one membership per user)
/// - Status transitions follow state machine rules
/// - Period dates: `current_period_start <= current_period_end`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Membership {
    /// Unique identifier for this membership.
    pub id: MembershipId,

    /// User who owns this membership.
    pub user_id: UserId,

    /// Subscription tier determining feature access.
    pub tier: MembershipTier,

    /// Current status in the subscription lifecycle.
    pub status: MembershipStatus,

    /// Start of current billing period.
    pub current_period_start: Timestamp,

    /// End of current billing period.
    pub current_period_end: Timestamp,

    /// Promo code used to create this membership (if any).
    pub promo_code: Option<String>,

    /// Stripe customer ID (for paid subscriptions).
    pub stripe_customer_id: Option<String>,

    /// Stripe subscription ID (for paid subscriptions).
    pub stripe_subscription_id: Option<String>,

    /// When the membership was created.
    pub created_at: Timestamp,

    /// When the membership was last updated.
    pub updated_at: Timestamp,

    /// When the membership was cancelled (if cancelled).
    pub cancelled_at: Option<Timestamp>,
}

impl Membership {
    /// Create a new free membership from a promo code.
    ///
    /// Free memberships are immediately Active.
    pub fn create_free(
        id: MembershipId,
        user_id: UserId,
        tier: MembershipTier,
        promo_code: String,
        period_start: Timestamp,
        period_end: Timestamp,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            id,
            user_id,
            tier,
            status: MembershipStatus::Active,
            current_period_start: period_start,
            current_period_end: period_end,
            promo_code: Some(promo_code),
            stripe_customer_id: None,
            stripe_subscription_id: None,
            created_at: now,
            updated_at: now,
            cancelled_at: None,
        }
    }

    /// Create a new pending paid membership awaiting payment.
    ///
    /// Paid memberships start in Pending status until payment is confirmed.
    pub fn create_paid(
        id: MembershipId,
        user_id: UserId,
        tier: MembershipTier,
        stripe_customer_id: String,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            id,
            user_id,
            tier,
            status: MembershipStatus::Pending,
            current_period_start: now,
            current_period_end: now, // Will be set when payment is confirmed
            promo_code: None,
            stripe_customer_id: Some(stripe_customer_id),
            stripe_subscription_id: None,
            created_at: now,
            updated_at: now,
            cancelled_at: None,
        }
    }

    /// Check if this membership grants access to the application.
    ///
    /// Returns true if status allows access AND current period hasn't ended.
    pub fn has_access(&self) -> bool {
        if !self.status.has_access() {
            return false;
        }

        // Check if still within period for Cancelled memberships
        if self.status == MembershipStatus::Cancelled {
            return Timestamp::now() <= self.current_period_end;
        }

        true
    }

    /// Activate this membership after successful payment.
    ///
    /// # Errors
    ///
    /// Returns error if transition from current status is not allowed.
    pub fn activate(
        &mut self,
        period_start: Timestamp,
        period_end: Timestamp,
        stripe_subscription_id: Option<String>,
    ) -> Result<(), DomainError> {
        self.transition_to(MembershipStatus::Active)?;
        self.current_period_start = period_start;
        self.current_period_end = period_end;
        if let Some(sub_id) = stripe_subscription_id {
            self.stripe_subscription_id = Some(sub_id);
        }
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Cancel this membership (effective at period end).
    ///
    /// # Errors
    ///
    /// Returns error if transition from current status is not allowed.
    pub fn cancel(&mut self) -> Result<(), DomainError> {
        self.transition_to(MembershipStatus::Cancelled)?;
        self.cancelled_at = Some(Timestamp::now());
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Mark this membership as expired.
    ///
    /// # Errors
    ///
    /// Returns error if transition from current status is not allowed.
    pub fn expire(&mut self) -> Result<(), DomainError> {
        self.transition_to(MembershipStatus::Expired)?;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Mark payment as past due (failed but in grace period).
    ///
    /// # Errors
    ///
    /// Returns error if transition from current status is not allowed.
    pub fn mark_past_due(&mut self) -> Result<(), DomainError> {
        self.transition_to(MembershipStatus::PastDue)?;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Recover from past due status after successful payment.
    ///
    /// # Errors
    ///
    /// Returns error if transition from current status is not allowed.
    pub fn recover_payment(&mut self, period_end: Timestamp) -> Result<(), DomainError> {
        self.transition_to(MembershipStatus::Active)?;
        self.current_period_end = period_end;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Renew the membership for a new billing period.
    ///
    /// # Errors
    ///
    /// Returns error if current status doesn't allow renewal.
    pub fn renew(&mut self, period_start: Timestamp, period_end: Timestamp) -> Result<(), DomainError> {
        self.transition_to(MembershipStatus::Active)?;
        self.current_period_start = period_start;
        self.current_period_end = period_end;
        self.cancelled_at = None;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Upgrade to a higher tier.
    ///
    /// # Errors
    ///
    /// Returns error if new tier is not higher than current.
    pub fn upgrade_tier(&mut self, new_tier: MembershipTier) -> Result<(), DomainError> {
        // Validate upgrade is actually higher
        let current_rank = self.tier.rank();
        let new_rank = new_tier.rank();

        if new_rank <= current_rank {
            return Err(DomainError::validation(
                "tier",
                format!(
                    "Cannot downgrade from {} to {}",
                    self.tier.display_name(),
                    new_tier.display_name()
                ),
            ));
        }

        self.tier = new_tier;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Days remaining in current period.
    ///
    /// Returns 0 if period has ended.
    pub fn days_remaining(&self) -> u32 {
        let now = Timestamp::now();
        if now >= self.current_period_end {
            return 0;
        }

        let duration = self.current_period_end.duration_since(&now);
        duration.num_days().max(0) as u32
    }

    /// Check if membership is expiring within the given number of days.
    pub fn expiring_within_days(&self, days: u32) -> bool {
        self.days_remaining() <= days && self.days_remaining() > 0
    }

    /// Transition to a new status using the state machine.
    fn transition_to(&mut self, target: MembershipStatus) -> Result<(), DomainError> {
        use crate::domain::foundation::StateMachine;

        self.status = self.status.transition_to(target).map_err(|_| {
            DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!(
                    "Cannot transition membership from {:?} to {:?}",
                    self.status, target
                ),
            )
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_membership_id() -> MembershipId {
        MembershipId::new()
    }

    fn test_user_id() -> UserId {
        UserId::new("user-123".to_string()).unwrap()
    }

    fn period_start() -> Timestamp {
        Timestamp::now()
    }

    fn period_end() -> Timestamp {
        Timestamp::now().add_days(30)
    }

    // Construction tests

    #[test]
    fn create_free_starts_active() {
        let membership = Membership::create_free(
            test_membership_id(),
            test_user_id(),
            MembershipTier::Annual,
            "WORKSHOP2026".to_string(),
            period_start(),
            period_end(),
        );

        assert_eq!(membership.status, MembershipStatus::Active);
        assert_eq!(membership.tier, MembershipTier::Annual);
        assert_eq!(membership.promo_code, Some("WORKSHOP2026".to_string()));
        assert!(membership.stripe_customer_id.is_none());
    }

    #[test]
    fn create_paid_starts_pending() {
        let membership = Membership::create_paid(
            test_membership_id(),
            test_user_id(),
            MembershipTier::Monthly,
            "cus_123".to_string(),
        );

        assert_eq!(membership.status, MembershipStatus::Pending);
        assert_eq!(membership.tier, MembershipTier::Monthly);
        assert_eq!(membership.stripe_customer_id, Some("cus_123".to_string()));
        assert!(membership.promo_code.is_none());
    }

    // Access tests

    #[test]
    fn active_membership_has_access() {
        let membership = Membership::create_free(
            test_membership_id(),
            test_user_id(),
            MembershipTier::Annual,
            "PROMO".to_string(),
            period_start(),
            period_end(),
        );

        assert!(membership.has_access());
    }

    #[test]
    fn pending_membership_no_access() {
        let membership = Membership::create_paid(
            test_membership_id(),
            test_user_id(),
            MembershipTier::Monthly,
            "cus_123".to_string(),
        );

        assert!(!membership.has_access());
    }

    // Lifecycle transition tests

    #[test]
    fn pending_can_activate() {
        let mut membership = Membership::create_paid(
            test_membership_id(),
            test_user_id(),
            MembershipTier::Monthly,
            "cus_123".to_string(),
        );

        let result = membership.activate(period_start(), period_end(), Some("sub_123".to_string()));
        assert!(result.is_ok());
        assert_eq!(membership.status, MembershipStatus::Active);
        assert_eq!(membership.stripe_subscription_id, Some("sub_123".to_string()));
    }

    #[test]
    fn active_can_cancel() {
        let mut membership = Membership::create_free(
            test_membership_id(),
            test_user_id(),
            MembershipTier::Annual,
            "PROMO".to_string(),
            period_start(),
            period_end(),
        );

        let result = membership.cancel();
        assert!(result.is_ok());
        assert_eq!(membership.status, MembershipStatus::Cancelled);
        assert!(membership.cancelled_at.is_some());
    }

    #[test]
    fn cancelled_can_expire() {
        let mut membership = Membership::create_free(
            test_membership_id(),
            test_user_id(),
            MembershipTier::Annual,
            "PROMO".to_string(),
            period_start(),
            period_end(),
        );

        membership.cancel().unwrap();
        let result = membership.expire();
        assert!(result.is_ok());
        assert_eq!(membership.status, MembershipStatus::Expired);
    }

    #[test]
    fn active_can_go_past_due() {
        let mut membership = Membership::create_free(
            test_membership_id(),
            test_user_id(),
            MembershipTier::Monthly,
            "PROMO".to_string(),
            period_start(),
            period_end(),
        );

        let result = membership.mark_past_due();
        assert!(result.is_ok());
        assert_eq!(membership.status, MembershipStatus::PastDue);
    }

    #[test]
    fn past_due_can_recover() {
        let mut membership = Membership::create_free(
            test_membership_id(),
            test_user_id(),
            MembershipTier::Monthly,
            "PROMO".to_string(),
            period_start(),
            period_end(),
        );

        membership.mark_past_due().unwrap();
        let new_end = Timestamp::now().add_days(30);
        let result = membership.recover_payment(new_end);
        assert!(result.is_ok());
        assert_eq!(membership.status, MembershipStatus::Active);
    }

    // Upgrade tests

    #[test]
    fn can_upgrade_free_to_monthly() {
        let mut membership = Membership::create_free(
            test_membership_id(),
            test_user_id(),
            MembershipTier::Free,
            "PROMO".to_string(),
            period_start(),
            period_end(),
        );

        let result = membership.upgrade_tier(MembershipTier::Monthly);
        assert!(result.is_ok());
        assert_eq!(membership.tier, MembershipTier::Monthly);
    }

    #[test]
    fn can_upgrade_monthly_to_annual() {
        let mut membership = Membership::create_free(
            test_membership_id(),
            test_user_id(),
            MembershipTier::Monthly,
            "PROMO".to_string(),
            period_start(),
            period_end(),
        );

        let result = membership.upgrade_tier(MembershipTier::Annual);
        assert!(result.is_ok());
        assert_eq!(membership.tier, MembershipTier::Annual);
    }

    #[test]
    fn cannot_downgrade_monthly_to_free() {
        let mut membership = Membership::create_free(
            test_membership_id(),
            test_user_id(),
            MembershipTier::Monthly,
            "PROMO".to_string(),
            period_start(),
            period_end(),
        );

        let result = membership.upgrade_tier(MembershipTier::Free);
        assert!(result.is_err());
        assert_eq!(membership.tier, MembershipTier::Monthly);
    }

    #[test]
    fn cannot_upgrade_to_same_tier() {
        let mut membership = Membership::create_free(
            test_membership_id(),
            test_user_id(),
            MembershipTier::Monthly,
            "PROMO".to_string(),
            period_start(),
            period_end(),
        );

        let result = membership.upgrade_tier(MembershipTier::Monthly);
        assert!(result.is_err());
    }

    // Renewal tests

    #[test]
    fn active_can_renew() {
        let mut membership = Membership::create_free(
            test_membership_id(),
            test_user_id(),
            MembershipTier::Monthly,
            "PROMO".to_string(),
            period_start(),
            period_end(),
        );

        let new_start = Timestamp::now();
        let new_end = new_start.add_days(30);
        let result = membership.renew(new_start, new_end);
        assert!(result.is_ok());
        assert_eq!(membership.status, MembershipStatus::Active);
    }

    #[test]
    fn renewal_clears_cancelled_at() {
        let mut membership = Membership::create_free(
            test_membership_id(),
            test_user_id(),
            MembershipTier::Monthly,
            "PROMO".to_string(),
            period_start(),
            period_end(),
        );

        membership.cancel().unwrap();
        assert!(membership.cancelled_at.is_some());

        // Reactivate through renew
        let new_start = Timestamp::now();
        let new_end = new_start.add_days(30);
        membership.renew(new_start, new_end).unwrap();
        assert!(membership.cancelled_at.is_none());
    }
}
