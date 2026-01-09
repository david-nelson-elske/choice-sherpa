# Module Checklist Generator

> **Purpose**: Generate a detailed implementation tracking checklist from a module specification.
> **Input**: `docs/modules/<name>.md` - Module specification
> **Output**: `REQUIREMENTS/CHECKLIST-<name>.md` - Trackable implementation checklist
> **Time**: ~1 minute (automated extraction)

---

## Usage

```
/module-checklist <spec-path>
/module-checklist docs/modules/waitlist.md
/module-checklist events                      # Shorthand for docs/modules/events.md
```

### Arguments
- `spec-path`: Path to module specification OR module name

---

## Output Format

Creates `REQUIREMENTS/CHECKLIST-<name>.md`:

```markdown
# [Module] Module Checklist

**Module:** [Name]
**Dependencies:** [modules]
**Phase:** [1-4]

---

## Overview
[Brief description from spec]

---

## File Inventory

### Domain Layer
| File | Description | Status |
|------|-------------|--------|
| `path/to/file.go` | Description | ⬜ |

### Ports
| File | Description | Status |

### Application Layer
| File | Description | Status |

### Adapters
| File | Description | Status |

---

## Test Inventory

### Domain Layer Tests
| Test Name | Description | Status |
|-----------|-------------|--------|

### Application Layer Tests
| Test Name | Description | Status |

### Adapter Layer Tests
| Test Name | Description | Status |

---

## API Endpoints
[From spec]

---

## Error Codes
[From spec]

---

## Business Rules
[Mapped to tests]

---

## Verification Commands
[Generated commands]

---

## Exit Criteria
[From spec with checkboxes]

---

## Exit Signal
[Expected completion signal]
```

---

## Extraction Rules

### File Inventory Extraction

From module spec's "Domain Layer" and "File Structure" sections:

**Input (from spec):**
```markdown
### File Structure
```
backend/internal/domain/events/
├── waitlist.go              # WaitlistEntry entity
├── waitlist_test.go         # Waitlist tests
```
```

**Output (checklist):**
```markdown
### Domain Layer
| File | Description | Status |
|------|-------------|--------|
| `backend/internal/domain/events/waitlist.go` | WaitlistEntry entity | ⬜ |
| `backend/internal/domain/events/waitlist_test.go` | Waitlist tests | ⬜ |
```

### Test Inventory Extraction

From module spec's "Test Inventory" section:

**Input (from spec):**
```markdown
#### Event Aggregate (Waitlist) Tests (12)
```
TestEvent_JoinWaitlist_WhenEventFull_CreatesEntry
TestEvent_JoinWaitlist_WhenEventNotFull_ReturnsErrEventNotFull
```
```

**Output (checklist):**
```markdown
### Domain Layer Tests

#### Event Aggregate (Waitlist) Tests
| Test Name | Description | Status |
|-----------|-------------|--------|
| `TestEvent_JoinWaitlist_WhenEventFull_CreatesEntry` | Join when full creates entry | ⬜ |
| `TestEvent_JoinWaitlist_WhenEventNotFull_ReturnsErrEventNotFull` | Cannot join when not full | ⬜ |
```

### Description Derivation

Generate descriptions from test names:

| Test Name Pattern | Description Pattern |
|-------------------|---------------------|
| `Test[A]_[M]_When[C]_[R]` | "[M] when [C] [R]" |
| `Test[A]_[M]_With[X]_[R]` | "[M] with [X] [R]" |
| `Test[A]_[M]_[S]` | "[M] [S]" |
| `Test[A]_[M]_ReturnsErr[E]` | "[M] returns error [E]" |

Examples:
- `TestEvent_JoinWaitlist_WhenEventFull_CreatesEntry` → "Join waitlist when event full creates entry"
- `TestEvent_Register_ReturnsErrEventFull` → "Register returns error EventFull"

---

## Business Rules Mapping

Link business rules to their tests:

