# Integration: Full PrOACT Journey with Event-Driven Architecture

**Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md
**Type:** User Journey + System Infrastructure
**Priority:** P0 (Foundation)

> End-to-end decision journey through all 9 PrOACT components, coordinated via an event bus that enables exceptional modularity and isolated testing.

---

## Executive Summary

This integration establishes **two interconnected capabilities**:

1. **Event-Driven Infrastructure** - A generic event bus pattern consistent with hexagonal architecture that becomes the canonical mechanism for all cross-module coordination
2. **Full PrOACT Journey** - The complete user flow from session creation through decision quality assessment, demonstrating the event-driven pattern

### Why Event-Driven?

| Benefit | Hexagonal Alignment | Testing Impact |
|---------|---------------------|----------------|
| **Loose Coupling** | Modules publish events without knowing subscribers | Unit tests mock EventPublisher, verify events emitted |
| **Swappable Transport** | Event bus is a port; Redis/Kafka are adapters | In-memory adapter for fast, isolated tests |
| **Audit Trail** | All state changes emit events | Event log verifiable in tests |
| **Async Capability** | Same events work sync or async | Sync in tests, async in production |
| **Extensibility** | New features subscribe to existing events | No modification to existing modules |

---

## Part 1: Event-Driven Infrastructure

### Domain Events (Foundation Module)

Domain events are **first-class domain concepts** - they live in the `foundation` module as shared types.

```rust
// backend/src/domain/foundation/events.rs

use serde::{Deserialize, Serialize};
use std::any::Any;

/// Marker trait for all domain events
pub trait DomainEvent: Send + Sync + Any {
    /// Event type name for routing (e.g., "session.created")
    fn event_type(&self) -> &'static str;

    /// Aggregate ID this event relates to
    fn aggregate_id(&self) -> String;

    /// When the event occurred
    fn occurred_at(&self) -> Timestamp;

    /// Unique event ID for idempotency
    fn event_id(&self) -> EventId;
}

/// Unique identifier for events
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub String);

impl EventId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

/// Envelope wrapping any domain event for transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: EventId,
    pub event_type: String,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub occurred_at: Timestamp,
    pub payload: serde_json::Value,
    pub metadata: EventMetadata,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventMetadata {
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,  // ID of event that caused this one
    pub user_id: Option<String>,
    pub trace_id: Option<String>,
}
```

### Event Bus Ports (Port Interfaces)

The event bus itself is defined as **ports** - interfaces in the domain that infrastructure adapters implement.

```rust
// backend/src/ports/events.rs

use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;

/// Port for publishing domain events
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a single event
    async fn publish(&self, event: EventEnvelope) -> Result<(), DomainError>;

    /// Publish multiple events atomically (all or nothing)
    async fn publish_all(&self, events: Vec<EventEnvelope>) -> Result<(), DomainError>;
}

/// Port for subscribing to domain events
pub trait EventSubscriber: Send + Sync {
    /// Subscribe to events of a specific type
    fn subscribe<H>(&self, event_type: &str, handler: H)
    where
        H: EventHandler + 'static;

    /// Subscribe to multiple event types
    fn subscribe_all<H>(&self, event_types: &[&str], handler: H)
    where
        H: EventHandler + 'static;
}

/// Handler for processing events
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError>;

    /// Handler name for logging/metrics
    fn name(&self) -> &'static str;
}

/// Combined bus interface for convenience
pub trait EventBus: EventPublisher + EventSubscriber {}

// Blanket implementation
impl<T: EventPublisher + EventSubscriber> EventBus for T {}
```

### Event Bus Adapters

