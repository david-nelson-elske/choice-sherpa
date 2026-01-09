# Membership Module Checklist

**Module:** Membership
**Language:** Rust
**Dependencies:** foundation
**Phase:** 2 (parallel with session, proact-types)

---

## Overview

The Membership module manages user subscriptions, access control, and payment integration. It gates access to the Choice Sherpa platform, supporting free workshop/beta users, monthly subscribers, and annual subscribers. This is a full hexagonal module with ports and adapters, including external payment provider integration (Stripe).

**CRITICAL:** All monetary values use integers (cents), never floats.

---

## File Inventory

### Domain Layer - Value Objects (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/membership/mod.rs` | Module exports | ✅ |
| `backend/src/domain/membership/status.rs` | MembershipStatus enum (18 tests inline) | ✅ |
| `backend/src/domain/membership/value_objects/mod.rs` | Value object exports | ⬜ |
| `backend/src/domain/membership/value_objects/money.rs` | Money value object (CENTS!) | ⬜ |
| `backend/src/domain/membership/value_objects/tier.rs` | MembershipTier enum | ⬜ |
| `backend/src/domain/membership/value_objects/billing_period.rs` | BillingPeriod enum | ⬜ |
| `backend/src/domain/membership/value_objects/promo_code.rs` | PromoCode value object | ⬜ |
| `backend/src/domain/membership/value_objects/plan_price.rs` | PlanPrice configuration | ⬜ |

> **Note:** Tests are inline in implementation files using `#[cfg(test)] mod tests` (Rust convention).

### Domain Layer - Value Object Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/membership/value_objects/money_test.rs` | Money tests | ⬜ |
| `backend/src/domain/membership/value_objects/tier_test.rs` | Tier tests | ⬜ |
| `backend/src/domain/membership/value_objects/status_test.rs` | Status tests | ⬜ |
| `backend/src/domain/membership/value_objects/promo_code_test.rs` | PromoCode tests | ⬜ |

### Domain Layer - Aggregate (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/membership/membership.rs` | Membership aggregate | ⬜ |
| `backend/src/domain/membership/events.rs` | MembershipEvent enum | ⬜ |
| `backend/src/domain/membership/errors.rs` | Membership-specific errors | ⬜ |

### Domain Layer - Aggregate Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/membership/membership_test.rs` | Aggregate tests | ⬜ |
| `backend/src/domain/membership/events_test.rs` | Event tests | ⬜ |

### Ports (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/ports/membership_repository.rs` | MembershipRepository trait | ⬜ |
| `backend/src/ports/membership_reader.rs` | MembershipReader trait (CQRS) | ⬜ |
| `backend/src/ports/access_checker.rs` | AccessChecker trait (cross-module) | ⬜ |
| `backend/src/ports/payment_provider.rs` | PaymentProvider trait (external) | ⬜ |
| `backend/src/ports/promo_code_validator.rs` | PromoCodeValidator trait | ⬜ |

### Application Layer - Commands (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/commands/create_free_membership.rs` | CreateFreeMembership handler | ⬜ |
| `backend/src/application/commands/create_paid_membership.rs` | CreatePaidMembership handler | ⬜ |
| `backend/src/application/commands/cancel_membership.rs` | CancelMembership handler | ⬜ |
| `backend/src/application/commands/handle_payment_webhook.rs` | HandlePaymentWebhook handler | ⬜ |

### Application Layer - Command Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/commands/create_free_membership_test.rs` | CreateFreeMembership tests | ⬜ |
| `backend/src/application/commands/create_paid_membership_test.rs` | CreatePaidMembership tests | ⬜ |
| `backend/src/application/commands/cancel_membership_test.rs` | CancelMembership tests | ⬜ |
| `backend/src/application/commands/handle_payment_webhook_test.rs` | HandlePaymentWebhook tests | ⬜ |

### Application Layer - Queries (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/queries/get_membership.rs` | GetMembership handler | ⬜ |
| `backend/src/application/queries/check_access.rs` | CheckAccess handler | ⬜ |
| `backend/src/application/queries/get_prices.rs` | GetPrices handler | ⬜ |

### Application Layer - Query Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/queries/get_membership_test.rs` | GetMembership tests | ⬜ |
| `backend/src/application/queries/check_access_test.rs` | CheckAccess tests | ⬜ |

