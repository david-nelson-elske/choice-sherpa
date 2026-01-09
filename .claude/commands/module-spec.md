# Module Specification

> **Purpose**: Generate a comprehensive module specification from a feature brief or description.
> **Output**: `docs/modules/<name>.md` - Full TDD context bundle for implementation.
> **Time**: ~5-10 minutes to produce
> **Thinking**: Extended (ultrathink enabled)

ultrathink: This skill requires thorough analysis to produce a complete module specification. Derive business rules, design database schema, enumerate all test cases, and ensure the specification is comprehensive enough for TDD implementation.

---

## Usage

```
/module-spec <source>
/module-spec features/waitlist.md           # From feature brief
/module-spec "Event waitlist system"        # From description
/module-spec                                # Interactive mode
```

### Arguments
- `source`: Feature brief file path OR module description string

---

## Output Format

Creates `docs/modules/<name>.md`:

```markdown
# [Module] Module - TDD Context Bundle

**Phase:** [1-4]
**Dependencies:** [modules]
**Read Time:** ~X minutes

---

## Overview
[2-3 paragraphs describing the module's purpose and scope]

---

## Business Rules
[Tables of rules with values and implementations]

---

## Database Schema
[SQL CREATE statements with indexes]

---

## API Endpoints
[Table of endpoints with auth requirements]

---

## Domain Layer
[File structure, aggregate design, domain events]

---

## Test Inventory
[Exact test names organized by layer]

---

## Exit Criteria
[Completion checklist with signals]

---

## Test Builder Pattern
[Code example for test fixtures]
```

---

## Section Generation Guide

### 1. Overview

Extract from source:
- What problem does this solve?
- Who are the users?
- What are the key capabilities?

```markdown
## Overview

The Waitlist module allows users to join a queue when events reach capacity.
When a spot opens (cancellation), the first person in line is automatically
promoted and notified via email.

Key capabilities:
- Join waitlist for full events
- Automatic FIFO promotion on cancellation
- Email notification on promotion
- 24-hour confirmation window
```

### 2. Business Rules

Transform constraints into structured rules:

```markdown
## Business Rules

### Waitlist Limits
| Rule | Value | Implementation |
|------|-------|----------------|
| Max waitlist size | 2x event capacity | `event.capacity * 2` |
| Promotion window | 24 hours | `WaitlistEntry.expiresAt` |
| Position assignment | FIFO order | `ORDER BY created_at ASC` |

### Eligibility
| Scenario | Result | Error |
|----------|--------|-------|
| Event not full | Cannot join | `ErrEventNotFull` |
| Already registered | Cannot join | `ErrAlreadyRegistered` |
| Already on waitlist | Cannot join | `ErrAlreadyOnWaitlist` |
| Waitlist full | Cannot join | `ErrWaitlistFull` |
```

### 3. Database Schema

Design tables to support the domain:

```markdown
## Database Schema

### waitlist_entries
```sql
CREATE TABLE waitlist_entries (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id    UUID NOT NULL REFERENCES events(id),
    user_id     UUID NOT NULL REFERENCES users(id),
    position    INTEGER NOT NULL,
    status      waitlist_status NOT NULL DEFAULT 'waiting',
    promoted_at TIMESTAMPTZ,
    expires_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(event_id, user_id)
);

CREATE TYPE waitlist_status AS ENUM ('waiting', 'promoted', 'expired', 'cancelled');
CREATE INDEX idx_waitlist_event_position ON waitlist_entries(event_id, position);
```
```

### 4. API Endpoints

Map user actions to HTTP endpoints:

```markdown
## API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/events/:id/waitlist` | Yes | Join waitlist |
| DELETE | `/api/events/:id/waitlist` | Yes | Leave waitlist |
| GET | `/api/events/:id/waitlist/position` | Yes | Get my position |
| POST | `/api/events/:id/waitlist/confirm` | Yes | Confirm promotion |
| GET | `/api/admin/events/:id/waitlist` | Admin | View full waitlist |
```

### 5. Domain Layer

Design the aggregate and its components:

```markdown
## Domain Layer

### File Structure
```
backend/internal/domain/events/
├── waitlist.go              # WaitlistEntry entity
├── waitlist_test.go         # Waitlist tests
└── (extends event.go)       # Event aggregate methods
```

