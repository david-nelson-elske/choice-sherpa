# Session Module Checklist

**Module:** Session
**Language:** Rust
**Dependencies:** foundation
**Phase:** 2 (parallel with proact-types)

---

## Overview

The Session module manages the top-level Decision Session - the container for all cycles exploring a single decision context. Each session belongs to a user and can contain multiple cycles (including branches). This is a full hexagonal module with ports and adapters.

---

## File Inventory

### Domain Layer (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/session/mod.rs` | Module exports | ✅ |
| `backend/src/domain/session/aggregate.rs` | Session aggregate | ✅ |
| `backend/src/domain/session/events.rs` | SessionEvent enum (13 tests inline) | ✅ |
| `backend/src/domain/session/errors.rs` | Session-specific errors | ✅ |

> **Note:** Tests are inline in implementation files using `#[cfg(test)] mod tests` (Rust convention).

### Domain Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/session/aggregate.rs` | Session aggregate tests (inline) | ✅ |
| `backend/src/domain/session/events.rs` | Event tests (inline) | ✅ |

### Ports (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/ports/session_repository.rs` | SessionRepository trait | ✅ |
| `backend/src/ports/session_reader.rs` | SessionReader trait (CQRS) | ✅ |
| `backend/src/ports/event_publisher.rs` | EventPublisher trait | ✅ |

### Application Layer - Handlers (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/handlers/session/mod.rs` | Session handlers module | ✅ |
| `backend/src/application/handlers/session/create_session.rs` | CreateSession handler (7 tests inline) | ✅ |
| `backend/src/application/handlers/session/rename_session.rs` | RenameSession handler (7 tests inline) | ✅ |
| `backend/src/application/handlers/session/archive_session.rs` | ArchiveSession handler (6 tests inline) | ✅ |
| `backend/src/application/handlers/session/session_cycle_tracker.rs` | SessionCycleTracker event handler (8 tests inline) | ✅ |

### Application Layer - Queries (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/queries/get_session.rs` | GetSession handler | ⬜ |
| `backend/src/application/queries/list_sessions.rs` | ListUserSessions handler | ⬜ |

### Application Layer - Query Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/queries/get_session_test.rs` | GetSession tests | ⬜ |
| `backend/src/application/queries/list_sessions_test.rs` | ListUserSessions tests | ⬜ |

### HTTP Adapter (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/http/session/mod.rs` | Module exports | ⬜ |
| `backend/src/adapters/http/session/handlers.rs` | HTTP handlers | ⬜ |
| `backend/src/adapters/http/session/dto.rs` | Request/Response DTOs | ⬜ |
| `backend/src/adapters/http/session/routes.rs` | Route definitions | ⬜ |

### HTTP Adapter Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/http/session/handlers_test.rs` | Handler tests | ⬜ |

### Postgres Adapter (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/postgres/session_repository.rs` | PostgresSessionRepository | ⬜ |
| `backend/src/adapters/postgres/session_reader.rs` | PostgresSessionReader | ⬜ |

### Postgres Adapter Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/postgres/session_repository_test.rs` | Repository tests | ⬜ |
| `backend/src/adapters/postgres/session_reader_test.rs` | Reader tests | ⬜ |

### Database Migrations

| File | Description | Status |
|------|-------------|--------|
| `backend/migrations/20260109000003_create_sessions.sql` | Sessions table | ⬜ |

### Frontend Domain (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/session/domain/session.ts` | Session types | ⬜ |

### Frontend Domain Tests (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/session/domain/session.test.ts` | Session tests | ⬜ |

### Frontend API (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/session/api/session-api.ts` | API client | ⬜ |
| `frontend/src/modules/session/api/use-sessions.ts` | List hook | ⬜ |
| `frontend/src/modules/session/api/use-session.ts` | Single session hook | ⬜ |

### Frontend Components (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/session/components/SessionList.tsx` | List component | ⬜ |
| `frontend/src/modules/session/components/SessionCard.tsx` | Card component | ⬜ |
| `frontend/src/modules/session/components/CreateSessionDialog.tsx` | Create dialog | ⬜ |
| `frontend/src/modules/session/index.ts` | Module exports | ⬜ |

### Frontend Component Tests (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/session/components/SessionList.test.tsx` | List tests | ⬜ |
| `frontend/src/modules/session/components/SessionCard.test.tsx` | Card tests | ⬜ |

---

## Test Inventory

