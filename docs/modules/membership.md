# Membership Module Specification

## Overview

The Membership module manages user subscriptions, access control, and payment integration. It gates access to the Choice Sherpa platform, supporting free workshop/beta users, monthly subscribers, and annual subscribers.

---

## Module Classification

| Attribute | Value |
|-----------|-------|
| **Type** | Full Module (Ports + Adapters) |
| **Language** | Rust |
| **Responsibility** | Membership lifecycle, access control, payment integration |
| **Domain Dependencies** | foundation |
| **External Dependencies** | `async-trait`, `sqlx`, `tokio`, `stripe-rust` |

---

## Architecture

### Hexagonal Structure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         MEMBERSHIP MODULE                                    │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         DOMAIN LAYER                                    │ │
│  │                                                                         │ │
│  │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│  │   │                  Membership Aggregate                             │ │ │
│  │   │                                                                   │ │ │
│  │   │   - id: MembershipId                                              │ │ │
│  │   │   - user_id: UserId                                               │ │ │
│  │   │   - tier: MembershipTier                                          │ │ │
│  │   │   - status: MembershipStatus                                      │ │ │
│  │   │   - billing_period: BillingPeriod                                 │ │ │
│  │   │   - current_period_start: Timestamp                               │ │ │
│  │   │   - current_period_end: Timestamp                                 │ │ │
│  │   │   - promo_code: Option<PromoCode>                                 │ │ │
│  │   │   - external_customer_id: Option<String>                          │ │ │
│  │   │   - external_subscription_id: Option<String>                      │ │ │
│  │   │                                                                   │ │ │
│  │   │   + create_free(user_id, promo_code) -> Result<Membership>        │ │ │
│  │   │   + create_paid(user_id, tier) -> Result<Membership>              │ │ │
│  │   │   + activate() -> Result<()>                                      │ │ │
│  │   │   + cancel() -> Result<()>                                        │ │ │
│  │   │   + expire() -> Result<()>                                        │ │ │
│  │   │   + renew(period_end: Timestamp) -> Result<()>                    │ │ │
│  │   │   + has_access() -> bool                                          │ │ │
│  │   └──────────────────────────────────────────────────────────────────┘ │ │
│  │                                                                         │ │
│  │   ┌────────────────────────────────────────────────────────────────┐   │ │
│  │   │                     Value Objects                               │   │ │
│  │   │   Money, MembershipTier, MembershipStatus, BillingPeriod,       │   │ │
│  │   │   PromoCode, PlanPrice                                          │   │ │
│  │   └────────────────────────────────────────────────────────────────┘   │ │
│  │                                                                         │ │
│  │   ┌────────────────────────────────────────────────────────────────┐   │ │
│  │   │                   Domain Events                                 │   │ │
│  │   │   MembershipCreated, MembershipActivated, MembershipCancelled,  │   │ │
│  │   │   MembershipExpired, MembershipRenewed, PaymentReceived,        │   │ │
│  │   │   PaymentFailed                                                 │   │ │
│  │   └────────────────────────────────────────────────────────────────┘   │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
│                                      ▼                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                          PORT LAYER                                     │ │
│  │                                                                         │ │
│  │   ┌──────────────────────────┐  ┌────────────────────────────────────┐ │ │
│  │   │  MembershipRepository    │  │  MembershipReader                   │ │ │
│  │   │  (Write operations)      │  │  (Query operations - CQRS)         │ │ │
│  │   │                          │  │                                     │ │ │
│  │   │  + save(membership)      │  │  + get_by_user(user_id) -> View    │ │ │
│  │   │  + update(membership)    │  │  + check_access(user_id) -> bool   │ │ │
│  │   │  + find_by_id(id)        │  │  + list_expiring(days) -> []       │ │ │
│  │   │  + find_by_user(user_id) │  │                                     │ │ │
│  │   └──────────────────────────┘  └────────────────────────────────────┘ │ │
│  │                                                                         │ │
│  │   ┌──────────────────────────┐  ┌────────────────────────────────────┐ │ │
│  │   │  PaymentProvider         │  │  AccessChecker                      │ │ │
│  │   │  (External - Stripe)     │  │  (Called by other modules)          │ │ │
│  │   │                          │  │                                     │ │ │
│  │   │  + create_customer()     │  │  + can_create_session(user_id)     │ │ │
│  │   │  + create_subscription() │  │  + get_tier(user_id) -> Tier       │ │ │
│  │   │  + cancel_subscription() │  │  + get_limits(user_id) -> Limits   │ │ │
│  │   │  + create_checkout()     │  │                                     │ │ │
│  │   │  + get_portal_url()      │  │                                     │ │ │
│  │   └──────────────────────────┘  └────────────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
│                                      ▼                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                        ADAPTER LAYER                                    │ │
│  │                                                                         │ │
│  │   ┌─────────────────┐  ┌─────────────────┐  ┌──────────────────────┐   │ │
│  │   │ PostgresMember  │  │ PostgresMember  │  │  StripePayment       │   │ │
│  │   │ Repository      │  │ Reader          │  │  Adapter             │   │ │
│  │   └─────────────────┘  └─────────────────┘  └──────────────────────┘   │ │
│  │                                                                         │ │
│  │   ┌─────────────────────────────────────────────────────────────────┐  │ │
│  │   │                    HTTP Handlers                                 │  │ │
│  │   │   GET /membership, POST /membership/checkout,                    │  │ │
│  │   │   POST /membership/cancel, POST /webhooks/stripe                 │  │ │
│  │   └─────────────────────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Domain Layer

### Value Objects

#### Money (CRITICAL: Uses Integers for Cents)

```rust
use serde::{Deserialize, Serialize};
use std::fmt;

/// Money value object - stores amounts in smallest currency unit (cents)
///
/// IMPORTANT: We use i64 (not f64) to avoid floating-point precision issues.
/// $19.99 is stored as 1999 cents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    /// Amount in smallest currency unit (e.g., cents for USD/CAD)
    amount_cents: i64,
    /// ISO 4217 currency code
    currency: Currency,
}

impl Money {
    /// Creates a new Money value from cents
    pub fn from_cents(cents: i64, currency: Currency) -> Self {
        Self {
            amount_cents: cents,
            currency,
        }
    }

    /// Creates a Money value from dollars (converts to cents internally)
    pub fn from_dollars(dollars: i64, cents: i64, currency: Currency) -> Self {
        Self {
            amount_cents: dollars * 100 + cents,
            currency,
        }
    }

    /// Returns the amount in cents
    pub fn cents(&self) -> i64 {
        self.amount_cents
    }

    /// Returns the currency
    pub fn currency(&self) -> Currency {
        self.currency
    }

    /// Returns a display string like "$19.99 CAD"
    pub fn display(&self) -> String {
        let dollars = self.amount_cents / 100;
        let cents = (self.amount_cents % 100).abs();
        format!("${}.{:02} {}", dollars, cents, self.currency)
    }

    /// Zero amount
    pub fn zero(currency: Currency) -> Self {
        Self::from_cents(0, currency)
    }

    /// Arithmetic operations
    pub fn add(&self, other: &Money) -> Result<Money, DomainError> {
        if self.currency != other.currency {
            return Err(DomainError::new(
                ErrorCode::ValidationFailed,
                "Cannot add money with different currencies",
            ));
        }
        Ok(Money::from_cents(
            self.amount_cents + other.amount_cents,
            self.currency,
        ))
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    CAD,
    USD,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::CAD => write!(f, "CAD"),
            Currency::USD => write!(f, "USD"),
        }
    }
}
```

