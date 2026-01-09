# CHOICE SHERPA: SPECIFICATION REMEDIATION PLAN

**Created:** 2026-01-08
**Based On:** SPECIFICATION-REVIEW-2026-01-08.md
**Objective:** Bring all specifications to development-ready state
**Scope:** Documentation and specification work only (no code implementation)

---

## Executive Summary

This plan addresses all ~100 specification issues identified in the comprehensive review, organized into 8 work streams that can be executed in parallel where dependencies allow. Each work stream produces specific deliverables with clear acceptance criteria.

**Total Deliverables:** 47 specification updates/additions
**Estimated Effort:** 5-7 days of focused specification work
**Critical Path:** Work Streams 1-3 must complete before development begins

---

## Work Stream Overview

| # | Work Stream | Priority | Deliverables | Dependencies |
|---|-------------|----------|--------------|--------------|
| 1 | Event Infrastructure Completion | **BLOCKING** | 6 | None |
| 2 | Authorization & Access Control | **BLOCKING** | 5 | None |
| 3 | Cross-Module Contracts | **BLOCKING** | 7 | WS1, WS2 |
| 4 | Component & Conversation Specs | HIGH | 8 | WS1 |
| 5 | Analysis & Dashboard Algorithms | HIGH | 6 | None |
| 6 | Membership & Payment Details | HIGH | 5 | WS2 |
| 7 | Infrastructure Integration Gaps | MEDIUM | 6 | WS1 |
| 8 | Consistency & Standards | LOW | 4 | All others |

---

## WORK STREAM 1: Event Infrastructure Completion

**Priority:** BLOCKING
**Issues Addressed:** #1, #2, #3, #5, #17
**Owner:** TBD
**Estimated Effort:** 1 day

### Background

The event infrastructure is specified in `features/foundation/event-infrastructure.md` but has critical gaps that block all cross-module communication. The DomainEvent trait is referenced throughout but not fully specified.

### Deliverable 1.1: Complete DomainEvent Trait Specification

**File:** `features/foundation/event-infrastructure.md`
**Action:** UPDATE (lines 43, 118-127, 234-263)

**Content to Add:**

```markdown
## DomainEvent Trait (Complete Specification)

### Trait Definition

\`\`\`rust
use std::any::Any;
use serde::Serialize;

/// Trait that all domain events must implement.
/// Provides routing, correlation, and serialization capabilities.
pub trait DomainEvent: Send + Sync + Any {
    /// Returns the event type string for routing (e.g., "session.created")
    /// Convention: lowercase, dot-separated: "{aggregate}.{action}"
    fn event_type(&self) -> &'static str;

    /// Returns the aggregate ID this event pertains to
    fn aggregate_id(&self) -> String;

    /// Returns the aggregate type (e.g., "session", "cycle")
    fn aggregate_type(&self) -> &'static str;

    /// Returns when this event occurred
    fn occurred_at(&self) -> Timestamp;

    /// Returns the unique event ID for deduplication
    fn event_id(&self) -> EventId;

    /// Converts the event to an envelope for transport
    /// Default implementation provided
    fn to_envelope(&self) -> EventEnvelope
    where
        Self: Serialize + Sized,
    {
        EventEnvelope::new(
            self.event_id(),
            self.event_type().to_string(),
            self.aggregate_id(),
            self.aggregate_type().to_string(),
            self.occurred_at(),
            serde_json::to_value(self).expect("Event must be serializable"),
        )
    }
}
\`\`\`

### Event Type Naming Convention

| Pattern | Example | Usage |
|---------|---------|-------|
| `{module}.{entity}.{action}` | `session.session.created` | Entity-level events |
| `{module}.{action}` | `session.created` | Aggregate-level events (preferred) |

**Canonical Event Types:**
- `session.created`, `session.archived`, `session.renamed`
- `cycle.created`, `cycle.archived`, `cycle.branched`
- `component.started`, `component.completed`, `component.revised`
- `conversation.started`, `conversation.message_sent`
- `membership.created`, `membership.upgraded`, `membership.cancelled`

### Implementation Example

\`\`\`rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreated {
    pub event_id: EventId,
    pub session_id: SessionId,
    pub user_id: UserId,
    pub title: String,
    pub occurred_at: Timestamp,
}

impl DomainEvent for SessionCreated {
    fn event_type(&self) -> &'static str { "session.created" }
    fn aggregate_id(&self) -> String { self.session_id.to_string() }
    fn aggregate_type(&self) -> &'static str { "session" }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
    fn event_id(&self) -> EventId { self.event_id.clone() }
}
\`\`\`
```

**Acceptance Criteria:**
- [ ] DomainEvent trait has all 6 required methods specified
- [ ] Default `to_envelope()` implementation shown
- [ ] Event type naming convention documented
- [ ] All canonical event types listed
- [ ] Implementation example provided for one event

---

### Deliverable 1.2: Transactional Outbox Pattern Specification

**File:** `features/foundation/event-infrastructure.md`
**Action:** ADD new section after "Event Bus Ports"

**Content to Add:**

```markdown
## Transactional Consistency

### Problem: Event Publishing Race Condition

When command handlers persist data then publish events, a failure in publishing leaves the system inconsistent:

\`\`\`rust
// PROBLEMATIC PATTERN (DO NOT USE)
self.repository.save(&entity).await?;     // ✅ Committed
self.event_publisher.publish(event).await?;  // ❌ Could fail!
// Result: Data persisted, event lost
\`\`\`

### Solution: Transactional Outbox Pattern

All events are stored in an outbox table within the same database transaction as the domain state change. A separate process publishes events from the outbox.

\`\`\`
┌─────────────────────────────────────────────────────────────────┐
│                    Command Handler                               │
│                                                                  │
│   BEGIN TRANSACTION                                              │
│   │                                                              │
│   ├── 1. repository.save(entity)                                 │
│   │                                                              │
│   ├── 2. outbox.store(events)    ←── Same transaction!          │
│   │                                                              │
│   COMMIT                                                         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ (async, separate process)
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Outbox Publisher                               │
│                                                                  │
│   POLL: SELECT * FROM event_outbox WHERE published_at IS NULL   │
│   │                                                              │
│   ├── event_bus.publish(event)                                   │
│   │                                                              │
│   └── UPDATE event_outbox SET published_at = NOW()               │
└─────────────────────────────────────────────────────────────────┘
\`\`\`

### Outbox Table Schema

\`\`\`sql
CREATE TABLE event_outbox (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL UNIQUE,
    event_type VARCHAR(255) NOT NULL,
    aggregate_id VARCHAR(255) NOT NULL,
    aggregate_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    metadata JSONB NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ,  -- NULL until published

    -- Indexing for polling
    INDEX idx_outbox_unpublished (published_at) WHERE published_at IS NULL
);
\`\`\`

### Command Handler Pattern (Correct)

\`\`\`rust
impl CreateSessionHandler {
    pub async fn handle(&self, cmd: CreateSessionCommand) -> Result<SessionId, CommandError> {
        let session = Session::new(cmd.user_id, cmd.title)?;
        let event = SessionCreated::from(&session);

        // Transaction spans both operations
        self.unit_of_work
            .execute(|tx| async move {
                self.session_repo.save_with_tx(&session, tx).await?;
                self.event_outbox.store_with_tx(&[event.to_envelope()], tx).await?;
                Ok(session.id)
            })
            .await
    }
}
\`\`\`

### Outbox Publisher Service

\`\`\`rust
pub struct OutboxPublisher {
    outbox_repo: Arc<dyn EventOutboxRepository>,
    event_bus: Arc<dyn EventPublisher>,
    poll_interval: Duration,
    batch_size: usize,
}

impl OutboxPublisher {
    pub async fn run(&self) -> Result<(), Error> {
        loop {
            let events = self.outbox_repo
                .fetch_unpublished(self.batch_size)
                .await?;

            for event in events {
                match self.event_bus.publish(event.clone()).await {
                    Ok(_) => {
                        self.outbox_repo.mark_published(event.event_id).await?;
                    }
                    Err(e) => {
                        // Log and continue - will retry on next poll
                        tracing::warn!("Failed to publish event {}: {}", event.event_id, e);
                    }
                }
            }

            tokio::time::sleep(self.poll_interval).await;
        }
    }
}
\`\`\`

### Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `OUTBOX_POLL_INTERVAL_MS` | 100 | How often to check for unpublished events |
| `OUTBOX_BATCH_SIZE` | 100 | Max events to publish per poll cycle |
| `OUTBOX_RETENTION_DAYS` | 7 | How long to keep published events |
```