```rust
// backend/src/adapters/events/in_memory.rs

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

/// In-memory event bus for testing and development
pub struct InMemoryEventBus {
    handlers: Arc<RwLock<HashMap<String, Vec<Arc<dyn EventHandler>>>>>,
    published: Arc<RwLock<Vec<EventEnvelope>>>,  // For test assertions
    broadcast_tx: broadcast::Sender<EventEnvelope>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            published: Arc::new(RwLock::new(Vec::new())),
            broadcast_tx: tx,
        }
    }

    /// Test helper: get all published events
    pub fn published_events(&self) -> Vec<EventEnvelope> {
        self.published.read().unwrap().clone()
    }

    /// Test helper: get events of specific type
    pub fn events_of_type(&self, event_type: &str) -> Vec<EventEnvelope> {
        self.published_events()
            .into_iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }

    /// Test helper: clear all published events
    pub fn clear(&self) {
        self.published.write().unwrap().clear();
    }
}

#[async_trait]
impl EventPublisher for InMemoryEventBus {
    async fn publish(&self, event: EventEnvelope) -> Result<(), DomainError> {
        // Store for test assertions
        self.published.write().unwrap().push(event.clone());

        // Notify handlers synchronously for deterministic testing
        let handlers = self.handlers.read().unwrap();
        if let Some(type_handlers) = handlers.get(&event.event_type) {
            for handler in type_handlers {
                handler.handle(event.clone()).await?;
            }
        }

        // Also broadcast for any async subscribers
        let _ = self.broadcast_tx.send(event);

        Ok(())
    }

    async fn publish_all(&self, events: Vec<EventEnvelope>) -> Result<(), DomainError> {
        for event in events {
            self.publish(event).await?;
        }
        Ok(())
    }
}

impl EventSubscriber for InMemoryEventBus {
    fn subscribe<H>(&self, event_type: &str, handler: H)
    where
        H: EventHandler + 'static,
    {
        let mut handlers = self.handlers.write().unwrap();
        handlers
            .entry(event_type.to_string())
            .or_default()
            .push(Arc::new(handler));
    }

    fn subscribe_all<H>(&self, event_types: &[&str], handler: H)
    where
        H: EventHandler + 'static,
    {
        let handler = Arc::new(handler);
        let mut handlers = self.handlers.write().unwrap();
        for event_type in event_types {
            handlers
                .entry(event_type.to_string())
                .or_default()
                .push(Arc::clone(&handler));
        }
    }
}
```

```rust
// backend/src/adapters/events/redis.rs

/// Redis-based event bus for production
pub struct RedisEventBus {
    client: redis::Client,
    stream_name: String,
    consumer_group: String,
}

#[async_trait]
impl EventPublisher for RedisEventBus {
    async fn publish(&self, event: EventEnvelope) -> Result<(), DomainError> {
        let payload = serde_json::to_string(&event)
            .map_err(|e| DomainError::new(ErrorCode::InternalError, &e.to_string()))?;

        // XADD to Redis Stream
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| DomainError::new(ErrorCode::InternalError, &e.to_string()))?;

        redis::cmd("XADD")
            .arg(&self.stream_name)
            .arg("*")
            .arg("type")
            .arg(&event.event_type)
            .arg("data")
            .arg(&payload)
            .query_async::<_, String>(&mut conn)
            .await
            .map_err(|e| DomainError::new(ErrorCode::InternalError, &e.to_string()))?;

        Ok(())
    }

    async fn publish_all(&self, events: Vec<EventEnvelope>) -> Result<(), DomainError> {
        // Use Redis pipeline for atomicity
        let mut pipe = redis::pipe();

        for event in &events {
            let payload = serde_json::to_string(event)
                .map_err(|e| DomainError::new(ErrorCode::InternalError, &e.to_string()))?;
            pipe.cmd("XADD")
                .arg(&self.stream_name)
                .arg("*")
                .arg("type")
                .arg(&event.event_type)
                .arg("data")
                .arg(&payload);
        }

        let mut conn = self.client.get_async_connection().await
            .map_err(|e| DomainError::new(ErrorCode::InternalError, &e.to_string()))?;

        pipe.query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| DomainError::new(ErrorCode::InternalError, &e.to_string()))?;

        Ok(())
    }
}
```

### Testing Pattern

The event-driven architecture enables exceptionally isolated testing:

