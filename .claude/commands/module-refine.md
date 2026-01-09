# Module Refinement

> **Purpose**: Validate and improve a module specification against project standards.
> **Input**: `docs/modules/<name>.md` - Module specification to validate
> **Output**: Refined specification with identified issues fixed
> **Time**: ~2-5 minutes (analysis + fixes)

---

## Usage

```
/module-refine <spec-path>
/module-refine docs/modules/waitlist.md
/module-refine waitlist                    # Shorthand
```

### Arguments
- `spec-path`: Path to module specification OR module name

---

## Validation Phases

### Phase 1: Structural Completeness

Check all required sections exist:

| Section | Required | Check |
|---------|----------|-------|
| Overview | Yes | 2+ paragraphs explaining purpose |
| Business Rules | Yes | Tables with concrete values |
| Database Schema | Conditional | Required if new tables needed |
| API Endpoints | Yes | Table with auth requirements |
| Domain Layer | Yes | File structure + aggregate methods |
| Test Inventory | Yes | Named tests organized by layer |
| Exit Criteria | Yes | Measurable completion checklist |
| Test Builder Pattern | Recommended | Code example for fixtures |

**Missing Section Format:**
```markdown
## ⚠️ MISSING: [Section Name]

This section is required. Please add:
- [Specific content needed]
```

---

### Phase 2: Business Rule Validation

Check business rules for:

1. **Concrete Values**: No "TBD" or placeholders
   ```markdown
   # BAD
   | Max waitlist | TBD | `event.capacity * ???` |

   # GOOD
   | Max waitlist | 2x capacity | `event.capacity * 2` |
   ```

2. **Implementation Mapping**: Each rule has code reference
   ```markdown
   | Rule | Value | Implementation |
   |------|-------|----------------|
   | GST rate | 5% | `GST_RATE = 500` (basis points) |
   ```

3. **Error Definitions**: Each constraint has corresponding error
   ```markdown
   | Scenario | Result | Error |
   |----------|--------|-------|
   | Event full | Cannot register | `ErrEventFull` |
   ```

---

### Phase 3: Test Inventory Validation

Check test coverage:

1. **Naming Convention**: Follows `Test[Subject]_[Method]_[Scenario]_[Result]`
   ```
   # GOOD
   TestEvent_JoinWaitlist_WhenEventFull_CreatesEntry

   # BAD - missing scenario
   TestEvent_JoinWaitlist
   ```

2. **Business Rule Coverage**: Every rule has test
   ```markdown
   | Rule | Test |
   |------|------|
   | Max waitlist = 2x | TestEvent_JoinWaitlist_WhenWaitlistFull_ReturnsErrWaitlistFull |
   ```

3. **Layer Distribution**:
   - Domain: 40-60% of tests (pure business logic)
   - Application: 20-30% of tests (use case orchestration)
   - Adapters: 20-30% of tests (infrastructure)

4. **Test Count Estimation**:
   ```
   Domain tests = (aggregate methods * 3) + (value objects * 2) + (domain events * 1)
   Application tests = (commands * 3) + (queries * 2)
   Adapter tests = (HTTP endpoints * 2) + (repository methods * 1)
   ```

---

### Phase 4: API Consistency

Check API endpoints for:

1. **RESTful Convention**:
   ```markdown
   # GOOD
   | POST | /api/events/:id/waitlist | Join waitlist |
   | DELETE | /api/events/:id/waitlist | Leave waitlist |

   # BAD - action in URL
   | POST | /api/events/:id/joinWaitlist | Join |
   ```

2. **Auth Specification**: Every endpoint has auth requirement
   ```markdown
   | Method | Path | Auth |
   |--------|------|------|
   | POST | /api/events | Admin |
   | GET | /api/events | None |
   ```

3. **Error Response Mapping**: HTTP status for each domain error
   ```markdown
   | Domain Error | HTTP Status |
   |--------------|-------------|
   | ErrNotFound | 404 |
   | ErrValidation | 400 |
   | ErrConflict | 409 |
   | ErrExpired | 410 |
   ```

---

### Phase 5: Domain Model Validation

Check domain design for:

1. **Aggregate Boundaries**: Single aggregate per operation
   ```go
   // GOOD - operation on Event aggregate
   func (e *Event) JoinWaitlist(userID string) error

   // BAD - crossing aggregate boundaries
   func (e *Event) JoinWaitlist(user *User) error
   ```

2. **Domain Events**: State changes emit events
   ```markdown
   | State Change | Domain Event |
   |--------------|--------------|
   | User joins waitlist | WaitlistJoined |
   | User promoted | WaitlistPromoted |
   ```

3. **Value Object Identification**: Immutable concepts are value objects
   ```markdown
   # Should be Value Objects
   - Money, EmailAddress, UserID, EventID
   - TimeRange, Position, Status enums

   # Should be Entities
   - User, Event, Registration, WaitlistEntry
   ```

---

### Phase 6: Dependency Validation

Check module dependencies:

1. **Phase Assignment**: Correct based on dependencies
   ```markdown
   | Phase | Criteria |
   |-------|----------|
   | 1 | No dependencies (Foundation) |
   | 2 | Depends only on Phase 1 |
   | 3 | Depends on Phase 2 modules |
   | 4+ | Depends on Phase 3+ |
   ```

2. **No Circular Dependencies**:
   ```
   # BAD
   Events depends on Cart
   Cart depends on Events

   # GOOD
   Cart depends on Events (one direction)
   ```

3. **Explicit Dependency List**:
   ```markdown
   **Dependencies:** Foundation, Events
   ```

---

### Phase 7: Exit Criteria Validation

