# Foundation Module Specification

## Overview

The Foundation module provides shared domain primitives used across all other modules. It contains value objects, base error types, and common enums that form the vocabulary of the Choice Sherpa domain.

---

## Module Classification

| Attribute | Value |
|-----------|-------|
| **Type** | Shared Domain (types only, no ports/adapters) |
| **Language** | Rust |
| **Responsibility** | Value objects, identifiers, enums, base errors |
| **Domain Dependencies** | None (root of dependency tree) |
| **External Dependencies** | `uuid`, `chrono`, `thiserror`, `serde` |

---

## Architecture

### Shared Domain Pattern

```
┌─────────────────────────────────────────────────────────────────┐
│                       FOUNDATION MODULE                          │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                      VALUE OBJECTS                           │ │
│  │                                                              │ │
│  │   ┌────────────┐  ┌────────────┐  ┌────────────────────┐   │ │
│  │   │ SessionId  │  │  CycleId   │  │   ComponentId      │   │ │
│  │   └────────────┘  └────────────┘  └────────────────────┘   │ │
│  │   ┌────────────┐  ┌────────────┐  ┌────────────────────┐   │ │
│  │   │   UserId   │  │ Timestamp  │  │    Percentage      │   │ │
│  │   └────────────┘  └────────────┘  └────────────────────┘   │ │
│  │   ┌────────────┐                                            │ │
│  │   │   Rating   │                                            │ │
│  │   └────────────┘                                            │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                         ENUMS                                │ │
│  │                                                              │ │
│  │   ┌─────────────────┐  ┌─────────────────────────────────┐  │ │
│  │   │ ComponentType   │  │ ComponentStatus                 │  │ │
│  │   │ (9 variants)    │  │ (NotStarted, InProgress, etc.)  │  │ │
│  │   └─────────────────┘  └─────────────────────────────────┘  │ │
│  │   ┌─────────────────┐  ┌─────────────────────────────────┐  │ │
│  │   │  CycleStatus    │  │    SessionStatus                │  │ │
│  │   └─────────────────┘  └─────────────────────────────────┘  │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                       ERROR TYPES                            │ │
│  │                                                              │ │
│  │   ┌─────────────────────────────────────────────────────┐   │ │
│  │   │ DomainError { code, message, details }              │   │ │
│  │   └─────────────────────────────────────────────────────┘   │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Domain Layer

### Identifier Value Objects

All identifiers are strongly-typed wrappers around UUIDs with validation.

```rust
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Unique identifier for a decision session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Creates a new random SessionId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a SessionId from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SessionId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// Unique identifier for a decision cycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CycleId(Uuid);

impl CycleId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for CycleId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CycleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for CycleId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// Unique identifier for a PrOACT component within a cycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComponentId(Uuid);

impl ComponentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for ComponentId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ComponentId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// User identifier (typically from auth provider)
///
/// # Data Classification: Internal
///
/// Per APPLICATION-SECURITY-STANDARD.md, this field requires:
/// - Authentication required to access
/// - Must not be logged in plain text in production
/// - Used for authorization checks
/// - Should be redacted in error messages exposed to clients
///
/// # Security Considerations
///
/// - UUIDs from identity provider (Zitadel) prevent enumeration
/// - Never expose in client-facing error messages
/// - Log only in security audit contexts with appropriate controls
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(String);

impl UserId {
    /// Creates a new UserId, returning error if empty
    pub fn new(id: impl Into<String>) -> Result<Self, ValidationError> {
        let id = id.into();
        if id.is_empty() {
            return Err(ValidationError::empty_field("user_id"));
        }
        Ok(Self(id))
    }

    /// Returns the inner string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

### Timestamp Value Object

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Immutable point in time, always UTC
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// Creates a timestamp for the current moment
    pub fn now() -> Self {
        Self(Utc::now())
    }

    /// Creates a timestamp from a DateTime<Utc>
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    /// Returns the inner DateTime
    pub fn as_datetime(&self) -> &DateTime<Utc> {
        &self.0
    }

