# Feature: Subscription State Machine

**Module:** membership
**Type:** Domain Model
**Priority:** P0
**Status:** Specification Complete

> Complete state machine specification for membership subscriptions, including all states, transitions, triggers, and side effects.

---

## Overview

The membership subscription follows a strict state machine. This specification defines:
1. All possible states
2. Valid state transitions
3. Triggers for each transition
4. Side effects (events, notifications)
5. Edge cases and recovery scenarios

---

## State Diagram

```
                                    ┌─────────────────┐
                                    │    PENDING      │
                                    │  (Created, no   │
                                    │   payment yet)  │
                                    └────────┬────────┘
                                             │
                    ┌────────────────────────┼────────────────────────┐
                    │                        │                        │
                    │ payment_timeout        │ checkout_complete      │ free_promo
                    │ (72 hours)             │ (Stripe webhook)       │ (immediate)
                    ▼                        ▼                        │
           ┌────────────────┐       ┌────────────────┐               │
           │    EXPIRED     │       │    ACTIVE      │◄──────────────┘
           │ (No access)    │       │  (Full access) │
           └───────┬────────┘       └───────┬────────┘
                   │                        │
                   │ reactivate             │ ┌─────────────────────┐
                   │ (new payment)          │ │ automatic_renewal   │
                   │                        │ │ (Stripe webhook)    │
                   └────────────────────────┤ └──────┬──────────────┘
                                            │        │
                                            │        │
                    ┌───────────────────────┴───┬────┘
                    │                           │
                    │ cancel_requested          │ payment_failed
                    │ (user action)             │ (Stripe webhook)
                    ▼                           ▼
           ┌────────────────┐       ┌────────────────┐
           │   CANCELLED    │       │   PAST_DUE     │
           │ (Access until  │       │ (Grace period, │
           │  period end)   │       │  still access) │
           └───────┬────────┘       └───────┬────────┘
                   │                        │
                   │ period_end             │ ┌─────────────────────┐
                   │ (scheduled job)        │ │ payment_recovered   │
                   │                        │ │ (Stripe webhook)    │
                   │                        │ └──────┬──────────────┘
                   │                        │        │
                   │ ┌──────────────────────┤        │
                   │ │ grace_period_expired │        │
                   │ │ (scheduled job)      │        │
                   │ │                      │        │
                   ▼ ▼                      ▼        │
           ┌────────────────┐       ┌───────────────┴┐
           │    EXPIRED     │◄──────│    ACTIVE      │
           │  (No access)   │       │  (Recovered)   │
           └────────────────┘       └────────────────┘
                   │
                   │ resubscribe
                   │ (new checkout)
                   ▼
           ┌────────────────┐
           │    PENDING     │
           │   (New sub)    │
           └────────────────┘
```

---

## States

### PENDING

Initial state for paid subscriptions awaiting first payment.

| Property | Value |
|----------|-------|
| **has_access** | `false` |
| **can_cancel** | `true` (deletes pending membership) |
| **timeout** | 72 hours (then auto-expire) |
| **stripe_status** | `incomplete` |

**Entry conditions:**
- User initiates paid checkout (`create_paid_membership`)
- Stripe customer created
- Checkout session created

**Exit transitions:**

| Trigger | Target State | Action |
|---------|--------------|--------|
| `checkout.session.completed` | ACTIVE | Set period start/end |
| Payment timeout (72h) | EXPIRED | Delete pending record |

---

### ACTIVE

Fully paid, user has complete access.

| Property | Value |
|----------|-------|
| **has_access** | `true` |
| **can_cancel** | `true` |
| **can_upgrade** | `true` |
| **stripe_status** | `active` |

**Entry conditions:**
- Payment received (new or renewal)
- Reactivation payment received
- Free promo code validated

**Exit transitions:**

| Trigger | Target State | Condition | Action |
|---------|--------------|-----------|--------|
| `customer.subscription.deleted` | CANCELLED | `cancel_at_period_end=true` | Set cancelled_at |
| `invoice.payment_failed` | PAST_DUE | - | Send payment failed email |
| Period end without renewal | EXPIRED | Free tier only | - |

---

### PAST_DUE

Payment failed but within grace period.

| Property | Value |
|----------|-------|
| **has_access** | `true` (grace period) |
| **can_cancel** | `true` |
| **grace_period** | 7 days (Stripe default) |
| **stripe_status** | `past_due` |

**Entry conditions:**
- `invoice.payment_failed` webhook received
- Stripe auto-retry in progress

**Exit transitions:**

| Trigger | Target State | Action |
|---------|--------------|--------|
| `invoice.payment_succeeded` | ACTIVE | Clear past_due flag |
| Grace period expired (7d) | EXPIRED | Revoke access |
| Manual cancellation | CANCELLED | Set end = now |

**Grace period behavior:**
- Stripe retries payment up to 4 times over 7 days
- User sees "Payment failed" warning in UI
- Email notifications sent at: Day 0, Day 3, Day 6

---

### CANCELLED

User requested cancellation; access continues until period end.

