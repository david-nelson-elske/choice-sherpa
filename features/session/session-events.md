# Feature: Session Domain Events

**Module:** session
**Type:** Event Publishing
**Priority:** P0
**Phase:** 2 of Full PrOACT Journey Integration
**Depends On:** features/foundation/event-infrastructure.md

> Session module publishes domain events for session lifecycle changes, enabling other modules to react without tight coupling.

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Required |
| Authorization Model | Ownership-based: user must own session to trigger events |
| Sensitive Data | Session titles, descriptions (Confidential); User decisions (Confidential) |
| Rate Limiting | Required: 100 requests/minute per user for session operations |
| Audit Logging | All session lifecycle events (create, rename, archive) |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| session_id | Internal | Log in audit events, no special handling |
| user_id | Internal | Log in audit events, do not expose in public APIs |
| title | Confidential | Encrypt at rest, include in audit logs |
| description | Confidential | Encrypt at rest, may contain decision details |
| old_title/new_title | Confidential | Include in audit logs for change tracking |
| correlation_id | Internal | Used for request tracing, no sensitive data |

### Security Events to Log

- SessionCreated: INFO level with user_id, session_id, timestamp
- SessionRenamed: INFO level with user_id, session_id, old_title (redacted), timestamp
- SessionArchived: INFO level with user_id, session_id, timestamp
- Authorization failures (WARN level): Unauthorized rename/archive attempts
- Event publishing failures (ERROR level): Failed event delivery

### Authorization Rules

1. **Session Creation**: Authenticated user can create sessions (ownership assigned automatically)
2. **Session Rename**: Only session owner can rename (`session.is_owner(&user_id)`)
3. **Session Archive**: Only session owner can archive
4. **Event Subscription**: Dashboard handlers receive events only for sessions the user owns

---

## Problem Statement

The session module currently operates in isolation. Other modules (dashboard, notifications) need to know when:
- A new session is created
- A session is renamed
- A session is archived

Without events, these modules would need to poll or be tightly coupled to session internals.

### Current State

- Session CRUD operations succeed silently
- Dashboard must poll for updates
- No audit trail of session changes

### Desired State

- Session operations publish domain events
- Dashboard receives real-time updates via event subscription
- Complete audit trail of all session changes

---

## Tasks

- [ ] Create session events module (backend/src/domain/session/events.rs)
- [ ] Implement SessionCreated event with DomainEvent trait
- [ ] Implement SessionRenamed event with DomainEvent trait
- [ ] Implement SessionDescriptionUpdated event with DomainEvent trait
- [ ] Implement SessionArchived event with DomainEvent trait
- [ ] Implement CycleAddedToSession event with DomainEvent trait
- [ ] Update CreateSessionHandler to inject EventPublisher and publish SessionCreated
- [ ] Update RenameSessionHandler to inject EventPublisher and publish SessionRenamed
- [ ] Update ArchiveSessionHandler to inject EventPublisher and publish SessionArchived
- [ ] Implement SessionCycleTracker event handler for CycleCreated events
- [ ] Add unit tests for session event types and serialization
- [ ] Add unit tests for command handlers verifying event publishing
- [ ] Add unit tests for SessionCycleTracker handler

---

## Domain Events

### SessionCreated

Published when a new decision session is created.

```rust
// backend/src/domain/session/events.rs

use serde::{Deserialize, Serialize};

/// Published when a new session is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreated {
    /// Unique identifier for this event
    pub event_id: EventId,

    /// ID of the created session
    pub session_id: SessionId,

    /// User who created the session
    pub user_id: UserId,

    /// Session title
    pub title: String,

    /// Optional description
    pub description: Option<String>,

    /// When the session was created
    pub created_at: Timestamp,
}

impl DomainEvent for SessionCreated {
    fn event_type(&self) -> &'static str {
        "session.created"
    }

    fn aggregate_id(&self) -> String {
        self.session_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.created_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

**Subscribers:**
- `DashboardUpdateHandler` - Initialize session in dashboard view
- `AnalyticsHandler` (future) - Track session creation metrics

---

### SessionRenamed

Published when a session's title is changed.

```rust
/// Published when a session title is changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRenamed {
    pub event_id: EventId,
    pub session_id: SessionId,
    pub user_id: UserId,
    pub old_title: String,
    pub new_title: String,
    pub renamed_at: Timestamp,
}