**Input (from spec):**
```markdown
## Business Rules
| Rule | Value | Implementation |
|------|-------|----------------|
| Max waitlist size | 2x capacity | `event.capacity * 2` |
```

**Output (checklist):**
```markdown
## Business Rules

| Rule | Implementation | Test | Status |
|------|----------------|------|--------|
| Max waitlist size = 2x capacity | `event.capacity * 2` | `TestEvent_JoinWaitlist_WhenWaitlistFull_ReturnsErrWaitlistFull` | ⬜ |
```

---

## Verification Commands Generation

Generate verification commands based on file paths:

```markdown
## Verification Commands

```bash
# Domain tests
go test ./backend/internal/domain/events/... -v -run "Waitlist"

# Application tests
go test ./backend/internal/application/commands/... -v -run "Waitlist"

# Adapter tests (requires Docker)
go test -tags=integration ./backend/internal/adapters/postgres/... -v -run "Waitlist"
go test ./backend/internal/adapters/http/events/... -v -run "Waitlist"

# Coverage check (target: 90%+)
go test ./backend/internal/domain/events/... -cover | grep -E "waitlist"

# Full verification
./scripts/verify-module.sh [module-name]
```
```

---

## Error Codes Extraction

From API endpoints and domain errors:

**Input (from spec):**
```markdown
### Eligibility
| Scenario | Result | Error |
|----------|--------|-------|
| Event not full | Cannot join | `ErrEventNotFull` |
```

**Output (checklist):**
```markdown
## Error Codes

| Error Code | HTTP Status | Condition |
|------------|-------------|-----------|
| `WAITLIST_EVENT_NOT_FULL` | 400 | Event has available spots |
| `WAITLIST_ALREADY_ON` | 409 | User already on waitlist |
| `WAITLIST_FULL` | 409 | Waitlist at capacity |
| `WAITLIST_PROMOTION_EXPIRED` | 410 | Confirmation window closed |
```

---

## Exit Criteria Formatting

Convert spec exit criteria to checkboxes:

**Input (from spec):**
```markdown
## Exit Criteria
### Files Complete
- [ ] Domain files (waitlist.go, waitlist_test.go)
```

**Output (checklist):**
```markdown
## Exit Criteria

### Module is COMPLETE when:
- [ ] All files in File Inventory exist
- [ ] All tests in Test Inventory pass
- [ ] Domain layer coverage >= 90%
- [ ] Application layer coverage >= 85%
- [ ] Adapter layer coverage >= 80%
- [ ] All API endpoints return correct responses
- [ ] No lint errors

### Exit Signal
```
MODULE COMPLETE: [module-name]
Files: XX/XX
Tests: XX/XX passing
Coverage: Domain XX%, Application XX%, Adapters XX%
```
```

---

## Status Symbols

| Symbol | Meaning | When to Use |
|--------|---------|-------------|
| ⬜ | Not started | Initial state |
| 🔄 | In progress | Currently working |
| ✅ | Complete | Tests passing |
| ❌ | Blocked | Cannot proceed |
| ⏭️ | Skipped | Intentionally omitted |

---

## Complete Example

Given `docs/modules/waitlist.md`, generates:

