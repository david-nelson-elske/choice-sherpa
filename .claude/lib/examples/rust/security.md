# Rust Security Patterns

## Input Validation

```rust
// Validate at boundaries, not internally
pub fn create_user(input: CreateUserInput) -> Result<User, ValidationError> {
    // Validate all input before processing
    let email = Email::try_from(input.email)?;
    let name = Name::try_from(input.name)?;

    Ok(User::new(email, name))
}

// Use newtypes to enforce validation
pub struct Email(String);

impl TryFrom<String> for Email {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ValidationError::EmptyEmail);
        }
        if !value.contains('@') {
            return Err(ValidationError::InvalidEmailFormat);
        }
        Ok(Email(value))
    }
}
```

## SQL Injection Prevention

```rust
// Good: parameterized queries with sqlx
sqlx::query_as!(
    User,
    "SELECT * FROM users WHERE email = $1",
    email  // Parameterized, safe
)
.fetch_one(&pool)
.await?;

// Bad: string interpolation (NEVER DO THIS)
// format!("SELECT * FROM users WHERE email = '{}'", email)
```

## Authorization Checks

```rust
// Use trait-based ownership verification
pub trait OwnedByUser {
    fn owner_id(&self) -> UserId;

    fn is_owned_by(&self, user_id: UserId) -> bool {
        self.owner_id() == user_id
    }
}

// Check before operations
pub fn update_session(
    &self,
    session_id: SessionId,
    user_id: UserId,
    updates: SessionUpdates,
) -> Result<Session, SessionError> {
    let session = self.repo.get(session_id)?;

    if !session.is_owned_by(user_id) {
        return Err(SessionError::AccessDenied);
    }

    // Proceed with update...
}
```

## Sensitive Data Handling

```rust
use secrecy::{Secret, ExposeSecret};

pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,  // Won't be logged accidentally
}

impl Credentials {
    pub fn verify(&self, input: &str) -> bool {
        // Only expose when absolutely necessary
        self.password.expose_secret() == input
    }
}

// Debug won't leak secrets
impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .finish()
    }
}
```

## Rate Limiting Pattern

```rust
use governor::{Quota, RateLimiter};

pub struct ApiGateway {
    limiter: RateLimiter<...>,
}

impl ApiGateway {
    pub async fn check_rate_limit(&self, key: &str) -> Result<(), RateLimitError> {
        self.limiter
            .check_key(&key)
            .map_err(|_| RateLimitError::TooManyRequests)
    }
}
```

## Audit Logging

```rust
// Log security-relevant events
tracing::info!(
    user_id = %user_id,
    resource = "session",
    action = "access_denied",
    reason = "not owner",
    "Authorization failure"
);

// Never log sensitive data
tracing::info!(
    user_id = %user_id,
    email = %email,  // OK if not sensitive
    // password = ...  // NEVER
    "User login attempt"
);
```

## CSRF Protection

```rust
// Validate CSRF token in handlers
pub async fn handle_form(
    State(state): State<AppState>,
    csrf: CsrfToken,
    Form(input): Form<FormInput>,
) -> Result<Response, AppError> {
    csrf.verify(&input.csrf_token)?;
    // Process form...
}
```

## Security Headers (Axum)

```rust
use tower_http::set_header::SetResponseHeaderLayer;

Router::new()
    .layer(SetResponseHeaderLayer::if_not_present(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    ))
    .layer(SetResponseHeaderLayer::if_not_present(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    ));
```
