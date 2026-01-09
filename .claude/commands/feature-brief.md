# Feature Brief

> **Purpose**: Quickly capture feature intent within the established architecture.
> **Output**: `features/<module>/<name>.md` - A minimal feature file ready for TDD execution.
> **Time**: ~30 seconds to produce

---

## Usage

```
/feature-brief <name> [description]
/feature-brief waitlist "Allow users to join queue when events full"
/feature-brief --module events waitlist
/feature-brief                              # Interactive mode
```

### Arguments
- `name`: Feature identifier (kebab-case, becomes filename)
- `description`: Optional one-line description
- `--module <name>`: Target module (auto-detected from context if omitted)
- `--architecture <path>`: Architecture file (defaults to `docs/architecture/SYSTEM-ARCHITECTURE.md`)

---

## Architecture Integration

### Automatic Detection

The skill automatically:
1. Locates system architecture at `docs/architecture/SYSTEM-ARCHITECTURE.md`
2. Validates the target module exists in architecture
3. Pulls phase and dependency information
4. Places file in correct directory (`features/<module>/<name>.md`)

### Architecture Reference

Every feature brief includes:

```markdown
**Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md
**Module:** Events
**Phase:** 2
**Module Dependencies:** Foundation
```

### Validation Rules

| Rule | Check | Error |
|------|-------|-------|
| Module exists | Module in architecture inventory | "Module 'xyz' not found in architecture" |
| Phase order | Dependencies are lower phase | "Cannot depend on Phase 3 from Phase 2 module" |
| No cycles | Feature doesn't create circular dep | "Circular dependency detected" |

---

## Output Format

Creates `features/<module>/<name>.md`:

```markdown
# Feature: [Title]

**Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md
**Module:** [Module Name]
**Phase:** [1-4]
**Module Dependencies:** [From architecture]
**Feature Dependencies:** [Other features this requires]

> [One-line description of what this feature accomplishes]

---

## Context

- [Technical constraint or requirement]
- [Business rule that applies]
- [Integration dependency]
- [Security consideration]

---

## Tasks

- [ ] [Specific implementation task 1]
- [ ] [Specific implementation task 2]
- [ ] [Specific implementation task 3]

---

## Acceptance Criteria

- [ ] [Testable criterion 1]
- [ ] [Testable criterion 2]
- [ ] [Testable criterion 3]

---

## Files Affected

### New Files
- `backend/internal/domain/<module>/<file>.go`

### Modified Files
- `backend/internal/domain/<module>/<existing>.go`

---

## Related

- **Module Spec:** docs/modules/<module>.md
- **Checklist:** REQUIREMENTS/CHECKLIST-<module>.md
- **Related Features:** [links to dependent features]
```

---

## Gathering Information

When invoked, extract or ask for:

### 1. Module Context
- Which module does this feature belong to?
- Auto-detect from: current directory, recent work, explicit flag

### 2. Feature Name & Description
- What is this feature called?
- What does it accomplish in one sentence?

### 3. Context (Constraints)
Ask: "What constraints or requirements apply?"
- Technical: "Must use existing User model", "PostgreSQL only"
- Business: "Members get 10% discount", "24-hour lead time"
- Integration: "Uses Stripe API", "Sends email via SendGrid"
- Security: "Requires authentication", "Admin only"

### 4. Tasks (Implementation Steps)
Ask: "What needs to be built?"
- Each task should be completable in one TDD cycle
- Tasks should be ordered by dependency
- Use action verbs: "Create", "Add", "Implement", "Update"

### 5. Feature Dependencies
Ask: "Does this depend on other features?"
- Link to features that must be complete first
- Validate dependencies exist

### 6. Files Affected
Ask: "What files will be created or modified?"
- Helps scope the change
- Identifies potential conflicts

---

## Task Granularity Guidelines

### Good Task Size (One TDD Cycle)
```markdown
- [ ] Create Money value object with validation
- [ ] Add Register method to Event aggregate
- [ ] Create POST /api/events endpoint
```

### Too Large (Break Down)
```markdown
# BAD: Too big
- [ ] Implement event registration system

# GOOD: Broken down
- [ ] Create Registration entity
- [ ] Add Register method to Event
- [ ] Create RegisterForEvent command handler
- [ ] Create POST /api/events/:id/register endpoint
```

### Too Small (Combine)
```markdown
# BAD: Too granular
- [ ] Create id field
- [ ] Create title field
- [ ] Create description field

# GOOD: Logical unit
- [ ] Create Event aggregate with core fields
```

---

## Examples

### Example 1: Single-Module Feature