```markdown
# Waitlist Module Checklist

**Module:** Waitlist
**Dependencies:** Events, Foundation
**Phase:** 2

---

## Overview

Allow users to join a queue when events reach capacity, with automatic
promotion when spots open.

---

## File Inventory

### Domain Layer
| File | Description | Status |
|------|-------------|--------|
| `backend/internal/domain/events/waitlist.go` | WaitlistEntry entity | ⬜ |
| `backend/internal/domain/events/waitlist_test.go` | Waitlist tests | ⬜ |

### Ports
| File | Description | Status |
|------|-------------|--------|
| `backend/internal/ports/event_repository.go` | Add waitlist methods | ⬜ |

### Application Layer
| File | Description | Status |
|------|-------------|--------|
| `backend/internal/application/commands/join_waitlist.go` | Join command | ⬜ |
| `backend/internal/application/commands/join_waitlist_test.go` | Command tests | ⬜ |
| `backend/internal/application/commands/leave_waitlist.go` | Leave command | ⬜ |
| `backend/internal/application/commands/confirm_promotion.go` | Confirm command | ⬜ |
| `backend/internal/application/commands/confirm_promotion_test.go` | Command tests | ⬜ |

### Adapters
| File | Description | Status |
|------|-------------|--------|
| `backend/internal/adapters/postgres/event_repository.go` | Add waitlist queries | ⬜ |
| `backend/internal/adapters/http/events/waitlist_handlers.go` | HTTP handlers | ⬜ |
| `backend/internal/adapters/http/events/waitlist_handlers_test.go` | Handler tests | ⬜ |

---

## Test Inventory

### Domain Layer Tests

#### Event Aggregate (Waitlist) Tests
| Test Name | Description | Status |
|-----------|-------------|--------|
| `TestEvent_JoinWaitlist_WhenEventFull_CreatesEntry` | Join when full creates entry | ⬜ |
| `TestEvent_JoinWaitlist_WhenEventNotFull_ReturnsErrEventNotFull` | Cannot join when not full | ⬜ |
| `TestEvent_JoinWaitlist_WhenAlreadyRegistered_ReturnsErrAlreadyRegistered` | Cannot join if registered | ⬜ |
| `TestEvent_JoinWaitlist_WhenAlreadyOnWaitlist_ReturnsErrAlreadyOnWaitlist` | Cannot join twice | ⬜ |
| `TestEvent_JoinWaitlist_WhenWaitlistFull_ReturnsErrWaitlistFull` | Cannot exceed limit | ⬜ |
| `TestEvent_JoinWaitlist_AssignsCorrectPosition` | Position is sequential | ⬜ |
| `TestEvent_JoinWaitlist_EmitsWaitlistJoinedEvent` | Domain event emitted | ⬜ |
| `TestEvent_LeaveWaitlist_RemovesEntry` | Entry removed | ⬜ |
| `TestEvent_LeaveWaitlist_ReordersPositions` | Positions recalculated | ⬜ |
| `TestEvent_PromoteNextFromWaitlist_PromotesFirstInLine` | FIFO order | ⬜ |
| `TestEvent_PromoteNextFromWaitlist_SetsExpirationTime` | 24h window set | ⬜ |
| `TestEvent_PromoteNextFromWaitlist_EmitsWaitlistPromotedEvent` | Domain event emitted | ⬜ |

#### WaitlistEntry Tests
| Test Name | Description | Status |
|-----------|-------------|--------|
| `TestWaitlistEntry_NewWaitlistEntry_SetsCorrectDefaults` | Default values | ⬜ |
| `TestWaitlistEntry_Promote_SetsPromotedAtAndExpiresAt` | Promotion timestamps | ⬜ |
| `TestWaitlistEntry_Confirm_TransitionsToConfirmed` | Status transition | ⬜ |
| `TestWaitlistEntry_Expire_TransitionsToExpired` | Expiration transition | ⬜ |
| `TestWaitlistEntry_IsExpired_ReturnsTrueAfterWindow` | Expiration check | ⬜ |
| `TestWaitlistEntry_CanConfirm_ReturnsFalseWhenExpired` | Cannot confirm expired | ⬜ |

### Application Layer Tests
| Test Name | Description | Status |
|-----------|-------------|--------|
| `TestJoinWaitlistHandler_Execute_Success` | Happy path | ⬜ |
| `TestJoinWaitlistHandler_Execute_EventNotFound` | 404 case | ⬜ |
| `TestJoinWaitlistHandler_Execute_EventNotFull` | 400 case | ⬜ |
| `TestLeaveWaitlistHandler_Execute_Success` | Leave success | ⬜ |
| `TestConfirmPromotionHandler_Execute_Success` | Confirm success | ⬜ |
| `TestConfirmPromotionHandler_Execute_Expired` | 410 case | ⬜ |
| `TestPromoteFromWaitlistHandler_Execute_OnCancellation` | Event handler | ⬜ |
| `TestExpirePromotionsHandler_Execute_BatchExpiration` | Scheduled job | ⬜ |

### HTTP Handler Tests
| Test Name | Description | Status |
|-----------|-------------|--------|
| `TestWaitlistAPI_Join_ReturnsPosition` | POST success | ⬜ |
| `TestWaitlistAPI_Join_EventNotFull_Returns400` | 400 response | ⬜ |
| `TestWaitlistAPI_Leave_Success` | DELETE success | ⬜ |
| `TestWaitlistAPI_GetPosition_ReturnsCurrentPosition` | GET position | ⬜ |
| `TestWaitlistAPI_Confirm_CreatesRegistration` | POST confirm | ⬜ |
| `TestWaitlistAPI_Confirm_Expired_Returns410` | 410 response | ⬜ |

---

## API Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | `/api/events/:id/waitlist` | Join waitlist | Yes |
| DELETE | `/api/events/:id/waitlist` | Leave waitlist | Yes |
| GET | `/api/events/:id/waitlist/position` | Get position | Yes |
| POST | `/api/events/:id/waitlist/confirm` | Confirm promotion | Yes |
| GET | `/api/admin/events/:id/waitlist` | View waitlist | Admin |

---

## Error Codes

| Error Code | HTTP Status | Condition |
|------------|-------------|-----------|
| `WAITLIST_EVENT_NOT_FULL` | 400 | Event has available spots |
| `WAITLIST_ALREADY_REGISTERED` | 409 | User already registered |
| `WAITLIST_ALREADY_ON` | 409 | User already on waitlist |
| `WAITLIST_FULL` | 409 | Waitlist at capacity |
| `WAITLIST_NOT_ON` | 404 | User not on waitlist |
| `WAITLIST_PROMOTION_EXPIRED` | 410 | Confirmation window closed |

---

## Business Rules

| Rule | Implementation | Test | Status |
|------|----------------|------|--------|
| Max waitlist = 2x capacity | `event.capacity * 2` | `TestEvent_JoinWaitlist_WhenWaitlistFull_ReturnsErrWaitlistFull` | ⬜ |
| Promotion window = 24h | `time.Now().Add(24 * time.Hour)` | `TestWaitlistEntry_IsExpired_ReturnsTrueAfterWindow` | ⬜ |
| FIFO promotion order | `ORDER BY created_at ASC` | `TestEvent_PromoteNextFromWaitlist_PromotesFirstInLine` | ⬜ |
| Must be full to join | Check `IsFull()` | `TestEvent_JoinWaitlist_WhenEventNotFull_ReturnsErrEventNotFull` | ⬜ |

---

## Verification Commands

```bash
# Domain tests
go test ./backend/internal/domain/events/... -v -run "Waitlist"

# Application tests
go test ./backend/internal/application/commands/... -v -run "Waitlist"

# HTTP handler tests
go test ./backend/internal/adapters/http/events/... -v -run "Waitlist"

# Coverage (target: 90%+)
go test ./backend/internal/domain/events/... -cover | grep waitlist

# Full module verification
./scripts/verify-module.sh waitlist
```

---

## Exit Criteria

### Module is COMPLETE when:
- [ ] All 10 files in File Inventory exist
- [ ] All 32 tests in Test Inventory pass
- [ ] Domain layer coverage >= 90%
- [ ] Application layer coverage >= 85%
- [ ] All API endpoints return correct responses
- [ ] Promotion triggers on registration cancellation
- [ ] Email notification sent on promotion
- [ ] No lint errors

### Exit Signal
```
MODULE COMPLETE: waitlist
Files: 10/10
Tests: 32/32 passing
Coverage: Domain 92%, Application 87%
```
```

---

## See Also

- `/module-spec` - Create module specification
- `/module-refine` - Validate and improve specification
- `/tdd-domain` - Implement domain layer
- `/dev-checkpoint` - Check progress against checklist

---

*Version: 1.0.0*
*Created: 2026-01-07*