    /// Checks if this timestamp is before another
    pub fn is_before(&self, other: &Timestamp) -> bool {
        self.0 < other.0
    }

    /// Checks if this timestamp is after another
    pub fn is_after(&self, other: &Timestamp) -> bool {
        self.0 > other.0
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}
```

### Percentage Value Object

```rust
use serde::{Deserialize, Serialize};
use std::fmt;

/// A value between 0 and 100 inclusive
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Percentage(u8);

impl Percentage {
    /// Creates a new Percentage, clamping to valid range
    pub fn new(value: u8) -> Self {
        Self(value.min(100))
    }

    /// Creates a Percentage, returning error if out of range
    pub fn try_new(value: u8) -> Result<Self, ValidationError> {
        if value > 100 {
            return Err(ValidationError::out_of_range("percentage", 0, 100, value as i32));
        }
        Ok(Self(value))
    }

    /// Returns the value as u8
    pub fn value(&self) -> u8 {
        self.0
    }

    /// Returns the value as a fraction (0.0 to 1.0)
    pub fn as_fraction(&self) -> f64 {
        f64::from(self.0) / 100.0
    }

    /// Zero percent
    pub const ZERO: Self = Self(0);

    /// One hundred percent
    pub const HUNDRED: Self = Self(100);
}

impl Default for Percentage {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}
```

### Rating Value Object (Pugh Matrix)

```rust
use serde::{Deserialize, Serialize};
use std::fmt;

/// Pugh matrix rating: -2 (much worse) to +2 (much better)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(i8)]
pub enum Rating {
    MuchWorse = -2,
    Worse = -1,
    Same = 0,
    Better = 1,
    MuchBetter = 2,
}

impl Rating {
    /// Creates a Rating from an integer, returning error if out of range
    pub fn try_from_i8(value: i8) -> Result<Self, ValidationError> {
        match value {
            -2 => Ok(Rating::MuchWorse),
            -1 => Ok(Rating::Worse),
            0 => Ok(Rating::Same),
            1 => Ok(Rating::Better),
            2 => Ok(Rating::MuchBetter),
            _ => Err(ValidationError::out_of_range("rating", -2, 2, value as i32)),
        }
    }

    /// Returns the numeric value
    pub fn value(&self) -> i8 {
        *self as i8
    }

    /// Returns the display label
    pub fn label(&self) -> &'static str {
        match self {
            Rating::MuchWorse => "Much Worse",
            Rating::Worse => "Worse",
            Rating::Same => "Same",
            Rating::Better => "Better",
            Rating::MuchBetter => "Much Better",
        }
    }

    /// Returns true if this is a positive rating
    pub fn is_positive(&self) -> bool {
        self.value() > 0
    }

    /// Returns true if this is a negative rating
    pub fn is_negative(&self) -> bool {
        self.value() < 0
    }
}

impl Default for Rating {
    fn default() -> Self {
        Rating::Same
    }
}

impl fmt::Display for Rating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.value() > 0 { "+" } else { "" };
        write!(f, "{}{}", sign, self.value())
    }
}
```

---

## Enums

### ComponentType

```rust
use serde::{Deserialize, Serialize};
use std::fmt;

/// The 9 PrOACT phases (including Issue Raising and Notes/Next Steps)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    NotesNextSteps,
}

