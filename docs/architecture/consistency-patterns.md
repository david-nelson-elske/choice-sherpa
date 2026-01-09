# Architecture: Consistency Patterns

**Type:** Cross-Cutting Standards
**Priority:** P0 (Required for implementation)
**Last Updated:** 2026-01-08

> Mandatory patterns and conventions ensuring consistency across all Choice Sherpa modules.

---

## Overview

This document defines the canonical patterns for:
1. Error handling and error types
2. Value object validation
3. Domain event naming and structure
4. API response formats
5. Code organization conventions
6. Testing conventions
7. Idempotency patterns (commands and event handlers)

All modules MUST follow these patterns. Deviations require explicit architectural review.

---

## 1. Error Handling Patterns

### Domain Error Structure

All modules use a unified `DomainError` type with standardized error codes.

```rust
// backend/src/domain/foundation/error.rs

use serde::{Deserialize, Serialize};
use std::fmt;

/// Canonical error type for all domain operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainError {
    /// Machine-readable error code
    pub code: ErrorCode,
    /// Human-readable message
    pub message: String,
    /// Optional field that caused the error
    pub field: Option<String>,
    /// Optional nested errors (for validation)
    pub errors: Option<Vec<DomainError>>,
}

impl DomainError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            field: None,
            errors: None,
        }
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    pub fn with_errors(mut self, errors: Vec<DomainError>) -> Self {
        self.errors = Some(errors);
        self
    }

    /// HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        self.code.status_code()
    }
}

impl std::error::Error for DomainError {}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
```

### Error Codes

```rust
/// Standardized error codes across all modules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Validation errors (400)
    ValidationFailed,
    InvalidFormat,
    InvalidValue,
    MissingRequired,
    OutOfRange,

    // Authentication errors (401)
    Unauthenticated,
    InvalidToken,
    TokenExpired,

    // Authorization errors (403)
    Unauthorized,
    AccessDenied,
    InsufficientPermissions,
    FeatureNotAvailable,

    // Not found errors (404)
    NotFound,
    SessionNotFound,
    CycleNotFound,
    ComponentNotFound,
    MembershipNotFound,
    UserNotFound,

    // Conflict errors (409)
    Conflict,
    AlreadyExists,
    ConcurrencyConflict,
    InvalidStateTransition,

    // Rate limiting (429)
    RateLimitExceeded,
    QuotaExceeded,

    // Internal errors (500)
    InternalError,
    DatabaseError,
    ExternalServiceError,
}

impl ErrorCode {
    pub fn status_code(&self) -> u16 {
        match self {
            // 400 Bad Request
            ErrorCode::ValidationFailed
            | ErrorCode::InvalidFormat
            | ErrorCode::InvalidValue
            | ErrorCode::MissingRequired
            | ErrorCode::OutOfRange => 400,

            // 401 Unauthorized
            ErrorCode::Unauthenticated
            | ErrorCode::InvalidToken
            | ErrorCode::TokenExpired => 401,

            // 403 Forbidden
            ErrorCode::Unauthorized
            | ErrorCode::AccessDenied
            | ErrorCode::InsufficientPermissions
            | ErrorCode::FeatureNotAvailable => 403,

            // 404 Not Found
            ErrorCode::NotFound
            | ErrorCode::SessionNotFound
            | ErrorCode::CycleNotFound
            | ErrorCode::ComponentNotFound
            | ErrorCode::MembershipNotFound
            | ErrorCode::UserNotFound => 404,

            // 409 Conflict
            ErrorCode::Conflict
            | ErrorCode::AlreadyExists
            | ErrorCode::ConcurrencyConflict
            | ErrorCode::InvalidStateTransition => 409,

            // 429 Too Many Requests
            ErrorCode::RateLimitExceeded
            | ErrorCode::QuotaExceeded => 429,

            // 500 Internal Server Error
            ErrorCode::InternalError
            | ErrorCode::DatabaseError
            | ErrorCode::ExternalServiceError => 500,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
```

### Error Creation Patterns

