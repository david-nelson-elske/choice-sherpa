# Integration: Membership Access Control

**Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md
**Type:** Cross-Module + External Service
**Priority:** P0 (Required for monetization)
**Depends On:** foundation module (Phase 1)

> Membership-based access control and feature gating across all modules, with Stripe payment integration.

---

## Overview

The Membership Access Control integration connects the membership module to other bounded contexts, enabling tier-based access control, usage limits, and external payment processing. This is the monetization backbone of Choice Sherpa.

### Key Integration Points

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           External: Stripe                                   │
│   Checkout Sessions │ Subscription Webhooks │ Customer Portal               │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ HTTPS webhooks
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    PaymentProvider Port (Adapter)                            │
│   - handle_webhook()                                                         │
│   - create_checkout_session()                                                │
│   - create_portal_session()                                                  │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ domain events
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Membership Module                                    │
│   Membership Aggregate │ Tier Limits │ Access Rules                          │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ AccessChecker port
                                    ▼
┌───────────────────┬───────────────────┬───────────────────┬─────────────────┐
│     Session       │       Cycle       │    Dashboard      │   Conversation  │
│  (create check)   │  (limit check)    │  (display tier)   │  (export check) │
└───────────────────┴───────────────────┴───────────────────┴─────────────────┘
```

---

## Modules Involved

| Module | Role | Changes Required |
|--------|------|------------------|
| `membership` | Owner | Full implementation (see module spec) |
| `session` | Consumer | Call AccessChecker before session creation |
| `cycle` | Consumer | Call AccessChecker before cycle creation |
| `conversation` | Consumer | Check export capability before export |
| `dashboard` | Consumer | Display membership tier, status, limits |
| `adapters/stripe` | Producer | Webhook handling, checkout creation |
| `frontend` | Both | Pricing page, account page, access denied UI |

---

## Data Flow

### Access Check Flow (Session Creation)

```
User                   Session Module           AccessChecker           Membership
  │                         │                        │                      │
  │── CreateSession ───────►│                        │                      │
  │                         │                        │                      │
  │                         │── can_create_session ─►│                      │
  │                         │      (user_id)         │                      │
  │                         │                        │── get membership ───►│
  │                         │                        │                      │
  │                         │                        │◄── Membership ───────│
  │                         │                        │                      │
  │                         │                        │── check tier limits ─┤
  │                         │                        │                      │
  │                         │◄─ AccessResult ────────│                      │
  │                         │   (allowed/denied)     │                      │
  │                         │                        │                      │
  │◄── Result ──────────────│                        │                      │
  │   (session or error)    │                        │                      │
```

### Payment Webhook Flow (Subscription Created)

```
Stripe                 Webhook Handler         Membership              Event Bus
  │                         │                      │                      │
  │── POST /webhooks/stripe ►                      │                      │
  │   customer.subscription │                      │                      │
  │   .created              │                      │                      │
  │                         │                      │                      │
  │                         │── verify signature ──┤                      │
  │                         │                      │                      │
  │                         │── parse event ───────┤                      │
  │                         │                      │                      │
  │                         │── activate_paid ────►│                      │
  │                         │   (user_id, tier,    │                      │
  │                         │    stripe_sub_id)    │                      │
  │                         │                      │                      │
  │                         │                      │── MembershipActivated ►
  │                         │                      │                      │
  │◄── 200 OK ──────────────│                      │                      │
```

### Promo Code Flow (Workshop User)

```
User                   Membership API          Membership              Event Bus
  │                         │                      │                      │
  │── POST /memberships     │                      │                      │
  │   { promo_code:         │                      │                      │
  │     "WORKSHOP2026-ABC"} │                      │                      │
  │                         │                      │                      │
  │                         │── validate_promo ────┤                      │
  │                         │                      │                      │
  │                         │── create_free ──────►│                      │
  │                         │   (user_id, Annual,  │                      │
  │                         │    promo_code)       │                      │
  │                         │                      │                      │
  │                         │                      │── MembershipCreated ──►
  │                         │                      │                      │
  │◄── 201 Created ─────────│                      │                      │
  │   { tier: Annual,       │                      │                      │
  │     status: Active }    │                      │                      │
