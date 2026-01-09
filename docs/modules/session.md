# Session Module Specification

## Overview

The Session module manages the top-level Decision Session - the container for all cycles exploring a single decision context. Each session belongs to a user and can contain multiple cycles (including branches).

---

## Module Classification

| Attribute | Value |
|-----------|-------|
| **Type** | Full Module (Ports + Adapters) |
| **Language** | Rust |
| **Responsibility** | Session lifecycle, user ownership, cycle references |
| **Domain Dependencies** | foundation |
| **External Dependencies** | `async-trait`, `sqlx`, `tokio` |

---

## Architecture

### Hexagonal Structure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           SESSION MODULE                                     │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         DOMAIN LAYER                                    │ │
│  │                                                                         │ │
│  │   ┌────────────────────────────────────────────────────────────────┐   │ │
│  │   │                    Session Aggregate                            │   │ │
│  │   │                                                                 │   │ │
│  │   │   - id: SessionId                                               │   │ │
│  │   │   - user_id: UserId                                             │   │ │
│  │   │   - title: String                                               │   │ │
│  │   │   - description: Option<String>                                 │   │ │
│  │   │   - status: SessionStatus                                       │   │ │
│  │   │   - cycle_ids: Vec<CycleId>  (references only)                  │   │ │
│  │   │   - created_at, updated_at: Timestamp                           │   │ │
│  │   │                                                                 │   │ │
│  │   │   + new(user_id, title) -> Result<Session>                      │   │ │
│  │   │   + rename(title) -> Result<()>                                 │   │ │
│  │   │   + add_cycle(cycle_id) -> Result<()>                           │   │ │
│  │   │   + archive() -> Result<()>                                     │   │ │
│  │   └────────────────────────────────────────────────────────────────┘   │ │
│  │                                                                         │ │
│  │   ┌─────────────────────────────────────────────────────────────────┐  │ │
│  │   │                   Domain Events                                  │  │ │
│  │   │   SessionCreated, SessionRenamed, CycleAddedToSession,           │  │ │
│  │   │   SessionArchived                                                │  │ │
│  │   └─────────────────────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
│                                      ▼                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                          PORT LAYER                                     │ │
│  │                                                                         │ │
│  │   ┌─────────────────────────────┐  ┌─────────────────────────────────┐ │ │
│  │   │   SessionRepository         │  │   SessionReader                  │ │ │
│  │   │   (Write operations)        │  │   (Query operations - CQRS)     │ │ │
│  │   │                             │  │                                  │ │ │
│  │   │   + save(session)           │  │   + get_by_id(id) -> SessionView │ │ │
│  │   │   + update(session)         │  │   + list_by_user(filter)         │ │ │
│  │   │   + find_by_id(id)          │  │   + search(query)                │ │ │
│  │   └─────────────────────────────┘  └─────────────────────────────────┘ │ │
│  │                                                                         │ │
│  │   ┌─────────────────────────────────────────────────────────────────┐  │ │
│  │   │                DomainEventPublisher                              │  │ │
│  │   │   + publish(events: Vec<DomainEvent>)                            │  │ │
│  │   └─────────────────────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
│                                      ▼                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                        ADAPTER LAYER                                    │ │
│  │                                                                         │ │
│  │   ┌─────────────────┐  ┌─────────────────┐  ┌──────────────────────┐   │ │
│  │   │ PostgresSession │  │ PostgresSession │  │ RedisEventPublisher  │   │ │
│  │   │ Repository      │  │ Reader          │  │                      │   │ │
│  │   └─────────────────┘  └─────────────────┘  └──────────────────────┘   │ │
│  │                                                                         │ │
│  │   ┌─────────────────────────────────────────────────────────────────┐  │ │
│  │   │                    HTTP Handlers                                 │  │ │
│  │   │   POST /sessions, GET /sessions, GET /sessions/:id, etc.         │  │ │
│  │   └─────────────────────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Domain Layer

### Session Aggregate