```markdown
# Feature: Event Waitlist

**Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md
**Module:** Events
**Phase:** 2
**Module Dependencies:** Foundation
**Feature Dependencies:** None

> Allow users to join a waitlist when events reach capacity.

---

## Context

- Builds on existing Event aggregate
- Auto-promote when spot opens (cancellation)
- Notify user via email when promoted
- Max waitlist size = 2x event capacity
- 24-hour confirmation window after promotion

---

## Tasks

- [ ] Add WaitlistEntry entity to Event aggregate
- [ ] Add JoinWaitlist method to Event
- [ ] Add LeaveWaitlist method to Event
- [ ] Add PromoteFromWaitlist method to Event
- [ ] Create WaitlistJoined domain event
- [ ] Create WaitlistPromoted domain event
- [ ] Create JoinWaitlist command handler
- [ ] Create POST /api/events/:id/waitlist endpoint
- [ ] Add event handler for registration cancellation

---

## Acceptance Criteria

- [ ] Can join waitlist when event is full
- [ ] Cannot join waitlist when waitlist is full (2x capacity)
- [ ] Cannot join waitlist if already registered
- [ ] Cannot join waitlist if already on waitlist
- [ ] Promotion happens in FIFO order
- [ ] Promoted user receives email notification
- [ ] Promoted user has 24 hours to confirm
- [ ] Expired promotions free up the spot

---

## Files Affected

### New Files
- `backend/internal/domain/events/waitlist.go`
- `backend/internal/domain/events/waitlist_test.go`
- `backend/internal/application/commands/join_waitlist.go`
- `backend/internal/application/commands/join_waitlist_test.go`
- `backend/internal/adapters/http/events/waitlist_handlers.go`

### Modified Files
- `backend/internal/domain/events/event.go` (add waitlist field)
- `backend/internal/ports/event_repository.go` (add waitlist methods)
- `backend/internal/adapters/postgres/event_repository.go`

---

## Related

- **Module Spec:** docs/modules/events.md
- **Checklist:** REQUIREMENTS/CHECKLIST-events.md
```

### Example 2: Feature with Dependencies

```markdown
# Feature: Member Discount on Events

**Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md
**Module:** Events
**Phase:** 2
**Module Dependencies:** Foundation, Memberships
**Feature Dependencies:**
- features/memberships/membership-tiers.md

> Apply membership-based discounts to event ticket prices.

---

## Context

- Members get tiered discounts (Bronze 5%, Silver 10%, Gold 15%)
- Discount applied at checkout, not display
- Must verify active membership status
- Discount shows in cart summary

---

## Tasks

- [ ] Add GetDiscountRate method to Event for member tier
- [ ] Update EventTicket.CalculatePrice to accept optional member tier
- [ ] Create GetMemberDiscount query handler
- [ ] Update cart line item display to show discount

---

## Acceptance Criteria

- [ ] Non-members see full price
- [ ] Bronze members see 5% discount in cart
- [ ] Silver members see 10% discount in cart
- [ ] Gold members see 15% discount in cart
- [ ] Expired memberships get no discount
- [ ] Discount calculated on subtotal before fees

---

## Files Affected

### Modified Files
- `backend/internal/domain/events/event_ticket.go`
- `backend/internal/domain/events/event_ticket_test.go`
- `frontend/src/features/cart/components/CartSummary.tsx`

---

## Related

- **Module Spec:** docs/modules/events.md
- **Depends On:** features/memberships/membership-tiers.md
```

---

## Cross-Module Features

If a feature touches multiple modules significantly, use `/integration-spec` instead:

```markdown
# Signs you need /integration-spec:
- Feature modifies 3+ modules
- Feature requires coordination between modules
- Feature has complex failure modes across boundaries
- Feature introduces new shared types

# Use /feature-brief when:
- Feature primarily lives in one module
- Other modules are only queried (read-only)
- Changes to other modules are minimal
```

---

## File Placement

| Feature Type | Location |
|--------------|----------|
| Module feature | `features/<module>/<name>.md` |
| Cross-module | `features/integrations/<name>.md` (use /integration-spec) |
| Bug fix | `features/<module>/fixes/<name>.md` |
| Refactoring | `features/<module>/refactor/<name>.md` |

---

## Validation Checklist

Before saving, verify:
- [ ] Architecture file exists and is referenced
- [ ] Module exists in architecture
- [ ] Phase is correct for module
- [ ] Dependencies are valid (no forward phase refs)
- [ ] Feature dependencies exist (if declared)
- [ ] Name is kebab-case
- [ ] Description is one clear sentence
- [ ] Context items are specific (not vague)
- [ ] Tasks are actionable and ordered
- [ ] Tasks are appropriately sized
- [ ] Acceptance criteria are testable
- [ ] Files affected are listed

---

## Error Handling

| Issue | Resolution |
|-------|------------|
| No architecture found | Create with `/hexagonal-design` first |
| Module not in architecture | Add module or check spelling |
| Invalid dependency | Fix phase order or remove dependency |
| Feature dependency not found | Create dependency feature first |
| File already exists | Ask to overwrite or rename |

---

## Workflow Integration

### From Architecture to Feature
```
/hexagonal-design → docs/architecture/SYSTEM-ARCHITECTURE.md
     ↓
/feature-brief --module events waitlist
     ↓
features/events/waitlist.md
     ↓
/architecture-validate features/events/waitlist.md
     ↓
/module-spec features/events/waitlist.md (if complex)
     ↓
/dev features/events/waitlist.md
```

### Upgrade Path
```
features/events/waitlist.md (brief)
     ↓ (if 10+ tasks or complex)
/module-spec features/events/waitlist.md
     ↓
docs/modules/waitlist.md (full spec)
     ↓
/module-checklist waitlist
     ↓
REQUIREMENTS/CHECKLIST-waitlist.md
```

---

## See Also

- `/hexagonal-design` - Create system architecture
- `/integration-spec` - Cross-module feature specification
- `/architecture-validate` - Validate against architecture
- `/module-spec` - Upgrade brief to full specification
- `/dev` - Execute feature file with TDD

---

*Version: 2.0.0*
*Created: 2026-01-07*
*Updated: 2026-01-07*