**Acceptance Criteria:**
- [ ] Race condition problem clearly explained
- [ ] Outbox pattern diagram and explanation provided
- [ ] Database schema for outbox table specified
- [ ] Correct command handler pattern shown
- [ ] Outbox publisher service specified
- [ ] Configuration options documented

---

### Deliverable 1.3: Idempotency Specification

**File:** `features/foundation/event-infrastructure.md`
**Action:** ADD new section

**Content to Add:**

```markdown
## Event Idempotency

### Deduplication Strategy

Events may be delivered more than once due to:
- Network retries
- Outbox publisher restarts
- Consumer crashes before acknowledgment

All event handlers MUST be idempotent.

### Idempotency Key

The `event_id` (UUID) is the idempotency key. Handlers track processed event IDs.

\`\`\`rust
pub struct IdempotentHandler<H: EventHandler> {
    inner: H,
    processed_events: Arc<dyn ProcessedEventStore>,
}

impl<H: EventHandler> EventHandler for IdempotentHandler<H> {
    async fn handle(&self, envelope: EventEnvelope) -> Result<(), HandlerError> {
        // Check if already processed
        if self.processed_events.contains(&envelope.event_id).await? {
            tracing::debug!("Skipping duplicate event: {}", envelope.event_id);
            return Ok(());
        }

        // Process event
        self.inner.handle(envelope.clone()).await?;

        // Mark as processed
        self.processed_events.mark_processed(&envelope.event_id).await?;

        Ok(())
    }
}
\`\`\`

### Processed Events Storage

\`\`\`sql
CREATE TABLE processed_events (
    event_id UUID PRIMARY KEY,
    handler_name VARCHAR(255) NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Composite key for handler-specific dedup
    UNIQUE(event_id, handler_name)
);

-- Cleanup old entries (events older than retention period)
CREATE INDEX idx_processed_events_cleanup ON processed_events(processed_at);
\`\`\`

### Handler Registration with Idempotency

\`\`\`rust
// All handlers should be wrapped with idempotency
event_bus.subscribe(
    "session.created",
    IdempotentHandler::new(
        DashboardUpdateHandler::new(dashboard_repo),
        processed_events_store.clone(),
    ),
);
\`\`\`

### Idempotency Guarantees

| Guarantee | Level | Notes |
|-----------|-------|-------|
| At-least-once delivery | ✅ Guaranteed | Events will be delivered |
| At-most-once processing | ✅ Guaranteed | Via idempotency wrapper |
| Exactly-once semantics | ✅ Effective | Combination of above |
| Ordering within aggregate | ⚠️ Best effort | Use `occurred_at` for ordering |
| Global ordering | ❌ Not guaranteed | Events may arrive out of order |
```

**Acceptance Criteria:**
- [ ] Deduplication strategy documented
- [ ] IdempotentHandler wrapper specified
- [ ] Processed events storage schema provided
- [ ] Handler registration pattern shown
- [ ] Guarantee levels clearly stated

---

### Deliverable 1.4: Update Event Infrastructure Tasks

**File:** `features/foundation/event-infrastructure.md`
**Action:** UPDATE Tasks section (line 37-49)

**Change:**
```markdown
## Tasks

- [x] Create backend project structure with Cargo.toml and src directory
- [x] Implement EventId value object with UUID generation and serialization
- [x] Implement EventMetadata struct with correlation, causation, user, trace IDs
- [x] Implement EventEnvelope struct with all fields and builder methods
- [ ] Implement DomainEvent trait with event_type, aggregate_id, aggregate_type, occurred_at, event_id methods
- [ ] Implement default to_envelope() method on DomainEvent trait
- [x] Implement EventPublisher port trait with publish and publish_all methods
- [x] Implement EventSubscriber and EventHandler port traits
- [x] Implement InMemoryEventBus adapter with test helper methods
- [ ] Implement EventOutboxRepository port for transactional outbox
- [ ] Implement ProcessedEventStore port for idempotency tracking
- [ ] Implement IdempotentHandler wrapper
- [ ] Implement OutboxPublisher background service
- [x] Add unit tests for EventId, EventEnvelope, EventMetadata
- [x] Add unit tests for InMemoryEventBus publish, subscribe, and handler invocation
- [ ] Add unit tests for idempotency behavior
- [ ] Add integration tests for outbox pattern
```

**Acceptance Criteria:**
- [ ] All new tasks added to task list
- [ ] Existing completed tasks remain marked [x]
- [ ] New tasks marked [ ] (not started)

---

### Deliverable 1.5: Synchronize CHECKLIST-events.md

**File:** `REQUIREMENTS/CHECKLIST-events.md`
**Action:** UPDATE to match feature spec and add new phases

**Content to Add (after Phase 1.4):**

```markdown
### 1.5 Transactional Outbox

| Task | Status | File | Tests |
|------|--------|------|-------|
| `EventOutboxRepository` port trait | [ ] | `backend/src/ports/event_outbox.rs` | |
| `event_outbox` table migration | [ ] | `backend/migrations/` | |
| PostgreSQL outbox adapter | [ ] | `backend/src/adapters/events/postgres_outbox.rs` | [ ] |
| `OutboxPublisher` service | [ ] | `backend/src/adapters/events/outbox_publisher.rs` | [ ] |
| Outbox cleanup job | [ ] | `backend/src/adapters/events/outbox_cleanup.rs` | |

### 1.6 Idempotency Infrastructure

| Task | Status | File | Tests |
|------|--------|------|-------|
| `ProcessedEventStore` port trait | [ ] | `backend/src/ports/processed_events.rs` | |
| `processed_events` table migration | [ ] | `backend/migrations/` | |
| PostgreSQL processed events adapter | [ ] | `backend/src/adapters/events/postgres_processed.rs` | [ ] |
| `IdempotentHandler` wrapper | [ ] | `backend/src/adapters/events/idempotent_handler.rs` | [ ] |
| Idempotency integration test | [ ] | `backend/tests/integration/idempotency_test.rs` | |
```

**Acceptance Criteria:**
- [ ] Phase 1.5 (Transactional Outbox) added with all tasks
- [ ] Phase 1.6 (Idempotency Infrastructure) added with all tasks
- [ ] All tasks have correct file paths
- [ ] Test columns populated where applicable

---

### Deliverable 1.6: Create Cross-Module Event Flow Diagram

**File:** `features/infrastructure/event-flow-architecture.md`
**Action:** CREATE new file

**Content:**

