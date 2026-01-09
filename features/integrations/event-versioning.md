# Integration: Event Versioning Strategy

**Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md
**Type:** Cross-Cutting Infrastructure
**Priority:** P1 (Required before event infrastructure goes to production)
**Depends On:** foundation module (event infrastructure)

> Schema evolution strategy for domain events, ensuring backward/forward compatibility and safe migrations.

---

## Overview

As Choice Sherpa evolves, domain events will change: new fields added, old fields removed, formats altered. Without a versioning strategy, event consumers break when producers emit new schemas, and event replay becomes impossible. This specification defines how events evolve safely.

### Key Challenges

| Challenge | Risk | Solution |
|-----------|------|----------|
| **Schema changes** | Consumers can't parse new events | Explicit versioning + migration |
| **Replay compatibility** | Historical events can't be replayed | Upcasters convert old → new |
| **Deployment coordination** | All services must update simultaneously | Backward-compatible changes first |
| **Event store pollution** | Multiple versions coexist indefinitely | Version metadata in envelope |

---

## Design Principles

### 1. Explicit Version Numbers

Every event type carries a version number in its type identifier.

```
# Format: {aggregate}.{event_name}.v{version}
session.created.v1
session.created.v2
cycle.component_completed.v1
```

### 2. Backward Compatibility by Default

New versions MUST be readable by old consumers (with graceful degradation).

| Change Type | Compatibility | Strategy |
|-------------|---------------|----------|
| Add optional field | Backward compatible | Old consumers ignore new field |
| Add required field | **Breaking** | Provide default or use new version |
| Remove field | Backward compatible | New consumers handle missing field |
| Rename field | **Breaking** | Add new field, deprecate old |
| Change field type | **Breaking** | New version required |

### 3. Upcaster Chain

Old events are transformed to current version on read, never mutated in storage.

```
Stored (v1) → Upcaster(v1→v2) → Upcaster(v2→v3) → Current (v3)
```

---

## Event Envelope

```rust
// backend/src/domain/foundation/events.rs

/// Envelope wrapping all domain events for transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Unique event identifier (for deduplication)
    pub event_id: EventId,

    /// Event type with version: "session.created.v1"
    pub event_type: String,

    /// Schema version number (extracted from event_type)
    pub schema_version: u32,

    /// Aggregate that emitted this event
    pub aggregate_id: String,
    pub aggregate_type: String,

    /// When the event occurred
    pub occurred_at: Timestamp,

    /// The event payload (JSON)
    pub payload: serde_json::Value,

    /// Metadata for tracing and correlation
    pub metadata: EventMetadata,
}

impl EventEnvelope {
    pub fn new<E: DomainEvent>(event: E) -> Self {
        Self {
            event_id: EventId::new(),
            event_type: event.event_type().to_string(),
            schema_version: event.schema_version(),
            aggregate_id: event.aggregate_id(),
            aggregate_type: event.aggregate_type().to_string(),
            occurred_at: Timestamp::now(),
            payload: serde_json::to_value(&event).unwrap(),
            metadata: EventMetadata::default(),
        }
    }

    /// Extract version from event_type (e.g., "session.created.v2" → 2)
    pub fn version(&self) -> u32 {
        self.schema_version
    }
}
```

---

## DomainEvent Trait

```rust
// backend/src/domain/foundation/events.rs

/// Trait implemented by all domain events
pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync {
    /// Event type identifier with version (e.g., "session.created.v1")
    fn event_type(&self) -> &str;

    /// Schema version number
    fn schema_version(&self) -> u32;

    /// Aggregate type (e.g., "Session")
    fn aggregate_type(&self) -> &str;

    /// Aggregate ID as string
    fn aggregate_id(&self) -> String;
}

/// Derive macro for DomainEvent (example usage)
/// ```rust
/// #[derive(Debug, Clone, Serialize, Deserialize, DomainEvent)]
/// #[event(type = "session.created", version = 1, aggregate = "Session")]
/// pub struct SessionCreatedV1 {
///     pub session_id: SessionId,
///     pub user_id: UserId,
///     pub title: String,
/// }
/// ```
```

---

## Version Evolution Example

### Version 1: Initial

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreatedV1 {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub title: String,
}

impl DomainEvent for SessionCreatedV1 {
    fn event_type(&self) -> &str { "session.created.v1" }
    fn schema_version(&self) -> u32 { 1 }
    fn aggregate_type(&self) -> &str { "Session" }
    fn aggregate_id(&self) -> String { self.session_id.to_string() }
}
```

