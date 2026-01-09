# Feature: Foundation Module - Core Types

> Shared domain primitives used across all modules: value objects, identifiers, enums, and error types that form the vocabulary of the Choice Sherpa domain.

## Context

- This is the **root** of the dependency tree - no domain dependencies
- **Shared Domain** module (types only, no ports/adapters)
- External dependencies: `uuid`, `chrono`, `thiserror`, `serde`
- All other modules depend on this module
- Types must be `Send + Sync` for async compatibility
- Must support serialization for API transport and persistence

## Tasks

- [x] Initialize Rust backend project with Cargo.toml
- [x] Create foundation module structure with mod.rs
- [x] Implement ValidationError enum with factory methods
- [x] Implement DomainError struct with ErrorCode enum
- [x] Implement SessionId value object with UUID
- [x] Implement CycleId value object with UUID
- [x] Implement ComponentId value object with UUID
- [x] Implement UserId value object with non-empty validation
- [x] Implement Timestamp value object with chrono DateTime<Utc>
- [x] Implement Percentage value object (0-100 clamped)
- [x] Implement Rating enum (-2 to +2 Pugh scale)
- [x] Implement ComponentType enum with 9 variants and ordering methods
- [x] Implement ComponentStatus enum with transition validation
- [x] Implement CycleStatus enum with transition validation
- [x] Implement SessionStatus enum with transition validation

## Acceptance Criteria

- [x] All ID types implement Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize
- [x] All ID types implement Display and FromStr
- [x] SessionId, CycleId, ComponentId generate unique UUIDs via new()
- [x] UserId rejects empty strings with ValidationError
- [x] Timestamp::now() returns current UTC time
- [x] Percentage::new(value) clamps to 0-100 range
- [x] Percentage::try_new(value) returns error for values > 100
- [x] Rating::try_from_i8 returns error for values outside -2 to +2
- [x] ComponentType::all() returns all 9 variants in canonical order
- [x] ComponentType::order_index() returns stable indices 0-8
- [x] ComponentType::next() and previous() navigate ordering correctly
- [x] ComponentStatus::can_transition_to() validates state machine rules
- [x] CycleStatus::can_transition_to() validates state machine rules
- [x] SessionStatus::can_transition_to() validates state machine rules
- [x] DomainError supports method chaining via with_detail()
- [x] All tests pass with `cargo test`
- [x] Code compiles with no warnings

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Not Required (shared types, no endpoints) |
| Authorization Model | N/A - types used by authenticated modules |
| Sensitive Data | UserId (Internal), EventMetadata.user_id (Internal) |
| Rate Limiting | Not Required (no endpoints) |
| Audit Logging | N/A - types only, logging handled by consuming modules |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| UserId | Internal | Do not expose in public API responses without authorization |
| SessionId | Internal | Opaque identifier, safe to include in URLs |
| CycleId | Internal | Opaque identifier, safe to include in URLs |
| ComponentId | Internal | Opaque identifier, safe to include in URLs |
| EventMetadata.user_id | Internal | Implement custom `Debug` trait to redact in logs |

### Implementation Notes

1. **Custom Debug for EventMetadata**: The `user_id` field in `EventMetadata` should be redacted when logged:

```rust
impl fmt::Debug for EventMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventMetadata")
            .field("correlation_id", &self.correlation_id)
            .field("causation_id", &self.causation_id)
            .field("user_id", &self.user_id.as_ref().map(|_| "[REDACTED]"))
            .field("trace_id", &self.trace_id)
            .finish()
    }
}
```

2. **ID Opacity**: All ID types use UUIDs to prevent enumeration attacks. IDs should not encode any user-identifiable information.

3. **Serialization Security**: All types implement `Serialize`/`Deserialize` for API transport. Ensure deserialization validates input bounds (e.g., Percentage 0-100, Rating -2 to +2)