```rust
use crate::foundation::{
    CycleId, DomainError, ErrorCode, SessionId, SessionStatus, Timestamp, UserId,
};

/// The Session aggregate - container for decision cycles
#[derive(Debug, Clone)]
pub struct Session {
    id: SessionId,
    user_id: UserId,
    title: String,
    description: Option<String>,
    status: SessionStatus,
    cycle_ids: Vec<CycleId>,
    created_at: Timestamp,
    updated_at: Timestamp,
    domain_events: Vec<SessionEvent>,
}

impl Session {
    /// Creates a new session for a user
    pub fn new(user_id: UserId, title: impl Into<String>) -> Result<Self, DomainError> {
        let title = title.into();
        Self::validate_title(&title)?;

        let now = Timestamp::now();
        let id = SessionId::new();

        let mut session = Self {
            id,
            user_id: user_id.clone(),
            title: title.clone(),
            description: None,
            status: SessionStatus::Active,
            cycle_ids: Vec::new(),
            created_at: now,
            updated_at: now,
            domain_events: Vec::new(),
        };

        session.record_event(SessionEvent::Created {
            session_id: id,
            user_id,
            title,
            created_at: now,
        });

        Ok(session)
    }

    /// Reconstitutes a session from persistence (no events emitted)
    pub fn reconstitute(
        id: SessionId,
        user_id: UserId,
        title: String,
        description: Option<String>,
        status: SessionStatus,
        cycle_ids: Vec<CycleId>,
        created_at: Timestamp,
        updated_at: Timestamp,
    ) -> Self {
        Self {
            id,
            user_id,
            title,
            description,
            status,
            cycle_ids,
            created_at,
            updated_at,
            domain_events: Vec::new(),
        }
    }

    // === Accessors ===

    pub fn id(&self) -> SessionId {
        self.id
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn status(&self) -> SessionStatus {
        self.status
    }

    pub fn cycle_ids(&self) -> &[CycleId] {
        &self.cycle_ids
    }

    pub fn cycle_count(&self) -> usize {
        self.cycle_ids.len()
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    // === Authorization ===

    /// Checks if the given user is the owner of this session
    pub fn is_owner(&self, user_id: &UserId) -> bool {
        &self.user_id == user_id
    }

    /// Ensures the user is the owner, returning error if not
    pub fn authorize(&self, user_id: &UserId) -> Result<(), DomainError> {
        if self.is_owner(user_id) {
            Ok(())
        } else {
            Err(DomainError::new(
                ErrorCode::Forbidden,
                "User is not the owner of this session",
            ))
        }
    }

    // === Mutations ===

    /// Renames the session
    pub fn rename(&mut self, new_title: impl Into<String>) -> Result<(), DomainError> {
        self.ensure_mutable()?;
        let new_title = new_title.into();
        Self::validate_title(&new_title)?;

        let old_title = std::mem::replace(&mut self.title, new_title.clone());
        self.updated_at = Timestamp::now();

        self.record_event(SessionEvent::Renamed {
            session_id: self.id,
            old_title,
            new_title,
        });

        Ok(())
    }

    /// Updates the session description
    pub fn update_description(&mut self, description: Option<String>) -> Result<(), DomainError> {
        self.ensure_mutable()?;
        self.description = description;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Adds a cycle reference to this session
    pub fn add_cycle(&mut self, cycle_id: CycleId) -> Result<(), DomainError> {
        self.ensure_mutable()?;

        // Prevent duplicates
        if self.cycle_ids.contains(&cycle_id) {
            return Ok(());
        }

        self.cycle_ids.push(cycle_id);
        self.updated_at = Timestamp::now();

        self.record_event(SessionEvent::CycleAdded {
            session_id: self.id,
            cycle_id,
        });

        Ok(())
    }

    /// Archives the session (soft delete)
    pub fn archive(&mut self) -> Result<(), DomainError> {
        if !self.status.can_transition_to(&SessionStatus::Archived) {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                "Session is already archived",
            ));
        }

        self.status = SessionStatus::Archived;
        self.updated_at = Timestamp::now();

        self.record_event(SessionEvent::Archived {
            session_id: self.id,
        });

        Ok(())
    }

    // === Domain Events ===

    /// Pulls and clears all pending domain events
    pub fn pull_domain_events(&mut self) -> Vec<SessionEvent> {
        std::mem::take(&mut self.domain_events)
    }

    // === Private Helpers ===

    fn validate_title(title: &str) -> Result<(), DomainError> {
        if title.trim().is_empty() {
            return Err(DomainError::new(
                ErrorCode::ValidationFailed,
                "Session title cannot be empty",
            ));
        }
        if title.len() > 500 {
            return Err(DomainError::new(
                ErrorCode::ValidationFailed,
                "Session title cannot exceed 500 characters",
            ));
        }
        Ok(())
    }

    fn ensure_mutable(&self) -> Result<(), DomainError> {
        if !self.status.is_mutable() {
            return Err(DomainError::new(
                ErrorCode::SessionArchived,
                "Cannot modify an archived session",
            ));
        }
        Ok(())
    }

    fn record_event(&mut self, event: SessionEvent) {
        self.domain_events.push(event);
    }
}
```