#### MembershipTier

```rust
use serde::{Deserialize, Serialize};

/// Available membership tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipTier {
    /// Free tier - workshop/beta users with promo code
    Free,
    /// Monthly subscription
    Monthly,
    /// Annual subscription
    Annual,
}

impl MembershipTier {
    /// Returns whether this tier requires payment
    pub fn requires_payment(&self) -> bool {
        !matches!(self, MembershipTier::Free)
    }

    /// Returns the billing period for this tier
    pub fn billing_period(&self) -> BillingPeriod {
        match self {
            MembershipTier::Free => BillingPeriod::Annual, // Free is still annual
            MembershipTier::Monthly => BillingPeriod::Monthly,
            MembershipTier::Annual => BillingPeriod::Annual,
        }
    }

    /// Returns display name
    pub fn display_name(&self) -> &'static str {
        match self {
            MembershipTier::Free => "Free (Workshop)",
            MembershipTier::Monthly => "Monthly",
            MembershipTier::Annual => "Annual",
        }
    }
}

impl Default for MembershipTier {
    fn default() -> Self {
        MembershipTier::Free
    }
}
```

#### MembershipStatus

```rust
use serde::{Deserialize, Serialize};

/// Membership lifecycle status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipStatus {
    /// Membership is pending activation (awaiting payment confirmation)
    #[default]
    Pending,
    /// Membership is active and has access
    Active,
    /// Payment failed, in grace period (typically 3-7 days)
    PastDue,
    /// User cancelled, but still active until period end
    Cancelled,
    /// Membership has expired (period ended without renewal)
    Expired,
    /// Trial period (if applicable in future)
    Trialing,
}

impl MembershipStatus {
    /// Returns true if the user should have access to the platform
    pub fn has_access(&self) -> bool {
        matches!(
            self,
            MembershipStatus::Active
                | MembershipStatus::PastDue
                | MembershipStatus::Cancelled
                | MembershipStatus::Trialing
        )
    }

    /// Returns true if this status can transition to the target status
    pub fn can_transition_to(&self, target: &MembershipStatus) -> bool {
        use MembershipStatus::*;
        matches!(
            (self, target),
            (Pending, Active)
                | (Pending, Expired) // Payment never completed
                | (Active, PastDue)
                | (Active, Cancelled)
                | (Active, Expired)
                | (PastDue, Active) // Payment recovered
                | (PastDue, Expired)
                | (Cancelled, Active) // Reactivation
                | (Cancelled, Expired)
                | (Trialing, Active)
                | (Trialing, Expired)
                | (Expired, Active) // Renewal
        )
    }

    /// Returns whether automatic renewal is possible
    pub fn can_renew(&self) -> bool {
        matches!(
            self,
            MembershipStatus::Active | MembershipStatus::PastDue
        )
    }
}
```

#### BillingPeriod

```rust
use chrono::{DateTime, Utc, Months};
use serde::{Deserialize, Serialize};

/// Billing period for subscriptions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingPeriod {
    Monthly,
    Annual,
}

impl BillingPeriod {
    /// Calculates the end date from a start date
    pub fn calculate_end(&self, start: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            BillingPeriod::Monthly => start + Months::new(1),
            BillingPeriod::Annual => start + Months::new(12),
        }
    }

    /// Returns the number of months in this period
    pub fn months(&self) -> u32 {
        match self {
            BillingPeriod::Monthly => 1,
            BillingPeriod::Annual => 12,
        }
    }
}
```

#### PromoCode

```rust
use serde::{Deserialize, Serialize};

/// Promotional code for workshop/beta access
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromoCode {
    code: String,
    promo_type: PromoType,
}

impl PromoCode {
    /// Creates a new promo code after validation
    pub fn new(code: impl Into<String>) -> Result<Self, DomainError> {
        let code = code.into().trim().to_uppercase();

        if code.is_empty() {
            return Err(DomainError::new(
                ErrorCode::ValidationFailed,
                "Promo code cannot be empty",
            ));
        }

        if code.len() > 50 {
            return Err(DomainError::new(
                ErrorCode::ValidationFailed,
                "Promo code cannot exceed 50 characters",
            ));
        }

        // Determine promo type from prefix
        let promo_type = if code.starts_with("WORKSHOP") {
            PromoType::Workshop
        } else if code.starts_with("BETA") {
            PromoType::BetaUser
        } else if code.starts_with("PARTNER") {
            PromoType::Partner
        } else {
            PromoType::General
        };

        Ok(Self { code, promo_type })
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn promo_type(&self) -> PromoType {
        self.promo_type
    }

    /// Returns the tier this promo code grants
    pub fn granted_tier(&self) -> MembershipTier {
        // All promo codes currently grant Free tier
        MembershipTier::Free
    }

    /// Returns the duration this promo grants
    pub fn granted_period(&self) -> BillingPeriod {
        BillingPeriod::Annual
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromoType {
    /// Workshop attendee
    Workshop,
    /// Beta tester
    BetaUser,
    /// Partner organization
    Partner,
    /// General promotional code
    General,
}
```

#### PlanPrice

```rust
use serde::{Deserialize, Serialize};

/// Pricing configuration for a membership tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanPrice {
    pub tier: MembershipTier,
    pub price: Money,
    pub billing_period: BillingPeriod,
    pub stripe_price_id: Option<String>,
}

impl PlanPrice {
    /// Returns the standard pricing configuration
    ///
    /// Note: Prices are in CAD cents
    pub fn get_standard_prices() -> Vec<PlanPrice> {
        vec![
            PlanPrice {
                tier: MembershipTier::Free,
                price: Money::zero(Currency::CAD),
                billing_period: BillingPeriod::Annual,
                stripe_price_id: None,
            },
            PlanPrice {
                tier: MembershipTier::Monthly,
                price: Money::from_cents(1999, Currency::CAD), // $19.99 CAD
                billing_period: BillingPeriod::Monthly,
                stripe_price_id: Some("price_monthly_cad".into()),
            },
            PlanPrice {
                tier: MembershipTier::Annual,
                price: Money::from_cents(14999, Currency::CAD), // $149.99 CAD
                billing_period: BillingPeriod::Annual,
                stripe_price_id: Some("price_annual_cad".into()),
            },
        ]
    }

    /// Returns the monthly equivalent price (for comparison display)
    pub fn monthly_equivalent(&self) -> Money {
        let months = self.billing_period.months() as i64;
        Money::from_cents(self.price.cents() / months, self.price.currency())
    }
}
```

### Membership Aggregate

