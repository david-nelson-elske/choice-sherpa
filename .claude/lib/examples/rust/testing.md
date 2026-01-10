# Rust Testing Patterns

## Test Naming Convention

```rust
#[test]
fn test_<subject>_<scenario>_<expected>() { }

// Examples:
fn test_user_validate_with_invalid_email_returns_error() { }
fn test_money_add_with_positive_values_returns_sum() { }
fn test_cycle_start_when_already_started_fails() { }
```

## AAA Pattern

```rust
#[test]
fn test_example() {
    // Arrange
    let user = User::new("test@example.com");

    // Act
    let result = user.validate();

    // Assert
    assert!(result.is_ok());
}
```

## Common Assertions

```rust
// Equality
assert_eq!(actual, expected);
assert_ne!(actual, unexpected);

// Boolean
assert!(condition);
assert!(!condition);

// Result types
assert!(result.is_ok());
assert!(result.is_err());

// Pattern matching
assert!(matches!(value, Pattern::Variant { .. }));
assert!(matches!(result, Err(MyError::NotFound)));

// With custom message
assert_eq!(actual, expected, "user email should match input");
```

## Testing Errors

```rust
#[test]
fn test_returns_error_on_invalid_input() {
    let result = validate("");

    assert!(result.is_err());
    assert!(matches!(result, Err(ValidationError::Empty)));
}

// Or using unwrap_err
#[test]
fn test_error_type() {
    let err = validate("").unwrap_err();
    assert_eq!(err, ValidationError::Empty);
}
```

## Testing Domain Events

```rust
#[test]
fn test_aggregate_emits_event() {
    // Arrange
    let mut aggregate = Aggregate::new(id);

    // Act
    aggregate.do_something();

    // Assert
    let events = aggregate.take_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(&events[0], DomainEvent::SomethingHappened { .. }));
}
```

## Mocking with mockall

```rust
use mockall::automock;

#[automock]
trait Repository {
    fn find(&self, id: Uuid) -> Option<Entity>;
    fn save(&self, entity: &Entity) -> Result<(), Error>;
}

#[test]
fn test_with_mock() {
    let mut mock = MockRepository::new();
    mock.expect_find()
        .with(eq(id))
        .returning(|_| Some(entity));

    let service = Service::new(mock);
    let result = service.get(id);

    assert!(result.is_some());
}
```

## Test Fixtures / Helpers

```rust
// In tests/common/mod.rs or as module
fn create_test_user() -> User {
    User::new(
        UserId::new(),
        "test@example.com".into(),
    )
}

fn create_test_cycle(session_id: SessionId) -> Cycle {
    Cycle::create(CycleId::new(), session_id, "Test cycle".into())
}
```

## Async Tests

```rust
#[tokio::test]
async fn test_async_operation() {
    let service = create_service().await;

    let result = service.fetch_data().await;

    assert!(result.is_ok());
}
```

## Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Group related tests
    mod validation {
        use super::*;

        #[test]
        fn test_valid_input_passes() { }

        #[test]
        fn test_empty_input_fails() { }
    }

    mod creation {
        use super::*;

        #[test]
        fn test_creates_with_defaults() { }
    }
}
```
