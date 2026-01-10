# Module Refinement

Validate and improve a module specification against project standards.

## Usage

```
/module-refine <spec-path>
/module-refine docs/modules/waitlist.md
/module-refine waitlist                    # Shorthand
```

---

## Validation Phases

### Phase 1: Structural Completeness

| Section | Required | Check |
|---------|----------|-------|
| Overview | Yes | 2+ paragraphs |
| Business Rules | Yes | Tables with concrete values |
| Database Schema | Conditional | If new tables needed |
| API Endpoints | Yes | Table with auth |
| Domain Layer | Yes | File structure + methods |
| Test Inventory | Yes | Named tests by layer |
| Exit Criteria | Yes | Measurable checklist |

### Phase 2: Business Rules

| Check | Requirement |
|-------|-------------|
| Concrete values | No "TBD" placeholders |
| Implementation mapping | Each rule has code ref |
| Error definitions | Each constraint has error |

### Phase 3: Test Coverage

| Layer | Distribution |
|-------|--------------|
| Domain | 40-60% of tests |
| Application | 20-30% |
| Adapters | 20-30% |

**Test count formula:**
```
Domain = (methods * 3) + (value_objects * 2) + events
Application = (commands * 3) + (queries * 2)
Adapters = (endpoints * 2) + (repo_methods * 1)
```

### Phase 4: API Consistency

| Check | Requirement |
|-------|-------------|
| RESTful paths | No verbs in URLs |
| Auth specified | Every endpoint |
| Error mapping | Domain error → HTTP status |

### Phase 5: Domain Model

| Check | Requirement |
|-------|-------------|
| Aggregate boundaries | Single aggregate per operation |
| Domain events | State changes emit events |
| Value objects | Immutable concepts identified |

### Phase 6: Dependencies

| Check | Requirement |
|-------|-------------|
| Phase assignment | Matches dependency depth |
| No circular deps | Unidirectional only |
| Explicit list | Dependencies documented |

---

## Report Format

```markdown
# Module Refinement Report: [Name]

## Summary
- **Issues:** X (must fix)
- **Warnings:** Y (should fix)
- **Suggestions:** Z (nice to have)

## Issues
### 1. [Title]
**Location:** [Section]
**Problem:** [Description]
**Fix:** [Action]

## Warnings
### 1. [Title]
**Recommendation:** [Action]

## Auto-Fixes Applied
1. [Change made]
```

---

## Auto-Fix Capabilities

| Issue | Auto-Fix |
|-------|----------|
| Missing section headers | Add placeholder |
| Missing test totals | Calculate from inventory |
| Missing Exit Signal | Generate template |
| Wrong phase | Recalculate from dependencies |

---

## Common Issues Checklist

| Category | Check |
|----------|-------|
| Business Rules | No TBD, has implementation refs, has errors |
| Tests | Follow naming, cover rules, include errors |
| API | RESTful, auth specified, errors documented |
| Domain | Clear boundaries, events for state changes |
| Structure | Correct phase, deps listed, exit measurable |
