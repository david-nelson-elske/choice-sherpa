//! Membership domain events.
//!
//! Events emitted during membership lifecycle changes. These events are used for:
//! - Audit logging (all state transitions)
//! - Integration with other modules (access control changes)
//! - Email notifications (welcome, payment failed, etc.)
//!
//! # Event Naming Convention
//!
//! Events are named in past tense to indicate something that has already happened:
//! - `MembershipCreated` not `CreateMembership`
//! - `PaymentFailed` not `FailPayment`

use crate::domain::foundation::{DomainEvent, EventId, MembershipId, Timestamp, UserId};
use serde::{Deserialize, Serialize};

use super::MembershipTier;

/// Events that occur during membership lifecycle.
///
/// All state transitions emit events for audit logging and integration.
/// Events follow the state machine transitions defined in the subscription
/// state machine specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MembershipEvent {
    /// A new membership was created (initial state: Active for free, Pending for paid).
    ///
    /// Emitted when:
    /// - Free membership created with valid promo code
    /// - Paid membership checkout initiated
    Created {
        event_id: EventId,
        membership_id: MembershipId,
        user_id: UserId,
        tier: MembershipTier,
        is_free: bool,
        promo_code: Option<String>,
        occurred_at: Timestamp,
    },

    /// Membership was activated after successful payment.
    ///
    /// State transition: Pending → Active
    ///
    /// Trigger: `checkout.session.completed` webhook
    Activated {
        event_id: EventId,
        membership_id: MembershipId,
        user_id: UserId,
        tier: MembershipTier,
        period_start: Timestamp,
        period_end: Timestamp,
        occurred_at: Timestamp,
    },

    /// Membership was renewed for a new billing period.
    ///
    /// State transition: Active → Active (renewal)
    ///
    /// Trigger: `invoice.payment_succeeded` webhook for existing subscription
    Renewed {
        event_id: EventId,
        membership_id: MembershipId,
        user_id: UserId,
        new_period_start: Timestamp,
        new_period_end: Timestamp,
        occurred_at: Timestamp,
    },

    /// Payment failed, membership is in grace period.
    ///
    /// State transition: Active → PastDue
    ///
    /// Trigger: `invoice.payment_failed` webhook
    PaymentFailed {
        event_id: EventId,
        membership_id: MembershipId,
        user_id: UserId,
        attempt_count: u32,
        next_retry_at: Option<Timestamp>,
        occurred_at: Timestamp,
    },

    /// Payment recovered after being past due.
    ///
    /// State transition: PastDue → Active
    ///
    /// Trigger: `invoice.payment_succeeded` webhook after failed attempts
    PaymentRecovered {
        event_id: EventId,
        membership_id: MembershipId,
        user_id: UserId,
        occurred_at: Timestamp,
    },

    /// User requested cancellation (access continues until period end).
    ///
    /// State transition: Active → Cancelled, or PastDue → Cancelled
    ///
    /// Trigger: User action via cancel endpoint
    Cancelled {
        event_id: EventId,
        membership_id: MembershipId,
        user_id: UserId,
        effective_at: Timestamp,
        occurred_at: Timestamp,
    },

    /// Cancelled membership was reactivated before period end.
    ///
    /// State transition: Cancelled → Active
    ///
    /// Trigger: User action via reactivate endpoint (before period_end)
    Reactivated {
        event_id: EventId,
        membership_id: MembershipId,
        user_id: UserId,
        occurred_at: Timestamp,
    },

    /// Membership expired (no longer has access).
    ///
    /// State transition: Cancelled → Expired, PastDue → Expired, or Pending → Expired
    ///
    /// Triggers:
    /// - Period end reached for Cancelled
    /// - Grace period exceeded for PastDue
    /// - Payment timeout (72h) for Pending
    Expired {
        event_id: EventId,
        membership_id: MembershipId,
        user_id: UserId,
        reason: ExpiredReason,
        occurred_at: Timestamp,
    },

    /// Membership tier was upgraded.
    ///
    /// Trigger: User initiated upgrade (e.g., Free → Monthly, Monthly → Annual)
    TierUpgraded {
        event_id: EventId,
        membership_id: MembershipId,
        user_id: UserId,
        previous_tier: MembershipTier,
        new_tier: MembershipTier,
        occurred_at: Timestamp,
    },

    /// Access was checked (for audit logging of access control).
    ///
    /// Note: This is a high-volume event, may be sampled in production.
    AccessChecked {
        event_id: EventId,
        membership_id: Option<MembershipId>,
        user_id: UserId,
        resource: String,
        granted: bool,
        occurred_at: Timestamp,
    },
}

