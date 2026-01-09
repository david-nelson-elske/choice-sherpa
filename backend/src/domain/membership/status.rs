//! Membership status state machine.
//!
//! Defines all possible membership states and valid transitions
//! according to the subscription lifecycle.

use crate::domain::foundation::StateMachine;
use serde::{Deserialize, Serialize};

/// Membership subscription status.
///
/// Represents the current state of a user's subscription in the
/// payment lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipStatus {
    /// Initial state for paid subscriptions awaiting first payment.
    /// No access until payment completes.
    Pending,

    /// Fully paid subscription with complete access.
    Active,

    /// Payment failed but within grace period.
    /// User retains access during retry attempts.
    PastDue,

    /// User requested cancellation.
    /// Access continues until period end.
    Cancelled,

    /// Subscription ended. No access.
    /// User must resubscribe to regain access.
    Expired,
}

impl MembershipStatus {
    /// Returns true if this status grants access to the application.
    ///
    /// Access is granted for:
    /// - Active: Full paid access
    /// - PastDue: Grace period during payment retry
    /// - Cancelled: Until period end
    ///
    /// Access is denied for:
    /// - Pending: Awaiting first payment
    /// - Expired: Subscription ended
    pub fn has_access(&self) -> bool {
        matches!(
            self,
            MembershipStatus::Active | MembershipStatus::PastDue | MembershipStatus::Cancelled
        )
    }
}

impl StateMachine for MembershipStatus {
    fn can_transition_to(&self, target: &Self) -> bool {
        use MembershipStatus::*;
        matches!(
            (self, target),
            // From PENDING
            (Pending, Active)
                | (Pending, Expired)
            // From ACTIVE
                | (Active, PastDue)
                | (Active, Cancelled)
                | (Active, Expired) // Free tier only
                | (Active, Active) // Renewal
            // From PAST_DUE
                | (PastDue, Active)
                | (PastDue, Expired)
                | (PastDue, Cancelled)
            // From CANCELLED
                | (Cancelled, Active)
                | (Cancelled, Expired)
            // From EXPIRED
                | (Expired, Pending) // Resubscribe creates new
        )
    }

    fn valid_transitions(&self) -> Vec<Self> {
        use MembershipStatus::*;
        match self {
            Pending => vec![Active, Expired],
            Active => vec![PastDue, Cancelled, Expired, Active],
            PastDue => vec![Active, Expired, Cancelled],
            Cancelled => vec![Active, Expired],
            Expired => vec![Pending],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Unit Tests - State Transitions

    #[test]
    fn pending_can_transition_to_active() {
        let status = MembershipStatus::Pending;
        assert!(status.can_transition_to(&MembershipStatus::Active));

        let result = status.transition_to(MembershipStatus::Active);
        assert_eq!(result, Ok(MembershipStatus::Active));
    }

    #[test]
    fn pending_can_transition_to_expired() {
        let status = MembershipStatus::Pending;
        assert!(status.can_transition_to(&MembershipStatus::Expired));

        let result = status.transition_to(MembershipStatus::Expired);
        assert_eq!(result, Ok(MembershipStatus::Expired));
    }

    #[test]
    fn pending_cannot_transition_to_cancelled() {
        let status = MembershipStatus::Pending;
        assert!(!status.can_transition_to(&MembershipStatus::Cancelled));

        let result = status.transition_to(MembershipStatus::Cancelled);
        assert!(result.is_err());
    }

    #[test]
    fn active_can_transition_to_past_due() {
        let status = MembershipStatus::Active;
        assert!(status.can_transition_to(&MembershipStatus::PastDue));

        let result = status.transition_to(MembershipStatus::PastDue);
        assert_eq!(result, Ok(MembershipStatus::PastDue));
    }

    #[test]
    fn active_can_transition_to_cancelled() {
        let status = MembershipStatus::Active;
        assert!(status.can_transition_to(&MembershipStatus::Cancelled));

        let result = status.transition_to(MembershipStatus::Cancelled);
        assert_eq!(result, Ok(MembershipStatus::Cancelled));
    }

    #[test]
    fn active_can_renew_to_active() {
        let status = MembershipStatus::Active;
        assert!(status.can_transition_to(&MembershipStatus::Active));

        let result = status.transition_to(MembershipStatus::Active);
        assert_eq!(result, Ok(MembershipStatus::Active));
    }

    #[test]
    fn past_due_can_recover_to_active() {
        let status = MembershipStatus::PastDue;
        assert!(status.can_transition_to(&MembershipStatus::Active));

        let result = status.transition_to(MembershipStatus::Active);
        assert_eq!(result, Ok(MembershipStatus::Active));
    }

    #[test]
    fn past_due_can_expire() {
        let status = MembershipStatus::PastDue;
        assert!(status.can_transition_to(&MembershipStatus::Expired));

        let result = status.transition_to(MembershipStatus::Expired);
        assert_eq!(result, Ok(MembershipStatus::Expired));
    }

    #[test]
    fn cancelled_can_reactivate_to_active() {
        let status = MembershipStatus::Cancelled;
        assert!(status.can_transition_to(&MembershipStatus::Active));

        let result = status.transition_to(MembershipStatus::Active);
        assert_eq!(result, Ok(MembershipStatus::Active));
    }

    #[test]
    fn cancelled_can_expire() {
        let status = MembershipStatus::Cancelled;
        assert!(status.can_transition_to(&MembershipStatus::Expired));

        let result = status.transition_to(MembershipStatus::Expired);
        assert_eq!(result, Ok(MembershipStatus::Expired));
    }

    #[test]
    fn expired_cannot_directly_activate() {
        let status = MembershipStatus::Expired;
        assert!(!status.can_transition_to(&MembershipStatus::Active));

        let result = status.transition_to(MembershipStatus::Active);
        assert!(result.is_err());
    }

    // Unit Tests - has_access

    #[test]
    fn has_access_true_for_active() {
        assert!(MembershipStatus::Active.has_access());
    }

    #[test]
    fn has_access_true_for_past_due_in_grace() {
        assert!(MembershipStatus::PastDue.has_access());
    }

    #[test]
    fn has_access_true_for_cancelled_before_period_end() {
        assert!(MembershipStatus::Cancelled.has_access());
    }

    #[test]
    fn has_access_false_for_expired() {
        assert!(!MembershipStatus::Expired.has_access());
    }

    #[test]
    fn has_access_false_for_pending() {
        assert!(!MembershipStatus::Pending.has_access());
    }

    // Additional validation tests

    #[test]
    fn valid_transitions_are_consistent_with_can_transition_to() {
        for status in [
            MembershipStatus::Pending,
            MembershipStatus::Active,
            MembershipStatus::PastDue,
            MembershipStatus::Cancelled,
            MembershipStatus::Expired,
        ] {
            for valid_target in status.valid_transitions() {
                assert!(
                    status.can_transition_to(&valid_target),
                    "can_transition_to should return true for {:?} -> {:?}",
                    status,
                    valid_target
                );
            }
        }
    }

    #[test]
    fn expired_is_not_terminal_can_resubscribe() {
        // Expired can go to Pending (resubscribe)
        assert!(!MembershipStatus::Expired.is_terminal());
    }
}