```rust
// ✅ Good: Specific error code with context
DomainError::new(ErrorCode::SessionNotFound, "Session does not exist")

// ✅ Good: Validation error with field
DomainError::new(ErrorCode::InvalidValue, "Title cannot be empty")
    .with_field("title")

// ✅ Good: Multiple validation errors
DomainError::new(ErrorCode::ValidationFailed, "Validation failed")
    .with_errors(vec![
        DomainError::new(ErrorCode::MissingRequired, "Title is required").with_field("title"),
        DomainError::new(ErrorCode::InvalidFormat, "Invalid email format").with_field("email"),
    ])

// ❌ Bad: Generic error without context
DomainError::new(ErrorCode::InternalError, "Something went wrong")

// ❌ Bad: Wrong error code for the situation
DomainError::new(ErrorCode::ValidationFailed, "User not found")  // Should be NotFound
```

### Result Type Alias

```rust
/// Standard Result type for domain operations
pub type DomainResult<T> = Result<T, DomainError>;

// Usage:
pub fn create_session(cmd: CreateSessionCommand) -> DomainResult<Session> {
    // ...
}
```

---

## 2. Value Object Validation

### Validation Pattern

All value objects validate on construction and are immutable.

```rust
/// Pattern: Validated value object
pub struct Title(String);

impl Title {
    /// Canonical constructor - MUST validate
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into().trim().to_string();

        if value.is_empty() {
            return Err(DomainError::new(
                ErrorCode::InvalidValue,
                "Title cannot be empty"
            ).with_field("title"));
        }

        if value.len() > 255 {
            return Err(DomainError::new(
                ErrorCode::OutOfRange,
                "Title cannot exceed 255 characters"
            ).with_field("title"));
        }

        Ok(Self(value))
    }

    /// Unchecked constructor - ONLY for trusted sources (DB, deserialization)
    pub fn from_trusted(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Title {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Title {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

### ID Value Objects

```rust
use uuid::Uuid;

/// Pattern: Type-safe ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Generate new ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Parse from string - validates format
    pub fn parse(s: &str) -> DomainResult<Self> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|_| DomainError::new(
                ErrorCode::InvalidFormat,
                format!("Invalid session ID format: {}", s)
            ))
    }

    /// From trusted source (DB)
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}
```

### Money Value Object

```rust
/// Pattern: Money in cents (never floats!)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    /// Amount in cents (e.g., 1999 = $19.99)
    cents: i64,
    /// ISO 4217 currency code
    currency: Currency,
}

impl Money {
    pub fn new(cents: i64, currency: Currency) -> DomainResult<Self> {
        if cents < 0 {
            return Err(DomainError::new(
                ErrorCode::InvalidValue,
                "Amount cannot be negative"
            ));
        }
        Ok(Self { cents, currency })
    }

    pub fn usd(cents: i64) -> DomainResult<Self> {
        Self::new(cents, Currency::USD)
    }

    pub fn cad(cents: i64) -> DomainResult<Self> {
        Self::new(cents, Currency::CAD)
    }

    pub fn cents(&self) -> i64 {
        self.cents
    }

    pub fn currency(&self) -> Currency {
        self.currency
    }

    /// Display as decimal (for UI only, not calculations!)
    pub fn to_decimal_string(&self) -> String {
        format!("{:.2}", self.cents as f64 / 100.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    USD,
    CAD,
}
```

### Enum Value Objects

```rust
/// Pattern: Exhaustive enum with serialization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    IssueRaising,
    ProblemFrame,
    Objectives,
    Alternatives,
    Consequences,
    Tradeoffs,
    Recommendation,
    DecisionQuality,
}

impl ComponentType {
    /// All component types in order
    pub const ALL: [ComponentType; 8] = [
        ComponentType::IssueRaising,
        ComponentType::ProblemFrame,
        ComponentType::Objectives,
        ComponentType::Alternatives,
        ComponentType::Consequences,
        ComponentType::Tradeoffs,
        ComponentType::Recommendation,
        ComponentType::DecisionQuality,
    ];

    /// Position in the PrOACT sequence (0-indexed)
    pub fn position(&self) -> usize {
        match self {
            ComponentType::IssueRaising => 0,
            ComponentType::ProblemFrame => 1,
            ComponentType::Objectives => 2,
            ComponentType::Alternatives => 3,
            ComponentType::Consequences => 4,
            ComponentType::Tradeoffs => 5,
            ComponentType::Recommendation => 6,
            ComponentType::DecisionQuality => 7,
        }
    }