### Event Aggregate Extensions
```go
// New fields
waitlist []WaitlistEntry

// New methods
func (e *Event) JoinWaitlist(userID string, now time.Time) (*WaitlistEntry, error)
func (e *Event) LeaveWaitlist(userID string) error
func (e *Event) PromoteNextFromWaitlist(now time.Time) (*WaitlistEntry, error)
func (e *Event) ConfirmPromotion(userID string) (*Registration, error)
func (e *Event) ExpireUnconfirmedPromotions(now time.Time) []WaitlistEntry
func (e *Event) WaitlistPosition(userID string) (int, error)
func (e *Event) IsWaitlistFull() bool
```

### WaitlistEntry Entity
```go
type WaitlistEntry struct {
    id         WaitlistEntryID
    eventID    EventID
    userID     string
    position   int
    status     WaitlistStatus
    promotedAt *time.Time
    expiresAt  *time.Time
    createdAt  time.Time
}
```

### Domain Events
| Event | Trigger | Payload |
|-------|---------|---------|
| `WaitlistJoined` | User joins waitlist | eventID, userID, position |
| `WaitlistLeft` | User leaves waitlist | eventID, userID |
| `WaitlistPromoted` | Spot opened, user promoted | eventID, userID, expiresAt |
| `WaitlistConfirmed` | User confirmed promotion | eventID, userID, registrationID |
| `WaitlistExpired` | Promotion window closed | eventID, userID |
```

### 6. Test Inventory

Generate test names following naming convention:

```markdown
## Test Inventory

### Domain Layer Tests

#### Event Aggregate (Waitlist) Tests (12)
```
TestEvent_JoinWaitlist_WhenEventFull_CreatesEntry
TestEvent_JoinWaitlist_WhenEventNotFull_ReturnsErrEventNotFull
TestEvent_JoinWaitlist_WhenAlreadyRegistered_ReturnsErrAlreadyRegistered
TestEvent_JoinWaitlist_WhenAlreadyOnWaitlist_ReturnsErrAlreadyOnWaitlist
TestEvent_JoinWaitlist_WhenWaitlistFull_ReturnsErrWaitlistFull
TestEvent_JoinWaitlist_AssignsCorrectPosition
TestEvent_JoinWaitlist_EmitsWaitlistJoinedEvent
TestEvent_LeaveWaitlist_RemovesEntry
TestEvent_LeaveWaitlist_ReordersPositions
TestEvent_PromoteNextFromWaitlist_PromotesFirstInLine
TestEvent_PromoteNextFromWaitlist_SetsExpirationTime
TestEvent_PromoteNextFromWaitlist_EmitsWaitlistPromotedEvent
```

#### WaitlistEntry Tests (6)
```
TestWaitlistEntry_NewWaitlistEntry_SetsCorrectDefaults
TestWaitlistEntry_Promote_SetsPromotedAtAndExpiresAt
TestWaitlistEntry_Confirm_TransitionsToConfirmed
TestWaitlistEntry_Expire_TransitionsToExpired
TestWaitlistEntry_IsExpired_ReturnsTrueAfterWindow
TestWaitlistEntry_CanConfirm_ReturnsFalseWhenExpired
```

### Application Layer Tests (8)
```
TestJoinWaitlistHandler_Execute_Success
TestJoinWaitlistHandler_Execute_EventNotFound
TestJoinWaitlistHandler_Execute_EventNotFull
TestLeaveWaitlistHandler_Execute_Success
TestConfirmPromotionHandler_Execute_Success
TestConfirmPromotionHandler_Execute_Expired
TestPromoteFromWaitlistHandler_Execute_OnCancellation
TestExpirePromotionsHandler_Execute_BatchExpiration
```

### HTTP Handler Tests (6)
```
TestWaitlistAPI_Join_ReturnsPosition
TestWaitlistAPI_Join_EventNotFull_Returns400
TestWaitlistAPI_Leave_Success
TestWaitlistAPI_GetPosition_ReturnsCurrentPosition
TestWaitlistAPI_Confirm_CreatesRegistration
TestWaitlistAPI_Confirm_Expired_Returns410
```

**Total: 32 tests**
```

### 7. Exit Criteria

Define completion checklist:

```markdown
## Exit Criteria

### Files Complete
- [ ] Domain files (waitlist.go, waitlist_test.go)
- [ ] Port files (waitlist methods in event_repository.go)
- [ ] Application files (4 command handlers)
- [ ] Adapter files (repository methods, HTTP handlers)