```

---

## AccessChecker Port Contract

The `AccessChecker` port is the primary integration point for other modules.

```rust
// backend/src/ports/access_checker.rs

/// Port for checking user access based on membership
#[async_trait]
pub trait AccessChecker: Send + Sync {
    /// Check if user can create a new session
    async fn can_create_session(&self, user_id: &UserId) -> Result<AccessResult, DomainError>;

    /// Check if user can create a new cycle in session
    async fn can_create_cycle(
        &self,
        user_id: &UserId,
        session_id: &SessionId
    ) -> Result<AccessResult, DomainError>;

    /// Check if user can export data (PDF, CSV)
    async fn can_export(&self, user_id: &UserId) -> Result<AccessResult, DomainError>;

    /// Get user's current tier limits
    async fn get_tier_limits(&self, user_id: &UserId) -> Result<TierLimits, DomainError>;

    /// Get user's current usage
    async fn get_usage(&self, user_id: &UserId) -> Result<UsageStats, DomainError>;
}

/// Result of an access check
#[derive(Debug, Clone)]
pub enum AccessResult {
    Allowed,
    Denied(AccessDeniedReason),
}

#[derive(Debug, Clone)]
pub enum AccessDeniedReason {
    NoMembership,
    MembershipExpired,
    MembershipPastDue,
    SessionLimitReached { current: u32, max: u32 },
    CycleLimitReached { current: u32, max: u32 },
    FeatureNotIncluded { feature: String, required_tier: MembershipTier },
}

/// Limits for a membership tier
#[derive(Debug, Clone)]
pub struct TierLimits {
    pub tier: MembershipTier,
    pub max_sessions: Option<u32>,      // None = unlimited
    pub max_cycles_per_session: Option<u32>,
    pub export_enabled: bool,
    pub api_access: bool,
}

/// Current usage statistics
#[derive(Debug, Clone)]
pub struct UsageStats {
    pub active_sessions: u32,
    pub total_cycles: u32,
    pub exports_this_month: u32,
}
```

---

## Tier Limits Configuration

| Tier | Max Sessions | Max Cycles/Session | Export | API Access |
|------|-------------|-------------------|--------|------------|
| Free | 3 | 5 | No | No |
| Monthly | 10 | 20 | Yes | No |
| Annual | Unlimited | Unlimited | Yes | Yes |

```rust
// backend/src/domain/membership/tier_limits.rs

impl TierLimits {
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
                max_sessions: None,  // Unlimited
                max_cycles_per_session: None,
                export_enabled: true,
                api_access: true,
            },
        }
    }
}
```

---

## Module Integration Examples

### Session Module Integration

```rust
// backend/src/application/session/commands/create_session.rs

pub struct CreateSessionHandler {
    session_repo: Arc<dyn SessionRepository>,
    access_checker: Arc<dyn AccessChecker>,  // Injected dependency
    event_bus: Arc<dyn EventPublisher>,
}

impl CreateSessionHandler {
    pub async fn handle(&self, cmd: CreateSession) -> Result<Session, DomainError> {
        // 1. Check access FIRST
        let access = self.access_checker.can_create_session(&cmd.user_id).await?;

        match access {
            AccessResult::Denied(reason) => {
                return Err(DomainError::AccessDenied(reason));
            }
            AccessResult::Allowed => {}
        }

        // 2. Create session (existing logic)
        let session = Session::create(cmd.user_id, cmd.title, cmd.description)?;
        self.session_repo.save(&session).await?;

        // 3. Publish event
        self.event_bus.publish(session.events()).await?;

        Ok(session)
    }
}
```

### Dashboard Module Integration

```rust
// backend/src/application/dashboard/queries/get_membership_status.rs