### HTTP Adapter (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/http/membership/mod.rs` | Module exports | ⬜ |
| `backend/src/adapters/http/membership/handlers.rs` | HTTP handlers | ⬜ |
| `backend/src/adapters/http/membership/dto.rs` | Request/Response DTOs | ⬜ |
| `backend/src/adapters/http/membership/routes.rs` | Route definitions | ⬜ |

### HTTP Adapter Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/http/membership/handlers_test.rs` | Handler tests | ⬜ |

### Postgres Adapter (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/postgres/membership_repository.rs` | PostgresMembershipRepository | ⬜ |
| `backend/src/adapters/postgres/membership_reader.rs` | PostgresMembershipReader | ⬜ |
| `backend/src/adapters/postgres/access_checker_impl.rs` | AccessChecker implementation | ⬜ |

### Postgres Adapter Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/postgres/membership_repository_test.rs` | Repository tests | ⬜ |
| `backend/src/adapters/postgres/membership_reader_test.rs` | Reader tests | ⬜ |

### Stripe Adapter (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/stripe/mod.rs` | Module exports | ⬜ |
| `backend/src/adapters/stripe/stripe_adapter.rs` | StripePaymentAdapter | ⬜ |
| `backend/src/adapters/stripe/webhook_types.rs` | Webhook event types | ⬜ |
| `backend/src/adapters/stripe/mock_payment_provider.rs` | Mock for testing | ⬜ |

### Stripe Adapter Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/stripe/stripe_adapter_test.rs` | Stripe adapter tests | ⬜ |

### Database Migrations

| File | Description | Status |
|------|-------------|--------|
| `backend/migrations/XXX_create_memberships.sql` | Memberships table | ⬜ |
| `backend/migrations/XXX_create_promo_codes.sql` | Promo codes table | ⬜ |

### Frontend Domain (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/membership/domain/membership.ts` | Membership types | ⬜ |
| `frontend/src/modules/membership/domain/money.ts` | Money type (cents!) | ⬜ |
| `frontend/src/modules/membership/domain/tier.ts` | MembershipTier type | ⬜ |

### Frontend Domain Tests (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/membership/domain/membership.test.ts` | Membership tests | ⬜ |
| `frontend/src/modules/membership/domain/money.test.ts` | Money tests | ⬜ |

### Frontend API (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/membership/api/membership-api.ts` | API client | ⬜ |
| `frontend/src/modules/membership/api/use-membership.ts` | Membership hook | ⬜ |
| `frontend/src/modules/membership/api/use-prices.ts` | Prices hook | ⬜ |

### Frontend Components (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/membership/components/MembershipBadge.svelte` | Status badge | ⬜ |
| `frontend/src/modules/membership/components/PricingTable.svelte` | Pricing display | ⬜ |
| `frontend/src/modules/membership/components/CheckoutButton.svelte` | Checkout CTA | ⬜ |
| `frontend/src/modules/membership/components/PromoCodeInput.svelte` | Promo input | ⬜ |
| `frontend/src/modules/membership/components/MembershipStatus.svelte` | Status display | ⬜ |
| `frontend/src/modules/membership/components/UpgradePrompt.svelte` | Upgrade CTA | ⬜ |
| `frontend/src/modules/membership/index.ts` | Module exports | ⬜ |

### Frontend Component Tests (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/membership/components/MembershipBadge.test.ts` | Badge tests | ⬜ |
| `frontend/src/modules/membership/components/PricingTable.test.ts` | Pricing tests | ⬜ |

### Frontend Pages (SvelteKit)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/routes/pricing/+page.svelte` | Pricing page | ⬜ |
| `frontend/src/routes/account/+page.svelte` | Account/membership page | ⬜ |

---

## Test Inventory

### Money Value Object Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_money_from_cents_preserves_value` | Cents preserved exactly | ⬜ |
| `test_money_from_dollars_converts_to_cents` | $19.99 becomes 1999 | ⬜ |
| `test_money_zero_is_zero_cents` | Zero is 0 cents | ⬜ |
| `test_money_display_formats_correctly` | "$19.99 CAD" format | ⬜ |
| `test_money_add_same_currency_works` | Addition works | ⬜ |
| `test_money_add_different_currency_fails` | Currency mismatch rejected | ⬜ |
| `test_money_cents_returns_integer` | No floating point | ⬜ |
| `test_money_serializes_as_cents` | JSON uses integer | ⬜ |

