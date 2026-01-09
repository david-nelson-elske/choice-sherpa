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
| `backend/src/domain/session/session.rs` | Session aggregate | ⬜ |
| `backend/src/domain/session/events.rs` | SessionEvent enum (13 tests inline) | ✅ |
| `backend/src/domain/session/errors.rs` | Session-specific errors | ⬜ |

> **Note:** Tests are inline in implementation files using `#[cfg(test)] mod tests` (Rust convention).

### Domain Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/session/session_test.rs` | Session aggregate tests | ⬜ |
| `backend/src/domain/session/events_test.rs` | Event tests | ⬜ |

### Ports (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/ports/session_repository.rs` | SessionRepository trait | ⬜ |
| `backend/src/ports/session_reader.rs` | SessionReader trait (CQRS) | ⬜ |
| `backend/src/ports/domain_event_publisher.rs` | DomainEventPublisher trait | ⬜ |

### Application Layer - Commands (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/commands/create_session.rs` | CreateSession handler | ⬜ |
| `backend/src/application/commands/rename_session.rs` | RenameSession handler | ⬜ |
| `backend/src/application/commands/archive_session.rs` | ArchiveSession handler | ⬜ |

### Application Layer - Command Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/commands/create_session_test.rs` | CreateSession tests | ⬜ |
| `backend/src/application/commands/rename_session_test.rs` | RenameSession tests | ⬜ |
| `backend/src/application/commands/archive_session_test.rs` | ArchiveSession tests | ⬜ |

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
| `backend/migrations/001_create_sessions.sql` | Sessions table | ⬜ |

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

### Session Aggregate Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_session_new_creates_with_active_status` | New session is active | ⬜ |
| `test_session_new_generates_unique_id` | Each call produces different ID | ⬜ |
| `test_session_new_sets_timestamps` | Created/updated are set | ⬜ |
| `test_session_new_requires_non_empty_title` | Empty title rejected | ⬜ |
| `test_session_new_rejects_title_over_500_chars` | Long title rejected | ⬜ |
| `test_session_new_emits_created_event` | Event is recorded | ⬜ |
| `test_session_reconstitute_preserves_all_fields` | Reconstitution works | ⬜ |
| `test_session_reconstitute_no_events` | No events on reconstitute | ⬜ |
| `test_session_is_owner_returns_true_for_owner` | Owner check works | ⬜ |
| `test_session_is_owner_returns_false_for_other` | Non-owner check works | ⬜ |
| `test_session_authorize_succeeds_for_owner` | Authorization works | ⬜ |
| `test_session_authorize_fails_for_non_owner` | Authorization rejects | ⬜ |
| `test_session_rename_updates_title` | Rename works | ⬜ |
| `test_session_rename_updates_timestamp` | Timestamp updates | ⬜ |
| `test_session_rename_emits_renamed_event` | Event is recorded | ⬜ |
| `test_session_rename_validates_title` | Validation runs | ⬜ |
| `test_session_rename_fails_when_archived` | Archived rejection | ⬜ |
| `test_session_update_description_works` | Description updates | ⬜ |
| `test_session_update_description_fails_when_archived` | Archived rejection | ⬜ |
| `test_session_add_cycle_appends_id` | Cycle added | ⬜ |
| `test_session_add_cycle_prevents_duplicates` | No duplicate cycles | ⬜ |
| `test_session_add_cycle_emits_event` | Event is recorded | ⬜ |
| `test_session_add_cycle_fails_when_archived` | Archived rejection | ⬜ |
| `test_session_archive_changes_status` | Status changes | ⬜ |
| `test_session_archive_emits_event` | Event is recorded | ⬜ |
| `test_session_archive_fails_when_already_archived` | Double archive rejected | ⬜ |
| `test_session_pull_domain_events_returns_and_clears` | Events pulled correctly | ⬜ |

### SessionEvent Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_session_event_session_id_returns_id` | ID accessor works | ⬜ |
| `test_session_event_type_returns_type_string` | Type accessor works | ⬜ |
| `test_session_event_serializes_to_json` | JSON serialization | ⬜ |
| `test_session_event_deserializes_from_json` | JSON deserialization | ⬜ |

### CreateSession Command Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_create_session_handler_success` | Happy path | ⬜ |
| `test_create_session_handler_with_description` | Description included | ⬜ |
| `test_create_session_handler_saves_to_repo` | Repo save called | ⬜ |
| `test_create_session_handler_publishes_events` | Events published | ⬜ |
| `test_create_session_handler_returns_id` | ID returned | ⬜ |
| `test_create_session_handler_validates_title` | Validation error | ⬜ |