```markdown
# Event Flow Architecture

**Type:** Cross-Cutting Reference
**Priority:** P0 (Required for understanding module integration)

> Visual and textual reference for how domain events flow between modules in Choice Sherpa.

---

## Event Flow Diagram

\`\`\`
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                              EVENT PRODUCERS                                          │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                       │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐           │
│  │   Session   │    │    Cycle    │    │Conversation │    │ Membership  │           │
│  │   Module    │    │   Module    │    │   Module    │    │   Module    │           │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘    └──────┬──────┘           │
│         │                  │                  │                  │                   │
│         ▼                  ▼                  ▼                  ▼                   │
│  SessionCreated     CycleCreated      ConversationStarted  MembershipCreated        │
│  SessionArchived    CycleBranched     MessageSent          MembershipUpgraded       │
│  SessionRenamed     ComponentStarted  AIResponseReceived   MembershipCancelled      │
│                     ComponentCompleted DataExtracted                                 │
│                     CycleArchived                                                    │
│                                                                                       │
└─────────────────────────────────────────────────────────────────────────────────────┘
                                        │
                                        │ EventOutbox (transactional)
                                        ▼
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                              EVENT BUS                                                │
│                                                                                       │
│                    ┌─────────────────────────────────┐                               │
│                    │     InMemoryEventBus (test)     │                               │
│                    │     RedisEventBus (production)  │                               │
│                    └─────────────────────────────────┘                               │
│                                                                                       │
└─────────────────────────────────────────────────────────────────────────────────────┘
                                        │
                                        │ EventSubscriber + IdempotentHandler
                                        ▼
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                              EVENT CONSUMERS                                          │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                       │
│  ┌─────────────────────────────────────────────────────────────────────────────┐    │
│  │                           Dashboard Module                                   │    │
│  │                                                                              │    │
│  │   Subscribes to: ALL events (for read model updates)                        │    │
│  │   Handlers:                                                                  │    │
│  │     - SessionCreated → Create dashboard entry                               │    │
│  │     - CycleCreated → Add cycle to session dashboard                         │    │
│  │     - ComponentCompleted → Update progress, trigger analysis                │    │
│  │     - DataExtracted → Update component detail view                          │    │
│  └─────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                       │
│  ┌─────────────────────────────────────────────────────────────────────────────┐    │
│  │                         Conversation Module                                  │    │
│  │                                                                              │    │
│  │   Subscribes to: ComponentStarted                                           │    │
│  │   Handlers:                                                                  │    │
│  │     - ComponentStarted → Initialize conversation for component              │    │
│  └─────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                       │
│  ┌─────────────────────────────────────────────────────────────────────────────┐    │
│  │                          Analysis Module                                     │    │
│  │                                                                              │    │
│  │   Subscribes to: ComponentCompleted (Consequences, DecisionQuality)         │    │
│  │   Handlers:                                                                  │    │
│  │     - ComponentCompleted(Consequences) → Compute Pugh scores                │    │
│  │     - ComponentCompleted(DecisionQuality) → Compute DQ overall              │    │
│  └─────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                       │
│  ┌─────────────────────────────────────────────────────────────────────────────┐    │
│  │                         WebSocket Bridge                                     │    │
│  │                                                                              │    │
│  │   Subscribes to: ALL events (for real-time client updates)                  │    │
│  │   Handlers:                                                                  │    │
│  │     - * → Filter by session, broadcast to connected clients                 │    │
│  └─────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                       │
└─────────────────────────────────────────────────────────────────────────────────────┘
\`\`\`

---

## Event Subscription Matrix

| Event | Dashboard | Conversation | Analysis | WebSocket | Notifications |
|-------|-----------|--------------|----------|-----------|---------------|
| SessionCreated | ✅ | | | ✅ | |
| SessionArchived | ✅ | | | ✅ | |
| CycleCreated | ✅ | | | ✅ | |
| CycleBranched | ✅ | | | ✅ | |
| ComponentStarted | ✅ | ✅ | | ✅ | |
| ComponentCompleted | ✅ | | ✅* | ✅ | ✅** |
| ConversationStarted | ✅ | | | ✅ | |
| MessageSent | ✅ | | | ✅ | |
| DataExtracted | ✅ | | | ✅ | |
| MembershipCreated | | | | | ✅ |
| MembershipUpgraded | | | | | ✅ |

*Analysis only subscribes to Consequences and DecisionQuality component completions
**Notifications sent when cycle reaches certain milestones

---

## Handler Registration Example

\`\`\`rust
pub fn register_event_handlers(
    event_bus: &mut dyn EventBus,
    processed_store: Arc<dyn ProcessedEventStore>,
    dashboard_repo: Arc<dyn DashboardRepository>,
    conversation_repo: Arc<dyn ConversationRepository>,
    ws_broadcaster: Arc<dyn WebSocketBroadcaster>,
) {
    // Dashboard handlers (subscribe to all)
    event_bus.subscribe_all(IdempotentHandler::new(
        DashboardUpdateHandler::new(dashboard_repo.clone()),
        processed_store.clone(),
    ));

    // Conversation initialization
    event_bus.subscribe(
        "component.started",
        IdempotentHandler::new(
            ConversationInitHandler::new(conversation_repo),
            processed_store.clone(),
        ),
    );

    // WebSocket bridge (all events, session-filtered)
    event_bus.subscribe_all(
        WebSocketBridgeHandler::new(ws_broadcaster),
        // No idempotency needed - broadcasts are stateless
    );
}
\`\`\`

---

## Event Ordering Guarantees

| Scope | Guarantee | Implementation |
|-------|-----------|----------------|
| Within aggregate | Ordered by `occurred_at` | Handlers should use timestamp for ordering |
| Cross-aggregate | No ordering | Events may arrive in any order |
| Within handler | Sequential | Single handler processes events sequentially |
| Across handlers | Parallel | Different handlers may run concurrently |

---

## Failure Scenarios

### Scenario 1: Handler Fails

\`\`\`
Event Published → Handler Throws → Event remains in outbox → Retry on next poll
\`\`\`

Handler failures do NOT block other handlers. Each handler processes independently.

### Scenario 2: Outbox Publisher Crashes

\`\`\`
Events in outbox → Publisher crashes → Publisher restarts → Resumes from unpublished events
\`\`\`

No events lost. Transactional outbox guarantees durability.

### Scenario 3: Duplicate Delivery

\`\`\`
Event delivered → Handler processes → Network timeout → Event redelivered → Idempotency check → Skipped
\`\`\`

IdempotentHandler prevents duplicate processing.
```

**Acceptance Criteria:**
- [ ] Complete event flow diagram showing all modules
- [ ] Event subscription matrix documenting all subscriptions
- [ ] Handler registration example code
- [ ] Event ordering guarantees documented
- [ ] Failure scenarios and recovery documented

---

## WORK STREAM 2: Authorization & Access Control

**Priority:** BLOCKING
**Issues Addressed:** #4, #6, Authorization Model Fragmentation
**Owner:** TBD
**Estimated Effort:** 1 day

### Background

Authorization is implemented inconsistently across modules. The Session module has `session.authorize()`, but Conversation has none documented. The `AccessChecker` port is referenced but not fully specified.

### Deliverable 2.1: Unified Authorization Model Document

**File:** `docs/authorization-model.md`
**Action:** CREATE new file

**Content:**

