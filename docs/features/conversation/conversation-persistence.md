# Conversation Persistence & HTTP Layer

**Module:** conversation
**Type:** Feature Specification
**Priority:** P1 (Required for MVP)
**Last Updated:** 2026-01-10

> Persistence layer, HTTP adapters, and WebSocket handlers for the conversation module.

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Required |
| Authorization Model | User must own parent session via session ownership chain |
| Sensitive Data | Messages (Confidential), extracted data (Confidential) |
| Rate Limiting | Required - per user, per session |
| Audit Logging | Message creation, state transitions, extraction events |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| Message content | Confidential | Encrypt at rest, do not log content |
| Extracted data | Confidential | Encrypt at rest, validate schema |
| Conversation ID | Internal | Safe to log |
| State/Phase | Internal | Safe to log |
| Token counts | Internal | Safe to log |

### Security Events to Log

- Conversation creation (component_id, user_id)
- State transitions (old_state -> new_state)
- Authorization failures (user_id, attempted resource)
- Rate limit violations

---

## Overview

This feature implements the persistence layer and HTTP/WebSocket handlers for conversations. It builds on the existing domain types (ConversationState, AgentPhase, etc.) to provide full CRUD operations and real-time streaming.

### Dependencies

- `foundation` - IDs, timestamps, errors
- `proact-types` - ComponentType
- `session` - Session ownership verification
- `cycle` - Component ownership verification

### Architectural Pattern

```
┌─────────────────────────────────────────────────────────────────┐
│                        HTTP Layer                                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ REST Routes │  │ WebSocket   │  │ DTOs                    │  │
│  │             │  │ Handler     │  │ (Request/Response)      │  │
│  └──────┬──────┘  └──────┬──────┘  └─────────────────────────┘  │
└─────────┼────────────────┼──────────────────────────────────────┘
          │                │
          ▼                ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Application Layer                            │
│  ┌─────────────────────┐  ┌─────────────────────────────────┐   │
│  │ Commands            │  │ Queries                         │   │
│  │ - SendMessage       │  │ - GetConversation               │   │
│  │ - RegenerateResponse│  │ - GetConversationHistory        │   │
│  └──────────┬──────────┘  └──────────────┬──────────────────┘   │
└─────────────┼────────────────────────────┼──────────────────────┘
              │                            │
              ▼                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Ports                                     │
│  ┌─────────────────────────┐  ┌─────────────────────────────┐   │
│  │ ConversationRepository  │  │ ConversationReader          │   │
│  │ (write)                 │  │ (read)                      │   │
│  └──────────┬──────────────┘  └──────────────┬──────────────┘   │
└─────────────┼────────────────────────────────┼──────────────────┘
              │                                │
              ▼                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Postgres Adapters                             │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ PostgresConversationRepository / Reader                  │    │
│  └──────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

---

## Domain Extensions

### Conversation Entity

```rust
/// A conversation within a PrOACT component.
pub struct Conversation {
    /// Unique identifier.
    id: ConversationId,
    /// The component this conversation belongs to.
    component_id: ComponentId,
    /// Type of component (for phase behavior).
    component_type: ComponentType,
    /// Current state.
    state: ConversationState,
    /// Current agent phase.
    phase: AgentPhase,
    /// System prompt for this conversation.
    system_prompt: String,
    /// Messages in the conversation.
    messages: Vec<Message>,
    /// Extracted structured data (if any).
    extracted_data: Option<serde_json::Value>,
    /// Owner of this conversation.
    user_id: UserId,
    /// When created.
    created_at: Timestamp,
    /// When last updated.
    updated_at: Timestamp,
}
```

### ConversationId

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConversationId(Uuid);
```

### Message Entity

```rust
/// A message in a conversation.
pub struct Message {
    /// Unique identifier.
    id: MessageId,
    /// Who sent this message.
    role: MessageRole,
    /// Content of the message.
    content: String,
    /// Token count (for budget tracking).
    token_count: Option<u32>,
    /// When created.
    created_at: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}
```

---

## Ports

### ConversationRepository (Write)

```rust
#[async_trait]
pub trait ConversationRepository: Send + Sync {
    /// Creates a new conversation.
    async fn create(&self, conversation: &Conversation) -> Result<(), RepositoryError>;

    /// Updates an existing conversation.
    async fn update(&self, conversation: &Conversation) -> Result<(), RepositoryError>;

    /// Adds a message to a conversation.
    async fn add_message(
        &self,
        conversation_id: &ConversationId,
        message: Message,
    ) -> Result<(), RepositoryError>;

    /// Updates extracted data.
    async fn update_extracted_data(
        &self,
        conversation_id: &ConversationId,
        data: serde_json::Value,
    ) -> Result<(), RepositoryError>;

    /// Finds by component ID.
    async fn find_by_component(
        &self,
        component_id: &ComponentId,
    ) -> Result<Option<Conversation>, RepositoryError>;
}
```