impl DomainEvent for SessionRenamed {
    fn event_type(&self) -> &'static str {
        "session.renamed"
    }

    fn aggregate_id(&self) -> String {
        self.session_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.renamed_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

**Subscribers:**
- `DashboardUpdateHandler` - Update session title in view

---

### SessionDescriptionUpdated

Published when a session's description changes.

```rust
/// Published when a session description is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDescriptionUpdated {
    pub event_id: EventId,
    pub session_id: SessionId,
    pub user_id: UserId,
    pub old_description: Option<String>,
    pub new_description: Option<String>,
    pub updated_at: Timestamp,
}

impl DomainEvent for SessionDescriptionUpdated {
    fn event_type(&self) -> &'static str {
        "session.description_updated"
    }

    fn aggregate_id(&self) -> String {
        self.session_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.updated_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

---

### SessionArchived

Published when a session is archived (soft delete).

```rust
/// Published when a session is archived
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArchived {
    pub event_id: EventId,
    pub session_id: SessionId,
    pub user_id: UserId,
    pub archived_at: Timestamp,
}

impl DomainEvent for SessionArchived {
    fn event_type(&self) -> &'static str {
        "session.archived"
    }

    fn aggregate_id(&self) -> String {
        self.session_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.archived_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

**Subscribers:**
- `DashboardUpdateHandler` - Remove session from active list
- `CleanupHandler` (future) - Schedule data cleanup

---

### CycleAddedToSession

Published when a new cycle is linked to a session.

```rust
/// Published when a cycle is added to a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleAddedToSession {
    pub event_id: EventId,
    pub session_id: SessionId,
    pub cycle_id: CycleId,
    pub is_root_cycle: bool,
    pub added_at: Timestamp,
}

impl DomainEvent for CycleAddedToSession {
    fn event_type(&self) -> &'static str {
        "session.cycle_added"
    }

    fn aggregate_id(&self) -> String {
        self.session_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.added_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

**Note:** This event is published by the Session module in response to `CycleCreated` from the Cycle module, maintaining session's cycle list.

---

## Acceptance Criteria

### AC1: CreateSession Publishes Event

**Given** a user creates a new session
**When** the session is successfully persisted
**Then** a `SessionCreated` event is published with:
- Correct session_id
- User who created it
- Title and description
- Creation timestamp

### AC2: RenameSession Publishes Event

**Given** a session exists
**When** the title is changed
**Then** a `SessionRenamed` event is published with:
- Old and new title
- User who renamed it
- Timestamp

### AC3: ArchiveSession Publishes Event

**Given** an active session exists
**When** the session is archived
**Then** a `SessionArchived` event is published with:
- Session ID
- User who archived it
- Archive timestamp

### AC4: Events Contain Correlation ID

**Given** an HTTP request triggers a session command
**When** events are published
**Then** events include the request's correlation ID in metadata

### AC5: Events Are Idempotent-Safe

**Given** the same command is processed twice (retry scenario)
**When** events are published
**Then** each event has a unique EventId (handlers can deduplicate)

---

## Technical Design

### Command Handler Changes

```rust
// backend/src/application/commands/create_session.rs

pub struct CreateSessionHandler {
    repo: Arc<dyn SessionRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl CreateSessionHandler {
    pub fn new(
        repo: Arc<dyn SessionRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self { repo, event_publisher }
    }

    pub async fn handle(
        &self,
        cmd: CreateSessionCommand,
        metadata: CommandMetadata,
    ) -> Result<SessionId, DomainError> {
        // 1. Create session aggregate
        let session = Session::new(cmd.user_id.clone(), cmd.title.clone())?;

        if let Some(desc) = &cmd.description {
            session.update_description(Some(desc.clone()))?;
        }

        // 2. Persist session
        self.repo.save(&session).await?;

        // 3. Create and publish event
        let event = SessionCreated {
            event_id: EventId::new(),
            session_id: session.id(),
            user_id: cmd.user_id,
            title: cmd.title,
            description: cmd.description,
            created_at: Timestamp::now(),
        };

        let envelope = EventEnvelope::from_event(&event, "Session")
            .with_correlation_id(metadata.correlation_id)
            .with_user_id(cmd.user_id.to_string());

        self.event_publisher.publish(envelope).await?;

        Ok(session.id())
    }
}
```

```rust
// backend/src/application/commands/rename_session.rs

pub struct RenameSessionHandler {
    repo: Arc<dyn SessionRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl RenameSessionHandler {
    pub async fn handle(
        &self,
        cmd: RenameSessionCommand,
        metadata: CommandMetadata,
    ) -> Result<(), DomainError> {
        // 1. Load session
        let mut session = self.repo
            .find_by_id(cmd.session_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::SessionNotFound, "Session not found"))?;

        // 2. Validate ownership
        if !session.is_owner(&cmd.user_id) {
            return Err(DomainError::new(ErrorCode::SessionUnauthorized, "Not session owner"));
        }

        // 3. Capture old title for event
        let old_title = session.title().to_string();

        // 4. Apply rename
        session.rename(cmd.new_title.clone())?;

        // 5. Persist
        self.repo.update(&session).await?;

        // 6. Publish event
        let event = SessionRenamed {
            event_id: EventId::new(),
            session_id: cmd.session_id,
            user_id: cmd.user_id,
            old_title,
            new_title: cmd.new_title,
            renamed_at: Timestamp::now(),
        };

        let envelope = EventEnvelope::from_event(&event, "Session")
            .with_correlation_id(metadata.correlation_id);

        self.event_publisher.publish(envelope).await?;

        Ok(())
    }
}
```

```rust
// backend/src/application/commands/archive_session.rs

pub struct ArchiveSessionHandler {
    repo: Arc<dyn SessionRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl ArchiveSessionHandler {
    pub async fn handle(
        &self,
        cmd: ArchiveSessionCommand,
        metadata: CommandMetadata,
    ) -> Result<(), DomainError> {
        // 1. Load session
        let mut session = self.repo
            .find_by_id(cmd.session_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::SessionNotFound, "Session not found"))?;

        // 2. Validate ownership
        if !session.is_owner(&cmd.user_id) {
            return Err(DomainError::new(ErrorCode::SessionUnauthorized, "Not session owner"));
        }

        // 3. Archive
        session.archive()?;

        // 4. Persist
        self.repo.update(&session).await?;

        // 5. Publish event
        let event = SessionArchived {
            event_id: EventId::new(),
            session_id: cmd.session_id,
            user_id: cmd.user_id,
            archived_at: Timestamp::now(),
        };

        let envelope = EventEnvelope::from_event(&event, "Session")
            .with_correlation_id(metadata.correlation_id);

        self.event_publisher.publish(envelope).await?;

        Ok(())
    }
}
```

### Session Cycle Tracker (Event Handler)

The session module listens for `CycleCreated` events to maintain its cycle list:

```rust
// backend/src/application/handlers/session_cycle_tracker.rs

/// Handles CycleCreated events to update session's cycle list
pub struct SessionCycleTracker {
    session_repo: Arc<dyn SessionRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl SessionCycleTracker {
    pub fn new(
        session_repo: Arc<dyn SessionRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self { session_repo, event_publisher }
    }
}

#[async_trait]
impl EventHandler for SessionCycleTracker {
    async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError> {
        // Parse cycle created event
        let cycle_created: CycleCreated = event.payload_as()
            .map_err(|e| DomainError::new(ErrorCode::ValidationFailed, &e.to_string()))?;

        // Load session
        let mut session = self.session_repo
            .find_by_id(cycle_created.session_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::SessionNotFound, "Session not found"))?;

        // Add cycle to session
        session.add_cycle(cycle_created.cycle_id)?;

        // Persist
        self.session_repo.update(&session).await?;

        // Publish session event
        let session_event = CycleAddedToSession {
            event_id: EventId::new(),
            session_id: cycle_created.session_id,
            cycle_id: cycle_created.cycle_id,
            is_root_cycle: cycle_created.parent_cycle_id.is_none(),
            added_at: Timestamp::now(),
        };

        let envelope = EventEnvelope::from_event(&session_event, "Session")
            .with_causation_id(event.event_id.as_str());

        self.event_publisher.publish(envelope).await?;

        Ok(())
    }

    fn name(&self) -> &'static str {
        "SessionCycleTracker"
    }
}
```

---

## File Structure

```
backend/src/domain/session/
├── mod.rs                    # Add events export
├── session.rs                # Existing aggregate
├── events.rs                 # NEW: SessionCreated, SessionRenamed, etc.
└── events_test.rs            # NEW: Event unit tests

backend/src/application/commands/
├── create_session.rs         # MODIFY: Add EventPublisher
├── create_session_test.rs    # MODIFY: Test event publishing
├── rename_session.rs         # MODIFY: Add EventPublisher
├── rename_session_test.rs    # MODIFY: Test event publishing
├── archive_session.rs        # MODIFY: Add EventPublisher
└── archive_session_test.rs   # MODIFY: Test event publishing

backend/src/application/handlers/
├── mod.rs                    # NEW: Module exports
├── session_cycle_tracker.rs  # NEW: Handle CycleCreated
└── session_cycle_tracker_test.rs # NEW
```

---

## Test Specifications

### Unit Tests: Event Types

```rust
#[test]
fn session_created_implements_domain_event() {
    let event = SessionCreated {
        event_id: EventId::new(),
        session_id: SessionId::new(),
        user_id: UserId::new("user-1"),
        title: "Test Decision".to_string(),
        description: None,
        created_at: Timestamp::now(),
    };

    assert_eq!(event.event_type(), "session.created");
    assert!(!event.aggregate_id().is_empty());
}

#[test]
fn session_created_serializes_to_json() {
    let event = SessionCreated {
        event_id: EventId::from_string("evt-1"),
        session_id: SessionId::from_string("sess-1"),
        user_id: UserId::new("user-1"),
        title: "My Decision".to_string(),
        description: Some("Description".to_string()),
        created_at: Timestamp::now(),
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("My Decision"));
    assert!(json.contains("sess-1"));
}

#[test]
fn session_renamed_captures_both_titles() {
    let event = SessionRenamed {
        event_id: EventId::new(),
        session_id: SessionId::new(),
        user_id: UserId::new("user-1"),
        old_title: "Old Title".to_string(),
        new_title: "New Title".to_string(),
        renamed_at: Timestamp::now(),
    };

    assert_eq!(event.old_title, "Old Title");
    assert_eq!(event.new_title, "New Title");
}
```

### Unit Tests: Command Handlers

```rust
#[tokio::test]
async fn create_session_publishes_session_created_event() {
    // Arrange
    let repo = Arc::new(InMemorySessionRepository::new());
    let event_bus = Arc::new(InMemoryEventBus::new());
    let handler = CreateSessionHandler::new(repo, event_bus.clone());

    let cmd = CreateSessionCommand {
        user_id: UserId::new("user-123"),
        title: "Career Decision".to_string(),
        description: Some("Should I change jobs?".to_string()),
    };

    let metadata = CommandMetadata {
        correlation_id: "req-456".to_string(),
    };

    // Act
    let session_id = handler.handle(cmd, metadata).await.unwrap();

    // Assert - event published
    let events = event_bus.events_of_type("session.created");
    assert_eq!(events.len(), 1);

    // Assert - event data
    let payload: SessionCreated = events[0].payload_as().unwrap();
    assert_eq!(payload.session_id, session_id);
    assert_eq!(payload.title, "Career Decision");
    assert_eq!(payload.user_id.as_str(), "user-123");

    // Assert - metadata
    assert_eq!(events[0].metadata.correlation_id, Some("req-456".to_string()));
}

#[tokio::test]
async fn rename_session_publishes_session_renamed_event() {
    // Arrange
    let repo = Arc::new(InMemorySessionRepository::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    // Create existing session
    let session = Session::new(UserId::new("user-1"), "Old Title".to_string()).unwrap();
    let session_id = session.id();
    repo.save(&session).await.unwrap();

    let handler = RenameSessionHandler::new(repo, event_bus.clone());

    let cmd = RenameSessionCommand {
        session_id,
        user_id: UserId::new("user-1"),
        new_title: "New Title".to_string(),
    };

    // Act
    handler.handle(cmd, CommandMetadata::default()).await.unwrap();

    // Assert
    let events = event_bus.events_of_type("session.renamed");
    assert_eq!(events.len(), 1);

    let payload: SessionRenamed = events[0].payload_as().unwrap();
    assert_eq!(payload.old_title, "Old Title");
    assert_eq!(payload.new_title, "New Title");
}

#[tokio::test]
async fn archive_session_publishes_session_archived_event() {
    // Arrange
    let repo = Arc::new(InMemorySessionRepository::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    let session = Session::new(UserId::new("user-1"), "Test".to_string()).unwrap();
    let session_id = session.id();
    repo.save(&session).await.unwrap();

    let handler = ArchiveSessionHandler::new(repo, event_bus.clone());

    let cmd = ArchiveSessionCommand {
        session_id,
        user_id: UserId::new("user-1"),
    };

    // Act
    handler.handle(cmd, CommandMetadata::default()).await.unwrap();

    // Assert
    let events = event_bus.events_of_type("session.archived");
    assert_eq!(events.len(), 1);

    let payload: SessionArchived = events[0].payload_as().unwrap();
    assert_eq!(payload.session_id, session_id);
}

#[tokio::test]
async fn unauthorized_rename_does_not_publish_event() {
    // Arrange
    let repo = Arc::new(InMemorySessionRepository::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    let session = Session::new(UserId::new("owner"), "Test".to_string()).unwrap();
    let session_id = session.id();
    repo.save(&session).await.unwrap();

    let handler = RenameSessionHandler::new(repo, event_bus.clone());

    let cmd = RenameSessionCommand {
        session_id,
        user_id: UserId::new("not-owner"), // Wrong user
        new_title: "Hacked".to_string(),
    };

    // Act
    let result = handler.handle(cmd, CommandMetadata::default()).await;

    // Assert - should fail
    assert!(result.is_err());

    // Assert - no event published
    assert_eq!(event_bus.event_count(), 0);
}
```

### Unit Tests: Event Handler

```rust
#[tokio::test]
async fn session_cycle_tracker_adds_cycle_to_session() {
    // Arrange
    let repo = Arc::new(InMemorySessionRepository::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    // Create session
    let session = Session::new(UserId::new("user-1"), "Test".to_string()).unwrap();
    let session_id = session.id();
    repo.save(&session).await.unwrap();

    let handler = SessionCycleTracker::new(repo.clone(), event_bus.clone());

    // Create CycleCreated event
    let cycle_id = CycleId::new();
    let event = EventEnvelope {
        event_id: EventId::new(),
        event_type: "cycle.created".to_string(),
        aggregate_id: cycle_id.to_string(),
        aggregate_type: "Cycle".to_string(),
        occurred_at: Timestamp::now(),
        payload: json!({
            "cycle_id": cycle_id.to_string(),
            "session_id": session_id.to_string(),
            "parent_cycle_id": null,
            "created_at": Timestamp::now().to_string(),
        }),
        metadata: EventMetadata::default(),
    };

    // Act
    handler.handle(event).await.unwrap();

    // Assert - session updated
    let updated_session = repo.find_by_id(session_id).await.unwrap().unwrap();
    assert!(updated_session.cycle_ids().contains(&cycle_id));

    // Assert - CycleAddedToSession event published
    let events = event_bus.events_of_type("session.cycle_added");
    assert_eq!(events.len(), 1);
}
```

---

## Integration Points

### Event Registration

```rust
// backend/src/main.rs or setup module

fn register_session_handlers(event_bus: &impl EventSubscriber, deps: &Dependencies) {
    // Session listens for cycle creation to maintain its cycle list
    event_bus.subscribe(
        "cycle.created",
        SessionCycleTracker::new(
            deps.session_repo.clone(),
            deps.event_publisher.clone(),
        ),
    );
}
```

### HTTP Handler Metadata

```rust
// backend/src/adapters/http/session/handlers.rs

pub async fn create_session(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(body): Json<CreateSessionRequest>,
) -> Result<Json<CreateSessionResponse>, ApiError> {
    let cmd = CreateSessionCommand {
        user_id: user.id,
        title: body.title,
        description: body.description,
    };

    // Extract correlation ID from request headers
    let metadata = CommandMetadata {
        correlation_id: extract_correlation_id(&headers),
    };

    let session_id = state.create_session_handler
        .handle(cmd, metadata)
        .await?;

    Ok(Json(CreateSessionResponse { id: session_id }))
}
```

---

## Dependencies

### Crate Dependencies

```toml
# No new dependencies - uses foundation event types
```

### Module Dependencies

- `foundation::events` - EventId, EventEnvelope, DomainEvent
- `foundation::ids` - SessionId, CycleId, UserId
- `foundation::timestamp` - Timestamp
- `ports::event_publisher` - EventPublisher trait

---

## Related Documents

- **Integration Spec:** features/integrations/full-proact-journey.md
- **Phase 1:** features/foundation/event-infrastructure.md
- **Checklist:** REQUIREMENTS/CHECKLIST-events.md (Phase 2)
- **Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md

---

*Version: 1.0.0*
*Created: 2026-01-07*
*Phase: 2 of 8*