impl ComponentType {
    /// Returns all component types in canonical order
    pub fn all() -> &'static [ComponentType] {
        &[
            ComponentType::IssueRaising,
            ComponentType::ProblemFrame,
            ComponentType::Objectives,
            ComponentType::Alternatives,
            ComponentType::Consequences,
            ComponentType::Tradeoffs,
            ComponentType::Recommendation,
            ComponentType::DecisionQuality,
            ComponentType::NotesNextSteps,
        ]
    }

    /// Returns the 0-based index of this component in the canonical order
    pub fn order_index(&self) -> usize {
        Self::all().iter().position(|c| c == self).unwrap()
    }

    /// Returns the next component in order, if any
    pub fn next(&self) -> Option<ComponentType> {
        let idx = self.order_index();
        Self::all().get(idx + 1).copied()
    }

    /// Returns the previous component in order, if any
    pub fn previous(&self) -> Option<ComponentType> {
        let idx = self.order_index();
        if idx == 0 {
            None
        } else {
            Self::all().get(idx - 1).copied()
        }
    }

    /// Returns true if this component comes before another in order
    pub fn is_before(&self, other: &ComponentType) -> bool {
        self.order_index() < other.order_index()
    }

    /// Returns the display name
    pub fn display_name(&self) -> &'static str {
        match self {
            ComponentType::IssueRaising => "Issue Raising",
            ComponentType::ProblemFrame => "Problem Frame",
            ComponentType::Objectives => "Objectives",
            ComponentType::Alternatives => "Alternatives",
            ComponentType::Consequences => "Consequences",
            ComponentType::Tradeoffs => "Tradeoffs",
            ComponentType::Recommendation => "Recommendation",
            ComponentType::DecisionQuality => "Decision Quality",
            ComponentType::NotesNextSteps => "Notes & Next Steps",
        }
    }

    /// Returns a short abbreviation (for compact displays)
    pub fn abbreviation(&self) -> &'static str {
        match self {
            ComponentType::IssueRaising => "IR",
            ComponentType::ProblemFrame => "PF",
            ComponentType::Objectives => "OBJ",
            ComponentType::Alternatives => "ALT",
            ComponentType::Consequences => "CON",
            ComponentType::Tradeoffs => "TRD",
            ComponentType::Recommendation => "REC",
            ComponentType::DecisionQuality => "DQ",
            ComponentType::NotesNextSteps => "NNS",
        }
    }
}

impl fmt::Display for ComponentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
```

### ComponentStatus

```rust
use serde::{Deserialize, Serialize};
use std::fmt;

/// Progress tracking for a component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ComponentStatus {
    #[default]
    NotStarted,
    InProgress,
    Complete,
    NeedsRevision,
}

impl ComponentStatus {
    /// Returns true if work has begun on this component
    pub fn is_started(&self) -> bool {
        !matches!(self, ComponentStatus::NotStarted)
    }

    /// Returns true if the component is finished
    pub fn is_complete(&self) -> bool {
        matches!(self, ComponentStatus::Complete)
    }

    /// Returns true if the component needs attention
    pub fn needs_work(&self) -> bool {
        matches!(
            self,
            ComponentStatus::NotStarted | ComponentStatus::InProgress | ComponentStatus::NeedsRevision
        )
    }

    /// Validates a transition from this status to another
    pub fn can_transition_to(&self, target: &ComponentStatus) -> bool {
        use ComponentStatus::*;
        matches!(
            (self, target),
            // Can start from not started
            (NotStarted, InProgress) |
            // Can complete from in progress
            (InProgress, Complete) |
            // Can mark for revision from complete or in progress
            (Complete, NeedsRevision) |
            (InProgress, NeedsRevision) |
            // Can restart work on revision
            (NeedsRevision, InProgress)
        )
    }
}

impl fmt::Display for ComponentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ComponentStatus::NotStarted => "Not Started",
            ComponentStatus::InProgress => "In Progress",
            ComponentStatus::Complete => "Complete",
            ComponentStatus::NeedsRevision => "Needs Revision",
        };
        write!(f, "{}", s)
    }
}
```

### CycleStatus

```rust
use serde::{Deserialize, Serialize};
use std::fmt;

/// Lifecycle status of a decision cycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CycleStatus {
    #[default]
    Active,
    Completed,
    Archived,
}

impl CycleStatus {
    /// Returns true if the cycle can be modified
    pub fn is_mutable(&self) -> bool {
        matches!(self, CycleStatus::Active)
    }

    /// Returns true if the cycle is finished (completed or archived)
    pub fn is_finished(&self) -> bool {
        matches!(self, CycleStatus::Completed | CycleStatus::Archived)
    }