pub struct GetMembershipStatusHandler {
    membership_reader: Arc<dyn MembershipReader>,
    access_checker: Arc<dyn AccessChecker>,
}

#[derive(Debug, Serialize)]
pub struct MembershipStatusView {
    pub tier: MembershipTier,
    pub status: MembershipStatus,
    pub expires_at: Option<Timestamp>,
    pub limits: TierLimits,
    pub usage: UsageStats,
    pub usage_percent: UsagePercent,
}

#[derive(Debug, Serialize)]
pub struct UsagePercent {
    pub sessions: Option<u8>,  // None if unlimited
    pub cycles: Option<u8>,
}

impl GetMembershipStatusHandler {
    pub async fn handle(&self, user_id: UserId) -> Result<MembershipStatusView, DomainError> {
        let membership = self.membership_reader.get_by_user(&user_id).await?;
        let limits = self.access_checker.get_tier_limits(&user_id).await?;
        let usage = self.access_checker.get_usage(&user_id).await?;

        let usage_percent = UsagePercent {
            sessions: limits.max_sessions.map(|max| {
                ((usage.active_sessions as f32 / max as f32) * 100.0) as u8
            }),
            cycles: None, // Calculated per-session
        };

        Ok(MembershipStatusView {
            tier: membership.tier,
            status: membership.status,
            expires_at: membership.current_period_end,
            limits,
            usage,
            usage_percent,
        })
    }
}
```

---

## Stripe Webhook Integration

### Webhook Events Handled

| Stripe Event | Action | Domain Event |
|--------------|--------|--------------|
| `customer.subscription.created` | Activate membership | `MembershipActivated` |
| `customer.subscription.updated` | Update tier/status | `MembershipTierChanged` |
| `customer.subscription.deleted` | Cancel membership | `MembershipCancelled` |
| `invoice.paid` | Renew membership | `MembershipRenewed` |
| `invoice.payment_failed` | Mark past due | `MembershipPastDue` |
| `customer.subscription.trial_will_end` | Send reminder | (notification only) |

### Webhook Handler

```rust
// backend/src/adapters/stripe/webhook_handler.rs

pub struct StripeWebhookHandler {
    membership_repo: Arc<dyn MembershipRepository>,
    event_bus: Arc<dyn EventPublisher>,
    stripe_secret: String,
}

impl StripeWebhookHandler {
    pub async fn handle(&self, payload: &str, signature: &str) -> Result<(), WebhookError> {
        // 1. Verify webhook signature
        let event = stripe::Webhook::construct_event(
            payload,
            signature,
            &self.stripe_secret,
        )?;

        // 2. Process based on event type
        match event.type_.as_str() {
            "customer.subscription.created" => {
                self.handle_subscription_created(event.data.object).await?;
            }
            "customer.subscription.updated" => {
                self.handle_subscription_updated(event.data.object).await?;
            }
            "customer.subscription.deleted" => {
                self.handle_subscription_deleted(event.data.object).await?;
            }
            "invoice.paid" => {
                self.handle_invoice_paid(event.data.object).await?;
            }
            "invoice.payment_failed" => {
                self.handle_payment_failed(event.data.object).await?;
            }
            _ => {
                // Log unhandled event types
                tracing::debug!("Unhandled Stripe event: {}", event.type_);
            }
        }

        Ok(())
    }

    async fn handle_subscription_created(&self, data: Value) -> Result<(), WebhookError> {
        let subscription: stripe::Subscription = serde_json::from_value(data)?;

        // Extract user_id from metadata
        let user_id = subscription.metadata
            .get("user_id")
            .ok_or(WebhookError::MissingMetadata("user_id"))?;

        // Determine tier from price_id
        let tier = self.price_id_to_tier(&subscription.items.data[0].price.id)?;

        // Activate membership
        let mut membership = self.membership_repo
            .get_by_user(&UserId::from_string(user_id))
            .await?;

        membership.activate_paid(
            tier,
            StripeSubscriptionId::new(subscription.id.to_string()),
            StripeCustomerId::new(subscription.customer.id().to_string()),
            Timestamp::from_unix(subscription.current_period_end),
        )?;

        self.membership_repo.save(&membership).await?;
        self.event_bus.publish(membership.events()).await?;

        Ok(())
    }

