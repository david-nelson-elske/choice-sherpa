# Feature: Event Infrastructure

**Module:** foundation + ports + adapters
**Type:** Cross-Cutting Infrastructure
**Priority:** P0 (Foundation)
**Phase:** 1 of Full PrOACT Journey Integration

> Establish the event-driven architecture foundation: domain event types, port interfaces, and in-memory adapter for testing.

---

## Problem Statement

Choice Sherpa's modules need to coordinate without tight coupling. Currently, there's no standardized mechanism for:

1. Modules to announce state changes
2. Other modules to react to those changes
3. Testing cross-module flows in isolation
4. Enabling future async processing

### Current State

- Direct method calls between modules (tight coupling)
- No standardized event format
- Integration tests require full stack
- No path to async processing

### Desired State

- Modules publish events, don't call each other directly
- Standardized `EventEnvelope` format for all events
- Unit tests verify events with in-memory bus
- Same event interface works sync (tests) and async (production)

---

## Tasks

- [x] Create backend project structure with Cargo.toml and src directory
- [x] Implement EventId value object with UUID generation and serialization
- [x] Implement EventMetadata struct with correlation, causation, user, trace IDs
- [x] Implement EventEnvelope struct with all fields and builder methods
- [x] Implement DomainEvent trait with event_type, aggregate_id, aggregate_type, occurred_at, event_id methods
- [x] Implement default to_envelope() method on DomainEvent trait
- [x] Implement EventPublisher port trait with publish and publish_all methods
- [x] Implement EventSubscriber and EventHandler port traits
- [x] Implement InMemoryEventBus adapter with test helper methods
- [x] Implement EventOutboxRepository port for transactional outbox
- [x] Implement ProcessedEventStore port for idempotency tracking
- [x] Implement IdempotentHandler wrapper
- [ ] Implement OutboxPublisher background service
- [x] Add unit tests for EventId, EventEnvelope, EventMetadata
- [x] Add unit tests for InMemoryEventBus publish, subscribe, and handler invocation
- [ ] Add unit tests for idempotency behavior
- [ ] Add integration tests for outbox pattern

---

## Domain Concepts

### Domain Event

A **domain event** is a record of something significant that happened in the domain. Events are:

- **Immutable** - Once created, never modified
- **Past tense** - Named for what already happened (e.g., `SessionCreated`, not `CreateSession`)
- **Self-contained** - Include all data needed to understand what happened
- **First-class domain objects** - Part of the ubiquitous language

### Event Envelope

The `EventEnvelope` is a transport wrapper that adds metadata for routing, tracing, and deduplication:

```
┌─────────────────────────────────────────────┐
│              EventEnvelope                   │
├─────────────────────────────────────────────┤
│ event_id: EventId          ← Deduplication  │
│ event_type: String         ← Routing        │
│ aggregate_id: String       ← Correlation    │
│ aggregate_type: String     ← Context        │
│ occurred_at: Timestamp     ← Ordering       │
│ payload: JSON              ← Event data     │
│ metadata: EventMetadata    ← Tracing        │
└─────────────────────────────────────────────┘
```

### Event Bus Ports

Following hexagonal architecture, the event bus is defined as **ports** (interfaces):

```
┌─────────────────────────────────────────────┐
│                 Domain                       │
│  ┌─────────────────────────────────────┐    │
│  │  DomainEvent trait                   │    │
│  │  EventId, EventEnvelope types        │    │
│  └─────────────────────────────────────┘    │
└─────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────┐
│                  Ports                       │
│  ┌──────────────────┐ ┌──────────────────┐  │
│  │ EventPublisher   │ │ EventSubscriber  │  │
│  │   publish()      │ │   subscribe()    │  │
│  │   publish_all()  │ │   subscribe_all()│  │
│  └──────────────────┘ └──────────────────┘  │
└─────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────┐
│               Adapters                       │
│  ┌──────────────────┐ ┌──────────────────┐  │
│  │ InMemoryEventBus │ │  RedisEventBus   │  │
│  │   (testing)      │ │  (production)    │  │
│  └──────────────────┘ └──────────────────┘  │
└─────────────────────────────────────────────┘
```

