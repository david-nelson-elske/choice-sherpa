# Implementation Checklist: Event Infrastructure

**Module:** events (cross-cutting)
**Integration Spec:** features/integrations/full-proact-journey.md
**Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md

---

## Overview

This checklist tracks implementation of the event-driven architecture that enables loose coupling between modules. The event bus is the canonical mechanism for cross-module coordination in Choice Sherpa.

---

## Phase 1: Event Infrastructure (Foundation)

### 1.1 Domain Event Types

| Task | Status | File | Tests |
|------|--------|------|-------|
| `DomainEvent` trait definition | [x] | `backend/src/domain/foundation/events.rs` | ✅ |
| `EventId` value object (UUID) | [x] | `backend/src/domain/foundation/events.rs` | ✅ |
| `EventEnvelope` struct | [x] | `backend/src/domain/foundation/events.rs` | ✅ |
| `EventMetadata` struct (correlation, causation, trace) | [x] | `backend/src/domain/foundation/events.rs` | ✅ |
| Serialization tests (JSON round-trip) | [x] | `backend/src/domain/foundation/events.rs` | ✅ |

### 1.2 Port Interfaces

| Task | Status | File | Tests |
|------|--------|------|-------|
| `EventPublisher` trait | [x] | `backend/src/ports/event_publisher.rs` | ✅ |
| `EventSubscriber` trait | [x] | `backend/src/ports/event_subscriber.rs` | ✅ |
| `EventHandler` trait | [x] | `backend/src/ports/event_subscriber.rs` | ✅ |
| `EventBus` combined trait | [x] | `backend/src/ports/event_subscriber.rs` | ✅ |

### 1.3 In-Memory Adapter

| Task | Status | File | Tests |
|------|--------|------|-------|
| `InMemoryEventBus` struct | [x] | `backend/src/adapters/events/in_memory.rs` | ✅ |
| `publish()` implementation | [x] | `backend/src/adapters/events/in_memory.rs` | ✅ |
| `publish_all()` implementation | [x] | `backend/src/adapters/events/in_memory.rs` | ✅ |
| `subscribe()` implementation | [x] | `backend/src/adapters/events/in_memory.rs` | ✅ |
| `subscribe_all()` implementation | [x] | `backend/src/adapters/events/in_memory.rs` | ✅ |
| Test helper: `published_events()` | [x] | `backend/src/adapters/events/in_memory.rs` | ✅ |
| Test helper: `events_of_type()` | [x] | `backend/src/adapters/events/in_memory.rs` | ✅ |
| Test helper: `clear()` | [x] | `backend/src/adapters/events/in_memory.rs` | ✅ |
| Handler invocation test | [x] | `backend/src/adapters/events/in_memory.rs` | ✅ |
| Event ordering test | [x] | `backend/src/adapters/events/in_memory.rs` | ✅ |
| Multiple subscribers test | [x] | `backend/src/adapters/events/in_memory.rs` | ✅ |

### 1.4 Module Exports

| Task | Status | File |
|------|--------|------|
| Export events from foundation | [x] | `backend/src/domain/foundation/mod.rs` |
| Export ports from ports module | [x] | `backend/src/ports/mod.rs` |
| Export adapters from adapters module | [x] | `backend/src/adapters/mod.rs` |
| Create events adapter module | [x] | `backend/src/adapters/events/mod.rs` |

### 1.5 Transactional Outbox

| Task | Status | File | Tests |
|------|--------|------|-------|
| `OutboxWriter` port trait | [x] | `backend/src/ports/outbox_writer.rs` | ✅ |
| `outbox` table migration | [x] | `backend/migrations/20260109000001_create_outbox.sql` | ✅ |
| PostgreSQL outbox adapter | [ ] | `backend/src/adapters/events/postgres_outbox.rs` | [ ] |
| `OutboxPublisher` background service | [x] | `backend/src/adapters/events/outbox_publisher.rs` | ✅ |
| Outbox cleanup job | [ ] | `backend/src/adapters/events/outbox_cleanup.rs` | |
| Unit of work with outbox support | [ ] | `backend/src/adapters/database/unit_of_work.rs` | [ ] |

