# Architecture Validation

Validate specs against the system architecture.

ultrathink: Check dependency directions, validate phase ordering, detect circular dependencies, and identify potential issues before implementation.

## Usage

```
/architecture-validate <spec-path>
/architecture-validate features/events/waitlist.md
/architecture-validate --all                    # All specs
```

| Option | Description |
|--------|-------------|
| `--fix` | Auto-fix issues where possible |
| `--strict` | Treat warnings as errors |

---

## Validation Categories

| Category | Severity | Description |
|----------|----------|-------------|
| Structure | Error | Missing sections, invalid format |
| Architecture | Error | Module not found, phase violations |
| Dependencies | Error | Circular deps, forward phase refs |
| Consistency | Warning | Naming mismatches, stale refs |
| Best Practice | Suggestion | Missing recommended sections |

---

## Required Fields by Spec Type

### Feature Brief

| Field | Required |
|-------|----------|
| Architecture reference | Yes |
| Module name | Yes |
| Phase | Yes |
| Module Dependencies | Yes |
| Context | Yes (1+) |
| Tasks | Yes (1+) |
| Acceptance Criteria | Yes (1+) |

### Module Spec

| Field | Required |
|-------|----------|
| Phase | Yes |
| Dependencies | Yes |
| Overview | Yes (2+ paragraphs) |
| Business Rules | Yes |
| API Endpoints | Yes |
| Domain Layer | Yes |
| Test Inventory | Yes |
| Exit Criteria | Yes |

### Integration Spec

| Field | Required |
|-------|----------|
| Architecture reference | Yes |
| Type | Yes |
| Modules Involved | Yes (3+) |
| Data Flow | Yes |
| Coordination Points | Yes |
| Failure Modes | Yes |
| Implementation Phases | Yes |
| Testing Strategy | Yes |

---

## Architecture Checks

| Check | Rule |
|-------|------|
| Module exists | Must be in architecture inventory |
| Phase matches | Spec phase = architecture phase |
| Dependency direction | Only lower phase dependencies |
| No circular deps | Build graph must be acyclic |

---

## Dependency Validation

| Check | Rule |
|-------|------|
| Feature deps exist | All declared features found |
| Phase order | Cannot depend on higher phase |
| Cross-refs valid | Linked files exist |

---

## Report Format

```markdown
# Architecture Validation Report

**Spec:** [path]
**Type:** [Feature Brief | Module Spec | Integration Spec]
**Status:** ✅ PASSED | ❌ FAILED (X errors)

## Summary
| Category | Errors | Warnings | Suggestions |
|----------|--------|----------|-------------|
| Structure | 0 | 1 | 0 |
| Architecture | 0 | 0 | 0 |
| Total | 0 | 1 | 0 |

## Errors
### E001: [Title]
**Location:** [Section]
**Problem:** [Description]
**Fix:** [Action]

## Warnings
### W001: [Title]
**Recommendation:** [Action]

## Suggestions
### S001: [Title]
**Benefit:** [Why]
```

---

## Error Codes

| Code | Category | Description |
|------|----------|-------------|
| E001 | Dependencies | Missing feature dependency |
| E002 | Architecture | Module not in architecture |
| E003 | Architecture | Phase mismatch |
| E004 | Dependencies | Circular dependency |
| E005 | Dependencies | Forward phase reference |
| E006 | Structure | Missing required section |

---

## Warning Codes

| Code | Category | Description |
|------|----------|-------------|
| W001 | Best Practice | Test count low |
| W002 | Structure | Missing recommended section |
| W003 | Consistency | Inconsistent naming |
| W004 | Consistency | Stale cross-reference |
| W005 | Best Practice | High complexity |

---

## Auto-Fix Capabilities

| Issue | Auto-Fix |
|-------|----------|
| Missing section headers | Add placeholder |
| Phase mismatch | Update from architecture |
| Stale references | Remove or flag |
