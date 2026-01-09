# Architecture Validation

> **Purpose**: Validate feature specs, module specs, and integration specs against the system architecture.
> **Input**: Any spec file (feature brief, module spec, integration spec)
> **Output**: Validation report with issues, warnings, and suggestions
> **Time**: ~30 seconds (automated analysis)
> **Thinking**: Extended (ultrathink enabled)

ultrathink: This skill requires careful analysis of architectural constraints. Check dependency directions, validate phase ordering, detect circular dependencies, and identify potential issues before implementation begins.

---

## Usage

```
/architecture-validate <spec-path>
/architecture-validate features/events/waitlist.md
/architecture-validate docs/modules/cart.md
/architecture-validate features/integrations/guest-checkout.md
/architecture-validate --all                    # Validate all specs
```

### Arguments
- `spec-path`: Path to spec file OR `--all` for full validation
- `--architecture <path>`: Architecture file (defaults to `docs/architecture/SYSTEM-ARCHITECTURE.md`)
- `--fix`: Attempt to auto-fix issues where possible
- `--strict`: Treat warnings as errors

---

## Validation Rules

### Rule Categories

| Category | Severity | Description |
|----------|----------|-------------|
| Structure | Error | Missing required sections, invalid format |
| Architecture | Error | Module not in architecture, phase violations |
| Dependencies | Error | Circular deps, forward phase references |
| Consistency | Warning | Naming mismatches, stale references |
| Best Practice | Suggestion | Improvements, missing recommended sections |

---

## Structural Validation

### Feature Brief Required Fields

```yaml
required:
  - Architecture reference
  - Module name
  - Phase number
  - Module Dependencies
  - Description (> line)
  - Context (1+ items)
  - Tasks (1+ items)
  - Acceptance Criteria (1+ items)

recommended:
  - Feature Dependencies
  - Files Affected
  - Related links
```

### Module Spec Required Fields

```yaml
required:
  - Phase
  - Dependencies
  - Overview (2+ paragraphs)
  - Business Rules (with values)
  - API Endpoints (with auth)
  - Domain Layer (file structure)
  - Test Inventory (named tests)
  - Exit Criteria

recommended:
  - Database Schema
  - Test Builder Pattern
  - Error Codes table
```

### Integration Spec Required Fields

```yaml
required:
  - Architecture reference
  - Type (User Journey | System Process | Data Flow)
  - Overview
  - Modules Involved (3+)
  - Data Flow diagram
  - Coordination Points
  - Failure Modes
  - Implementation Phases
  - Testing Strategy

recommended:
  - Shared Types
  - API Contracts
  - Rollout Plan
```

---

## Architecture Validation

### Module Existence

Check that referenced modules exist in architecture:

```markdown
# From spec:
**Module:** Events

# Validation:
✅ Module "Events" found in architecture (Phase 2)
❌ Module "Notifications" not found in architecture
```

### Phase Correctness

Verify phase matches architecture:

```markdown
# From spec:
**Module:** Cart
**Phase:** 2

# From architecture:
Cart: Phase 3 (depends on Events, Programs, Facilities)

# Validation:
❌ Phase mismatch: spec says Phase 2, architecture says Phase 3
```

### Dependency Direction

Ensure dependencies flow from higher to lower phases:

```markdown
# Valid (lower phase dependency):
Events (Phase 2) → Foundation (Phase 1) ✅

# Invalid (forward phase reference):
Events (Phase 2) → Cart (Phase 3) ❌
  Error: Cannot depend on higher-phase module
```

---

## Dependency Validation

### Circular Dependency Detection

Build dependency graph and check for cycles:

```markdown
# Cycle detected:
Cart → Checkout → Payments → Cart
       ↑_________________________↓

❌ Circular dependency: Cart → Checkout → Payments → Cart
   Suggestion: Extract shared interface to Foundation
```

### Feature Dependency Validation

Verify declared feature dependencies exist:

```markdown
# From spec:
**Feature Dependencies:**
- features/memberships/membership-tiers.md

# Validation:
✅ Feature "membership-tiers" exists
❌ Feature "membership-discounts" not found at features/memberships/membership-discounts.md
```

### Module Import Validation

For integration specs, validate all modules are compatible:

```markdown
# Modules Involved: Cart, Checkout, Payments, Email

# Validation:
✅ All modules exist in architecture
✅ No circular dependencies between modules
⚠️ Email (Phase 5) is much higher than Cart (Phase 3)
   Consider: Is this dependency necessary?
```

---

## Consistency Validation

### Naming Convention

Check names follow conventions:

```markdown
# File naming:
✅ waitlist.md (kebab-case)
❌ WaitList.md (should be kebab-case)
❌ wait_list.md (should be kebab-case)

# Test naming:
✅ TestEvent_JoinWaitlist_WhenFull_CreatesEntry
⚠️ TestJoinWaitlist (missing subject and scenario)
```