```markdown
# Choice Sherpa Authorization Model

**Version:** 1.0
**Last Updated:** 2026-01-08

---

## Overview

Choice Sherpa uses a resource-based authorization model where access is determined by ownership and membership tier. This document defines the canonical authorization patterns used across all modules.

---

## Core Principles

1. **Session Ownership**: Users own their sessions. All resources within a session inherit ownership.
2. **Hierarchical Access**: Session → Cycles → Components → Conversations
3. **Membership Gating**: Some operations require specific membership tiers
4. **Defense in Depth**: Authorization checked at multiple layers

---

## Resource Hierarchy

\`\`\`
User
  └── Membership
        └── Tier (Free/Monthly/Annual)
              └── Limits (sessions, cycles, AI features, exports)
  └── Sessions (owns)
        └── Cycles (owns via session)
              └── Components (owns via cycle)
                    └── Conversations (owns via component)
\`\`\`

---

## Authorization Checks by Module

### Session Module

| Operation | Check | Implementation |
|-----------|-------|----------------|
| CreateSession | User has active membership | AccessChecker.can_create_session(user_id) |
| GetSession | User owns session | session.user_id == user_id |
| ListSessions | Filter by user | query WHERE user_id = $1 |
| UpdateSession | User owns session | session.user_id == user_id |
| ArchiveSession | User owns session | session.user_id == user_id |

### Cycle Module

| Operation | Check | Implementation |
|-----------|-------|----------------|
| CreateCycle | User owns parent session + within limits | session.authorize(user_id) + AccessChecker.can_create_cycle(user_id) |
| GetCycle | User owns parent session | Load session, session.authorize(user_id) |
| UpdateCycle | User owns parent session | Load session, session.authorize(user_id) |
| BranchCycle | User owns parent session | Load session, session.authorize(user_id) |
| ArchiveCycle | User owns parent session | Load session, session.authorize(user_id) |

### Conversation Module

| Operation | Check | Implementation |
|-----------|-------|----------------|
| GetConversation | User owns parent component's cycle's session | Load cycle → session, session.authorize(user_id) |
| SendMessage | User owns parent component's cycle's session | Load cycle → session, session.authorize(user_id) |
| StreamMessage | User owns parent component's cycle's session | Load cycle → session, session.authorize(user_id) |

### Dashboard Module

| Operation | Check | Implementation |
|-----------|-------|----------------|
| GetOverview | User owns session | reader.authorize_session(session_id, user_id) |
| GetComponentDetail | User owns session | Load session, session.authorize(user_id) |
| CompareCycles | User owns both sessions | Check both sessions owned |

---

## Authorization Errors

\`\`\`rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthorizationError {
    #[error("User does not own this resource")]
    NotOwner,

    #[error("Membership required for this operation")]
    MembershipRequired,

    #[error("Membership tier insufficient")]
    TierInsufficient { required: Tier, actual: Tier },

    #[error("Resource limit exceeded")]
    LimitExceeded { resource: String, limit: u32, current: u32 },

    #[error("Resource not found")]
    NotFound,
}
```

**Acceptance Criteria:**
- [ ] Core authorization principles documented
- [ ] Resource hierarchy diagram provided
- [ ] All modules' authorization checks specified
- [ ] Error types defined
- [ ] Implementation patterns shown

---

### Deliverable 2.2: Complete AccessChecker Port Specification

**File:** `features/integrations/membership-access-control.md`
**Action:** UPDATE (add AccessChecker port details)

**Content to Add:**

```markdown
## AccessChecker Port

The AccessChecker port provides a unified interface for membership-based access control. It is implemented by the membership module and consumed by session, cycle, and conversation modules.

### Port Definition

\`\`\`rust
use async_trait::async_trait;
use crate::foundation::{UserId, SessionId, CycleId};

/// Result of an access check
#[derive(Debug, Clone)]
pub struct AccessResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub tier: Option<Tier>,
    pub limits: Option<TierLimits>,
}

impl AccessResult {
    pub fn allowed() -> Self {
        Self { allowed: true, reason: None, tier: None, limits: None }
    }

    pub fn denied(reason: impl Into<String>) -> Self {
        Self { allowed: false, reason: Some(reason.into()), tier: None, limits: None }
    }

    pub fn with_tier(mut self, tier: Tier, limits: TierLimits) -> Self {
        self.tier = Some(tier);
        self.limits = Some(limits);
        self
    }
}

/// Port for checking membership-based access
#[async_trait]
pub trait AccessChecker: Send + Sync {
    /// Check if user can create a new session
    /// Returns denied if: no membership, expired membership, session limit reached
    async fn can_create_session(&self, user_id: &UserId) -> Result<AccessResult, AccessError>;

    /// Check if user can create a new cycle within a session
    /// Returns denied if: cycle limit per session reached
    async fn can_create_cycle(
        &self,
        user_id: &UserId,
        session_id: &SessionId,
    ) -> Result<AccessResult, AccessError>;

    /// Check if user can use AI features
    /// Returns denied if: tier doesn't include AI, daily token quota exceeded
    async fn can_use_ai(&self, user_id: &UserId) -> Result<AccessResult, AccessError>;

    /// Check if user can export data
    /// Returns denied if: tier doesn't include export
    async fn can_export(&self, user_id: &UserId) -> Result<AccessResult, AccessError>;

    /// Get user's current tier and limits (for UI display)
    async fn get_tier_info(&self, user_id: &UserId) -> Result<Option<TierInfo>, AccessError>;
}

#[derive(Debug, Clone)]
pub struct TierInfo {
    pub tier: Tier,
    pub limits: TierLimits,
    pub usage: TierUsage,
}

#[derive(Debug, Clone)]
pub struct TierUsage {
    pub active_sessions: u32,
    pub cycles_in_session: HashMap<SessionId, u32>,
    pub ai_tokens_today: u64,
}
\`\`\`

### Error Types

\`\`\`rust
#[derive(Debug, thiserror::Error)]
pub enum AccessError {
    #[error("Membership not found for user")]
    NoMembership,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}
\`\`\`

### Implementation (Membership Module)

\`\`\`rust
pub struct MembershipAccessChecker {
    membership_repo: Arc<dyn MembershipRepository>,
    session_reader: Arc<dyn SessionReader>,
    cycle_reader: Arc<dyn CycleReader>,
    usage_tracker: Arc<dyn UsageTracker>,
}

#[async_trait]
impl AccessChecker for MembershipAccessChecker {
    async fn can_create_session(&self, user_id: &UserId) -> Result<AccessResult, AccessError> {
        // 1. Get membership
        let membership = self.membership_repo
            .find_by_user(user_id)
            .await?
            .ok_or(AccessError::NoMembership)?;

        // 2. Check membership is active
        if !membership.has_access() {
            return Ok(AccessResult::denied("Membership expired or cancelled"));
        }

        // 3. Check session limit
        let limits = membership.tier().limits();
        if let Some(max_sessions) = limits.max_active_sessions {
            let current = self.session_reader.count_active(user_id).await?;
            if current >= max_sessions {
                return Ok(AccessResult::denied(format!(
                    "Session limit reached ({}/{})",
                    current, max_sessions
                )));
            }
        }

        Ok(AccessResult::allowed().with_tier(membership.tier(), limits))
    }

    // ... other methods
}
\`\`\`

### Consumer Usage (Session Module)

\`\`\`rust
pub struct CreateSessionHandler {
    access_checker: Arc<dyn AccessChecker>,
    session_repo: Arc<dyn SessionRepository>,
    event_outbox: Arc<dyn EventOutboxRepository>,
}

impl CreateSessionHandler {
    pub async fn handle(&self, cmd: CreateSessionCommand) -> Result<SessionId, CommandError> {
        // Authorization check FIRST
        let access = self.access_checker
            .can_create_session(&cmd.user_id)
            .await
            .map_err(|e| CommandError::Internal(e.to_string()))?;

        if !access.allowed {
            return Err(CommandError::AccessDenied(
                access.reason.unwrap_or_else(|| "Access denied".to_string())
            ));
        }

        // Proceed with creation...
        let session = Session::new(cmd.user_id, cmd.title)?;
        // ...
    }
}
\`\`\`
```

**Acceptance Criteria:**
- [ ] Complete AccessChecker trait with all methods
- [ ] AccessResult type with allowed/denied patterns
- [ ] TierInfo and TierUsage types specified
- [ ] Error types defined
- [ ] Implementation example in membership module
- [ ] Consumer usage example in session module

---

### Deliverable 2.3: Add Authorization to Conversation Module

**File:** `docs/modules/conversation.md`
**Action:** UPDATE (add authorization to all command handlers)

**Content to Add (in Command Handlers section):**

