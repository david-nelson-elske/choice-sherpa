# Authorization Model

**Type:** Architecture Reference
**Priority:** P0 (Security-Critical)
**Last Updated:** 2026-01-08

> Unified authorization model for Choice Sherpa, covering ownership verification, membership-based access control, and per-module authorization patterns.

---

## Overview

Choice Sherpa uses a two-layer authorization model:

1. **Ownership Authorization**: Verifies that a user owns the resource they're accessing
2. **Access Control**: Verifies that a user's membership tier permits the action

Both layers MUST pass for any protected operation.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        AUTHORIZATION FLOW                                    │
│                                                                              │
│   Request                                                                    │
│     │                                                                        │
│     ▼                                                                        │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ Layer 1: AUTHENTICATION                                              │   │
│   │                                                                      │   │
│   │   Zitadel OIDC → JWT Validation → Extract user_id                   │   │
│   │                                                                      │   │
│   │   Result: UserId from verified JWT                                   │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│     │                                                                        │
│     ▼                                                                        │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ Layer 2: OWNERSHIP AUTHORIZATION                                     │   │
│   │                                                                      │   │
│   │   Resource.authorize(user_id) → Ok or Err(Unauthorized)             │   │
│   │                                                                      │   │
│   │   "Does this user OWN this resource?"                               │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│     │                                                                        │
│     ▼                                                                        │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ Layer 3: ACCESS CONTROL                                              │   │
│   │                                                                      │   │
│   │   AccessChecker.can_<action>(user_id) → Allowed or Denied(reason)   │   │
│   │                                                                      │   │
│   │   "Is the user's membership tier PERMITTED to do this?"             │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│     │                                                                        │
│     ▼                                                                        │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ Layer 4: BUSINESS LOGIC                                              │   │
│   │                                                                      │   │
│   │   Domain invariants, state machine transitions, etc.                │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Core Concepts

### Ownership Chain

Resources form an ownership hierarchy:

```
User
  └── Session (user owns sessions)
        └── Cycle (session contains cycles)
              └── Component (cycle owns components)
                    └── Conversation (component has conversation)
```

Authorization follows the ownership chain:
- To access a Component, verify user owns the Session that contains its Cycle
- To access a Conversation, verify user owns the Session that contains its Component's Cycle

### Authorization Result Types

```rust
/// Result of ownership authorization
pub enum AuthorizationResult {
    Authorized,
    Unauthorized(UnauthorizedReason),
}

#[derive(Debug, Clone)]
pub enum UnauthorizedReason {
    NotOwner,
    ResourceNotFound,
    InsufficientPermission,
}

/// Result of access control check
pub enum AccessResult {
    Allowed,
    Denied(AccessDeniedReason),
}

#[derive(Debug, Clone)]
pub enum AccessDeniedReason {
    NoMembership,
    MembershipExpired,
    MembershipPastDue,
    SessionLimitReached { current: u32, max: u32 },
    CycleLimitReached { current: u32, max: u32 },
    FeatureNotIncluded { feature: String, required_tier: MembershipTier },
}
```

---

## Module Authorization Requirements

### Session Module

| Operation | Ownership Check | Access Check |
|-----------|-----------------|--------------|
| CreateSession | N/A (new resource) | `can_create_session(user_id)` |
| GetSession | `session.user_id == user_id` | None |
| ListSessions | Implicit (filter by user_id) | None |
| ArchiveSession | `session.user_id == user_id` | None |
| RenameSession | `session.user_id == user_id` | None |

```rust
impl CreateSessionHandler {
    pub async fn handle(&self, cmd: CreateSession) -> Result<Session, CommandError> {
        // 1. Access control check FIRST
        match self.access_checker.can_create_session(&cmd.user_id).await? {
            AccessResult::Denied(reason) => {
                return Err(CommandError::AccessDenied(reason));
            }
            AccessResult::Allowed => {}
        }

        // 2. Business logic (create session)
        let session = Session::create(cmd.user_id, cmd.title)?;
        // ...
    }
}

impl GetSessionHandler {
    pub async fn handle(&self, query: GetSession) -> Result<SessionView, QueryError> {
        let session = self.session_reader.get_by_id(query.session_id).await?
            .ok_or(QueryError::NotFound)?;

        // Ownership check
        session.authorize(&query.user_id)?;

        Ok(session)
    }
}
```

### Cycle Module