### Domain Events

```rust
use crate::foundation::{CycleId, SessionId, Timestamp, UserId};
use serde::{Deserialize, Serialize};

/// Events emitted by the Session aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionEvent {
    Created {
        session_id: SessionId,
        user_id: UserId,
        title: String,
        created_at: Timestamp,
    },
    Renamed {
        session_id: SessionId,
        old_title: String,
        new_title: String,
    },
    CycleAdded {
        session_id: SessionId,
        cycle_id: CycleId,
    },
    Archived {
        session_id: SessionId,
    },
}

impl SessionEvent {
    pub fn session_id(&self) -> SessionId {
        match self {
            SessionEvent::Created { session_id, .. } => *session_id,
            SessionEvent::Renamed { session_id, .. } => *session_id,
            SessionEvent::CycleAdded { session_id, .. } => *session_id,
            SessionEvent::Archived { session_id } => *session_id,
        }
    }

    pub fn event_type(&self) -> &'static str {
        match self {
            SessionEvent::Created { .. } => "session.created",
            SessionEvent::Renamed { .. } => "session.renamed",
            SessionEvent::CycleAdded { .. } => "session.cycle_added",
            SessionEvent::Archived { .. } => "session.archived",
        }
    }
}
```

---

## Ports

### SessionRepository (Write)

```rust
use async_trait::async_trait;
use crate::foundation::SessionId;
use super::Session;

/// Repository port for Session aggregate persistence (write side)
#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Persists a new session
    async fn save(&self, session: &Session) -> Result<(), RepositoryError>;

    /// Updates an existing session
    async fn update(&self, session: &Session) -> Result<(), RepositoryError>;

    /// Finds a session by ID for modification
    async fn find_by_id(&self, id: SessionId) -> Result<Option<Session>, RepositoryError>;

    /// Checks if a session exists
    async fn exists(&self, id: SessionId) -> Result<bool, RepositoryError>;
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Session not found: {0}")]
    NotFound(SessionId),

    #[error("Duplicate session ID: {0}")]
    DuplicateId(SessionId),

    #[error("Concurrency conflict on session: {0}")]
    ConcurrencyConflict(SessionId),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
```

### SessionReader (Query - CQRS)

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::foundation::{SessionId, SessionStatus, UserId};

/// Read-only port for session queries (CQRS query side)
#[async_trait]
pub trait SessionReader: Send + Sync {
    /// Gets a session view by ID
    async fn get_by_id(&self, id: SessionId) -> Result<Option<SessionView>, ReaderError>;

    /// Lists sessions for a user with filtering/pagination
    async fn list_by_user(
        &self,
        user_id: &UserId,
        filter: SessionFilter,
    ) -> Result<Vec<SessionSummary>, ReaderError>;

    /// Searches sessions by title/description
    async fn search(
        &self,
        user_id: &UserId,
        query: &str,
    ) -> Result<Vec<SessionSummary>, ReaderError>;

    /// Gets total count for pagination
    async fn count_by_user(
        &self,
        user_id: &UserId,
        filter: &SessionFilter,
    ) -> Result<u64, ReaderError>;
}

