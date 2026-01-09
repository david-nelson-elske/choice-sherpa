# Error Handling Strategy

**Type:** Cross-Cutting Architecture
**Priority:** P1 (Required for consistent API behavior)
**Last Updated:** 2026-01-08

> Unified error handling patterns, error code inventory, and HTTP mapping for Choice Sherpa.

---

## Overview

Choice Sherpa uses a layered error handling approach that:
1. **Preserves domain semantics** - Errors originate with meaningful domain context
2. **Maps consistently to HTTP** - Every error has a deterministic HTTP response
3. **Enables client recovery** - Clients can programmatically handle errors
4. **Supports debugging** - Errors include correlation IDs and context

---

## Error Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              ERROR FLOW                                          │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   Domain Layer            Application Layer          HTTP Adapter                │
│   (DomainError)           (CommandError)             (Problem JSON)              │
│                                                                                  │
│   ┌─────────────┐         ┌─────────────┐            ┌─────────────────────┐    │
│   │ValidationErr│────────►│ CommandErr  │───────────►│ 400 Bad Request     │    │
│   │             │         │ (Validation)│            │ (Problem+JSON)      │    │
│   └─────────────┘         └─────────────┘            └─────────────────────┘    │
│                                                                                  │
│   ┌─────────────┐         ┌─────────────┐            ┌─────────────────────┐    │
│   │NotFoundErr  │────────►│ CommandErr  │───────────►│ 404 Not Found       │    │
│   │             │         │ (NotFound)  │            │ (Problem+JSON)      │    │
│   └─────────────┘         └─────────────┘            └─────────────────────┘    │
│                                                                                  │
│   ┌─────────────┐         ┌─────────────┐            ┌─────────────────────┐    │
│   │AuthzError   │────────►│ CommandErr  │───────────►│ 403 Forbidden       │    │
│   │             │         │(Unauthorized)            │ (Problem+JSON)      │    │
│   └─────────────┘         └─────────────┘            └─────────────────────┘    │
│                                                                                  │
│   ┌─────────────┐         ┌─────────────┐            ┌─────────────────────┐    │
│   │  Internal   │────────►│ CommandErr  │───────────►│ 500 Internal Error  │    │
│   │             │         │ (Internal)  │            │ (Sanitized)         │    │
│   └─────────────┘         └─────────────┘            └─────────────────────┘    │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Foundation Error Types

### DomainError

```rust
// backend/src/domain/foundation/errors.rs

use serde::{Deserialize, Serialize};
use std::fmt;

/// Standardized domain error with code and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainError {
    /// Machine-readable error code
    pub code: ErrorCode,
    /// Human-readable message
    pub message: String,
    /// Optional field that caused the error
    pub field: Option<String>,
    /// Additional context for debugging
    pub context: Option<serde_json::Value>,
}

impl DomainError {
    pub fn new<S: Into<String>>(code: ErrorCode, message: S) -> Self {
        Self {
            code,
            message: message.into(),
            field: None,
            context: None,
        }
    }

    pub fn with_field<S: Into<String>>(mut self, field: S) -> Self {
        self.field = Some(field.into());
        self
    }

    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }

    // === Common constructors ===

    pub fn not_found<S: Into<String>>(entity: S) -> Self {
        Self::new(
            ErrorCode::NotFound,
            format!("{} not found", entity.into()),
        )
    }

    pub fn validation<S: Into<String>, M: Into<String>>(field: S, message: M) -> Self {
        Self::new(ErrorCode::ValidationFailed, message)
            .with_field(field)
    }

    pub fn unauthorized<S: Into<String>>(reason: S) -> Self {
        Self::new(ErrorCode::Unauthorized, reason)
    }

    pub fn conflict<S: Into<String>>(message: S) -> Self {
        Self::new(ErrorCode::Conflict, message)
    }

    pub fn invalid_state<S: Into<String>>(message: S) -> Self {
        Self::new(ErrorCode::InvalidStateTransition, message)
    }
}

impl std::error::Error for DomainError {}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code.as_str(), self.message)
    }
}
```

### ErrorCode Enum