### MembershipTier Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_tier_free_does_not_require_payment` | Free is free | ⬜ |
| `test_tier_monthly_requires_payment` | Monthly needs payment | ⬜ |
| `test_tier_annual_requires_payment` | Annual needs payment | ⬜ |
| `test_tier_billing_period_matches_tier` | Periods are correct | ⬜ |

### MembershipStatus Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_status_active_has_access` | Active grants access | ⬜ |
| `test_status_past_due_has_access` | Past due still has access | ⬜ |
| `test_status_cancelled_has_access` | Cancelled still has access | ⬜ |
| `test_status_expired_no_access` | Expired loses access | ⬜ |
| `test_status_pending_no_access` | Pending no access | ⬜ |
| `test_status_can_transition_pending_to_active` | Valid transition | ⬜ |
| `test_status_cannot_transition_active_to_pending` | Invalid transition | ⬜ |

### PromoCode Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_promo_code_new_validates_not_empty` | Empty rejected | ⬜ |
| `test_promo_code_new_validates_max_length` | Too long rejected | ⬜ |
| `test_promo_code_new_uppercases` | Code uppercased | ⬜ |
| `test_promo_code_workshop_prefix_sets_type` | WORKSHOP* is workshop | ⬜ |
| `test_promo_code_beta_prefix_sets_type` | BETA* is beta | ⬜ |
| `test_promo_code_grants_free_tier` | Promo grants free | ⬜ |

### Membership Aggregate Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_membership_create_free_is_immediately_active` | Free is active | ⬜ |
| `test_membership_create_free_requires_promo` | Promo required | ⬜ |
| `test_membership_create_free_sets_annual_period` | Annual duration | ⬜ |
| `test_membership_create_paid_is_pending` | Paid starts pending | ⬜ |
| `test_membership_create_paid_rejects_free_tier` | Use create_free instead | ⬜ |
| `test_membership_activate_transitions_to_active` | Activation works | ⬜ |
| `test_membership_activate_sets_period_end` | Period end set | ⬜ |
| `test_membership_has_access_true_when_active_in_period` | Access check | ⬜ |
| `test_membership_has_access_false_when_expired` | No access after expiry | ⬜ |
| `test_membership_has_access_true_when_cancelled_in_period` | Cancelled still works | ⬜ |
| `test_membership_cancel_sets_cancelled_at` | Timestamp set | ⬜ |
| `test_membership_cancel_keeps_active_until_period_end` | Grace period | ⬜ |
| `test_membership_expire_removes_access` | Expiry works | ⬜ |
| `test_membership_renew_extends_period` | Renewal works | ⬜ |
| `test_membership_mark_payment_failed_sets_past_due` | Past due status | ⬜ |
| `test_membership_recover_payment_restores_active` | Recovery works | ⬜ |
| `test_membership_upgrade_changes_tier` | Upgrade works | ⬜ |
| `test_membership_upgrade_rejects_same_tier` | No-op rejected | ⬜ |
| `test_membership_upgrade_rejects_downgrade` | Downgrade rejected | ⬜ |
| `test_membership_set_external_ids_stores_ids` | External IDs stored | ⬜ |

### MembershipEvent Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_membership_event_membership_id_returns_id` | ID accessor | ⬜ |
| `test_membership_event_type_returns_type_string` | Type accessor | ⬜ |
| `test_membership_event_serializes_to_json` | Serialization | ⬜ |
| `test_membership_event_deserializes_from_json` | Deserialization | ⬜ |

### CreateFreeMembership Command Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_create_free_membership_handler_success` | Happy path | ⬜ |
| `test_create_free_membership_handler_rejects_invalid_promo` | Invalid promo | ⬜ |
| `test_create_free_membership_handler_rejects_duplicate_user` | Already has membership | ⬜ |
| `test_create_free_membership_handler_saves_to_repo` | Repo save called | ⬜ |
| `test_create_free_membership_handler_publishes_events` | Events published | ⬜ |