    /// Validates a transition from this status to another
    pub fn can_transition_to(&self, target: &CycleStatus) -> bool {
        use CycleStatus::*;
        matches!(
            (self, target),
            (Active, Completed) | (Active, Archived) | (Completed, Archived)
        )
    }
}

impl fmt::Display for CycleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CycleStatus::Active => "Active",
            CycleStatus::Completed => "Completed",
            CycleStatus::Archived => "Archived",
        };
        write!(f, "{}", s)
    }
}
```

### SessionStatus

```rust
use serde::{Deserialize, Serialize};
use std::fmt;

/// Lifecycle status of a decision session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    #[default]
    Active,
    Archived,
}

impl SessionStatus {
    /// Returns true if the session can be modified
    pub fn is_mutable(&self) -> bool {
        matches!(self, SessionStatus::Active)
    }

    /// Validates a transition from this status to another
    pub fn can_transition_to(&self, target: &SessionStatus) -> bool {
        use SessionStatus::*;
        matches!((self, target), (Active, Archived))
    }
}

impl fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SessionStatus::Active => "Active",
            SessionStatus::Archived => "Archived",
        };
        write!(f, "{}", s)
    }
}
```

---

## Error Types

### Base Domain Error

```rust
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use thiserror::Error;

/// Standard domain error with code, message, and optional details
///
/// # Security Note
///
/// The `details` field is for internal debugging only. Production API
/// responses MUST NOT include `details` to prevent information leakage.
/// Use `to_client_error()` for API responses.
///
/// # Example
///
/// ```rust
/// // Internal logging - full details
/// tracing::error!(error = ?domain_error, "Operation failed");
///
/// // API response - sanitized
/// let client_error = domain_error.to_client_error();
/// return Err(ApiError::from(client_error));
/// ```
#[derive(Debug, Clone)]
pub struct DomainError {
    pub code: ErrorCode,
    pub message: String,
    /// Internal debugging details - DO NOT expose in production API responses
    pub details: HashMap<String, String>,
}

/// Client-safe error response (excludes internal details)
#[derive(Debug, Clone, Serialize)]
pub struct ClientError {
    pub code: String,
    pub message: String,
}

impl DomainError {
    /// Creates a new domain error
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: HashMap::new(),
        }
    }

    /// Adds a detail to the error
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }

    /// Convert to a client-safe error response.
    ///
    /// This method strips internal details that could expose system
    /// information to attackers. Always use this for API responses.
    pub fn to_client_error(&self) -> ClientError {
        ClientError {
            code: self.code.to_string(),
            message: self.sanitized_message(),
        }
    }

    /// Returns a sanitized message safe for client exposure.
    ///
    /// Removes any potentially sensitive information like:
    /// - Internal IDs or paths
    /// - Stack traces
    /// - Database error details
    fn sanitized_message(&self) -> String {
        // For certain error codes, return generic messages
        match self.code {
            ErrorCode::DatabaseError => "A database error occurred".to_string(),
            ErrorCode::CacheError => "A cache error occurred".to_string(),
            ErrorCode::AIProviderError => "An AI service error occurred".to_string(),
            _ => self.message.clone(),
        }
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl Error for DomainError {}

/// Error codes organized by category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // Validation errors
    ValidationFailed,
    EmptyField,
    OutOfRange,
    InvalidFormat,

    // Not found errors
    SessionNotFound,
    CycleNotFound,
    ComponentNotFound,
    ConversationNotFound,

    // State errors
    InvalidStateTransition,
    SessionArchived,
    CycleArchived,
    ComponentLocked,

    // Authorization errors
    Unauthorized,
    Forbidden,

    // AI errors
    AIProviderError,
    RateLimited,

    // Infrastructure errors
    DatabaseError,
    CacheError,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ErrorCode::ValidationFailed => "VALIDATION_FAILED",
            ErrorCode::EmptyField => "EMPTY_FIELD",
            ErrorCode::OutOfRange => "OUT_OF_RANGE",
            ErrorCode::InvalidFormat => "INVALID_FORMAT",
            ErrorCode::SessionNotFound => "SESSION_NOT_FOUND",
            ErrorCode::CycleNotFound => "CYCLE_NOT_FOUND",
            ErrorCode::ComponentNotFound => "COMPONENT_NOT_FOUND",
            ErrorCode::ConversationNotFound => "CONVERSATION_NOT_FOUND",
            ErrorCode::InvalidStateTransition => "INVALID_STATE_TRANSITION",
            ErrorCode::SessionArchived => "SESSION_ARCHIVED",
            ErrorCode::CycleArchived => "CYCLE_ARCHIVED",
            ErrorCode::ComponentLocked => "COMPONENT_LOCKED",
            ErrorCode::Unauthorized => "UNAUTHORIZED",
            ErrorCode::Forbidden => "FORBIDDEN",
            ErrorCode::AIProviderError => "AI_PROVIDER_ERROR",
            ErrorCode::RateLimited => "RATE_LIMITED",
            ErrorCode::DatabaseError => "DATABASE_ERROR",
            ErrorCode::CacheError => "CACHE_ERROR",
        };
        write!(f, "{}", s)
    }
}
```

### Validation Error

```rust
use thiserror::Error;