### 1.6 Idempotency Infrastructure

| Task | Status | File | Tests |
|------|--------|------|-------|
| `ProcessedEventStore` port trait | [x] | `backend/src/ports/processed_event_store.rs` | ✅ |
| `processed_events` table migration | [x] | `backend/migrations/20260109000001_create_outbox.sql` | ✅ |
| PostgreSQL processed events adapter | [ ] | `backend/src/adapters/events/postgres_processed.rs` | [ ] |
| `IdempotentHandler` wrapper | [x] | `backend/src/adapters/events/idempotent_handler.rs` | ✅ |
| Idempotency integration test | [ ] | `backend/tests/integration/idempotency_test.rs` | |

---

## Phase 2: Session Events

### 2.1 Session Domain Events

| Task | Status | File | Tests |
|------|--------|------|-------|
| `SessionCreated` event struct | [ ] | `backend/src/domain/session/events.rs` | [ ] |
| `SessionArchived` event struct | [ ] | `backend/src/domain/session/events.rs` | [ ] |
| `SessionRenamed` event struct | [ ] | `backend/src/domain/session/events.rs` | [ ] |
| Implement `DomainEvent` for all | [ ] | `backend/src/domain/session/events.rs` | |

### 2.2 Session Command Handlers (Event Publishing)

| Task | Status | File | Tests |
|------|--------|------|-------|
| Add `EventPublisher` to `CreateSessionHandler` | [ ] | `backend/src/application/commands/create_session.rs` | |
| Publish `SessionCreated` on success | [ ] | `backend/src/application/commands/create_session.rs` | [ ] |
| Add `EventPublisher` to `ArchiveSessionHandler` | [ ] | `backend/src/application/commands/archive_session.rs` | |
| Publish `SessionArchived` on success | [ ] | `backend/src/application/commands/archive_session.rs` | [ ] |
| Add `EventPublisher` to `RenameSessionHandler` | [ ] | `backend/src/application/commands/rename_session.rs` | |
| Publish `SessionRenamed` on success | [ ] | `backend/src/application/commands/rename_session.rs` | [ ] |

### 2.3 Integration Tests

| Task | Status | File |
|------|--------|------|
| Session creation emits event test | [ ] | `backend/tests/integration/session_events_test.rs` |
| Event contains correct payload test | [ ] | `backend/tests/integration/session_events_test.rs` |

---

## Phase 3: Cycle Events

### 3.1 Cycle Domain Events

| Task | Status | File | Tests |
|------|--------|------|-------|
| `CycleCreated` event struct | [ ] | `backend/src/domain/cycle/events.rs` | [ ] |
| `CycleBranched` event struct | [ ] | `backend/src/domain/cycle/events.rs` | [ ] |
| `ComponentStarted` event struct | [ ] | `backend/src/domain/cycle/events.rs` | [ ] |
| `ComponentCompleted` event struct | [ ] | `backend/src/domain/cycle/events.rs` | [ ] |
| `ComponentOutputUpdated` event struct | [ ] | `backend/src/domain/cycle/events.rs` | [ ] |
| `CycleCompleted` event struct | [ ] | `backend/src/domain/cycle/events.rs` | [ ] |
| Implement `DomainEvent` for all | [ ] | `backend/src/domain/cycle/events.rs` | |

### 3.2 Cycle Command Handlers (Event Publishing)

| Task | Status | File | Tests |
|------|--------|------|-------|
| Add `EventPublisher` to `CreateCycleHandler` | [ ] | `backend/src/application/commands/create_cycle.rs` | |
| Publish `CycleCreated` on success | [ ] | `backend/src/application/commands/create_cycle.rs` | [ ] |
| Add `EventPublisher` to `BranchCycleHandler` | [ ] | `backend/src/application/commands/branch_cycle.rs` | |
| Publish `CycleBranched` on success | [ ] | `backend/src/application/commands/branch_cycle.rs` | [ ] |
| Add `EventPublisher` to `StartComponentHandler` | [ ] | `backend/src/application/commands/start_component.rs` | |
| Publish `ComponentStarted` on success | [ ] | `backend/src/application/commands/start_component.rs` | [ ] |
| Add `EventPublisher` to `CompleteComponentHandler` | [ ] | `backend/src/application/commands/complete_component.rs` | |
| Publish `ComponentCompleted` on success | [ ] | `backend/src/application/commands/complete_component.rs` | [ ] |
| Add `EventPublisher` to `UpdateComponentOutputHandler` | [ ] | `backend/src/application/commands/update_component_output.rs` | |
| Publish `ComponentOutputUpdated` on success | [ ] | `backend/src/application/commands/update_component_output.rs` | [ ] |