/// Reason why a membership expired.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpiredReason {
    /// Cancelled membership reached period end.
    CancelledPeriodEnd,

    /// PastDue membership exceeded grace period.
    GracePeriodExceeded,

    /// Pending membership timed out (72h without payment).
    PaymentTimeout,

    /// Free tier reached annual expiry.
    FreeTierExpiry,
}

impl std::fmt::Display for ExpiredReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpiredReason::CancelledPeriodEnd => write!(f, "cancelled_period_end"),
            ExpiredReason::GracePeriodExceeded => write!(f, "grace_period_exceeded"),
            ExpiredReason::PaymentTimeout => write!(f, "payment_timeout"),
            ExpiredReason::FreeTierExpiry => write!(f, "free_tier_expiry"),
        }
    }
}

impl MembershipEvent {
    /// Returns the event type string for routing and filtering.
    pub fn event_type(&self) -> &'static str {
        match self {
            MembershipEvent::Created { .. } => "membership.created.v1",
            MembershipEvent::Activated { .. } => "membership.activated.v1",
            MembershipEvent::Renewed { .. } => "membership.renewed.v1",
            MembershipEvent::PaymentFailed { .. } => "membership.payment_failed.v1",
            MembershipEvent::PaymentRecovered { .. } => "membership.payment_recovered.v1",
            MembershipEvent::Cancelled { .. } => "membership.cancelled.v1",
            MembershipEvent::Reactivated { .. } => "membership.reactivated.v1",
            MembershipEvent::Expired { .. } => "membership.expired.v1",
            MembershipEvent::TierUpgraded { .. } => "membership.tier_upgraded.v1",
            MembershipEvent::AccessChecked { .. } => "membership.access_checked.v1",
        }
    }

    /// Returns the membership ID associated with this event, if any.
    pub fn membership_id(&self) -> Option<&MembershipId> {
        match self {
            MembershipEvent::Created { membership_id, .. }
            | MembershipEvent::Activated { membership_id, .. }
            | MembershipEvent::Renewed { membership_id, .. }
            | MembershipEvent::PaymentFailed { membership_id, .. }
            | MembershipEvent::PaymentRecovered { membership_id, .. }
            | MembershipEvent::Cancelled { membership_id, .. }
            | MembershipEvent::Reactivated { membership_id, .. }
            | MembershipEvent::Expired { membership_id, .. }
            | MembershipEvent::TierUpgraded { membership_id, .. } => Some(membership_id),
            MembershipEvent::AccessChecked { membership_id, .. } => membership_id.as_ref(),
        }
    }

    /// Returns the user ID associated with this event.
    pub fn user_id(&self) -> &UserId {
        match self {
            MembershipEvent::Created { user_id, .. }
            | MembershipEvent::Activated { user_id, .. }
            | MembershipEvent::Renewed { user_id, .. }
            | MembershipEvent::PaymentFailed { user_id, .. }
            | MembershipEvent::PaymentRecovered { user_id, .. }
            | MembershipEvent::Cancelled { user_id, .. }
            | MembershipEvent::Reactivated { user_id, .. }
            | MembershipEvent::Expired { user_id, .. }
            | MembershipEvent::TierUpgraded { user_id, .. }
            | MembershipEvent::AccessChecked { user_id, .. } => user_id,
        }
    }

    /// Returns when this event occurred.
    pub fn occurred_at(&self) -> Timestamp {
        match self {
            MembershipEvent::Created { occurred_at, .. }
            | MembershipEvent::Activated { occurred_at, .. }
            | MembershipEvent::Renewed { occurred_at, .. }
            | MembershipEvent::PaymentFailed { occurred_at, .. }
            | MembershipEvent::PaymentRecovered { occurred_at, .. }
            | MembershipEvent::Cancelled { occurred_at, .. }
            | MembershipEvent::Reactivated { occurred_at, .. }
            | MembershipEvent::Expired { occurred_at, .. }
            | MembershipEvent::TierUpgraded { occurred_at, .. }
            | MembershipEvent::AccessChecked { occurred_at, .. } => *occurred_at,
        }
    }

    /// Returns the event ID for this event.
    pub fn event_id(&self) -> &EventId {
        match self {
            MembershipEvent::Created { event_id, .. }
            | MembershipEvent::Activated { event_id, .. }
            | MembershipEvent::Renewed { event_id, .. }
            | MembershipEvent::PaymentFailed { event_id, .. }
            | MembershipEvent::PaymentRecovered { event_id, .. }
            | MembershipEvent::Cancelled { event_id, .. }
            | MembershipEvent::Reactivated { event_id, .. }
            | MembershipEvent::Expired { event_id, .. }
            | MembershipEvent::TierUpgraded { event_id, .. }
            | MembershipEvent::AccessChecked { event_id, .. } => event_id,
        }
    }
}