```rust
// Example: Testing session creation
#[tokio::test]
async fn test_create_session_emits_event() {
    // Arrange
    let event_bus = Arc::new(InMemoryEventBus::new());
    let repo = Arc::new(InMemorySessionRepository::new());
    let handler = CreateSessionHandler::new(repo, event_bus.clone());

    // Act
    let cmd = CreateSessionCommand {
        user_id: UserId::new("user-123"),
        title: "Career Decision".to_string(),
    };
    let session_id = handler.handle(cmd).await.unwrap();

    // Assert - verify event was emitted
    let events = event_bus.events_of_type("session.created");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].aggregate_id, session_id.to_string());

    // Verify event payload
    let payload: SessionCreatedPayload = serde_json::from_value(events[0].payload.clone()).unwrap();
    assert_eq!(payload.title, "Career Decision");
}

#[tokio::test]
async fn test_event_handler_integration() {
    // Arrange - set up event bus with handlers
    let event_bus = Arc::new(InMemoryEventBus::new());
    let notification_service = Arc::new(MockNotificationService::new());

    // Register handler
    event_bus.subscribe("cycle.completed", CycleCompletedHandler::new(notification_service.clone()));

    // Act - publish event
    let event = EventEnvelope {
        event_id: EventId::new(),
        event_type: "cycle.completed".to_string(),
        aggregate_id: "cycle-123".to_string(),
        aggregate_type: "Cycle".to_string(),
        occurred_at: Timestamp::now(),
        payload: json!({ "session_id": "session-456", "dq_score": 85 }),
        metadata: Default::default(),
    };
    event_bus.publish(event).await.unwrap();

    // Assert - handler was invoked
    assert!(notification_service.was_called());
}
```

---

## Part 2: Full PrOACT Journey

### Overview

The Full PrOACT Journey guides a user through all 9 decision components, from initial issue raising to decision quality assessment. Each step involves:

1. **Conversation** with AI agent
2. **Structured output** extraction
3. **Component state** update
4. **Events published** for coordination

### Modules Involved

| Module | Role | Changes Required |
|--------|------|------------------|
| `foundation` | Producer | Add domain events, EventId, EventEnvelope |
| `session` | Both | Publish SessionCreated, subscribe to CycleCompleted |
| `cycle` | Both | Publish ComponentStarted/Completed, orchestrate component flow |
| `conversation` | Both | Publish MessageReceived, subscribe to ComponentStarted |
| `analysis` | Consumer | Subscribe to ConsequencesCompleted for Pugh calculation |
| `dashboard` | Consumer | Subscribe to all events for view model updates |

---

## Data Flow

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                           EVENT BUS (Redis/In-Memory)                         │
└──────────────────────────────────────────────────────────────────────────────┘
       ▲           ▲              ▲              ▲              ▲
       │           │              │              │              │
   SessionCreated  │      ComponentStarted   MessageSent    DashboardUpdated
       │           │              │              │              │
┌──────┴──────┐  ┌─┴─────────┐  ┌─┴──────────┐  ┌┴────────────┐  ┌┴────────────┐
│   Session   │  │   Cycle   │  │Conversation│  │  Analysis   │  │  Dashboard  │
└─────────────┘  └───────────┘  └────────────┘  └─────────────┘  └─────────────┘
       │              │               │               │               │
       │         CycleCreated    MessageReceived  ScoresComputed     │
       │        ComponentCompleted ConversationEnded   │               │
       │              │               │               │               │
       └──────────────┴───────────────┴───────────────┴───────────────┘
                                      │
                                      ▼
                              ┌───────────────┐
                              │   PostgreSQL   │
                              └───────────────┘
```

### User Journey Flow

```
User Creates Session
        │
        ▼
┌───────────────────┐    SessionCreated
│  Session Module   │ ─────────────────────────────────────────┐
└───────────────────┘                                          │
        │                                                      ▼
        ▼                                             ┌───────────────┐
┌───────────────────┐    CycleCreated                 │   Dashboard   │
│   Cycle Module    │ ─────────────────────────────────┤  (Updates)    │
└───────────────────┘                                  └───────────────┘
        │                                                      ▲
        ▼                                                      │
┌───────────────────┐    ComponentStarted                      │
│  Start Issue      │ ─────────────────────────────────────────┤
│  Raising          │                                          │
└───────────────────┘                                          │
        │                                                      │
        ▼                                                      │
┌───────────────────┐    MessageSent, MessageReceived          │
│  Conversation     │ ─────────────────────────────────────────┤
│  (AI Agent)       │                                          │
└───────────────────┘                                          │
        │                                                      │
        ▼                                                      │