```markdown
### Authorization Pattern

All conversation commands require authorization through the parent hierarchy:

\`\`\`
Conversation → Component → Cycle → Session → User
\`\`\`

#### Authorization Helper

\`\`\`rust
impl ConversationAuthorizationService {
    pub async fn authorize_for_component(
        &self,
        component_id: &ComponentId,
        user_id: &UserId,
    ) -> Result<AuthorizedContext, CommandError> {
        // 1. Find cycle containing this component
        let cycle = self.cycle_reader
            .find_by_component(component_id)
            .await?
            .ok_or(CommandError::NotFound("Component not found in any cycle"))?;

        // 2. Load session
        let session = self.session_reader
            .get_by_id(cycle.session_id)
            .await?
            .ok_or(CommandError::NotFound("Session not found"))?;

        // 3. Authorize
        if session.user_id != *user_id {
            return Err(CommandError::Unauthorized);
        }

        Ok(AuthorizedContext { session, cycle })
    }
}
\`\`\`

### SendMessageHandler (with authorization)

\`\`\`rust
pub struct SendMessageHandler {
    auth_service: Arc<ConversationAuthorizationService>,
    conversation_repo: Arc<dyn ConversationRepository>,
    ai_provider: Arc<dyn AIProvider>,
    cycle_repo: Arc<dyn CycleRepository>,
    access_checker: Arc<dyn AccessChecker>,
    event_outbox: Arc<dyn EventOutboxRepository>,
}

impl SendMessageHandler {
    pub async fn handle(&self, cmd: SendMessageCommand) -> Result<Message, CommandError> {
        // 1. AUTHORIZE - Must be first!
        let ctx = self.auth_service
            .authorize_for_component(&cmd.component_id, &cmd.user_id)
            .await?;

        // 2. Check AI access (membership tier)
        let ai_access = self.access_checker
            .can_use_ai(&cmd.user_id)
            .await
            .map_err(|e| CommandError::Internal(e.to_string()))?;

        if !ai_access.allowed {
            return Err(CommandError::AccessDenied(
                ai_access.reason.unwrap_or_else(|| "AI access denied".to_string())
            ));
        }

        // 3. Proceed with message handling...
        let conversation = self.conversation_repo
            .find_by_component(&cmd.component_id)
            .await?
            .ok_or(CommandError::NotFound("Conversation not found"))?;

        // ... rest of handler
    }
}
\`\`\`

### StreamMessageHandler (with authorization)

Same pattern as SendMessageHandler - authorize before any business logic.

### GetConversationQuery (with authorization)

\`\`\`rust
impl GetConversationHandler {
    pub async fn handle(&self, query: GetConversationQuery) -> Result<ConversationView, QueryError> {
        // Authorize through parent hierarchy
        let ctx = self.auth_service
            .authorize_for_component(&query.component_id, &query.user_id)
            .await
            .map_err(|e| QueryError::Unauthorized)?;

        // Proceed with query
        self.conversation_reader
            .get_by_component(&query.component_id)
            .await
    }
}
\`\`\`
```

**Acceptance Criteria:**
- [ ] Authorization pattern documented for conversation module
- [ ] ConversationAuthorizationService specified
- [ ] SendMessageHandler updated with authorization
- [ ] StreamMessageHandler updated with authorization
- [ ] Query handlers updated with authorization
- [ ] AI access check integrated

---

### Deliverable 2.4: Add Authorization to Dashboard Module

**File:** `docs/modules/dashboard.md`
**Action:** UPDATE (expand authorization section)

**Content to Add:**

```markdown
### Complete Authorization Specification

#### authorize_session Method

\`\`\`rust
impl PostgresDashboardReader {
    async fn authorize_session(
        &self,
        session_id: SessionId,
        user_id: &UserId,
    ) -> Result<Session, DashboardError> {
        let session = sqlx::query_as!(
            SessionRow,
            r#"
            SELECT id, user_id, title, description, status, created_at, updated_at
            FROM sessions
            WHERE id = $1
            "#,
            session_id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DashboardError::SessionNotFound(session_id))?;

        if session.user_id != user_id.as_str() {
            return Err(DashboardError::Unauthorized);
        }

        Ok(session.into())
    }
}
\`\`\`

#### Authorization for All Queries

| Query | Authorization Check |
|-------|---------------------|
| GetDashboardOverview | `authorize_session(session_id, user_id)` |
| GetComponentDetail | `authorize_session(session_id, user_id)` via cycle lookup |
| CompareCycles | `authorize_session` for BOTH sessions |

#### Branched Cycle Access

Branched cycles inherit authorization from their root session:

\`\`\`rust
async fn authorize_cycle(
    &self,
    cycle_id: CycleId,
    user_id: &UserId,
) -> Result<Cycle, DashboardError> {
    let cycle = self.get_cycle(cycle_id).await?;

    // Authorization through session
    self.authorize_session(cycle.session_id, user_id).await?;

    Ok(cycle)
}
\`\`\`
```

**Acceptance Criteria:**
- [ ] authorize_session implementation specified
- [ ] All query authorization documented
- [ ] Branched cycle authorization pattern documented
- [ ] Error cases documented

---

### Deliverable 2.5: Update Session Module with Full Integration

**File:** `docs/modules/session.md`
**Action:** UPDATE (add complete AccessChecker integration)

**Content to Add (in CreateSessionHandler section):**

```markdown
### CreateSessionHandler (Complete with AccessChecker)

\`\`\`rust
pub struct CreateSessionHandler {
    access_checker: Arc<dyn AccessChecker>,
    session_repo: Arc<dyn SessionRepository>,
    event_outbox: Arc<dyn EventOutboxRepository>,
    unit_of_work: Arc<dyn UnitOfWork>,
}

impl CreateSessionHandler {
    pub async fn handle(&self, cmd: CreateSessionCommand) -> Result<SessionId, CommandError> {
        // 1. Check membership access
        let access = self.access_checker
            .can_create_session(&cmd.user_id)
            .await
            .map_err(|e| match e {
                AccessError::NoMembership => CommandError::MembershipRequired,
                AccessError::Database(e) => CommandError::Internal(e.to_string()),
                AccessError::Internal(msg) => CommandError::Internal(msg),
            })?;

        if !access.allowed {
            return Err(CommandError::AccessDenied(
                access.reason.unwrap_or_else(|| "Session creation not allowed".to_string())
            ));
        }

        // 2. Create session
        let session = Session::new(cmd.user_id.clone(), cmd.title)?;
        let event = SessionCreated::from(&session);

        // 3. Persist with transactional outbox
        self.unit_of_work
            .execute(|tx| async move {
                self.session_repo.save_with_tx(&session, tx).await?;
                self.event_outbox.store_with_tx(&[event.to_envelope()], tx).await?;
                Ok(session.id)
            })
            .await
    }
}
\`\`\`

### CommandError Updates

\`\`\`rust
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    // Existing errors...

    #[error("Membership required")]
    MembershipRequired,

    #[error("Access denied: {0}")]
    AccessDenied(String),

    #[error("Session limit exceeded: {current}/{limit}")]
    SessionLimitExceeded { current: u32, limit: u32 },
}
\`\`\`

### HTTP Error Mapping

| CommandError | HTTP Status | Response |
|--------------|-------------|----------|
| MembershipRequired | 402 Payment Required | `{ "error": "membership_required" }` |
| AccessDenied | 403 Forbidden | `{ "error": "access_denied", "reason": "..." }` |
| SessionLimitExceeded | 403 Forbidden | `{ "error": "limit_exceeded", "current": N, "limit": M }` |
```

**Acceptance Criteria:**
- [ ] CreateSessionHandler shows full AccessChecker integration
- [ ] CommandError has new membership-related variants
- [ ] HTTP error mapping documented
- [ ] Error handling patterns shown

---

## WORK STREAM 3: Cross-Module Contracts

**Priority:** BLOCKING
**Issues Addressed:** #9, #22, Pattern 4 (Component Validation)
**Owner:** TBD
**Estimated Effort:** 1.5 days

### Deliverable 3.1: Component Output Schemas

**File:** `features/proact-types/component-schemas.md`
**Action:** CREATE new file

**Content:**