| Operation | Ownership Check | Access Check |
|-----------|-----------------|--------------|
| CreateCycle | Session ownership | `can_create_cycle(user_id, session_id)` |
| GetCycle | Session ownership | None |
| BranchCycle | Session ownership | `can_create_cycle(user_id, session_id)` |
| ArchiveCycle | Session ownership | None |
| StartComponent | Session ownership | None |
| CompleteComponent | Session ownership | None |
| UpdateComponentOutput | Session ownership | None |

```rust
impl CreateCycleHandler {
    pub async fn handle(&self, cmd: CreateCycle) -> Result<Cycle, CommandError> {
        // 1. Load session and verify ownership
        let session = self.session_repo.find_by_id(cmd.session_id).await?
            .ok_or(CommandError::SessionNotFound)?;
        session.authorize(&cmd.user_id)?;

        // 2. Access control check
        match self.access_checker.can_create_cycle(&cmd.user_id, &cmd.session_id).await? {
            AccessResult::Denied(reason) => {
                return Err(CommandError::AccessDenied(reason));
            }
            AccessResult::Allowed => {}
        }

        // 3. Business logic
        let cycle = Cycle::create(cmd.session_id)?;
        // ...
    }
}
```

### Conversation Module (CRITICAL - WAS MISSING)

| Operation | Ownership Check | Access Check |
|-----------|-----------------|--------------|
| SendMessage | Session ownership (via cycle lookup) | None |
| StreamMessage | Session ownership (via cycle lookup) | None |
| GetConversation | Session ownership (via cycle lookup) | None |
| RegenerateResponse | Session ownership (via cycle lookup) | None |

**The conversation module MUST verify session ownership before any operation.**

```rust
impl SendMessageHandler {
    pub async fn handle(&self, cmd: SendMessageCommand) -> Result<SendMessageResult, CommandError> {
        // 1. AUTHORIZATION: Load cycle and verify session ownership
        let cycle = self.cycle_repo.find_by_id(cmd.cycle_id).await?
            .ok_or(CommandError::CycleNotFound)?;

        let session = self.session_repo.find_by_id(cycle.session_id()).await?
            .ok_or(CommandError::SessionNotFound)?;

        session.authorize(&cmd.user_id)?;

        // 2. Business logic (rest of handler)
        // ...
    }
}

impl GetConversationHandler {
    pub async fn handle(&self, query: GetConversationQuery) -> Result<ConversationView, QueryError> {
        // 1. AUTHORIZATION: Look up component → cycle → session
        let component_id = query.component_id;

        // Find cycle containing this component
        let cycle = self.cycle_reader.find_by_component(component_id).await?
            .ok_or(QueryError::CycleNotFound)?;

        // Load session and verify ownership
        let session = self.session_reader.get_by_id(cycle.session_id).await?
            .ok_or(QueryError::SessionNotFound)?;

        session.authorize(&query.user_id)?;

        // 2. Return conversation
        self.conversation_reader.get_by_component(component_id).await
    }
}
```

### Dashboard Module

| Operation | Ownership Check | Access Check |
|-----------|-----------------|--------------|
| GetOverview | Implicit (filter by user_id) | None |
| GetSessionDetail | Session ownership | None |
| GetCycleDetail | Session ownership | None |
| GetComponentDetail | Session ownership | None |
| ExportSession | Session ownership | `can_export(user_id)` |

```rust
impl GetSessionDetailHandler {
    pub async fn handle(&self, query: GetSessionDetail) -> Result<SessionDetailView, QueryError> {
        let session = self.session_reader.get_by_id(query.session_id).await?
            .ok_or(QueryError::NotFound)?;

        // Ownership check
        session.authorize(&query.user_id)?;

        // Build detail view
        self.build_session_detail(session).await
    }
}

impl ExportSessionHandler {
    pub async fn handle(&self, cmd: ExportSession) -> Result<ExportResult, CommandError> {
        // 1. Ownership check
        let session = self.session_reader.get_by_id(cmd.session_id).await?
            .ok_or(CommandError::NotFound)?;
        session.authorize(&cmd.user_id)?;

        // 2. Access control check
        match self.access_checker.can_export(&cmd.user_id).await? {
            AccessResult::Denied(reason) => {
                return Err(CommandError::AccessDenied(reason));
            }
            AccessResult::Allowed => {}
        }

        // 3. Export logic
        // ...
    }
}
```

---

## Session.authorize() Implementation

Every aggregate root that belongs to a user implements the `authorize()` method:

