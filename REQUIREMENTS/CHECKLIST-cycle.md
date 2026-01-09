# Cycle Module Checklist

**Module:** Cycle
**Language:** Rust
**Dependencies:** foundation, proact-types, session
**Phase:** 3 (parallel with conversation, analysis)

---

## Overview

The Cycle module manages the Cycle aggregate - a complete or partial path through PrOACT. The Cycle is the aggregate root that owns and persists all components as child entities. This module supports branching for "what-if" exploration without losing work.

---

## File Inventory

### Domain Layer (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/cycle/mod.rs` | Module exports | ✅ |
| `backend/src/domain/cycle/aggregate.rs` | Cycle aggregate (38 tests inline) | ✅ |
| `backend/src/domain/cycle/progress.rs` | CycleProgress value object | ⬜ |
| `backend/src/domain/cycle/events.rs` | CycleEvent enum | ✅ |
| `backend/src/domain/cycle/errors.rs` | Cycle-specific errors | ⬜ |

> **Note:** Tests are inline in implementation files using `#[cfg(test)] mod tests` (Rust convention). The file `cycle.rs` was renamed to `aggregate.rs`.

### Domain Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/cycle/cycle_test.rs` | Cycle aggregate tests | ⬜ |
| `backend/src/domain/cycle/progress_test.rs` | CycleProgress tests | ⬜ |
| `backend/src/domain/cycle/events_test.rs` | CycleEvent tests | ⬜ |

### Ports (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/ports/cycle_repository.rs` | CycleRepository trait | ⬜ |
| `backend/src/ports/cycle_reader.rs` | CycleReader trait (CQRS) | ⬜ |

### Application Layer - Commands (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/commands/create_cycle.rs` | CreateCycle handler | ⬜ |
| `backend/src/application/commands/branch_cycle.rs` | BranchCycle handler | ⬜ |
| `backend/src/application/commands/start_component.rs` | StartComponent handler | ⬜ |
| `backend/src/application/commands/complete_component.rs` | CompleteComponent handler | ⬜ |
| `backend/src/application/commands/update_component_output.rs` | UpdateComponentOutput handler | ⬜ |
| `backend/src/application/commands/navigate_component.rs` | NavigateToComponent handler | ⬜ |
| `backend/src/application/commands/complete_cycle.rs` | CompleteCycle handler | ⬜ |
| `backend/src/application/commands/archive_cycle.rs` | ArchiveCycle handler | ⬜ |

### Application Layer - Command Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/commands/create_cycle_test.rs` | CreateCycle tests | ⬜ |
| `backend/src/application/commands/branch_cycle_test.rs` | BranchCycle tests | ⬜ |
| `backend/src/application/commands/start_component_test.rs` | StartComponent tests | ⬜ |
| `backend/src/application/commands/complete_component_test.rs` | CompleteComponent tests | ⬜ |
| `backend/src/application/commands/update_component_output_test.rs` | UpdateComponentOutput tests | ⬜ |
| `backend/src/application/commands/navigate_component_test.rs` | NavigateToComponent tests | ⬜ |

### Application Layer - Queries (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/queries/get_cycle.rs` | GetCycle handler | ⬜ |
| `backend/src/application/queries/get_cycle_tree.rs` | GetCycleTree handler | ⬜ |
| `backend/src/application/queries/get_component.rs` | GetComponent handler | ⬜ |

### Application Layer - Query Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/queries/get_cycle_test.rs` | GetCycle tests | ⬜ |
| `backend/src/application/queries/get_cycle_tree_test.rs` | GetCycleTree tests | ⬜ |
| `backend/src/application/queries/get_component_test.rs` | GetComponent tests | ⬜ |

### HTTP Adapter (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/http/cycle/mod.rs` | Module exports | ⬜ |
| `backend/src/adapters/http/cycle/handlers.rs` | HTTP handlers | ⬜ |
| `backend/src/adapters/http/cycle/dto.rs` | Request/Response DTOs | ⬜ |
| `backend/src/adapters/http/cycle/routes.rs` | Route definitions | ⬜ |

### HTTP Adapter Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/http/cycle/handlers_test.rs` | Handler tests | ⬜ |

### Postgres Adapter (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/postgres/cycle_repository.rs` | PostgresCycleRepository | ⬜ |
| `backend/src/adapters/postgres/cycle_reader.rs` | PostgresCycleReader | ⬜ |
| `backend/src/adapters/postgres/component_mapper.rs` | JSONB to Rust mapper | ⬜ |

### Postgres Adapter Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/postgres/cycle_repository_test.rs` | Repository tests | ⬜ |
| `backend/src/adapters/postgres/cycle_reader_test.rs` | Reader tests | ⬜ |