### Session Aggregate Tests (15 tests in aggregate.rs)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `new_session_is_active` | New session is active | ✅ |
| `new_session_has_no_cycles` | Session starts with no cycles | ✅ |
| `new_session_rejects_empty_title` | Empty title rejected | ✅ |
| `new_session_rejects_whitespace_title` | Whitespace title rejected | ✅ |
| `new_session_rejects_too_long_title` | Long title rejected | ✅ |
| `rename_returns_old_title` | Rename returns previous title | ✅ |
| `rename_fails_when_archived` | Archived rejection | ✅ |
| `update_description_returns_old` | Description updates | ✅ |
| `add_cycle_first_is_root` | First cycle is root | ✅ |
| `add_cycle_second_is_not_root` | Subsequent cycles are not root | ✅ |
| `add_cycle_duplicate_returns_false` | No duplicate cycles | ✅ |
| `archive_changes_status` | Status changes | ✅ |
| `archive_twice_fails` | Double archive rejected | ✅ |
| `owner_is_authorized` | Authorization works | ✅ |
| `non_owner_is_forbidden` | Authorization rejects | ✅ |

### SessionEvent Tests (13 tests in events.rs)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `session_created_implements_domain_event` | SessionCreated event works | ✅ |
| `session_created_serializes_to_json` | JSON serialization | ✅ |
| `session_created_to_envelope_works` | Envelope creation works | ✅ |
| `session_archived_implements_domain_event` | SessionArchived event works | ✅ |
| `session_archived_serializes_correctly` | JSON serialization | ✅ |
| `session_renamed_serializes_correctly` | JSON serialization | ✅ |
| `session_renamed_captures_both_titles` | Old/new title captured | ✅ |
| `session_description_updated_captures_both_descriptions` | Old/new description captured | ✅ |
| `session_description_updated_handles_none_values` | None description handled | ✅ |
| `cycle_added_to_session_implements_domain_event` | CycleAddedToSession event works | ✅ |
| `cycle_added_to_session_tracks_root_status` | Root status tracked | ✅ |
| `cycle_added_serialization_round_trip` | Serialization round trip | ✅ |
| `all_events_produce_valid_envelopes` | All events create valid envelopes | ✅ |

### CreateSession Handler Tests (7 tests)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `creates_session_with_valid_input` | Happy path | ✅ |
| `includes_description_when_provided` | Description included | ✅ |
| `publishes_session_created_event` | Events published | ✅ |
| `includes_correlation_id_in_event` | Correlation ID tracked | ✅ |
| `fails_when_access_denied` | Access check rejection | ✅ |
| `fails_with_empty_title` | Validation error | ✅ |
| `does_not_publish_event_on_save_failure` | No event on failure | ✅ |

### RenameSession Handler Tests (7 tests)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `renames_session_successfully` | Happy path | ✅ |
| `publishes_session_renamed_event` | Events published | ✅ |
| `includes_correlation_id_in_event` | Correlation ID tracked | ✅ |
| `fails_when_session_not_found` | 404 case | ✅ |
| `fails_when_not_owner` | 403 case | ✅ |
| `fails_with_empty_title` | Validation error | ✅ |
| `fails_when_session_archived` | Archived rejection | ✅ |

### ArchiveSession Handler Tests (6 tests)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `archives_session_successfully` | Happy path | ✅ |
| `publishes_session_archived_event` | Events published | ✅ |
| `includes_correlation_id_in_event` | Correlation ID tracked | ✅ |
| `fails_when_session_not_found` | 404 case | ✅ |
| `fails_when_not_owner` | 403 case | ✅ |
| `fails_when_already_archived` | Double archive rejection | ✅ |

### SessionCycleTracker Handler Tests (8 tests)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `adds_cycle_to_session` | Cycle added to session | ✅ |
| `publishes_cycle_added_to_session_event` | Event published | ✅ |
| `first_cycle_is_marked_as_root` | Root cycle identified | ✅ |
| `second_cycle_is_not_root` | Non-root cycle identified | ✅ |
| `fails_when_session_not_found` | 404 case | ✅ |
| `includes_causation_id` | Event causation tracked | ✅ |
| `handler_name_is_correct` | Handler name correct | ✅ |
| `duplicate_cycle_is_handled_idempotently` | Idempotent handling | ✅ |