Check exit criteria are:

1. **Measurable**: Can be verified automatically
   ```markdown
   # GOOD
   - [ ] 32 tests passing
   - [ ] Domain coverage >= 90%

   # BAD
   - [ ] Code is clean
   - [ ] Works well
   ```

2. **Complete**: Cover all deliverables
   ```markdown
   - [ ] All files in File Inventory exist
   - [ ] All tests in Test Inventory pass
   - [ ] All API endpoints respond correctly
   - [ ] No lint errors
   ```

3. **Include Exit Signal**:
   ```markdown
   ### Exit Signal
   ```
   MODULE COMPLETE: waitlist
   Files: 10/10
   Tests: 32/32 passing
   Coverage: Domain 92%, Application 87%
   ```
   ```

---

## Refinement Report Format

After validation, output:

```markdown
# Module Refinement Report: [Module Name]

## Summary
- **Issues Found:** X
- **Warnings:** Y
- **Suggestions:** Z

## Issues (Must Fix)

### 1. [Issue Title]
**Location:** [Section]
**Problem:** [Description]
**Fix:** [Specific action]

## Warnings (Should Fix)

### 1. [Warning Title]
**Location:** [Section]
**Concern:** [Description]
**Recommendation:** [Suggested action]

## Suggestions (Nice to Have)

### 1. [Suggestion Title]
**Opportunity:** [Description]
**Benefit:** [Why it helps]

## Validation Checklist

- [x] All required sections present
- [ ] Business rules have concrete values
- [x] Test naming follows convention
- [ ] API endpoints are RESTful
- [x] Domain events defined for state changes
- [x] Dependencies correctly listed
- [ ] Exit criteria are measurable

## Auto-Fixes Applied

The following changes were made automatically:
1. [Change 1]
2. [Change 2]

## Manual Fixes Required

Please update the following:
1. [Manual fix needed]
```

---

## Auto-Fix Rules

The skill can automatically fix:

| Issue | Auto-Fix |
|-------|----------|
| Missing section headers | Add placeholder with instructions |
| Inconsistent formatting | Normalize markdown tables |
| Missing test count totals | Calculate from inventory |
| Missing Exit Signal | Generate from inventory |
| Incorrect phase assignment | Recalculate from dependencies |

---

## Common Issues Checklist

### Business Rules
- [ ] No "TBD" placeholders
- [ ] All rules have implementation references
- [ ] All constraints have error definitions
- [ ] Values use correct units (cents, basis points)

### Tests
- [ ] Follow naming convention
- [ ] Cover all business rules
- [ ] Include happy path AND error cases
- [ ] Domain events tested

### API
- [ ] RESTful paths (no verbs)
- [ ] Auth specified for all endpoints
- [ ] Error responses documented
- [ ] Request/response schemas defined

### Domain
- [ ] Clear aggregate boundaries
- [ ] Domain events for state changes
- [ ] Value objects identified
- [ ] No database types in domain

### Structure
- [ ] Correct phase assignment
- [ ] Dependencies listed
- [ ] Exit criteria measurable
- [ ] Exit signal defined

---

## Example Refinement Session

```
User: /module-refine waitlist

Claude: 📋 Analyzing docs/modules/waitlist.md...

# Module Refinement Report: Waitlist

## Summary
- **Issues Found:** 2
- **Warnings:** 3
- **Suggestions:** 1

## Issues (Must Fix)

### 1. Missing Error HTTP Mapping
**Location:** API Endpoints section
**Problem:** ErrPromotionExpired not mapped to HTTP status
**Fix:** Add `| WAITLIST_PROMOTION_EXPIRED | 410 |` to error table

### 2. Test Count Mismatch
**Location:** Test Inventory
**Problem:** Summary says 32 tests but only 30 listed
**Fix:** Add missing tests or correct summary

## Warnings (Should Fix)

### 1. No Query Handler Tests
**Location:** Application Layer Tests
**Problem:** GetWaitlistPosition query has no tests
**Recommendation:** Add TestGetWaitlistPositionQuery_* tests

### 2. Missing Domain Event Test
**Location:** Domain Layer Tests
**Problem:** WaitlistExpired event emission not tested
**Recommendation:** Add TestEvent_ExpirePromotions_EmitsWaitlistExpiredEvent

### 3. Builder Missing Expired State
**Location:** Test Builder Pattern
**Problem:** No way to create expired entry easily
**Recommendation:** Add `.Expired()` method to builder

## Suggestions (Nice to Have)

### 1. Add Scheduled Job Documentation
**Opportunity:** ExpirePromotionsHandler is a scheduled job
**Benefit:** Document cron schedule for operational clarity

## Auto-Fixes Applied
1. Added Exit Signal template
2. Corrected phase from 3 to 2 (depends only on Foundation, Events)

Shall I apply the manual fixes? (y/n)
```

---

## Integration with Other Skills

### Workflow
```
/feature-brief → features/waitlist.md
     ↓
/module-spec features/waitlist.md → docs/modules/waitlist.md
     ↓
/module-refine docs/modules/waitlist.md → Validated spec
     ↓
/module-checklist docs/modules/waitlist.md → REQUIREMENTS/CHECKLIST-waitlist.md
```

### When to Use
- After creating a new module spec with `/module-spec`
- Before generating checklist with `/module-checklist`
- When updating an existing module spec
- During code review of specification PRs

---

## See Also

- `/module-spec` - Create module specification
- `/module-checklist` - Generate tracking checklist
- `/feature-brief` - Create lightweight feature file
- `/hexagonal-design` - Design full system architecture

---

*Version: 1.0.0*
*Created: 2026-01-07*