### CreatePaidMembership Command Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_create_paid_membership_handler_success` | Happy path | ⬜ |
| `test_create_paid_membership_handler_creates_stripe_customer` | Customer created | ⬜ |
| `test_create_paid_membership_handler_returns_checkout_url` | URL returned | ⬜ |
| `test_create_paid_membership_handler_rejects_free_tier` | Use free endpoint | ⬜ |
| `test_create_paid_membership_handler_rejects_existing_active` | Already subscribed | ⬜ |

### CancelMembership Command Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_cancel_membership_handler_success` | Happy path | ⬜ |
| `test_cancel_membership_handler_calls_stripe` | Stripe cancellation | ⬜ |
| `test_cancel_membership_handler_not_found` | 404 case | ⬜ |
| `test_cancel_membership_handler_publishes_events` | Events published | ⬜ |

### HandlePaymentWebhook Command Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_webhook_handler_verifies_signature` | Signature checked | ⬜ |
| `test_webhook_handler_rejects_invalid_signature` | Bad sig rejected | ⬜ |
| `test_webhook_handler_activates_on_payment_succeeded` | Activation | ⬜ |
| `test_webhook_handler_marks_past_due_on_payment_failed` | Past due | ⬜ |
| `test_webhook_handler_expires_on_subscription_deleted` | Expiration | ⬜ |
| `test_webhook_handler_renews_on_invoice_paid` | Renewal | ⬜ |

### GetMembership Query Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_get_membership_handler_success` | Happy path | ⬜ |
| `test_get_membership_handler_returns_none` | No membership | ⬜ |
| `test_get_membership_handler_includes_days_remaining` | Days calculated | ⬜ |

### CheckAccess Query Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_check_access_handler_true_for_active` | Active has access | ⬜ |
| `test_check_access_handler_false_for_expired` | Expired no access | ⬜ |
| `test_check_access_handler_false_for_no_membership` | No membership | ⬜ |

### AccessChecker Implementation Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_access_checker_can_create_session_true` | Active can create | ⬜ |
| `test_access_checker_can_create_session_false` | Expired cannot | ⬜ |
| `test_access_checker_get_tier_returns_tier` | Tier returned | ⬜ |
| `test_access_checker_get_limits_returns_tier_limits` | Limits correct | ⬜ |
| `test_access_checker_get_limits_no_membership` | No access limits | ⬜ |

### HTTP Handler Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_get_membership_returns_view` | GET works | ⬜ |
| `test_get_membership_returns_null_for_none` | No membership | ⬜ |
| `test_post_free_membership_creates_membership` | POST free works | ⬜ |
| `test_post_free_membership_returns_400_for_invalid_promo` | Bad promo | ⬜ |
| `test_post_checkout_returns_checkout_url` | Checkout URL | ⬜ |
| `test_post_cancel_cancels_membership` | Cancel works | ⬜ |
| `test_get_portal_returns_url` | Portal URL | ⬜ |
| `test_post_webhook_processes_event` | Webhook works | ⬜ |
| `test_post_webhook_returns_400_for_invalid_signature` | Bad signature | ⬜ |
| `test_get_prices_returns_all_plans` | Prices returned | ⬜ |
| `test_endpoints_require_authentication` | Auth required | ⬜ |

### Postgres Repository Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_postgres_repo_save_persists_membership` | Save works | ⬜ |
| `test_postgres_repo_save_handles_duplicate_user` | Duplicate error | ⬜ |
| `test_postgres_repo_update_modifies_membership` | Update works | ⬜ |
| `test_postgres_repo_find_by_id_returns_membership` | Find by ID | ⬜ |
| `test_postgres_repo_find_by_user_returns_membership` | Find by user | ⬜ |
| `test_postgres_repo_find_expiring_within_days` | Expiring query | ⬜ |

### Postgres Reader Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_postgres_reader_get_by_user_returns_view` | Get works | ⬜ |
| `test_postgres_reader_check_access_returns_bool` | Access check | ⬜ |
| `test_postgres_reader_get_tier_returns_tier` | Tier query | ⬜ |
| `test_postgres_reader_list_expiring` | Expiring list | ⬜ |
| `test_postgres_reader_get_statistics` | Stats query | ⬜ |