/// Errors that occur during value object construction
#[derive(Debug, Clone, Error)]
pub enum ValidationError {
    #[error("Field '{field}' cannot be empty")]
    EmptyField { field: String },

    #[error("Field '{field}' must be between {min} and {max}, got {actual}")]
    OutOfRange {
        field: String,
        min: i32,
        max: i32,
        actual: i32,
    },

    #[error("Field '{field}' has invalid format: {reason}")]
    InvalidFormat { field: String, reason: String },
}

impl ValidationError {
    pub fn empty_field(field: impl Into<String>) -> Self {
        ValidationError::EmptyField { field: field.into() }
    }

    pub fn out_of_range(field: impl Into<String>, min: i32, max: i32, actual: i32) -> Self {
        ValidationError::OutOfRange {
            field: field.into(),
            min,
            max,
            actual,
        }
    }

    pub fn invalid_format(field: impl Into<String>, reason: impl Into<String>) -> Self {
        ValidationError::InvalidFormat {
            field: field.into(),
            reason: reason.into(),
        }
    }

    /// Convert to a client-safe message.
    ///
    /// Returns a generic message that doesn't expose internal field names
    /// or validation rules that could help attackers probe the API.
    pub fn to_client_message(&self) -> &'static str {
        match self {
            Self::EmptyField { .. } => "A required field is missing",
            Self::OutOfRange { .. } => "A value is out of the allowed range",
            Self::InvalidFormat { .. } => "A field has an invalid format",
        }
    }
}
```

---

## File Structure

```
backend/src/domain/foundation/
├── mod.rs                  # Module exports
├── ids.rs                  # SessionId, CycleId, ComponentId, UserId
├── ids_test.rs             # ID tests
├── timestamp.rs            # Timestamp value object
├── timestamp_test.rs       # Timestamp tests
├── percentage.rs           # Percentage (0-100)
├── percentage_test.rs      # Percentage tests
├── rating.rs               # Pugh Rating (-2 to +2)
├── rating_test.rs          # Rating tests
├── component_type.rs       # ComponentType enum
├── component_type_test.rs  # ComponentType tests
├── component_status.rs     # ComponentStatus enum
├── cycle_status.rs         # CycleStatus enum
├── session_status.rs       # SessionStatus enum
├── errors.rs               # DomainError, ErrorCode, ValidationError
└── errors_test.rs          # Error tests