### 3.3 Event Handlers

| Task | Status | File | Tests |
|------|--------|------|-------|
| `SessionCycleTracker` handler | [ ] | `backend/src/application/handlers/session_cycle_tracker.rs` | [ ] |
| Subscribe to `CycleCreated` | [ ] | `backend/src/application/handlers/session_cycle_tracker.rs` | |
| Update session's cycle list | [ ] | `backend/src/application/handlers/session_cycle_tracker.rs` | [ ] |

### 3.4 Integration Tests

| Task | Status | File |
|------|--------|------|
| Component lifecycle events test | [ ] | `backend/tests/integration/cycle_events_test.rs` |
| Branch emits correct events test | [ ] | `backend/tests/integration/cycle_events_test.rs` |
| Session updates on cycle create test | [ ] | `backend/tests/integration/cycle_events_test.rs` |

---

## Phase 4: Conversation Events

### 4.1 Conversation Domain Events

| Task | Status | File | Tests |
|------|--------|------|-------|
| `ConversationStarted` event struct | [ ] | `backend/src/domain/conversation/events.rs` | [ ] |
| `MessageSent` event struct | [ ] | `backend/src/domain/conversation/events.rs` | [ ] |
| `StructuredDataExtracted` event struct | [ ] | `backend/src/domain/conversation/events.rs` | [ ] |
| Implement `DomainEvent` for all | [ ] | `backend/src/domain/conversation/events.rs` | |

### 4.2 Conversation Command Handlers

| Task | Status | File | Tests |
|------|--------|------|-------|
| Add `EventPublisher` to `SendMessageHandler` | [ ] | `backend/src/application/commands/send_message.rs` | |
| Publish `MessageSent` on user message | [ ] | `backend/src/application/commands/send_message.rs` | [ ] |
| Publish `MessageSent` on assistant response | [ ] | `backend/src/application/commands/send_message.rs` | [ ] |
| Publish `StructuredDataExtracted` when data extracted | [ ] | `backend/src/application/commands/send_message.rs` | [ ] |

### 4.3 Event Handlers

| Task | Status | File | Tests |
|------|--------|------|-------|
| `ConversationInitHandler` | [ ] | `backend/src/application/handlers/conversation_init.rs` | [ ] |
| Subscribe to `ComponentStarted` | [ ] | `backend/src/application/handlers/conversation_init.rs` | |
| Initialize conversation for component | [ ] | `backend/src/application/handlers/conversation_init.rs` | [ ] |

---

## Phase 5: Analysis Events

### 5.1 Analysis Domain Events

| Task | Status | File | Tests |
|------|--------|------|-------|
| `PughScoresComputed` event struct | [ ] | `backend/src/domain/analysis/events.rs` | [ ] |
| `DQScoresComputed` event struct | [ ] | `backend/src/domain/analysis/events.rs` | [ ] |
| Implement `DomainEvent` for all | [ ] | `backend/src/domain/analysis/events.rs` | |

### 5.2 Analysis Trigger Handler

| Task | Status | File | Tests |
|------|--------|------|-------|
| `AnalysisTriggerHandler` | [ ] | `backend/src/application/handlers/analysis_trigger.rs` | [ ] |
| Subscribe to `ComponentCompleted` | [ ] | `backend/src/application/handlers/analysis_trigger.rs` | |
| Trigger Pugh analysis on Consequences complete | [ ] | `backend/src/application/handlers/analysis_trigger.rs` | [ ] |
| Trigger DQ analysis on DecisionQuality complete | [ ] | `backend/src/application/handlers/analysis_trigger.rs` | [ ] |
| Publish `PughScoresComputed` | [ ] | `backend/src/application/handlers/analysis_trigger.rs` | [ ] |
| Publish `DQScoresComputed` | [ ] | `backend/src/application/handlers/analysis_trigger.rs` | [ ] |