/// Detailed session view for single-session display
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionView {
    pub id: SessionId,
    pub title: String,
    pub description: Option<String>,
    pub status: SessionStatus,
    pub cycle_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Summary view for session lists
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionSummary {
    pub id: SessionId,
    pub title: String,
    pub status: SessionStatus,
    pub cycle_count: usize,
    pub updated_at: DateTime<Utc>,
}

/// Filter options for listing sessions
#[derive(Debug, Clone, Default)]
pub struct SessionFilter {
    pub status: Option<SessionStatus>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub order_by: Option<SessionOrderBy>,
}

#[derive(Debug, Clone, Copy)]
pub enum SessionOrderBy {
    UpdatedAtDesc,
    UpdatedAtAsc,
    CreatedAtDesc,
    CreatedAtAsc,
    TitleAsc,
    TitleDesc,
}

impl Default for SessionOrderBy {
    fn default() -> Self {
        SessionOrderBy::UpdatedAtDesc
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReaderError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

### DomainEventPublisher

```rust
use async_trait::async_trait;
use super::SessionEvent;

/// Port for publishing domain events
#[async_trait]
pub trait DomainEventPublisher: Send + Sync {
    /// Publishes domain events (fire-and-forget)
    async fn publish(&self, events: Vec<SessionEvent>) -> Result<(), PublishError>;
}

#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error("Failed to publish event: {0}")]
    PublishFailed(String),
}
```

---

## Application Layer

### Commands

#### CreateSession

```rust
use crate::foundation::UserId;
use crate::ports::{SessionRepository, DomainEventPublisher};
use crate::domain::Session;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CreateSessionCommand {
    pub user_id: UserId,
    pub title: String,
    pub description: Option<String>,
}

pub struct CreateSessionHandler {
    repo: Arc<dyn SessionRepository>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl CreateSessionHandler {
    pub fn new(
        repo: Arc<dyn SessionRepository>,
        publisher: Arc<dyn DomainEventPublisher>,
    ) -> Self {
        Self { repo, publisher }
    }

    pub async fn handle(&self, cmd: CreateSessionCommand) -> Result<SessionId, CommandError> {
        // Create aggregate
        let mut session = Session::new(cmd.user_id, cmd.title)?;

        // Set optional description
        if let Some(desc) = cmd.description {
            session.update_description(Some(desc))?;
        }

        // Persist
        self.repo.save(&session).await?;

        // Publish events
        let events = session.pull_domain_events();
        self.publisher.publish(events).await?;

        Ok(session.id())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Publish error: {0}")]
    Publish(#[from] PublishError),

    #[error("Session not found: {0}")]
    NotFound(SessionId),

    #[error("Unauthorized")]
    Unauthorized,
}
```

#### RenameSession

```rust
#[derive(Debug, Clone)]
pub struct RenameSessionCommand {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub new_title: String,
}

pub struct RenameSessionHandler {
    repo: Arc<dyn SessionRepository>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl RenameSessionHandler {
    pub fn new(
        repo: Arc<dyn SessionRepository>,
        publisher: Arc<dyn DomainEventPublisher>,
    ) -> Self {
        Self { repo, publisher }
    }

    pub async fn handle(&self, cmd: RenameSessionCommand) -> Result<(), CommandError> {
        // Load aggregate
        let mut session = self.repo
            .find_by_id(cmd.session_id)
            .await?
            .ok_or(CommandError::NotFound(cmd.session_id))?;

        // Authorize
        session.authorize(&cmd.user_id)
            .map_err(|_| CommandError::Unauthorized)?;

        // Execute business logic
        session.rename(cmd.new_title)?;

        // Persist
        self.repo.update(&session).await?;

        // Publish events
        let events = session.pull_domain_events();
        self.publisher.publish(events).await?;

        Ok(())
    }
}
```

#### ArchiveSession

```rust
#[derive(Debug, Clone)]
pub struct ArchiveSessionCommand {
    pub session_id: SessionId,
    pub user_id: UserId,
}

pub struct ArchiveSessionHandler {
    repo: Arc<dyn SessionRepository>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl ArchiveSessionHandler {
    pub fn new(
        repo: Arc<dyn SessionRepository>,
        publisher: Arc<dyn DomainEventPublisher>,
    ) -> Self {
        Self { repo, publisher }
    }

    pub async fn handle(&self, cmd: ArchiveSessionCommand) -> Result<(), CommandError> {
        let mut session = self.repo
            .find_by_id(cmd.session_id)
            .await?
            .ok_or(CommandError::NotFound(cmd.session_id))?;

        session.authorize(&cmd.user_id)
            .map_err(|_| CommandError::Unauthorized)?;

        session.archive()?;

        self.repo.update(&session).await?;

        let events = session.pull_domain_events();
        self.publisher.publish(events).await?;

        Ok(())
    }
}
```

### Queries

#### GetSession

```rust
use crate::ports::{SessionReader, SessionView};

#[derive(Debug, Clone)]
pub struct GetSessionQuery {
    pub session_id: SessionId,
    pub user_id: UserId,
}

pub struct GetSessionHandler {
    reader: Arc<dyn SessionReader>,
    repo: Arc<dyn SessionRepository>,
}

impl GetSessionHandler {
    pub fn new(
        reader: Arc<dyn SessionReader>,
        repo: Arc<dyn SessionRepository>,
    ) -> Self {
        Self { reader, repo }
    }

    pub async fn handle(&self, query: GetSessionQuery) -> Result<SessionView, QueryError> {
        // SECURITY: Load session via repository to get owner for authorization check
        let session = self.repo
            .find_by_id(query.session_id)
            .await
            .map_err(|e| QueryError::Repository(e.to_string()))?
            .ok_or(QueryError::NotFound(query.session_id))?;

        // CRITICAL: Verify ownership to prevent IDOR (A01 Broken Access Control)
        if session.user_id() != &query.user_id {
            return Err(QueryError::Forbidden);
        }

        // Now safe to return the view
        let view = self.reader
            .get_by_id(query.session_id)
            .await?
            .ok_or(QueryError::NotFound(query.session_id))?;

        Ok(view)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Session not found: {0}")]
    NotFound(SessionId),

    #[error("Access denied")]
    Forbidden,

    #[error("Reader error: {0}")]
    Reader(#[from] ReaderError),

    #[error("Repository error: {0}")]
    Repository(String),
}
```

#### ListUserSessions

```rust
#[derive(Debug, Clone)]
pub struct ListUserSessionsQuery {
    pub user_id: UserId,
    pub filter: SessionFilter,
}

pub struct ListUserSessionsHandler {
    reader: Arc<dyn SessionReader>,
}

impl ListUserSessionsHandler {
    pub fn new(reader: Arc<dyn SessionReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(&self, query: ListUserSessionsQuery) -> Result<SessionListResult, QueryError> {
        let sessions = self.reader
            .list_by_user(&query.user_id, query.filter.clone())
            .await?;

        let total = self.reader
            .count_by_user(&query.user_id, &query.filter)
            .await?;

        Ok(SessionListResult { sessions, total })
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionListResult {
    pub sessions: Vec<SessionSummary>,
    pub total: u64,
}
```

---

## Adapters

### HTTP Endpoints

| Method | Path | Handler | Auth | Description |
|--------|------|---------|------|-------------|
| `POST` | `/api/sessions` | CreateSession | Required | Create a new session |
| `GET` | `/api/sessions` | ListUserSessions | Required | List user's sessions |
| `GET` | `/api/sessions/:id` | GetSession | Owner | Get session details |
| `PATCH` | `/api/sessions/:id` | RenameSession | Owner | Update session title |
| `DELETE` | `/api/sessions/:id` | ArchiveSession | Owner | Archive session |

#### Request/Response DTOs

```rust
use serde::{Deserialize, Serialize};

// === Requests ===

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RenameSessionRequest {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct ListSessionsQuery {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,
    #[serde(default)]
    pub order_by: Option<String>,
}

// === Responses ===

#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub cycle_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionSummaryResponse>,
    pub total: u64,
}

#[derive(Debug, Serialize)]
pub struct SessionSummaryResponse {
    pub id: String,
    pub title: String,
    pub status: String,
    pub cycle_count: usize,
    pub updated_at: String,
}
```

### Database Schema

```sql
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT sessions_title_not_empty CHECK (title <> ''),
    CONSTRAINT sessions_status_valid CHECK (status IN ('active', 'archived'))
);

-- Indexes for common query patterns
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_updated_at ON sessions(updated_at DESC);
CREATE INDEX idx_sessions_user_status ON sessions(user_id, status);

-- Full-text search on title and description
CREATE INDEX idx_sessions_search ON sessions
    USING gin(to_tsvector('english', title || ' ' || COALESCE(description, '')));
```

### PostgresSessionRepository

```rust
use sqlx::PgPool;
use async_trait::async_trait;
use crate::ports::{SessionRepository, RepositoryError};
use crate::domain::Session;
use crate::foundation::SessionId;

pub struct PostgresSessionRepository {
    pool: PgPool,
}

impl PostgresSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for PostgresSessionRepository {
    async fn save(&self, session: &Session) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO sessions (id, user_id, title, description, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            session.id().as_uuid(),
            session.user_id().as_str(),
            session.title(),
            session.description(),
            session.status().to_string(),
            session.created_at().as_datetime(),
            session.updated_at().as_datetime(),
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update(&self, session: &Session) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            UPDATE sessions
            SET title = $2, description = $3, status = $4, updated_at = $5
            WHERE id = $1
            "#,
            session.id().as_uuid(),
            session.title(),
            session.description(),
            session.status().to_string(),
            session.updated_at().as_datetime(),
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(session.id()));
        }

        Ok(())
    }

    async fn find_by_id(&self, id: SessionId) -> Result<Option<Session>, RepositoryError> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, title, description, status, created_at, updated_at
            FROM sessions
            WHERE id = $1
            "#,
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => {
                // Also fetch cycle_ids
                let cycle_ids = sqlx::query_scalar!(
                    r#"SELECT id FROM cycles WHERE session_id = $1"#,
                    id.as_uuid()
                )
                .fetch_all(&self.pool)
                .await?;

                let session = Session::reconstitute(
                    SessionId::from_uuid(r.id),
                    UserId::new(r.user_id).unwrap(),
                    r.title,
                    r.description,
                    r.status.parse().unwrap(),
                    cycle_ids.into_iter().map(CycleId::from_uuid).collect(),
                    Timestamp::from_datetime(r.created_at),
                    Timestamp::from_datetime(r.updated_at),
                );
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    async fn exists(&self, id: SessionId) -> Result<bool, RepositoryError> {
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM sessions WHERE id = $1)"#,
            id.as_uuid()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(exists.unwrap_or(false))
    }
}
```

---

## File Structure

```
backend/src/domain/session/
├── mod.rs                  # Module exports
├── session.rs              # Session aggregate
├── session_test.rs         # Aggregate tests
├── events.rs               # SessionEvent enum
└── errors.rs               # Session-specific errors

backend/src/ports/
├── session_repository.rs   # SessionRepository trait
└── session_reader.rs       # SessionReader trait

backend/src/application/
├── commands/
│   ├── create_session.rs
│   ├── create_session_test.rs
│   ├── rename_session.rs
│   ├── rename_session_test.rs
│   ├── archive_session.rs
│   └── archive_session_test.rs
└── queries/
    ├── get_session.rs
    ├── get_session_test.rs
    ├── list_sessions.rs
    └── list_sessions_test.rs

backend/src/adapters/
├── http/session/
│   ├── handlers.rs
│   ├── handlers_test.rs
│   ├── dto.rs
│   └── routes.rs
└── postgres/
    ├── session_repository.rs
    ├── session_repository_test.rs
    ├── session_reader.rs
    └── session_reader_test.rs

frontend/src/modules/session/
├── domain/
│   ├── session.ts
│   └── session.test.ts
├── api/
│   ├── session-api.ts
│   ├── use-sessions.ts
│   └── use-session.ts
├── components/
│   ├── SessionList.tsx
│   ├── SessionList.test.tsx
│   ├── SessionCard.tsx
│   ├── SessionCard.test.tsx
│   └── CreateSessionDialog.tsx
└── index.ts
```

---

## Invariants

| Invariant | Enforcement |
|-----------|-------------|
| Title is non-empty | Constructor and rename validation |
| Title max 500 chars | Validation in validate_title() |
| Only owner can modify | authorize() check in commands |
| Archived sessions immutable | ensure_mutable() check |
| CycleIDs are unique in session | Duplicate check in add_cycle() |
| Status transitions are valid | can_transition_to() check |

---

## Test Categories

### Unit Tests (Domain)

| Category | Example Tests |
|----------|---------------|
| Creation | `new_session_has_active_status` |
| Creation | `new_session_requires_non_empty_title` |
| Rename | `rename_updates_title_and_timestamp` |
| Rename | `archived_session_cannot_be_renamed` |
| Ownership | `is_owner_returns_true_for_owner` |
| Archive | `archive_changes_status_to_archived` |
| Events | `new_session_emits_created_event` |
| Events | `rename_emits_renamed_event` |

### Integration Tests (Repository)

| Category | Example Tests |
|----------|---------------|
| Save | `save_persists_session_to_database` |
| Update | `update_modifies_existing_session` |
| Find | `find_by_id_returns_session_with_cycles` |
| List | `list_by_user_filters_by_status` |
| Search | `search_finds_sessions_by_title` |

### API Tests (HTTP)

| Category | Example Tests |
|----------|---------------|
| Create | `post_sessions_creates_session` |
| Create | `post_sessions_returns_400_for_empty_title` |
| Get | `get_session_returns_404_for_missing` |
| List | `get_sessions_returns_paginated_list` |
| Auth | `endpoints_require_authentication` |

---

*Module Version: 1.0.0*
*Based on: SYSTEM-ARCHITECTURE.md v1.1.0*
*Language: Rust*
