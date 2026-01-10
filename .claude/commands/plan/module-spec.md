# Module Specification

Generate a comprehensive module specification from a feature brief or description.

ultrathink: Derive business rules, design database schema, enumerate all test cases, and ensure the specification is comprehensive enough for TDD implementation.

## Usage

```
/module-spec <source>
/module-spec features/waitlist.md           # From feature brief
/module-spec "Event waitlist system"        # From description
```

---

## Output Location

`docs/modules/<name>.md`

---

## Output Structure

| Section | Content |
|---------|---------|
| Overview | Purpose and scope (2-3 paragraphs) |
| Business Rules | Tables with concrete values |
| Database Schema | SQL CREATE statements |
| API Endpoints | Table with auth requirements |
| Domain Layer | File structure, methods, events |
| Test Inventory | Named tests by layer |
| Exit Criteria | Measurable completion checklist |
| Test Builder | Fixture code example |

---

## Business Rules Format

```markdown
### [Rule Category]
| Rule | Value | Implementation |
|------|-------|----------------|
| Max waitlist | 2x capacity | `event.capacity * 2` |

### Eligibility
| Scenario | Result | Error |
|----------|--------|-------|
| Event not full | Cannot join | `ErrEventNotFull` |
```

---

## Domain Layer Format

```markdown
### Aggregate Extensions
| Method | Description |
|--------|-------------|
| `join_waitlist(user_id)` | Add user to waitlist |
| `promote_next()` | Promote first in line |

### Domain Events
| Event | Trigger | Payload |
|-------|---------|---------|
| `WaitlistJoined` | User joins | event_id, user_id, position |
```

---

## Test Inventory Format

```markdown
### Domain Layer Tests (12)
| Test Name | Description |
|-----------|-------------|
| `test_event_join_waitlist_when_full_creates_entry` | Join when full |
| `test_event_join_waitlist_when_not_full_returns_err` | Cannot join when not full |

### Application Layer Tests (8)
| Test Name | Description |
|-----------|-------------|
| `test_join_waitlist_handler_success` | Happy path |
```

---

## Test Count Estimation

| Component | Tests Per |
|-----------|-----------|
| Aggregate method | 3-5 (happy + errors + events) |
| Value object | 2-4 |
| Command handler | 3-5 |
| Query handler | 2-3 |
| HTTP endpoint | 2-4 |

---

## Phase Assignment

| Phase | Criteria |
|-------|----------|
| 1 | No dependencies (foundation) |
| 2 | Depends only on Phase 1 |
| 3 | Depends on Phase 2 modules |
| 4+ | Depends on Phase 3+ |

---

## Exit Criteria Format

```markdown
### Module is COMPLETE when:
- [ ] All files in File Inventory exist
- [ ] All tests pass
- [ ] Domain coverage >= 90%
- [ ] Application coverage >= 85%

### Exit Signal
MODULE COMPLETE: [module]
Files: XX/XX
Tests: XX/XX passing
Coverage: Domain XX%, Application XX%
```

---

## Reference

- Testing patterns: `.claude/lib/examples/rust/testing.md`
- Common patterns: `.claude/lib/examples/rust/common-patterns.md`
