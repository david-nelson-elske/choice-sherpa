# Event Versioning Guide

## Overview

This guide describes the event versioning system for Choice Sherpa, enabling safe schema evolution and backward compatibility for domain events.

**Status:** ✅ Implemented (Phase 1-4 Complete)

## Table of Contents

- [Architecture](#architecture)
- [Core Concepts](#core-concepts)
- [Quick Start](#quick-start)
- [Creating a New Event Version](#creating-a-new-event-version)
- [Writing Upcasters](#writing-upcasters)
- [Testing Event Versions](#testing-event-versions)
- [Best Practices](#best-practices)
- [Migration Checklist](#migration-checklist)
- [Troubleshooting](#troubleshooting)

---

## Architecture

Event versioning consists of four core components:

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ EventEnvelope   │───▶│ UpcasterRegistry │───▶│ EventDeserializer│
│ (schema_version)│    │  (v1→v2→v3)      │    │  (automatic)     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                      │                        │
         │                      │                        ▼
         ▼                      ▼              ┌──────────────────┐
┌─────────────────┐    ┌──────────────────┐   │  Event Handler  │
│   Event Store   │    │  EventReplayer   │   │  (current ver)  │
│ (with versions) │    │  (rebuilding)    │   └──────────────────┘
└─────────────────┘    └──────────────────┘
```

### Components

| Component | Purpose |
|-----------|---------|
| **EventEnvelope** | Wraps events with schema_version field |
| **DomainEvent trait** | Defines schema_version() method |
| **Upcaster trait** | Transforms events from v1→v2 |
| **UpcasterRegistry** | Chains multiple upcasters (v1→v2→v3) |
| **EventDeserializer** | Deserializes with automatic upcasting |
| **EventReplayer** | Replays historical events with upcasting |

---

## Core Concepts

### Event Versions

Every event has two representations of its version:

1. **Event Type String** - Includes version suffix (e.g., `"session.created.v2"`)
2. **Schema Version Number** - Integer field in envelope (e.g., `2`)

Both MUST match. The domain_event! macro enforces this.

### Version Naming

Events follow the pattern: `<aggregate>.<event>.v<number>`

```
session.created.v1        ✅ Correct
session.created.v2        ✅ Correct
session.created          ❌ Missing version (legacy only)
session.created.2        ❌ Wrong format
```

### Backward Compatibility

**Golden Rule:** New event versions MUST be upcasted from ALL previous versions.

```
v1 ──[Upcaster]──▶ v2 ──[Upcaster]──▶ v3
```

Consumers only see v3. Old events are automatically upcasted.

---

## Quick Start

### 1. Using Events (Consumer)

Most consumers only need EventDeserializer:

```rust
use choice_sherpa::domain::foundation::{EventDeserializer, UpcasterRegistry};

// Setup (usually done once at app startup)
let mut registry = UpcasterRegistry::new();
registry.register(Arc::new(SessionCreatedV1ToV2));
registry.register(Arc::new(SessionCreatedV2ToV3));
registry.set_current_version("session.created", 3);

let deserializer = EventDeserializer::new(registry);

// Usage (in event handlers)
let envelope = load_from_event_store("evt-123")?;
let event: SessionCreatedV3 = deserializer.deserialize(envelope)?;
// Event is automatically upcasted from v1→v2→v3 if needed
```

### 2. Defining Events (Producer)

Use the domain_event! macro:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreated {
    pub event_id: EventId,
    pub session_id: SessionId,
    pub title: String,
    pub description: Option<String>, // Added in v2
    pub created_at: Timestamp,
}

domain_event!(
    SessionCreated,
    event_type = "session.created.v2",  // ← Include version
    schema_version = 2,                  // ← Must match suffix
    aggregate_id = session_id,
    aggregate_type = "Session",
    occurred_at = created_at,
    event_id = event_id
);
```

---

## Creating a New Event Version

### Step 1: Define New Event Structure

Create a new version with your schema changes:

```rust
// OLD: SessionCreated v1
pub struct SessionCreated {
    pub event_id: EventId,
    pub session_id: SessionId,
    pub title: String,
    pub created_at: Timestamp,
}

// NEW: SessionCreated v2 (adds optional description)
pub struct SessionCreated {
    pub event_id: EventId,
    pub session_id: SessionId,
    pub title: String,
    pub description: Option<String>,  // NEW FIELD
    pub created_at: Timestamp,
}
```

### Step 2: Update Event Declaration

Update the domain_event! macro:

```rust
domain_event!(
    SessionCreated,
    event_type = "session.created.v2",  // v1 → v2
    schema_version = 2,                  // 1 → 2
    aggregate_id = session_id,
    aggregate_type = "Session",
    occurred_at = created_at,
    event_id = event_id
);
```

### Step 3: Create Upcaster

Write an upcaster to transform v1→v2:

```rust
pub struct SessionCreatedV1ToV2;

impl Upcaster for SessionCreatedV1ToV2 {
    fn source_type(&self) -> &str {
        "session.created.v1"
    }

    fn target_type(&self) -> &str {
        "session.created.v2"
    }

    fn upcast(&self, mut payload: JsonValue) -> Result<JsonValue, UpcastError> {
        // Add new optional field with default value
        payload["description"] = JsonValue::Null;
        Ok(payload)
    }
}
```

### Step 4: Register Upcaster

Register at application startup:

```rust
registry.register(Arc::new(SessionCreatedV1ToV2));
registry.set_current_version("session.created", 2);
```

### Step 5: Update Tests

Update test assertions to expect v2:

```rust
assert_eq!(event.event_type(), "session.created.v2");
assert_eq!(envelope.schema_version, 2);
```

---

## Writing Upcasters

### Basic Upcaster

Add a new optional field:

```rust
impl Upcaster for SessionCreatedV1ToV2 {
    fn source_type(&self) -> &str { "session.created.v1" }
    fn target_type(&self) -> &str { "session.created.v2" }

    fn upcast(&self, mut payload: JsonValue) -> Result<JsonValue, UpcastError> {
        // Add optional description field (defaults to null)
        payload["description"] = JsonValue::Null;
        Ok(payload)
    }
}
```

### Field Transformation

Transform an existing field:

```rust
impl Upcaster for SessionCreatedV2ToV3 {
    fn source_type(&self) -> &str { "session.created.v2" }
    fn target_type(&self) -> &str { "session.created.v3" }

    fn upcast(&self, mut payload: JsonValue) -> Result<JsonValue, UpcastError> {
        // Transform user_id into owner object
        let user_id = payload
            .get("user_id")
            .ok_or_else(|| UpcastError::MissingField("user_id".into()))?
            .clone();

        payload["owner"] = json!({
            "user_id": user_id,
            "display_name": "Unknown", // Default value
            "role": "Creator"
        });

        // Remove old field
        payload.as_object_mut().unwrap().remove("user_id");

        Ok(payload)
    }
}
```

### Field Renaming

Rename a field:

```rust
impl Upcaster for CycleCompletedV1ToV2 {
    fn source_type(&self) -> &str { "cycle.completed.v1" }
    fn target_type(&self) -> &str { "cycle.completed.v2" }

    fn upcast(&self, mut payload: JsonValue) -> Result<JsonValue, UpcastError> {
        // Rename field: finish_time → completed_at
        if let Some(obj) = payload.as_object_mut() {
            if let Some(value) = obj.remove("finish_time") {
                obj.insert("completed_at".to_string(), value);
            }
        }
        Ok(payload)
    }
}
```

### Complex Migration

Multiple field changes:

```rust
impl Upcaster for MembershipCreatedV2ToV3 {
    fn source_type(&self) -> &str { "membership.created.v2" }
    fn target_type(&self) -> &str { "membership.created.v3" }

    fn upcast(&self, mut payload: JsonValue) -> Result<JsonValue, UpcastError> {
        // 1. Rename field
        if let Some(obj) = payload.as_object_mut() {
            if let Some(tier) = obj.remove("plan_tier") {
                obj.insert("tier".to_string(), tier);
            }
        }

        // 2. Add new required field with computed value
        let is_free = payload
            .get("tier")
            .and_then(|t| t.as_str())
            .map(|t| t == "Free")
            .unwrap_or(false);
        payload["is_free"] = json!(is_free);

        // 3. Transform nested object
        if let Some(metadata) = payload.get_mut("metadata") {
            metadata["version"] = json!("3.0");
        }

        Ok(payload)
    }
}
```

---

## Testing Event Versions

### Unit Tests for Upcasters

Test each upcaster independently:

```rust
#[test]
fn session_created_v1_to_v2_adds_description() {
    let upcaster = SessionCreatedV1ToV2;

    let v1_payload = json!({
        "event_id": "evt-123",
        "session_id": "session-456",
        "title": "My Decision",
        "created_at": "2026-01-11T00:00:00Z"
    });

    let v2_payload = upcaster.upcast(v1_payload).unwrap();

    // New field added
    assert!(v2_payload["description"].is_null());

    // Existing fields preserved
    assert_eq!(v2_payload["title"], "My Decision");
    assert_eq!(v2_payload["session_id"], "session-456");
}
```

### Integration Tests

Test the full chain:

```rust
#[test]
fn v1_event_deserializes_to_v3() {
    let mut registry = UpcasterRegistry::new();
    registry.register(Arc::new(SessionCreatedV1ToV2));
    registry.register(Arc::new(SessionCreatedV2ToV3));
    registry.set_current_version("session.created", 3);

    let deserializer = EventDeserializer::new(registry);

    // V1 envelope from storage
    let v1_envelope = EventEnvelope {
        event_id: EventId::new(),
        event_type: "session.created.v1".to_string(),
        schema_version: 1,
        aggregate_id: "session-123".to_string(),
        aggregate_type: "Session".to_string(),
        occurred_at: Timestamp::now(),
        payload: json!({
            "event_id": "evt-123",
            "session_id": "session-123",
            "title": "Test",
            "user_id": "user-456",
            "created_at": "2026-01-11T00:00:00Z"
        }),
        metadata: EventMetadata::default(),
    };

    // Deserialize to v3
    let event: SessionCreatedV3 = deserializer.deserialize(v1_envelope).unwrap();

    assert_eq!(event.title, "Test");
    assert_eq!(event.owner.user_id, "user-456");
    assert!(event.description.is_none());
}
```

### Replay Tests

Test event replay:

```rust
#[test]
fn replay_rebuilds_projection_correctly() {
    let mut registry = UpcasterRegistry::new();
    registry.register(Arc::new(SessionCreatedV1ToV2));
    registry.set_current_version("session.created", 2);

    let replayer = EventReplayer::new(registry);

    let events = vec![/* v1 events from storage */];

    let mut session_index = HashMap::new();

    let stats = replayer.replay_events(events, |envelope| {
        let event: SessionCreatedV2 = deserializer.deserialize(envelope)?;
        session_index.insert(event.session_id.clone(), event);
        Ok(true)
    })?;

    assert_eq!(stats.processed, 10);
    assert_eq!(session_index.len(), 10);
}
```

---

## Best Practices

### DO ✅

1. **Always version new events**
   - Use `event_type = "session.created.v1"` from day one
   - Include `schema_version = 1` in domain_event! macro

2. **Make new fields optional**
   - Use `Option<T>` for new fields when possible
   - Upcasters can default to `None`/`null`

3. **Preserve old data**
   - Don't delete information during upcasting
   - Keep deprecated fields if removing them loses data

4. **Test upcaster chains**
   - Test v1→v2, v2→v3, AND v1→v3
   - Ensure transitive property holds

5. **Document breaking changes**
   - Add comments explaining why version changed
   - Document upcaster logic

6. **Use semantic versioning**
   - v1, v2, v3 (sequential)
   - Don't skip versions

### DON'T ❌

1. **Don't change existing event versions**
   - Never modify v1 schema after events exist
   - Create v2 instead

2. **Don't make fields required without defaults**
   - Old events won't have the field
   - Upcaster needs a sensible default

3. **Don't delete upcasters**
   - Keep all upcasters in the codebase
   - Events may still exist in v1 format

4. **Don't skip version numbers**
   - Go v1→v2→v3, not v1→v3
   - Registry expects sequential chains

5. **Don't change event semantics**
   - v2 should mean the same thing as v1
   - Different meaning = different event type

6. **Don't make upcasters stateful**
   - Upcasters must be pure functions
   - Same input → same output

---

## Migration Checklist

Use this checklist when creating a new event version:

### Planning Phase

- [ ] Identify events that need new version
- [ ] Document schema changes and rationale
- [ ] Design backward-compatible migration path
- [ ] Check if any handlers need updates

### Implementation Phase

- [ ] Create new event struct with schema changes
- [ ] Update domain_event! macro (event_type + schema_version)
- [ ] Write upcaster implementation (Upcaster trait)
- [ ] Write upcaster unit tests
- [ ] Register upcaster in application startup
- [ ] Update UpcasterRegistry current version

### Testing Phase

- [ ] Run all upcaster unit tests
- [ ] Test deserialization from old version
- [ ] Test upcaster chain (v1→v2→v3)
- [ ] Test replay with old events
- [ ] Update all test assertions to expect new version
- [ ] Run full integration test suite

### Deployment Phase

- [ ] Run database migration (if schema changed)
- [ ] Deploy new code with upcaster
- [ ] Monitor for deserialization errors
- [ ] Verify old events still work
- [ ] Document version in changelog

### Post-Deployment

- [ ] Monitor replay success rate
- [ ] Check for any failed upcasts
- [ ] Validate projections rebuilt correctly
- [ ] Update API documentation (if applicable)

---

## Troubleshooting

### Error: "incompatible version transition"

**Problem:** No upcaster exists for the version gap.

```
Error: incompatible version transition: session.created.v1 → session.created.v3
```

**Solution:** Register missing upcaster or ensure chain is complete:

```rust
registry.register(Arc::new(SessionCreatedV1ToV2)); // Missing!
registry.register(Arc::new(SessionCreatedV2ToV3));
```

### Error: "missing required field"

**Problem:** Upcaster expects a field that doesn't exist in old version.

```rust
Error: missing required field: user_id
```

**Solution:** Add validation and fallback:

```rust
let user_id = payload
    .get("user_id")
    .or_else(|| payload.get("creator_id")) // Fallback to old field
    .ok_or_else(|| UpcastError::MissingField("user_id".into()))?;
```

### Error: "parse failed"

**Problem:** Deserialization failed after upcasting.

**Solution:** Check that:
1. Upcaster output matches target schema
2. All required fields are present
3. Field types match (string vs number)

```rust
// Debug: Print upcasted payload
let upcasted = registry.upcast_to_current(envelope)?;
println!("Upcasted payload: {}", serde_json::to_string_pretty(&upcasted.payload)?);
```

### Performance Issues

**Problem:** Upcasting is slow for large replays.

**Solution:**
1. Optimize upcaster implementations (avoid cloning)
2. Use `deserialize_raw()` if handler supports multiple versions
3. Consider migrating old events in background job

### Version Mismatch

**Problem:** `event_type` version doesn't match `schema_version`.

```
Event: session.created.v2
Schema Version: 1  ❌ MISMATCH
```

**Solution:** Ensure domain_event! macro has matching values:

```rust
domain_event!(
    SessionCreated,
    event_type = "session.created.v2",  // Must match ↓
    schema_version = 2,                  // Must match ↑
    // ...
);
```

---

## Reference

### Event Versioning Files

```
backend/src/domain/foundation/
├── events.rs              # EventEnvelope, DomainEvent trait
├── upcaster.rs            # Upcaster, Registry, Deserializer, Replayer
└── mod.rs                 # Public exports

backend/migrations/
└── 20260111000003_add_schema_version_to_outbox.sql

docs/architecture/
└── EVENT-VERSIONING-GUIDE.md  # This file

features/integrations/
└── event-versioning.md    # Feature specification
```

### Key Types

| Type | Module | Purpose |
|------|--------|---------|
| `EventEnvelope` | foundation::events | Event wrapper with schema_version |
| `DomainEvent` | foundation::events | Trait with schema_version() method |
| `Upcaster` | foundation::upcaster | Trait for version transformations |
| `UpcasterRegistry` | foundation::upcaster | Registry and chaining |
| `EventDeserializer` | foundation::upcaster | Automatic deserialization |
| `EventReplayer` | foundation::upcaster | Event replay with stats |
| `UpcastError` | foundation::upcaster | Upcaster error types |
| `DeserializeError` | foundation::upcaster | Deserialization errors |
| `ReplayStats` | foundation::upcaster | Replay statistics |

### Macro

```rust
domain_event!(
    EventName,
    event_type = "aggregate.event.v1",
    schema_version = 1,
    aggregate_id = field_name,
    aggregate_type = "AggregateType",
    occurred_at = timestamp_field,
    event_id = event_id_field
);
```

---

## Examples

See test code for complete examples:
- `backend/src/domain/foundation/upcaster.rs` - Complete test suite
- `backend/src/domain/session/events.rs` - v1 event examples
- `backend/src/domain/membership/events.rs` - Enum event example

---

## Support

For questions or issues:
1. Check this guide first
2. Review test examples in `upcaster.rs`
3. Check feature spec: `features/integrations/event-versioning.md`
4. Open issue on GitHub with [Event Versioning] tag

---

**Last Updated:** 2026-01-11
**Version:** 1.0
**Status:** ✅ Complete (Phases 1-4)
