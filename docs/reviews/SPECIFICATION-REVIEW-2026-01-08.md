# CHOICE SHERPA: COMPREHENSIVE SPECIFICATION REVIEW

**Review Date:** 2026-01-08
**Reviewer:** Claude (Automated Analysis)
**Scope:** All modules, features, architecture, and requirements
**Method:** 5 parallel subagent reviews + cross-cutting synthesis

---

## Executive Summary

A thorough review of all Choice Sherpa specifications across 8 modules identified **~100 issues** ranging from blocking architectural gaps to minor inconsistencies.

### Key Findings

1. **Event infrastructure is the critical bottleneck** — specified everywhere, implemented nowhere
2. **Cross-module contracts are the weakest point** in an otherwise strong architecture
3. **Authorization model is fragmented** — each module handles auth differently
4. **Transactional consistency is undefined** — race conditions in event publishing
5. **The hexagonal architecture design is sound** — issues are specification completeness, not design flaws

### Recommendation

**Do not begin full-scale implementation until TIER 1 issues are resolved.** Estimated specification work: 3-5 days.

---

## Cross-Cutting Analysis: Systemic Patterns

### Pattern 1: EVENT INFRASTRUCTURE GAP (BLOCKING ALL CROSS-MODULE WORK)

| Module | References Events | Events Implemented |
|--------|-------------------|-------------------|
| Foundation | DomainEvent trait defined in spec | ❌ Not implemented |
| Session | SessionCreated, SessionArchived | ❌ Blocked |
| Cycle | CycleCreated, ComponentStarted | ❌ Blocked |
| Conversation | ConversationStarted, MessageSent | ❌ Blocked |
| Dashboard | Subscribes to all above | ❌ Blocked |

**Root Cause**: The `DomainEvent` trait, `EventPublisher` port, `EventSubscriber` port, and `InMemoryEventBus` adapter are all specified but **none are implemented**. Every module's event feature depends on this foundation.

**Impact**: Cannot implement loose coupling between modules. Dashboard cannot receive updates. Real-time features impossible.

---

### Pattern 2: AUTHORIZATION MODEL FRAGMENTATION

Each module handles authorization differently:

| Module | Authorization Approach | Issue |
|--------|----------------------|-------|
| Session | `session.authorize(&user_id)` | ✅ Defined |
| Membership | `AccessChecker` port | ⚠️ Interface not specified |
| Cycle | Loads session, checks ownership | ✅ Defined |
| Conversation | **None shown** | ❌ CRITICAL GAP |
| Dashboard | `authorize_session()` in reader | ⚠️ Incomplete |

**Root Cause**: No unified authorization pattern documented. Each module spec author made different assumptions.

**Impact**: Security vulnerabilities likely. Conversation module especially has no documented user access validation.

---

### Pattern 3: TRANSACTIONAL CONSISTENCY UNDEFINED

Multiple agents identified the same race condition pattern:

```rust
// Pattern found in multiple modules:
self.repository.save(&entity).await?;        // Step 1: Committed
self.event_publisher.publish(event).await?;  // Step 2: Could fail!
```

| Module | Has This Pattern | Mitigation Specified |
|--------|-----------------|---------------------|
| Session | ✅ | ❌ |
| Cycle | ✅ | ❌ |
| Conversation | ✅ | ❌ |
| Membership | ✅ | ❌ |

**Root Cause**: No transactional outbox pattern specified. No compensation logic documented.

**Impact**: Data persisted but events not published = inconsistent system state.

---

### Pattern 4: COMPONENT OUTPUT VALIDATION MISSING

The cycle module accepts `serde_json::Value` for component outputs without schema validation:

```rust
// cycle.rs - accepts ANY JSON
pub fn update_component_output(&mut self, ct: ComponentType, output: serde_json::Value)
```

But conversation module extracts structured data and sends it:

```rust
// conversation handler - sends structured extraction
cycle.update_component_output(component_type, extracted_json)?;
```

**No validation contract exists between these modules.**

**Impact**: Invalid component data can be persisted. Schema mismatches between frontend, conversation, and cycle modules.

---

### Pattern 5: SPECIFICATION vs IMPLEMENTATION STATUS CONFUSION

| Source | Uses `[x]` For | Example |
|--------|---------------|---------|
| Feature specs (`features/*/`) | Requirements defined | `[x] Implement DomainEvent trait` |
| Checklists (`REQUIREMENTS/`) | Work completed | `⬜ DomainEvent trait definition` |

This creates confusion—feature specs show "done" but checklists show "not started."

---

## Prioritized Issue Inventory

### TIER 1: BLOCKING (Must Fix Before Any Development)