```rust
use crate::foundation::{DomainError, ErrorCode, MembershipId, Timestamp, UserId};

/// The Membership aggregate - manages user subscription state
#[derive(Debug, Clone)]
pub struct Membership {
    id: MembershipId,
    user_id: UserId,
    tier: MembershipTier,
    status: MembershipStatus,
    billing_period: BillingPeriod,
    current_period_start: Timestamp,
    current_period_end: Timestamp,
    promo_code: Option<PromoCode>,
    external_customer_id: Option<String>,
    external_subscription_id: Option<String>,
    created_at: Timestamp,
    updated_at: Timestamp,
    cancelled_at: Option<Timestamp>,
    domain_events: Vec<MembershipEvent>,
}

impl Membership {
    /// Creates a free membership for a workshop/beta user
    pub fn create_free(
        user_id: UserId,
        promo_code: PromoCode,
    ) -> Result<Self, DomainError> {
        let now = Timestamp::now();
        let id = MembershipId::new();
        let period = promo_code.granted_period();
        let period_end = period.calculate_end(now.as_datetime());

        let mut membership = Self {
            id,
            user_id: user_id.clone(),
            tier: MembershipTier::Free,
            status: MembershipStatus::Active, // Free is immediately active
            billing_period: period,
            current_period_start: now,
            current_period_end: Timestamp::from_datetime(period_end),
            promo_code: Some(promo_code.clone()),
            external_customer_id: None,
            external_subscription_id: None,
            created_at: now,
            updated_at: now,
            cancelled_at: None,
            domain_events: Vec::new(),
        };

        membership.record_event(MembershipEvent::Created {
            membership_id: id,
            user_id,
            tier: MembershipTier::Free,
            promo_code: Some(promo_code.code().to_string()),
            created_at: now,
        });

        Ok(membership)
    }

    /// Creates a paid membership (pending until payment confirmed)
    pub fn create_paid(user_id: UserId, tier: MembershipTier) -> Result<Self, DomainError> {
        if !tier.requires_payment() {
            return Err(DomainError::new(
                ErrorCode::ValidationFailed,
                "Use create_free for free tier memberships",
            ));
        }

        let now = Timestamp::now();
        let id = MembershipId::new();
        let period = tier.billing_period();

        let mut membership = Self {
            id,
            user_id: user_id.clone(),
            tier,
            status: MembershipStatus::Pending,
            billing_period: period,
            current_period_start: now,
            current_period_end: now, // Will be set when payment confirms
            promo_code: None,
            external_customer_id: None,
            external_subscription_id: None,
            created_at: now,
            updated_at: now,
            cancelled_at: None,
            domain_events: Vec::new(),
        };

        membership.record_event(MembershipEvent::Created {
            membership_id: id,
            user_id,
            tier,
            promo_code: None,
            created_at: now,
        });

        Ok(membership)
    }

    /// Reconstitutes from persistence (no events emitted)
    pub fn reconstitute(
        id: MembershipId,
        user_id: UserId,
        tier: MembershipTier,
        status: MembershipStatus,
        billing_period: BillingPeriod,
        current_period_start: Timestamp,
        current_period_end: Timestamp,
        promo_code: Option<PromoCode>,
        external_customer_id: Option<String>,
        external_subscription_id: Option<String>,
        created_at: Timestamp,
        updated_at: Timestamp,
        cancelled_at: Option<Timestamp>,
    ) -> Self {
        Self {
            id,
            user_id,
            tier,
            status,
            billing_period,
            current_period_start,
            current_period_end,
            promo_code,
            external_customer_id,
            external_subscription_id,
            created_at,
            updated_at,
            cancelled_at,
            domain_events: Vec::new(),
        }
    }

    // === Accessors ===

    pub fn id(&self) -> MembershipId {
        self.id
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn tier(&self) -> MembershipTier {
        self.tier
    }

    pub fn status(&self) -> MembershipStatus {
        self.status
    }

    pub fn billing_period(&self) -> BillingPeriod {
        self.billing_period
    }

    pub fn current_period_start(&self) -> Timestamp {
        self.current_period_start
    }

    pub fn current_period_end(&self) -> Timestamp {
        self.current_period_end
    }

    pub fn promo_code(&self) -> Option<&PromoCode> {
        self.promo_code.as_ref()
    }

    pub fn external_customer_id(&self) -> Option<&str> {
        self.external_customer_id.as_deref()
    }

    pub fn external_subscription_id(&self) -> Option<&str> {
        self.external_subscription_id.as_deref()
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    pub fn cancelled_at(&self) -> Option<Timestamp> {
        self.cancelled_at
    }

    // === Business Logic ===

    /// Returns true if the user has access to the platform
    pub fn has_access(&self) -> bool {
        if !self.status.has_access() {
            return false;
        }

        // Check if within current period
        let now = Timestamp::now();
        now <= self.current_period_end
    }

    /// Returns true if the user is the owner of this membership
    pub fn is_owner(&self, user_id: &UserId) -> bool {
        &self.user_id == user_id
    }

    /// Activates a pending membership (called when payment succeeds)
    pub fn activate(&mut self, period_end: Timestamp) -> Result<(), DomainError> {
        if !self.status.can_transition_to(&MembershipStatus::Active) {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!("Cannot activate membership with status {:?}", self.status),
            ));
        }

        let now = Timestamp::now();
        self.status = MembershipStatus::Active;
        self.current_period_start = now;
        self.current_period_end = period_end;
        self.updated_at = now;

        self.record_event(MembershipEvent::Activated {
            membership_id: self.id,
            activated_at: now,
            period_end,
        });

        Ok(())
    }

    /// Sets the external payment provider IDs
    pub fn set_external_ids(
        &mut self,
        customer_id: String,
        subscription_id: Option<String>,
    ) -> Result<(), DomainError> {
        self.external_customer_id = Some(customer_id);
        self.external_subscription_id = subscription_id;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Marks payment as failed (enters past due status)
    pub fn mark_payment_failed(&mut self) -> Result<(), DomainError> {
        if !self.status.can_transition_to(&MembershipStatus::PastDue) {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                "Cannot mark as past due from current status",
            ));
        }

        let now = Timestamp::now();
        self.status = MembershipStatus::PastDue;
        self.updated_at = now;

        self.record_event(MembershipEvent::PaymentFailed {
            membership_id: self.id,
            failed_at: now,
        });

        Ok(())
    }

    /// Recovers from past due when payment succeeds
    pub fn recover_payment(&mut self) -> Result<(), DomainError> {
        if self.status != MembershipStatus::PastDue {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                "Can only recover from past due status",
            ));
        }

        let now = Timestamp::now();
        self.status = MembershipStatus::Active;
        self.updated_at = now;

        self.record_event(MembershipEvent::PaymentReceived {
            membership_id: self.id,
            received_at: now,
        });

        Ok(())
    }

    /// Cancels the membership (remains active until period end)
    pub fn cancel(&mut self) -> Result<(), DomainError> {
        if !self.status.can_transition_to(&MembershipStatus::Cancelled) {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                "Cannot cancel membership with current status",
            ));
        }

        let now = Timestamp::now();
        self.status = MembershipStatus::Cancelled;
        self.cancelled_at = Some(now);
        self.updated_at = now;

        self.record_event(MembershipEvent::Cancelled {
            membership_id: self.id,
            cancelled_at: now,
            effective_end: self.current_period_end,
        });

        Ok(())
    }

    /// Expires the membership (no more access)
    pub fn expire(&mut self) -> Result<(), DomainError> {
        if !self.status.can_transition_to(&MembershipStatus::Expired) {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                "Cannot expire membership with current status",
            ));
        }

        let now = Timestamp::now();
        self.status = MembershipStatus::Expired;
        self.updated_at = now;

        self.record_event(MembershipEvent::Expired {
            membership_id: self.id,
            expired_at: now,
        });

        Ok(())
    }

    /// Renews the membership for another period
    pub fn renew(&mut self, new_period_end: Timestamp) -> Result<(), DomainError> {
        if !self.status.can_renew() {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                "Cannot renew membership with current status",
            ));
        }

        let now = Timestamp::now();
        self.current_period_start = self.current_period_end;
        self.current_period_end = new_period_end;
        self.status = MembershipStatus::Active;
        self.updated_at = now;

        self.record_event(MembershipEvent::Renewed {
            membership_id: self.id,
            renewed_at: now,
            new_period_end,
        });

        Ok(())
    }

    /// Reactivates an expired or cancelled membership
    pub fn reactivate(&mut self, period_end: Timestamp) -> Result<(), DomainError> {
        if !matches!(
            self.status,
            MembershipStatus::Expired | MembershipStatus::Cancelled
        ) {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                "Can only reactivate expired or cancelled memberships",
            ));
        }

        let now = Timestamp::now();
        self.status = MembershipStatus::Active;
        self.current_period_start = now;
        self.current_period_end = period_end;
        self.cancelled_at = None;
        self.updated_at = now;

        self.record_event(MembershipEvent::Activated {
            membership_id: self.id,
            activated_at: now,
            period_end,
        });

        Ok(())
    }

    /// Upgrades the membership tier
    pub fn upgrade(&mut self, new_tier: MembershipTier) -> Result<(), DomainError> {
        if new_tier == self.tier {
            return Err(DomainError::new(
                ErrorCode::ValidationFailed,
                "Already on this tier",
            ));
        }

        // Can only upgrade to a "higher" tier
        let tier_order = |t: &MembershipTier| match t {
            MembershipTier::Free => 0,
            MembershipTier::Monthly => 1,
            MembershipTier::Annual => 2,
        };

        if tier_order(&new_tier) <= tier_order(&self.tier) {
            return Err(DomainError::new(
                ErrorCode::ValidationFailed,
                "Can only upgrade to a higher tier",
            ));
        }

        let old_tier = self.tier;
        self.tier = new_tier;
        self.billing_period = new_tier.billing_period();
        self.updated_at = Timestamp::now();

        self.record_event(MembershipEvent::TierChanged {
            membership_id: self.id,
            old_tier,
            new_tier,
            changed_at: self.updated_at,
        });

        Ok(())
    }

    // === Domain Events ===

    pub fn pull_domain_events(&mut self) -> Vec<MembershipEvent> {
        std::mem::take(&mut self.domain_events)
    }

    fn record_event(&mut self, event: MembershipEvent) {
        self.domain_events.push(event);
    }
}
```