---

## Acceptance Criteria

### AC1: DomainEvent Trait

**Given** a module needs to define domain events
**When** implementing the `DomainEvent` trait
**Then** the trait provides:
- `event_type()` → routing key (e.g., "session.created")
- `aggregate_id()` → ID of affected aggregate
- `occurred_at()` → timestamp of event
- `event_id()` → unique ID for deduplication

### AC2: EventEnvelope Serialization

**Given** an event needs to be transported
**When** wrapping in `EventEnvelope`
**Then**:
- Envelope serializes to JSON
- Payload preserves event-specific data
- Metadata includes correlation_id, causation_id, trace_id
- Round-trip serialization is lossless

### AC3: EventPublisher Port

**Given** a command handler completes successfully
**When** publishing events via `EventPublisher`
**Then**:
- `publish(event)` sends single event
- `publish_all(events)` sends multiple atomically
- Errors propagate to caller

### AC4: EventSubscriber Port

**Given** a handler needs to react to events
**When** subscribing via `EventSubscriber`
**Then**:
- `subscribe(event_type, handler)` registers for one type
- `subscribe_all(event_types, handler)` registers for multiple
- Handler receives `EventEnvelope` with full context

### AC5: InMemoryEventBus (Testing)

**Given** unit tests need to verify events
**When** using `InMemoryEventBus`
**Then**:
- Events are delivered synchronously (deterministic)
- `published_events()` returns all published events
- `events_of_type(type)` filters by event type
- `clear()` resets for test isolation
- Handlers are invoked in registration order

### AC6: Handler Invocation

**Given** multiple handlers subscribe to same event type
**When** event is published
**Then**:
- All handlers are invoked
- Handler errors don't prevent other handlers
- Failed handler errors are collected and returned

---

## Technical Design

### File Structure

```
backend/src/domain/foundation/
├── mod.rs                    # Add events export
├── events.rs                 # NEW: DomainEvent, EventId, EventEnvelope
└── events_test.rs            # NEW: Unit tests

backend/src/ports/
├── mod.rs                    # Add event ports export
├── event_publisher.rs        # NEW: EventPublisher trait
└── event_subscriber.rs       # NEW: EventSubscriber, EventHandler traits

backend/src/adapters/
├── mod.rs                    # Add events module export
└── events/
    ├── mod.rs                # NEW: Module exports
    ├── in_memory.rs          # NEW: InMemoryEventBus
    └── in_memory_test.rs     # NEW: Adapter tests
```

### Type Definitions