### Ports Tests (5 tests)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `session_repository_is_object_safe` | Repository trait object safe | ✅ |
| `session_reader_is_object_safe` | Reader trait object safe | ✅ |
| `list_options_default_excludes_archived` | Default filter excludes archived | ✅ |
| `list_options_can_include_archived` | Filter can include archived | ✅ |
| `list_options_pagination_calculates_offset` | Pagination offset works | ✅ |

### GetSession Query Tests (Not yet implemented)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_get_session_handler_success` | Happy path | ⬜ |
| `test_get_session_handler_not_found` | 404 case | ⬜ |
| `test_get_session_handler_returns_view` | View returned | ⬜ |

### ListUserSessions Query Tests (Not yet implemented)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_list_sessions_handler_success` | Happy path | ⬜ |
| `test_list_sessions_handler_empty_list` | No sessions | ⬜ |
| `test_list_sessions_handler_filters_by_status` | Status filter | ⬜ |
| `test_list_sessions_handler_paginates` | Pagination works | ⬜ |
| `test_list_sessions_handler_returns_total` | Total count included | ⬜ |

### HTTP Handler Tests (Not yet implemented)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_post_sessions_creates_session` | POST creates | ⬜ |
| `test_post_sessions_returns_201` | Status code correct | ⬜ |
| `test_post_sessions_returns_400_for_empty_title` | Validation error | ⬜ |
| `test_get_sessions_returns_list` | GET list works | ⬜ |
| `test_get_sessions_supports_filters` | Query params work | ⬜ |
| `test_get_session_returns_detail` | GET single works | ⬜ |
| `test_get_session_returns_404_for_missing` | 404 case | ⬜ |
| `test_patch_session_updates_title` | PATCH works | ⬜ |
| `test_delete_session_archives` | DELETE archives | ⬜ |
| `test_endpoints_require_authentication` | Auth required | ⬜ |

### Postgres Repository Tests (Not yet implemented)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_postgres_repo_save_persists_session` | Save works | ⬜ |
| `test_postgres_repo_save_handles_duplicate` | Duplicate error | ⬜ |
| `test_postgres_repo_update_modifies_session` | Update works | ⬜ |
| `test_postgres_repo_update_returns_not_found` | Missing error | ⬜ |
| `test_postgres_repo_find_by_id_returns_session` | Find works | ⬜ |
| `test_postgres_repo_find_by_id_includes_cycles` | Cycles loaded | ⬜ |
| `test_postgres_repo_find_by_id_returns_none` | Missing returns None | ⬜ |
| `test_postgres_repo_exists_returns_true` | Exists works | ⬜ |
| `test_postgres_repo_exists_returns_false` | Not exists works | ⬜ |

### Postgres Reader Tests (Not yet implemented)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_postgres_reader_get_by_id_returns_view` | Get works | ⬜ |
| `test_postgres_reader_get_by_id_returns_none` | Missing returns None | ⬜ |
| `test_postgres_reader_list_by_user_returns_all` | List all works | ⬜ |
| `test_postgres_reader_list_by_user_filters_status` | Status filter works | ⬜ |
| `test_postgres_reader_list_by_user_paginates` | Pagination works | ⬜ |
| `test_postgres_reader_list_by_user_orders` | Ordering works | ⬜ |
| `test_postgres_reader_search_finds_by_title` | Title search works | ⬜ |
| `test_postgres_reader_search_finds_by_description` | Description search works | ⬜ |
| `test_postgres_reader_count_by_user_returns_total` | Count works | ⬜ |

---

## Error Codes

| Error Code | HTTP Status | Condition |
|------------|-------------|-----------|
| `VALIDATION_FAILED` | 400 | Title empty or too long |
| `SESSION_NOT_FOUND` | 404 | Session does not exist |
| `SESSION_ARCHIVED` | 400 | Cannot modify archived session |
| `FORBIDDEN` | 403 | User is not session owner |
| `DUPLICATE_ID` | 409 | Session ID already exists |
| `DATABASE_ERROR` | 500 | Database operation failed |

---

## Business Rules

| Rule | Implementation | Test | Status |
|------|----------------|------|--------|
| Title is non-empty | Constructor validation | `new_session_rejects_empty_title` | ✅ |
| Title max 500 chars | Constructor validation | `new_session_rejects_too_long_title` | ✅ |
| Only owner can modify | authorize() check | `non_owner_is_forbidden` | ✅ |
| Archived sessions immutable | ensure_mutable() check | `rename_fails_when_archived` | ✅ |
| Cycle IDs are unique | Duplicate check | `add_cycle_duplicate_returns_false` | ✅ |
| Status transitions valid | can_transition_to() | `archive_twice_fails` | ✅ |

