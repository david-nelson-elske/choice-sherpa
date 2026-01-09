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
| `backend/src/domain/foundation/mod.rs` | Module exports | ⬜ |
| `backend/src/domain/foundation/ids.rs` | SessionId, CycleId, ComponentId, UserId | ⬜ |
| `backend/src/domain/foundation/timestamp.rs` | Timestamp value object | ⬜ |
| `backend/src/domain/foundation/percentage.rs` | Percentage (0-100) value object | ⬜ |
| `backend/src/domain/foundation/rating.rs` | Pugh Rating (-2 to +2) value object | ⬜ |
| `backend/src/domain/foundation/component_type.rs` | ComponentType enum (9 variants) | ⬜ |
| `backend/src/domain/foundation/component_status.rs` | ComponentStatus enum | ⬜ |
| `backend/src/domain/foundation/cycle_status.rs` | CycleStatus enum | ⬜ |
| `backend/src/domain/foundation/session_status.rs` | SessionStatus enum | ⬜ |
| `backend/src/domain/foundation/errors.rs` | DomainError, ErrorCode, ValidationError | ⬜ |

### Domain Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/foundation/ids_test.rs` | ID value object tests | ⬜ |
| `backend/src/domain/foundation/timestamp_test.rs` | Timestamp tests | ⬜ |
| `backend/src/domain/foundation/percentage_test.rs` | Percentage tests | ⬜ |
| `backend/src/domain/foundation/rating_test.rs` | Rating tests | ⬜ |
| `backend/src/domain/foundation/component_type_test.rs` | ComponentType tests | ⬜ |
| `backend/src/domain/foundation/status_test.rs` | Status enums tests | ⬜ |
| `backend/src/domain/foundation/errors_test.rs` | Error types tests | ⬜ |

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
| `test_session_id_new_generates_unique` | Each call produces different ID | ⬜ |
| `test_session_id_from_uuid_preserves_value` | Wrapping preserves UUID | ⬜ |
| `test_session_id_display_formats_correctly` | Display shows UUID string | ⬜ |
| `test_session_id_from_str_parses_valid` | Valid UUID string parses | ⬜ |
| `test_session_id_from_str_rejects_invalid` | Invalid string returns error | ⬜ |
| `test_session_id_equality` | Same UUID produces equal IDs | ⬜ |
| `test_session_id_hash_consistent` | Equal IDs have equal hashes | ⬜ |
| `test_session_id_serialize_deserialize` | JSON roundtrip preserves value | ⬜ |
| `test_cycle_id_new_generates_unique` | Each call produces different ID | ⬜ |
| `test_cycle_id_from_str_parses_valid` | Valid UUID string parses | ⬜ |
| `test_component_id_new_generates_unique` | Each call produces different ID | ⬜ |
| `test_component_id_from_str_parses_valid` | Valid UUID string parses | ⬜ |
| `test_user_id_new_rejects_empty` | Empty string returns error | ⬜ |
| `test_user_id_new_accepts_valid` | Non-empty string succeeds | ⬜ |
| `test_user_id_display_formats_correctly` | Display shows inner string | ⬜ |

### Timestamp Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_timestamp_now_returns_current_time` | Now is approximately current | ⬜ |
| `test_timestamp_from_datetime_preserves_value` | Wrapping preserves DateTime | ⬜ |
| `test_timestamp_is_before_returns_true_for_earlier` | Earlier < Later | ⬜ |
| `test_timestamp_is_after_returns_true_for_later` | Later > Earlier | ⬜ |
| `test_timestamp_ordering_is_consistent` | Ord trait works correctly | ⬜ |
| `test_timestamp_serialize_deserialize` | JSON roundtrip preserves value | ⬜ |

### Percentage Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_percentage_new_clamps_to_100` | Values > 100 become 100 | ⬜ |
| `test_percentage_try_new_rejects_over_100` | > 100 returns error | ⬜ |
| `test_percentage_try_new_accepts_valid` | 0-100 succeeds | ⬜ |
| `test_percentage_value_returns_inner` | Getter returns stored value | ⬜ |
| `test_percentage_as_fraction_converts` | 50 -> 0.5 | ⬜ |
| `test_percentage_zero_constant` | ZERO is 0% | ⬜ |
| `test_percentage_hundred_constant` | HUNDRED is 100% | ⬜ |
| `test_percentage_display_formats_with_percent` | Shows "50%" | ⬜ |
| `test_percentage_ordering` | 25 < 50 < 75 | ⬜ |

### Rating Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_rating_try_from_i8_valid_values` | -2 to +2 all succeed | ⬜ |
| `test_rating_try_from_i8_invalid_values` | -3, +3 return error | ⬜ |
| `test_rating_value_returns_numeric` | Enum converts to i8 | ⬜ |
| `test_rating_label_returns_text` | MuchBetter -> "Much Better" | ⬜ |
| `test_rating_is_positive_for_plus_values` | +1, +2 are positive | ⬜ |
| `test_rating_is_negative_for_minus_values` | -1, -2 are negative | ⬜ |
| `test_rating_same_is_neutral` | Same is neither positive nor negative | ⬜ |
| `test_rating_display_shows_sign` | +2 shows "+2", -1 shows "-1" | ⬜ |
| `test_rating_default_is_same` | Default is Same (0) | ⬜ |
| `test_rating_ordering` | MuchWorse < Worse < Same < Better < MuchBetter | ⬜ |