    pub fn from_str(s: &str) -> DomainResult<Self> {
        match s.to_lowercase().as_str() {
            "issue_raising" | "issueraising" => Ok(ComponentType::IssueRaising),
            "problem_frame" | "problemframe" => Ok(ComponentType::ProblemFrame),
            "objectives" => Ok(ComponentType::Objectives),
            "alternatives" => Ok(ComponentType::Alternatives),
            "consequences" => Ok(ComponentType::Consequences),
            "tradeoffs" => Ok(ComponentType::Tradeoffs),
            "recommendation" => Ok(ComponentType::Recommendation),
            "decision_quality" | "decisionquality" => Ok(ComponentType::DecisionQuality),
            _ => Err(DomainError::new(
                ErrorCode::InvalidValue,
                format!("Unknown component type: {}", s)
            )),
        }
    }
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentType::IssueRaising => write!(f, "Issue Raising"),
            ComponentType::ProblemFrame => write!(f, "Problem Frame"),
            ComponentType::Objectives => write!(f, "Objectives"),
            ComponentType::Alternatives => write!(f, "Alternatives"),
            ComponentType::Consequences => write!(f, "Consequences"),
            ComponentType::Tradeoffs => write!(f, "Tradeoffs"),
            ComponentType::Recommendation => write!(f, "Recommendation"),
            ComponentType::DecisionQuality => write!(f, "Decision Quality"),
        }
    }
}
```

---

## 3. Domain Event Conventions

### Event Naming

```
{aggregate}.{action}
{aggregate}.{entity}.{action}
```

| Pattern | Examples |
|---------|----------|
| `{aggregate}.{action}` | `session.created`, `session.archived` |
| `{aggregate}.{entity}.{action}` | `cycle.component.started`, `cycle.component.completed` |

### Event Structure

```rust
/// All events MUST include these fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Unique event ID
    pub event_id: EventId,
    /// Event type (e.g., "session.created")
    pub event_type: String,
    /// Aggregate type (e.g., "session")
    pub aggregate_type: String,
    /// Aggregate ID
    pub aggregate_id: String,
    /// Event-specific payload
    pub payload: serde_json::Value,
    /// When the event occurred
    pub occurred_at: DateTime<Utc>,
    /// Event metadata
    pub metadata: EventMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Correlation ID for tracing related events
    pub correlation_id: Option<String>,
    /// ID of the event that caused this event
    pub causation_id: Option<String>,
    /// User who triggered the action (if applicable)
    pub user_id: Option<String>,
    /// Schema version for forward compatibility
    pub schema_version: u32,
}
```

### Event Payload Pattern

```rust
/// Pattern: Event with typed payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreated {
    pub session_id: String,
    pub user_id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
}

impl SessionCreated {
    pub const EVENT_TYPE: &'static str = "session.created";
    pub const AGGREGATE_TYPE: &'static str = "session";
    pub const SCHEMA_VERSION: u32 = 1;
}

impl From<&Session> for EventEnvelope {
    fn from(session: &Session) -> Self {
        let payload = SessionCreated {
            session_id: session.id.to_string(),
            user_id: session.user_id.to_string(),
            title: session.title.to_string(),
            created_at: session.created_at,
        };

        EventEnvelope {
            event_id: EventId::new(),
            event_type: SessionCreated::EVENT_TYPE.to_string(),
            aggregate_type: SessionCreated::AGGREGATE_TYPE.to_string(),
            aggregate_id: session.id.to_string(),
            payload: serde_json::to_value(&payload).unwrap(),
            occurred_at: Utc::now(),
            metadata: EventMetadata {
                correlation_id: None,
                causation_id: None,
                user_id: Some(session.user_id.to_string()),
                schema_version: SessionCreated::SCHEMA_VERSION,
            },
        }
    }
}
```

### Event Versioning

When event schema changes:

1. **Additive changes**: Add fields with defaults, increment `schema_version`
2. **Breaking changes**: Create new event type, maintain old handler for transition

```rust
// Version 1
#[derive(Deserialize)]
pub struct SessionCreatedV1 {
    pub session_id: String,
    pub user_id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
}