### Cross-Reference Validation

Check links point to existing files:

```markdown
# From spec:
- **Module Spec:** docs/modules/events.md
- **Checklist:** REQUIREMENTS/CHECKLIST-events.md

# Validation:
✅ docs/modules/events.md exists
❌ REQUIREMENTS/CHECKLIST-events.md not found
   Suggestion: Run /module-checklist events
```

### Terminology Consistency

Check consistent naming across specs:

```markdown
# Inconsistency detected:
- features/events/waitlist.md uses "WaitlistEntry"
- docs/modules/events.md uses "WaitListEntry"

⚠️ Inconsistent naming: "WaitlistEntry" vs "WaitListEntry"
   Recommendation: Use "WaitlistEntry" (matches domain code)
```

---

## Best Practice Validation

### Coverage Estimation

Estimate if test count is reasonable:

```markdown
# From spec:
- 5 aggregate methods
- 3 value objects
- 4 command handlers
- 3 HTTP endpoints

# Expected tests:
- Domain: ~25 tests (5*3 + 3*2 + events)
- Application: ~15 tests (4*3 + queries)
- Adapters: ~10 tests (3*2 + repository)
- Total: ~50 tests

# From spec: 32 tests

⚠️ Test count (32) seems low for scope
   Expected: ~50 tests based on component count
   Consider: Are error cases covered?
```

### Complexity Warning

Flag potentially over-scoped specs:

```markdown
# From spec:
- 15 new files
- 8 modified files
- 60+ tests
- 12 API endpoints

⚠️ High complexity detected
   Consider breaking into smaller features:
   - Phase 1: Core functionality (5 files, 20 tests)
   - Phase 2: Extended features (10 files, 25 tests)
   - Phase 3: Edge cases (8 files, 15 tests)
```

### Missing Sections

Flag recommended but missing sections:

```markdown
⚠️ Missing recommended section: "Test Builder Pattern"
   Benefit: Enables readable test setup
   Template:
   ```go
   func NewEventBuilder() *EventBuilder { ... }
   ```

⚠️ Missing recommended section: "Error Codes"
   Benefit: Documents API error responses
   Template:
   | Error Code | HTTP Status | Condition |
```

---

## Output Format

### Validation Report

```markdown
# Architecture Validation Report

**Spec:** features/events/waitlist.md
**Type:** Feature Brief
**Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md
**Validated:** 2026-01-07 14:30:00

---

## Summary

| Category | Errors | Warnings | Suggestions |
|----------|--------|----------|-------------|
| Structure | 0 | 1 | 0 |
| Architecture | 0 | 0 | 0 |
| Dependencies | 1 | 0 | 1 |
| Consistency | 0 | 2 | 0 |
| Best Practice | 0 | 1 | 2 |
| **Total** | **1** | **4** | **3** |

**Status:** ❌ FAILED (1 error)

---

## Errors (Must Fix)

### E001: Missing Feature Dependency
**Location:** Feature Dependencies section
**Problem:** Referenced feature does not exist
**Details:**
```
Declared: features/memberships/membership-discounts.md
Status: File not found
```
**Fix:** Create the dependency feature first, or remove the reference

---

## Warnings (Should Fix)

### W001: Test Count Low
**Location:** Test Inventory
**Problem:** Fewer tests than expected for scope
**Details:**
```
Declared: 32 tests
Expected: ~50 tests (based on 5 aggregates, 4 handlers)
```
**Recommendation:** Review error case coverage

### W002: Missing Files Affected Section
**Location:** Document structure
**Problem:** Recommended section not present
**Recommendation:** Add "Files Affected" section listing new/modified files

### W003: Inconsistent Naming
**Location:** Domain Layer
**Problem:** "WaitlistEntry" vs "WaitListEntry"
**Recommendation:** Standardize to "WaitlistEntry"

### W004: Stale Cross-Reference
**Location:** Related section
**Problem:** Linked file doesn't exist
**Details:**
```
Link: REQUIREMENTS/CHECKLIST-events.md
Status: File not found
```
**Fix:** Run `/module-checklist events` to generate

---

## Suggestions (Nice to Have)

### S001: Add Test Builder Pattern
**Benefit:** Improves test readability and maintenance
**Template available:** Yes (see /tdd-fixture)

### S002: Add Error Codes Table
**Benefit:** Documents API error responses for consumers
**Template:**
```markdown
| Error Code | HTTP Status | Condition |
|------------|-------------|-----------|
| WAITLIST_FULL | 409 | Waitlist at capacity |
```

### S003: Consider Phased Implementation
**Reason:** High file count (15 files)
**Suggestion:** Break into 2-3 implementation phases

---

## Validation Checklist

- [x] Architecture file exists
- [x] Module exists in architecture
- [x] Phase matches architecture
- [x] Dependencies are lower phase
- [ ] Feature dependencies exist
- [x] No circular dependencies
- [x] Required sections present
- [ ] Recommended sections present
- [x] Naming conventions followed
- [ ] Cross-references valid

---

## Auto-Fix Available

The following can be auto-fixed with `--fix`:
- [ ] Add missing section headers (W002)
- [ ] Update phase number from architecture

Run: `/architecture-validate features/events/waitlist.md --fix`

---

## Next Steps

1. Fix error E001 (create missing feature or remove reference)
2. Address warnings W001-W004
3. Consider suggestions S001-S003
4. Re-run validation: `/architecture-validate features/events/waitlist.md`
```