### ComponentType Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_component_type_all_returns_nine` | All() has 9 elements | ⬜ |
| `test_component_type_order_is_stable` | Order never changes | ⬜ |
| `test_component_type_order_index_matches_position` | IssueRaising is 0, NotesNextSteps is 8 | ⬜ |
| `test_component_type_next_returns_successor` | IssueRaising.next() = ProblemFrame | ⬜ |
| `test_component_type_next_returns_none_for_last` | NotesNextSteps.next() = None | ⬜ |
| `test_component_type_previous_returns_predecessor` | ProblemFrame.previous() = IssueRaising | ⬜ |
| `test_component_type_previous_returns_none_for_first` | IssueRaising.previous() = None | ⬜ |
| `test_component_type_is_before_returns_true` | IssueRaising.is_before(Alternatives) = true | ⬜ |
| `test_component_type_display_name_returns_text` | IssueRaising -> "Issue Raising" | ⬜ |
| `test_component_type_abbreviation_returns_short` | IssueRaising -> "IR" | ⬜ |
| `test_component_type_serialize_snake_case` | Serializes as "issue_raising" | ⬜ |

### ComponentStatus Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_component_status_default_is_not_started` | Default is NotStarted | ⬜ |
| `test_component_status_is_started_for_active` | InProgress, Complete, NeedsRevision are started | ⬜ |
| `test_component_status_is_complete_only_for_complete` | Only Complete returns true | ⬜ |
| `test_component_status_needs_work_for_incomplete` | NotStarted, InProgress, NeedsRevision need work | ⬜ |
| `test_component_status_can_transition_not_started_to_in_progress` | Valid transition | ⬜ |
| `test_component_status_can_transition_in_progress_to_complete` | Valid transition | ⬜ |
| `test_component_status_can_transition_complete_to_needs_revision` | Valid transition | ⬜ |
| `test_component_status_cannot_transition_complete_to_not_started` | Invalid transition | ⬜ |
| `test_component_status_display_formats_correctly` | InProgress -> "In Progress" | ⬜ |

### CycleStatus Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_cycle_status_default_is_active` | Default is Active | ⬜ |
| `test_cycle_status_is_mutable_for_active` | Only Active is mutable | ⬜ |
| `test_cycle_status_is_finished_for_completed_and_archived` | Completed and Archived are finished | ⬜ |
| `test_cycle_status_can_transition_active_to_completed` | Valid transition | ⬜ |
| `test_cycle_status_can_transition_active_to_archived` | Valid transition | ⬜ |
| `test_cycle_status_cannot_transition_completed_to_active` | Invalid transition | ⬜ |

### SessionStatus Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_session_status_default_is_active` | Default is Active | ⬜ |
| `test_session_status_is_mutable_for_active` | Only Active is mutable | ⬜ |
| `test_session_status_can_transition_active_to_archived` | Valid transition | ⬜ |
| `test_session_status_cannot_transition_archived_to_active` | Invalid transition | ⬜ |

### Error Types Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_domain_error_new_creates_with_code_and_message` | Basic construction | ⬜ |
| `test_domain_error_with_detail_adds_entry` | Builder pattern works | ⬜ |
| `test_domain_error_display_shows_code_and_message` | Format: "[CODE] message" | ⬜ |
| `test_error_code_display_uppercase_snake_case` | ValidationFailed -> "VALIDATION_FAILED" | ⬜ |
| `test_validation_error_empty_field_message` | Includes field name | ⬜ |
| `test_validation_error_out_of_range_message` | Includes min, max, actual | ⬜ |
| `test_validation_error_invalid_format_message` | Includes field and reason | ⬜ |

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
| IDs are valid UUIDs | `FromStr` validation | `test_session_id_from_str_rejects_invalid` | ⬜ |
| UserId is non-empty | Constructor validation | `test_user_id_new_rejects_empty` | ⬜ |
| Percentage is 0-100 | `try_new()` returns Result | `test_percentage_try_new_rejects_over_100` | ⬜ |
| Rating is -2 to +2 | Enum restricts values | `test_rating_try_from_i8_invalid_values` | ⬜ |
| ComponentType order is fixed | Static `all()` array | `test_component_type_order_is_stable` | ⬜ |
| Status transitions are validated | `can_transition_to()` method | `test_component_status_cannot_transition_complete_to_not_started` | ⬜ |

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

- [ ] All 23 files in File Inventory exist
- [ ] All 58 tests in Test Inventory pass
- [ ] Rust coverage >= 95%
- [ ] TypeScript types match Rust definitions
- [ ] All value objects are immutable
- [ ] All enums have serialization tests
- [ ] No clippy warnings
- [ ] No TypeScript lint errors

### Exit Signal

```
MODULE COMPLETE: foundation
Files: 23/23
Tests: 58/58 passing
Coverage: 96%
```

---

## Implementation Phases

### Phase 1: Core IDs
- [ ] SessionId, CycleId, ComponentId, UserId implementations
- [ ] FromStr, Display, Serialize, Deserialize traits
- [ ] ID unit tests

### Phase 2: Value Objects
- [ ] Timestamp implementation
- [ ] Percentage implementation
- [ ] Rating implementation
- [ ] Value object tests

### Phase 3: Enums
- [ ] ComponentType with all 9 variants
- [ ] ComponentStatus with transitions
- [ ] CycleStatus with transitions
- [ ] SessionStatus with transitions
- [ ] Enum tests

### Phase 4: Errors
- [ ] DomainError implementation
- [ ] ErrorCode enum
- [ ] ValidationError enum
- [ ] Error tests

### Phase 5: Frontend Mirroring
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
