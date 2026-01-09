# TDD GREEN Phase - Make Test Pass

Write the MINIMUM code necessary to make the failing test pass.

## Usage
```
/tdd-green
```

This phase follows `/tdd-red`. A failing test must exist.

---

## The Golden Rule

> Write the simplest code that makes the test pass. Nothing more.

---

## Process

### 1. Understand the Failing Test

Before writing code, confirm:
- What behavior is being tested?
- What is the expected output?
- What inputs are provided?

### 2. Write Minimal Implementation

**Go:**
```go
// Test expects: ErrEmailAlreadyExists when email is taken

func (s *UserService) Register(email, password string) (*User, error) {
    existing, _ := s.repo.FindByEmail(email)
    if existing != nil {
        return nil, ErrEmailAlreadyExists
    }
    // Minimal: just enough to pass
    return &User{Email: email}, nil
}
```

**TypeScript:**
```typescript
// Test expects: error when email already exists

async register(email: string, password: string): Promise<Result<User>> {
  const existing = await this.repo.findByEmail(email);
  if (existing) {
    return { error: 'EMAIL_ALREADY_EXISTS' };
  }
  // Minimal implementation
  return { data: { id: crypto.randomUUID(), email } };
}
```

**Python:**
```python
# Test expects: raises EmailExistsError when email is taken

def register(self, email: str, password: str) -> User:
    existing = self.repo.find_by_email(email)
    if existing:
        raise EmailExistsError(email)
    # Minimal implementation
    return User(email=email)
```

### 3. Run Test (Confirm GREEN)

```bash
# Go
go test -run "TestUser_Register_WithExistingEmail" ./...

# TypeScript
npx vitest run -t "should return error when email already exists"

# Python
pytest -k "test_user_service_register_with_existing_email"
```

Expected output:
```
PASS
✅ TestUser_Register_WithExistingEmail_ReturnsError
```

### 4. Run ALL Related Tests

Ensure no regressions:
```bash
# Run all tests in the module/package
go test ./internal/user/...
npx vitest run src/services/user
pytest tests/unit/user/
```

---

## Common GREEN Patterns

### Pattern: Return Expected Value
```go
// Test expects sum of two Money values
func (m Money) Add(other Money) Money {
    return Money{cents: m.cents + other.cents}
}
```

### Pattern: Return Error for Invalid State
```go
// Test expects error when event is full
func (e *Event) Register(userID string) (*Registration, error) {
    if len(e.registrations) >= e.capacity {
        return nil, ErrEventFull
    }
    return &Registration{UserID: userID}, nil
}
```

### Pattern: Simple State Change
```typescript
// Test expects item added to cart
addItem(item: CartItem): void {
  this.items.push(item);
}
```

### Pattern: Delegate to Dependency
```python
# Test expects user to be saved
def create_user(self, data: UserData) -> User:
    user = User(**data)
    return self.repo.save(user)
```

---

## What NOT to Do

### 1. Don't Optimize
```go
// ❌ Bad: Premature optimization
func (s *Service) GetUsers() []User {
    // Adding caching before tests require it
    if cached := s.cache.Get("users"); cached != nil {
        return cached
    }
    users := s.repo.FindAll()
    s.cache.Set("users", users)
    return users
}

// ✅ Good: Just what the test needs
func (s *Service) GetUsers() []User {
    return s.repo.FindAll()
}
```

### 2. Don't Add Extra Features
```typescript
// ❌ Bad: Features not required by test
register(email: string, password: string) {
  this.validateEmail(email);        // Not tested yet
  this.validatePassword(password);  // Not tested yet
  this.sendWelcomeEmail(email);     // Not tested yet
  return this.repo.create({ email, password });
}

// ✅ Good: Only what test requires
register(email: string, password: string) {
  return this.repo.create({ email, password });
}
```

### 3. Don't Refactor Yet
```python
# ❌ Bad: Refactoring in GREEN phase
def calculate_total(self, items):
    subtotal = self._calculate_subtotal(items)  # Extracted method
    tax = self._calculate_tax(subtotal)          # Extracted method
    return self._apply_discounts(subtotal + tax) # Extracted method

# ✅ Good: Inline, wait for REFACTOR phase
def calculate_total(self, items):
    subtotal = sum(item.price for item in items)
    tax = subtotal * 0.05
    return subtotal + tax
```

### 4. Don't Handle Hypothetical Errors
```go
// ❌ Bad: Error handling not required by tests
func (s *Service) GetUser(id string) (*User, error) {
    if id == "" {
        return nil, ErrInvalidID  // No test for this yet
    }
    user, err := s.repo.Find(id)
    if err != nil {
        s.logger.Error("failed", err)  // No test for this
        return nil, fmt.Errorf("get user: %w", err)
    }
    return user, nil
}

// ✅ Good: Just what tests cover
func (s *Service) GetUser(id string) (*User, error) {
    return s.repo.Find(id)
}
```

---

## Hardcoding is OK (Temporarily)

In GREEN phase, hardcoding to pass a test is valid:

```go
// Test: expects GetPrice() to return 1000 for "BASIC" plan
func (p *Plan) GetPrice() int {
    return 1000  // Hardcoded! Will generalize when more tests exist
}
```

The next test will force generalization:
```go
// New test: expects GetPrice() to return 2000 for "PRO" plan
// Now we must generalize
func (p *Plan) GetPrice() int {
    switch p.Type {
    case "BASIC":
        return 1000
    case "PRO":
        return 2000
    }
    return 0
}
```

---

## Checklist Before Moving to REFACTOR

- [ ] The specific test that was RED is now GREEN
- [ ] All other tests in the module still pass
- [ ] No compilation errors or warnings
- [ ] Implementation is minimal (no extra features)

---

## Output

After making the test pass:

```
🟢 GREEN: Test passing

File: src/services/user.service.ts
Implementation: Added register() method

Test results:
  ✅ should return error when email already exists
  ✅ 3 other tests still passing

Ready for REFACTOR phase: /tdd-refactor
```

---

## See Also

- `/tdd-red` - Previous phase: write failing test
- `/tdd-refactor` - Next phase: improve code quality
- `/tdd` - Complete TDD workflow