┌───────────────────┐    ComponentOutputUpdated                │
│  Extract Output   │ ─────────────────────────────────────────┤
└───────────────────┘                                          │
        │                                                      │
        ▼                                                      │
┌───────────────────┐    ComponentCompleted                    │
│  Complete Issue   │ ─────────────────────────────────────────┤
│  Raising          │                                          │
└───────────────────┘                                          │
        │                                                      │
        ├──────── (Repeat for each component) ─────────────────┤
        │                                                      │
        ▼                                                      │
┌───────────────────┐    ConsequencesCompleted                 │
│  Consequences     │ ──────────────────────┐                  │
│  Component        │                       ▼                  │
└───────────────────┘              ┌───────────────┐           │
                                   │   Analysis    │           │
                                   │ (Pugh Scores) │           │
                                   └───────────────┘           │
                                          │                    │
                                   ScoresComputed ─────────────┤
        │                                                      │
        ▼                                                      │
┌───────────────────┐    CycleCompleted, DQScored              │
│  Decision Quality │ ─────────────────────────────────────────┘
│  Component        │
└───────────────────┘
```

---

## Domain Events Catalog

### Session Events

```rust
// backend/src/domain/session/events.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreated {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub title: String,
    pub created_at: Timestamp,
}

impl DomainEvent for SessionCreated {
    fn event_type(&self) -> &'static str { "session.created" }
    fn aggregate_id(&self) -> String { self.session_id.to_string() }
    fn occurred_at(&self) -> Timestamp { self.created_at }
    fn event_id(&self) -> EventId { EventId::new() }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArchived {
    pub session_id: SessionId,
    pub archived_at: Timestamp,
}

impl DomainEvent for SessionArchived {
    fn event_type(&self) -> &'static str { "session.archived" }
    // ...
}
```

### Cycle Events

```rust
// backend/src/domain/cycle/events.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleCreated {
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    pub parent_cycle_id: Option<CycleId>,
    pub created_at: Timestamp,
}