### Domain Events

```rust
use crate::foundation::{MembershipId, Timestamp, UserId};
use serde::{Deserialize, Serialize};

/// Events emitted by the Membership aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MembershipEvent {
    Created {
        membership_id: MembershipId,
        user_id: UserId,
        tier: MembershipTier,
        promo_code: Option<String>,
        created_at: Timestamp,
    },
    Activated {
        membership_id: MembershipId,
        activated_at: Timestamp,
        period_end: Timestamp,
    },
    Cancelled {
        membership_id: MembershipId,
        cancelled_at: Timestamp,
        effective_end: Timestamp,
    },
    Expired {
        membership_id: MembershipId,
        expired_at: Timestamp,
    },
    Renewed {
        membership_id: MembershipId,
        renewed_at: Timestamp,
        new_period_end: Timestamp,
    },
    PaymentReceived {
        membership_id: MembershipId,
        received_at: Timestamp,
    },
    PaymentFailed {
        membership_id: MembershipId,
        failed_at: Timestamp,
    },
    TierChanged {
        membership_id: MembershipId,
        old_tier: MembershipTier,
        new_tier: MembershipTier,
        changed_at: Timestamp,
    },
}

impl MembershipEvent {
    pub fn membership_id(&self) -> MembershipId {
        match self {
            MembershipEvent::Created { membership_id, .. } => *membership_id,
            MembershipEvent::Activated { membership_id, .. } => *membership_id,
            MembershipEvent::Cancelled { membership_id, .. } => *membership_id,
            MembershipEvent::Expired { membership_id, .. } => *membership_id,
            MembershipEvent::Renewed { membership_id, .. } => *membership_id,
            MembershipEvent::PaymentReceived { membership_id, .. } => *membership_id,
            MembershipEvent::PaymentFailed { membership_id, .. } => *membership_id,
            MembershipEvent::TierChanged { membership_id, .. } => *membership_id,
        }
    }

    pub fn event_type(&self) -> &'static str {
        match self {
            MembershipEvent::Created { .. } => "membership.created",
            MembershipEvent::Activated { .. } => "membership.activated",
            MembershipEvent::Cancelled { .. } => "membership.cancelled",
            MembershipEvent::Expired { .. } => "membership.expired",
            MembershipEvent::Renewed { .. } => "membership.renewed",
            MembershipEvent::PaymentReceived { .. } => "membership.payment_received",
            MembershipEvent::PaymentFailed { .. } => "membership.payment_failed",
            MembershipEvent::TierChanged { .. } => "membership.tier_changed",
        }
    }
}
```

---

## Ports

### MembershipRepository (Write)

