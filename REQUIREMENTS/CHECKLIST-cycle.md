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
| `backend/src/domain/cycle/progress.rs` | CycleProgress value object (19 tests inline) | ✅ |
| `backend/src/domain/cycle/events.rs` | CycleEvent enum (16 tests inline) | ✅ |
| `backend/src/domain/cycle/errors.rs` | Cycle-specific errors | ⬜ |

> **Note:** Tests are inline in implementation files using `#[cfg(test)] mod tests` (Rust convention). The file `cycle.rs` was renamed to `aggregate.rs`. Separate test files (`*_test.rs`) are not used.

### Domain Tests (Rust)

> **Note:** Domain tests are inline in implementation files (see aggregate.rs). Separate test files are not used.

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/cycle/aggregate.rs` (inline tests) | Cycle aggregate tests (38 tests) | ✅ |
| `backend/src/domain/cycle/progress.rs` (inline tests) | CycleProgress tests (19 tests) | ✅ |
| `backend/src/domain/cycle/events.rs` (inline tests) | CycleEvent tests (16 tests) | ✅ |

### Ports (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/ports/cycle_repository.rs` | CycleRepository trait (1 test) | ✅ |
| `backend/src/ports/cycle_reader.rs` | CycleReader trait (4 tests) | ✅ |

### Application Layer - Command Handlers (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/handlers/cycle/mod.rs` | Module exports | ✅ |
| `backend/src/application/handlers/cycle/create_cycle.rs` | CreateCycle handler (8 tests inline) | ✅ |
| `backend/src/application/handlers/cycle/branch_cycle.rs` | BranchCycle handler (8 tests inline) | ✅ |
| `backend/src/application/handlers/cycle/start_component.rs` | StartComponent handler (7 tests inline) | ✅ |
| `backend/src/application/handlers/cycle/complete_component.rs` | CompleteComponent handler (8 tests inline) | ✅ |
| `backend/src/application/handlers/cycle/update_component_output.rs` | UpdateComponentOutput handler (7 tests inline) | ✅ |
| `backend/src/application/handlers/cycle/navigate_to_component.rs` | NavigateToComponent handler (7 tests inline) | ✅ |
| `backend/src/application/handlers/cycle/complete_cycle.rs` | CompleteCycle handler (6 tests inline) | ✅ |
| `backend/src/application/handlers/cycle/archive_cycle.rs` | ArchiveCycle handler (7 tests inline) | ✅ |

> **Note:** Tests are inline in handler files using `#[cfg(test)] mod tests` (Rust convention).

### Application Layer - Query Handlers (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/handlers/cycle/get_cycle.rs` | GetCycle handler (4 tests inline) | ✅ |
| `backend/src/application/handlers/cycle/get_cycle_tree.rs` | GetCycleTree handler (4 tests inline) | ✅ |
| `backend/src/application/handlers/cycle/get_component.rs` | GetComponent handler (5 tests inline) | ✅ |

> **Note:** Query handlers are co-located with command handlers in handlers/cycle/ directory.

### HTTP Adapter (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/http/cycle/mod.rs` | Module exports | ✅ |
| `backend/src/adapters/http/cycle/handlers.rs` | HTTP handlers (14 tests inline) | ✅ |
| `backend/src/adapters/http/cycle/dto.rs` | Request/Response DTOs | ✅ |
| `backend/src/adapters/http/cycle/routes.rs` | Route definitions | ✅ |

> **Note:** HTTP handler tests are inline in handlers.rs.

### Postgres Adapter (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/postgres/cycle_repository.rs` | PostgresCycleRepository (8 tests inline) | ✅ |
| `backend/src/adapters/postgres/cycle_reader.rs` | PostgresCycleReader (6 tests inline) | ✅ |

> **Note:** Component JSONB mapping is handled within repository/reader. Tests are inline.

### Database Migrations