### Database Migrations

| File | Description | Status |
|------|-------------|--------|
| `backend/migrations/002_create_cycles.sql` | Cycles table | ⬜ |
| `backend/migrations/003_create_components.sql` | Components table | ⬜ |

### Frontend Domain (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/cycle/domain/cycle.ts` | Cycle types | ⬜ |
| `frontend/src/modules/cycle/domain/progress.ts` | Progress types | ⬜ |
| `frontend/src/modules/cycle/domain/cycle-tree.ts` | Tree types | ⬜ |

### Frontend Domain Tests (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/cycle/domain/cycle.test.ts` | Cycle tests | ⬜ |

### Frontend API (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/cycle/api/cycle-api.ts` | API client | ⬜ |
| `frontend/src/modules/cycle/api/use-cycle.ts` | Single cycle hook | ⬜ |
| `frontend/src/modules/cycle/api/use-cycle-tree.ts` | Tree hook | ⬜ |

### Frontend Components (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/cycle/components/CycleTree.tsx` | Tree component | ⬜ |
| `frontend/src/modules/cycle/components/CycleProgress.tsx` | Progress bar | ⬜ |
| `frontend/src/modules/cycle/components/ComponentNav.tsx` | Component navigation | ⬜ |
| `frontend/src/modules/cycle/components/BranchDialog.tsx` | Branch dialog | ⬜ |
| `frontend/src/modules/cycle/index.ts` | Module exports | ⬜ |

### Frontend Component Tests (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/cycle/components/CycleTree.test.tsx` | Tree tests | ⬜ |
| `frontend/src/modules/cycle/components/ComponentNav.test.tsx` | Nav tests | ⬜ |

---

## Test Inventory

### Cycle Aggregate Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_cycle_new_creates_all_nine_components` | All components initialized | ⬜ |
| `test_cycle_new_starts_at_issue_raising` | Current step is first | ⬜ |
| `test_cycle_new_has_active_status` | Status is active | ⬜ |
| `test_cycle_new_is_root` | No parent cycle | ⬜ |
| `test_cycle_new_emits_created_event` | Event is recorded | ⬜ |
| `test_cycle_reconstitute_preserves_all_fields` | Reconstitution works | ⬜ |
| `test_cycle_reconstitute_no_events` | No events on reconstitute | ⬜ |
| `test_cycle_branch_at_copies_components_up_to_point` | Branch inherits correctly | ⬜ |
| `test_cycle_branch_at_has_fresh_remaining_components` | Remaining are new | ⬜ |
| `test_cycle_branch_at_sets_parent_and_branch_point` | Parent reference set | ⬜ |
| `test_cycle_branch_at_emits_branched_event` | Event is recorded | ⬜ |
| `test_cycle_cannot_branch_at_not_started_component` | Validation error | ⬜ |
| `test_cycle_cannot_branch_when_archived` | Archived rejection | ⬜ |
| `test_cycle_get_component_returns_correct_type` | Accessor works | ⬜ |
| `test_cycle_current_component_matches_current_step` | Current accessor | ⬜ |

### Component Lifecycle Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_cycle_start_component_changes_status` | Status becomes in_progress | ⬜ |
| `test_cycle_start_component_sets_current_step` | Current step updated | ⬜ |
| `test_cycle_start_component_emits_event` | Event is recorded | ⬜ |
| `test_cycle_start_first_component_succeeds` | IssueRaising can start | ⬜ |
| `test_cycle_cannot_start_without_previous_started` | Order enforcement | ⬜ |
| `test_cycle_complete_component_changes_status` | Status becomes complete | ⬜ |
| `test_cycle_complete_component_auto_advances` | Current step advances | ⬜ |
| `test_cycle_complete_component_emits_event` | Event is recorded | ⬜ |
| `test_cycle_update_component_output_persists` | Output saved | ⬜ |
| `test_cycle_update_component_output_emits_event` | Event is recorded | ⬜ |
| `test_cycle_component_lifecycle_fails_when_archived` | Archived rejection | ⬜ |

### Navigation Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_cycle_navigate_to_started_component_succeeds` | Can return to started | ⬜ |
| `test_cycle_navigate_to_next_available_succeeds` | Can advance | ⬜ |
| `test_cycle_navigate_cannot_skip_unstarted` | Order enforcement | ⬜ |
| `test_cycle_navigate_updates_current_step` | Current step updated | ⬜ |

### CycleProgress Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_cycle_progress_percent_complete_zero_initially` | 0% at start | ⬜ |
| `test_cycle_progress_percent_complete_calculates_correctly` | Math is right | ⬜ |
| `test_cycle_progress_is_complete_when_all_done` | 100% detection | ⬜ |
| `test_cycle_progress_first_incomplete_finds_correct` | First incomplete finder | ⬜ |
| `test_cycle_progress_step_statuses_map_all_components` | All 9 present | ⬜ |

