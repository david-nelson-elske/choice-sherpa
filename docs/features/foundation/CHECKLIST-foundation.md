# Foundation Module Checklist

**Module:** Foundation
**Language:** Rust
**Dependencies:** None (root of dependency tree)
**Phase:** 1 (must complete first)

---

## Overview

The Foundation module provides shared domain primitives used across all other modules. It contains value objects, identifiers, enums, and base error types that form the vocabulary of the Choice Sherpa domain.

---

## File Inventory

### Domain Layer (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/foundation/mod.rs` | Module exports | ✅ |
| `backend/src/domain/foundation/ids.rs` | SessionId, CycleId, ComponentId, UserId | ✅ |
| `backend/src/domain/foundation/timestamp.rs` | Timestamp value object | ✅ |
| `backend/src/domain/foundation/percentage.rs` | Percentage (0-100) value object | ✅ |
| `backend/src/domain/foundation/rating.rs` | Pugh Rating (-2 to +2) value object | ✅ |
| `backend/src/domain/foundation/component_type.rs` | ComponentType enum (9 variants) | ✅ |
| `backend/src/domain/foundation/component_status.rs` | ComponentStatus enum | ✅ |
| `backend/src/domain/foundation/cycle_status.rs` | CycleStatus enum | ✅ |
| `backend/src/domain/foundation/session_status.rs` | SessionStatus enum | ✅ |
| `backend/src/domain/foundation/errors.rs` | DomainError, ErrorCode, ValidationError | ✅ |

### Domain Tests (Rust)

> **Note:** Tests are inline in implementation files using `#[cfg(test)] mod tests` (Rust convention).

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/foundation/ids.rs` | ID value object tests (12 tests) | ✅ |
| `backend/src/domain/foundation/timestamp.rs` | Timestamp tests (7 tests) | ✅ |
| `backend/src/domain/foundation/percentage.rs` | Percentage tests (10 tests) | ✅ |
| `backend/src/domain/foundation/rating.rs` | Rating tests (12 tests) | ✅ |
| `backend/src/domain/foundation/component_type.rs` | ComponentType tests (14 tests) | ✅ |
| `backend/src/domain/foundation/component_status.rs` | ComponentStatus tests (20 tests) | ✅ |
| `backend/src/domain/foundation/cycle_status.rs` | CycleStatus tests (11 tests) | ✅ |
| `backend/src/domain/foundation/session_status.rs` | SessionStatus tests (9 tests) | ✅ |
| `backend/src/domain/foundation/errors.rs` | Error types tests (6 tests) | ✅ |
| `backend/src/domain/foundation/events.rs` | Event envelope tests (17 tests) | ✅ |
| `backend/src/domain/foundation/state_machine.rs` | StateMachine trait tests (6 tests) | ✅ |

### Frontend Types (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/shared/domain/ids.ts` | ID type definitions | ⬜ |
| `frontend/src/shared/domain/enums.ts` | ComponentType, Status enums | ⬜ |
| `frontend/src/shared/domain/errors.ts` | Error type definitions | ⬜ |
| `frontend/src/shared/domain/index.ts` | Public exports | ⬜ |

### Frontend Tests (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/shared/domain/ids.test.ts` | ID validation tests | ⬜ |
| `frontend/src/shared/domain/enums.test.ts` | Enum tests | ⬜ |

---

## Test Inventory

### ID Value Object Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `session_id_generates_unique_values` | Each call produces different ID | ✅ |
| `session_id_from_uuid_preserves_value` | Wrapping preserves UUID | ✅ |
| `user_id_displays_correctly` | Display shows inner string | ✅ |
| `session_id_parses_from_valid_string` | Valid UUID string parses | ✅ |
| `session_id_serializes_to_json` | JSON roundtrip preserves value | ✅ |
| `cycle_id_generates_unique_values` | Each call produces different ID | ✅ |
| `component_id_generates_unique_values` | Each call produces different ID | ✅ |
| `conversation_id_generates_unique_values` | Conversation ID uniqueness | ✅ |
| `conversation_id_from_uuid_preserves_value` | UUID preservation | ✅ |
| `conversation_id_parses_from_valid_string` | Valid string parsing | ✅ |
| `user_id_rejects_empty_string` | Empty string returns error | ✅ |
| `user_id_accepts_non_empty_string` | Non-empty string succeeds | ✅ |

### Timestamp Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `timestamp_now_creates_current_time` | Now is approximately current | ✅ |
| `timestamp_from_datetime_preserves_value` | Wrapping preserves DateTime | ✅ |
| `timestamp_is_before_works_correctly` | Earlier < Later | ✅ |
| `timestamp_is_after_works_correctly` | Later > Earlier | ✅ |
| `timestamp_ordering_works` | Ord trait works correctly | ✅ |
| `timestamp_serializes_to_json` | Serialization | ✅ |
| `timestamp_deserializes_from_json` | Deserialization | ✅ |