### Version 2: Add Optional Field (Backward Compatible)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreatedV2 {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub title: String,
    #[serde(default)]  // Makes it backward compatible
    pub description: Option<String>,
}

impl DomainEvent for SessionCreatedV2 {
    fn event_type(&self) -> &str { "session.created.v2" }
    fn schema_version(&self) -> u32 { 2 }
    // ...
}
```

### Version 3: Structural Change (Breaking)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreatedV3 {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub title: String,
    pub description: Option<String>,
    pub owner: SessionOwner,  // NEW: replaces user_id with richer type
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionOwner {
    pub user_id: UserId,
    pub display_name: String,
    pub email: Option<String>,
}
```

---

## Upcaster System

Upcasters transform events from older versions to newer versions.

```rust
// backend/src/infrastructure/events/upcasters.rs

/// Transforms events from one version to another
pub trait Upcaster: Send + Sync {
    /// Source event type (e.g., "session.created.v1")
    fn source_type(&self) -> &str;

    /// Target event type (e.g., "session.created.v2")
    fn target_type(&self) -> &str;

    /// Transform the payload
    fn upcast(&self, payload: serde_json::Value) -> Result<serde_json::Value, UpcastError>;
}

#[derive(Debug, thiserror::Error)]
pub enum UpcastError {
    #[error("missing required field: {0}")]
    MissingField(String),

    #[error("invalid field value: {0}")]
    InvalidValue(String),

    #[error("incompatible version transition: {from} → {to}")]
    IncompatibleVersions { from: String, to: String },
}
```

### Example Upcasters

```rust
// V1 → V2: Add optional description field
pub struct SessionCreatedV1ToV2;

impl Upcaster for SessionCreatedV1ToV2 {
    fn source_type(&self) -> &str { "session.created.v1" }
    fn target_type(&self) -> &str { "session.created.v2" }

    fn upcast(&self, mut payload: serde_json::Value) -> Result<serde_json::Value, UpcastError> {
        // Add description: null for old events
        payload["description"] = serde_json::Value::Null;
        Ok(payload)
    }
}

// V2 → V3: Transform user_id to owner object
pub struct SessionCreatedV2ToV3;

impl Upcaster for SessionCreatedV2ToV3 {
    fn source_type(&self) -> &str { "session.created.v2" }
    fn target_type(&self) -> &str { "session.created.v3" }

    fn upcast(&self, mut payload: serde_json::Value) -> Result<serde_json::Value, UpcastError> {
        let user_id = payload.get("user_id")
            .ok_or_else(|| UpcastError::MissingField("user_id".to_string()))?
            .clone();

        // Transform flat user_id to owner object
        payload["owner"] = serde_json::json!({
            "user_id": user_id,
            "display_name": "Unknown",  // Default for historical events
            "email": null
        });

        Ok(payload)
    }
}
```

---

## Upcaster Registry

```rust
// backend/src/infrastructure/events/upcaster_registry.rs

pub struct UpcasterRegistry {
    /// Map from source_type to upcaster
    upcasters: HashMap<String, Box<dyn Upcaster>>,
    /// Current version for each event base type
    current_versions: HashMap<String, u32>,
}

impl UpcasterRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            upcasters: HashMap::new(),
            current_versions: HashMap::new(),
        };

        // Register all upcasters
        registry.register(Box::new(SessionCreatedV1ToV2));
        registry.register(Box::new(SessionCreatedV2ToV3));
        // ... more upcasters

        // Set current versions
        registry.set_current_version("session.created", 3);
        registry.set_current_version("cycle.created", 1);
        // ... more versions

        registry
    }

    pub fn register(&mut self, upcaster: Box<dyn Upcaster>) {
        self.upcasters.insert(upcaster.source_type().to_string(), upcaster);
    }

    /// Upcast event to current version
    pub fn upcast_to_current(
        &self,
        envelope: EventEnvelope
    ) -> Result<EventEnvelope, UpcastError> {
        let base_type = self.extract_base_type(&envelope.event_type);
        let current_version = self.current_versions.get(&base_type)
            .copied()
            .unwrap_or(1);

        let mut current = envelope;

        // Chain upcasters until we reach current version
        while current.schema_version < current_version {
            let upcaster = self.upcasters.get(&current.event_type)
                .ok_or_else(|| UpcastError::IncompatibleVersions {
                    from: current.event_type.clone(),
                    to: format!("{}.v{}", base_type, current_version),
                })?;

            let new_payload = upcaster.upcast(current.payload)?;
            current = EventEnvelope {
                event_type: upcaster.target_type().to_string(),
                schema_version: current.schema_version + 1,
                payload: new_payload,
                ..current
            };
        }

        Ok(current)
    }

    fn extract_base_type(&self, event_type: &str) -> String {
        // "session.created.v2" → "session.created"
        event_type.rsplit_once(".v")
            .map(|(base, _)| base.to_string())
            .unwrap_or_else(|| event_type.to_string())
    }
}
```