```rust
use async_trait::async_trait;
use crate::foundation::{MembershipId, UserId};

/// Repository port for Membership aggregate persistence (write side)
#[async_trait]
pub trait MembershipRepository: Send + Sync {
    /// Persists a new membership
    async fn save(&self, membership: &Membership) -> Result<(), RepositoryError>;

    /// Updates an existing membership
    async fn update(&self, membership: &Membership) -> Result<(), RepositoryError>;

    /// Finds a membership by ID
    async fn find_by_id(&self, id: MembershipId) -> Result<Option<Membership>, RepositoryError>;

    /// Finds a membership by user ID (each user has at most one membership)
    async fn find_by_user(&self, user_id: &UserId) -> Result<Option<Membership>, RepositoryError>;

    /// Finds memberships expiring within N days (for renewal reminders)
    async fn find_expiring_within_days(&self, days: u32) -> Result<Vec<Membership>, RepositoryError>;
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Membership not found: {0}")]
    NotFound(MembershipId),

    #[error("User already has a membership: {0}")]
    DuplicateUser(UserId),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

### MembershipReader (Query - CQRS)

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::foundation::{MembershipId, UserId};

/// Read-only port for membership queries (CQRS query side)
#[async_trait]
pub trait MembershipReader: Send + Sync {
    /// Gets a membership view by user ID
    async fn get_by_user(&self, user_id: &UserId) -> Result<Option<MembershipView>, ReaderError>;

    /// Checks if a user has access (quick check, returns bool)
    async fn check_access(&self, user_id: &UserId) -> Result<bool, ReaderError>;

    /// Gets the user's current tier
    async fn get_tier(&self, user_id: &UserId) -> Result<Option<MembershipTier>, ReaderError>;

    /// Lists memberships expiring within N days (for admin/notifications)
    async fn list_expiring(&self, days: u32) -> Result<Vec<MembershipSummary>, ReaderError>;

    /// Gets membership statistics (for admin dashboard)
    async fn get_statistics(&self) -> Result<MembershipStatistics, ReaderError>;
}

/// Detailed membership view
#[derive(Debug, Clone, serde::Serialize)]
pub struct MembershipView {
    pub id: MembershipId,
    pub tier: MembershipTier,
    pub status: MembershipStatus,
    pub billing_period: BillingPeriod,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub has_access: bool,
    pub days_remaining: i64,
    pub promo_code: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Summary for lists
#[derive(Debug, Clone, serde::Serialize)]
pub struct MembershipSummary {
    pub id: MembershipId,
    pub user_id: String,
    pub tier: MembershipTier,
    pub status: MembershipStatus,
    pub current_period_end: DateTime<Utc>,
}

/// Statistics for admin
#[derive(Debug, Clone, serde::Serialize)]
pub struct MembershipStatistics {
    pub total_memberships: u64,
    pub active_memberships: u64,
    pub by_tier: Vec<TierCount>,
    pub by_status: Vec<StatusCount>,
    pub monthly_recurring_revenue_cents: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TierCount {
    pub tier: MembershipTier,
    pub count: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StatusCount {
    pub status: MembershipStatus,
    pub count: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum ReaderError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

### AccessChecker (Used by Other Modules)

```rust
use async_trait::async_trait;
use crate::foundation::UserId;

/// Port for other modules to check membership access
/// This is the integration point with the session module
///
/// SECURITY: All methods follow fail-secure patterns:
/// - can_create_session returns Result<(), AccessDenied> to enforce explicit authorization
/// - get_limits returns no_access() limits on any error (fail-secure default)
/// - Callers must handle errors as denials, not silent failures
#[async_trait]
pub trait AccessChecker: Send + Sync {
    /// Checks if a user can create new sessions
    /// SECURITY: Returns Ok(()) if allowed, Err(AccessDenied) otherwise
    /// This forces callers to explicitly handle the denial case
    async fn can_create_session(&self, user_id: &UserId) -> Result<(), AccessDenied>;

    /// Gets the user's membership tier (None if no membership)
    async fn get_tier(&self, user_id: &UserId) -> Result<Option<MembershipTier>, AccessError>;

    /// Gets feature limits for the user's tier
    /// SECURITY: Returns no_access() limits on any error (fail-secure)
    async fn get_limits(&self, user_id: &UserId) -> TierLimits;
}

/// Error returned when access is denied
#[derive(Debug, thiserror::Error)]
#[error("Access denied: {reason}")]
pub struct AccessDenied {
    pub reason: String,
}

impl AccessDenied {
    pub fn no_membership() -> Self {
        Self { reason: "No active membership".into() }
    }

    pub fn expired() -> Self {
        Self { reason: "Membership expired".into() }
    }

    pub fn limit_reached() -> Self {
        Self { reason: "Feature limit reached".into() }
    }
}

/// Feature limits per tier
#[derive(Debug, Clone, serde::Serialize)]
pub struct TierLimits {
    /// Maximum number of active sessions
    pub max_active_sessions: Option<u32>,
    /// Maximum number of cycles per session
    pub max_cycles_per_session: Option<u32>,
    /// Can use AI features
    pub ai_features_enabled: bool,
    /// Can export to PDF
    pub export_enabled: bool,
}

impl TierLimits {
    pub fn for_tier(tier: MembershipTier) -> Self {
        match tier {
            MembershipTier::Free => Self {
                max_active_sessions: Some(3),
                max_cycles_per_session: Some(2),
                ai_features_enabled: true,
                export_enabled: false,
            },
            MembershipTier::Monthly => Self {
                max_active_sessions: Some(10),
                max_cycles_per_session: Some(5),
                ai_features_enabled: true,
                export_enabled: true,
            },
            MembershipTier::Annual => Self {
                max_active_sessions: None, // Unlimited
                max_cycles_per_session: None,
                ai_features_enabled: true,
                export_enabled: true,
            },
        }
    }

