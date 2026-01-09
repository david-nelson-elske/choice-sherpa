# TDD REFACTOR Phase - Improve Code Quality

With all tests passing, improve code quality without changing behavior.

## Usage
```
/tdd-refactor
```

This phase follows `/tdd-green`. All tests must be passing.

---

## The Refactoring Rule

> Change structure, not behavior. Tests must stay GREEN.

---

## Process

### 1. Verify Tests Are GREEN

Before refactoring, confirm all tests pass:

```bash
# Go
go test ./...

# TypeScript
npm test

# Python
pytest
```

**Never refactor with failing tests.**

### 2. Identify Refactoring Opportunities

Look for code smells:

| Smell | Symptom | Refactoring |
|-------|---------|-------------|
| Primitive Obsession | `int` for money, `string` for IDs | Extract Value Object |
| Long Method | Method > 20 lines | Extract Method |
| Duplicate Code | Same logic repeated | Extract Function |
| Magic Numbers | Hardcoded `0.05`, `100` | Extract Constants |
| Long Parameter List | > 3 parameters | Parameter Object |
| Feature Envy | Uses other object's data heavily | Move Method |
| Large Class | Too many responsibilities | Extract Class |

### 3. Refactor in Small Steps

Make ONE small change at a time:

```
Change → Run Tests → GREEN? → Next Change
                  ↓
                RED? → Undo → Try Different Approach
```

### 4. Run Tests After EACH Change

```bash
# Quick feedback - run focused tests
go test -run "TestUser" ./...
npx vitest run src/services/user
```

---

## DRY - Don't Repeat Yourself

> "Every piece of knowledge must have a single, unambiguous, authoritative representation within a system."

During refactoring, actively look for DRY violations:

### Check for Duplication

| Duplication Type | Symptom | Action |
|-----------------|---------|--------|
| **Code clones** | Same 3+ lines appear twice | Extract function/method |
| **Structural** | Same pattern with different types | Extract generic/trait/macro |
| **Knowledge** | Same business rule in multiple places | Single source of truth |
| **Data** | Same constant defined twice | Extract to shared location |

### Use Existing Project Abstractions

Before writing new code, check if these project patterns apply:

| Pattern | Use When | Location |
|---------|----------|----------|
| `declare_uuid_id!` | Adding new ID types | `foundation/ids.rs` |
| `domain_event!` | Implementing DomainEvent trait | `foundation/events.rs` |
| `delegate_to_variant!` | Match blocks over ComponentVariant | `proact/macros.rs` |
| `StateMachine` trait | Status enums with transitions | `foundation/state_machine.rs` |
| `Repository<T, ID>` | New repository interfaces | `foundation/repository.rs` |
| `OwnedByUser` trait | Entities with user ownership | `foundation/ownership.rs` |

### DRY Checklist

Before finishing refactor:
- [ ] No copy-pasted code blocks (3+ lines)
- [ ] No repeated match patterns that could use macros
- [ ] Business rules defined in one place only
- [ ] Using existing abstractions where applicable
- [ ] New patterns documented if reusable

> **Reference:** See `docs/architecture/DRY-ANALYSIS-REPORT.md` for full pattern inventory

---

## Common Refactoring Patterns

### Extract Value Object

```go
// Before: Primitive obsession
type Order struct {
    totalCents int
    currency   string
}

// After: Value object
type Order struct {
    total Money
}

type Money struct {
    cents    int
    currency string
}

func (m Money) Add(other Money) Money {
    return Money{cents: m.cents + other.cents, currency: m.currency}
}
```

### Extract Method

```typescript
// Before: Long method
async register(email: string, password: string) {
  // Validation (10 lines)
  if (!email) throw new Error('Email required');
  if (!email.includes('@')) throw new Error('Invalid email');
  if (password.length < 8) throw new Error('Password too short');
  // ... more validation

  // Creation (10 lines)
  const hash = await bcrypt.hash(password, 10);
  const user = await this.repo.create({ email, passwordHash: hash });

  // Notification (5 lines)
  await this.emailService.sendWelcome(email);

  return user;
}

// After: Extracted methods
async register(email: string, password: string) {
  this.validateRegistration(email, password);
  const user = await this.createUser(email, password);
  await this.sendWelcomeEmail(user);
  return user;
}

private validateRegistration(email: string, password: string) {
  if (!email) throw new Error('Email required');
  if (!email.includes('@')) throw new Error('Invalid email');
  if (password.length < 8) throw new Error('Password too short');
}

private async createUser(email: string, password: string) {
  const hash = await bcrypt.hash(password, 10);
  return this.repo.create({ email, passwordHash: hash });
}

private async sendWelcomeEmail(user: User) {
  await this.emailService.sendWelcome(user.email);
}
```