### ConversationReader (Read)

```rust
#[async_trait]
pub trait ConversationReader: Send + Sync {
    /// Gets a conversation by ID.
    async fn get(&self, id: &ConversationId) -> Result<Option<ConversationView>, ReaderError>;

    /// Gets conversation by component ID.
    async fn get_by_component(
        &self,
        component_id: &ComponentId,
    ) -> Result<Option<ConversationView>, ReaderError>;

    /// Gets messages for a conversation with pagination.
    async fn get_messages(
        &self,
        conversation_id: &ConversationId,
        pagination: Pagination,
    ) -> Result<Page<MessageView>, ReaderError>;
}
```

---

## Application Commands

### SendMessageCommand

```rust
pub struct SendMessageCommand {
    pub component_id: ComponentId,
    pub user_id: UserId,
    pub content: String,
}

pub struct SendMessageResult {
    pub user_message_id: MessageId,
    pub assistant_message_id: MessageId,
    pub new_phase: AgentPhase,
    pub new_state: ConversationState,
    pub usage: TokenUsage,
}
```

### RegenerateResponseCommand

```rust
pub struct RegenerateResponseCommand {
    pub conversation_id: ConversationId,
    pub user_id: UserId,
}

pub struct RegenerateResult {
    pub new_message_id: MessageId,
    pub content: String,
    pub usage: TokenUsage,
}
```

---

## HTTP Endpoints

### REST Endpoints

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/api/components/{id}/conversation` | `get_conversation` | Get conversation for component |
| GET | `/api/conversations/{id}/messages` | `get_messages` | Get messages with pagination |
| POST | `/api/components/{id}/conversation/regenerate` | `regenerate` | Regenerate last response |

### WebSocket Endpoint

```
ws://{host}/api/components/{id}/stream
```

**Messages:** See `docs/api/streaming-protocol.md`

---

## Database Schema

### conversations Table

```sql
CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id UUID NOT NULL UNIQUE REFERENCES components(id) ON DELETE CASCADE,
    component_type VARCHAR(50) NOT NULL,
    state VARCHAR(20) NOT NULL DEFAULT 'initializing',
    phase VARCHAR(20) NOT NULL DEFAULT 'intro',
    system_prompt TEXT NOT NULL,
    extracted_data JSONB,
    user_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_conversations_component_id ON conversations(component_id);
CREATE INDEX idx_conversations_user_id ON conversations(user_id);
```

### messages Table

```sql
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL,
    content TEXT NOT NULL,
    token_count INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_messages_conversation_id ON messages(conversation_id);
CREATE INDEX idx_messages_created_at ON messages(conversation_id, created_at);
```

---

## Tasks

- [ ] Create ConversationId value object in `backend/src/domain/conversation/conversation_id.rs`
- [ ] Create Conversation entity in `backend/src/domain/conversation/conversation.rs`
- [ ] Create ConversationError enum in `backend/src/domain/conversation/errors.rs`
- [ ] Create ConversationRepository port in `backend/src/ports/conversation_repository.rs`
- [ ] Create ConversationReader port in `backend/src/ports/conversation_reader.rs`
- [ ] Create SendMessageCommand handler in `backend/src/application/commands/`
- [ ] Create GetConversation query handler in `backend/src/application/queries/`
- [ ] Create PostgreSQL migration for conversations and messages tables
- [ ] Implement PostgresConversationRepository adapter
- [ ] Implement PostgresConversationReader adapter
- [ ] Create HTTP handlers in `backend/src/adapters/http/conversation/`
- [ ] Create WebSocket handler for streaming
- [ ] Write unit tests for domain entities
- [ ] Write integration tests for repository
- [ ] Write integration tests for HTTP endpoints

---

## Related Documents

- **Archived Spec:** `docs/features/conversation/conversation-lifecycle.md`
- **Module Checklist:** `REQUIREMENTS/CHECKLIST-conversation.md`
- **Streaming Protocol:** `docs/api/streaming-protocol.md`
- **System Architecture:** `docs/architecture/SYSTEM-ARCHITECTURE.md`

---

*Version: 1.0.0*
*Created: 2026-01-10*