impl DomainEvent for CycleCreated {
    fn event_type(&self) -> &'static str { "cycle.created" }
    fn aggregate_id(&self) -> String { self.cycle_id.to_string() }
    fn occurred_at(&self) -> Timestamp { self.created_at }
    fn event_id(&self) -> EventId { EventId::new() }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleBranched {
    pub cycle_id: CycleId,
    pub parent_cycle_id: CycleId,
    pub branch_point: ComponentType,
    pub branched_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStarted {
    pub cycle_id: CycleId,
    pub component_id: ComponentId,
    pub component_type: ComponentType,
    pub started_at: Timestamp,
}

impl DomainEvent for ComponentStarted {
    fn event_type(&self) -> &'static str { "component.started" }
    fn aggregate_id(&self) -> String { self.cycle_id.to_string() }
    fn occurred_at(&self) -> Timestamp { self.started_at }
    fn event_id(&self) -> EventId { EventId::new() }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentOutputUpdated {
    pub cycle_id: CycleId,
    pub component_id: ComponentId,
    pub component_type: ComponentType,
    pub output_summary: String,  // Brief summary for logging
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentCompleted {
    pub cycle_id: CycleId,
    pub component_id: ComponentId,
    pub component_type: ComponentType,
    pub completed_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleCompleted {
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    pub dq_score: Option<Percentage>,
    pub completed_at: Timestamp,
}
```

### Conversation Events

```rust
// backend/src/domain/conversation/events.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationStarted {
    pub conversation_id: ConversationId,
    pub component_id: ComponentId,
    pub component_type: ComponentType,
    pub started_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSent {
    pub conversation_id: ConversationId,
    pub message_id: MessageId,
    pub role: Role,
    pub content_preview: String,  // First 100 chars
    pub sent_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredDataExtracted {
    pub conversation_id: ConversationId,
    pub component_id: ComponentId,
    pub component_type: ComponentType,
    pub extraction_summary: String,
    pub extracted_at: Timestamp,
}
```

### Analysis Events

```rust
// backend/src/domain/analysis/events.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PughScoresComputed {
    pub cycle_id: CycleId,
    pub scores: HashMap<String, i32>,  // AlternativeID -> score
    pub dominated_count: i32,
    pub computed_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DQScoresComputed {
    pub cycle_id: CycleId,
    pub element_scores: Vec<(String, Percentage)>,
    pub overall_score: Percentage,
    pub computed_at: Timestamp,
}
```

---

## Coordination Points

### Synchronous Calls (Within Request)

| From | To | Method | Purpose |
|------|----|--------|---------|
| HTTP Handler | Command Handler | Direct call | Process user request |
| Command Handler | Repository | Port call | Persist aggregate |
| Command Handler | EventPublisher | Port call | Emit events |

### Asynchronous Events (Background Processing)

| Event | Publisher | Subscribers | Purpose |
|-------|-----------|-------------|---------|
| `session.created` | Session | Dashboard | Initialize overview |
| `cycle.created` | Cycle | Session, Dashboard | Track cycle in session |
| `component.started` | Cycle | Conversation, Dashboard | Initialize conversation |
| `component.completed` | Cycle | Dashboard, Analysis (for Consequences) | Update views, trigger analysis |
| `component.output_updated` | Cycle | Dashboard | Real-time view updates |
| `message.sent` | Conversation | Dashboard | Live chat display |
| `pugh_scores.computed` | Analysis | Dashboard | Update scores in view |
| `dq_scores.computed` | Analysis | Dashboard, Session | Display scores, notify completion |
| `cycle.completed` | Cycle | Session, Dashboard | Mark completion |

### Event Handler Registration

```rust
// backend/src/main.rs or setup module

fn register_event_handlers(event_bus: &impl EventSubscriber, deps: &Dependencies) {
    // Dashboard updates (subscribes to everything)
    event_bus.subscribe_all(
        &[
            "session.created",
            "cycle.created",
            "component.started",
            "component.completed",
            "component.output_updated",
            "message.sent",
            "pugh_scores.computed",
            "dq_scores.computed",
            "cycle.completed",
        ],
        DashboardUpdateHandler::new(deps.dashboard_cache.clone())
    );

    // Conversation initialization
    event_bus.subscribe(
        "component.started",
        ConversationInitHandler::new(deps.conversation_repo.clone())
    );

    // Analysis triggers
    event_bus.subscribe(
        "component.completed",
        AnalysisTriggerHandler::new(deps.analysis_service.clone())
    );

    // Session cycle tracking
    event_bus.subscribe(
        "cycle.created",
        SessionCycleTracker::new(deps.session_repo.clone())
    );
}
```

---

## Failure Modes

| Failure | Impact | Detection | Recovery |
|---------|--------|-----------|----------|
| Event publish fails | State saved but handlers not triggered | Async publish error | Retry with exponential backoff |
| Handler throws | Other handlers still process | Handler exception | Log, continue, dead-letter queue |
| Redis unavailable | Events queue locally | Connection error | In-memory fallback, replay when connected |
| Out-of-order events | Dashboard may show stale data | Timestamp comparison | Event ordering in stream, optimistic refresh |
| Duplicate events | Handler runs twice | EventId tracking | Idempotent handlers |

### Compensation Actions

Events are designed for **at-least-once delivery**. All handlers must be idempotent:

```rust
// Example: Idempotent dashboard update handler
#[async_trait]
impl EventHandler for DashboardUpdateHandler {
    async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError> {
        // Check if already processed (using event_id)
        if self.processed_events.contains(&event.event_id) {
            return Ok(());  // Already handled, skip
        }

        // Process event
        match event.event_type.as_str() {
            "component.completed" => {
                self.update_component_status(&event).await?;
            }
            // ... other handlers
            _ => {}
        }

        // Mark as processed
        self.processed_events.insert(event.event_id.clone());

        Ok(())
    }

    fn name(&self) -> &'static str { "DashboardUpdateHandler" }
}
```

### Dead Letter Queue

Events that fail after max retries go to dead-letter for manual investigation:

```rust
// backend/src/adapters/events/dlq.rs

pub struct DeadLetterQueue {
    storage: Arc<dyn DLQStorage>,
}

impl DeadLetterQueue {
    pub async fn send(&self, event: EventEnvelope, error: DomainError, attempts: i32) {
        let dlq_entry = DLQEntry {
            event,
            error: error.to_string(),
            attempts,
            failed_at: Timestamp::now(),
        };
        self.storage.store(dlq_entry).await;
    }

    pub async fn replay(&self, event_id: EventId) -> Result<(), DomainError> {
        // Retrieve and republish
    }
}
```

---

## Shared Types

### New Interfaces (Foundation Module)

```rust
// backend/src/domain/foundation/events.rs

/// All domain events implement this trait
pub trait DomainEvent: Send + Sync + Any {
    fn event_type(&self) -> &'static str;
    fn aggregate_id(&self) -> String;
    fn occurred_at(&self) -> Timestamp;
    fn event_id(&self) -> EventId;
}

/// Wrapper for transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: EventId,
    pub event_type: String,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub occurred_at: Timestamp,
    pub payload: serde_json::Value,
    pub metadata: EventMetadata,
}

/// Event ID for deduplication
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub String);

/// Metadata for tracing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventMetadata {
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub user_id: Option<String>,
    pub trace_id: Option<String>,
}
```

### Existing Types Used

- `SessionId`, `CycleId`, `ComponentId` (Foundation) - Aggregate identifiers
- `ComponentType`, `ComponentStatus` (Foundation) - Enums
- `Timestamp` (Foundation) - Time values
- `Percentage` (Foundation) - DQ scores
- `Rating` (Foundation) - Pugh ratings

---

## API Contracts

### No New HTTP Endpoints

The event system is internal. Events are published as a side effect of existing commands.

### Events Published by Command Handlers

| Command | Events Published |
|---------|------------------|
| `CreateSession` | `SessionCreated` |
| `ArchiveSession` | `SessionArchived` |
| `CreateCycle` | `CycleCreated` |
| `BranchCycle` | `CycleBranched` |
| `StartComponent` | `ComponentStarted` |
| `CompleteComponent` | `ComponentCompleted` |
| `UpdateComponentOutput` | `ComponentOutputUpdated` |
| `SendMessage` | `MessageSent` |
| `CompleteCycle` | `CycleCompleted` |

### WebSocket Events (Dashboard Real-Time Updates)

```typescript
// Frontend subscribes to dashboard updates
const ws = new WebSocket('/api/sessions/:id/live');

ws.onmessage = (event) => {
  const update: DashboardUpdate = JSON.parse(event.data);
  // { type: 'component.completed', data: { componentType: 'objectives', ... } }
  dashboardStore.applyUpdate(update);
};
```

---

## Implementation Phases

### Phase 1: Event Infrastructure

**Goal:** Establish event bus pattern with in-memory adapter

**Modules:** foundation, ports

**Deliverables:**
- [ ] `DomainEvent` trait in foundation
- [ ] `EventId`, `EventEnvelope`, `EventMetadata` types
- [ ] `EventPublisher` port interface
- [ ] `EventSubscriber` port interface
- [ ] `InMemoryEventBus` adapter with test helpers
- [ ] Unit tests for event bus

**Exit Criteria:** Can publish and subscribe to events in tests

---

### Phase 2: Session Events

**Goal:** Session module emits and handles events

**Modules:** session, dashboard

**Deliverables:**
- [ ] `SessionCreated`, `SessionArchived` events
- [ ] `CreateSessionHandler` publishes `SessionCreated`
- [ ] `DashboardUpdateHandler` subscribes to session events
- [ ] Integration test: session creation updates dashboard

**Exit Criteria:** Creating a session triggers dashboard update

---

### Phase 3: Cycle Events

**Goal:** Cycle module emits component lifecycle events

**Modules:** cycle, conversation, dashboard

**Deliverables:**
- [ ] `CycleCreated`, `CycleBranched` events
- [ ] `ComponentStarted`, `ComponentCompleted`, `ComponentOutputUpdated` events
- [ ] `CycleCompleted` event
- [ ] Conversation initializes on `ComponentStarted`
- [ ] Dashboard updates on all cycle events

**Exit Criteria:** Full component lifecycle triggers appropriate handlers

---

### Phase 4: Conversation Events

**Goal:** Conversation module emits message events

**Modules:** conversation, dashboard

**Deliverables:**
- [ ] `ConversationStarted`, `MessageSent` events
- [ ] `StructuredDataExtracted` event
- [ ] Dashboard shows live chat updates

**Exit Criteria:** Messages appear in real-time on dashboard

---

### Phase 5: Analysis Events

**Goal:** Analysis service publishes computation results

**Modules:** analysis, cycle, dashboard

**Deliverables:**
- [ ] `PughScoresComputed` event
- [ ] `DQScoresComputed` event
- [ ] Analysis triggers on `ComponentCompleted` (for Consequences, DecisionQuality)
- [ ] Dashboard displays computed scores

**Exit Criteria:** Completing Consequences triggers Pugh score display

---

### Phase 6: Production Adapter

**Goal:** Redis event bus for production

**Modules:** adapters

**Deliverables:**
- [ ] `RedisEventBus` adapter using Redis Streams
- [ ] Consumer group for reliable delivery
- [ ] Dead letter queue for failed events
- [ ] Configuration switch: in-memory vs Redis

**Exit Criteria:** Production environment uses Redis for events

---

## Testing Strategy

### Unit Tests (Per Module)

| Module | Test Focus |
|--------|------------|
| Foundation | EventId generation, EventEnvelope serialization |
| Session | Command handlers emit correct events |
| Cycle | Component lifecycle events, branching events |
| Conversation | Message events, extraction events |
| Analysis | Score computation events |
| Dashboard | Handler idempotency, view model updates |

### Integration Tests

| Test | Modules | Scenario |
|------|---------|----------|
| SessionToDb | Session, Event Bus, Dashboard | Session creation flows to dashboard |
| FullComponent | Cycle, Conversation, Dashboard | Component start → conversation → complete |
| AnalysisTrigger | Cycle, Analysis, Dashboard | Consequences complete → Pugh scores |
| BranchFlow | Cycle, Event Bus | Branch emits correct events |

### Event Bus Tests

```rust
#[tokio::test]
async fn test_event_ordering() {
    let bus = InMemoryEventBus::new();

    bus.publish(event1).await.unwrap();
    bus.publish(event2).await.unwrap();
    bus.publish(event3).await.unwrap();

    let events = bus.published_events();
    assert_eq!(events.len(), 3);
    assert!(events[0].occurred_at <= events[1].occurred_at);
    assert!(events[1].occurred_at <= events[2].occurred_at);
}

#[tokio::test]
async fn test_handler_receives_events() {
    let bus = Arc::new(InMemoryEventBus::new());
    let received = Arc::new(AtomicUsize::new(0));

    struct CountingHandler(Arc<AtomicUsize>);

    #[async_trait]
    impl EventHandler for CountingHandler {
        async fn handle(&self, _: EventEnvelope) -> Result<(), DomainError> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        fn name(&self) -> &'static str { "CountingHandler" }
    }

    bus.subscribe("test.event", CountingHandler(received.clone()));

    bus.publish(test_event()).await.unwrap();
    bus.publish(test_event()).await.unwrap();

    assert_eq!(received.load(Ordering::SeqCst), 2);
}
```

### E2E Tests

| Journey | Steps | Verification |
|---------|-------|--------------|
| Complete Decision | Create session → 9 components → DQ score | Events logged, dashboard updates, scores computed |
| Branch Decision | Complete 3 components → Branch → Different path | Both cycles have correct events |
| Resume Session | Create session → Close → Reopen | Dashboard reconstructs from stored state |

---

## Rollout Plan

### Feature Flags

| Flag | Purpose | Default |
|------|---------|---------|
| `event_bus_enabled` | Use event-driven updates | off |
| `redis_events` | Use Redis vs in-memory | off |
| `async_handlers` | Process events async | off |

### Migration Steps

1. Deploy event infrastructure (flag off)
2. Enable event publishing (handlers still sync)
3. Monitor event volume and latency
4. Enable async handlers
5. Switch to Redis adapter
6. Remove feature flags

---

## File Structure Changes

```
backend/src/domain/foundation/
├── mod.rs
├── ... (existing)
├── events.rs               # NEW: DomainEvent trait, EventId, EventEnvelope
└── events_test.rs          # NEW

backend/src/ports/
├── mod.rs
├── ... (existing)
├── event_publisher.rs      # NEW
└── event_subscriber.rs     # NEW

backend/src/adapters/events/
├── mod.rs                  # NEW
├── in_memory.rs            # NEW: InMemoryEventBus
├── in_memory_test.rs       # NEW
├── redis.rs                # NEW: RedisEventBus
├── redis_test.rs           # NEW
└── dlq.rs                  # NEW: Dead Letter Queue

backend/src/domain/session/
├── ... (existing)
└── events.rs               # NEW: SessionCreated, SessionArchived

backend/src/domain/cycle/
├── ... (existing)
└── events.rs               # NEW: CycleCreated, ComponentStarted, etc.

backend/src/domain/conversation/
├── ... (existing)
└── events.rs               # NEW: ConversationStarted, MessageSent

backend/src/domain/analysis/
├── ... (existing)
└── events.rs               # NEW: PughScoresComputed, DQScoresComputed

backend/src/application/handlers/
├── mod.rs                  # NEW
├── dashboard_update.rs     # NEW: DashboardUpdateHandler
├── conversation_init.rs    # NEW: ConversationInitHandler
├── analysis_trigger.rs     # NEW: AnalysisTriggerHandler
└── session_cycle_tracker.rs # NEW: SessionCycleTracker
```

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Required for all user-initiated operations |
| Authorization Model | User must own session; AccessChecker enforces tier limits |
| Sensitive Data | Decision content (objectives, alternatives, recommendations) |
| Rate Limiting | Required at API layer (see rate-limiting.md) |
| Audit Logging | All component transitions, cycle completions |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| Session title/description | Confidential | User-owned, not shared without consent |
| Objectives | Confidential | Core decision content |
| Alternatives | Confidential | Core decision content |
| Consequences | Confidential | Core decision content |
| Recommendations | Confidential | Core decision content |
| DQ scores | Confidential | Analysis results |
| Conversation messages | Confidential | Contains user reasoning |
| `session_id`, `cycle_id` | Internal | Safe to log |
| `component_type` | Internal | Safe to log |

### Authorization Flow Through Journey

```
User Request
     │
     ▼
┌─────────────────┐
│ Authentication  │  ← Verify user identity (Zitadel)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Session Access  │  ← User owns session OR has share link
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Membership Tier │  ← AccessChecker validates limits
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Component Access│  ← Some components gated by tier (DQ)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Execute Command │  ← Authorized operation proceeds
└─────────────────┘
```

### Security Controls

- **Session Isolation**: Users can only access their own sessions unless explicitly shared
- **Event Metadata**: All events include `user_id` in metadata for audit
- **No Cross-Session Leakage**: Event handlers must verify session ownership
- **Sensitive Data in Events**: Use `content_preview` (truncated) rather than full content in events

### Inherited Security

This integration spec inherits security requirements from component modules:
- Session: `features/session/` security requirements
- Cycle: `features/cycle/` security requirements
- Conversation: `features/conversation/` security requirements
- Analysis: `features/analysis/` security requirements (pure functions, no direct security)

---

## Related Documents

- **Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md
- **Module Specs:**
  - docs/modules/foundation.md
  - docs/modules/session.md
  - docs/modules/cycle.md
  - docs/modules/conversation.md
  - docs/modules/analysis.md
  - docs/modules/dashboard.md
- **Feature Dependencies:**
  - features/foundation/value-objects.md
  - features/session/create-session.md
  - features/cycle/component-lifecycle.md

---

## Appendix: Event Bus Comparison

| Adapter | Use Case | Pros | Cons |
|---------|----------|------|------|
| **InMemory** | Testing, Development | Fast, deterministic, no infra | No persistence, single process |
| **Redis Streams** | Production | Persistence, consumer groups, replay | Redis dependency |
| **PostgreSQL LISTEN/NOTIFY** | Simple production | No new infra | No persistence, limited throughput |
| **Kafka** | High-scale | Partitioning, replay, exactly-once | Operational complexity |

Recommended: Start with **InMemory** for tests, **Redis Streams** for production.

---

*Version: 1.0.0*
*Created: 2026-01-07*
*Integration Type: Foundation + User Journey*