### Stripe Adapter Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_stripe_adapter_create_customer` | Customer creation | ⬜ |
| `test_stripe_adapter_create_checkout_session` | Checkout session | ⬜ |
| `test_stripe_adapter_create_portal_session` | Portal session | ⬜ |
| `test_stripe_adapter_cancel_subscription` | Cancellation | ⬜ |
| `test_stripe_adapter_verify_webhook_signature` | Signature verify | ⬜ |
| `test_stripe_adapter_verify_webhook_rejects_invalid` | Bad sig | ⬜ |

### Frontend Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_money_format_cents_to_display` | $19.99 display | ⬜ |
| `test_money_format_handles_zero` | $0.00 display | ⬜ |
| `test_membership_badge_shows_tier` | Badge display | ⬜ |
| `test_pricing_table_shows_all_plans` | Plans display | ⬜ |
| `test_checkout_button_redirects` | Checkout redirect | ⬜ |

---

## Error Codes

| Error Code | HTTP Status | Condition |
|------------|-------------|-----------|
| `VALIDATION_FAILED` | 400 | Invalid promo code, tier, etc. |
| `MEMBERSHIP_NOT_FOUND` | 404 | No membership for user |
| `MEMBERSHIP_EXISTS` | 409 | User already has membership |
| `MEMBERSHIP_EXPIRED` | 402 | Membership has expired |
| `INVALID_TIER` | 400 | Unknown tier specified |
| `INVALID_PROMO_CODE` | 400 | Promo code not valid |
| `PROMO_CODE_EXHAUSTED` | 400 | Promo code max uses reached |
| `PAYMENT_FAILED` | 402 | Payment processing failed |
| `INVALID_WEBHOOK_SIGNATURE` | 400 | Stripe signature invalid |
| `DATABASE_ERROR` | 500 | Database operation failed |

---

## Business Rules

| Rule | Implementation | Test | Status |
|------|----------------|------|--------|
| Money stored as cents (integers) | Money value object | `test_money_from_dollars_converts_to_cents` | ⬜ |
| Each user has at most one membership | Unique constraint + check | `test_create_free_membership_handler_rejects_duplicate_user` | ⬜ |
| Free tier requires valid promo code | Validation in create_free | `test_membership_create_free_requires_promo` | ⬜ |
| Paid tier starts as pending | Status logic | `test_membership_create_paid_is_pending` | ⬜ |
| Cancelled members keep access until period end | has_access() check | `test_membership_has_access_true_when_cancelled_in_period` | ⬜ |
| Status transitions follow rules | can_transition_to() | `test_status_cannot_transition_active_to_pending` | ⬜ |
| Webhook signatures verified | verify_webhook_signature | `test_webhook_handler_verifies_signature` | ⬜ |

---

## Verification Commands

```bash
# Run all membership tests
cargo test --package membership -- --nocapture

# Domain layer tests
cargo test --package membership domain:: -- --nocapture

# Value object tests
cargo test --package membership domain::value_objects:: -- --nocapture

# Application layer tests
cargo test --package membership application:: -- --nocapture

# Adapter tests (requires database)
cargo test --package membership adapters:: -- --ignored

# Stripe adapter tests (mock only)
cargo test --package membership adapters::stripe:: -- --nocapture

# Coverage check (target: 85%+)
cargo tarpaulin --package membership --out Html

# Full verification
cargo test --package membership -- --nocapture && cargo clippy --package membership

# Frontend tests
cd frontend && npm test -- --testPathPattern="modules/membership"
```

---

## Exit Criteria

### Module is COMPLETE when:

- [ ] All 65 files in File Inventory exist
- [ ] All 95 tests in Test Inventory pass
- [ ] Domain layer coverage >= 90%
- [ ] Application layer coverage >= 85%
- [ ] Adapter layer coverage >= 80%
- [ ] Database migrations run successfully
- [ ] HTTP endpoints return correct status codes
- [ ] CQRS pattern implemented (Repository + Reader)
- [ ] Domain events published correctly
- [ ] Stripe integration tested with mock
- [ ] Money always stored as cents (verify no floats!)
- [ ] No clippy warnings
- [ ] Frontend components render correctly
- [ ] No TypeScript lint errors
- [ ] AccessChecker integration with session module verified