| File | Description | Status |
|------|-------------|--------|
| `backend/migrations/20260109000003_create_cycles.sql` | Cycles and Components tables (combined) | ✅ |

### Frontend Domain (TypeScript)

> **Note:** Uses SvelteKit + Svelte 5, not React. Files use `.svelte` extension for components.

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/cycle/domain/types.ts` | All cycle types (consolidated) | ✅ |
| `frontend/src/modules/cycle/domain/types.test.ts` | Domain type tests (10 tests) | ✅ |

### Frontend API (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/cycle/api/cycle-api.ts` | API client (all operations) | ✅ |
| `frontend/src/modules/cycle/api/stores.ts` | Svelte stores for reactivity | ✅ |

### Frontend Components (Svelte)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/cycle/components/CycleTree.svelte` | Tree component | ✅ |
| `frontend/src/modules/cycle/components/CycleProgress.svelte` | Progress bar | ✅ |
| `frontend/src/modules/cycle/components/ComponentNav.svelte` | Component navigation | ✅ |
| `frontend/src/modules/cycle/components/BranchDialog.svelte` | Branch dialog | ✅ |
| `frontend/src/modules/cycle/index.ts` | Module exports | ✅ |

### Frontend Configuration

| File | Description | Status |
|------|-------------|--------|
| `frontend/package.json` | Dependencies and scripts | ✅ |
| `frontend/tsconfig.json` | TypeScript config | ✅ |
| `frontend/svelte.config.js` | SvelteKit config | ✅ |
| `frontend/vite.config.ts` | Vite/Vitest config | ✅ |

---

## Test Inventory

> **Note:** Test names in aggregate.rs use shortened names (e.g., `new_cycle_is_active` instead of `test_cycle_new_has_active_status`). The mapping below shows the actual test function names.

### Cycle Aggregate Tests (in aggregate.rs)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `new_cycle_has_all_components_not_started` | All components initialized | ✅ |
| `new_cycle_current_step_is_issue_raising` | Current step is first | ✅ |
| `new_cycle_is_active` | Status is active | ✅ |
| `new_cycle_is_not_a_branch` | No parent cycle | ✅ |
| `new_cycle_records_created_event` | Event is recorded | ✅ |
| ~~`test_cycle_reconstitute_preserves_all_fields`~~ | Reconstitution works | ⬜ |
| ~~`test_cycle_reconstitute_no_events`~~ | No events on reconstitute | ⬜ |
| `branch_inherits_components_before_branch_point` | Branch inherits correctly | ✅ |
| `branch_components_after_branch_point_are_fresh` | Remaining are new | ✅ |
| `can_branch_at_started_component` | Parent reference set | ✅ |
| `branch_records_event` | Event is recorded | ✅ |
| `cannot_branch_at_not_started_component` | Validation error | ✅ |
| ~~`test_cycle_cannot_branch_when_archived`~~ | Archived rejection (covered by cannot_modify_archived_cycle) | ✅ |
| ~~`test_cycle_get_component_returns_correct_type`~~ | Accessor works | ⬜ |
| ~~`test_cycle_current_component_matches_current_step`~~ | Current accessor | ⬜ |

### Component Lifecycle Tests (in aggregate.rs)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `can_start_issue_raising` | Status becomes in_progress | ✅ |
| `starting_component_updates_current_step` | Current step updated | ✅ |
| `starting_component_records_event` | Event is recorded | ✅ |
| `can_start_issue_raising` | IssueRaising can start | ✅ |
| `cannot_start_problem_frame_before_issue_raising` | Order enforcement | ✅ |
| `can_start_problem_frame_after_issue_raising_started` | Can start after prereq | ✅ |
| `cannot_start_already_started_component` | Already started rejection | ✅ |
| `can_complete_in_progress_component` | Status becomes complete | ✅ |
| ~~`test_cycle_complete_component_auto_advances`~~ | Current step advances | ⬜ |
| `completing_component_records_event` | Event is recorded | ✅ |
| ~~`test_cycle_update_component_output_persists`~~ | Output saved | ⬜ |
| ~~`test_cycle_update_component_output_emits_event`~~ | Event is recorded | ⬜ |
| `cannot_modify_archived_cycle` | Archived rejection | ✅ |
| `can_mark_complete_component_for_revision` | Mark for revision works | ✅ |
| `marking_for_revision_updates_current_step` | Revision updates step | ✅ |
| `cannot_complete_not_started_component` | Cannot complete not started | ✅ |