```rust
// backend/src/domain/foundation/events.rs

use serde::{Deserialize, Serialize};
use std::any::Any;

/// Unique identifier for events (used for deduplication)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(String);

impl EventId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait that all domain events must implement.
/// Provides routing, correlation, and serialization capabilities.
pub trait DomainEvent: Send + Sync + Any {
    /// Event type name for routing (e.g., "session.created")
    /// Convention: lowercase, dot-separated: "{aggregate}.{action}"
    fn event_type(&self) -> &'static str;

    /// ID of the aggregate this event relates to
    fn aggregate_id(&self) -> String;

    /// Type of aggregate (e.g., "session", "cycle")
    fn aggregate_type(&self) -> &'static str;

    /// When the event occurred
    fn occurred_at(&self) -> Timestamp;

    /// Unique event ID for idempotency/deduplication
    fn event_id(&self) -> EventId;

    /// Convert to envelope for transport
    /// Default implementation provided - override only if custom serialization needed
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

// === Event Type Naming Convention ===
//
// | Pattern | Example | Usage |
// |---------|---------|-------|
// | `{module}.{action}` | `session.created` | Aggregate-level events (preferred) |
// | `{module}.{entity}.{action}` | `session.session.created` | Entity-level events |
//
// Canonical Event Types:
// - session.created, session.archived, session.renamed
// - cycle.created, cycle.archived, cycle.branched
// - component.started, component.completed, component.revised
// - conversation.started, conversation.message_sent
// - membership.created, membership.upgraded, membership.cancelled

/// Transport envelope for domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Unique ID for this event instance
    pub event_id: EventId,

    /// Event type for routing (e.g., "session.created")
    pub event_type: String,

    /// ID of the aggregate that emitted this event
    pub aggregate_id: String,

    /// Type of aggregate (e.g., "Session", "Cycle")
    pub aggregate_type: String,

    /// When the event occurred
    pub occurred_at: Timestamp,

    /// Event-specific payload as JSON
    pub payload: serde_json::Value,

    /// Tracing and correlation metadata
    pub metadata: EventMetadata,
}

impl EventEnvelope {
    /// Create envelope from a domain event
    pub fn from_event<E: DomainEvent + Serialize>(event: &E, aggregate_type: &str) -> Self {
        event.to_envelope(aggregate_type)
    }

    /// Add correlation ID for request tracing
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.metadata.correlation_id = Some(id.into());
        self
    }

    /// Add causation ID (ID of event that caused this one)
    pub fn with_causation_id(mut self, id: impl Into<String>) -> Self {
        self.metadata.causation_id = Some(id.into());
        self
    }

    /// Add user ID for audit
    pub fn with_user_id(mut self, id: impl Into<String>) -> Self {
        self.metadata.user_id = Some(id.into());
        self
    }

    /// Deserialize payload to specific event type
    pub fn payload_as<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.payload.clone())
    }
}

/// Metadata for tracing and correlation
/// SECURITY: Custom Debug implementation redacts user_id to prevent sensitive data in logs
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct EventMetadata {
    /// ID linking related events across a request
    pub correlation_id: Option<String>,

    /// ID of the event that caused this event
    pub causation_id: Option<String>,

    /// User who triggered this event
    /// SECURITY: Classified as Internal - redacted in Debug output
    pub user_id: Option<String>,

    /// Distributed tracing ID
    pub trace_id: Option<String>,
}

// SECURITY: Custom Debug implementation to redact sensitive user_id field
// This prevents accidental logging of user identifiers in debug output
impl std::fmt::Debug for EventMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventMetadata")
            .field("correlation_id", &self.correlation_id)
            .field("causation_id", &self.causation_id)
            .field("user_id", &self.user_id.as_ref().map(|_| "[REDACTED]"))
            .field("trace_id", &self.trace_id)
            .finish()
    }
}
```

### Port Interfaces

```rust
// backend/src/ports/event_publisher.rs

use async_trait::async_trait;

/// Port for publishing domain events
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a single event
    async fn publish(&self, event: EventEnvelope) -> Result<(), DomainError>;

    /// Publish multiple events atomically
    /// All events are published or none are (where supported by adapter)
    async fn publish_all(&self, events: Vec<EventEnvelope>) -> Result<(), DomainError>;
}

// backend/src/ports/event_subscriber.rs

use async_trait::async_trait;

/// Handler for processing domain events
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Process an event
    /// Implementations should be idempotent (safe to call multiple times)
    async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError>;

    /// Handler name for logging and metrics
    fn name(&self) -> &'static str;
}

/// Port for subscribing to domain events
pub trait EventSubscriber: Send + Sync {
    /// Subscribe handler to a specific event type
    fn subscribe<H: EventHandler + 'static>(&self, event_type: &str, handler: H);

    /// Subscribe handler to multiple event types
    fn subscribe_all<H: EventHandler + 'static>(&self, event_types: &[&str], handler: H);
}

/// Combined trait for convenience
pub trait EventBus: EventPublisher + EventSubscriber {}

// Blanket implementation
impl<T: EventPublisher + EventSubscriber> EventBus for T {}
```

### In-Memory Adapter

