# Feature Brief

Quickly capture feature intent within the established architecture.

## Usage

```
/feature-brief <name> [description]
/feature-brief waitlist "Allow users to join queue when events full"
/feature-brief --module events waitlist
```

| Option | Description |
|--------|-------------|
| `name` | Feature identifier (kebab-case) |
| `description` | One-line description |
| `--module` | Target module (auto-detected if omitted) |

---

## Output Location

`features/<module>/<name>.md`

---

## Output Template

```markdown
# Feature: [Title]

**Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md
**Module:** [Name]
**Phase:** [1-4]
**Module Dependencies:** [from architecture]
**Feature Dependencies:** [other features required]

> [One-line description]

---

## Context
- [Technical constraint]
- [Business rule]
- [Security consideration]

---

## Tasks
- [ ] [Specific task 1]
- [ ] [Specific task 2]

---

## Acceptance Criteria
- [ ] [Testable criterion 1]
- [ ] [Testable criterion 2]

---

## Files Affected

### New Files
- `backend/src/domain/<module>/<file>.rs`

### Modified Files
- `backend/src/domain/<module>/<existing>.rs`
```

---

## Validation Rules

| Rule | Error |
|------|-------|
| Module exists in architecture | "Module 'xyz' not found" |
| Dependencies are lower phase | "Cannot depend on Phase 3 from Phase 2" |
| Feature dependencies exist | "Feature 'xyz' not found" |

---

## Task Granularity

### Good (One TDD Cycle)

| Task Type | Example |
|-----------|---------|
| Value object | "Create Money value object with validation" |
| Aggregate method | "Add Register method to Event aggregate" |
| Endpoint | "Create POST /api/events endpoint" |

### Too Large (Break Down)

| Bad | Better |
|-----|--------|
| "Implement registration system" | Create entity, Add method, Create handler, Create endpoint |

---

## File Placement

| Type | Location |
|------|----------|
| Module feature | `features/<module>/<name>.md` |
| Cross-module | `features/integrations/<name>.md` (use `/integration-spec`) |
| Bug fix | `features/<module>/fixes/<name>.md` |

---

## When to Use /integration-spec Instead

| Indicator | Threshold |
|-----------|-----------|
| Modules modified | 3+ |
| Module coordination | Required |
| Cross-module failures | Complex |
| New shared types | Needed |