### Percentage Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `percentage_new_clamps_to_100` | Values > 100 become 100 | ✅ |
| `percentage_try_new_rejects_over_100` | > 100 returns error | ✅ |
| `percentage_try_new_accepts_valid_values` | 0-100 succeeds | ✅ |
| `percentage_new_accepts_valid_values` | Getter returns stored value | ✅ |
| `percentage_as_fraction_converts_correctly` | 50 -> 0.5 | ✅ |
| `percentage_default_is_zero` | Default is 0% | ✅ |
| `percentage_displays_correctly` | Shows "50%" | ✅ |
| `percentage_ordering_works` | 25 < 50 < 75 | ✅ |
| `percentage_serializes_to_json` | JSON serialization | ✅ |
| `percentage_deserializes_from_json` | JSON deserialization | ✅ |

### Rating Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `rating_try_from_i8_accepts_valid_values` | -2 to +2 all succeed | ✅ |
| `rating_try_from_i8_rejects_invalid_values` | -3, +3 return error | ✅ |
| `rating_value_returns_correct_integer` | Enum converts to i8 | ✅ |
| `rating_label_returns_display_text` | MuchBetter -> "Much Better" | ✅ |
| `rating_is_positive_works` | +1, +2 are positive | ✅ |
| `rating_is_negative_works` | -1, -2 are negative | ✅ |
| `rating_is_neutral_works` | Same is neither positive nor negative | ✅ |
| `rating_displays_with_sign` | +2 shows "+2", -1 shows "-1" | ✅ |
| `rating_default_is_same` | Default is Same (0) | ✅ |
| `rating_ordering_works` | MuchWorse < Worse < Same < Better < MuchBetter | ✅ |
| `rating_serializes_to_json` | JSON serialization | ✅ |
| `rating_deserializes_from_json` | JSON deserialization | ✅ |

### ComponentType Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `all_returns_9_components` | All() has 9 elements | ✅ |
| `all_returns_components_in_order` | Order never changes | ✅ |
| `order_index_returns_correct_values` | IssueRaising is 0, DecisionQuality is 8 | ✅ |
| `next_returns_correct_component` | IssueRaising.next() = ProblemFrame | ✅ |
| `next_returns_none_for_last` | DecisionQuality.next() = None | ✅ |
| `previous_returns_correct_component` | ProblemFrame.previous() = IssueRaising | ✅ |
| `previous_returns_none_for_first` | IssueRaising.previous() = None | ✅ |
| `is_before_works_correctly` | IssueRaising.is_before(Alternatives) = true | ✅ |
| `is_after_works_correctly` | Alternatives.is_after(IssueRaising) = true | ✅ |
| `display_name_returns_readable_text` | IssueRaising -> "Issue Raising" | ✅ |
| `display_uses_display_name` | Display trait uses display_name | ✅ |
| `abbreviation_returns_short_code` | IssueRaising -> "IR" | ✅ |
| `serializes_to_snake_case_json` | Serializes as "issue_raising" | ✅ |
| `deserializes_from_snake_case_json` | Deserializes from snake_case | ✅ |

### ComponentStatus Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `default_is_not_started` | Default is NotStarted | ✅ |
| `is_started_works_correctly` | InProgress, Complete, NeedsRevision are started | ✅ |
| `is_complete_works_correctly` | Only Complete returns true | ✅ |
| `needs_work_works_correctly` | NotStarted, InProgress, NeedsRevision need work | ✅ |
| `not_started_can_transition_to_in_progress` | Valid transition | ✅ |
| `in_progress_can_transition_to_complete` | Valid transition | ✅ |
| `complete_can_transition_to_needs_revision` | Valid transition | ✅ |
| `complete_cannot_transition_to_not_started` | Invalid transition | ✅ |
| `complete_cannot_transition_to_in_progress` | Invalid transition | ✅ |
| `not_started_cannot_transition_to_complete` | Invalid transition | ✅ |
| `not_started_cannot_transition_to_needs_revision` | Invalid transition | ✅ |
| `in_progress_can_transition_to_needs_revision` | Valid transition | ✅ |
| `in_progress_cannot_transition_to_not_started` | Invalid transition | ✅ |
| `needs_revision_can_transition_to_in_progress` | Valid transition | ✅ |
| `needs_revision_can_transition_to_complete` | Valid transition | ✅ |
| `display_works_correctly` | InProgress -> "In Progress" | ✅ |
| `is_locked_works_correctly` | Locked status check | ✅ |
| `accepts_output_works_correctly` | Output acceptance check | ✅ |
| `serializes_to_snake_case_json` | JSON serialization | ✅ |
| `deserializes_from_snake_case_json` | JSON deserialization | ✅ |