frontend/src/shared/domain/
├── ids.ts                  # ID type definitions
├── ids.test.ts             # ID tests
├── enums.ts                # ComponentType, Status enums
├── enums.test.ts           # Enum tests
├── errors.ts               # Error type definitions
└── index.ts                # Public exports
```

---

## Invariants

| Invariant | Enforcement |
|-----------|-------------|
| IDs are valid UUIDs | Parse validation in `FromStr` |
| UserId is non-empty | Constructor validation |
| Percentage is 0-100 | `try_new()` returns Result |
| Rating is -2 to +2 | Enum restricts values |
| Status transitions are valid | `can_transition_to()` method |
| ComponentType order is fixed | Static `all()` array |

---

## Test Categories

### Unit Tests

| Category | Example Tests |
|----------|---------------|
| ID creation | `session_id_generates_unique_values` |
| ID parsing | `session_id_parses_from_valid_string` |
| ID equality | `same_uuid_produces_equal_ids` |
| Percentage bounds | `percentage_clamps_to_100` |
| Percentage validation | `try_new_rejects_over_100` |
| Rating conversion | `rating_from_i8_valid_range` |
| Rating validation | `rating_rejects_invalid_values` |
| ComponentType order | `component_type_order_is_stable` |
| ComponentType navigation | `next_returns_none_for_last` |
| Status transitions | `not_started_can_transition_to_in_progress` |
| Status validation | `complete_cannot_transition_to_not_started` |
| Error formatting | `domain_error_displays_code_and_message` |

### Property-Based Tests

| Property | Description |
|----------|-------------|
| ID uniqueness | Generated IDs never collide |
| Percentage clamping | Any u8 input produces valid percentage |
| Rating roundtrip | Rating -> i8 -> Rating preserves value |
| ComponentType ordering | `is_before` is transitive |

---

## Integration Points

### Consumed By

| Module | Usage |
|--------|-------|
| proact-types | Component interface uses ComponentId, ComponentType, ComponentStatus |
| session | Session aggregate uses SessionId, UserId, SessionStatus, Timestamp |
| cycle | Cycle aggregate uses CycleId, ComponentId, ComponentType, CycleStatus |
| conversation | Conversation uses ComponentId, Timestamp |
| analysis | Analysis uses Rating, Percentage |
| dashboard | Views use all ID types and statuses |

### Frontend Mirroring

TypeScript types must match Rust definitions:

```typescript
// ids.ts
export type SessionId = string;  // UUID as string
export type CycleId = string;
export type ComponentId = string;
export type UserId = string;

// enums.ts
export enum ComponentType {
  IssueRaising = 'issue_raising',
  ProblemFrame = 'problem_frame',
  Objectives = 'objectives',
  Alternatives = 'alternatives',
  Consequences = 'consequences',
  Tradeoffs = 'tradeoffs',
  Recommendation = 'recommendation',
  DecisionQuality = 'decision_quality',
  NotesNextSteps = 'notes_next_steps',
}

export enum ComponentStatus {
  NotStarted = 'not_started',
  InProgress = 'in_progress',
  Complete = 'complete',
  NeedsRevision = 'needs_revision',
}

// errors.ts
export interface DomainError {
  code: string;
  message: string;
  details?: Record<string, string>;
}
```

---

## Security Considerations

### Data Classification

| Type | Classification | Handling |
|------|----------------|----------|
| UserId | Internal | Redact in logs, never expose in client errors |
| SessionId | Internal | Safe to include in URLs (UUID) |
| CycleId | Internal | Safe to include in URLs (UUID) |
| ComponentId | Internal | Safe to include in URLs (UUID) |
| DomainError.details | Internal | Never expose to clients |

### Error Handling Security

1. **Client-Facing Errors**: Always use `to_client_error()` or `to_client_message()` methods
2. **Internal Logging**: Full error details logged with `tracing::error!`
3. **Generic Messages**: Database, cache, and AI provider errors return generic messages

### ID Security

All entity IDs use UUID v4 which:
- Prevents enumeration attacks (128-bit random)
- Is safe to expose in URLs and responses
- Cannot be guessed or predicted

---

*Module Version: 1.0.0*
*Based on: SYSTEM-ARCHITECTURE.md v1.1.0*
*Language: Rust*