### Extract Constants

```python
# Before: Magic numbers
def calculate_total(self, subtotal: int) -> int:
    tax = subtotal * 0.05
    if subtotal > 10000:
        discount = subtotal * 0.1
    else:
        discount = 0
    return subtotal + tax - discount

# After: Named constants
TAX_RATE = 0.05
DISCOUNT_RATE = 0.10
DISCOUNT_THRESHOLD_CENTS = 10000

def calculate_total(self, subtotal: int) -> int:
    tax = subtotal * TAX_RATE
    discount = self._calculate_discount(subtotal)
    return subtotal + tax - discount

def _calculate_discount(self, subtotal: int) -> int:
    if subtotal > DISCOUNT_THRESHOLD_CENTS:
        return int(subtotal * DISCOUNT_RATE)
    return 0
```

### Introduce Guard Clauses

```go
// Before: Nested conditionals
func (e *Event) Cancel(now time.Time) error {
    if e.status == StatusPublished {
        if now.Before(e.startTime) {
            e.status = StatusCancelled
            return nil
        } else {
            return ErrEventAlreadyStarted
        }
    } else {
        return ErrEventNotPublished
    }
}

// After: Guard clauses
func (e *Event) Cancel(now time.Time) error {
    if e.status != StatusPublished {
        return ErrEventNotPublished
    }
    if now.After(e.startTime) {
        return ErrEventAlreadyStarted
    }

    e.status = StatusCancelled
    return nil
}
```

### Remove Duplication

```typescript
// Before: Duplicated logic
function validateEmail(email: string): boolean {
  const regex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return regex.test(email);
}

function validateUserEmail(user: User): boolean {
  const regex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return regex.test(user.email);
}

// After: Shared function
const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

function isValidEmail(email: string): boolean {
  return EMAIL_REGEX.test(email);
}

function validateUserEmail(user: User): boolean {
  return isValidEmail(user.email);
}
```

---

## What NOT to Do

### 1. Don't Add Features
```go
// ❌ Bad: Adding new functionality
func (s *Service) GetUser(id string) (*User, error) {
    user, err := s.repo.Find(id)
    // Adding caching during refactor
    s.cache.Set(id, user)  // NEW FEATURE!
    return user, err
}

// ✅ Good: Only restructure
func (s *Service) GetUser(id string) (*User, error) {
    return s.findUser(id)  // Just renamed/moved
}
```

### 2. Don't Change Behavior
```python
# ❌ Bad: Changing logic
def calculate_tax(amount):
    return amount * 0.07  # Changed from 0.05!

# ✅ Good: Same behavior, better structure
TAX_RATE = 0.05

def calculate_tax(amount):
    return amount * TAX_RATE
```

### 3. Don't Make Large Changes
```typescript
// ❌ Bad: Complete rewrite
// Rewrote entire class structure

// ✅ Good: Incremental changes
// Step 1: Extract method A
// Step 2: Run tests
// Step 3: Extract method B
// Step 4: Run tests
```

---

## Refactoring Checklist

Before finishing refactor phase:

- [ ] All tests still pass
- [ ] No new functionality added
- [ ] Methods are < 20 lines
- [ ] **DRY:** No duplicate code (3+ lines repeated)
- [ ] **DRY:** Using project macros/traits where applicable
- [ ] Magic numbers replaced with constants
- [ ] Variable names are descriptive
- [ ] Complex conditionals simplified

---

## When to Stop Refactoring

Stop when:
- Code is "clean enough" to understand
- No obvious code smells remain
- You're tempted to add features (save for next RED)

> "Leave the code cleaner than you found it" - Boy Scout Rule

---

## Output

After refactoring:

```
🔵 REFACTOR: Code improved

Changes made:
- Extracted validateEmail() method
- Renamed 'x' to 'userCount' for clarity
- Replaced magic number 100 with MAX_USERS constant
- Simplified nested if statements with guard clauses
- DRY: Used existing declare_uuid_id! macro for new ID type

Tests: ✅ All passing (no regressions)
DRY: ✅ No violations detected

Cycle complete. Ready to:
- /commit - Commit this work
- /tdd-red <next-behavior> - Start next test
- /dev - Continue with feature
```

---

## See Also

- `/tdd-green` - Previous phase: minimal implementation
- `/tdd-red` - Start new cycle with failing test
- `/tdd` - Complete TDD workflow
- `/lint` - Additional code quality checks
