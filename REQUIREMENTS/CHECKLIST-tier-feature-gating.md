# Tier Feature Gating Checklist

**Feature:** Tier Feature Matrix Enforcement
**Specification:** features/membership/tier-feature-matrix.md
**Dependencies:** membership module (complete)
**Priority:** P1

---

## Overview

This checklist tracks implementation of tier-based feature gating across modules. The membership module provides the `AccessChecker` port - this work connects it to enforce limits in each consuming module.

**Already Complete:**
- AccessChecker port and implementations
- Session creation gating (can_create_session)
- Cycle creation gating (can_create_cycle)
- Frontend pricing/account UI

**Remaining Work:**
- AI usage limits enforcement
- Export capability gating
- Decision Quality gating
- Session history retention
- Advanced AI model selection

---

## File Inventory

### AI Usage Limits (conversation module)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/conversation/usage_tracker.rs` | Track AI messages per user per day | ⬜ |
| `backend/src/ports/ai_usage_limiter.rs` | Port for checking AI usage limits | ⬜ |
| `backend/src/adapters/postgres/ai_usage_repository.rs` | Persist daily usage counts | ⬜ |
| `backend/migrations/YYYYMMDD_create_ai_usage.sql` | AI usage tracking table | ⬜ |

### Export Gating (cycle/dashboard modules)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/handlers/cycle/export_cycle.rs` | Add tier check before export | ⬜ |
| `backend/src/adapters/http/cycle/handlers.rs` | Return 402 if export not allowed | ⬜ |
| `frontend/src/lib/components/cycle/ExportButton.svelte` | Disable/upgrade prompt for free tier | ⬜ |

### Decision Quality Gating (analysis module)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/handlers/analysis/calculate_dq.rs` | Add tier check before DQ calculation | ⬜ |
| `backend/src/adapters/http/analysis/handlers.rs` | Return 402 if DQ not allowed | ⬜ |
| `frontend/src/lib/components/analysis/DQScore.svelte` | Upgrade prompt for free tier | ⬜ |

### Session History Retention (session module)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/jobs/cleanup_expired_sessions.rs` | Scheduled job to archive old sessions | ⬜ |
| `backend/src/domain/session/retention.rs` | Retention policy by tier | ⬜ |
| `backend/migrations/YYYYMMDD_add_session_archived_at.sql` | Add archived_at column | ⬜ |

### AI Model Selection (conversation module)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/conversation/model_selector.rs` | Select AI model based on tier | ⬜ |
| `backend/src/ports/ai_provider.rs` | Add model parameter to generate() | ⬜ |

---

## Test Inventory

### AI Usage Limits Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_free_tier_limited_to_50_messages_per_day` | Free tier limit | ⬜ |
| `test_premium_tier_limited_to_200_messages_per_day` | Premium limit | ⬜ |
| `test_pro_tier_unlimited_messages` | Pro unlimited | ⬜ |
| `test_usage_resets_at_midnight_utc` | Daily reset | ⬜ |
| `test_returns_limit_exceeded_error` | Error response | ⬜ |

### Export Gating Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_free_tier_cannot_export` | Export blocked | ⬜ |
| `test_premium_tier_can_export` | Export allowed | ⬜ |
| `test_export_returns_402_for_free` | HTTP 402 | ⬜ |

### Decision Quality Gating Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_free_tier_cannot_calculate_dq` | DQ blocked | ⬜ |
| `test_premium_tier_can_calculate_dq` | DQ allowed | ⬜ |
| `test_dq_returns_402_for_free` | HTTP 402 | ⬜ |

### Session Retention Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_free_tier_sessions_archived_after_90_days` | 90 day retention | ⬜ |
| `test_premium_tier_sessions_archived_after_1_year` | 1 year retention | ⬜ |
| `test_pro_tier_sessions_never_archived` | Forever retention | ⬜ |
| `test_cleanup_job_respects_tier` | Job behavior | ⬜ |

### AI Model Selection Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_free_tier_uses_standard_model` | Standard model | ⬜ |
| `test_pro_tier_uses_advanced_model` | Advanced model | ⬜ |

---

## Feature Limits Reference

### AI Messages per Day

| Tier | Limit |
|------|-------|
| Free | 50 |
| Premium | 200 |
| Pro | Unlimited |

### Export Capability

| Tier | Allowed |
|------|---------|
| Free | ❌ |
| Premium | ✅ |
| Pro | ✅ |

### Decision Quality

| Tier | Allowed |
|------|---------|
| Free | ❌ |
| Premium | ✅ |
| Pro | ✅ |

### Session History Retention

| Tier | Retention |
|------|-----------|
| Free | 90 days |
| Premium | 1 year |
| Pro | Forever |

### AI Model

| Tier | Model |
|------|-------|
| Free | Standard |
| Premium | Standard |
| Pro | Advanced |

---

## Implementation Order

1. **AI Usage Limits** - Most user-visible, prevents abuse
2. **Export Gating** - Revenue driver (upgrade prompt)
3. **Decision Quality Gating** - Revenue driver (upgrade prompt)
4. **Session Retention** - Background job, less urgent
5. **AI Model Selection** - Requires AI provider changes

---

## Verification Commands

```bash
# Run tier gating tests
cargo test --package choice-sherpa tier_gating -- --nocapture

# Check AI usage
cargo test --package choice-sherpa ai_usage -- --nocapture

# Check export gating
cargo test --package choice-sherpa export -- --nocapture

# Full verification
cargo test && cargo clippy
```

---

## Exit Criteria

### Feature is COMPLETE when:

- [ ] AI usage limits enforced per tier
- [ ] Export blocked for free tier with upgrade prompt
- [ ] Decision Quality blocked for free tier with upgrade prompt
- [ ] Session cleanup job runs daily
- [ ] Pro tier uses advanced AI model
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Frontend shows appropriate upgrade prompts

### Current Status

```
NOT STARTED: tier-feature-gating
Files: 0/15
Tests: 0/18
Status: Specification complete, implementation pending
```

---

*Generated: 2026-01-10*
*Specification: features/membership/tier-feature-matrix.md*