// Version 2 - added optional field
#[derive(Deserialize)]
pub struct SessionCreatedV2 {
    pub session_id: String,
    pub user_id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]  // Defaults to None for V1 events
    pub description: Option<String>,
}
```

---

## 4. API Response Format

### Success Response

```json
{
  "data": { ... },
  "meta": {
    "request_id": "req-123",
    "timestamp": "2026-01-08T10:30:00Z"
  }
}
```

### Error Response

```json
{
  "error": {
    "code": "VALIDATION_FAILED",
    "message": "Validation failed",
    "field": null,
    "errors": [
      {
        "code": "MISSING_REQUIRED",
        "message": "Title is required",
        "field": "title"
      },
      {
        "code": "INVALID_FORMAT",
        "message": "Invalid email format",
        "field": "email"
      }
    ]
  },
  "meta": {
    "request_id": "req-123",
    "timestamp": "2026-01-08T10:30:00Z"
  }
}
```

### Response Types

```rust
use serde::Serialize;

/// Successful response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
    pub meta: ResponseMeta,
}

/// Error response wrapper
#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error: DomainError,
    pub meta: ResponseMeta,
}

#[derive(Debug, Serialize)]
pub struct ResponseMeta {
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(data: T, request_id: String) -> Self {
        Self {
            data,
            meta: ResponseMeta {
                request_id,
                timestamp: Utc::now(),
            },
        }
    }
}

impl ApiErrorResponse {
    pub fn new(error: DomainError, request_id: String) -> Self {
        Self {
            error,
            meta: ResponseMeta {
                request_id,
                timestamp: Utc::now(),
            },
        }
    }
}
```

### Paginated Response

```json
{
  "data": [ ... ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total_items": 156,
    "total_pages": 8,
    "has_next": true,
    "has_prev": false
  },
  "meta": {
    "request_id": "req-123",
    "timestamp": "2026-01-08T10:30:00Z"
  }
}
```

```rust
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
    pub meta: ResponseMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub per_page: u32,
    pub total_items: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl PaginationInfo {
    pub fn new(page: u32, per_page: u32, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / (per_page as f64)).ceil() as u32;
        Self {
            page,
            per_page,
            total_items,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}
```

---

## 5. Code Organization

### File Naming

| Type | Convention | Example |
|------|------------|---------|
| Module root | `mod.rs` | `session/mod.rs` |
| Domain entity | `{entity}.rs` | `session.rs` |
| Value object | `{name}.rs` or grouped in `value_objects.rs` | `title.rs`, `value_objects.rs` |
| Command | `commands.rs` or `{command}_command.rs` | `commands.rs` |
| Query | `queries.rs` or `{query}_query.rs` | `queries.rs` |
| Repository | `{name}_repository.rs` | `session_repository.rs` |
| Events | `events.rs` | `events.rs` |
| Tests | `{module}_test.rs` or `tests/` directory | `session_test.rs` |

### Module Structure

```
backend/src/domain/{module}/
├── mod.rs                      # Public exports
├── {entity}.rs                 # Core domain entity
├── value_objects.rs            # Value objects (or individual files)
├── events.rs                   # Domain events
├── commands.rs                 # Command handlers
├── queries.rs                  # Query handlers
└── {module}_test.rs            # Unit tests

backend/src/ports/
├── mod.rs
└── {module}_repository.rs      # Repository trait

backend/src/adapters/
├── postgres/
│   └── {module}_repository.rs  # PostgreSQL implementation
└── http/
    └── {module}_handlers.rs    # HTTP handlers
```

### Import Order

```rust
// 1. Standard library
use std::collections::HashMap;
use std::sync::Arc;

// 2. External crates
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::RwLock;

// 3. Crate root imports
use crate::domain::foundation::{DomainError, ErrorCode};
use crate::ports::SessionRepository;

// 4. Super/sibling module imports
use super::events::SessionCreated;
```

### Visibility Rules

| Item | Visibility | Reason |
|------|------------|--------|
| Domain types | `pub` | Shared across modules |
| Value object internals | `pub(crate)` | Prevent external construction |
| Repository traits | `pub` | Dependency injection |
| HTTP handlers | `pub(crate)` | Only used by router |
| Test helpers | `#[cfg(test)]` | Test-only |

---

## 6. Testing Conventions

### Test Naming

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Pattern: {method}_{scenario}_{expected_result}
    #[test]
    fn new_with_valid_title_creates_session() { }

    #[test]
    fn new_with_empty_title_returns_validation_error() { }

    #[test]
    fn archive_already_archived_returns_conflict_error() { }

    // Pattern: integration tests use descriptive names
    #[tokio::test]
    async fn creating_session_publishes_event_and_updates_dashboard() { }
}
```

### Test Structure (AAA Pattern)

```rust
#[test]
fn new_with_valid_title_creates_session() {
    // Arrange
    let user_id = UserId::new();
    let title = "Important Decision";

    // Act
    let result = Session::new(user_id.clone(), title);

    // Assert
    assert!(result.is_ok());
    let session = result.unwrap();
    assert_eq!(session.user_id, user_id);
    assert_eq!(session.title.as_str(), "Important Decision");
}
```

### Test Fixtures

```rust
// tests/fixtures/mod.rs