```rust
// backend/src/adapters/events/in_memory.rs

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// In-memory event bus for testing
///
/// Features:
/// - Synchronous delivery (deterministic for tests)
/// - Event capture for assertions
/// - Handler registration and invocation
pub struct InMemoryEventBus {
    handlers: Arc<RwLock<HashMap<String, Vec<Arc<dyn EventHandler>>>>>,
    published: Arc<RwLock<Vec<EventEnvelope>>>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            published: Arc::new(RwLock::new(Vec::new())),
        }
    }

    // === Test Helpers ===

    /// Get all published events (for test assertions)
    pub fn published_events(&self) -> Vec<EventEnvelope> {
        self.published.read().unwrap().clone()
    }

    /// Get events of a specific type
    pub fn events_of_type(&self, event_type: &str) -> Vec<EventEnvelope> {
        self.published_events()
            .into_iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }

    /// Get events for a specific aggregate
    pub fn events_for_aggregate(&self, aggregate_id: &str) -> Vec<EventEnvelope> {
        self.published_events()
            .into_iter()
            .filter(|e| e.aggregate_id == aggregate_id)
            .collect()
    }

    /// Clear all published events (for test isolation)
    pub fn clear(&self) {
        self.published.write().unwrap().clear();
    }

    /// Get count of published events
    pub fn event_count(&self) -> usize {
        self.published.read().unwrap().len()
    }

    /// Check if a specific event type was published
    pub fn has_event(&self, event_type: &str) -> bool {
        self.published
            .read()
            .unwrap()
            .iter()
            .any(|e| e.event_type == event_type)
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventPublisher for InMemoryEventBus {
    async fn publish(&self, event: EventEnvelope) -> Result<(), DomainError> {
        // Store for test assertions
        self.published.write().unwrap().push(event.clone());

        // Invoke handlers synchronously (deterministic for tests)
        let handlers = self.handlers.read().unwrap();
        if let Some(type_handlers) = handlers.get(&event.event_type) {
            let mut errors = Vec::new();

            for handler in type_handlers {
                if let Err(e) = handler.handle(event.clone()).await {
                    errors.push(format!("{}: {}", handler.name(), e));
                }
            }

            if !errors.is_empty() {
                return Err(DomainError::new(
                    ErrorCode::InternalError,
                    &format!("Handler errors: {}", errors.join(", ")),
                ));
            }
        }

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
    fn subscribe<H: EventHandler + 'static>(&self, event_type: &str, handler: H) {
        let mut handlers = self.handlers.write().unwrap();
        handlers
            .entry(event_type.to_string())
            .or_default()
            .push(Arc::new(handler));
    }

    fn subscribe_all<H: EventHandler + 'static>(&self, event_types: &[&str], handler: H) {
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

---

## Test Specifications

### Unit Tests: EventId

```rust
#[test]
fn event_id_generates_unique_values() {
    let id1 = EventId::new();
    let id2 = EventId::new();
    assert_ne!(id1, id2);
}

#[test]
fn event_id_from_string_preserves_value() {
    let id = EventId::from_string("test-id-123");
    assert_eq!(id.as_str(), "test-id-123");
}

#[test]
fn event_id_serializes_to_json() {
    let id = EventId::from_string("test-id");
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, r#""test-id""#);
}
```

### Unit Tests: EventEnvelope

```rust
#[test]
fn envelope_serialization_round_trip() {
    let envelope = EventEnvelope {
        event_id: EventId::from_string("evt-1"),
        event_type: "session.created".to_string(),
        aggregate_id: "session-123".to_string(),
        aggregate_type: "Session".to_string(),
        occurred_at: Timestamp::now(),
        payload: json!({"title": "Test Decision"}),
        metadata: EventMetadata::default(),
    };

    let json = serde_json::to_string(&envelope).unwrap();
    let restored: EventEnvelope = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.event_id, envelope.event_id);
    assert_eq!(restored.event_type, envelope.event_type);
    assert_eq!(restored.aggregate_id, envelope.aggregate_id);
}

#[test]
fn envelope_with_metadata_chain() {
    let envelope = EventEnvelope { /* ... */ }
        .with_correlation_id("req-123")
        .with_causation_id("evt-0")
        .with_user_id("user-456");

    assert_eq!(envelope.metadata.correlation_id, Some("req-123".to_string()));
    assert_eq!(envelope.metadata.causation_id, Some("evt-0".to_string()));
    assert_eq!(envelope.metadata.user_id, Some("user-456".to_string()));
}