    fn price_id_to_tier(&self, price_id: &str) -> Result<MembershipTier, WebhookError> {
        // Map Stripe price IDs to tiers (configured via env vars)
        match price_id {
            id if id == std::env::var("STRIPE_MONTHLY_PRICE_ID").unwrap_or_default() => {
                Ok(MembershipTier::Monthly)
            }
            id if id == std::env::var("STRIPE_ANNUAL_PRICE_ID").unwrap_or_default() => {
                Ok(MembershipTier::Annual)
            }
            _ => Err(WebhookError::UnknownPriceId(price_id.to_string())),
        }
    }
}
```

---

## Frontend Integration

### Pricing Page

```svelte
<!-- frontend/src/routes/pricing/+page.svelte -->
<script lang="ts">
  import { createCheckoutSession } from '$lib/api/membership';

  const plans = [
    {
      tier: 'monthly',
      name: 'Monthly',
      price: 1999,  // cents!
      priceDisplay: '$19.99/mo',
      features: ['10 active sessions', '20 cycles per session', 'PDF/CSV export'],
    },
    {
      tier: 'annual',
      name: 'Annual',
      price: 14999,  // cents!
      priceDisplay: '$149.99/yr',
      badge: 'Save 37%',
      features: ['Unlimited sessions', 'Unlimited cycles', 'PDF/CSV export', 'API access'],
    },
  ];

  async function subscribe(tier: string) {
    const { checkoutUrl } = await createCheckoutSession(tier);
    window.location.href = checkoutUrl;
  }
</script>