---

## Phase 6: Dashboard Events

### 6.1 Dashboard Update Handler

| Task | Status | File | Tests |
|------|--------|------|-------|
| `DashboardUpdateHandler` struct | [ ] | `backend/src/application/handlers/dashboard_update.rs` | |
| Subscribe to all relevant events | [ ] | `backend/src/application/handlers/dashboard_update.rs` | |
| Handle `SessionCreated` | [ ] | `backend/src/application/handlers/dashboard_update.rs` | [ ] |
| Handle `CycleCreated` | [ ] | `backend/src/application/handlers/dashboard_update.rs` | [ ] |
| Handle `ComponentStarted` | [ ] | `backend/src/application/handlers/dashboard_update.rs` | [ ] |
| Handle `ComponentCompleted` | [ ] | `backend/src/application/handlers/dashboard_update.rs` | [ ] |
| Handle `ComponentOutputUpdated` | [ ] | `backend/src/application/handlers/dashboard_update.rs` | [ ] |
| Handle `MessageSent` | [ ] | `backend/src/application/handlers/dashboard_update.rs` | [ ] |
| Handle `PughScoresComputed` | [ ] | `backend/src/application/handlers/dashboard_update.rs` | [ ] |
| Handle `DQScoresComputed` | [ ] | `backend/src/application/handlers/dashboard_update.rs` | [ ] |
| Idempotency check (EventId dedup) | [ ] | `backend/src/application/handlers/dashboard_update.rs` | [ ] |

### 6.2 Dashboard Cache

| Task | Status | File | Tests |
|------|--------|------|-------|
| `DashboardCache` struct | [ ] | `backend/src/adapters/cache/dashboard_cache.rs` | |
| Cache invalidation on events | [ ] | `backend/src/adapters/cache/dashboard_cache.rs` | [ ] |
| Incremental update logic | [ ] | `backend/src/adapters/cache/dashboard_cache.rs` | [ ] |

---

## Phase 7: Production Adapter (Redis)

### 7.1 Redis Event Bus

| Task | Status | File | Tests |
|------|--------|------|-------|
| `RedisEventBus` struct | [ ] | `backend/src/adapters/events/redis.rs` | |
| Redis Streams XADD for publish | [ ] | `backend/src/adapters/events/redis.rs` | [ ] |
| Redis Streams XREADGROUP for subscribe | [ ] | `backend/src/adapters/events/redis.rs` | [ ] |
| Consumer group management | [ ] | `backend/src/adapters/events/redis.rs` | [ ] |
| Acknowledgment (XACK) on success | [ ] | `backend/src/adapters/events/redis.rs` | [ ] |
| Retry logic for failed handlers | [ ] | `backend/src/adapters/events/redis.rs` | [ ] |

### 7.2 Dead Letter Queue

| Task | Status | File | Tests |
|------|--------|------|-------|
| `DeadLetterQueue` struct | [ ] | `backend/src/adapters/events/dlq.rs` | |
| Store failed events | [ ] | `backend/src/adapters/events/dlq.rs` | [ ] |
| Replay mechanism | [ ] | `backend/src/adapters/events/dlq.rs` | [ ] |
| Admin API for DLQ inspection | [ ] | `backend/src/adapters/http/admin/dlq_handlers.rs` | |

### 7.3 Configuration

| Task | Status | File |
|------|--------|------|
| Event bus adapter selection config | [ ] | `backend/src/config/mod.rs` |
| Redis connection config | [ ] | `backend/src/config/mod.rs` |
| Feature flags for event system | [ ] | `backend/src/config/mod.rs` |

---

## Phase 8: WebSocket Real-Time Updates

### 8.1 WebSocket Infrastructure