#[test]
fn envelope_payload_deserializes() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct TestPayload { value: i32 }

    let envelope = EventEnvelope {
        payload: json!({"value": 42}),
        // ... other fields
    };

    let payload: TestPayload = envelope.payload_as().unwrap();
    assert_eq!(payload.value, 42);
}
```

### Unit Tests: InMemoryEventBus

```rust
#[tokio::test]
async fn publish_stores_event() {
    let bus = InMemoryEventBus::new();
    let event = test_envelope("test.event", "agg-1");

    bus.publish(event.clone()).await.unwrap();

    assert_eq!(bus.event_count(), 1);
    assert!(bus.has_event("test.event"));
}

#[tokio::test]
async fn events_of_type_filters_correctly() {
    let bus = InMemoryEventBus::new();

    bus.publish(test_envelope("type.a", "1")).await.unwrap();
    bus.publish(test_envelope("type.b", "2")).await.unwrap();
    bus.publish(test_envelope("type.a", "3")).await.unwrap();

    let type_a = bus.events_of_type("type.a");
    assert_eq!(type_a.len(), 2);
}

#[tokio::test]
async fn handler_receives_published_event() {
    let bus = Arc::new(InMemoryEventBus::new());
    let received = Arc::new(AtomicBool::new(false));

    struct TestHandler(Arc<AtomicBool>);

    #[async_trait]
    impl EventHandler for TestHandler {
        async fn handle(&self, _: EventEnvelope) -> Result<(), DomainError> {
            self.0.store(true, Ordering::SeqCst);
            Ok(())
        }
        fn name(&self) -> &'static str { "TestHandler" }
    }

    bus.subscribe("test.event", TestHandler(received.clone()));
    bus.publish(test_envelope("test.event", "1")).await.unwrap();

    assert!(received.load(Ordering::SeqCst));
}