| Property | Value |
|----------|-------|
| **has_access** | `true` (until period_end) |
| **can_reactivate** | `true` (before period_end) |
| **auto_renew** | `false` |
| **stripe_status** | `active` with `cancel_at_period_end=true` |

**Entry conditions:**
- User clicks "Cancel membership"
- API receives `cancel_membership` command
- Stripe updated with `cancel_at_period_end=true`

**Exit transitions:**

| Trigger | Target State | Condition | Action |
|---------|--------------|-----------|--------|
| Period end reached | EXPIRED | - | Send expiry email |
| Reactivation requested | ACTIVE | Before period_end | Resume billing |

**Note:** Cancelled subscriptions can be reactivated before period_end without new checkout.

---

### EXPIRED

No longer has access; needs to resubscribe.

| Property | Value |
|----------|-------|
| **has_access** | `false` |
| **can_resubscribe** | `true` |
| **data_retention** | Sessions preserved for 90 days |
| **stripe_status** | `canceled` (fully ended) |

**Entry conditions:**
- Cancelled subscription reached period_end
- Past_due subscription exceeded grace period
- Pending subscription timed out (72h)
- Free tier reached annual expiry

**Exit transitions:**

| Trigger | Target State | Action |
|---------|--------------|--------|
| New checkout completed | PENDING → ACTIVE | Create new subscription |

**Data handling:**
- User's sessions remain accessible (read-only) for 90 days
- After 90 days, sessions are archived (can be restored on resubscribe)
- User data never deleted, just made inaccessible

---

### TRIALING (Future)

Reserved for future trial period implementation.

| Property | Value |
|----------|-------|
| **has_access** | `true` |
| **trial_days** | 14 (planned) |
| **payment_method_required** | `true` |
| **stripe_status** | `trialing` |

**Not implemented in v1.0** - included for schema completeness.

---

## Transition Rules

### State Transition Matrix

| From State | To State | Valid? | Trigger |
|------------|----------|--------|---------|
| PENDING | ACTIVE | ✓ | Payment success |
| PENDING | EXPIRED | ✓ | Payment timeout |
| PENDING | CANCELLED | ✗ | (Delete instead) |
| ACTIVE | PAST_DUE | ✓ | Payment failed |
| ACTIVE | CANCELLED | ✓ | User request |
| ACTIVE | EXPIRED | ✓ | Free tier period end |
| ACTIVE | ACTIVE | ✓ | Renewal |
| PAST_DUE | ACTIVE | ✓ | Payment recovered |
| PAST_DUE | EXPIRED | ✓ | Grace period exceeded |
| PAST_DUE | CANCELLED | ✓ | User request |
| CANCELLED | EXPIRED | ✓ | Period end |
| CANCELLED | ACTIVE | ✓ | Reactivation |
| EXPIRED | PENDING | ✓ | New checkout |
| EXPIRED | ACTIVE | ✗ | (Must go through PENDING) |

### Transition Implementation

```rust
impl MembershipStatus {
    /// Validates if transition is allowed
    pub fn can_transition_to(&self, target: &MembershipStatus) -> bool {
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

    /// Performs transition with validation
    pub fn transition_to(&self, target: MembershipStatus) -> Result<MembershipStatus, DomainError> {
        if self.can_transition_to(&target) {
            Ok(target)
        } else {
            Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!("Cannot transition from {:?} to {:?}", self, target),
            ))
        }
    }
}
```

---

## Triggers & Events

### Stripe Webhook Triggers

| Webhook Event | Membership Action | State Transition |
|---------------|-------------------|------------------|
| `checkout.session.completed` | Activate membership | PENDING → ACTIVE |
| `invoice.payment_succeeded` | Renew/recover membership | ACTIVE → ACTIVE or PAST_DUE → ACTIVE |
| `invoice.payment_failed` | Mark past due | ACTIVE → PAST_DUE |
| `customer.subscription.updated` | Sync status | Update period dates |
| `customer.subscription.deleted` | Handle cancellation | Various |

### Internal Triggers (Scheduled Jobs)

| Job | Schedule | Action |
|-----|----------|--------|
| Expire pending | Every hour | PENDING → EXPIRED if > 72h |
| Expire cancelled | Daily at 00:00 UTC | CANCELLED → EXPIRED if period_end passed |
| Expire past_due | Daily at 00:00 UTC | PAST_DUE → EXPIRED if grace exceeded |
| Renewal reminders | Daily at 09:00 UTC | Email users 7 days before expiry |

### Domain Events Emitted

| State Transition | Event(s) Emitted |
|-----------------|------------------|
| → ACTIVE (new) | `MembershipCreated`, `MembershipActivated` |
| → ACTIVE (renewal) | `MembershipRenewed` |
| → PAST_DUE | `PaymentFailed` |
| → CANCELLED | `MembershipCancelled` |
| → EXPIRED | `MembershipExpired` |
| PAST_DUE → ACTIVE | `PaymentReceived` |

---

## Side Effects

### Email Notifications

