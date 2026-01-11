# Spec - Requirements-Driven Feature Specification

Generate minimal, TDD-ready specifications where requirements map directly to test cycles.

## Usage

```
/spec <name> [description]
/spec waitlist "Queue when events full"
/spec --module session rename-session
```

---

## Output Location

`features/<module>/<name>.md`

---

## Output Template

```markdown
# Feature: [Title]

**Module:** [name] | **Phase:** [1-4] | **Priority:** [P0-P3]

> [One-line description]

---

## Requirements

| ID | Rule | Given | When | Then | Error |
|----|------|-------|------|------|-------|
| R1 | [Business rule] | [Precondition] | [Action] | [Expected] | [ErrorType] |
| R2 | ... | ... | ... | ... | ... |
| S1 | [Security rule] | ... | ... | ... | ... |

---

## Tasks

- [ ] R1: [Imperative task matching requirement]
- [ ] R2: [Imperative task matching requirement]
- [ ] S1: [Security task] `sec`
- [ ] INT: [Integration task if needed]

---

## Context

- [Key constraint or assumption]
- [Relevant existing pattern to follow]

---

## Security (if any S* requirements)

| Field | Classification | Handling |
|-------|---------------|----------|
| [field] | [Internal/Confidential/Public] | [Requirement] |
```

---

## Requirement Format

### The Given-When-Then Structure

Each requirement row defines one testable behavior:

| Column | Purpose | Example |
|--------|---------|---------|
| ID | Reference for tasks | R1, R2, R3 |
| Rule | Business rule name | "Title required" |
| Given | Precondition/state | "Empty title" |
| When | Action taken | "Create session" |
| Then | Expected outcome | "Returns error" |
| Error | Error type (if failure) | `ValidationError` |

### Requirement Categories

| Prefix | Meaning |
|--------|---------|
| R | Core requirement (must test) |
| E | Edge case (important boundary) |
| S | Security requirement (triggers Security section load) |
| INT | Integration point |

### Task Tags

Append tags to tasks for selective context loading by `/flow`:

| Tag | Meaning | Loads |
|-----|---------|-------|
| `sec` | Security-sensitive | Security section |
| `perf` | Performance-critical | Perf notes if present |
| (none) | Standard task | Just requirement row |

```markdown
- [ ] S1: Validate user ownership `sec`
- [ ] R3: Cache result for 5 minutes `perf`
```

---

## Task-Requirement Mapping

**Every task references a requirement ID:**

```markdown
- [ ] R1: Reject empty title with ValidationError
- [ ] R2: Accept valid title up to 500 chars
- [ ] E1: Truncate description at 2000 chars
- [ ] S1: Verify user owns session before rename
```

**Task granularity = one TDD cycle:**
- Write failing test for requirement
- Implement minimal code to pass
- Refactor

---

## Minimal Context

Context section contains ONLY:
- Constraints that affect implementation
- Patterns to follow from existing code
- Dependencies that must be imported

**Bad context (too verbose):**
```
This feature implements session renaming which allows users to change
the title of their decision sessions after creation...
```

**Good context (actionable):**
```
- Follow Session aggregate pattern in `domain/session/aggregate.rs`
- Reuse ValidationError from foundation
- Title validation same as CreateSession
```

---

## When to Use

| Scenario | Use |
|----------|-----|
| New domain entity method | `/spec` |
| New HTTP endpoint | `/spec` |
| Bug fix with regression test | `/spec` |
| Complex multi-module feature | `/integration-spec` |
| Initial module design | `/module-spec` |

---

## Example Output

```markdown
# Feature: Rename Session

**Module:** session | **Phase:** 3 | **Priority:** P1

> Allow session owner to change session title

---

## Requirements

| ID | Rule | Given | When | Then | Error |
|----|------|-------|------|------|-------|
| R1 | Title required | Empty title | Rename | Reject | `ValidationError` |
| R2 | Title max 500 | 501 char title | Rename | Reject | `ValidationError` |
| S1 | Owner only | Non-owner user | Rename | Reject | `ForbiddenError` |
| R3 | Not archived | Archived session | Rename | Reject | `SessionArchived` |
| R4 | Title updated | Valid title | Rename | Returns old title | - |
| R5 | Event emitted | Successful rename | - | SessionRenamed published | - |

---

## Tasks

- [ ] R1: Reject rename with empty title
- [ ] R2: Reject rename with title over 500 chars
- [ ] S1: Reject rename by non-owner `sec`
- [ ] R3: Reject rename on archived session
- [ ] R4: Update title and return previous value
- [ ] R5: Emit SessionRenamed event on success

---

## Context

- Extend Session aggregate with `rename()` method
- Follow authorize/ensure_mutable pattern from archive()
- Reuse title validation from constructor

---

## Security

| Field | Classification | Handling |
|-------|---------------|----------|
| session.owner_id | Internal | Compare against authenticated user_id |
```

---

## Validation

Before saving spec, verify:

- [ ] Every requirement has Given/When/Then
- [ ] Every task references a requirement ID
- [ ] Tasks are test-sized (one assertion focus)
- [ ] Context has ≤5 actionable items
- [ ] S* requirements have Security section
- [ ] S* tasks have `sec` tag
- [ ] No redundant prose or explanations

---

## Token Efficiency

Target: **≤80 lines** for simple features, **≤100 lines** with security.

| Section | Max Lines | Loaded By /flow |
|---------|-----------|-----------------|
| Header | 5 | Always (planning) |
| Requirements table | 15 | Per-row (implementation) |
| Tasks | 15 | Always (tracking) |
| Context | 5 | Always |
| Security | 10 | Only for `sec` tagged tasks |

**Progressive loading saves ~60% context** during implementation:
- Planning: Header + Tasks + Context (~25 lines)
- Per task: 1 requirement row + Context (~10 lines)
- Security tasks: + Security section (~15 lines)

**If exceeding limits:** Split into multiple specs or use `/integration-spec`.