### Tests Complete
- [ ] 32 tests passing
- [ ] Domain coverage >= 90%
- [ ] Application coverage >= 85%

### Integration Complete
- [ ] Cancellation triggers promotion (event handler)
- [ ] Promotion sends email notification
- [ ] Expired promotions handled by scheduled job

### Exit Signal
```
MODULE COMPLETE: waitlist
Files: 8/8
Tests: 32/32 passing
Coverage: Domain 92%, Application 87%
```
```

### 8. Test Builder Pattern

Provide fixture helpers:

```markdown
## Test Builder Pattern

```go
func NewWaitlistEntryBuilder() *WaitlistEntryBuilder {
    return &WaitlistEntryBuilder{
        entry: &WaitlistEntry{
            id:        NewWaitlistEntryID(),
            status:    WaitlistStatusWaiting,
            position:  1,
            createdAt: time.Now(),
        },
    }
}

func (b *WaitlistEntryBuilder) WithPosition(pos int) *WaitlistEntryBuilder {
    b.entry.position = pos
    return b
}

func (b *WaitlistEntryBuilder) Promoted() *WaitlistEntryBuilder {
    now := time.Now()
    b.entry.status = WaitlistStatusPromoted
    b.entry.promotedAt = &now
    expires := now.Add(24 * time.Hour)
    b.entry.expiresAt = &expires
    return b
}

func (b *WaitlistEntryBuilder) Expired() *WaitlistEntryBuilder {
    past := time.Now().Add(-25 * time.Hour)
    b.entry.status = WaitlistStatusPromoted
    b.entry.promotedAt = &past
    expires := past.Add(24 * time.Hour)
    b.entry.expiresAt = &expires
    return b
}

func (b *WaitlistEntryBuilder) Build() *WaitlistEntry {
    return b.entry
}
```
```

---

## Derivation Rules

### Test Name Generation

From business rule → test name:

| Rule | Test Name Pattern |
|------|-------------------|
| "Cannot X when Y" | `Test[Aggregate]_[Method]_When[Y]_ReturnsErr[X]` |
| "X must be Y" | `Test[Aggregate]_[Method]_With[Invalid]_ReturnsErr[Validation]` |
| "X triggers Y" | `Test[Aggregate]_[Method]_Emits[Y]Event` |
| "X results in Y" | `Test[Aggregate]_[Method]_[Scenario]_[Y]` |

### File Count Estimation

| Component | Files per Feature |
|-----------|-------------------|
| New aggregate | 2 (impl + test) |
| New entity | 2 (impl + test) |
| New value object | 1-2 |
| Command handler | 2 (impl + test) |
| Query handler | 1-2 |
| HTTP endpoints | 2-3 (handlers, dto, routes) |
| Repository methods | 0-1 (usually extends existing) |

### Test Count Estimation

| Component | Tests |
|-----------|-------|
| Aggregate method | 3-5 (happy + errors + events) |
| Value object | 2-4 |
| Command handler | 3-5 |
| Query handler | 2-3 |
| HTTP endpoint | 2-4 |

---

## Phase Assignment

Determine module phase based on dependencies:

| Phase | Criteria | Examples |
|-------|----------|----------|
| 1 | No dependencies (foundation) | shared, errors |
| 2 | Depends only on Phase 1 | events, facilities, memberships |
| 3 | Depends on Phase 2 modules | cart, checkout |
| 4 | Depends on Phase 3+ | reporting, analytics |

For feature extensions to existing modules, inherit the module's phase.

---

## Validation Before Output

Verify spec completeness:

- [ ] Overview explains purpose clearly
- [ ] All business rules have values (not "TBD")
- [ ] Database schema is valid SQL
- [ ] All API endpoints have auth specified
- [ ] Domain model matches business rules
- [ ] Test inventory covers all rules
- [ ] Exit criteria are measurable
- [ ] No circular dependencies

---

## See Also

- `/feature-brief` - Create lightweight feature file
- `/module-checklist` - Generate tracking checklist from spec
- `/module-refine` - Validate and improve spec
- `/hexagonal-design` - Full system architecture
- `/tdd-domain` - Implement domain layer
- `/dev-module` - Execute full module development

---

*Version: 1.0.0*
*Created: 2026-01-07*