```rust
/// All error codes in the system
/// Pattern: {MODULE}_{ERROR_TYPE} or {GENERAL}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // === General Errors (000-099) ===
    #[serde(rename = "INTERNAL_ERROR")]
    InternalError,
    #[serde(rename = "NOT_FOUND")]
    NotFound,
    #[serde(rename = "VALIDATION_FAILED")]
    ValidationFailed,
    #[serde(rename = "UNAUTHORIZED")]
    Unauthorized,
    #[serde(rename = "FORBIDDEN")]
    Forbidden,
    #[serde(rename = "CONFLICT")]
    Conflict,
    #[serde(rename = "RATE_LIMITED")]
    RateLimited,

    // === Session Errors (100-199) ===
    #[serde(rename = "SESSION_NOT_FOUND")]
    SessionNotFound,
    #[serde(rename = "SESSION_ARCHIVED")]
    SessionArchived,
    #[serde(rename = "SESSION_LIMIT_REACHED")]
    SessionLimitReached,

    // === Cycle Errors (200-299) ===
    #[serde(rename = "CYCLE_NOT_FOUND")]
    CycleNotFound,
    #[serde(rename = "CYCLE_ARCHIVED")]
    CycleArchived,
    #[serde(rename = "INVALID_STATE_TRANSITION")]
    InvalidStateTransition,
    #[serde(rename = "CANNOT_BRANCH")]
    CannotBranch,

    // === Component Errors (300-399) ===
    #[serde(rename = "COMPONENT_NOT_FOUND")]
    ComponentNotFound,
    #[serde(rename = "COMPONENT_NOT_STARTED")]
    ComponentNotStarted,
    #[serde(rename = "COMPONENT_ALREADY_COMPLETE")]
    ComponentAlreadyComplete,
    #[serde(rename = "INVALID_COMPONENT_OUTPUT")]
    InvalidComponentOutput,
    #[serde(rename = "PREVIOUS_COMPONENT_REQUIRED")]
    PreviousComponentRequired,

    // === Conversation Errors (400-499) ===
    #[serde(rename = "CONVERSATION_NOT_FOUND")]
    ConversationNotFound,
    #[serde(rename = "MESSAGE_TOO_LONG")]
    MessageTooLong,
    #[serde(rename = "CONVERSATION_LOCKED")]
    ConversationLocked,

    // === AI Provider Errors (500-599) ===
    #[serde(rename = "AI_RATE_LIMITED")]
    AIRateLimited,
    #[serde(rename = "AI_CONTEXT_TOO_LONG")]
    AIContextTooLong,
    #[serde(rename = "AI_CONTENT_FILTERED")]
    AIContentFiltered,
    #[serde(rename = "AI_UNAVAILABLE")]
    AIUnavailable,
    #[serde(rename = "AI_DAILY_LIMIT_REACHED")]
    AIDailyLimitReached,

    // === Membership Errors (600-699) ===
    #[serde(rename = "MEMBERSHIP_NOT_FOUND")]
    MembershipNotFound,
    #[serde(rename = "TIER_LIMIT_EXCEEDED")]
    TierLimitExceeded,
    #[serde(rename = "PAYMENT_REQUIRED")]
    PaymentRequired,
    #[serde(rename = "PAYMENT_FAILED")]
    PaymentFailed,
    #[serde(rename = "SUBSCRIPTION_CANCELLED")]
    SubscriptionCancelled,

    // === Authentication Errors (700-799) ===
    #[serde(rename = "AUTHENTICATION_REQUIRED")]
    AuthenticationRequired,
    #[serde(rename = "TOKEN_EXPIRED")]
    TokenExpired,
    #[serde(rename = "TOKEN_INVALID")]
    TokenInvalid,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::InternalError => "INTERNAL_ERROR",
            ErrorCode::NotFound => "NOT_FOUND",
            ErrorCode::ValidationFailed => "VALIDATION_FAILED",
            ErrorCode::Unauthorized => "UNAUTHORIZED",
            ErrorCode::Forbidden => "FORBIDDEN",
            ErrorCode::Conflict => "CONFLICT",
            ErrorCode::RateLimited => "RATE_LIMITED",
            ErrorCode::SessionNotFound => "SESSION_NOT_FOUND",
            ErrorCode::SessionArchived => "SESSION_ARCHIVED",
            ErrorCode::SessionLimitReached => "SESSION_LIMIT_REACHED",
            ErrorCode::CycleNotFound => "CYCLE_NOT_FOUND",
            ErrorCode::CycleArchived => "CYCLE_ARCHIVED",
            ErrorCode::InvalidStateTransition => "INVALID_STATE_TRANSITION",
            ErrorCode::CannotBranch => "CANNOT_BRANCH",
            ErrorCode::ComponentNotFound => "COMPONENT_NOT_FOUND",
            ErrorCode::ComponentNotStarted => "COMPONENT_NOT_STARTED",
            ErrorCode::ComponentAlreadyComplete => "COMPONENT_ALREADY_COMPLETE",
            ErrorCode::InvalidComponentOutput => "INVALID_COMPONENT_OUTPUT",
            ErrorCode::PreviousComponentRequired => "PREVIOUS_COMPONENT_REQUIRED",
            ErrorCode::ConversationNotFound => "CONVERSATION_NOT_FOUND",
            ErrorCode::MessageTooLong => "MESSAGE_TOO_LONG",
            ErrorCode::ConversationLocked => "CONVERSATION_LOCKED",
            ErrorCode::AIRateLimited => "AI_RATE_LIMITED",
            ErrorCode::AIContextTooLong => "AI_CONTEXT_TOO_LONG",
            ErrorCode::AIContentFiltered => "AI_CONTENT_FILTERED",
            ErrorCode::AIUnavailable => "AI_UNAVAILABLE",
            ErrorCode::AIDailyLimitReached => "AI_DAILY_LIMIT_REACHED",
            ErrorCode::MembershipNotFound => "MEMBERSHIP_NOT_FOUND",
            ErrorCode::TierLimitExceeded => "TIER_LIMIT_EXCEEDED",
            ErrorCode::PaymentRequired => "PAYMENT_REQUIRED",
            ErrorCode::PaymentFailed => "PAYMENT_FAILED",
            ErrorCode::SubscriptionCancelled => "SUBSCRIPTION_CANCELLED",
            ErrorCode::AuthenticationRequired => "AUTHENTICATION_REQUIRED",
            ErrorCode::TokenExpired => "TOKEN_EXPIRED",
            ErrorCode::TokenInvalid => "TOKEN_INVALID",
        }
    }

    /// Map error code to HTTP status
    pub fn http_status(&self) -> u16 {
        match self {
            // 400 Bad Request
            ErrorCode::ValidationFailed |
            ErrorCode::InvalidComponentOutput |
            ErrorCode::MessageTooLong => 400,

            // 401 Unauthorized
            ErrorCode::AuthenticationRequired |
            ErrorCode::TokenExpired |
            ErrorCode::TokenInvalid => 401,

            // 402 Payment Required
            ErrorCode::PaymentRequired |
            ErrorCode::PaymentFailed => 402,

            // 403 Forbidden
            ErrorCode::Unauthorized |
            ErrorCode::Forbidden |
            ErrorCode::TierLimitExceeded |
            ErrorCode::SubscriptionCancelled => 403,

            // 404 Not Found
            ErrorCode::NotFound |
            ErrorCode::SessionNotFound |
            ErrorCode::CycleNotFound |
            ErrorCode::ComponentNotFound |
            ErrorCode::ConversationNotFound |
            ErrorCode::MembershipNotFound => 404,

            // 409 Conflict
            ErrorCode::Conflict |
            ErrorCode::InvalidStateTransition |
            ErrorCode::CannotBranch |
            ErrorCode::SessionArchived |
            ErrorCode::CycleArchived |
            ErrorCode::ComponentNotStarted |
            ErrorCode::ComponentAlreadyComplete |
            ErrorCode::PreviousComponentRequired |
            ErrorCode::ConversationLocked => 409,

            // 429 Too Many Requests
            ErrorCode::RateLimited |
            ErrorCode::AIRateLimited |
            ErrorCode::AIDailyLimitReached |
            ErrorCode::SessionLimitReached => 429,

            // 500 Internal Server Error
            ErrorCode::InternalError => 500,

            // 502 Bad Gateway (external service errors)
            ErrorCode::AIUnavailable => 502,

            // 422 Unprocessable Entity (valid syntax but semantic error)
            ErrorCode::AIContextTooLong |
            ErrorCode::AIContentFiltered => 422,
        }
    }
}
```

