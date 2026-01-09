# Choice Sherpa Development Roadmap

**Created:** 2026-01-08
**Goal:** Implement all remaining modules to match architecture specifications

---

## Current Status

### Completed (Phase 1-2)
- [x] foundation (32/32 tasks)
- [x] proact-types (37/37 tasks)
- [x] event-infrastructure (17/17 tasks)
- [x] conversation-lifecycle (11/11 tasks)
- [x] subscription-state-machine (PR #2) - MembershipStatus enum + state transitions
- [x] stripe-webhook-handling (PR #3) - WebhookError, StripeEvent, signature verification, idempotency
- [x] session-events (PR #4) - SessionCreated, SessionRenamed, SessionArchived, CycleAddedToSession domain events
- [x] component-status-validation (PR #5) - Cycle aggregate, status transitions, branching, prerequisite checking

### Remaining Work

---

## Phase 2: Membership Module

### Feature 1: Subscription State Machine
```
/dev features/membership/subscription-state-machine.md
```
**Tasks:** 23 (16 unit tests + 7 integration tests)
**Focus:** MembershipStatus enum, state transitions, has_access logic

### Feature 2: Stripe Webhook Handling
```
/dev features/membership/stripe-webhook-handling.md
```
**Tasks:** 35
**Focus:** Webhook verification, idempotency, event handlers

---

## Phase 3: Session Module

### Feature: Session Events
```
/dev features/session/session-events.md
```
**Tasks:** 13
**Focus:** Session aggregate, lifecycle events, user ownership

---

## Phase 4: Cycle, Analysis (Parallel with Conversation)

### Feature 1: Component Status Validation
```
/dev features/cycle/component-status-validation.md
```
**Tasks:** 9
**Focus:** Cycle aggregate, component ownership, status transitions

### Feature 2: Analysis Algorithms
```
/dev features/analysis/algorithm-specifications.md
```
**Tasks:** TBD
**Focus:** Pugh matrix, DQ scoring, dominance analysis

### Feature 3: Component Schemas
```
/dev features/proact-types/component-schemas.md
```
**Tasks:** 7
**Focus:** JSON schemas for all 9 PrOACT component types

---

## Phase 5: Integrations

### Feature 1: AI Provider Integration
```
/dev features/integrations/ai-provider-integration.md
```
**Tasks:** 30
**Focus:** AIProvider port, OpenAI/Anthropic adapters, streaming

### Feature 2: Authentication Identity
```
/dev features/integrations/authentication-identity.md
```
**Tasks:** 17
**Focus:** Zitadel OIDC integration, JWT validation

### Feature 3: Membership Access Control
```
/dev features/integrations/membership-access-control.md
```
**Tasks:** 22
**Focus:** AccessChecker port, tier-based gating

### Feature 4: WebSocket Dashboard
```
/dev features/integrations/websocket-dashboard.md
```
**Tasks:** 16
**Focus:** Real-time updates, event broadcasting

---

## Phase 6: Dashboard

### Feature 1: Consequences Table UI
```
/dev features/dashboard/consequences-table-ui.md
```
**Tasks:** 9
**Focus:** Pugh matrix visualization, ratings display

### Feature 2: Frontend Accessibility
```
/dev features/dashboard/frontend-accessibility.md
```
**Tasks:** 16
**Focus:** WCAG 2.1 AA compliance, screen reader support

---

## Phase 7: Remaining Integrations

### Notification Service
```
/dev features/integrations/notification-service.md
```
**Tasks:** 25

### Rate Limiting
```
/dev features/integrations/rate-limiting.md
```
**Tasks:** 25

### Observability
```
/dev features/integrations/observability.md
```
**Tasks:** 25

### Event Versioning
```
/dev features/integrations/event-versioning.md
```
**Tasks:** 35

### Full PrOACT Journey
```
/dev features/integrations/full-proact-journey.md
```
**Tasks:** 26

---

## Execution Order

Process features in this exact order (respects dependencies):

1. `features/membership/subscription-state-machine.md`
2. `features/membership/stripe-webhook-handling.md`
3. `features/session/session-events.md`
4. `features/cycle/component-status-validation.md`
5. `features/proact-types/component-schemas.md`
6. `features/analysis/algorithm-specifications.md`
7. `features/integrations/ai-provider-integration.md`
8. `features/integrations/authentication-identity.md`
9. `features/integrations/membership-access-control.md`
10. `features/integrations/websocket-dashboard.md`
11. `features/dashboard/consequences-table-ui.md`
12. `features/dashboard/frontend-accessibility.md`
13. `features/integrations/notification-service.md`
14. `features/integrations/rate-limiting.md`
15. `features/integrations/observability.md`
16. `features/integrations/event-versioning.md`
17. `features/integrations/full-proact-journey.md`

---

## Ralph Loop Command

```bash
/ralph-loop "/dev features/membership/" --max-iterations 200 --completion-promise "DEV_COMPLETE: All features done"
```

Or process all:
```bash
/ralph-loop "/dev features/" --max-iterations 500 --completion-promise "DEV_COMPLETE: All features done"
```

---

## Success Criteria

- All feature file checkboxes marked [x]
- All tests passing (cargo test)
- All lint checks passing (cargo clippy)
- PRs created for each major feature
- Architecture specifications fully implemented