---

## Event Deserialization

```rust
// backend/src/infrastructure/events/deserializer.rs

pub struct EventDeserializer {
    registry: UpcasterRegistry,
}

impl EventDeserializer {
    /// Deserialize and upcast event to current version
    pub fn deserialize<E: DomainEvent>(&self, envelope: EventEnvelope) -> Result<E, DeserializeError> {
        // First upcast to current version
        let current = self.registry.upcast_to_current(envelope)?;

        // Then deserialize the payload
        serde_json::from_value(current.payload)
            .map_err(|e| DeserializeError::Parse(e.to_string()))
    }

    /// Deserialize without upcasting (for handlers that support multiple versions)
    pub fn deserialize_raw(&self, envelope: &EventEnvelope) -> Result<serde_json::Value, DeserializeError> {
        Ok(envelope.payload.clone())
    }
}
```

---

## Handler Version Support

Handlers can declare which versions they support.

```rust
// backend/src/application/handlers/mod.rs

/// Handler that only supports current version (uses upcasting)
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Event types this handler processes
    fn handles(&self) -> &[&str];

    /// Process the event (receives current version)
    async fn handle(&self, event: EventEnvelope) -> Result<(), HandlerError>;
}

/// Handler that supports multiple versions explicitly
#[async_trait]
pub trait MultiVersionEventHandler: Send + Sync {
    /// Event types with version ranges: [("session.created", 1..=3)]
    fn handles_versions(&self) -> &[(&str, std::ops::RangeInclusive<u32>)];

    /// Process any supported version
    async fn handle(&self, event: EventEnvelope) -> Result<(), HandlerError>;
}
```

### Version-Aware Handler Example

```rust
pub struct SessionIndexHandler {
    search_index: Arc<dyn SearchIndex>,
}

impl MultiVersionEventHandler for SessionIndexHandler {
    fn handles_versions(&self) -> &[(&str, std::ops::RangeInclusive<u32>)] {
        &[("session.created", 1..=3)]
    }

    async fn handle(&self, event: EventEnvelope) -> Result<(), HandlerError> {
        match event.schema_version {
            1 => {
                let v1: SessionCreatedV1 = serde_json::from_value(event.payload)?;
                self.index_session(&v1.session_id, &v1.title, None).await
            }
            2 => {
                let v2: SessionCreatedV2 = serde_json::from_value(event.payload)?;
                self.index_session(&v2.session_id, &v2.title, v2.description.as_deref()).await
            }
            3 => {
                let v3: SessionCreatedV3 = serde_json::from_value(event.payload)?;
                self.index_session(&v3.session_id, &v3.title, v3.description.as_deref()).await
            }
            v => Err(HandlerError::UnsupportedVersion(v)),
        }
    }
}
```

---

## Migration Strategy

### Rolling Deployment

```
Timeline:
─────────────────────────────────────────────────────────────────────────►
│                                                                         │
│  T1: Deploy consumers     T2: Deploy producers    T3: Cleanup          │
│      that handle v1+v2        emitting v2             (optional)       │
│                                                                         │

During T1→T2:
  - Producers emit v1
  - Consumers handle v1 and v2

During T2→T3:
  - Producers emit v2
  - Consumers handle v1 and v2

After T3 (optional):
  - Remove v1 handling code
  - Keep upcasters for replay
```