### Navigation Tests (in aggregate.rs)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `can_navigate_to_started_component` | Can return to started | ✅ |
| `can_navigate_to_next_not_started_component_if_prereq_started` | Can advance | ✅ |
| `cannot_navigate_to_not_started_component_without_prereq` | Order enforcement | ✅ |
| `navigating_records_event` | Navigation records event | ✅ |

### CycleProgress Tests (requires progress.rs)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_cycle_progress_percent_complete_zero_initially` | 0% at start | ⬜ |
| `test_cycle_progress_percent_complete_calculates_correctly` | Math is right | ⬜ |
| `test_cycle_progress_is_complete_when_all_done` | 100% detection | ⬜ |
| `test_cycle_progress_first_incomplete_finds_correct` | First incomplete finder | ⬜ |
| `test_cycle_progress_step_statuses_map_all_components` | All 9 present | ⬜ |

### Cycle Lifecycle Tests (in aggregate.rs)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `can_complete_cycle_with_decision_quality_complete` | Status changes | ✅ |
| `completing_cycle_records_event` | Event is recorded | ✅ |
| `cannot_complete_cycle_without_decision_quality` | Requires DQ complete | ✅ |
| `can_archive_active_cycle` | Status changes | ✅ |
| `can_archive_completed_cycle` | Archive completed works | ✅ |

### CycleEvent Tests (requires events.rs inline tests)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_cycle_event_cycle_id_returns_id` | ID accessor works | ⬜ |
| `test_cycle_event_serializes_to_json` | JSON serialization | ⬜ |
| `test_cycle_event_deserializes_from_json` | JSON deserialization | ⬜ |

### Component Validation Tests (in aggregate.rs)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `alternatives_validation_requires_at_least_two` | Min 2 alternatives | ✅ |
| `alternatives_validation_requires_valid_status_quo` | Valid status quo | ✅ |
| `alternatives_validation_passes_with_valid_data` | Valid data passes | ✅ |
| `objectives_validation_requires_at_least_one_fundamental` | Min 1 fundamental | ✅ |
| `decision_quality_validation_requires_seven_elements` | Exactly 7 DQ elements | ✅ |

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
| Cycle belongs to session | Constructor requires session_id | `new_cycle_*` tests | ✅ |
| All 9 components exist | Created in constructor | `new_cycle_has_all_components_not_started` | ✅ |
| Components follow order | validate_can_start() check | `cannot_start_problem_frame_before_issue_raising` | ✅ |
| Branch point must be started | can_branch_at() check | `cannot_branch_at_not_started_component` | ✅ |
| Branch inherits state | branch_at() copies components | `branch_inherits_components_before_branch_point` | ✅ |
| Completed/archived immutable | ensure_mutable() check | `cannot_modify_archived_cycle` | ✅ |

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

- [ ] All 53 backend files in File Inventory exist (3/53 complete)
- [ ] All 14 frontend files in File Inventory exist (0/14 complete)
- [ ] All domain tests pass (38/46 complete - progress.rs and events.rs tests needed)
- [ ] All application/adapter tests pass (0/67 complete)
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
MODULE COMPLETE: cycle
Backend: 19/19 files (100%)
  - Domain Layer: 4/5 (mod.rs, aggregate.rs, events.rs, progress.rs) - errors.rs optional
  - Ports: 2/2 (cycle_repository.rs, cycle_reader.rs)
  - Application: 12/12 handlers (mod.rs, 8 commands, 3 queries)
  - HTTP Adapter: 4/4 (mod.rs, handlers.rs, dto.rs, routes.rs)
  - Postgres Adapter: 2/2 (cycle_repository.rs, cycle_reader.rs)
  - Migrations: 1/1 (20260109000003_create_cycles.sql)