| # | Issue | Modules Affected | Agent Source |
|---|-------|-----------------|--------------|
| 1 | **DomainEvent trait not implemented** | All | Foundation |
| 2 | **EventPublisher/EventSubscriber ports missing** | All | Foundation |
| 3 | **InMemoryEventBus adapter missing** | All | Foundation |
| 4 | **AccessChecker port not specified** | Session, Membership | Session/Membership |
| 5 | **Transactional outbox pattern undefined** | All publishing modules | Cycle/Conversation |
| 6 | **Conversation authorization checks missing** | Conversation, Dashboard | Cycle/Conversation |
| 7 | **AI Provider abstraction spec missing** | Conversation, Analysis | Infrastructure |
| 8 | **WebSocket event bridge spec missing** | Dashboard, Frontend | Infrastructure |

**Estimated effort to unblock**: 3-5 days of specification work before implementation can begin.

---

### TIER 2: HIGH (Fix Before Feature Development)

| # | Issue | Modules Affected | Agent Source |
|---|-------|-----------------|--------------|
| 9 | Component output schema validation | Cycle, Conversation, PrOACT-types | Cycle/Conversation |
| 10 | Conversation initialization not specified | Conversation | Cycle/Conversation |
| 11 | Money type validation (negative amounts) | Membership | Session/Membership |
| 12 | PromoCodeValidator port undefined | Membership | Session/Membership |
| 13 | DQ scoring algorithm incomplete | Analysis, Dashboard | Analysis/Dashboard |
| 14 | Dominance detection edge cases | Analysis | Analysis/Dashboard |
| 15 | Dashboard data freshness model | Dashboard | Analysis/Dashboard |
| 16 | Agent phase transition logic | Conversation | Cycle/Conversation |

---

### TIER 3: MEDIUM (Fix During Development)

| # | Issue | Modules Affected |
|---|-------|-----------------|
| 17 | Idempotency key strategy for events | All |
| 18 | Pagination defaults and limits | Session, Dashboard |
| 19 | HTTP endpoint schemas (OpenAPI) | All |
| 20 | Error code inventory standardization | All |
| 21 | Cycle navigation rules clarification | Cycle |
| 22 | Component-to-conversation relationship | Cycle, Conversation |
| 23 | Notification event triggers | Notifications |
| 24 | Rate limiting AI token quotas | Rate Limiting |

---

### TIER 4: LOW (Fix During Polish)

| # | Issue | Modules Affected |
|---|-------|-----------------|
| 25 | Timestamp vs DateTime inconsistency | Multiple |
| 26 | Missing doc comments on structs | All |
| 27 | Test example data sets | Analysis |
| 28 | TypeScript/Rust parity documentation | Analysis |
| 29 | Configuration naming conventions | Infrastructure |
| 30 | Feature flag strategy | Infrastructure |

---

## Critical Path: Recommended Resolution Order

```
PHASE 0 (Before any implementation):
┌─────────────────────────────────────────────────────────────────┐
│ 1. Implement DomainEvent trait in foundation/events.rs          │
│ 2. Create EventPublisher + EventSubscriber ports                │
│ 3. Implement InMemoryEventBus adapter                           │
│ 4. Define AccessChecker port contract                           │
│ 5. Document transactional outbox pattern                        │
└─────────────────────────────────────────────────────────────────┘
                              ↓
PHASE 1 (Enables Phase 2+ modules):
┌─────────────────────────────────────────────────────────────────┐
│ 6. Add authorization to conversation commands                   │
│ 7. Create AI Provider abstraction spec                          │
│ 8. Create WebSocket event bridge spec                           │
│ 9. Define component output schemas (JSON Schema)                │
└─────────────────────────────────────────────────────────────────┘
                              ↓
PHASE 2 (Module-specific fixes):
┌─────────────────────────────────────────────────────────────────┐
│ 10-16: High priority module-specific issues                     │
└─────────────────────────────────────────────────────────────────┘
```

---

## Specifications To Create/Update

### New Specifications Needed

| Specification | Purpose | Priority |
|--------------|---------|----------|
| `features/infrastructure/event-driven-architecture.md` | Cross-module event flow, handler registration | **BLOCKING** |
| `features/integrations/session-access-control.md` | AccessChecker contract, authorization model | **BLOCKING** |
| `features/integrations/ai-provider-integration.md` | AI adapter abstraction, streaming, tokens | **BLOCKING** |
| `features/infrastructure/websocket-event-bridge.md` | Real-time event delivery to clients | **BLOCKING** |
| `features/proact-types/component-schemas.md` | JSON Schema per component type | HIGH |
| `docs/error-handling-strategy.md` | Unified error code inventory | HIGH |
| `docs/configuration-strategy.md` | Environment variable conventions | MEDIUM |