### Cycle Lifecycle Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_cycle_complete_changes_status_to_completed` | Status changes | ⬜ |
| `test_cycle_complete_emits_event` | Event is recorded | ⬜ |
| `test_cycle_cannot_complete_when_archived` | Archived rejection | ⬜ |
| `test_cycle_archive_changes_status_to_archived` | Status changes | ⬜ |
| `test_cycle_archive_emits_event` | Event is recorded | ⬜ |

### CycleEvent Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_cycle_event_cycle_id_returns_id` | ID accessor works | ⬜ |
| `test_cycle_event_serializes_to_json` | JSON serialization | ⬜ |
| `test_cycle_event_deserializes_from_json` | JSON deserialization | ⬜ |

### CreateCycle Command Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_create_cycle_handler_success` | Happy path | ⬜ |
| `test_create_cycle_handler_session_not_found` | 404 case | ⬜ |
| `test_create_cycle_handler_unauthorized` | 403 case | ⬜ |
| `test_create_cycle_handler_links_to_session` | Session updated | ⬜ |
| `test_create_cycle_handler_publishes_events` | Events published | ⬜ |

### BranchCycle Command Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_branch_cycle_handler_success` | Happy path | ⬜ |
| `test_branch_cycle_handler_cycle_not_found` | 404 case | ⬜ |
| `test_branch_cycle_handler_unauthorized` | 403 case | ⬜ |
| `test_branch_cycle_handler_invalid_branch_point` | Validation error | ⬜ |
| `test_branch_cycle_handler_links_to_session` | Session updated | ⬜ |

### Component Command Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_start_component_handler_success` | Happy path | ⬜ |
| `test_start_component_handler_order_validation` | Order enforced | ⬜ |
| `test_complete_component_handler_success` | Happy path | ⬜ |
| `test_update_component_output_handler_success` | Happy path | ⬜ |
| `test_navigate_component_handler_success` | Happy path | ⬜ |

### Query Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_get_cycle_handler_success` | Happy path | ⬜ |
| `test_get_cycle_handler_not_found` | 404 case | ⬜ |
| `test_get_cycle_tree_handler_success` | Returns tree | ⬜ |
| `test_get_cycle_tree_handler_builds_hierarchy` | Parent-child correct | ⬜ |
| `test_get_component_handler_success` | Happy path | ⬜ |

### HTTP Handler Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_post_cycles_creates_cycle` | POST creates | ⬜ |
| `test_post_cycles_returns_201` | Status code correct | ⬜ |
| `test_get_cycle_returns_detail` | GET single works | ⬜ |
| `test_get_cycle_tree_returns_hierarchy` | GET tree works | ⬜ |
| `test_post_branch_creates_branch` | Branch endpoint works | ⬜ |
| `test_post_component_start_works` | Start endpoint works | ⬜ |
| `test_post_component_complete_works` | Complete endpoint works | ⬜ |
| `test_put_component_output_works` | Update endpoint works | ⬜ |

### Postgres Repository Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_postgres_repo_save_persists_cycle_with_components` | Save works | ⬜ |
| `test_postgres_repo_save_persists_component_jsonb` | JSONB saved | ⬜ |
| `test_postgres_repo_update_modifies_cycle` | Update works | ⬜ |
| `test_postgres_repo_update_modifies_components` | Component update | ⬜ |
| `test_postgres_repo_find_by_id_returns_cycle` | Find works | ⬜ |
| `test_postgres_repo_find_by_id_loads_all_components` | Components loaded | ⬜ |
| `test_postgres_repo_find_by_session_returns_all` | Session filter | ⬜ |

### Postgres Reader Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_postgres_reader_get_by_id_returns_view` | Get works | ⬜ |
| `test_postgres_reader_get_cycle_tree_builds_hierarchy` | Tree building | ⬜ |
| `test_postgres_reader_get_component_view_returns_output` | Component view | ⬜ |

---

## Error Codes

| Error Code | HTTP Status | Condition |
|------------|-------------|-----------|
| `CYCLE_NOT_FOUND` | 404 | Cycle does not exist |
| `COMPONENT_NOT_FOUND` | 404 | Component type not found |
| `CYCLE_ARCHIVED` | 400 | Cannot modify archived cycle |
| `INVALID_STATE_TRANSITION` | 400 | Invalid component order |
| `FORBIDDEN` | 403 | User is not session owner |
| `DATABASE_ERROR` | 500 | Database operation failed |
| `SERIALIZATION_ERROR` | 500 | JSON serialization failed |