### Migration Checklist

```markdown
## Event Schema Change: session.created v2 → v3

### Pre-deployment
- [ ] Write SessionCreatedV3 type
- [ ] Write SessionCreatedV2ToV3 upcaster
- [ ] Register upcaster in registry
- [ ] Update current_version to 3
- [ ] Write tests for upcaster
- [ ] Test replay of v1 and v2 events

### Consumer Deployment
- [ ] Deploy services with v3 support
- [ ] Verify v2 events still processed correctly
- [ ] Monitor for deserialization errors

### Producer Deployment
- [ ] Deploy services emitting v3
- [ ] Verify v3 events processed correctly
- [ ] Monitor event flow

### Cleanup (Optional)
- [ ] Remove v2 type (keep upcaster)
- [ ] Update documentation
```

---

## Event Store Schema

```sql
-- PostgreSQL schema for event store

CREATE TABLE events (
    event_id UUID PRIMARY KEY,
    event_type VARCHAR(255) NOT NULL,
    schema_version INTEGER NOT NULL,
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_id VARCHAR(255) NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    payload JSONB NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Indexes for common queries
    INDEX idx_events_aggregate (aggregate_type, aggregate_id),
    INDEX idx_events_type (event_type),
    INDEX idx_events_occurred (occurred_at)
);

-- Version history for auditing
CREATE TABLE event_schema_versions (
    event_base_type VARCHAR(255) PRIMARY KEY,
    current_version INTEGER NOT NULL,
    versions JSONB NOT NULL,  -- Array of version metadata
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

---

## Replay Support

```rust
// backend/src/infrastructure/events/replay.rs

pub struct EventReplayer {
    event_store: Arc<dyn EventStore>,
    upcaster_registry: UpcasterRegistry,
    handler_registry: HandlerRegistry,
}

impl EventReplayer {
    /// Replay all events for an aggregate
    pub async fn replay_aggregate(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
    ) -> Result<(), ReplayError> {
        let events = self.event_store
            .get_events_for_aggregate(aggregate_type, aggregate_id)
            .await?;

        for envelope in events {
            // Upcast to current version
            let current = self.upcaster_registry.upcast_to_current(envelope)?;

            // Dispatch to handlers
            self.handler_registry.dispatch(current).await?;
        }

        Ok(())
    }

    /// Replay events in a time range (for rebuilding projections)
    pub async fn replay_range(
        &self,
        from: Timestamp,
        to: Timestamp,
        handler: &dyn EventHandler,
    ) -> Result<ReplayStats, ReplayError> {
        let mut stats = ReplayStats::default();

        let events = self.event_store.get_events_in_range(from, to).await?;

        for envelope in events {
            stats.total += 1;

            let current = match self.upcaster_registry.upcast_to_current(envelope) {
                Ok(e) => e,
                Err(e) => {
                    stats.failed += 1;
                    stats.errors.push(e.to_string());
                    continue;
                }
            };

            if handler.handles().contains(&current.event_type.as_str()) {
                handler.handle(current).await?;
                stats.processed += 1;
            } else {
                stats.skipped += 1;
            }
        }

        Ok(stats)
    }
}

#[derive(Debug, Default)]
pub struct ReplayStats {
    pub total: u64,
    pub processed: u64,
    pub skipped: u64,
    pub failed: u64,
    pub errors: Vec<String>,
}
```

---

## Testing Strategy

### Upcaster Tests

```rust
#[test]
fn test_session_created_v1_to_v2() {
    let upcaster = SessionCreatedV1ToV2;

    let v1_payload = json!({
        "session_id": "sess-123",
        "user_id": "user-456",
        "title": "Career Decision"
    });

    let v2_payload = upcaster.upcast(v1_payload).unwrap();

    assert_eq!(v2_payload["session_id"], "sess-123");
    assert_eq!(v2_payload["user_id"], "user-456");
    assert_eq!(v2_payload["title"], "Career Decision");
    assert!(v2_payload["description"].is_null());
}