<div class="pricing-grid">
  {#each plans as plan}
    <div class="plan-card">
      <h2>{plan.name}</h2>
      {#if plan.badge}
        <span class="badge">{plan.badge}</span>
      {/if}
      <p class="price">{plan.priceDisplay}</p>
      <ul>
        {#each plan.features as feature}
          <li>{feature}</li>
        {/each}
      </ul>
      <button on:click={() => subscribe(plan.tier)}>
        Subscribe
      </button>
    </div>
  {/each}
</div>
```

### Access Denied Component

```svelte
<!-- frontend/src/lib/components/AccessDenied.svelte -->
<script lang="ts">
  import type { AccessDeniedReason } from '$lib/types';

  export let reason: AccessDeniedReason;

  const messages: Record<string, { title: string; action: string; href: string }> = {
    NoMembership: {
      title: 'Membership Required',
      action: 'Choose a plan',
      href: '/pricing',
    },
    MembershipExpired: {
      title: 'Membership Expired',
      action: 'Renew now',
      href: '/account/billing',
    },
    SessionLimitReached: {
      title: 'Session Limit Reached',
      action: 'Upgrade for more sessions',
      href: '/pricing',
    },
    FeatureNotIncluded: {
      title: 'Feature Not Available',
      action: 'Upgrade to unlock',
      href: '/pricing',
    },
  };

  $: config = messages[reason.type] || messages.NoMembership;
</script>

<div class="access-denied">
  <h3>{config.title}</h3>

  {#if reason.type === 'SessionLimitReached'}
    <p>You've used {reason.current} of {reason.max} sessions.</p>
  {:else if reason.type === 'FeatureNotIncluded'}
    <p>This feature requires {reason.required_tier} tier.</p>
  {/if}

  <a href={config.href} class="btn-primary">{config.action}</a>
</div>
```

### Membership Status in Dashboard

```svelte
<!-- frontend/src/lib/components/MembershipBadge.svelte -->
<script lang="ts">
  import { membershipStatus } from '$lib/stores/membership';

  const tierColors = {
    Free: 'gray',
    Monthly: 'blue',
    Annual: 'gold',
  };
</script>

{#if $membershipStatus}
  <div class="membership-badge tier-{$membershipStatus.tier.toLowerCase()}">
    <span class="tier-name">{$membershipStatus.tier}</span>

    {#if $membershipStatus.usage_percent.sessions}
      <div class="usage-bar">
        <div
          class="usage-fill"
          style="width: {$membershipStatus.usage_percent.sessions}%"
        />
      </div>
      <span class="usage-text">
        {$membershipStatus.usage.active_sessions} / {$membershipStatus.limits.max_sessions} sessions
      </span>
    {/if}

    {#if $membershipStatus.expires_at}
      <span class="expires">
        Renews {formatDate($membershipStatus.expires_at)}
      </span>
    {/if}
  </div>
{/if}
```

---

## Failure Modes

| Failure | Impact | Detection | Recovery |
|---------|--------|-----------|----------|
| Stripe webhook fails | Membership not updated | Webhook returns 5xx | Stripe retries with backoff |
| AccessChecker unavailable | Session creation fails | Timeout/error | Return error to user, log alert |
| Stale membership cache | Wrong access decision | N/A (eventual consistency) | Short TTL (5 min), invalidate on webhook |
| Stripe customer mismatch | Wrong user charged | user_id metadata missing | Reject webhook, alert ops |
| Promo code abuse | Revenue loss | Multiple uses of same code | One-time use validation |

### Idempotency

Stripe webhooks can be delivered multiple times. All handlers must be idempotent:

```rust
async fn handle_subscription_created(&self, subscription: &Subscription) -> Result<()> {
    let user_id = extract_user_id(subscription)?;

    // Check if already processed
    let existing = self.membership_repo.get_by_stripe_subscription(
        &subscription.id
    ).await;

    if existing.is_ok() {
        // Already processed, skip
        tracing::info!("Subscription {} already processed", subscription.id);
        return Ok(());
    }

    // Process normally...
}
```

---

## Security Considerations

1. **Webhook Signature Verification**: Always verify Stripe webhook signatures
2. **User ID Validation**: Validate user_id from metadata exists in our system
3. **Promo Code Security**: Rate limit promo code attempts, use cryptographic codes
4. **Access Check Caching**: Cache results briefly (5 min) but invalidate on events
5. **PCI Compliance**: Never store card numbers; use Stripe Checkout/Elements

```rust
// Promo code format: PREFIX-RANDOM
// Example: WORKSHOP2026-A7K9M3
fn generate_promo_code(prefix: &str) -> String {
    let random: String = (0..6)
        .map(|_| {
            let idx = rand::random::<usize>() % 36;
            if idx < 10 {
                (b'0' + idx as u8) as char
            } else {
                (b'A' + (idx - 10) as u8) as char
            }
        })
        .collect();

    format!("{}-{}", prefix, random)
}
```

---

## API Contracts

### Create Checkout Session

```
POST /api/memberships/checkout

Request:
{
  "tier": "monthly" | "annual",
  "success_url": "https://app.choicesherpa.com/account?checkout=success",
  "cancel_url": "https://app.choicesherpa.com/pricing"
}

Response:
{
  "checkout_url": "https://checkout.stripe.com/c/pay/cs_test_..."
}
```

### Redeem Promo Code

```
POST /api/memberships/promo

Request:
{
  "promo_code": "WORKSHOP2026-A7K9M3"
}

Response (201):
{
  "membership": {
    "id": "mem_123",
    "tier": "Annual",
    "status": "Active",
    "expires_at": "2027-01-07T00:00:00Z"
  }
}

Response (400):
{
  "error": {
    "code": "INVALID_PROMO_CODE",
    "message": "This promo code is invalid or has expired"
  }
}
```

### Get Customer Portal

```
POST /api/memberships/portal

Response:
{
  "portal_url": "https://billing.stripe.com/p/session/..."
}
```

### Stripe Webhook

```
POST /webhooks/stripe

Headers:
  Stripe-Signature: t=1234567890,v1=abc123...

Body: (raw Stripe event JSON)

Response: 200 OK (empty body)
```

---

## Implementation Phases

### Phase 1: Core Access Control

**Goal:** AccessChecker port with in-memory tier limits

**Deliverables:**
- [x] AccessChecker port definition
- [x] TierLimits value object (+ MembershipTier enum)
- [x] StubAccessChecker implementation (for development/testing)
- [ ] Session module integration (blocked: requires Session aggregate from Loop 3)

**Exit Criteria:** Session creation blocked when limit reached

---

### Phase 2: Promo Code Flow

**Goal:** Workshop users can redeem codes for free membership

**Deliverables:**
- [ ] PromoCode value object
- [ ] Promo validation logic
- [ ] POST /api/memberships/promo endpoint
- [ ] Promo code generation admin tool

**Exit Criteria:** Workshop user redeems code, gets Annual membership

---

### Phase 3: Stripe Integration

**Goal:** Paid subscriptions via Stripe Checkout

**Deliverables:**
- [ ] PaymentProvider port
- [ ] StripePaymentAdapter
- [ ] Checkout session creation
- [ ] Customer portal session

**Exit Criteria:** User can subscribe to Monthly plan via Stripe

---

### Phase 4: Webhook Handling

**Goal:** Membership state synced with Stripe

**Deliverables:**
- [ ] Webhook signature verification
- [ ] subscription.created handler
- [ ] subscription.updated handler
- [ ] subscription.deleted handler
- [ ] invoice.payment_failed handler

**Exit Criteria:** Membership status updates automatically on Stripe events

---

### Phase 5: Frontend Integration

**Goal:** Complete user-facing flows

**Deliverables:**
- [ ] Pricing page with plan selection
- [ ] AccessDenied component
- [ ] MembershipBadge component
- [ ] Account billing page
- [ ] Upgrade prompts

**Exit Criteria:** End-to-end subscription flow works

---

## Testing Strategy

### Unit Tests

| Component | Test Focus |
|-----------|------------|
| AccessChecker | Tier limits, access decisions |
| TierLimits | Configuration per tier |
| PromoCode | Validation, expiry, single-use |
| WebhookHandler | Event parsing, idempotency |

### Integration Tests

| Test | Scenario |
|------|----------|
| SessionAccessControl | Create session blocked at limit |
| PromoRedemption | Valid code → Active membership |
| StripeCheckout | Checkout → Webhook → Active membership |
| Cancellation | Cancel → Webhook → Cancelled status |

### E2E Tests

| Journey | Steps |
|---------|-------|
| FreeTierLimit | Create 3 sessions → 4th blocked → Upgrade → 4th succeeds |
| WorkshopUser | Redeem promo → Create session → Full access |
| PaidSubscription | Checkout → Payment → Active → Cancel |

---

## Coordination Points

### Events Published

| Event | Published By | Consumed By |
|-------|--------------|-------------|
| `MembershipCreated` | membership | dashboard (refresh status) |
| `MembershipActivated` | membership | session (invalidate cache) |
| `MembershipCancelled` | membership | dashboard (show warning) |
| `MembershipExpired` | membership | session (enforce limits) |
| `MembershipTierChanged` | membership | all (refresh limits) |

### Cache Invalidation

When membership changes, invalidate access check cache:

```rust
// Subscribe to membership events
#[async_trait]
impl EventHandler for AccessCacheInvalidator {
    async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError> {
        if event.aggregate_type == "Membership" {
            let user_id = UserId::from_string(&event.aggregate_id);
            self.cache.invalidate(&user_id).await;
        }
        Ok(())
    }
}
```

---

## File Structure

```
backend/src/
├── domain/membership/
│   ├── access_checker.rs         # MembershipAccessChecker impl
│   └── tier_limits.rs            # TierLimits configuration
├── ports/
│   ├── access_checker.rs         # AccessChecker trait
│   └── payment_provider.rs       # PaymentProvider trait
├── adapters/
│   ├── stripe/
│   │   ├── mod.rs
│   │   ├── payment_adapter.rs    # StripePaymentAdapter
│   │   ├── webhook_handler.rs    # Webhook processing
│   │   └── types.rs              # Stripe type mappings
│   └── http/
│       └── membership_routes.rs  # API endpoints
└── application/
    └── membership/
        └── commands/
            └── redeem_promo.rs   # Promo code redemption

frontend/src/
├── routes/
│   ├── pricing/
│   │   └── +page.svelte          # Pricing page
│   └── account/
│       └── billing/
│           └── +page.svelte      # Billing management
├── lib/
│   ├── api/
│   │   └── membership.ts         # API client
│   ├── components/
│   │   ├── AccessDenied.svelte
│   │   └── MembershipBadge.svelte
│   └── stores/
│       └── membership.ts         # Membership state
```

---

## Environment Variables

```bash
# Stripe Configuration
STRIPE_SECRET_KEY=sk_test_...
STRIPE_PUBLISHABLE_KEY=pk_test_...
STRIPE_WEBHOOK_SECRET=whsec_...
STRIPE_MONTHLY_PRICE_ID=price_monthly_...
STRIPE_ANNUAL_PRICE_ID=price_annual_...

# Promo Code Configuration
PROMO_CODE_SECRET=random_secret_for_code_generation
```

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Required for all endpoints except webhook |
| Authorization Model | Fail-secure AccessChecker; user owns membership |
| Sensitive Data | Payment data, subscription IDs, promo codes |
| Rate Limiting | Required (see below) |
| Audit Logging | Membership changes, promo redemptions, access denials |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| `stripe_customer_id` | Confidential | Do not expose to frontend |
| `stripe_subscription_id` | Confidential | Do not expose to frontend |
| `promo_code` | Confidential | One-time use; rate limit redemption |
| `tier` | Public | Can be displayed |
| `usage_stats` | Internal | Safe to display to user |
| `STRIPE_SECRET_KEY` | Secret | Never log; load from vault |
| `STRIPE_WEBHOOK_SECRET` | Secret | Never log; load from vault |

### Security Controls

- **Fail-Secure AccessChecker**: On ANY error, deny access:
  ```rust
  // No membership = no access (not free tier)
  match membership {
      Some(m) if m.has_access => allow,
      _ => deny,  // Fail secure
  }
  ```
- **No Implicit Free Tier**: Users without membership get zero access
- **Webhook Signature Verification**: See `stripe-webhook-handling.md`
- **Promo Code Security**:
  - Cryptographically random codes (6+ alphanumeric characters)
  - One-time use (mark used immediately, before granting access)
  - Rate limit: 5 attempts per IP per hour
- **Cache Invalidation**: Clear AccessChecker cache on any membership event

### Rate Limiting

| Endpoint | Limit | Window |
|----------|-------|--------|
| `POST /api/memberships/checkout` | 5 | per minute |
| `POST /api/memberships/promo` | 5 | per hour per IP |
| `POST /webhooks/stripe` | 60 | per minute |
| `POST /api/memberships/portal` | 10 | per minute |

### PCI Compliance Note

- **Never store card numbers**: Use Stripe Checkout/Elements exclusively
- **Card last4 only**: The only card data stored is display-safe last 4 digits
- Stripe handles all PCI-DSS compliance

---

## Related Documents

- **Module Spec:** docs/modules/membership.md
- **Checklist:** REQUIREMENTS/CHECKLIST-membership.md
- **Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md
- **Session Module:** docs/modules/session.md

---

*Version: 1.0.0*
*Created: 2026-01-07*
*Depends On: foundation module (Phase 1)*