---

## Application Layer Errors

### CommandError

```rust
// backend/src/application/errors.rs

use crate::domain::foundation::{DomainError, ErrorCode};

/// Errors that can occur during command execution
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Event publishing failed: {0}")]
    EventPublishing(String),

    #[error("External service error: {0}")]
    ExternalService(String),
}

impl CommandError {
    /// Convert to DomainError for HTTP response
    pub fn into_domain_error(self) -> DomainError {
        match self {
            CommandError::Domain(e) => e,
            CommandError::Repository(msg) => {
                // Log full error internally
                tracing::error!("Repository error: {}", msg);
                DomainError::new(ErrorCode::InternalError, "Database operation failed")
            }
            CommandError::EventPublishing(msg) => {
                tracing::error!("Event publishing error: {}", msg);
                DomainError::new(ErrorCode::InternalError, "Failed to publish event")
            }
            CommandError::ExternalService(msg) => {
                tracing::error!("External service error: {}", msg);
                DomainError::new(ErrorCode::InternalError, "External service unavailable")
            }
        }
    }
}
```

### QueryError

```rust
/// Errors that can occur during query execution
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Database error: {0}")]
    Database(String),
}

impl QueryError {
    pub fn into_domain_error(self) -> DomainError {
        match self {
            QueryError::NotFound(entity) => DomainError::not_found(entity),
            QueryError::Unauthorized => DomainError::unauthorized("Access denied"),
            QueryError::Database(msg) => {
                tracing::error!("Database query error: {}", msg);
                DomainError::new(ErrorCode::InternalError, "Query failed")
            }
        }
    }
}
```

