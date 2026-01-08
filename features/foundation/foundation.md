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