| Task | Status | File | Tests |
|------|--------|------|-------|
| `WebSocketEventBridge` struct | [ ] | `backend/src/adapters/websocket/event_bridge.rs` | |
| Subscribe to dashboard events | [ ] | `backend/src/adapters/websocket/event_bridge.rs` | |
| Broadcast to connected clients | [ ] | `backend/src/adapters/websocket/event_bridge.rs` | [ ] |
| Room management (per session) | [ ] | `backend/src/adapters/websocket/rooms.rs` | [ ] |
| Connection lifecycle | [ ] | `backend/src/adapters/websocket/connection.rs` | [ ] |

### 8.2 WebSocket Handlers

| Task | Status | File | Tests |
|------|--------|------|-------|
| `/api/sessions/:id/live` endpoint | [ ] | `backend/src/adapters/http/websocket/routes.rs` | |
| Authentication for WebSocket | [ ] | `backend/src/adapters/http/websocket/auth.rs` | [ ] |
| Message serialization | [ ] | `backend/src/adapters/http/websocket/messages.rs` | [ ] |

---

## Frontend Tasks

### Frontend Event Types

| Task | Status | File | Tests |
|------|--------|------|-------|
| `DashboardUpdate` type | [ ] | `frontend/src/lib/events/types.ts` | |
| Event discriminated union | [ ] | `frontend/src/lib/events/types.ts` | |
| Type guards for events | [ ] | `frontend/src/lib/events/types.ts` | [ ] |

### Frontend WebSocket Client

| Task | Status | File | Tests |
|------|--------|------|-------|
| `useDashboardLive` hook | [ ] | `frontend/src/lib/hooks/use-dashboard-live.ts` | [ ] |
| Reconnection logic | [ ] | `frontend/src/lib/hooks/use-dashboard-live.ts` | [ ] |
| Event dispatching to stores | [ ] | `frontend/src/lib/hooks/use-dashboard-live.ts` | [ ] |

### Frontend Store Updates

| Task | Status | File | Tests |
|------|--------|------|-------|
| Dashboard store event handlers | [ ] | `frontend/src/lib/stores/dashboard.ts` | [ ] |
| Optimistic UI updates | [ ] | `frontend/src/lib/stores/dashboard.ts` | [ ] |

---

## Summary

| Phase | Tasks | Completed |
|-------|-------|-----------|
| Phase 1.1: Domain Event Types | 5 | 5 |
| Phase 1.2: Port Interfaces | 4 | 4 |
| Phase 1.3: In-Memory Adapter | 11 | 11 |
| Phase 1.4: Module Exports | 4 | 4 |
| Phase 1.5: Transactional Outbox | 6 | 3 |
| Phase 1.6: Idempotency Infrastructure | 5 | 3 |
| Phase 2: Session Events | 12 | 0 |
| Phase 3: Cycle Events | 20 | 0 |
| Phase 4: Conversation Events | 10 | 0 |
| Phase 5: Analysis Events | 9 | 0 |
| Phase 6: Dashboard Events | 13 | 0 |
| Phase 7: Redis Adapter | 12 | 0 |
| Phase 8: WebSocket | 10 | 0 |
| Frontend | 9 | 0 |
| **Total** | **130** | **30** |

### Current Status

```
PHASE 1 NEARLY COMPLETE: Core event infrastructure implemented
- Domain types: DomainEvent trait, EventId, EventMetadata, EventEnvelope ✅
- Port interfaces: EventPublisher, EventSubscriber, EventHandler, EventBus ✅
- In-memory adapter: Full InMemoryEventBus with test helpers ✅
- Module exports: All properly wired in foundation, ports, adapters ✅
- Transactional outbox: OutboxWriter port + OutboxPublisher service ✅
- Idempotency: ProcessedEventStore port + IdempotentHandler wrapper ✅
- Database migrations: outbox + processed_events tables ✅

REMAINING FOR PHASE 1:
- PostgreSQL adapters for outbox persistence
- PostgreSQL adapters for processed events persistence
- Outbox cleanup job
- Unit of work integration
- Idempotency integration test
```

---

*Last Updated: 2026-01-09*
*Version: 1.1.0*