impl DomainEvent for MembershipEvent {
    fn event_type(&self) -> &'static str {
        MembershipEvent::event_type(self)
    }

    fn schema_version(&self) -> u32 {
        // All membership events are currently at version 1
        1
    }

    fn aggregate_id(&self) -> String {
        // For AccessChecked, use user_id if no membership_id
        self.membership_id()
            .map(|id| id.to_string())
            .unwrap_or_else(|| self.user_id().to_string())
    }

    fn aggregate_type(&self) -> &'static str {
        "Membership"
    }

    fn occurred_at(&self) -> Timestamp {
        MembershipEvent::occurred_at(self)
    }

    fn event_id(&self) -> EventId {
        MembershipEvent::event_id(self).clone()
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

    fn test_event_id() -> EventId {
        EventId::new()
    }

    fn now() -> Timestamp {
        Timestamp::now()
    }

    // ============================================================
    // Event Construction Tests
    // ============================================================

    #[test]
    fn created_event_for_free_membership() {
        let event = MembershipEvent::Created {
            event_id: test_event_id(),
            membership_id: test_membership_id(),
            user_id: test_user_id(),
            tier: MembershipTier::Free,
            is_free: true,
            promo_code: Some("WORKSHOP2026".to_string()),
            occurred_at: now(),
        };

        assert_eq!(event.event_type(), "membership.created.v1");
        assert!(event.membership_id().is_some());
    }

    #[test]
    fn created_event_for_paid_membership() {
        let event = MembershipEvent::Created {
            event_id: test_event_id(),
            membership_id: test_membership_id(),
            user_id: test_user_id(),
            tier: MembershipTier::Monthly,
            is_free: false,
            promo_code: None,
            occurred_at: now(),
        };

        assert_eq!(event.event_type(), "membership.created.v1");
        assert!(!matches!(
            event,
            MembershipEvent::Created { is_free: true, .. }
        ));
    }

    #[test]
    fn activated_event_contains_period_dates() {
        let period_start = now();
        let period_end = now().add_days(30);

        let event = MembershipEvent::Activated {
            event_id: test_event_id(),
            membership_id: test_membership_id(),
            user_id: test_user_id(),
            tier: MembershipTier::Monthly,
            period_start,
            period_end,
            occurred_at: now(),
        };

        assert_eq!(event.event_type(), "membership.activated.v1");
        if let MembershipEvent::Activated {
            period_start: ps,
            period_end: pe,
            ..
        } = event
        {
            assert_eq!(ps, period_start);
            assert_eq!(pe, period_end);
        } else {
            panic!("Expected Activated event");
        }
    }

    #[test]
    fn payment_failed_event_tracks_retries() {
        let next_retry = now().add_days(1);

        let event = MembershipEvent::PaymentFailed {
            event_id: test_event_id(),
            membership_id: test_membership_id(),
            user_id: test_user_id(),
            attempt_count: 2,
            next_retry_at: Some(next_retry),
            occurred_at: now(),
        };

        assert_eq!(event.event_type(), "membership.payment_failed.v1");
        if let MembershipEvent::PaymentFailed {
            attempt_count,
            next_retry_at,
            ..
        } = event
        {
            assert_eq!(attempt_count, 2);
            assert_eq!(next_retry_at, Some(next_retry));
        } else {
            panic!("Expected PaymentFailed event");
        }
    }

    #[test]
    fn cancelled_event_has_effective_date() {
        let effective = now().add_days(30);

        let event = MembershipEvent::Cancelled {
            event_id: test_event_id(),
            membership_id: test_membership_id(),
            user_id: test_user_id(),
            effective_at: effective,
            occurred_at: now(),
        };

        assert_eq!(event.event_type(), "membership.cancelled.v1");
        if let MembershipEvent::Cancelled { effective_at, .. } = event {
            assert_eq!(effective_at, effective);
        } else {
            panic!("Expected Cancelled event");
        }
    }

    #[test]
    fn expired_event_captures_reason() {
        let event = MembershipEvent::Expired {
            event_id: test_event_id(),
            membership_id: test_membership_id(),
            user_id: test_user_id(),
            reason: ExpiredReason::GracePeriodExceeded,
            occurred_at: now(),
        };

        assert_eq!(event.event_type(), "membership.expired.v1");
        if let MembershipEvent::Expired { reason, .. } = event {
            assert_eq!(reason, ExpiredReason::GracePeriodExceeded);
        } else {
            panic!("Expected Expired event");
        }
    }

    #[test]
    fn tier_upgraded_event_captures_both_tiers() {
        let event = MembershipEvent::TierUpgraded {
            event_id: test_event_id(),
            membership_id: test_membership_id(),
            user_id: test_user_id(),
            previous_tier: MembershipTier::Free,
            new_tier: MembershipTier::Monthly,
            occurred_at: now(),
        };

        assert_eq!(event.event_type(), "membership.tier_upgraded.v1");
        if let MembershipEvent::TierUpgraded {
            previous_tier,
            new_tier,
            ..
        } = event
        {
            assert_eq!(previous_tier, MembershipTier::Free);
            assert_eq!(new_tier, MembershipTier::Monthly);
        } else {
            panic!("Expected TierUpgraded event");
        }
    }

    #[test]
    fn access_checked_event_allows_none_membership() {
        let event = MembershipEvent::AccessChecked {
            event_id: test_event_id(),
            membership_id: None,
            user_id: test_user_id(),
            resource: "session.create".to_string(),
            granted: false,
            occurred_at: now(),
        };

        assert_eq!(event.event_type(), "membership.access_checked.v1");
        assert!(event.membership_id().is_none());
    }

    // ============================================================
    // Event Type Tests
    // ============================================================

    #[test]
    fn all_event_types_are_namespaced() {
        let events = vec![
            MembershipEvent::Created {
                event_id: test_event_id(),
                membership_id: test_membership_id(),
                user_id: test_user_id(),
                tier: MembershipTier::Free,
                is_free: true,
                promo_code: None,
                occurred_at: now(),
            },
            MembershipEvent::Activated {
                event_id: test_event_id(),
                membership_id: test_membership_id(),
                user_id: test_user_id(),
                tier: MembershipTier::Monthly,
                period_start: now(),
                period_end: now(),
                occurred_at: now(),
            },
            MembershipEvent::Renewed {
                event_id: test_event_id(),
                membership_id: test_membership_id(),
                user_id: test_user_id(),
                new_period_start: now(),
                new_period_end: now(),
                occurred_at: now(),
            },
            MembershipEvent::PaymentFailed {
                event_id: test_event_id(),
                membership_id: test_membership_id(),
                user_id: test_user_id(),
                attempt_count: 1,
                next_retry_at: None,
                occurred_at: now(),
            },
            MembershipEvent::PaymentRecovered {
                event_id: test_event_id(),
                membership_id: test_membership_id(),
                user_id: test_user_id(),
                occurred_at: now(),
            },
            MembershipEvent::Cancelled {
                event_id: test_event_id(),
                membership_id: test_membership_id(),
                user_id: test_user_id(),
                effective_at: now(),
                occurred_at: now(),
            },
            MembershipEvent::Reactivated {
                event_id: test_event_id(),
                membership_id: test_membership_id(),
                user_id: test_user_id(),
                occurred_at: now(),
            },
            MembershipEvent::Expired {
                event_id: test_event_id(),
                membership_id: test_membership_id(),
                user_id: test_user_id(),
                reason: ExpiredReason::CancelledPeriodEnd,
                occurred_at: now(),
            },
            MembershipEvent::TierUpgraded {
                event_id: test_event_id(),
                membership_id: test_membership_id(),
                user_id: test_user_id(),
                previous_tier: MembershipTier::Free,
                new_tier: MembershipTier::Monthly,
                occurred_at: now(),
            },
            MembershipEvent::AccessChecked {
                event_id: test_event_id(),
                membership_id: Some(test_membership_id()),
                user_id: test_user_id(),
                resource: "test".to_string(),
                granted: true,
                occurred_at: now(),
            },
        ];

        for event in events {
            assert!(
                event.event_type().starts_with("membership."),
                "Event type {} should be namespaced with 'membership.'",
                event.event_type()
            );
        }
    }

    // ============================================================
    // ExpiredReason Tests
    // ============================================================

    #[test]
    fn expired_reason_display() {
        assert_eq!(
            ExpiredReason::CancelledPeriodEnd.to_string(),
            "cancelled_period_end"
        );
        assert_eq!(
            ExpiredReason::GracePeriodExceeded.to_string(),
            "grace_period_exceeded"
        );
        assert_eq!(ExpiredReason::PaymentTimeout.to_string(), "payment_timeout");
        assert_eq!(ExpiredReason::FreeTierExpiry.to_string(), "free_tier_expiry");
    }

    #[test]
    fn expired_reason_serialization_round_trip() {
        let reasons = vec![
            ExpiredReason::CancelledPeriodEnd,
            ExpiredReason::GracePeriodExceeded,
            ExpiredReason::PaymentTimeout,
            ExpiredReason::FreeTierExpiry,
        ];

        for reason in reasons {
            let json = serde_json::to_string(&reason).unwrap();
            let restored: ExpiredReason = serde_json::from_str(&json).unwrap();
            assert_eq!(reason, restored);
        }
    }

    // ============================================================
    // Serialization Tests
    // ============================================================

    #[test]
    fn membership_event_serializes_to_json() {
        let event = MembershipEvent::Created {
            event_id: test_event_id(),
            membership_id: test_membership_id(),
            user_id: test_user_id(),
            tier: MembershipTier::Monthly,
            is_free: false,
            promo_code: None,
            occurred_at: now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Created"));
        assert!(json.contains("membership_id"));
        assert!(json.contains("user_id"));
        assert!(json.contains("tier"));
    }

    #[test]
    fn membership_event_deserializes_from_json() {
        let event = MembershipEvent::PaymentRecovered {
            event_id: test_event_id(),
            membership_id: test_membership_id(),
            user_id: test_user_id(),
            occurred_at: now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let restored: MembershipEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.event_type(), restored.event_type());
    }

    // ============================================================
    // Accessor Method Tests
    // ============================================================

    #[test]
    fn user_id_accessor_returns_correct_value() {
        let user_id = test_user_id();
        let events = vec![
            MembershipEvent::Created {
                event_id: test_event_id(),
                membership_id: test_membership_id(),
                user_id: user_id.clone(),
                tier: MembershipTier::Free,
                is_free: true,
                promo_code: None,
                occurred_at: now(),
            },
            MembershipEvent::AccessChecked {
                event_id: test_event_id(),
                membership_id: None,
                user_id: user_id.clone(),
                resource: "test".to_string(),
                granted: true,
                occurred_at: now(),
            },
        ];

        for event in events {
            assert_eq!(event.user_id(), &user_id);
        }
    }

    #[test]
    fn occurred_at_accessor_returns_correct_value() {
        let occurred_at = now();
        let event = MembershipEvent::Reactivated {
            event_id: test_event_id(),
            membership_id: test_membership_id(),
            user_id: test_user_id(),
            occurred_at,
        };

        assert_eq!(event.occurred_at(), occurred_at);
    }
}