#[tokio::test]
async fn multiple_handlers_all_invoked() {
    let bus = Arc::new(InMemoryEventBus::new());
    let counter = Arc::new(AtomicUsize::new(0));

    struct CountingHandler(Arc<AtomicUsize>);

    #[async_trait]
    impl EventHandler for CountingHandler {
        async fn handle(&self, _: EventEnvelope) -> Result<(), DomainError> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        fn name(&self) -> &'static str { "CountingHandler" }
    }

    bus.subscribe("test.event", CountingHandler(counter.clone()));
    bus.subscribe("test.event", CountingHandler(counter.clone()));
    bus.subscribe("test.event", CountingHandler(counter.clone()));

    bus.publish(test_envelope("test.event", "1")).await.unwrap();

    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn subscribe_all_registers_for_multiple_types() {
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

    bus.subscribe_all(
        &["type.a", "type.b", "type.c"],
        CountingHandler(received.clone()),
    );

    bus.publish(test_envelope("type.a", "1")).await.unwrap();
    bus.publish(test_envelope("type.b", "2")).await.unwrap();
    bus.publish(test_envelope("type.d", "3")).await.unwrap(); // Not subscribed

    assert_eq!(received.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn clear_removes_all_events() {
    let bus = InMemoryEventBus::new();

    bus.publish(test_envelope("test.event", "1")).await.unwrap();
    bus.publish(test_envelope("test.event", "2")).await.unwrap();

    assert_eq!(bus.event_count(), 2);

    bus.clear();

    assert_eq!(bus.event_count(), 0);
}

#[tokio::test]
async fn publish_all_atomically_publishes_events() {
    let bus = InMemoryEventBus::new();

    let events = vec![
        test_envelope("type.a", "1"),
        test_envelope("type.b", "2"),
        test_envelope("type.c", "3"),
    ];

    bus.publish_all(events).await.unwrap();

    assert_eq!(bus.event_count(), 3);
}

// Test helper
fn test_envelope(event_type: &str, aggregate_id: &str) -> EventEnvelope {
    EventEnvelope {
        event_id: EventId::new(),
        event_type: event_type.to_string(),
        aggregate_id: aggregate_id.to_string(),
        aggregate_type: "Test".to_string(),
        occurred_at: Timestamp::now(),
        payload: json!({}),
        metadata: EventMetadata::default(),
    }
}
```

---

## Integration Points

### Command Handler Integration Pattern

```rust
// Example: How command handlers use EventPublisher

pub struct CreateSessionHandler {
    repo: Arc<dyn SessionRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl CreateSessionHandler {
    pub async fn handle(&self, cmd: CreateSessionCommand) -> Result<SessionId, DomainError> {
        // 1. Create aggregate
        let session = Session::new(cmd.user_id, cmd.title)?;

        // 2. Persist aggregate
        self.repo.save(&session).await?;

        // 3. Create and publish event
        let event = SessionCreated {
            session_id: session.id(),
            user_id: cmd.user_id,
            title: cmd.title,
            created_at: Timestamp::now(),
        };

        let envelope = EventEnvelope::from_event(&event, "Session")
            .with_user_id(cmd.user_id.to_string());

        self.event_publisher.publish(envelope).await?;

        Ok(session.id())
    }
}
```

### Testing Pattern

```rust
#[tokio::test]
async fn create_session_publishes_event() {
    // Arrange
    let repo = Arc::new(InMemorySessionRepository::new());
    let event_bus = Arc::new(InMemoryEventBus::new());
    let handler = CreateSessionHandler::new(repo, event_bus.clone());

    // Act
    let cmd = CreateSessionCommand {
        user_id: UserId::new("user-1"),
        title: "Career Decision".to_string(),
    };
    let session_id = handler.handle(cmd).await.unwrap();

    // Assert - event was published
    let events = event_bus.events_of_type("session.created");
    assert_eq!(events.len(), 1);

    // Assert - event has correct data
    let payload: SessionCreated = events[0].payload_as().unwrap();
    assert_eq!(payload.title, "Career Decision");
    assert_eq!(events[0].aggregate_id, session_id.to_string());
}
```

---

## Transactional Consistency

### Problem: Event Publishing Race Condition

When command handlers persist data then publish events, a failure in publishing leaves the system inconsistent:

```rust
// PROBLEMATIC PATTERN (DO NOT USE)
self.repository.save(&entity).await?;       // ✅ Committed
self.event_publisher.publish(event).await?; // ❌ Could fail!
// Result: Data persisted, event lost
```

### Solution: Transactional Outbox Pattern

All events are stored in an outbox table within the same database transaction as the domain state change. A separate process publishes events from the outbox.

```
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
```

### EventOutboxRepository Port

```rust
/// Port for storing events in transactional outbox
#[async_trait]
pub trait EventOutboxRepository: Send + Sync {
    /// Store events in outbox (called within transaction)
    async fn store(&self, events: &[EventEnvelope]) -> Result<(), RepositoryError>;

    /// Store events with explicit transaction handle
    async fn store_with_tx(
        &self,
        events: &[EventEnvelope],
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<(), RepositoryError>;

    /// Fetch unpublished events for publishing
    async fn fetch_unpublished(&self, limit: usize) -> Result<Vec<EventEnvelope>, RepositoryError>;

    /// Mark event as published
    async fn mark_published(&self, event_id: &EventId) -> Result<(), RepositoryError>;

    /// Delete old published events (cleanup)
    async fn delete_published_before(&self, timestamp: Timestamp) -> Result<u64, RepositoryError>;
}
```

### Outbox Table Schema

```sql
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

    -- Indexing for efficient polling
    INDEX idx_outbox_unpublished (published_at) WHERE published_at IS NULL
);

-- Cleanup old published events periodically
CREATE INDEX idx_outbox_published_cleanup ON event_outbox(published_at)
    WHERE published_at IS NOT NULL;
```

### Command Handler Pattern (Correct)

```rust
impl CreateSessionHandler {
    pub async fn handle(&self, cmd: CreateSessionCommand) -> Result<SessionId, CommandError> {
        let session = Session::new(cmd.user_id.clone(), cmd.title)?;
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
```

### Outbox Publisher Service

```rust
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
                        self.outbox_repo.mark_published(&event.event_id).await?;
                    }
                    Err(e) => {
                        // Log and continue - will retry on next poll
                        tracing::warn!("Failed to publish event {}: {}", event.event_id.as_str(), e);
                    }
                }
            }

            tokio::time::sleep(self.poll_interval).await;
        }
    }
}
```

### Outbox Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `OUTBOX_POLL_INTERVAL_MS` | 100 | How often to check for unpublished events |
| `OUTBOX_BATCH_SIZE` | 100 | Max events to publish per poll cycle |
| `OUTBOX_RETENTION_DAYS` | 7 | How long to keep published events |

---

## Event Idempotency

### Problem: Duplicate Event Delivery

Events may be delivered more than once due to:
- Network retries
- Outbox publisher restarts
- Consumer crashes before acknowledgment

All event handlers MUST be idempotent.

### Solution: Idempotency Wrapper

The `event_id` (UUID) is the idempotency key. Handlers track processed event IDs.

### ProcessedEventStore Port

```rust
/// Port for tracking which events have been processed
#[async_trait]
pub trait ProcessedEventStore: Send + Sync {
    /// Check if an event has been processed by a specific handler
    async fn contains(
        &self,
        event_id: &EventId,
        handler_name: &str,
    ) -> Result<bool, RepositoryError>;

