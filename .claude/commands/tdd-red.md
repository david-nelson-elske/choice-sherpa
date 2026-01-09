# TDD RED Phase - Write Failing Test

Write a failing test that defines expected behavior BEFORE implementing.

## Usage
```
/tdd-red <behavior-description>
/tdd-red "user registration fails when email already exists"
```

## Arguments
- `behavior-description`: What behavior the test should verify

---

## Process

### 1. Identify Test Location

Determine where the test should live based on project structure.

Read from `CLAUDE.md` or detect from project:

| Layer | Common Locations |
|-------|-----------------|
| Domain/Model | `src/models/*.test.ts`, `*_test.go`, `tests/unit/` |
| Service | `src/services/*.test.ts`, `internal/service/*_test.go` |
| API/Handler | `src/api/*.test.ts`, `internal/handler/*_test.go` |
| Component | `src/components/*.test.tsx`, `__tests__/` |
| Utility | `src/utils/*.test.ts`, `pkg/*_test.go` |

### 2. Write the Test

Follow the AAA pattern (Arrange-Act-Assert):

**Go:**
```go
func Test<Type>_<Method>_<Scenario>_<Expected>(t *testing.T) {
    // Arrange: Set up test fixtures
    svc := NewUserService(mockRepo)
    existingUser := &User{Email: "taken@example.com"}
    mockRepo.On("FindByEmail", "taken@example.com").Return(existingUser, nil)

    // Act: Execute the behavior
    _, err := svc.Register("taken@example.com", "password123")

    // Assert: Verify expected outcome
    assert.ErrorIs(t, err, ErrEmailAlreadyExists)
}
```

**TypeScript:**
```typescript
describe('UserService', () => {
  it('should return error when email already exists', async () => {
    // Arrange
    const mockRepo = { findByEmail: vi.fn().mockResolvedValue({ id: '1' }) };
    const svc = new UserService(mockRepo);

    // Act
    const result = await svc.register('taken@example.com', 'password123');

    // Assert
    expect(result.error).toBe('EMAIL_ALREADY_EXISTS');
  });
});
```

**Python:**
```python
def test_user_service_register_with_existing_email_returns_error():
    # Arrange
    mock_repo = Mock()
    mock_repo.find_by_email.return_value = User(email="taken@example.com")
    svc = UserService(mock_repo)

    # Act
    result = svc.register("taken@example.com", "password123")

    # Assert
    assert result.error == "EMAIL_ALREADY_EXISTS"
```

### 3. Test Naming Convention

**Format:** `Test<Subject>_<Scenario>_<Expected>`

Good names:
```
TestUser_Register_WithExistingEmail_ReturnsError
TestMoney_Add_WithNegativeValue_ReturnsValidationError
TestCart_AddItem_WhenFull_RejectsItem
```

Bad names:
```
TestRegister          # What about register?
TestUserWorks         # What does "works" mean?
Test1                 # Meaningless
```

### 4. Run Test (Confirm RED)

The test MUST fail before proceeding:

```bash
# Go
go test -run "TestUser_Register_WithExistingEmail" ./...

# TypeScript/JavaScript
npx vitest run -t "should return error when email already exists"

# Python
pytest -k "test_user_service_register_with_existing_email"
```

### 5. Verify Failure Reason

The test should fail for the RIGHT reason:

| Failure Type | Action |
|-------------|--------|
| Missing function/method | ✅ Expected - proceed to GREEN |
| Wrong return value | ✅ Expected - proceed to GREEN |
| Compilation/syntax error | ❌ Fix the test first |
| Wrong assertion | ❌ Fix the test first |
| Test passes | ❌ Either behavior exists or test is wrong |

---

## Test Categories

### Happy Path Tests
Test that correct behavior works:
```go
func TestUser_Register_WithValidData_CreatesUser(t *testing.T) {
    // Valid input should succeed
}
```

### Error/Edge Case Tests
Test that invalid input is handled:
```go
func TestUser_Register_WithEmptyEmail_ReturnsValidationError(t *testing.T) {
    // Empty email should fail with specific error
}
```

### Boundary Tests
Test limits and boundaries:
```go
func TestCart_AddItem_AtMaxCapacity_RejectsItem(t *testing.T) {
    // Cart at 100 items should reject 101st
}
```

---

## What Makes a Good Test

### 1. Tests Behavior, Not Implementation
```go
// ❌ Bad: Tests internal details
assert.Equal(t, 2, len(user.validationCalls))

// ✅ Good: Tests observable behavior
assert.ErrorIs(t, err, ErrInvalidEmail)
```

### 2. One Behavior Per Test
```go
// ❌ Bad: Multiple behaviors
func TestUser_Everything(t *testing.T) {
    // Tests validation AND creation AND notification
}

// ✅ Good: Focused tests
func TestUser_Register_WithValidData_CreatesUser(t *testing.T) {}
func TestUser_Register_WithValidData_SendsWelcomeEmail(t *testing.T) {}
```

### 3. Descriptive Failure Messages
```go
// ❌ Bad: Unclear failure
assert.True(t, result)

// ✅ Good: Clear failure message
assert.Equal(t, expected, actual, "user email should match input")
```

### 4. Independent Tests
```go
// ❌ Bad: Depends on other tests
func TestB(t *testing.T) {
    // Assumes TestA ran first and set up data
}

// ✅ Good: Self-contained
func TestB(t *testing.T) {
    // Sets up its own data
    user := createTestUser(t)
}
```

---

## Common Patterns

### Testing Errors
```go
// Go
assert.ErrorIs(t, err, ErrNotFound)
assert.ErrorContains(t, err, "not found")

// TypeScript
expect(result.error).toBe('NOT_FOUND');
expect(() => fn()).toThrow('not found');

// Python
with pytest.raises(NotFoundError):
    service.find(invalid_id)
```

### Testing Async Code
```typescript
// TypeScript
it('should fetch user', async () => {
    const user = await service.getUser('123');
    expect(user.id).toBe('123');
});
```

### Testing Events/Side Effects
```go
func TestOrder_Complete_EmitsOrderCompletedEvent(t *testing.T) {
    order := createTestOrder(t)

    order.Complete()

    events := order.PullEvents()
    require.Len(t, events, 1)
    assert.IsType(t, OrderCompleted{}, events[0])
}
```

---

## Output

After writing the failing test:

```
🔴 RED: Test written and failing

File: src/services/user.service.test.ts
Test: should return error when email already exists

Failure reason: ✅ Method 'register' not implemented
  Expected: ErrEmailAlreadyExists
  Actual:   TypeError: svc.register is not a function

Ready for GREEN phase: /tdd-green
```

---

## See Also

- `/tdd-green` - Next phase: implement minimal solution
- `/tdd` - Complete TDD workflow
- `/test` - Run tests