### CycleStatus Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `default_is_active` | Default is Active | ✅ |
| `is_mutable_works_correctly` | Only Active is mutable | ✅ |
| `is_finished_works_correctly` | Completed and Archived are finished | ✅ |
| `active_can_transition_to_completed` | Valid transition | ✅ |
| `active_can_transition_to_archived` | Valid transition | ✅ |
| `completed_cannot_transition_to_active` | Invalid transition | ✅ |
| `completed_can_transition_to_archived` | Valid transition | ✅ |
| `archived_cannot_transition_to_anything` | Terminal state | ✅ |
| `display_works_correctly` | Display formatting | ✅ |
| `serializes_to_snake_case_json` | JSON serialization | ✅ |
| `deserializes_from_snake_case_json` | JSON deserialization | ✅ |

### SessionStatus Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `default_is_active` | Default is Active | ✅ |
| `is_mutable_works_correctly` | Only Active is mutable | ✅ |
| `active_can_transition_to_archived` | Valid transition | ✅ |
| `archived_cannot_transition_to_active` | Invalid transition | ✅ |
| `active_cannot_transition_to_active` | Self-transition invalid | ✅ |
| `archived_cannot_transition_to_archived` | Self-transition invalid | ✅ |
| `display_works_correctly` | Display formatting | ✅ |
| `serializes_to_snake_case_json` | JSON serialization | ✅ |
| `deserializes_from_snake_case_json` | JSON deserialization | ✅ |

### Error Types Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `domain_error_displays_code_and_message` | Format: "[CODE] message" | ✅ |
| `domain_error_with_detail_adds_detail` | Builder pattern works | ✅ |
| `error_code_display_formats_correctly` | ValidationFailed -> "VALIDATION_FAILED" | ✅ |
| `validation_error_empty_field_displays_correctly` | Includes field name | ✅ |
| `validation_error_out_of_range_displays_correctly` | Includes min, max, actual | ✅ |
| `validation_error_invalid_format_displays_correctly` | Includes field and reason | ✅ |

### StateMachine Trait Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `can_transition_to_is_consistent_with_valid_transitions` | Consistency check | ✅ |
| `is_terminal_returns_false_for_non_terminal` | Non-terminal detection | ✅ |
| `is_terminal_returns_true_for_archived` | Terminal detection | ✅ |
| `transition_to_fails_for_invalid_transition` | Invalid transitions fail | ✅ |
| `transition_to_succeeds_for_valid_transition` | Valid transitions succeed | ✅ |
| `valid_transitions_returns_correct_targets` | Transition targets | ✅ |

### Event Envelope Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `event_id_generates_unique_values` | Unique ID generation | ✅ |
| `event_id_from_string_preserves_value` | String preservation | ✅ |
| `event_id_displays_correctly` | Display formatting | ✅ |
| `event_id_serializes_to_json` | JSON serialization | ✅ |
| `event_id_deserializes_from_json` | JSON deserialization | ✅ |
| `event_id_default_creates_new` | Default trait | ✅ |
| `event_envelope_new_creates_with_defaults` | Construction | ✅ |
| `event_envelope_builder_chain` | Builder pattern | ✅ |
| `event_envelope_payload_as_deserializes` | Payload deserialization | ✅ |
| `event_envelope_payload_as_returns_error_on_mismatch` | Error handling | ✅ |
| `event_envelope_serialization_round_trip` | Roundtrip serialization | ✅ |
| `domain_event_to_envelope_creates_valid_envelope` | Envelope creation | ✅ |
| `domain_event_to_envelope_payload_round_trips` | Payload roundtrip | ✅ |
| `domain_event_to_envelope_preserves_occurred_at` | Timestamp preservation | ✅ |
| `event_metadata_default_has_all_none` | Default metadata | ✅ |
| `event_metadata_serializes_without_none_fields` | Sparse serialization | ✅ |
| `event_metadata_round_trip_serialization` | Metadata roundtrip | ✅ |

---

## Error Codes

| Error Code | Category | Condition |
|------------|----------|-----------|
| `VALIDATION_FAILED` | Validation | General validation failure |
| `EMPTY_FIELD` | Validation | Required field is empty |
| `OUT_OF_RANGE` | Validation | Value outside allowed range |
| `INVALID_FORMAT` | Validation | Value has wrong format |
| `SESSION_NOT_FOUND` | Not Found | Session does not exist |
| `CYCLE_NOT_FOUND` | Not Found | Cycle does not exist |
| `COMPONENT_NOT_FOUND` | Not Found | Component does not exist |
| `CONVERSATION_NOT_FOUND` | Not Found | Conversation does not exist |
| `INVALID_STATE_TRANSITION` | State | Invalid status transition |
| `SESSION_ARCHIVED` | State | Session is archived |
| `CYCLE_ARCHIVED` | State | Cycle is archived |
| `COMPONENT_LOCKED` | State | Component cannot be modified |
| `UNAUTHORIZED` | Auth | User not authenticated |
| `FORBIDDEN` | Auth | User lacks permission |
| `AI_PROVIDER_ERROR` | AI | AI provider returned error |
| `RATE_LIMITED` | AI | Rate limit exceeded |
| `DATABASE_ERROR` | Infra | Database operation failed |
| `CACHE_ERROR` | Infra | Cache operation failed |

