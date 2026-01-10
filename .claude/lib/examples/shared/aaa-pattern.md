# AAA Pattern (Arrange-Act-Assert)

## Structure

Every test follows three distinct phases:

```
┌─────────────────────────────────────────┐
│ ARRANGE                                 │
│ Set up test data, fixtures, mocks       │
├─────────────────────────────────────────┤
│ ACT                                     │
│ Execute the behavior being tested       │
├─────────────────────────────────────────┤
│ ASSERT                                  │
│ Verify expected outcome                 │
└─────────────────────────────────────────┘
```

## Arrange

**Purpose:** Set up preconditions and inputs.

What goes here:
- Create test objects
- Configure mocks
- Set up initial state
- Prepare input data

```
// Arrange
const user = createTestUser();
const mockRepo = createMockRepository();
mockRepo.findById.mockResolvedValue(user);
const service = new UserService(mockRepo);
```

## Act

**Purpose:** Execute the single behavior being tested.

Rules:
- Usually ONE line of code
- Call the method/function being tested
- Capture result if needed

```
// Act
const result = await service.getUser(userId);
```

## Assert

**Purpose:** Verify expected outcome.

What goes here:
- Check return values
- Verify state changes
- Confirm mock interactions (sparingly)

```
// Assert
expect(result).toBeDefined();
expect(result.id).toBe(userId);
expect(result.email).toBe('test@example.com');
```

## Complete Examples

### Rust

```rust
#[test]
fn test_user_validate_with_valid_email_succeeds() {
    // Arrange
    let user = User::new("valid@example.com");

    // Act
    let result = user.validate();

    // Assert
    assert!(result.is_ok());
}
```

### TypeScript

```typescript
it('should return user when found', async () => {
  // Arrange
  const userId = '123';
  const expectedUser = { id: userId, email: 'test@example.com' };
  mockRepo.findById.mockResolvedValue(expectedUser);
  const service = new UserService(mockRepo);

  // Act
  const result = await service.getUser(userId);

  // Assert
  expect(result).toEqual(expectedUser);
});
```

## Guidelines

### Keep Arrange Focused
- Only set up what the test needs
- Use factory functions for complex objects
- Don't duplicate setup across tests (use beforeEach)

### Keep Act Simple
- Should be one line when possible
- If it takes multiple lines, consider if you're testing too much

### Keep Assert Clear
- Test one logical concept per test
- Multiple assertions OK if testing same thing
- Avoid asserting on mocks unless behavior depends on it

## Anti-Patterns

| Anti-Pattern | Problem | Fix |
|--------------|---------|-----|
| Act in Arrange | Setup does work | Move to Act section |
| Assert in Arrange | Verifying setup | Remove or make explicit |
| Multiple Acts | Testing too much | Split into separate tests |
| No clear sections | Hard to read | Add comments or blank lines |

## Comments Optional

If code is clear, comments are optional:

```rust
#[test]
fn test_money_add_returns_sum() {
    let a = Money::cents(100);
    let b = Money::cents(200);

    let sum = a.add(b);

    assert_eq!(sum.cents(), 300);
}
```

Use comments when sections aren't obvious:

```rust
#[test]
fn test_complex_workflow() {
    // Arrange: Set up user with expired subscription
    let user = User::new(...);
    user.subscription = Subscription::expired();
    let mock_notifier = MockNotifier::new();
    let service = RenewalService::new(mock_notifier.clone());

    // Act
    service.process_renewal(user);

    // Assert
    assert!(mock_notifier.was_called_with("renewal_reminder"));
}
```