### RenameSession Command Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_rename_session_handler_success` | Happy path | ⬜ |
| `test_rename_session_handler_not_found` | 404 case | ⬜ |
| `test_rename_session_handler_unauthorized` | 403 case | ⬜ |
| `test_rename_session_handler_archived` | Archived rejection | ⬜ |
| `test_rename_session_handler_updates_repo` | Repo update called | ⬜ |
| `test_rename_session_handler_publishes_events` | Events published | ⬜ |

### ArchiveSession Command Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_archive_session_handler_success` | Happy path | ⬜ |
| `test_archive_session_handler_not_found` | 404 case | ⬜ |
| `test_archive_session_handler_unauthorized` | 403 case | ⬜ |
| `test_archive_session_handler_already_archived` | Double archive rejection | ⬜ |
| `test_archive_session_handler_publishes_events` | Events published | ⬜ |

### GetSession Query Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_get_session_handler_success` | Happy path | ⬜ |
| `test_get_session_handler_not_found` | 404 case | ⬜ |
| `test_get_session_handler_returns_view` | View returned | ⬜ |

### ListUserSessions Query Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_list_sessions_handler_success` | Happy path | ⬜ |
| `test_list_sessions_handler_empty_list` | No sessions | ⬜ |
| `test_list_sessions_handler_filters_by_status` | Status filter | ⬜ |
| `test_list_sessions_handler_paginates` | Pagination works | ⬜ |
| `test_list_sessions_handler_returns_total` | Total count included | ⬜ |

### HTTP Handler Tests

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

### Postgres Repository Tests

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

### Postgres Reader Tests

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
| Title is non-empty | Constructor validation | `test_session_new_requires_non_empty_title` | ⬜ |
| Title max 500 chars | Constructor validation | `test_session_new_rejects_title_over_500_chars` | ⬜ |
| Only owner can modify | authorize() check | `test_session_authorize_fails_for_non_owner` | ⬜ |
| Archived sessions immutable | ensure_mutable() check | `test_session_rename_fails_when_archived` | ⬜ |
| Cycle IDs are unique | Duplicate check | `test_session_add_cycle_prevents_duplicates` | ⬜ |
| Status transitions valid | can_transition_to() | `test_session_archive_fails_when_already_archived` | ⬜ |

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

- [ ] All 45 files in File Inventory exist
- [ ] All 85 tests in Test Inventory pass
- [ ] Domain layer coverage >= 90%
- [ ] Application layer coverage >= 85%
- [ ] Adapter layer coverage >= 80%
- [ ] Database migrations run successfully
- [ ] HTTP endpoints return correct status codes
- [ ] CQRS pattern implemented (Repository + Reader)
- [ ] Domain events published correctly
- [ ] No clippy warnings
- [ ] Frontend components render correctly
- [ ] No TypeScript lint errors

### Current Status

```
RUST BACKEND IN PROGRESS: session
Files: 2/45 (4%)
Tests: 13/85 passing (15%)
Frontend: Not started
```

### Exit Signal

```
MODULE COMPLETE: session
Files: 45/45
Tests: 85/85 passing
Coverage: Domain 92%, Application 87%, Adapters 82%
```

---

## Implementation Phases

### Phase 1: Domain Layer (In Progress)
- [ ] Session aggregate implementation
- [x] SessionEvent enum (13 tests passing)
- [ ] Domain validation rules
- [ ] Domain layer tests (partial - events.rs)

### Phase 2: Ports
- [ ] SessionRepository trait
- [ ] SessionReader trait
- [ ] DomainEventPublisher trait
- [ ] View DTOs (SessionView, SessionSummary)

### Phase 3: Commands
- [ ] CreateSessionCommand + Handler
- [ ] RenameSessionCommand + Handler
- [ ] ArchiveSessionCommand + Handler
- [ ] Command tests with mock repos

### Phase 4: Queries
- [ ] GetSessionQuery + Handler
- [ ] ListUserSessionsQuery + Handler
- [ ] Query tests with mock readers

### Phase 5: HTTP Adapter
- [ ] Request/Response DTOs
- [ ] HTTP handlers
- [ ] Route definitions
- [ ] Handler tests

### Phase 6: Postgres Adapter
- [ ] Database migrations
- [ ] PostgresSessionRepository
- [ ] PostgresSessionReader
- [ ] Integration tests

### Phase 7: Frontend
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
*Specification: docs/modules/session.md*