    /// Mark an event as processed by a specific handler
    async fn mark_processed(
        &self,
        event_id: &EventId,
        handler_name: &str,
    ) -> Result<(), RepositoryError>;

    /// Delete old entries (cleanup)
    async fn delete_before(&self, timestamp: Timestamp) -> Result<u64, RepositoryError>;
}
```

### Processed Events Table

```sql
CREATE TABLE processed_events (
    event_id UUID NOT NULL,
    handler_name VARCHAR(255) NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Composite primary key for handler-specific dedup
    PRIMARY KEY (event_id, handler_name)
);

-- Cleanup old entries (events older than retention period)
CREATE INDEX idx_processed_events_cleanup ON processed_events(processed_at);
```

### IdempotentHandler Wrapper

```rust
pub struct IdempotentHandler<H: EventHandler> {
    inner: H,
    processed_events: Arc<dyn ProcessedEventStore>,
}

impl<H: EventHandler> IdempotentHandler<H> {
    pub fn new(inner: H, processed_events: Arc<dyn ProcessedEventStore>) -> Self {
        Self { inner, processed_events }
    }
}

#[async_trait]
impl<H: EventHandler> EventHandler for IdempotentHandler<H> {
    async fn handle(&self, envelope: EventEnvelope) -> Result<(), DomainError> {
        let handler_name = self.inner.name();

        // Check if already processed
        if self.processed_events.contains(&envelope.event_id, handler_name).await? {
            tracing::debug!(
                "Skipping duplicate event {} for handler {}",
                envelope.event_id.as_str(),
                handler_name
            );
            return Ok(());
        }

        // Process event
        self.inner.handle(envelope.clone()).await?;

        // Mark as processed (after successful handling)
        self.processed_events.mark_processed(&envelope.event_id, handler_name).await?;

        Ok(())
    }