```rust
impl Session {
    /// Verifies that the given user_id owns this session.
    /// Returns Ok(()) if authorized, Err(Unauthorized) otherwise.
    pub fn authorize(&self, user_id: &UserId) -> Result<(), AuthorizationError> {
        if &self.user_id == user_id {
            Ok(())
        } else {
            Err(AuthorizationError::Unauthorized(UnauthorizedReason::NotOwner))
        }
    }
}
```

### AuthorizationError

```rust
#[derive(Debug, thiserror::Error)]
pub enum AuthorizationError {
    #[error("Unauthorized: {0:?}")]
    Unauthorized(UnauthorizedReason),
}

impl From<AuthorizationError> for CommandError {
    fn from(e: AuthorizationError) -> Self {
        CommandError::Unauthorized(e)
    }
}

impl From<AuthorizationError> for QueryError {
    fn from(e: AuthorizationError) -> Self {
        QueryError::Unauthorized(e)
    }
}
```

---

## AccessChecker Port

The `AccessChecker` port (defined in `features/integrations/membership-access-control.md`) provides membership-based access control:

```rust
#[async_trait]
pub trait AccessChecker: Send + Sync {
    async fn can_create_session(&self, user_id: &UserId) -> Result<AccessResult, DomainError>;
    async fn can_create_cycle(&self, user_id: &UserId, session_id: &SessionId) -> Result<AccessResult, DomainError>;
    async fn can_export(&self, user_id: &UserId) -> Result<AccessResult, DomainError>;
    async fn get_tier_limits(&self, user_id: &UserId) -> Result<TierLimits, DomainError>;
    async fn get_usage(&self, user_id: &UserId) -> Result<UsageStats, DomainError>;
}
```

### Implementation

The `MembershipAccessChecker` implements this port:

```rust
pub struct MembershipAccessChecker {
    membership_reader: Arc<dyn MembershipReader>,
    session_reader: Arc<dyn SessionReader>,
    cycle_reader: Arc<dyn CycleReader>,
}

#[async_trait]
impl AccessChecker for MembershipAccessChecker {
    async fn can_create_session(&self, user_id: &UserId) -> Result<AccessResult, DomainError> {
        let membership = self.membership_reader.get_by_user(user_id).await?;

        // Check membership status
        if !membership.is_active() {
            return Ok(AccessResult::Denied(AccessDeniedReason::MembershipExpired));
        }

        // Check session limit
        let limits = TierLimits::for_tier(membership.tier);
        if let Some(max) = limits.max_sessions {
            let current = self.session_reader.count_active_by_user(user_id).await?;
            if current >= max as usize {
                return Ok(AccessResult::Denied(AccessDeniedReason::SessionLimitReached {
                    current: current as u32,
                    max,
                }));
            }
        }

        Ok(AccessResult::Allowed)
    }

    // ... other methods
}
```

---

## HTTP Layer Integration

Authorization is enforced at multiple layers for defense in depth:

### 1. Authentication Middleware (Extract user_id)

```rust
// Applied to all /api/* routes
pub async fn auth_middleware(
    State(zitadel): State<ZitadelClient>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = zitadel.verify_token(token).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user_id = UserId::from_string(&claims.sub);
    request.extensions_mut().insert(user_id);

    Ok(next.run(request).await)
}
```

### 2. Handler Layer (Ownership + Access Control)

```rust
// Every handler receives authenticated user_id
pub async fn send_message(
    State(handler): State<Arc<SendMessageHandler>>,
    Extension(user_id): Extension<UserId>,  // From auth middleware
    Path(cycle_id): Path<CycleId>,
    Json(payload): Json<SendMessagePayload>,
) -> Result<Json<MessageView>, ApiError> {
    let cmd = SendMessageCommand {
        user_id,  // Always include user_id in commands
        cycle_id,
        component_type: payload.component_type,
        content: payload.content,
    };

    let result = handler.handle(cmd).await?;
    Ok(Json(result.into()))
}
```

### 3. Error Mapping

```rust
impl From<CommandError> for ApiError {
    fn from(e: CommandError) -> Self {
        match e {
            CommandError::Unauthorized(_) => ApiError {
                status: StatusCode::FORBIDDEN,
                code: "FORBIDDEN",
                message: "You don't have permission to access this resource".to_string(),
            },
            CommandError::AccessDenied(reason) => ApiError {
                status: StatusCode::FORBIDDEN,
                code: "ACCESS_DENIED",
                message: reason.to_string(),
                details: Some(serde_json::to_value(&reason).ok()),
            },
            CommandError::NotFound => ApiError {
                status: StatusCode::NOT_FOUND,
                code: "NOT_FOUND",
                message: "Resource not found".to_string(),
            },
            // ...
        }
    }
}
```