---

## Verification Commands

```bash
# Run all session tests
cargo test --package session -- --nocapture

# Domain layer tests
cargo test --package session domain:: -- --nocapture

# Application layer tests
cargo test --package session application:: -- --nocapture

# Adapter tests (requires database)
cargo test --package session adapters:: -- --ignored

# HTTP handler tests
cargo test --package session adapters::http:: -- --nocapture

# Coverage check (target: 85%+)
cargo tarpaulin --package session --out Html

# Full verification
cargo test --package session -- --nocapture && cargo clippy --package session

# Frontend tests
cd frontend && npm test -- --testPathPattern="modules/session"
```

---

## Exit Criteria

### Module is COMPLETE when:

- [ ] All files in File Inventory exist (12/37 complete - 32%)
- [x] Domain layer tests pass (61/61 tests passing - 100%)
- [ ] Query handlers implemented
- [ ] Database migrations run successfully
- [ ] HTTP endpoints return correct status codes
- [x] CQRS pattern implemented (Repository + Reader ports defined)
- [x] Domain events published correctly (EventPublisher + handlers)
- [ ] Postgres adapters implemented
- [ ] No clippy warnings
- [ ] Frontend components render correctly
- [ ] No TypeScript lint errors

### Current Status

```
RUST BACKEND IN PROGRESS: session
Files: 12/26 backend files (46%)
Tests: 61/93 passing (66%)
Frontend: 0/11 files (Not started)
```

**Completed:**
- Domain layer (aggregate, events, errors) - 4 files
- Ports (SessionRepository, SessionReader, EventPublisher) - 3 files
- Command handlers (Create, Rename, Archive, SessionCycleTracker) - 5 files

**Remaining:**
- Query handlers (GetSession, ListUserSessions) - 4 files
- HTTP adapter (handlers, routes, DTOs) - 5 files
- Postgres adapter (repository, reader implementations) - 4 files
- Database migrations - 1 file
- Frontend - 11 files

### Exit Signal

```
MODULE COMPLETE: session
Files: 37/37 (26 backend + 11 frontend)
Tests: 93/93 passing
Coverage: Domain 92%, Application 87%, Adapters 82%
```

---

## Implementation Phases

### Phase 1: Domain Layer (COMPLETE)
- [x] Session aggregate implementation (aggregate.rs - 15 tests)
- [x] SessionEvent enum (events.rs - 13 tests)
- [x] Session-specific errors (errors.rs)
- [x] Domain validation rules

### Phase 2: Ports (COMPLETE)
- [x] SessionRepository trait (session_repository.rs)
- [x] SessionReader trait (session_reader.rs)
- [x] EventPublisher trait (event_publisher.rs)

### Phase 3: Commands (COMPLETE)
- [x] CreateSessionCommand + Handler (7 tests)
- [x] RenameSessionCommand + Handler (7 tests)
- [x] ArchiveSessionCommand + Handler (6 tests)
- [x] SessionCycleTracker event handler (8 tests)

### Phase 4: Queries (Not Started)
- [ ] GetSessionQuery + Handler
- [ ] ListUserSessionsQuery + Handler
- [ ] Query tests with mock readers

### Phase 5: HTTP Adapter (Not Started)
- [ ] Request/Response DTOs
- [ ] HTTP handlers
- [ ] Route definitions
- [ ] Handler tests

### Phase 6: Postgres Adapter (Not Started)
- [ ] Database migrations
- [ ] PostgresSessionRepository
- [ ] PostgresSessionReader
- [ ] Integration tests

### Phase 7: Frontend (Not Started)
- [ ] TypeScript types
- [ ] API client
- [ ] React hooks
- [ ] Components
- [ ] Component tests

---

## Notes

- Session only holds references to CycleIds, not the cycles themselves
- CQRS pattern: SessionRepository for writes, SessionReader for queries
- Domain events are pulled and published in command handlers
- PostgresSessionReader joins cycles table to get cycle_count
- Full-text search uses PostgreSQL tsvector for title/description

---

*Generated: 2026-01-07*
*Last Updated: 2026-01-09 (checklist-sync - verified)*
*Specification: docs/modules/session.md*