pub fn test_user_id() -> UserId {
    UserId::parse("550e8400-e29b-41d4-a716-446655440000").unwrap()
}

pub fn test_session() -> Session {
    Session::new(test_user_id(), "Test Session").unwrap()
}

pub fn test_event_bus() -> Arc<InMemoryEventBus> {
    Arc::new(InMemoryEventBus::new())
}
```

---

## 7. Idempotency Patterns

All operations should be safely retryable. This section defines patterns for ensuring idempotency across commands and event handlers.

### Command Idempotency

Clients can provide an `Idempotency-Key` header to ensure retried requests don't cause duplicate operations.

#### Request Context

```rust
/// Request context that flows through all operations.
///
/// Automatically propagated via tower middleware.
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Unique request ID (generated at edge)
    pub request_id: String,

    /// User making the request
    pub user_id: UserId,

    /// Correlation ID for distributed tracing
    pub correlation_id: String,

    /// Optional idempotency key (client-provided)
    pub idempotency_key: Option<String>,

    /// Request timestamp
    pub timestamp: Timestamp,
}

impl RequestContext {
    /// Create context from HTTP headers.
    pub fn from_headers(headers: &HeaderMap, user_id: UserId) -> Self {
        let request_id = headers
            .get("X-Request-ID")
            .and_then(|v| v.to_str().ok())
            .map(String::from)
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let correlation_id = headers
            .get("X-Correlation-ID")
            .and_then(|v| v.to_str().ok())
            .map(String::from)
            .unwrap_or_else(|| request_id.clone());

        let idempotency_key = headers
            .get("Idempotency-Key")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        Self {
            request_id,
            user_id,
            correlation_id,
            idempotency_key,
            timestamp: Timestamp::now(),
        }
    }

    /// Propagate to event metadata.
    pub fn to_event_metadata(&self) -> EventMetadata {
        EventMetadata {
            correlation_id: Some(self.correlation_id.clone()),
            causation_id: None,
            user_id: Some(self.user_id.to_string()),
            trace_id: Some(self.request_id.clone()),
            schema_version: 1,
        }
    }
}
```

#### Idempotency Storage

```sql
-- Track idempotent requests
CREATE TABLE idempotency_keys (
    key             VARCHAR(255) PRIMARY KEY,
    user_id         VARCHAR(255) NOT NULL,

    -- Request info
    request_path    VARCHAR(500) NOT NULL,
    request_hash    VARCHAR(64) NOT NULL,  -- SHA256 of request body

    -- Response info
    response_status SMALLINT NOT NULL,
    response_body   JSONB,

    -- Lifecycle
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '24 hours'
);

-- Auto-cleanup expired keys
CREATE INDEX idx_idempotency_expires ON idempotency_keys (expires_at);
```

#### Client Usage

```typescript
// Frontend: Include idempotency key for mutating operations
async function createSession(data: CreateSessionRequest): Promise<Session> {
    const idempotencyKey = crypto.randomUUID();

    return fetch('/api/sessions', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Idempotency-Key': idempotencyKey,  // Safe to retry
        },
        body: JSON.stringify(data),
    });
}
```

### Event Handler Idempotency

All event handlers MUST be idempotent. Use the `IdempotentHandler` wrapper for database-backed deduplication.

#### Event Processing Tracker

```sql
-- Track which events each handler has processed
CREATE TABLE event_processing (
    handler_name    VARCHAR(255) NOT NULL,
    event_id        VARCHAR(255) NOT NULL,
    processed_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (handler_name, event_id)
);
```

#### Idempotent Handler Wrapper

```rust
/// Wrapper that ensures handler idempotency via database tracking.
pub struct IdempotentHandler<H: EventHandler> {
    inner: H,
    pool: PgPool,
}