    /// Returns default limits for users without membership
    /// SECURITY: This is the fail-secure default - no access to anything
    pub fn no_access() -> Self {
        Self {
            max_active_sessions: Some(0),
            max_cycles_per_session: Some(0),
            ai_features_enabled: false,
            export_enabled: false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AccessError {
    #[error("Database error: {0}")]
    Database(String),
}

/// SECURITY: Example fail-secure implementation of AccessChecker
/// On any error, returns denial or no_access() limits
impl AccessChecker for AccessCheckerImpl {
    async fn can_create_session(&self, user_id: &UserId) -> Result<(), AccessDenied> {
        // FAIL SECURE: On any error, deny access
        let membership = self.membership_reader
            .get_by_user(user_id)
            .await
            .map_err(|_| AccessDenied::no_membership())?;

        match membership {
            Some(m) if m.has_access => Ok(()),
            Some(_) => Err(AccessDenied::expired()),
            None => Err(AccessDenied::no_membership()),
        }
    }

    async fn get_limits(&self, user_id: &UserId) -> TierLimits {
        // FAIL SECURE: On any error, return no_access() limits
        match self.membership_reader.get_by_user(user_id).await {
            Ok(Some(m)) if m.has_access => TierLimits::for_tier(m.tier),
            _ => TierLimits::no_access(), // FAIL SECURE: deny on any error or missing membership
        }
    }
}
```

### PaymentProvider (External - Stripe)

```rust
use async_trait::async_trait;
use crate::foundation::UserId;

/// Port for payment provider integration
#[async_trait]
pub trait PaymentProvider: Send + Sync {
    /// Creates a customer in the payment provider
    async fn create_customer(
        &self,
        user_id: &UserId,
        email: &str,
        name: Option<&str>,
    ) -> Result<CreateCustomerResult, PaymentError>;

    /// Creates a checkout session for subscription signup
    async fn create_checkout_session(
        &self,
        customer_id: &str,
        price_id: &str,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CreateCheckoutResult, PaymentError>;

    /// Creates a customer portal session for managing subscription
    async fn create_portal_session(
        &self,
        customer_id: &str,
        return_url: &str,
    ) -> Result<CreatePortalResult, PaymentError>;

    /// Cancels a subscription
    async fn cancel_subscription(
        &self,
        subscription_id: &str,
        cancel_at_period_end: bool,
    ) -> Result<(), PaymentError>;

    /// Retrieves subscription details
    async fn get_subscription(
        &self,
        subscription_id: &str,
    ) -> Result<SubscriptionDetails, PaymentError>;

    /// Verifies a webhook signature
    fn verify_webhook_signature(
        &self,
        payload: &[u8],
        signature: &str,
    ) -> Result<(), PaymentError>;
}

#[derive(Debug, Clone)]
pub struct CreateCustomerResult {
    pub customer_id: String,
}

#[derive(Debug, Clone)]
pub struct CreateCheckoutResult {
    pub session_id: String,
    pub checkout_url: String,
}

#[derive(Debug, Clone)]
pub struct CreatePortalResult {
    pub portal_url: String,
}

#[derive(Debug, Clone)]
pub struct SubscriptionDetails {
    pub subscription_id: String,
    pub customer_id: String,
    pub status: String,
    pub current_period_start: i64, // Unix timestamp
    pub current_period_end: i64,
    pub cancel_at_period_end: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("Payment provider error: {0}")]
    ProviderError(String),

    #[error("Invalid webhook signature")]
    InvalidSignature,

    #[error("Customer not found: {0}")]
    CustomerNotFound(String),

    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(String),
}
```

---

## Application Layer

### Commands

#### CreateFreeMembership

```rust
#[derive(Debug, Clone)]
pub struct CreateFreeMembershipCommand {
    pub user_id: UserId,
    pub promo_code: String,
}

pub struct CreateFreeMembershipHandler {
    repo: Arc<dyn MembershipRepository>,
    promo_validator: Arc<dyn PromoCodeValidator>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl CreateFreeMembershipHandler {
    pub async fn handle(&self, cmd: CreateFreeMembershipCommand) -> Result<MembershipId, CommandError> {
        // Check user doesn't already have a membership
        if self.repo.find_by_user(&cmd.user_id).await?.is_some() {
            return Err(CommandError::AlreadyExists);
        }

        // Validate promo code
        let promo = PromoCode::new(&cmd.promo_code)?;
        self.promo_validator.validate(&promo).await?;

        // Create membership
        let mut membership = Membership::create_free(cmd.user_id, promo)?;

        // Persist
        self.repo.save(&membership).await?;

        // Publish events
        let events = membership.pull_domain_events();
        self.publisher.publish(events).await?;

        Ok(membership.id())
    }
}
```

#### CreatePaidMembership

```rust
#[derive(Debug, Clone)]
pub struct CreatePaidMembershipCommand {
    pub user_id: UserId,
    pub tier: MembershipTier,
    pub email: String,
    pub success_url: String,
    pub cancel_url: String,
}

pub struct CreatePaidMembershipHandler {
    repo: Arc<dyn MembershipRepository>,
    payment: Arc<dyn PaymentProvider>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl CreatePaidMembershipHandler {
    pub async fn handle(&self, cmd: CreatePaidMembershipCommand) -> Result<CheckoutResult, CommandError> {
        // Check for existing membership
        if let Some(existing) = self.repo.find_by_user(&cmd.user_id).await? {
            if existing.has_access() {
                return Err(CommandError::AlreadyExists);
            }
        }

        // Get price ID for tier
        let prices = PlanPrice::get_standard_prices();
        let price = prices.iter()
            .find(|p| p.tier == cmd.tier)
            .ok_or(CommandError::InvalidTier)?;

        let stripe_price_id = price.stripe_price_id.as_ref()
            .ok_or(CommandError::InvalidTier)?;

        // Create customer in Stripe
        let customer = self.payment.create_customer(
            &cmd.user_id,
            &cmd.email,
            None,
        ).await?;

        // Create membership (pending)
        let mut membership = Membership::create_paid(cmd.user_id, cmd.tier)?;
        membership.set_external_ids(customer.customer_id.clone(), None)?;

        // Persist
        self.repo.save(&membership).await?;

        // Create checkout session
        let checkout = self.payment.create_checkout_session(
            &customer.customer_id,
            stripe_price_id,
            &cmd.success_url,
            &cmd.cancel_url,
        ).await?;

        // Publish events
        let events = membership.pull_domain_events();
        self.publisher.publish(events).await?;

        Ok(CheckoutResult {
            membership_id: membership.id(),
            checkout_url: checkout.checkout_url,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CheckoutResult {
    pub membership_id: MembershipId,
    pub checkout_url: String,
}
```

#### HandlePaymentWebhook

```rust
#[derive(Debug, Clone)]
pub struct HandlePaymentWebhookCommand {
    pub payload: Vec<u8>,
    pub signature: String,
}

pub struct HandlePaymentWebhookHandler {
    repo: Arc<dyn MembershipRepository>,
    payment: Arc<dyn PaymentProvider>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl HandlePaymentWebhookHandler {
    pub async fn handle(&self, cmd: HandlePaymentWebhookCommand) -> Result<(), CommandError> {
        // Verify webhook signature
        self.payment.verify_webhook_signature(&cmd.payload, &cmd.signature)?;

        // Parse webhook event
        let event: WebhookEvent = serde_json::from_slice(&cmd.payload)?;

        match event.event_type.as_str() {
            "customer.subscription.created" |
            "customer.subscription.updated" => {
                self.handle_subscription_update(&event).await?;
            }
            "customer.subscription.deleted" => {
                self.handle_subscription_deleted(&event).await?;
            }
            "invoice.payment_succeeded" => {
                self.handle_payment_succeeded(&event).await?;
            }
            "invoice.payment_failed" => {
                self.handle_payment_failed(&event).await?;
            }
            _ => {
                // Ignore unknown events
            }
        }

        Ok(())
    }

    async fn handle_subscription_update(&self, event: &WebhookEvent) -> Result<(), CommandError> {
        let sub = &event.data.subscription;

        // Find membership by external subscription ID
        // (Would need a find_by_external_subscription method)
        // For now, assume we store customer_id -> user_id mapping

        // Update membership status based on subscription status
        // ...

        Ok(())
    }

    async fn handle_payment_succeeded(&self, event: &WebhookEvent) -> Result<(), CommandError> {
        // Find membership and activate/renew
        Ok(())
    }

    async fn handle_payment_failed(&self, event: &WebhookEvent) -> Result<(), CommandError> {
        // Find membership and mark as past due
        Ok(())
    }

    async fn handle_subscription_deleted(&self, event: &WebhookEvent) -> Result<(), CommandError> {
        // Find membership and expire
        Ok(())
    }
}
```

#### CancelMembership

```rust
#[derive(Debug, Clone)]
pub struct CancelMembershipCommand {
    pub user_id: UserId,
}

pub struct CancelMembershipHandler {
    repo: Arc<dyn MembershipRepository>,
    payment: Arc<dyn PaymentProvider>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl CancelMembershipHandler {
    pub async fn handle(&self, cmd: CancelMembershipCommand) -> Result<(), CommandError> {
        // Find membership
        let mut membership = self.repo
            .find_by_user(&cmd.user_id)
            .await?
            .ok_or(CommandError::NotFound)?;

        // Cancel in Stripe (if paid subscription)
        if let Some(sub_id) = membership.external_subscription_id() {
            self.payment.cancel_subscription(sub_id, true).await?;
        }

        // Cancel in domain
        membership.cancel()?;

        // Persist
        self.repo.update(&membership).await?;

        // Publish events
        let events = membership.pull_domain_events();
        self.publisher.publish(events).await?;

        Ok(())
    }
}
```

### Queries

#### GetMembership

```rust
#[derive(Debug, Clone)]
pub struct GetMembershipQuery {
    pub user_id: UserId,
}

pub struct GetMembershipHandler {
    reader: Arc<dyn MembershipReader>,
}

impl GetMembershipHandler {
    pub async fn handle(&self, query: GetMembershipQuery) -> Result<Option<MembershipView>, QueryError> {
        self.reader.get_by_user(&query.user_id).await
            .map_err(QueryError::from)
    }
}
```

#### CheckAccess

```rust
#[derive(Debug, Clone)]
pub struct CheckAccessQuery {
    pub user_id: UserId,
}

pub struct CheckAccessHandler {
    reader: Arc<dyn MembershipReader>,
}

impl CheckAccessHandler {
    pub async fn handle(&self, query: CheckAccessQuery) -> Result<bool, QueryError> {
        self.reader.check_access(&query.user_id).await
            .map_err(QueryError::from)
    }
}
```

---

## Adapters

### HTTP Endpoints

| Method | Path | Handler | Auth | Description |
|--------|------|---------|------|-------------|
| `GET` | `/api/membership` | GetMembership | Required | Get current user's membership |
| `POST` | `/api/membership/free` | CreateFreeMembership | Required | Create free membership with promo |
| `POST` | `/api/membership/checkout` | CreatePaidMembership | Required | Create checkout session |
| `POST` | `/api/membership/cancel` | CancelMembership | Required | Cancel membership |
| `GET` | `/api/membership/portal` | GetPortalUrl | Required | Get Stripe portal URL |
| `POST` | `/api/webhooks/stripe` | HandlePaymentWebhook | Signature | Handle Stripe webhooks |
| `GET` | `/api/membership/prices` | GetPrices | Public | Get available plans/prices |

#### Request/Response DTOs

```rust
use serde::{Deserialize, Serialize};

// === Requests ===

#[derive(Debug, Deserialize)]
pub struct CreateFreeMembershipRequest {
    pub promo_code: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePaidMembershipRequest {
    pub tier: String, // "monthly" or "annual"
    pub email: String,
    pub success_url: String,
    pub cancel_url: String,
}

// === Responses ===

#[derive(Debug, Serialize)]
pub struct MembershipResponse {
    pub id: String,
    pub tier: String,
    pub status: String,
    pub billing_period: String,
    pub current_period_start: String,
    pub current_period_end: String,
    pub has_access: bool,
    pub days_remaining: i64,
    pub promo_code: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub membership_id: String,
    pub checkout_url: String,
}

#[derive(Debug, Serialize)]
pub struct PortalResponse {
    pub portal_url: String,
}

#[derive(Debug, Serialize)]
pub struct PricesResponse {
    pub prices: Vec<PriceResponse>,
}

#[derive(Debug, Serialize)]
pub struct PriceResponse {
    pub tier: String,
    pub name: String,
    pub price_cents: i64,
    pub currency: String,
    pub billing_period: String,
    pub monthly_equivalent_cents: i64,
}
```

### Database Schema

```sql
-- Membership table
CREATE TABLE memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL UNIQUE,
    tier VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    billing_period VARCHAR(20) NOT NULL,
    current_period_start TIMESTAMPTZ NOT NULL,
    current_period_end TIMESTAMPTZ NOT NULL,
    promo_code VARCHAR(50),
    external_customer_id VARCHAR(255),
    external_subscription_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    cancelled_at TIMESTAMPTZ,

    CONSTRAINT memberships_tier_valid CHECK (tier IN ('free', 'monthly', 'annual')),
    CONSTRAINT memberships_status_valid CHECK (
        status IN ('pending', 'active', 'past_due', 'cancelled', 'expired', 'trialing')
    ),
    CONSTRAINT memberships_billing_period_valid CHECK (
        billing_period IN ('monthly', 'annual')
    )
);

-- Indexes
CREATE INDEX idx_memberships_user_id ON memberships(user_id);
CREATE INDEX idx_memberships_status ON memberships(status);
CREATE INDEX idx_memberships_tier ON memberships(tier);
CREATE INDEX idx_memberships_period_end ON memberships(current_period_end);
CREATE INDEX idx_memberships_external_customer ON memberships(external_customer_id);
CREATE INDEX idx_memberships_external_subscription ON memberships(external_subscription_id);

-- Promo codes table (for validation)
CREATE TABLE promo_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) NOT NULL UNIQUE,
    promo_type VARCHAR(50) NOT NULL,
    max_uses INT,
    current_uses INT NOT NULL DEFAULT 0,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT promo_codes_type_valid CHECK (
        promo_type IN ('workshop', 'beta_user', 'partner', 'general')
    )
);

CREATE INDEX idx_promo_codes_code ON promo_codes(code);
```

### Stripe Adapter

```rust
use stripe::{Client, Customer, CheckoutSession, BillingPortalSession, Subscription};
use async_trait::async_trait;

pub struct StripePaymentAdapter {
    client: Client,
    webhook_secret: String,
}

impl StripePaymentAdapter {
    pub fn new(api_key: &str, webhook_secret: String) -> Self {
        Self {
            client: Client::new(api_key),
            webhook_secret,
        }
    }
}

#[async_trait]
impl PaymentProvider for StripePaymentAdapter {
    async fn create_customer(
        &self,
        user_id: &UserId,
        email: &str,
        name: Option<&str>,
    ) -> Result<CreateCustomerResult, PaymentError> {
        let mut params = stripe::CreateCustomer::new();
        params.email = Some(email);
        params.name = name;
        params.metadata = Some([
            ("user_id".to_string(), user_id.to_string()),
        ].into());

        let customer = Customer::create(&self.client, params).await
            .map_err(|e| PaymentError::ProviderError(e.to_string()))?;

        Ok(CreateCustomerResult {
            customer_id: customer.id.to_string(),
        })
    }

    async fn create_checkout_session(
        &self,
        customer_id: &str,
        price_id: &str,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CreateCheckoutResult, PaymentError> {
        let params = stripe::CreateCheckoutSession {
            customer: Some(customer_id.parse().unwrap()),
            mode: Some(stripe::CheckoutSessionMode::Subscription),
            line_items: Some(vec![
                stripe::CreateCheckoutSessionLineItems {
                    price: Some(price_id.to_string()),
                    quantity: Some(1),
                    ..Default::default()
                }
            ]),
            success_url: Some(success_url),
            cancel_url: Some(cancel_url),
            ..Default::default()
        };

        let session = CheckoutSession::create(&self.client, params).await
            .map_err(|e| PaymentError::ProviderError(e.to_string()))?;

        Ok(CreateCheckoutResult {
            session_id: session.id.to_string(),
            checkout_url: session.url.unwrap_or_default(),
        })
    }

    async fn create_portal_session(
        &self,
        customer_id: &str,
        return_url: &str,
    ) -> Result<CreatePortalResult, PaymentError> {
        let params = stripe::CreateBillingPortalSession {
            customer: customer_id.parse().unwrap(),
            return_url: Some(return_url),
            ..Default::default()
        };

        let session = BillingPortalSession::create(&self.client, params).await
            .map_err(|e| PaymentError::ProviderError(e.to_string()))?;

        Ok(CreatePortalResult {
            portal_url: session.url,
        })
    }

    async fn cancel_subscription(
        &self,
        subscription_id: &str,
        cancel_at_period_end: bool,
    ) -> Result<(), PaymentError> {
        if cancel_at_period_end {
            let params = stripe::UpdateSubscription {
                cancel_at_period_end: Some(true),
                ..Default::default()
            };
            Subscription::update(&self.client, &subscription_id.parse().unwrap(), params).await
                .map_err(|e| PaymentError::ProviderError(e.to_string()))?;
        } else {
            Subscription::cancel(&self.client, &subscription_id.parse().unwrap(), stripe::CancelSubscription::default()).await
                .map_err(|e| PaymentError::ProviderError(e.to_string()))?;
        }
        Ok(())
    }

    async fn get_subscription(
        &self,
        subscription_id: &str,
    ) -> Result<SubscriptionDetails, PaymentError> {
        let sub = Subscription::retrieve(&self.client, &subscription_id.parse().unwrap(), &[]).await
            .map_err(|e| PaymentError::ProviderError(e.to_string()))?;

        Ok(SubscriptionDetails {
            subscription_id: sub.id.to_string(),
            customer_id: sub.customer.id().to_string(),
            status: sub.status.to_string(),
            current_period_start: sub.current_period_start,
            current_period_end: sub.current_period_end,
            cancel_at_period_end: sub.cancel_at_period_end,
        })
    }

    fn verify_webhook_signature(
        &self,
        payload: &[u8],
        signature: &str,
    ) -> Result<(), PaymentError> {
        stripe::Webhook::construct_event(
            std::str::from_utf8(payload).unwrap(),
            signature,
            &self.webhook_secret,
        ).map_err(|_| PaymentError::InvalidSignature)?;
        Ok(())
    }
}
```

---

## Integration with Session Module

The session module needs to check membership access before creating sessions. This is done via the `AccessChecker` port.

### Modification to CreateSession Command

```rust
// In session module's CreateSession handler:

pub struct CreateSessionHandler {
    repo: Arc<dyn SessionRepository>,
    access_checker: Arc<dyn AccessChecker>, // NEW: Membership check
    publisher: Arc<dyn DomainEventPublisher>,
}

impl CreateSessionHandler {
    pub async fn handle(&self, cmd: CreateSessionCommand) -> Result<SessionId, CommandError> {
        // NEW: Check membership access
        if !self.access_checker.can_create_session(&cmd.user_id).await? {
            return Err(CommandError::MembershipRequired);
        }

        // Optional: Check session limits
        let limits = self.access_checker.get_limits(&cmd.user_id).await?;
        if let Some(max) = limits.max_active_sessions {
            let current_count = self.count_active_sessions(&cmd.user_id).await?;
            if current_count >= max as usize {
                return Err(CommandError::SessionLimitReached);
            }
        }

        // ... rest of existing logic ...
    }
}
```

---

## File Structure

```
backend/src/domain/membership/
├── mod.rs                    # Module exports
├── membership.rs             # Membership aggregate
├── membership_test.rs        # Aggregate tests
├── value_objects/
│   ├── mod.rs
│   ├── money.rs              # Money value object (CENTS!)
│   ├── money_test.rs
│   ├── tier.rs               # MembershipTier
│   ├── tier_test.rs
│   ├── status.rs             # MembershipStatus
│   ├── status_test.rs
│   ├── billing_period.rs
│   ├── promo_code.rs
│   └── plan_price.rs
├── events.rs                 # MembershipEvent enum
└── errors.rs                 # Membership-specific errors

backend/src/ports/
├── membership_repository.rs  # MembershipRepository trait
├── membership_reader.rs      # MembershipReader trait
├── access_checker.rs         # AccessChecker trait
└── payment_provider.rs       # PaymentProvider trait

backend/src/application/
├── commands/
│   ├── create_free_membership.rs
│   ├── create_free_membership_test.rs
│   ├── create_paid_membership.rs
│   ├── create_paid_membership_test.rs
│   ├── cancel_membership.rs
│   ├── cancel_membership_test.rs
│   ├── handle_payment_webhook.rs
│   └── handle_payment_webhook_test.rs
└── queries/
    ├── get_membership.rs
    ├── get_membership_test.rs
    ├── check_access.rs
    └── get_prices.rs

backend/src/adapters/
├── http/membership/
│   ├── mod.rs
│   ├── handlers.rs
│   ├── handlers_test.rs
│   ├── dto.rs
│   └── routes.rs
├── postgres/
│   ├── membership_repository.rs
│   ├── membership_repository_test.rs
│   ├── membership_reader.rs
│   ├── membership_reader_test.rs
│   └── access_checker_impl.rs
└── stripe/
    ├── mod.rs
    ├── stripe_adapter.rs
    ├── stripe_adapter_test.rs
    ├── mock_payment_provider.rs
    └── webhook_types.rs

backend/migrations/
├── XXX_create_memberships.sql
└── XXX_create_promo_codes.sql

frontend/src/modules/membership/
├── domain/
│   ├── membership.ts
│   ├── membership.test.ts
│   ├── money.ts              # Money type with cents
│   └── tier.ts
├── api/
│   ├── membership-api.ts
│   ├── use-membership.ts
│   └── use-prices.ts
├── components/
│   ├── MembershipBadge.svelte
│   ├── MembershipBadge.test.ts
│   ├── PricingTable.svelte
│   ├── PricingTable.test.ts
│   ├── CheckoutButton.svelte
│   ├── PromoCodeInput.svelte
│   ├── MembershipStatus.svelte
│   └── UpgradePrompt.svelte
├── pages/
│   ├── pricing/+page.svelte
│   └── account/+page.svelte
└── index.ts
```

---

## Invariants

| Invariant | Enforcement |
|-----------|-------------|
| Money amounts are always in cents (integers) | Money value object uses i64 |
| Each user has at most one membership | Unique constraint on user_id |
| Free tier requires valid promo code | Validation in create_free |
| Paid tier requires checkout | Pending status until payment |
| Only owner can cancel | Authorization check |
| Status transitions follow rules | can_transition_to() check |
| Expired memberships lose access | has_access() checks period end |

---

## Test Categories

### Unit Tests (Domain)

| Category | Example Tests |
|----------|---------------|
| Money | `money_from_cents_preserves_value` |
| Money | `money_from_dollars_converts_to_cents` |
| Money | `money_cannot_add_different_currencies` |
| Membership | `create_free_requires_promo_code` |
| Membership | `create_paid_starts_as_pending` |
| Membership | `has_access_false_after_expiry` |
| Status | `can_transition_from_pending_to_active` |

### Integration Tests (Repository)

| Category | Example Tests |
|----------|---------------|
| Save | `save_persists_membership_to_database` |
| Find | `find_by_user_returns_membership` |
| Unique | `save_rejects_duplicate_user` |

### API Tests (HTTP)

| Category | Example Tests |
|----------|---------------|
| Checkout | `post_checkout_creates_session` |
| Webhook | `webhook_verifies_signature` |
| Access | `get_membership_returns_current_status` |

---

*Module Version: 1.0.0*
*Based on: SYSTEM-ARCHITECTURE.md v1.1.0*
*Language: Rust*