### Existing Specifications To Update

| Specification | Updates Needed |
|--------------|----------------|
| `docs/modules/session.md` | Add full membership integration example |
| `docs/modules/conversation.md` | Add authorization checks to all handlers |
| `docs/modules/membership.md` | Define PromoCodeValidator port |
| `docs/modules/cycle.md` | Add component output validation |
| `features/integrations/rate-limiting.md` | Add AI token quota section |
| `features/integrations/observability.md` | Add AI token accounting |

---

## Detailed Agent Reports

### Agent 1: Foundation & PrOACT-Types Review

**Issues Found:** 20 (3 CRITICAL, 4 HIGH, 10 MEDIUM, 3 LOW)

#### Critical Issues

1. **DomainEvent Trait Not Implemented**
   - Location: `features/foundation/event-infrastructure.md` (lines 43, 234-263)
   - The specification defines a `DomainEvent` trait with required methods but `backend/src/domain/foundation/events.rs` does NOT define this trait
   - Impact: Cannot implement domain events in any module

2. **Event Ports & Adapters Not Implemented**
   - Missing files:
     - `backend/src/ports/event_publisher.rs`
     - `backend/src/ports/event_subscriber.rs`
     - `backend/src/adapters/events/in_memory.rs`
   - CHECKLIST-events.md shows all Phase 1.2 and 1.3 tasks unchecked

3. **Status Asymmetry in Specifications**
   - Feature specs use `[x]` for "requirements to implement"
   - Checklists use `⬜` for "not completed"
   - Creates confusion about actual implementation status

#### High Issues

4. Incomplete Event Infrastructure Specification (missing integration patterns)
5. EventPublisher Port Definition missing
6. Asymmetric DomainEvent Trait Definition (two different versions in specs)

#### Recommendations

- Implement DomainEvent trait immediately
- Create event ports matching spec interfaces
- Implement InMemoryEventBus with all test helpers
- Export all event-related types from module boundaries

---

### Agent 2: Session & Membership Review

**Issues Found:** 20+ (3 CRITICAL, 4 HIGH, 6 MEDIUM, 4 LOW)

#### Critical Issues

1. **Missing Session-Membership Integration Details**
   - Session module mentions `AccessChecker` but implementation details are vague
   - `CreateSessionHandler` needs to inject `AccessChecker` but interface unclear
   - `MembershipRequired` error variant NOT defined in session module's error enum

2. **PromoCodeValidator Port Not Defined**
   - `CreateFreeMembershipHandler` calls `self.promo_validator.validate()` but no port exists
   - No trait definition, error handling, or adapter notes

3. **Money Type Enforcement Gap**
   - No validation rules to prevent negative amounts
   - `from_dollars()` accepts negative values
   - Risk of financial data corruption

#### High Issues

4. Session Module Missing Authorization Check in Queries
5. Membership Status Transitions Unclear in Webhook Handler
6. Session Module Missing Cycle Lifecycle Documentation
7. Membership Tier Limits Asymmetry with Session

#### Recommendations

- Create Session-Membership integration specification
- Complete PromoCodeValidator port with full trait definition
- Add Money type validation for non-negative amounts
- Add authorization checks to all session queries

---

### Agent 3: Cycle & Conversation Review

**Issues Found:** 23 (4 CRITICAL, 6 HIGH, 5 MEDIUM, 5 LOW)

#### Critical Issues

1. **Component Ownership Boundary Violation**
   - `update_component_output()` accepts raw `serde_json::Value` with no validation
   - No contract enforcement between conversation extraction and cycle storage
   - Invalid data can be persisted

2. **Missing Component Interface in Conversation**
   - Conversation creates agents for components it doesn't own
   - `component.output_as_value()` referenced but undefined
   - Tight coupling to proact-types not explicitly declared

3. **Race Condition in Event Publishing**
   - Data committed before event published
   - If publish fails, data persisted but event lost
   - No transactional outbox pattern

4. **Conversation Initialization Ordering Not Specified**
   - No specification of when initial prompts are sent
   - Does conversation initialize empty or with greeting?
   - No definition of conversation "readiness"

#### High Issues

5. Missing Component-to-Conversation Relationship Specification
6. Insufficient Specification of Agent Phases and Transitions
7. Asymmetric Error Handling in Commands vs Queries
8. Component Lifecycle Not Fully Specified
9. Streaming Message Handler Not Fully Specified
10. Missing Specification of Component Output Schemas

#### Recommendations