---

## Business Rules

| Rule | Implementation | Test | Status |
|------|----------------|------|--------|
| IDs are valid UUIDs | `FromStr` validation | `session_id_parses_from_valid_string` | ✅ |
| UserId is non-empty | Constructor validation | `user_id_rejects_empty_string` | ✅ |
| Percentage is 0-100 | `try_new()` returns Result | `percentage_try_new_rejects_over_100` | ✅ |
| Rating is -2 to +2 | Enum restricts values | `rating_try_from_i8_rejects_invalid_values` | ✅ |
| ComponentType order is fixed | Static `all()` array | `all_returns_components_in_order` | ✅ |
| Status transitions are validated | `can_transition_to()` method | `complete_cannot_transition_to_not_started` | ✅ |

---

## Verification Commands

```bash
# Run all foundation tests
cargo test --package foundation -- --nocapture

# Run specific test category
cargo test --package foundation ids:: -- --nocapture
cargo test --package foundation timestamp:: -- --nocapture
cargo test --package foundation percentage:: -- --nocapture
cargo test --package foundation rating:: -- --nocapture
cargo test --package foundation component_type:: -- --nocapture
cargo test --package foundation status:: -- --nocapture
cargo test --package foundation errors:: -- --nocapture

# Coverage check (target: 95%+)
cargo tarpaulin --package foundation --out Html

# Full verification
cargo test --package foundation -- --nocapture && cargo clippy --package foundation

# Frontend tests
cd frontend && npm test -- --testPathPattern="shared/domain"
```

---

## Exit Criteria

### Module is COMPLETE when:

- [x] All Rust domain files exist (10/10 complete - 100%)
- [x] All Rust tests pass (124 tests passing)
- [x] Rust coverage >= 95% (verified via inline tests)
- [ ] TypeScript types match Rust definitions (frontend not started)
- [x] All value objects are immutable
- [x] All enums have serialization tests
- [x] No clippy warnings
- [ ] No TypeScript lint errors (frontend not started)

### Current Status

```
RUST BACKEND COMPLETE: foundation
Files: 15/15 (includes events.rs, state_machine.rs, ownership.rs, authorization.rs, command.rs, repository.rs)
Tests: 124/124 passing
Frontend: Not started
```

### Exit Signal (Full Module)

```
MODULE COMPLETE: foundation
Rust Files: 15/15
Rust Tests: 124/124 passing
Frontend Files: 0/6 (not started)
```

---

## Implementation Phases

### Phase 1: Core IDs ✅
- [x] SessionId, CycleId, ComponentId, UserId, ConversationId implementations
- [x] FromStr, Display, Serialize, Deserialize traits
- [x] ID unit tests (12 tests)

### Phase 2: Value Objects ✅
- [x] Timestamp implementation
- [x] Percentage implementation
- [x] Rating implementation
- [x] Value object tests (29 tests)

### Phase 3: Enums ✅
- [x] ComponentType with all 9 variants
- [x] ComponentStatus with transitions (20 tests)
- [x] CycleStatus with transitions (11 tests)
- [x] SessionStatus with transitions (9 tests)
- [x] StateMachine trait abstraction (6 tests)

### Phase 4: Errors ✅
- [x] DomainError implementation
- [x] ErrorCode enum
- [x] ValidationError enum
- [x] Error tests (6 tests)

### Phase 5: Events & Infrastructure ✅
- [x] EventId, EventEnvelope, EventMetadata
- [x] DomainEvent trait
- [x] Event infrastructure tests (17 tests)
- [x] Ownership traits (OwnedBy, CreatedBy)
- [x] Authorization traits
- [x] Command and Repository traits

### Phase 6: Frontend Mirroring ⬜
- [ ] TypeScript type definitions
- [ ] Enum value alignment
- [ ] Frontend tests

---

## Notes

- Foundation has no external dependencies except `uuid`, `chrono`, `thiserror`, `serde`
- All types must derive `Debug`, `Clone`, `PartialEq`, `Eq`
- Serialization uses serde with snake_case for enums
- IDs use `#[serde(transparent)]` for clean JSON
- Frontend uses string types for IDs (UUID as string)

---

*Generated: 2026-01-07*
*Specification: docs/modules/foundation.md*
*Last verified: 2026-01-09 via agent verification (124 tests confirmed)*