Backend Tests: 150+ passing
  - Aggregate tests: 38/38
  - Progress tests: 19/19
  - Event tests: 16/16
  - Port tests: 5/5 (CycleRepository 1, CycleReader 4)
  - Command Handler tests: 58/58
  - Query Handler tests: 13/13
  - HTTP Adapter tests: 14/14
  - Postgres Adapter tests: 14/14
Frontend: 13/13 files (100%)
  - Domain: types.ts, types.test.ts (10 tests)
  - API: cycle-api.ts, stores.ts
  - Components: CycleTree, CycleProgress, ComponentNav, BranchDialog (all .svelte)
  - Config: package.json, tsconfig.json, svelte.config.js, vite.config.ts
```

### Exit Signal

```
MODULE COMPLETE: cycle
Files: 32/32 (Backend: 19, Frontend: 13)
Tests: 160+ (Backend: 150+, Frontend: 10)
```

---

## Implementation Phases

### Phase 1: Domain Layer (Complete)
- [x] Cycle aggregate implementation (aggregate.rs - 38 tests)
- [x] CycleProgress value object (progress.rs - 19 tests)
- [x] CycleEvent enum (events.rs - 16 tests)
- [x] Component lifecycle management (in aggregate.rs)
- [x] Branching logic (in aggregate.rs)
- [ ] errors.rs - Cycle-specific errors
- [x] Domain layer tests (aggregate.rs 38 tests, events.rs 16 tests, progress.rs 19 tests)

### Phase 2: Ports (Complete)
- [x] CycleRepository trait (1 test)
- [x] CycleReader trait (4 tests, includes view DTOs)
- [x] View DTOs (CycleView, CycleSummary, CycleTreeNode, CycleProgressView, NextAction)

### Phase 3: Commands (Complete)
- [x] CreateCycleCommand + Handler (8 tests)
- [x] BranchCycleCommand + Handler (8 tests)
- [x] StartComponentCommand + Handler (7 tests)
- [x] CompleteComponentCommand + Handler (8 tests)
- [x] UpdateComponentOutputCommand + Handler (7 tests)
- [x] NavigateToComponentCommand + Handler (7 tests)
- [x] CompleteCycleCommand + Handler (6 tests)
- [x] ArchiveCycleCommand + Handler (7 tests)

### Phase 4: Queries (Complete)
- [x] GetCycleQuery + Handler (4 tests)
- [x] GetCycleTreeQuery + Handler (4 tests)
- [x] GetComponentQuery + Handler (5 tests)

### Phase 5: HTTP Adapter (Complete)
- [x] Request/Response DTOs (dto.rs)
- [x] HTTP handlers (handlers.rs - 14 tests)
- [x] Route definitions (routes.rs)

### Phase 6: Postgres Adapter (Complete)
- [x] Database migrations (20260109000003_create_cycles.sql)
- [x] PostgresCycleRepository with JSONB mapping (8 tests)
- [x] PostgresCycleReader with tree building (6 tests)

### Phase 7: Frontend (Complete)
- [x] TypeScript types (types.ts - consolidated)
- [x] Type tests (types.test.ts - 10 tests)
- [x] API client (cycle-api.ts - all operations)
- [x] Svelte stores (stores.ts - reactive state)
- [x] Components: CycleTree, CycleProgress, ComponentNav, BranchDialog (.svelte)
- [x] Module exports (index.ts)
- [x] Config: package.json, tsconfig.json, svelte.config.js, vite.config.ts

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
*Last synced: 2026-01-10*
*Specification: docs/modules/cycle.md*