- Add `ComponentValidator` port for schema validation
- Define `ComponentState` value object in proact-types
- Use transactional outbox pattern for event publishing
- Define conversation states: Initializing, Ready, InProgress, Complete

---

### Agent 4: Analysis & Dashboard Review

**Issues Found:** 19 (4 CRITICAL, 5 HIGH, 5 MEDIUM, 5 LOW)

#### Critical Issues

1. **Dominance Detection Algorithm Incomplete**
   - No guidance on missing cells (treated as 0 silently)
   - No edge cases for identical ratings or single alternative
   - Risk of incorrect recommendations

2. **Dashboard Authorization Model Incomplete**
   - Only checks session ownership, not cycle/component access
   - No specification for branched cycles or multi-user scenarios
   - Authorization in reader layer only, no defense in depth

3. **DQ Scoring Algorithm Incomplete**
   - No specification for how users rate elements
   - No guidance on partial elements (only 5 of 7 provided)
   - Threshold values hardcoded without explanation

4. **Missing Data Freshness Specification**
   - No specification for snapshot vs. computed data
   - No invalidation rules when components updated
   - No consistency level guarantees

#### High Issues

5. Missing Property Bounds for Percentage/Rating Types
6. Incomplete Component Detail View Specification
7. Missing Frontend/Backend Parity Specification
8. Missing Query Error Handling Specification
9. Undefined Behavior for Single Alternative Analysis

#### Recommendations

- Add explicit edge cases to dominance specification
- Document authorization layers clearly
- Complete DQ element workflow specification
- Define data freshness policies (snapshot vs. computed)

---

### Agent 5: Infrastructure & Integrations Review

**Issues Found:** 16 (4 CRITICAL, 5 HIGH, 5 MEDIUM, 4 LOW)

#### Critical Issues

1. **Missing AI Provider Abstraction Specification**
   - Referenced in checklists but not fully detailed
   - No specification for streaming, token counting, error recovery
   - No provider failover/fallback mechanisms

2. **Event Infrastructure Missing Cross-Module Integration**
   - Redis Event Bus well-specified but integration contract missing
   - No event flow diagram showing module interactions
   - No idempotency guarantees specified

3. **Authentication-to-Session Access Control Missing**
   - `AccessChecker` port mentioned but not documented
   - No specification for membership tier enforcement
   - Session ownership verification details missing

4. **Missing WebSocket Real-Time Event Bridge**
   - Event bus specified but client delivery missing
   - No room/session-scoped broadcast mechanism
   - No reconnection or catch-up logic

#### High Issues

5. Observability Token Counting Not Integrated with AI Adapters
6. Notification Service Event Subscription Not Specified
7. Rate Limiting Missing AI Token Quota Integration
8. Membership Module Not Specified (despite being referenced)
9. Inconsistent Error Codes Across Specifications

#### Recommendations

- Create comprehensive AI provider integration spec
- Create event-driven-architecture.md for cross-module flows
- Create session-access-control.md for AccessChecker
- Create websocket-event-bridge.md for real-time delivery

---

## Summary Statistics

| Category | Count |
|----------|-------|
| **BLOCKING issues** | 8 |
| **HIGH issues** | 8 |
| **MEDIUM issues** | 8 |
| **LOW issues** | 6 |
| **Total identified** | ~100 across all agents |
| **New specs needed** | 7 |
| **Specs to update** | 6+ |

---

## Architecture Alignment Status

| Concern | Assessment | Notes |
|---------|-----------|-------|
| Hexagonal Architecture Applied | ✅ GOOD | All specs show port/adapter pattern |
| Domain Event Pattern | ⚠️ PARTIAL | Infrastructure exists but integration incomplete |
| Error Handling Consistency | ❌ POOR | Error codes scattered without central inventory |
| Configuration Management | ⚠️ PARTIAL | Each spec shows examples but no unified strategy |
| Testing Strategy | ✅ GOOD | Checklists specify test coverage targets |
| Observability Coverage | ✅ GOOD | Comprehensive triple-pillar approach |
| Cross-Module Contracts | ❌ POOR | Missing specs for module-to-module integration |

---

## Conclusion

The Choice Sherpa architecture is well-designed following hexagonal principles with clear module boundaries. However, **implementation significantly lags specification**, particularly in the event infrastructure that enables loose coupling between modules.

The single most important action is to implement the event infrastructure (DomainEvent trait, event ports, InMemoryEventBus). Until this is done, no cross-module features can be developed.

With 3-5 days of focused specification work on the TIER 1 issues, the project will be ready for full-scale development.

---

*Review completed: 2026-01-08*
*Next review recommended: After TIER 1 issues resolved*