### Current Status

```
STARTED: membership
Files: 2/65
Tests: 18/95 passing
Status: Only MembershipStatus enum implemented
Next: Money value object, remaining value objects
```

### Exit Signal

```
MODULE COMPLETE: membership
Files: 65/65
Tests: 95/95 passing
Coverage: Domain 92%, Application 87%, Adapters 82%
Money: All values in cents (integer) ✓
```

---

## Implementation Phases

### Phase 1: Value Objects (In Progress)
- [ ] Money value object (CENTS - CRITICAL)
- [ ] MembershipTier enum
- [x] MembershipStatus enum (18 tests passing)
- [ ] BillingPeriod enum
- [ ] PromoCode value object
- [ ] PlanPrice configuration
- [x] Value object tests (partial - status.rs)

### Phase 2: Domain Aggregate
- [ ] Membership aggregate
- [ ] MembershipEvent enum
- [ ] Domain invariants
- [ ] Aggregate tests

### Phase 3: Ports
- [ ] MembershipRepository trait
- [ ] MembershipReader trait
- [ ] AccessChecker trait
- [ ] PaymentProvider trait
- [ ] PromoCodeValidator trait
- [ ] View DTOs

### Phase 4: Commands
- [ ] CreateFreeMembershipCommand + Handler
- [ ] CreatePaidMembershipCommand + Handler
- [ ] CancelMembershipCommand + Handler
- [ ] HandlePaymentWebhookCommand + Handler
- [ ] Command tests with mock repos

### Phase 5: Queries
- [ ] GetMembershipQuery + Handler
- [ ] CheckAccessQuery + Handler
- [ ] GetPricesQuery + Handler
- [ ] Query tests with mock readers

### Phase 6: HTTP Adapter
- [ ] Request/Response DTOs
- [ ] HTTP handlers
- [ ] Route definitions
- [ ] Webhook endpoint
- [ ] Handler tests

### Phase 7: Postgres Adapter
- [ ] Database migrations
- [ ] PostgresMembershipRepository
- [ ] PostgresMembershipReader
- [ ] AccessChecker implementation
- [ ] Integration tests

### Phase 8: Stripe Adapter
- [ ] StripePaymentAdapter
- [ ] Webhook signature verification
- [ ] Mock payment provider
- [ ] Adapter tests

### Phase 9: Frontend
- [ ] TypeScript types (with Money in cents!)
- [ ] API client
- [ ] Svelte hooks
- [ ] Components
- [ ] Pricing page
- [ ] Account page
- [ ] Component tests

### Phase 10: Integration
- [ ] Session module integration (AccessChecker)
- [ ] End-to-end checkout flow test
- [ ] Webhook processing test

---

## Integration Points

### Session Module Integration

The session module's `CreateSessionHandler` must be updated to:

1. Inject `AccessChecker` dependency
2. Call `access_checker.can_create_session(user_id)` before creating
3. Return `CommandError::MembershipRequired` if access denied
4. Optionally check session limits via `access_checker.get_limits(user_id)`

```rust
// New error variant needed in session module:
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    // ... existing variants ...
    #[error("Active membership required")]
    MembershipRequired,

    #[error("Session limit reached for your plan")]
    SessionLimitReached,
}
```

---

## Notes

- **CRITICAL**: All monetary values MUST use integers (cents). Never use f64/f32 for money.
- Stripe webhook secret should be stored securely (environment variable or secrets manager)
- Promo codes are case-insensitive (stored uppercase)
- Free memberships are immediately active, paid start as pending
- Cancelled memberships retain access until period end
- Past due status gives a grace period before expiration
- The AccessChecker port is the integration point with other modules

---

## Pricing Configuration

| Tier | Price (CAD cents) | Price (Display) | Billing Period |
|------|-------------------|-----------------|----------------|
| Free | 0 | $0.00 CAD | Annual |
| Monthly | 1999 | $19.99 CAD | Monthly |
| Annual | 14999 | $149.99 CAD | Annual |

**Note**: Monthly equivalent of annual = 14999 / 12 = 1249 cents = $12.49/month (16% savings)

---

*Generated: 2026-01-07*
*Specification: docs/modules/membership.md*
