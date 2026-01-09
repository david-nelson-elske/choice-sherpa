# Event Flow Architecture

**Type:** Cross-Cutting Reference
**Priority:** P0 (Required for understanding module integration)
**Last Updated:** 2026-01-08

> Visual and textual reference for how domain events flow between modules in Choice Sherpa.

---

## Overview

Choice Sherpa uses an event-driven architecture to enable loose coupling between modules. Domain events are published when significant state changes occur, and interested modules subscribe to react accordingly.

**Key Properties:**
- Events are immutable records of what happened
- Transactional outbox ensures events are never lost
- Idempotency wrappers prevent duplicate processing
- All handlers are registered at application startup

---

## Event Flow Diagram

```
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
│  ┌─────────────────────────────────────────────────────────────────────────────┐    │
│  │                       Notification Service                                   │    │
│  │                                                                              │    │
│  │   Subscribes to: Membership events, Cycle milestones                        │    │
│  │   Handlers:                                                                  │    │
│  │     - MembershipCreated → Send welcome email                                │    │
│  │     - CycleCompleted → Send completion summary                              │    │
│  └─────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                       │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Event Subscription Matrix

This matrix shows which modules subscribe to which events:

| Event | Dashboard | Conversation | Analysis | WebSocket | Notifications |
|-------|-----------|--------------|----------|-----------|---------------|
| **Session Events** |
| SessionCreated | ✅ | | | ✅ | |
| SessionArchived | ✅ | | | ✅ | |
| SessionRenamed | ✅ | | | ✅ | |
| **Cycle Events** |
| CycleCreated | ✅ | | | ✅ | |
| CycleBranched | ✅ | | | ✅ | |
| CycleArchived | ✅ | | | ✅ | |
| CycleCompleted | ✅ | | | ✅ | ✅ |
| **Component Events** |
| ComponentStarted | ✅ | ✅ | | ✅ | |
| ComponentCompleted | ✅ | | ✅* | ✅ | |
| ComponentOutputUpdated | ✅ | | | ✅ | |
| **Conversation Events** |
| ConversationStarted | ✅ | | | ✅ | |
| MessageSent | ✅ | | | ✅ | |
| DataExtracted | ✅ | | | ✅ | |
| **Membership Events** |
| MembershipCreated | | | | | ✅ |
| MembershipUpgraded | | | | | ✅ |
| MembershipCancelled | | | | | ✅ |
| **Analysis Events** |
| PughScoresComputed | ✅ | | | ✅ | |
| DQScoresComputed | ✅ | | | ✅ | |

*Analysis only subscribes to ComponentCompleted for Consequences and DecisionQuality component types

---

## Canonical Event Types

All event types follow the pattern: `{module}.{action}` or `{module}.{entity}.{action}`

### Session Module Events

| Event Type | Aggregate | Payload Fields |
|------------|-----------|----------------|
| `session.created` | Session | session_id, user_id, title, created_at |
| `session.archived` | Session | session_id, archived_at |
| `session.renamed` | Session | session_id, old_title, new_title |

### Cycle Module Events

| Event Type | Aggregate | Payload Fields |
|------------|-----------|----------------|
| `cycle.created` | Cycle | cycle_id, session_id, created_at |
| `cycle.branched` | Cycle | cycle_id, parent_cycle_id, branch_point_component |
| `cycle.archived` | Cycle | cycle_id, archived_at |
| `cycle.completed` | Cycle | cycle_id, completed_at |
| `component.started` | Cycle | cycle_id, component_id, component_type |
| `component.completed` | Cycle | cycle_id, component_id, component_type |
| `component.output_updated` | Cycle | cycle_id, component_id, component_type |

### Conversation Module Events

| Event Type | Aggregate | Payload Fields |
|------------|-----------|----------------|
| `conversation.started` | Conversation | conversation_id, component_id |
| `conversation.message_sent` | Conversation | conversation_id, message_id, role |
| `conversation.data_extracted` | Conversation | conversation_id, component_type, output |

### Membership Module Events

| Event Type | Aggregate | Payload Fields |
|------------|-----------|----------------|
| `membership.created` | Membership | membership_id, user_id, tier |
| `membership.upgraded` | Membership | membership_id, old_tier, new_tier |
| `membership.cancelled` | Membership | membership_id, cancelled_at, reason |

### Analysis Module Events

| Event Type | Aggregate | Payload Fields |
|------------|-----------|----------------|
| `analysis.pugh_computed` | Cycle | cycle_id, dominated_alternatives, irrelevant_objectives |
| `analysis.dq_computed` | Cycle | cycle_id, element_scores, overall_score |

---

## Handler Registration Example

```rust
pub fn register_event_handlers(
    event_bus: &mut dyn EventBus,
    processed_store: Arc<dyn ProcessedEventStore>,
    dashboard_repo: Arc<dyn DashboardRepository>,
    conversation_repo: Arc<dyn ConversationRepository>,
    ws_broadcaster: Arc<dyn WebSocketBroadcaster>,
    notification_service: Arc<dyn NotificationService>,
) {
    // Dashboard handlers (subscribe to all state changes)
    event_bus.subscribe_all(
        &[
            "session.created", "session.archived", "session.renamed",
            "cycle.created", "cycle.branched", "cycle.archived", "cycle.completed",
            "component.started", "component.completed", "component.output_updated",
            "conversation.started", "conversation.message_sent", "conversation.data_extracted",
            "analysis.pugh_computed", "analysis.dq_computed",
        ],
        IdempotentHandler::new(
            DashboardUpdateHandler::new(dashboard_repo.clone()),
            processed_store.clone(),
        ),
    );

    // Conversation initialization (triggered by component start)
    event_bus.subscribe(
        "component.started",
        IdempotentHandler::new(
            ConversationInitHandler::new(conversation_repo),
            processed_store.clone(),
        ),
    );

    // Analysis triggers (specific component completions)
    event_bus.subscribe(
        "component.completed",
        IdempotentHandler::new(
            AnalysisTriggerHandler::new(),
            processed_store.clone(),
        ),
    );

    // WebSocket bridge (all events, session-filtered)
    // No idempotency needed - broadcasts are stateless
    event_bus.subscribe_all(
        &["*"], // Wildcard subscription
        WebSocketBridgeHandler::new(ws_broadcaster),
    );

    // Notification service
    event_bus.subscribe_all(
        &[
            "membership.created", "membership.upgraded",
            "cycle.completed",
        ],
        IdempotentHandler::new(
            NotificationHandler::new(notification_service),
            processed_store.clone(),
        ),
    );
}
```

---

## Event Ordering Guarantees

| Scope | Guarantee | Implementation |
|-------|-----------|----------------|
| Within aggregate | Ordered by `occurred_at` | Handlers should use timestamp for ordering |
| Cross-aggregate | No ordering | Events may arrive in any order |
| Within handler | Sequential | Single handler processes events sequentially |
| Across handlers | Parallel | Different handlers may run concurrently |

### Ordering Considerations

1. **Within a session**: Events for a single session are published in order, but handlers may process them concurrently.

2. **Cross-session**: No ordering guarantees. A `SessionCreated` from User A may be processed after a `ComponentCompleted` from User B.

3. **Causally related events**: Use `correlation_id` and `causation_id` in metadata to track event chains.

---

## Failure Scenarios

### Scenario 1: Handler Fails

```
Event Published → Handler Throws → Event remains in outbox → Retry on next poll
```

Handler failures do NOT block other handlers. Each handler processes independently.

```rust
// If DashboardUpdateHandler fails, ConversationInitHandler still runs
for event in events {
    for handler in handlers_for_event(&event.event_type) {
        if let Err(e) = handler.handle(event.clone()).await {
            // Log error but continue with other handlers
            tracing::error!("Handler {} failed: {}", handler.name(), e);
        }
    }
}
```

### Scenario 2: Outbox Publisher Crashes

```
Events in outbox → Publisher crashes → Publisher restarts → Resumes from unpublished events
```

No events lost. Transactional outbox guarantees durability.

### Scenario 3: Duplicate Delivery

```
Event delivered → Handler processes → Network timeout → Event redelivered → Idempotency check → Skipped
```

IdempotentHandler prevents duplicate processing.

### Scenario 4: Consumer Slow

```
Events accumulate in outbox → Publisher batches → Consumer processes at own pace
```

Backpressure is handled by the outbox. Events are never dropped.

---

## Metrics and Observability

### Key Metrics

| Metric | Description | Alert Threshold |
|--------|-------------|-----------------|
| `events_published_total` | Total events published by type | N/A |
| `events_processed_total` | Total events processed by handler | N/A |
| `event_processing_duration_seconds` | Handler processing time | p99 > 1s |
| `outbox_pending_count` | Unpublished events in outbox | > 1000 |
| `outbox_age_seconds` | Age of oldest unpublished event | > 60s |
| `duplicate_events_skipped_total` | Events skipped by idempotency | High rate indicates retry storms |

### Logging

```rust
// Standard log format for events
tracing::info!(
    event_id = %event.event_id.as_str(),
    event_type = %event.event_type,
    aggregate_id = %event.aggregate_id,
    handler = %handler.name(),
    "Processing event"
);
```

---

## Testing Patterns

### Unit Testing Event Publishing

```rust
#[tokio::test]
async fn session_creation_publishes_event() {
    let event_bus = Arc::new(InMemoryEventBus::new());
    let handler = CreateSessionHandler::new(repo, event_bus.clone());

    handler.handle(CreateSessionCommand { /* ... */ }).await.unwrap();

    // Assert event was published
    let events = event_bus.events_of_type("session.created");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].aggregate_type, "session");
}
```

### Unit Testing Event Handlers

```rust
#[tokio::test]
async fn dashboard_updates_on_session_created() {
    let dashboard_repo = Arc::new(InMemoryDashboardRepo::new());
    let handler = DashboardUpdateHandler::new(dashboard_repo.clone());

    let event = EventEnvelope {
        event_type: "session.created".to_string(),
        payload: json!({ "session_id": "sess-1", "title": "Test" }),
        // ...
    };

    handler.handle(event).await.unwrap();

    // Assert dashboard was updated
    let dashboard = dashboard_repo.get("sess-1").await.unwrap();
    assert_eq!(dashboard.title, "Test");
}
```

### Integration Testing Event Flow

```rust
#[tokio::test]
async fn full_event_flow_session_to_dashboard() {
    // Setup with InMemoryEventBus and registered handlers
    let (app, event_bus) = setup_test_app().await;

    // Create session via HTTP
    let response = app.post("/api/sessions")
        .json(&json!({ "title": "Test Decision" }))
        .send()
        .await;

    assert_eq!(response.status(), 201);

    // Wait for event processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Assert dashboard was updated
    let dashboard = app.get("/api/dashboard/overview").send().await;
    assert!(dashboard.text().contains("Test Decision"));
}
```

---

## Related Documents

- **Event Infrastructure Spec:** `features/foundation/event-infrastructure.md`
- **Redis Event Bus:** `features/infrastructure/redis-event-bus.md`
- **WebSocket Bridge:** `features/infrastructure/websocket-event-bridge.md`
- **Implementation Checklist:** `REQUIREMENTS/CHECKLIST-events.md`

---

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Not Required (internal architecture reference) |
| Authorization Model | Event handlers verify access before processing |
| Sensitive Data | Event payloads classified per source module |
| Rate Limiting | Not Required (internal event routing) |
| Audit Logging | Log event_type, aggregate_id, correlation_id; never log payloads |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| Event payloads | Varies (see below) | Classify based on source module |
| correlation_id | Public | Safe to log, used for distributed tracing |
| causation_id | Public | Safe to log, shows event chains |
| aggregate_id | Internal | May reveal resource structure, log with care |
| event_type | Public | Safe to log, used for routing |

### Event Payload Classifications by Module

| Module | Event Types | Payload Classification |
|--------|-------------|------------------------|
| Session | session.* | Internal (titles may hint at decisions) |
| Cycle | cycle.*, component.* | Confidential (contains component outputs) |
| Conversation | conversation.* | Confidential (contains user messages) |
| Membership | membership.* | Internal (subscription details) |
| Analysis | analysis.* | Confidential (derived from user decisions) |

### Security Guidelines

1. **Correlation ID Safety**: Correlation IDs MUST NOT contain sensitive data. Use UUIDs or opaque tokens only:

```rust
// CORRECT
let correlation_id = Uuid::new_v4().to_string();

// INCORRECT - never encode user data in correlation IDs
let correlation_id = format!("user-{}-session-{}", user_id, session_id); // DO NOT DO THIS
```

2. **Event Routing Security**: The event subscription matrix shows which handlers receive which events. Handlers MUST verify authorization before processing:

```rust
// WebSocket Bridge MUST filter events by user authorization
event_bus.subscribe_all(
    &["*"],
    AuthorizedWebSocketBridge::new(ws_broadcaster, access_checker),
);
```

3. **Metrics and Logging**: Log event metadata for observability but never payload contents:

```rust
// CORRECT
tracing::info!(
    event_id = %event.event_id.as_str(),
    event_type = %event.event_type,
    aggregate_type = %event.aggregate_type,
    correlation_id = ?event.metadata.correlation_id,
    "Event processed"
);

// INCORRECT
tracing::debug!("Event payload: {:?}", event.payload); // NEVER DO THIS
```

4. **Cross-Module Event Security**: When events cross module boundaries, the receiving handler is responsible for:
   - Verifying the event is from a trusted source (internal event bus)
   - Checking authorization for the affected resource
   - Not logging confidential payload data

---

*Version: 1.0.0*
*Created: 2026-01-08*
