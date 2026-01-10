# Rust Error Handling Patterns

## Result Type

```rust
// Function returning Result
fn validate_email(email: &str) -> Result<Email, ValidationError> {
    if email.is_empty() {
        return Err(ValidationError::Empty);
    }
    if !email.contains('@') {
        return Err(ValidationError::InvalidFormat);
    }
    Ok(Email(email.to_string()))
}
```

## The ? Operator

```rust
// Propagate errors with ?
fn create_user(email: &str, name: &str) -> Result<User, UserError> {
    let validated_email = validate_email(email)?;
    let validated_name = validate_name(name)?;
    Ok(User::new(validated_email, validated_name))
}
```

## Custom Error Types

```rust
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum CycleError {
    #[error("Cycle not found: {0}")]
    NotFound(CycleId),

    #[error("Cannot start component: cycle not in progress")]
    NotInProgress,

    #[error("Component {0} already completed")]
    AlreadyCompleted(ComponentType),

    #[error("Invalid state transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },
}
```

## Domain-Specific Errors

```rust
// Per-module error types
pub enum SessionError {
    NotFound(SessionId),
    AccessDenied(UserId),
    AlreadyArchived,
}

pub enum MembershipError {
    SubscriptionRequired,
    TierInsufficientForFeature(String),
    PaymentFailed(String),
}
```

## Error Conversion

```rust
impl From<RepositoryError> for ServiceError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound(id) => ServiceError::NotFound(id),
            RepositoryError::Connection(e) => ServiceError::Internal(e.to_string()),
        }
    }
}
```

## Option vs Result

```rust
// Use Option when absence is normal
fn find_by_id(&self, id: UserId) -> Option<User>;

// Use Result when absence is an error
fn get_by_id(&self, id: UserId) -> Result<User, UserError>;

// Convert Option to Result
fn get_user(&self, id: UserId) -> Result<User, UserError> {
    self.find_by_id(id).ok_or(UserError::NotFound(id))
}
```

## Handling Multiple Error Types

```rust
// With anyhow for application code
use anyhow::{Context, Result};

fn process() -> Result<()> {
    let data = read_file(path)
        .context("Failed to read configuration")?;
    let parsed = parse_config(&data)
        .context("Failed to parse configuration")?;
    Ok(())
}

// With thiserror for library code (prefer this in domain)
```

## Never Use unwrap() in Production

```rust
// Bad: panics on None/Err
let value = maybe_value.unwrap();

// Good: handle the case
let value = maybe_value.ok_or(MyError::Missing)?;

// Good: provide default
let value = maybe_value.unwrap_or_default();

// Good: with context
let value = maybe_value.expect("invariant: value set in constructor");
```

## Validation Pattern

```rust
pub fn validate(&self) -> Result<(), ValidationError> {
    if self.name.is_empty() {
        return Err(ValidationError::EmptyName);
    }
    if self.email.is_empty() {
        return Err(ValidationError::EmptyEmail);
    }
    Ok(())
}

// Collect all errors
pub fn validate_all(&self) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    if self.name.is_empty() {
        errors.push(ValidationError::EmptyName);
    }
    if self.email.is_empty() {
        errors.push(ValidationError::EmptyEmail);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```