---

## Batch Validation

When using `--all`:

```markdown
# Architecture Validation Report (Batch)

**Scope:** All specs in project
**Files Scanned:** 23
**Validated:** 2026-01-07 14:30:00

---

## Summary by File

| File | Errors | Warnings | Status |
|------|--------|----------|--------|
| features/events/waitlist.md | 0 | 2 | ⚠️ |
| features/events/recurring.md | 1 | 0 | ❌ |
| features/cart/guest-cart.md | 0 | 0 | ✅ |
| features/integrations/guest-checkout.md | 0 | 3 | ⚠️ |
| docs/modules/events.md | 0 | 1 | ⚠️ |
| docs/modules/cart.md | 0 | 0 | ✅ |

---

## Global Issues

### Orphaned Features
Features not referenced by any module spec:
- features/events/legacy-import.md

### Missing Specs
Modules in architecture without specs:
- Volunteering (Phase 5)
- Reporting (Phase 6)

### Dependency Graph

```
Foundation (Phase 1)
├── Events (Phase 2) ✅
├── Programs (Phase 2) ⚠️ spec incomplete
├── Facilities (Phase 2) ✅
├── Memberships (Phase 2) ✅
│
├── Cart (Phase 3) ✅
│   └── depends on: Events, Programs, Facilities ✅
│
└── Checkout (Phase 4) ⚠️ missing spec
    └── depends on: Cart ⚠️
```

---

## Recommended Actions

1. **Fix 1 error** in features/events/recurring.md
2. **Create specs** for Volunteering, Reporting, Checkout
3. **Complete** Programs module spec
4. **Review** 3 orphaned features
```

---

## Integration with CI/CD

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Validate changed spec files
changed_specs=$(git diff --cached --name-only | grep -E '\.(md)$' | grep -E '(features|docs/modules)/')

if [ -n "$changed_specs" ]; then
    for spec in $changed_specs; do
        claude "/architecture-validate $spec --strict"
        if [ $? -ne 0 ]; then
            echo "❌ Validation failed for $spec"
            exit 1
        fi
    done
fi
```

### GitHub Action

```yaml
name: Validate Specs
on:
  pull_request:
    paths:
      - 'features/**/*.md'
      - 'docs/modules/**/*.md'

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Validate specs
        run: |
          for spec in $(git diff --name-only origin/main | grep -E '(features|docs/modules)/.*\.md$'); do
            echo "Validating $spec..."
            # Run validation (adjust command for your setup)
          done
```

---

## Error Reference

### Error Codes

| Code | Category | Description |
|------|----------|-------------|
| E001 | Dependencies | Missing feature dependency |
| E002 | Architecture | Module not in architecture |
| E003 | Architecture | Phase mismatch |
| E004 | Dependencies | Circular dependency detected |
| E005 | Dependencies | Forward phase reference |
| E006 | Structure | Missing required section |
| E007 | Structure | Invalid format |

### Warning Codes

| Code | Category | Description |
|------|----------|-------------|
| W001 | Best Practice | Test count low |
| W002 | Structure | Missing recommended section |
| W003 | Consistency | Inconsistent naming |
| W004 | Consistency | Stale cross-reference |
| W005 | Best Practice | High complexity |
| W006 | Architecture | Phase gap in dependencies |

### Suggestion Codes

| Code | Category | Description |
|------|----------|-------------|
| S001 | Best Practice | Add test builder pattern |
| S002 | Best Practice | Add error codes table |
| S003 | Best Practice | Consider phased implementation |
| S004 | Structure | Add files affected section |

---

## See Also

- `/feature-brief` - Create feature specifications
- `/integration-spec` - Create cross-module specifications
- `/module-spec` - Create module specifications
- `/module-refine` - Improve module specifications
- `/hexagonal-design` - Create system architecture

---

*Version: 1.0.0*
*Created: 2026-01-07*