```markdown
# PrOACT Component Output Schemas

**Module:** proact-types
**Type:** Cross-Module Contract
**Priority:** HIGH

> JSON Schema definitions for each PrOACT component's structured output. These schemas enforce the contract between conversation extraction and cycle storage.

---

## Overview

Each component type has a defined output schema. The conversation module extracts data matching these schemas, and the cycle module validates before storage.

---

## Schema Validation Port

\`\`\`rust
use serde_json::Value;

/// Port for validating component outputs against their schemas
pub trait ComponentSchemaValidator: Send + Sync {
    /// Validate output against component type's schema
    /// Returns Ok(()) if valid, Err with validation errors if not
    fn validate(
        &self,
        component_type: ComponentType,
        output: &Value,
    ) -> Result<(), SchemaValidationError>;

    /// Get the JSON Schema for a component type
    fn schema_for(&self, component_type: ComponentType) -> &serde_json::Value;
}

#[derive(Debug, thiserror::Error)]
pub enum SchemaValidationError {
    #[error("Missing required field: {field}")]
    MissingRequired { field: String },

    #[error("Invalid type for field {field}: expected {expected}, got {actual}")]
    InvalidType { field: String, expected: String, actual: String },

    #[error("Array too short for field {field}: minimum {min}, got {actual}")]
    ArrayTooShort { field: String, min: usize, actual: usize },

    #[error("Validation errors: {0:?}")]
    Multiple(Vec<SchemaValidationError>),
}
\`\`\`

---

## IssueRaising Output Schema

\`\`\`json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "IssueRaisingOutput",
  "type": "object",
  "required": ["potential_decisions", "objectives", "uncertainties", "considerations"],
  "properties": {
    "potential_decisions": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "description"],
        "properties": {
          "id": { "type": "string", "format": "uuid" },
          "description": { "type": "string", "minLength": 1 },
          "priority": { "type": "string", "enum": ["high", "medium", "low"] }
        }
      }
    },
    "objectives": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "description"],
        "properties": {
          "id": { "type": "string", "format": "uuid" },
          "description": { "type": "string", "minLength": 1 }
        }
      }
    },
    "uncertainties": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "description"],
        "properties": {
          "id": { "type": "string", "format": "uuid" },
          "description": { "type": "string", "minLength": 1 },
          "driver": { "type": "string" }
        }
      }
    },
    "considerations": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "text"],
        "properties": {
          "id": { "type": "string", "format": "uuid" },
          "text": { "type": "string", "minLength": 1 }
        }
      }
    }
  }
}
\`\`\`

---

## ProblemFrame Output Schema

\`\`\`json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ProblemFrameOutput",
  "type": "object",
  "required": ["decision_maker", "focal_decision", "decision_hierarchy"],
  "properties": {
    "decision_maker": {
      "type": "object",
      "required": ["name", "role"],
      "properties": {
        "name": { "type": "string", "minLength": 1 },
        "role": { "type": "string" }
      }
    },
    "focal_decision": {
      "type": "object",
      "required": ["statement", "scope"],
      "properties": {
        "statement": { "type": "string", "minLength": 10 },
        "scope": { "type": "string" },
        "constraints": {
          "type": "array",
          "items": { "type": "string" }
        }
      }
    },
    "decision_hierarchy": {
      "type": "object",
      "required": ["already_made", "focal", "deferred"],
      "properties": {
        "already_made": {
          "type": "array",
          "items": { "$ref": "#/definitions/LinkedDecision" }
        },
        "focal": { "$ref": "#/definitions/LinkedDecision" },
        "deferred": {
          "type": "array",
          "items": { "$ref": "#/definitions/LinkedDecision" }
        }
      }
    },
    "parties": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name", "role"],
        "properties": {
          "name": { "type": "string" },
          "role": { "type": "string", "enum": ["stakeholder", "advisor", "decision_maker"] }
        }
      }
    }
  },
  "definitions": {
    "LinkedDecision": {
      "type": "object",
      "required": ["id", "statement"],
      "properties": {
        "id": { "type": "string", "format": "uuid" },
        "statement": { "type": "string", "minLength": 1 }
      }
    }
  }
}
\`\`\`

---

## Objectives Output Schema

\`\`\`json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ObjectivesOutput",
  "type": "object",
  "required": ["fundamental_objectives", "means_objectives"],
  "properties": {
    "fundamental_objectives": {
      "type": "array",
      "minItems": 1,
      "items": {
        "type": "object",
        "required": ["id", "description"],
        "properties": {
          "id": { "type": "string", "format": "uuid" },
          "description": { "type": "string", "minLength": 1 },
          "performance_measure": {
            "type": "object",
            "properties": {
              "metric": { "type": "string" },
              "direction": { "type": "string", "enum": ["maximize", "minimize", "target"] },
              "target": { "type": "string" }
            }
          }
        }
      }
    },
    "means_objectives": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "description", "supports"],
        "properties": {
          "id": { "type": "string", "format": "uuid" },
          "description": { "type": "string", "minLength": 1 },
          "supports": {
            "type": "array",
            "items": { "type": "string", "format": "uuid" },
            "description": "IDs of fundamental objectives this supports"
          }
        }
      }
    }
  }
}
\`\`\`

---

## Alternatives Output Schema

\`\`\`json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AlternativesOutput",
  "type": "object",
  "required": ["alternatives", "status_quo_id"],
  "properties": {
    "alternatives": {
      "type": "array",
      "minItems": 2,
      "items": {
        "type": "object",
        "required": ["id", "name", "description"],
        "properties": {
          "id": { "type": "string", "format": "uuid" },
          "name": { "type": "string", "minLength": 1, "maxLength": 100 },
          "description": { "type": "string" },
          "is_status_quo": { "type": "boolean", "default": false }
        }
      }
    },
    "status_quo_id": {
      "type": "string",
      "format": "uuid",
      "description": "ID of the alternative designated as status quo (baseline)"
    },
    "strategy_table": {
      "type": "object",
      "description": "Optional strategy table for complex decisions",
      "properties": {
        "decision_columns": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["id", "name", "options"],
            "properties": {
              "id": { "type": "string" },
              "name": { "type": "string" },
              "options": {
                "type": "array",
                "items": { "type": "string" }
              }
            }
          }
        },
        "strategies": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["alternative_id", "selections"],
            "properties": {
              "alternative_id": { "type": "string", "format": "uuid" },
              "selections": {
                "type": "object",
                "additionalProperties": { "type": "string" }
              }
            }
          }
        }
      }
    }
  }
}
\`\`\`

---

## Consequences Output Schema

\`\`\`json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ConsequencesOutput",
  "type": "object",
  "required": ["table"],
  "properties": {
    "table": {
      "type": "object",
      "required": ["alternative_ids", "objective_ids", "cells"],
      "properties": {
        "alternative_ids": {
          "type": "array",
          "items": { "type": "string", "format": "uuid" },
          "minItems": 2
        },
        "objective_ids": {
          "type": "array",
          "items": { "type": "string", "format": "uuid" },
          "minItems": 1
        },
        "cells": {
          "type": "object",
          "description": "Map of 'alt_id:obj_id' -> Cell",
          "additionalProperties": {
            "type": "object",
            "required": ["alternative_id", "objective_id", "rating"],
            "properties": {
              "alternative_id": { "type": "string", "format": "uuid" },
              "objective_id": { "type": "string", "format": "uuid" },
              "rating": {
                "type": "integer",
                "minimum": -2,
                "maximum": 2,
                "description": "Pugh rating: -2 (much worse) to +2 (much better) vs status quo"
              },
              "rationale": { "type": "string" },
              "uncertainty": {
                "type": "object",
                "properties": {
                  "level": { "type": "string", "enum": ["low", "medium", "high"] },
                  "driver": { "type": "string" }
                }
              }
            }
          }
        }
      }
    }
  }
}
\`\`\`

---

## Tradeoffs Output Schema

\`\`\`json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "TradeoffsOutput",
  "type": "object",
  "properties": {
    "dominated_alternatives": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["alternative_id", "dominated_by"],
        "properties": {
          "alternative_id": { "type": "string", "format": "uuid" },
          "dominated_by": { "type": "string", "format": "uuid" },
          "explanation": { "type": "string" }
        }
      }
    },
    "irrelevant_objectives": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["objective_id", "reason"],
        "properties": {
          "objective_id": { "type": "string", "format": "uuid" },
          "reason": { "type": "string" }
        }
      }
    },
    "tensions": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["alternative_id", "gains", "losses"],
        "properties": {
          "alternative_id": { "type": "string", "format": "uuid" },
          "gains": {
            "type": "array",
            "items": { "type": "string", "format": "uuid" },
            "description": "Objective IDs where this alternative excels"
          },
          "losses": {
            "type": "array",
            "items": { "type": "string", "format": "uuid" },
            "description": "Objective IDs where this alternative is weakest"
          }
        }
      }
    }
  }
}
\`\`\`

---

## Recommendation Output Schema

\`\`\`json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "RecommendationOutput",
  "type": "object",
  "required": ["synthesis"],
  "properties": {
    "synthesis": {
      "type": "string",
      "minLength": 50,
      "description": "Summary of the analysis and potential paths forward"
    },
    "standout_option": {
      "type": "object",
      "description": "Optional: If one alternative clearly stands out",
      "properties": {
        "alternative_id": { "type": "string", "format": "uuid" },
        "rationale": { "type": "string" }
      }
    },
    "key_considerations": {
      "type": "array",
      "items": { "type": "string" }
    },
    "remaining_uncertainties": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "description": { "type": "string" },
          "impact": { "type": "string", "enum": ["high", "medium", "low"] }
        }
      }
    }
  }
}
\`\`\`

---

## DecisionQuality Output Schema

\`\`\`json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DecisionQualityOutput",
  "type": "object",
  "required": ["elements"],
  "properties": {
    "elements": {
      "type": "array",
      "minItems": 7,
      "maxItems": 7,
      "items": {
        "type": "object",
        "required": ["name", "score"],
        "properties": {
          "name": {
            "type": "string",
            "enum": [
              "Helpful Problem Frame",
              "Clear Objectives",
              "Creative Alternatives",
              "Reliable Consequence Information",
              "Logically Correct Reasoning",
              "Clear Tradeoffs",
              "Commitment to Follow Through"
            ]
          },
          "score": {
            "type": "integer",
            "minimum": 0,
            "maximum": 100,
            "description": "0-100 percentage score"
          },
          "rationale": { "type": "string" },
          "improvement_path": { "type": "string" }
        }
      }
    },
    "overall_score": {
      "type": "integer",
      "minimum": 0,
      "maximum": 100,
      "description": "Computed as minimum of all element scores"
    }
  }
}
\`\`\`

---

## NotesNextSteps Output Schema

\`\`\`json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "NotesNextStepsOutput",
  "type": "object",
  "properties": {
    "notes": {
      "type": "array",
      "items": { "type": "string" }
    },
    "open_questions": {
      "type": "array",
      "items": { "type": "string" }
    },
    "planned_actions": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["action"],
        "properties": {
          "action": { "type": "string", "minLength": 1 },
          "owner": { "type": "string" },
          "due_date": { "type": "string", "format": "date" },
          "status": { "type": "string", "enum": ["planned", "in_progress", "completed"] }
        }
      }
    },
    "decision_affirmation": {
      "type": "string",
      "description": "When DQ is 100%: affirmation that this was a good decision at time made"
    }
  }
}
\`\`\`

---

## Validation Integration

### In Cycle Module

\`\`\`rust
impl Cycle {
    pub fn update_component_output(
        &mut self,
        ct: ComponentType,
        output: serde_json::Value,
        validator: &dyn ComponentSchemaValidator,
    ) -> Result<(), DomainError> {
        // Validate BEFORE accepting
        validator.validate(ct, &output)
            .map_err(|e| DomainError::validation("component_output", e.to_string()))?;

        // Proceed with update
        let component = self.components.get_mut(&ct)
            .ok_or(DomainError::not_found("component"))?;

        component.set_output(output)?;
        self.updated_at = Timestamp::now();

        Ok(())
    }
}
\`\`\`

### In Conversation Module

\`\`\`rust
impl DataExtractor {
    pub fn extract(
        &self,
        component_type: ComponentType,
        messages: &[Message],
        validator: &dyn ComponentSchemaValidator,
    ) -> Result<serde_json::Value, ExtractionError> {
        // Extract structured data from conversation
        let extracted = self.ai_extractor.extract(component_type, messages).await?;

        // Validate before returning
        validator.validate(component_type, &extracted)
            .map_err(|e| ExtractionError::InvalidOutput(e.to_string()))?;

        Ok(extracted)
    }
}
\`\`\`
```