    fn name(&self) -> &'static str {
        self.inner.name()
    }
}
```

### Handler Registration with Idempotency

```rust
// All handlers should be wrapped with idempotency
event_bus.subscribe(
    "session.created",
    IdempotentHandler::new(
        DashboardUpdateHandler::new(dashboard_repo),
        processed_events_store.clone(),
    ),
);
```

### Idempotency Guarantees

| Guarantee | Level | Notes |
|-----------|-------|-------|
| At-least-once delivery | ✅ Guaranteed | Events will be delivered (outbox ensures durability) |
| At-most-once processing | ✅ Guaranteed | Via IdempotentHandler wrapper |
| Exactly-once semantics | ✅ Effective | Combination of above two |
| Ordering within aggregate | ⚠️ Best effort | Use `occurred_at` for ordering |
| Global ordering | ❌ Not guaranteed | Events may arrive out of order across aggregates |

---

## DomainEvent Implementation Example

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreated {
    pub event_id: EventId,
    pub session_id: SessionId,
    pub user_id: UserId,
    pub title: String,
    pub occurred_at: Timestamp,
}

impl SessionCreated {
    pub fn new(session: &Session) -> Self {
        Self {
            event_id: EventId::new(),
            session_id: session.id(),
            user_id: session.user_id().clone(),
            title: session.title().to_string(),
            occurred_at: Timestamp::now(),
        }
    }
}

impl DomainEvent for SessionCreated {
    fn event_type(&self) -> &'static str {
        "session.created"
    }

    fn aggregate_id(&self) -> String {
        self.session_id.to_string()
    }

    fn aggregate_type(&self) -> &'static str {
        "session"
    }

    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

---

## Non-Functional Requirements

### Performance

- In-memory publish: < 1ms for 100 events
- Handler invocation: synchronous, no thread spawning
- Memory: Events stored until `clear()` called

### Thread Safety

- All types are `Send + Sync`
- Uses `RwLock` for interior mutability
- Safe for concurrent publish/subscribe

### Testability

- No external dependencies
- Deterministic behavior
- Full event inspection

---

## Dependencies

### Crate Dependencies

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4"] }
async-trait = "0.1"
```

### Module Dependencies

- `foundation::Timestamp` - For event timing
- `foundation::DomainError` - For error handling
- `foundation::ErrorCode` - For error categorization

---

## Migration Notes

This is a new feature with no migration required. Existing code can adopt incrementally:

1. Add `EventPublisher` to command handler constructors
2. Create domain event types
3. Publish events after successful operations
4. Create handlers that subscribe to events

---

## Related Documents

- **Integration Spec:** features/integrations/full-proact-journey.md
- **Checklist:** REQUIREMENTS/CHECKLIST-events.md
- **Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md

---

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Not Required (internal infrastructure) |
| Authorization Model | Event handlers must verify authorization before processing |
| Sensitive Data | EventMetadata.user_id (Internal), Event payloads may contain Confidential data |
| Rate Limiting | Not Required (internal event bus) |
| Audit Logging | Event publishing and handler invocations logged with correlation_id |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| EventMetadata.user_id | Internal | Redact in logs via custom Debug impl |
| EventMetadata.correlation_id | Public | Safe to log, used for distributed tracing |
| EventMetadata.trace_id | Public | Safe to log, used for observability |
| EventEnvelope.payload | Varies (up to Confidential) | Do not log payload contents; log event_type only |
| EventId | Public | Safe to log, used for deduplication |

### Security Guidelines

1. **Event Payload Logging**: Event handlers MUST NOT log the full event payload. Payloads may contain user decision data classified as CONFIDENTIAL:

```rust
// CORRECT: Log event type and ID only
tracing::info!(
    event_id = %envelope.event_id.as_str(),
    event_type = %envelope.event_type,
    "Processing event"
);

// INCORRECT: Never log payload
tracing::debug!("Event payload: {:?}", envelope.payload); // DO NOT DO THIS
```

2. **Handler Authorization**: Event handlers that modify data MUST verify the operation is authorized. The `user_id` in metadata indicates who triggered the event, but handlers should verify access rights:

```rust
async fn handle(&self, envelope: EventEnvelope) -> Result<(), DomainError> {
    let user_id = envelope.metadata.user_id
        .as_ref()
        .ok_or_else(|| DomainError::unauthorized("Missing user context"))?;

    // Verify user has access before processing
    self.access_checker.check_access(user_id, &resource_id).await?;

    // Process event...
}
```

3. **Idempotency Store Security**: The `ProcessedEventStore` contains event IDs and handler names. This is operational data (Public classification) safe to log and monitor.

4. **Outbox Table Security**: The `event_outbox` table stores full event payloads. Apply same database access controls as other tables containing user data.

---

*Version: 1.0.0*
*Created: 2026-01-07*
*Phase: 1 of 8*