---

## Business Rules

| Rule | Implementation | Test | Status |
|------|----------------|------|--------|
| Cycle belongs to session | Constructor requires session_id | `test_create_cycle_handler_links_to_session` | ⬜ |
| All 9 components exist | Created in constructor | `test_cycle_new_creates_all_nine_components` | ⬜ |
| Components follow order | validate_can_start() check | `test_cycle_cannot_start_without_previous_started` | ⬜ |
| Branch point must be started | can_branch_at() check | `test_cycle_cannot_branch_at_not_started_component` | ⬜ |
| Branch inherits state | branch_at() copies components | `test_cycle_branch_at_copies_components_up_to_point` | ⬜ |
| Completed/archived immutable | ensure_mutable() check | `test_cycle_component_lifecycle_fails_when_archived` | ⬜ |

---

## Verification Commands

```bash
# Run all cycle tests
cargo test --package cycle -- --nocapture

# Domain layer tests
cargo test --package cycle domain:: -- --nocapture

# Application layer tests
cargo test --package cycle application:: -- --nocapture

# Adapter tests (requires database)
cargo test --package cycle adapters:: -- --ignored

# HTTP handler tests
cargo test --package cycle adapters::http:: -- --nocapture

# Coverage check (target: 85%+)
cargo tarpaulin --package cycle --out Html

# Full verification
cargo test --package cycle -- --nocapture && cargo clippy --package cycle

# Frontend tests
cd frontend && npm test -- --testPathPattern="modules/cycle"
```

---

## Exit Criteria

### Module is COMPLETE when:

- [ ] All 58 files in File Inventory exist
- [ ] All 82 tests in Test Inventory pass
- [ ] Domain layer coverage >= 90%
- [ ] Application layer coverage >= 85%
- [ ] Adapter layer coverage >= 80%
- [ ] Database migrations run successfully
- [ ] Branching creates correct parent-child relationships
- [ ] Component JSONB serialization roundtrips correctly
- [ ] Cycle tree query builds correct hierarchy
- [ ] No clippy warnings
- [ ] Frontend components render correctly
- [ ] No TypeScript lint errors

### Current Status

```
RUST BACKEND IN PROGRESS: cycle
Files: 3/58 (5%)
Tests: 38/82 passing (46%)
Frontend: Not started
```

### Exit Signal

```
MODULE COMPLETE: cycle
Files: 58/58
Tests: 82/82 passing
Coverage: Domain 91%, Application 86%, Adapters 81%
```

---

## Implementation Phases

### Phase 1: Domain Layer (In Progress)
- [x] Cycle aggregate implementation (aggregate.rs - 38 tests)
- [ ] CycleProgress value object
- [x] CycleEvent enum
- [ ] Component lifecycle management
- [ ] Branching logic
- [ ] Domain layer tests (partial - aggregate.rs)

### Phase 2: Ports
- [ ] CycleRepository trait
- [ ] CycleReader trait
- [ ] View DTOs (CycleView, CycleTree, ComponentView)

### Phase 3: Commands
- [ ] CreateCycleCommand + Handler
- [ ] BranchCycleCommand + Handler
- [ ] StartComponentCommand + Handler
- [ ] CompleteComponentCommand + Handler
- [ ] UpdateComponentOutputCommand + Handler
- [ ] NavigateToComponentCommand + Handler
- [ ] Command tests with mock repos

### Phase 4: Queries
- [ ] GetCycleQuery + Handler
- [ ] GetCycleTreeQuery + Handler
- [ ] GetComponentQuery + Handler
- [ ] Query tests with mock readers

### Phase 5: HTTP Adapter
- [ ] Request/Response DTOs
- [ ] HTTP handlers
- [ ] Route definitions
- [ ] Handler tests

### Phase 6: Postgres Adapter
- [ ] Database migrations (cycles, components tables)
- [ ] PostgresCycleRepository with JSONB mapping
- [ ] PostgresCycleReader with tree building
- [ ] ComponentMapper utility
- [ ] Integration tests

### Phase 7: Frontend
- [ ] TypeScript types
- [ ] API client
- [ ] React hooks
- [ ] Components (CycleTree, ComponentNav, BranchDialog)
- [ ] Component tests

---

## Notes

- Cycle is the aggregate root that owns all components
- Components stored as JSONB in separate table for efficient queries
- ComponentMapper handles JSONB <-> Rust type conversion
- Branching creates new cycle with copied components up to branch point
- CycleTree query requires recursive CTE or application-level building
- Progress calculation counts completed components / 9

---

*Generated: 2026-01-07*
*Specification: docs/modules/cycle.md*