**Acceptance Criteria:**
- [ ] All 9 component output schemas defined in JSON Schema format
- [ ] ComponentSchemaValidator port trait specified
- [ ] SchemaValidationError types defined
- [ ] Integration with Cycle module shown
- [ ] Integration with Conversation module shown
- [ ] Each schema has required fields, types, and constraints

---

### Deliverable 3.2: Component-to-Cycle Lookup Specification

**File:** `docs/modules/cycle.md`
**Action:** UPDATE (add CycleReader method for component lookup)

**Content to Add:**

```markdown
### CycleReader (Additional Methods)

\`\`\`rust
/// Additional query methods for cross-module lookups
#[async_trait]
pub trait CycleReader: Send + Sync {
    // ... existing methods ...

    /// Find the cycle containing a specific component
    /// Used by conversation module for authorization
    async fn find_by_component(
        &self,
        component_id: &ComponentId,
    ) -> Result<Option<CycleView>, ReaderError>;

    /// Get cycle_id for a component (lightweight lookup)
    async fn get_cycle_id_for_component(
        &self,
        component_id: &ComponentId,
    ) -> Result<Option<CycleId>, ReaderError>;
}
\`\`\`

### Database Index for Component Lookup

\`\`\`sql
-- Enable efficient component -> cycle lookup
CREATE INDEX idx_cycle_components_component_id
ON cycle_components(component_id);
\`\`\`

### PostgresCycleReader Implementation

\`\`\`rust
impl CycleReader for PostgresCycleReader {
    async fn find_by_component(
        &self,
        component_id: &ComponentId,
    ) -> Result<Option<CycleView>, ReaderError> {
        let row = sqlx::query_as!(
            CycleRow,
            r#"
            SELECT c.*
            FROM cycles c
            JOIN cycle_components cc ON c.id = cc.cycle_id
            WHERE cc.component_id = $1
            "#,
            component_id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    async fn get_cycle_id_for_component(
        &self,
        component_id: &ComponentId,
    ) -> Result<Option<CycleId>, ReaderError> {
        let row = sqlx::query_scalar!(
            r#"
            SELECT cycle_id
            FROM cycle_components
            WHERE component_id = $1
            "#,
            component_id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(CycleId::from))
    }
}
\`\`\`
```

**Acceptance Criteria:**
- [ ] find_by_component method added to CycleReader trait
- [ ] get_cycle_id_for_component lightweight method added
- [ ] Database index specified
- [ ] PostgreSQL implementation shown

---

### Deliverable 3.3: Conversation Initialization Specification

**File:** `docs/modules/conversation.md`
**Action:** UPDATE (add initialization section)

**Content to Add:**