---

## Authorization Checklist by Module

Use this checklist when implementing or reviewing handlers:

### Session Module

- [ ] `CreateSession`: Check `can_create_session` before creation
- [ ] `GetSession`: Check `session.authorize(user_id)`
- [ ] `ListSessions`: Filter by `user_id` in query
- [ ] `ArchiveSession`: Check `session.authorize(user_id)`
- [ ] `RenameSession`: Check `session.authorize(user_id)`

### Cycle Module

- [ ] `CreateCycle`: Check session ownership + `can_create_cycle`
- [ ] `GetCycle`: Check session ownership
- [ ] `BranchCycle`: Check session ownership + `can_create_cycle`
- [ ] `ArchiveCycle`: Check session ownership
- [ ] `StartComponent`: Check session ownership
- [ ] `CompleteComponent`: Check session ownership
- [ ] `UpdateComponentOutput`: Check session ownership

### Conversation Module

- [ ] `SendMessage`: Check session ownership via cycle lookup
- [ ] `StreamMessage`: Check session ownership via cycle lookup
- [ ] `GetConversation`: Check session ownership via cycle lookup
- [ ] `RegenerateResponse`: Check session ownership via cycle lookup

### Dashboard Module

- [ ] `GetOverview`: Filter by user_id
- [ ] `GetSessionDetail`: Check session ownership
- [ ] `GetCycleDetail`: Check session ownership
- [ ] `GetComponentDetail`: Check session ownership
- [ ] `ExportSession`: Check session ownership + `can_export`

---

## Security Considerations

### 1. Defense in Depth

Authorization is checked at multiple layers:
- HTTP middleware (authentication)
- Handler layer (ownership + access control)
- Domain layer (invariant enforcement)

### 2. Fail Closed

If authorization cannot be determined (e.g., database error), deny access:

```rust
async fn check_access(&self, user_id: &UserId) -> Result<AccessResult, DomainError> {
    // If we can't determine access, fail closed
    match self.membership_reader.get_by_user(user_id).await {
        Ok(membership) => { /* check limits */ }
        Err(e) => {
            tracing::error!("Access check failed: {}", e);
            return Ok(AccessResult::Denied(AccessDeniedReason::NoMembership));
        }
    }
}
```

### 3. Audit Logging

Log all authorization decisions:

```rust
let result = session.authorize(&user_id);

tracing::info!(
    user_id = %user_id,
    session_id = %session.id(),
    authorized = %result.is_ok(),
    "Authorization check"
);
```

### 4. Rate Limiting

Prevent brute-force enumeration attacks:
- Rate limit by user_id and IP
- Use consistent error messages (don't reveal if resource exists)

---

## Testing Authorization

### Unit Tests

```rust
#[test]
fn session_authorize_allows_owner() {
    let user_id = UserId::new("user-1");
    let session = Session::create(user_id.clone(), "Test").unwrap();

    assert!(session.authorize(&user_id).is_ok());
}

#[test]
fn session_authorize_denies_non_owner() {
    let owner = UserId::new("user-1");
    let other = UserId::new("user-2");
    let session = Session::create(owner, "Test").unwrap();

    assert!(session.authorize(&other).is_err());
}
```

### Integration Tests

```rust
#[tokio::test]
async fn send_message_requires_session_ownership() {
    let app = setup_test_app().await;

    // Create session as user-1
    let session = create_session(&app, "user-1").await;
    let cycle = create_cycle(&app, "user-1", session.id).await;

    // Try to send message as user-2 (should fail)
    let response = app.send_message("user-2", cycle.id, "Hello").await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn create_session_respects_tier_limits() {
    let app = setup_test_app().await;

    // Create free user with 3 sessions (at limit)
    let user_id = "free-user";
    for _ in 0..3 {
        create_session(&app, user_id).await;
    }

    // 4th session should fail
    let response = app.create_session(user_id, "Fourth").await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body: ApiError = response.json().await;
    assert_eq!(body.code, "ACCESS_DENIED");
}
```

---

## Related Documents

- **Membership Access Control**: `features/integrations/membership-access-control.md`
- **Session Module**: `docs/modules/session.md`
- **Conversation Module**: `docs/modules/conversation.md`
- **Dashboard Module**: `docs/modules/dashboard.md`
- **System Architecture**: `docs/architecture/SYSTEM-ARCHITECTURE.md`

---

*Version: 1.0.0*
*Created: 2026-01-08*