---

## HTTP Layer: Problem JSON

All API errors follow [RFC 7807 Problem Details](https://datatracker.ietf.org/doc/html/rfc7807).

### Problem Response Structure

```rust
// backend/src/adapters/http/error.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use crate::domain::foundation::DomainError;

/// RFC 7807 Problem Details response
#[derive(Debug, Serialize)]
pub struct ProblemResponse {
    /// URI reference identifying the problem type
    #[serde(rename = "type")]
    pub problem_type: String,

    /// Short human-readable summary
    pub title: String,

    /// HTTP status code
    pub status: u16,

    /// Human-readable explanation
    pub detail: String,

    /// URI reference identifying the specific occurrence
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,

    /// Machine-readable error code (extension)
    pub code: String,

    /// Field that caused the error (extension)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,

    /// Correlation ID for tracing (extension)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

impl ProblemResponse {
    pub fn from_domain_error(error: DomainError, trace_id: Option<String>) -> Self {
        let status = error.code.http_status();

        Self {
            problem_type: format!(
                "https://api.choicesherpa.com/problems/{}",
                error.code.as_str().to_lowercase().replace('_', "-")
            ),
            title: Self::title_for_status(status),
            status,
            detail: error.message,
            instance: None,
            code: error.code.as_str().to_string(),
            field: error.field,
            trace_id,
        }
    }

    fn title_for_status(status: u16) -> String {
        match status {
            400 => "Bad Request",
            401 => "Unauthorized",
            402 => "Payment Required",
            403 => "Forbidden",
            404 => "Not Found",
            409 => "Conflict",
            422 => "Unprocessable Entity",
            429 => "Too Many Requests",
            500 => "Internal Server Error",
            502 => "Bad Gateway",
            _ => "Error",
        }
        .to_string()
    }
}

impl IntoResponse for ProblemResponse {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (
            status,
            [("content-type", "application/problem+json")],
            Json(self),
        )
            .into_response()
    }
}
```

### Example Response

```json
HTTP/1.1 409 Conflict
Content-Type: application/problem+json

{
  "type": "https://api.choicesherpa.com/problems/invalid-state-transition",
  "title": "Conflict",
  "status": 409,
  "detail": "Cannot start Alternatives before Objectives is started",
  "code": "INVALID_STATE_TRANSITION",
  "field": null,
  "trace_id": "abc123-def456"
}
```

---

## Error Code Inventory

### Session Module (100-199)

| Code | HTTP | Description | Recovery |
|------|------|-------------|----------|
| `SESSION_NOT_FOUND` | 404 | Session ID doesn't exist | Check ID, list user's sessions |
| `SESSION_ARCHIVED` | 409 | Session is archived, read-only | Unarchive or create new |
| `SESSION_LIMIT_REACHED` | 429 | User hit session creation limit | Upgrade tier or wait |

### Cycle Module (200-299)

| Code | HTTP | Description | Recovery |
|------|------|-------------|----------|
| `CYCLE_NOT_FOUND` | 404 | Cycle ID doesn't exist | Check ID, list session cycles |
| `CYCLE_ARCHIVED` | 409 | Cycle is archived, read-only | Create new cycle |
| `INVALID_STATE_TRANSITION` | 409 | Invalid status change | Check current state first |
| `CANNOT_BRANCH` | 409 | Cannot branch at this component | Component must be started |

### Component Module (300-399)

| Code | HTTP | Description | Recovery |
|------|------|-------------|----------|
| `COMPONENT_NOT_FOUND` | 404 | Component doesn't exist | Check cycle and type |
| `COMPONENT_NOT_STARTED` | 409 | Component not yet started | Start component first |
| `COMPONENT_ALREADY_COMPLETE` | 409 | Component already completed | Cannot modify completed |
| `INVALID_COMPONENT_OUTPUT` | 400 | Output doesn't match schema | Fix validation errors |
| `PREVIOUS_COMPONENT_REQUIRED` | 409 | Must start previous first | Follow PrOACT order |

### Conversation Module (400-499)

| Code | HTTP | Description | Recovery |
|------|------|-------------|----------|
| `CONVERSATION_NOT_FOUND` | 404 | Conversation doesn't exist | Start component first |
| `MESSAGE_TOO_LONG` | 400 | Message exceeds limit | Shorten message |
| `CONVERSATION_LOCKED` | 409 | AI is generating response | Wait for completion |

### AI Provider (500-599)

| Code | HTTP | Description | Recovery |
|------|------|-------------|----------|
| `AI_RATE_LIMITED` | 429 | Provider rate limit hit | Retry with backoff |
| `AI_CONTEXT_TOO_LONG` | 422 | Conversation too long | Summarize or new cycle |
| `AI_CONTENT_FILTERED` | 422 | Content policy violation | Rephrase message |
| `AI_UNAVAILABLE` | 502 | AI provider down | Retry or use fallback |
| `AI_DAILY_LIMIT_REACHED` | 429 | Daily AI usage exhausted | Upgrade or wait |

### Membership Module (600-699)

| Code | HTTP | Description | Recovery |
|------|------|-------------|----------|
| `MEMBERSHIP_NOT_FOUND` | 404 | No membership for user | Create membership |
| `TIER_LIMIT_EXCEEDED` | 403 | Feature not in tier | Upgrade tier |
| `PAYMENT_REQUIRED` | 402 | Payment needed | Complete payment |
| `PAYMENT_FAILED` | 402 | Payment processing error | Update payment method |
| `SUBSCRIPTION_CANCELLED` | 403 | Subscription cancelled | Resubscribe |

### Authentication (700-799)

| Code | HTTP | Description | Recovery |
|------|------|-------------|----------|
| `AUTHENTICATION_REQUIRED` | 401 | No auth token provided | Login first |
| `TOKEN_EXPIRED` | 401 | JWT has expired | Refresh token |
| `TOKEN_INVALID` | 401 | JWT is malformed | Re-authenticate |

---

## Error Handling Patterns

### Handler Error Mapping

```rust
// Standard pattern for HTTP handlers
pub async fn create_session(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(request): Json<CreateSessionRequest>,
) -> Result<Json<SessionResponse>, ProblemResponse> {
    let trace_id = tracing::Span::current()
        .context()
        .span()
        .span_context()
        .trace_id()
        .to_string();

    state
        .create_session_handler
        .handle(CreateSessionCommand {
            user_id: user.id,
            title: request.title,
        })
        .await
        .map(|session| Json(SessionResponse::from(session)))
        .map_err(|e| ProblemResponse::from_domain_error(e.into_domain_error(), Some(trace_id)))
}
```

### Validation Errors

```rust
// Aggregate multiple validation errors
pub fn validate_create_session(request: &CreateSessionRequest) -> Result<(), DomainError> {
    let mut errors = Vec::new();

    if request.title.is_empty() {
        errors.push(DomainError::validation("title", "Title is required"));
    }

    if request.title.len() > 200 {
        errors.push(DomainError::validation("title", "Title too long (max 200 chars)"));
    }

    if errors.is_empty() {
        Ok(())
    } else if errors.len() == 1 {
        Err(errors.pop().unwrap())
    } else {
        Err(DomainError::new(
            ErrorCode::ValidationFailed,
            "Multiple validation errors",
        ).with_context(serde_json::json!({
            "errors": errors.iter().map(|e| {
                serde_json::json!({
                    "field": e.field,
                    "message": e.message
                })
            }).collect::<Vec<_>>()
        })))
    }
}
```

### Domain Error Propagation

```rust
// In domain aggregate
impl Cycle {
    pub fn start_component(&mut self, ct: ComponentType) -> Result<(), DomainError> {
        self.ensure_mutable()?;  // Returns DomainError::invalid_state if archived

        if !self.can_start(&ct) {
            return Err(DomainError::new(
                ErrorCode::PreviousComponentRequired,
                format!("Cannot start {:?} - previous component not started", ct),
            ).with_context(serde_json::json!({
                "requested": ct.as_str(),
                "current": self.current_step.as_str()
            })));
        }

        // ... rest of logic
        Ok(())
    }
}
```

---

## Logging and Observability

### Error Logging

```rust
// Structured error logging
fn log_error(error: &DomainError, trace_id: &str) {
    match error.code.http_status() {
        500..=599 => {
            tracing::error!(
                error_code = %error.code.as_str(),
                message = %error.message,
                trace_id = %trace_id,
                field = ?error.field,
                "Internal server error"
            );
        }
        400..=499 => {
            tracing::warn!(
                error_code = %error.code.as_str(),
                message = %error.message,
                trace_id = %trace_id,
                field = ?error.field,
                "Client error"
            );
        }
        _ => {
            tracing::info!(
                error_code = %error.code.as_str(),
                message = %error.message,
                trace_id = %trace_id,
                "Request error"
            );
        }
    }
}
```

### Error Metrics

```rust
// Prometheus metrics for error monitoring
use prometheus::{IntCounterVec, register_int_counter_vec};

lazy_static! {
    static ref ERROR_COUNTER: IntCounterVec = register_int_counter_vec!(
        "choicesherpa_errors_total",
        "Total errors by code and module",
        &["code", "module", "status"]
    ).unwrap();
}

fn record_error(error: &DomainError) {
    let module = error.code.module();
    let status = error.code.http_status().to_string();

    ERROR_COUNTER
        .with_label_values(&[error.code.as_str(), module, &status])
        .inc();
}
```

---

## Frontend Error Handling

### TypeScript Error Types

```typescript
// frontend/src/lib/errors.ts

export interface ProblemResponse {
  type: string;
  title: string;
  status: number;
  detail: string;
  code: string;
  field?: string;
  traceId?: string;
}

export class ApiError extends Error {
  constructor(
    public readonly problem: ProblemResponse,
    public readonly raw?: unknown
  ) {
    super(problem.detail);
    this.name = 'ApiError';
  }

  get code(): string {
    return this.problem.code;
  }

  get status(): number {
    return this.problem.status;
  }

  get isRetryable(): boolean {
    return this.status === 429 || this.status === 502 || this.status === 503;
  }

  get isAuthError(): boolean {
    return this.status === 401;
  }
}

// API client wrapper
export async function apiRequest<T>(
  url: string,
  options?: RequestInit
): Promise<T> {
  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });

  if (!response.ok) {
    const contentType = response.headers.get('content-type');
    if (contentType?.includes('application/problem+json')) {
      const problem: ProblemResponse = await response.json();
      throw new ApiError(problem);
    }
    throw new ApiError({
      type: 'about:blank',
      title: response.statusText,
      status: response.status,
      detail: 'An unexpected error occurred',
      code: 'UNKNOWN_ERROR',
    });
  }

  return response.json();
}
```

### Error Display Component

```svelte
<!-- frontend/src/lib/components/ErrorAlert.svelte -->
<script lang="ts">
  import type { ApiError } from '$lib/errors';

  export let error: ApiError;

  const errorMessages: Record<string, string> = {
    SESSION_NOT_FOUND: 'This session no longer exists.',
    INVALID_STATE_TRANSITION: 'This action is not allowed in the current state.',
    AI_DAILY_LIMIT_REACHED: 'You\'ve reached your daily AI usage limit. Upgrade your plan for more.',
    TIER_LIMIT_EXCEEDED: 'This feature requires a higher subscription tier.',
    // ... etc
  };

  $: userMessage = errorMessages[error.code] ?? error.problem.detail;
</script>

<div class="error-alert" role="alert">
  <h4>{error.problem.title}</h4>
  <p>{userMessage}</p>
  {#if error.isRetryable}
    <button on:click={() => dispatch('retry')}>Try Again</button>
  {/if}
</div>
```

---

## Tasks

- [ ] Implement ErrorCode enum in `backend/src/domain/foundation/errors.rs`
- [ ] Implement DomainError struct with constructors
- [ ] Create ProblemResponse adapter in HTTP layer
- [ ] Add error code to all existing domain errors
- [ ] Create frontend ApiError class
- [ ] Add error metrics collection
- [ ] Document all error codes in OpenAPI spec
- [ ] Write unit tests for error code → HTTP mapping

---

## Related Documents

- **Foundation Module:** `docs/modules/foundation.md`
- **HTTP Adapter:** `docs/adapters/http.md`
- **OpenAPI Spec:** `docs/api/openapi.yaml`

---

*Version: 1.0.0*
*Created: 2026-01-08*