```markdown
## Conversation Initialization

### Lifecycle States

\`\`\`rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationState {
    /// Conversation created but not yet ready for user input
    Initializing,

    /// System message sent, ready for user input
    Ready,

    /// User has sent at least one message
    InProgress,

    /// Component marked complete, conversation read-only
    Complete,
}
\`\`\`

### Initialization Flow

\`\`\`
ComponentStarted Event
        │
        ▼
┌───────────────────────────────────────┐
│ ConversationInitHandler               │
│                                       │
│ 1. Create Conversation (Initializing) │
│ 2. Load AgentConfig for component     │
│ 3. Generate system prompt             │
│ 4. Add system message                 │
│ 5. Generate opening question          │
│ 6. Add assistant message (greeting)   │
│ 7. Set state to Ready                 │
│ 8. Persist conversation               │
│ 9. Publish ConversationStarted        │
└───────────────────────────────────────┘
        │
        ▼
Conversation Ready for User Input
\`\`\`

### ConversationInitHandler

\`\`\`rust
pub struct ConversationInitHandler {
    conversation_repo: Arc<dyn ConversationRepository>,
    config_loader: Arc<dyn AgentConfigLoader>,
    ai_provider: Arc<dyn AIProvider>,
    event_outbox: Arc<dyn EventOutboxRepository>,
}

impl EventHandler for ConversationInitHandler {
    fn event_types(&self) -> &[&'static str] {
        &["component.started"]
    }

    async fn handle(&self, envelope: EventEnvelope) -> Result<(), HandlerError> {
        let event: ComponentStarted = envelope.deserialize_payload()?;

        // Check if conversation already exists (idempotency)
        if self.conversation_repo.find_by_component(&event.component_id).await?.is_some() {
            return Ok(()); // Already initialized
        }

        // Load component-specific config
        let config = self.config_loader.load(event.component_type)?;

        // Create conversation
        let mut conversation = Conversation::new(
            event.component_id.clone(),
            event.component_type,
        );

        // Add system message
        let system_prompt = self.build_system_prompt(&config, &event);
        conversation.add_system_message(system_prompt);

        // Generate and add opening message from AI
        let opening = self.generate_opening(&config, event.component_type).await?;
        conversation.add_assistant_message(opening);

        // Mark ready
        conversation.set_state(ConversationState::Ready);

        // Persist and publish event
        let started_event = ConversationStarted::from(&conversation);

        // TODO: Use unit of work for transactional consistency
        self.conversation_repo.save(&conversation).await?;
        self.event_outbox.store(&[started_event.to_envelope()]).await?;

        Ok(())
    }
}
\`\`\`

### Opening Messages by Component

| Component | Opening Message Pattern |
|-----------|------------------------|
| IssueRaising | "Let's start by exploring what's on your mind. What situation or decision are you thinking about?" |
| ProblemFrame | "Now let's get clear on the decision we're making. Who is the primary decision maker here?" |
| Objectives | "What outcomes matter most to you in this decision? What are you trying to achieve?" |
| Alternatives | "What options are you considering? Let's start with any ideas you already have, including doing nothing (status quo)." |
| Consequences | "Now let's think through how each alternative performs on your objectives. Starting with {first_objective}..." |
| Tradeoffs | "Looking at the consequences table, I notice some interesting patterns. Let me highlight the key tradeoffs..." |
| Recommendation | "Based on our analysis, let me summarize what we've found and what it might mean for your decision..." |
| DecisionQuality | "Let's assess the quality of this decision. For each element, rate how confident you are from 0-100%..." |
| NotesNextSteps | "What remaining questions or uncertainties do you have? What are your next steps?" |
```

**Acceptance Criteria:**
- [ ] ConversationState enum defined
- [ ] Initialization flow diagram provided
- [ ] ConversationInitHandler fully specified
- [ ] Opening messages documented per component type
- [ ] Idempotency check included

---

### Deliverable 3.4-3.7: Additional Cross-Module Contracts

Due to length, I'll summarize the remaining deliverables for Work Stream 3:

**3.4: Agent Phase Transition Specification**
- File: `docs/modules/conversation.md`
- Add: Complete phase transition rules per component type
- Add: When phases advance, loop, or complete

**3.5: Streaming Protocol Specification**
- File: `features/integrations/ai-provider-integration.md`
- Add: WebSocket upgrade path
- Add: Chunk format specification
- Add: Error handling mid-stream
- Add: Timeout and cancellation

**3.6: WebSocket Event Bridge Specification**
- File: `features/infrastructure/websocket-event-bridge.md` (CREATE)
- Add: Room management per session
- Add: Event filtering for authorization
- Add: Reconnection with replay
- Add: Message envelope format

**3.7: Error Code Inventory**
- File: `docs/error-handling-strategy.md` (CREATE)
- Add: Complete error code list across all modules
- Add: HTTP status code mapping
- Add: Error response envelope format
- Add: Client handling guidance

---

## WORK STREAM 4-8: Summary

Due to document length constraints, I'll provide summaries for Work Streams 4-8:

### Work Stream 4: Component & Conversation Specs (HIGH)
**Deliverables:**
- 4.1: Component lifecycle state machine documentation
- 4.2: Agent phase transition rules per component
- 4.3: Data extraction specification
- 4.4: Message context window specification
- 4.5: Streaming handler completion
- 4.6: Component revision workflow
- 4.7: Cycle branching inheritance rules
- 4.8: Component status validation

### Work Stream 5: Analysis & Dashboard Algorithms (HIGH)
**Deliverables:**
- 5.1: Dominance detection edge cases
- 5.2: DQ scoring complete algorithm
- 5.3: Tradeoff analysis algorithm
- 5.4: Dashboard data freshness model
- 5.5: Single alternative behavior
- 5.6: Frontend/backend parity specification

### Work Stream 6: Membership & Payment Details (HIGH)
**Deliverables:**
- 6.1: Money type validation rules
- 6.2: PromoCodeValidator port
- 6.3: Webhook event mapping
- 6.4: Subscription lifecycle states
- 6.5: Tier limits enforcement

### Work Stream 7: Infrastructure Integration Gaps (MEDIUM)
**Deliverables:**
- 7.1: AI token accounting in observability
- 7.2: Notification event triggers
- 7.3: Rate limiting AI quotas
- 7.4: Redis failover specification
- 7.5: Configuration strategy
- 7.6: Health check endpoints

### Work Stream 8: Consistency & Standards (LOW)
**Deliverables:**
- 8.1: Timestamp standardization
- 8.2: Feature spec vs checklist status convention
- 8.3: Pagination defaults
- 8.4: Test example data sets

---

## Execution Schedule

```
Day 1:
├── Work Stream 1: Event Infrastructure (morning)
└── Work Stream 2: Authorization (afternoon)

Day 2:
├── Work Stream 3: Cross-Module Contracts (all day)

Day 3:
├── Work Stream 4: Component & Conversation (morning)
└── Work Stream 5: Analysis & Dashboard (afternoon)

Day 4:
├── Work Stream 6: Membership & Payment (morning)
└── Work Stream 7: Infrastructure (afternoon)

Day 5:
├── Work Stream 8: Consistency (morning)
└── Review and validation (afternoon)

Day 6-7:
└── Buffer for revisions and cross-review
```

---

## Validation Checklist

After completing all work streams, verify:

- [ ] All TIER 1 (BLOCKING) issues resolved
- [ ] All TIER 2 (HIGH) issues resolved
- [ ] All TIER 3 (MEDIUM) issues resolved
- [ ] All TIER 4 (LOW) issues resolved
- [ ] DomainEvent trait fully specified with implementation example
- [ ] Transactional outbox pattern documented
- [ ] AccessChecker port completely specified
- [ ] All conversation handlers have authorization
- [ ] All 9 component schemas defined
- [ ] Event flow diagram complete
- [ ] Error code inventory complete
- [ ] All cross-module contracts documented

---

## Next Steps After Plan Completion

1. **Implementation Priority**: Begin with foundation module event infrastructure
2. **First Integration Test**: Session → Cycle → Dashboard event flow
3. **Security Review**: Verify all authorization paths before deployment
4. **Performance Baseline**: Establish metrics for event processing latency

---

*Plan Created: 2026-01-08*
*Status: Ready for Execution*