#[test]
fn test_full_upcaster_chain() {
    let registry = UpcasterRegistry::new();

    let v1_envelope = EventEnvelope {
        event_type: "session.created.v1".to_string(),
        schema_version: 1,
        payload: json!({
            "session_id": "sess-123",
            "user_id": "user-456",
            "title": "Career Decision"
        }),
        ..Default::default()
    };

    let v3_envelope = registry.upcast_to_current(v1_envelope).unwrap();

    assert_eq!(v3_envelope.event_type, "session.created.v3");
    assert_eq!(v3_envelope.schema_version, 3);
    assert!(v3_envelope.payload["owner"].is_object());
}
```

### Backward Compatibility Tests

```rust
#[test]
fn test_v3_handler_can_process_upcasted_v1() {
    let registry = UpcasterRegistry::new();
    let handler = SessionCreatedHandler::new();

    // Old event from storage
    let v1 = EventEnvelope {
        event_type: "session.created.v1".to_string(),
        schema_version: 1,
        payload: json!({
            "session_id": "sess-123",
            "user_id": "user-456",
            "title": "Test"
        }),
        ..Default::default()
    };

    // Upcast and handle
    let v3 = registry.upcast_to_current(v1).unwrap();
    let result = handler.handle(v3);

    assert!(result.is_ok());
}
```

---

## Implementation Phases

### Phase 1: Core Infrastructure

- [ ] Add schema_version to EventEnvelope
- [ ] Update DomainEvent trait with schema_version()
- [ ] Create Upcaster trait
- [ ] Create UpcasterRegistry
- [ ] Write unit tests for upcaster chain

### Phase 2: Existing Event Migration

- [ ] Audit all existing event types
- [ ] Add version suffix to all event types
- [ ] Create identity upcasters (v1 → v1) for migration
- [ ] Update event store schema

### Phase 3: Deserialization Layer

- [ ] Implement EventDeserializer with upcasting
- [ ] Integrate into event bus consumers
- [ ] Add version to event metadata logging
- [ ] Write integration tests

### Phase 4: Replay Support

- [ ] Implement EventReplayer
- [ ] Add replay CLI command
- [ ] Test full replay with upcasting
- [ ] Document replay procedures

### Phase 5: Documentation & Governance

- [ ] Create "Adding a New Event Version" guide
- [ ] Create migration checklist template
- [ ] Add schema change review to PR process
- [ ] Document version deprecation policy

---

## Governance

### Version Numbering

- Start at v1, increment by 1
- Never reuse version numbers
- Never modify a released version

### Deprecation Policy

| Age | Status | Action |
|-----|--------|--------|
| < 6 months | Active | Fully supported |
| 6-12 months | Deprecated | Warning in logs, upcasters active |
| > 12 months | Legacy | Upcasters only, no direct support |

### Breaking Change Approval

Breaking changes require:
1. RFC document explaining the change
2. Migration plan with rollback strategy
3. Upcaster implementation and tests
4. Review by architecture team

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Not directly applicable (internal infrastructure) |
| Authorization Model | Events inherit authorization from originating command |
| Sensitive Data | Event payloads may contain classified data |
| Rate Limiting | Not Required (internal) |
| Audit Logging | Event publication is itself audit logging |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| `event_id` | Internal | Safe to log |
| `event_type` | Internal | Safe to log |
| `aggregate_id` | Internal | Safe to log |
| `payload` | Varies | Classification inherited from source data |
| `metadata.user_id` | PII | Include for audit, but protect in exports |

### Security Controls

- **Classification Preservation**: When upcasting events from v1 to v2+, the security classification of fields MUST be preserved
- **No Classification Downgrade**: An upcaster MUST NOT change a field from Confidential to Internal/Public
- **Payload Encryption**: For events containing Confidential data, consider encrypting payload at rest
- **Event Replay Authorization**: Replay operations must verify the operator has appropriate access level
- **Audit Trail Integrity**: Event store records must be append-only; no deletion or modification

### Versioning Security Considerations

- New versions MUST NOT accidentally expose previously hidden fields
- Upcasters MUST NOT retrieve additional sensitive data during transformation
- Schema changes that alter security classification require security review

---

## Exit Criteria

1. **All events versioned**: Every event type has explicit version number
2. **Upcaster chain works**: v1 events replay correctly to current version
3. **Handlers robust**: Handlers process any upcasted version
4. **Replay tested**: Full event store replay succeeds
5. **Documentation complete**: Versioning guide and migration templates exist