| Transition | Email Template | Send Time |
|------------|----------------|-----------|
| → ACTIVE | `welcome` | Immediate |
| → PAST_DUE | `payment_failed` | Immediate |
| → PAST_DUE (day 3) | `payment_reminder` | 3 days after |
| → CANCELLED | `cancellation_confirmed` | Immediate |
| → EXPIRED (from cancelled) | `subscription_ended` | Immediate |
| → EXPIRED (from past_due) | `access_revoked` | Immediate |
| 7 days before expiry | `renewal_reminder` | Scheduled |

### Access Control Effects

| State | Session Creation | Session Read | Export | AI Features |
|-------|-----------------|--------------|--------|-------------|
| PENDING | ✗ | ✗ | ✗ | ✗ |
| ACTIVE | ✓ | ✓ | ✓ | ✓ |
| PAST_DUE | ✓ | ✓ | ✓ | ✓ |
| CANCELLED | ✓ | ✓ | ✓ | ✓ |
| EXPIRED | ✗ | Read-only | ✗ | ✗ |

---

## Edge Cases

### 1. Webhook Out of Order

**Scenario:** `invoice.payment_succeeded` arrives before `checkout.session.completed`

**Solution:**
- Store Stripe subscription ID on checkout creation
- Both webhooks find membership by subscription_id
- If already ACTIVE, `payment_succeeded` is a no-op

### 2. Multiple Payment Attempts

**Scenario:** User in PAST_DUE updates card, multiple `payment_succeeded` webhooks

**Solution:**
- Use webhook event ID for idempotency
- Track `last_processed_event_id`
- Ignore duplicate events

### 3. Cancellation During Past Due

**Scenario:** User cancels while in PAST_DUE state

**Solution:**
- Allow transition PAST_DUE → CANCELLED
- Set period_end to now (not billing cycle end)
- Cancel Stripe subscription immediately (not at period end)

### 4. Free Tier Expiry

**Scenario:** Workshop code expires after 1 year

**Solution:**
- Free tier has real period_end (created_at + 1 year)
- Scheduled job expires ACTIVE → EXPIRED for free tier
- User sees prompt to upgrade before expiry

### 5. Reactivation Race Condition

**Scenario:** User reactivates CANCELLED subscription same moment period ends

**Solution:**
- Always check period_end before reactivation
- If period_end passed during request, redirect to new checkout
- Use optimistic locking on membership record

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Required |
| Authorization Model | User owns membership; only owner can view/modify |
| Sensitive Data | Payment status, subscription amounts, Stripe IDs |
| Rate Limiting | Not Required (internal state machine) |
| Audit Logging | All state transitions must be logged |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| `stripe_subscription_id` | Confidential | Do not expose in API responses to end users |
| `stripe_customer_id` | Confidential | Do not expose in API responses to end users |
| `amount_paid` | Confidential | Mask in logs, display only to account owner |
| `period_end` | Internal | May be shown to user for billing clarity |
| `status` | Internal | Can be displayed to user |
| `tier` | Public | Can be displayed in UI |

### Security Controls

- **State Transition Logging**: All state transitions must emit audit events including:
  - Previous state
  - New state
  - Trigger source (webhook, scheduled job, user action)
  - Timestamp
  - User ID / Stripe event ID as appropriate
- **Fail-Secure State**: If state cannot be determined, treat as `Pending` (no access)
- **Idempotent Transitions**: State machine must handle duplicate webhook events gracefully

---

## Testing Checklist

### Unit Tests

- [x] `pending_can_transition_to_active`
- [x] `pending_can_transition_to_expired`
- [x] `pending_cannot_transition_to_cancelled`
- [x] `active_can_transition_to_past_due`
- [x] `active_can_transition_to_cancelled`
- [x] `active_can_renew_to_active`
- [x] `past_due_can_recover_to_active`
- [x] `past_due_can_expire`
- [x] `cancelled_can_reactivate_to_active`
- [x] `cancelled_can_expire`
- [x] `expired_cannot_directly_activate`
- [x] `has_access_true_for_active`
- [x] `has_access_true_for_past_due_in_grace`
- [x] `has_access_true_for_cancelled_before_period_end`
- [x] `has_access_false_for_expired`
- [x] `has_access_false_for_pending`

### Integration Tests

> **Note:** These tests depend on infrastructure from `stripe-webhook-handling.md` feature.
> They will be implemented as part of that feature.

- [ ] `webhook_checkout_complete_activates_pending` _(requires: webhook handler)_
- [ ] `webhook_payment_failed_marks_past_due` _(requires: webhook handler)_
- [ ] `webhook_payment_recovered_clears_past_due` _(requires: webhook handler)_
- [ ] `scheduled_job_expires_old_pending` _(requires: scheduled jobs)_
- [ ] `scheduled_job_expires_cancelled_at_period_end` _(requires: scheduled jobs)_
- [x] `reactivation_before_period_end_succeeds` _(requires: Membership aggregate)_
- [x] `reactivation_after_period_end_fails` _(requires: Membership aggregate)_

---

*Version: 1.0.0*
*Created: 2026-01-08*
*Module: membership*