#[async_trait]
impl<H: EventHandler> EventHandler for IdempotentHandler<H> {
    async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError> {
        let event_id = &event.event_id;
        let handler_name = self.inner.name();

        // Try to insert processing record (idempotent via ON CONFLICT)
        let result = sqlx::query!(
            r#"
            INSERT INTO event_processing (handler_name, event_id)
            VALUES ($1, $2)
            ON CONFLICT (handler_name, event_id) DO NOTHING
            "#,
            handler_name,
            event_id.as_str()
        )
        .execute(&self.pool)
        .await?;

        // If no rows inserted, already processed - skip
        if result.rows_affected() == 0 {
            tracing::debug!(
                "Event {} already processed by {}, skipping",
                event_id,
                handler_name
            );
            return Ok(());
        }

        // Process the event
        self.inner.handle(event).await
    }

    fn name(&self) -> &'static str {
        self.inner.name()
    }
}
```

### Idempotency Best Practices

| Scenario | Pattern | Implementation |
|----------|---------|----------------|
| **Create operations** | Client idempotency key | Return existing resource if key matches |
| **Update operations** | Optimistic locking | Include version in request, reject stale updates |
| **Event handlers** | Database deduplication | `INSERT ON CONFLICT DO NOTHING` pattern |
| **Aggregate state changes** | Event sourcing | Replay events to rebuild state |

### Examples

```rust
// ✅ Good: Idempotent event handler
impl EventHandler for DashboardUpdater {
    async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError> {
        // Use UPSERT for idempotent state updates
        sqlx::query!(
            r#"
            INSERT INTO dashboard_cache (session_id, last_updated, data)
            VALUES ($1, $2, $3)
            ON CONFLICT (session_id) DO UPDATE SET
                last_updated = EXCLUDED.last_updated,
                data = EXCLUDED.data
            "#,
            session_id,
            event.occurred_at,
            payload
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// ✅ Good: Idempotent aggregate operation
impl Cycle {
    pub fn complete_component(&mut self, comp_type: ComponentType) -> Result<(), DomainError> {
        let component = self.get_component_mut(comp_type)?;

        // Idempotent: completing an already-completed component is a no-op
        if component.status == ComponentStatus::Completed {
            return Ok(());
        }

        component.status = ComponentStatus::Completed;
        self.add_event(ComponentCompleted { cycle_id: self.id, comp_type });
        Ok(())
    }
}

// ❌ Bad: Non-idempotent counter increment
impl EventHandler for MetricsHandler {
    async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError> {
        // This will double-count on retry!
        sqlx::query!("UPDATE metrics SET count = count + 1 WHERE name = $1", "sessions")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
```

---

## Summary Checklist

When implementing a new feature, verify:

- [ ] Errors use `DomainError` with appropriate `ErrorCode`
- [ ] Value objects validate on construction
- [ ] IDs are type-safe wrappers around `Uuid`
- [ ] Events follow naming convention `{aggregate}.{action}`
- [ ] Events include all required metadata fields
- [ ] API responses use standard wrapper format
- [ ] Files follow naming conventions
- [ ] Tests follow AAA pattern with descriptive names
- [ ] Commands support idempotency keys for safe retries
- [ ] Event handlers are idempotent (use UPSERT or deduplication)

---

## Related Documents

- **System Architecture**: `docs/architecture/SYSTEM-ARCHITECTURE.md`
- **Scaling Readiness**: `docs/architecture/SCALING-READINESS.md`
- **Event Infrastructure**: `features/foundation/event-infrastructure.md`
- **Module Template**: `.claude/templates/module-template.md`

---

*Version: 1.1.0*
*Created: 2026-01-08*
*Updated: 2026-01-08 (Added Idempotency Patterns)*
