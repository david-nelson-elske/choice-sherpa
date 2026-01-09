# Feature: Conversation Domain Events

**Module:** conversation
**Type:** Event Publishing + Event Handling
**Priority:** P0
**Phase:** 4 of Full PrOACT Journey Integration
**Depends On:** features/cycle/cycle-events.md

> Conversation module publishes events for AI interactions and subscribes to component events for automatic initialization.

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Required |
| Authorization Model | User must own the parent session to access conversation |
| Sensitive Data | Message content (Confidential), AI responses (Confidential) |
| Rate Limiting | Required - per-user message send rate |
| Audit Logging | Message sent, conversation started/ended, data extraction |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| Message content | Confidential | Encrypt at rest, never log full content |
| content_preview | Confidential | **MUST NOT be logged** - contains user decision data |
| AI responses | Confidential | Sanitize before storage, encrypt at rest |
| conversation_id | Internal | Safe to log |
| component_id | Internal | Safe to log |
| session_id | Internal | Safe to log |
| tokens_used | Internal | Safe to log |

### Security Events to Log

- `conversation.started` - Log conversation_id, component_id, session_id (no content)
- `message.sent` - Log message_id, role, timestamp (NO content_preview)
- `conversation.ended` - Log conversation_id, total_messages, total_tokens
- `data_extracted` - Log extraction_summary, item_count (no raw extracted data)
- Authorization failures - Log user_id, attempted session_id, reason

### Critical Security Notes

1. **NEVER log `content_preview`** - This field contains user decision data which is Confidential
2. Events should include IDs and metadata only, not message content
3. AI responses must be sanitized to remove any injected content before storage
4. WebSocket broadcasts must verify session ownership before delivery

---

## Problem Statement

The conversation module operates in isolation, requiring manual coordination:
- No automatic conversation initialization when components start
- Dashboard can't show real-time chat updates
- No visibility into AI response generation
- Structured data extraction isn't communicated to other modules

### Current State

- Conversations must be manually created
- No real-time message visibility
- Extraction happens silently

### Desired State

- Conversations auto-initialize when components start (via event subscription)
- Messages stream to dashboard in real-time
- Structured data extraction publishes events for component updates

---

## Domain Events

### ConversationStarted

Published when a new conversation is created for a component.

```rust
// backend/src/domain/conversation/events.rs

use serde::{Deserialize, Serialize};

/// Published when a conversation is started for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationStarted {
    pub event_id: EventId,
    pub conversation_id: ConversationId,
    pub component_id: ComponentId,
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    pub component_type: ComponentType,
    pub started_at: Timestamp,
}

impl DomainEvent for ConversationStarted {
    fn event_type(&self) -> &'static str {
        "conversation.started"
    }

    fn aggregate_id(&self) -> String {
        self.conversation_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.started_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

**Subscribers:**
- `DashboardUpdateHandler` - Show conversation panel active

---

### MessageSent

Published when a message is added to the conversation (user or assistant).

```rust
/// Published when a message is sent in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSent {
    pub event_id: EventId,
    pub conversation_id: ConversationId,
    pub message_id: MessageId,
    pub component_id: ComponentId,
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    pub component_type: ComponentType,
    pub role: Role,
    /// First 200 characters of message content (for preview)
    pub content_preview: String,
    pub sent_at: Timestamp,
}

impl DomainEvent for MessageSent {
    fn event_type(&self) -> &'static str {
        "message.sent"
    }

    fn aggregate_id(&self) -> String {
        self.conversation_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.sent_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

**Subscribers:**
- `DashboardUpdateHandler` - Show new message in chat view
- `WebSocketEventBridge` - Push to connected clients

---

### AssistantResponseStarted

Published when the AI begins generating a response (for streaming indicators).

```rust
/// Published when AI response generation begins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantResponseStarted {
    pub event_id: EventId,
    pub conversation_id: ConversationId,
    pub component_id: ComponentId,
    pub session_id: SessionId,
    pub started_at: Timestamp,
}

impl DomainEvent for AssistantResponseStarted {
    fn event_type(&self) -> &'static str {
        "assistant.response_started"
    }

    fn aggregate_id(&self) -> String {
        self.conversation_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.started_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

**Subscribers:**
- `WebSocketEventBridge` - Show typing indicator

---

### AssistantResponseCompleted

Published when the AI finishes generating a response.

```rust
/// Published when AI response generation completes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantResponseCompleted {
    pub event_id: EventId,
    pub conversation_id: ConversationId,
    pub message_id: MessageId,
    pub component_id: ComponentId,
    pub session_id: SessionId,
    pub tokens_used: i32,
    pub completed_at: Timestamp,
}

impl DomainEvent for AssistantResponseCompleted {
    fn event_type(&self) -> &'static str {
        "assistant.response_completed"
    }

    fn aggregate_id(&self) -> String {
        self.conversation_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.completed_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

---

### StructuredDataExtracted

Published when AI extracts structured data from conversation.

```rust
/// Published when structured data is extracted from conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredDataExtracted {
    pub event_id: EventId,
    pub conversation_id: ConversationId,
    pub component_id: ComponentId,
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    pub component_type: ComponentType,
    /// Brief description of what was extracted
    pub extraction_summary: String,
    /// Number of items extracted (e.g., 3 objectives, 5 alternatives)
    pub item_count: i32,
    pub extracted_at: Timestamp,
}

impl DomainEvent for StructuredDataExtracted {
    fn event_type(&self) -> &'static str {
        "conversation.data_extracted"
    }

    fn aggregate_id(&self) -> String {
        self.conversation_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.extracted_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

**Subscribers:**
- `DashboardUpdateHandler` - Refresh component output view
- (Indirectly triggers `ComponentOutputUpdated` via cycle module)

---

### ConversationEnded

Published when a conversation is completed (component marked complete).

```rust
/// Published when a conversation ends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEnded {
    pub event_id: EventId,
    pub conversation_id: ConversationId,
    pub component_id: ComponentId,
    pub session_id: SessionId,
    pub total_messages: i32,
    pub total_tokens: i32,
    pub ended_at: Timestamp,
}

impl DomainEvent for ConversationEnded {
    fn event_type(&self) -> &'static str {
        "conversation.ended"
    }

    fn aggregate_id(&self) -> String {
        self.conversation_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.ended_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

---

## Event Handlers

### ConversationInitHandler

Subscribes to `ComponentStarted` to auto-create conversations.

```rust
// backend/src/application/handlers/conversation_init.rs

/// Initializes conversations when components are started
pub struct ConversationInitHandler {
    conversation_repo: Arc<dyn ConversationRepository>,
    event_publisher: Arc<dyn EventPublisher>,
    agent_configs: Arc<HashMap<ComponentType, AgentConfig>>,
}

impl ConversationInitHandler {
    pub fn new(
        conversation_repo: Arc<dyn ConversationRepository>,
        event_publisher: Arc<dyn EventPublisher>,
        agent_configs: Arc<HashMap<ComponentType, AgentConfig>>,
    ) -> Self {
        Self {
            conversation_repo,
            event_publisher,
            agent_configs,
        }
    }

    async fn create_conversation(
        &self,
        component_started: &ComponentStarted,
    ) -> Result<Conversation, DomainError> {
        // Get agent config for component type
        let config = self.agent_configs
            .get(&component_started.component_type)
            .ok_or_else(|| DomainError::new(
                ErrorCode::NotFound,
                &format!("No agent config for {:?}", component_started.component_type),
            ))?;

        // Create conversation with system prompt
        let conversation = Conversation::new(
            component_started.component_id,
            component_started.component_type,
            config.system_prompt.clone(),
        )?;

        Ok(conversation)
    }
}

#[async_trait]
impl EventHandler for ConversationInitHandler {
    async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError> {
        // Parse component started event
        let component_started: ComponentStarted = event.payload_as()
            .map_err(|e| DomainError::new(ErrorCode::ValidationFailed, &e.to_string()))?;

        // Check if conversation already exists (idempotency)
        if self.conversation_repo
            .find_by_component(component_started.component_id)
            .await?
            .is_some()
        {
            return Ok(()); // Already initialized
        }

        // Create conversation
        let conversation = self.create_conversation(&component_started).await?;

        // Persist
        self.conversation_repo.save(&conversation).await?;

        // Publish ConversationStarted event
        let started_event = ConversationStarted {
            event_id: EventId::new(),
            conversation_id: conversation.id(),
            component_id: component_started.component_id,
            cycle_id: component_started.cycle_id,
            session_id: component_started.session_id,
            component_type: component_started.component_type,
            started_at: Timestamp::now(),
        };

        let envelope = EventEnvelope::from_event(&started_event, "Conversation")
            .with_causation_id(event.event_id.as_str());

        self.event_publisher.publish(envelope).await?;

        Ok(())
    }

    fn name(&self) -> &'static str {
        "ConversationInitHandler"
    }
}
```

---

## Acceptance Criteria

### AC1: Auto-Initialize on ComponentStarted

**Given** a `ComponentStarted` event is published
**When** `ConversationInitHandler` processes it
**Then** a new conversation is created with:
- Correct component reference
- Appropriate system prompt for component type
- `ConversationStarted` event published

### AC2: Idempotent Initialization

**Given** a conversation already exists for a component
**When** `ConversationInitHandler` receives duplicate `ComponentStarted`
**Then** no new conversation is created (idempotent)

### AC3: MessageSent Published for User Messages

**Given** a user sends a message
**When** the message is processed
**Then** a `MessageSent` event is published with:
- Role = User
- Content preview (first 200 chars)
- Session ID for routing

### AC4: MessageSent Published for Assistant Messages

**Given** AI generates a response
**When** the response is complete
**Then** a `MessageSent` event is published with:
- Role = Assistant
- Content preview
- Token count

### AC5: StructuredDataExtracted on Extraction

**Given** AI extracts structured data from conversation
**When** extraction succeeds
**Then** `StructuredDataExtracted` event is published with:
- Extraction summary
- Item count
- This triggers component output update

### AC6: Typing Indicator Events

**Given** user sends a message
**When** AI starts generating response
**Then** `AssistantResponseStarted` is published
**When** AI finishes response
**Then** `AssistantResponseCompleted` is published

---

## Technical Design

### Command Handler Changes

```rust
// backend/src/application/commands/send_message.rs

pub struct SendMessageHandler {
    conversation_repo: Arc<dyn ConversationRepository>,
    cycle_repo: Arc<dyn CycleRepository>,
    ai_provider: Arc<dyn AIProvider>,
    event_publisher: Arc<dyn EventPublisher>,
    agent_configs: Arc<HashMap<ComponentType, AgentConfig>>,
}

impl SendMessageHandler {
    pub async fn handle(
        &self,
        cmd: SendMessageCommand,
        metadata: CommandMetadata,
    ) -> Result<Message, DomainError> {
        // Load conversation
        let mut conversation = self.conversation_repo
            .find_by_component(cmd.component_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::ConversationNotFound, "Conversation not found"))?;

        // Get cycle for session_id
        let cycle = self.find_cycle_for_component(cmd.component_id).await?;

        // Add user message
        let user_msg = conversation.add_user_message(&cmd.content);

        // Publish user message event
        self.publish_message_sent(
            &conversation,
            &user_msg,
            &cycle,
            &metadata,
        ).await?;

        // Publish response started
        let response_started = AssistantResponseStarted {
            event_id: EventId::new(),
            conversation_id: conversation.id(),
            component_id: cmd.component_id,
            session_id: cycle.session_id(),
            started_at: Timestamp::now(),
        };

        self.event_publisher.publish(
            EventEnvelope::from_event(&response_started, "Conversation")
                .with_correlation_id(metadata.correlation_id.clone())
        ).await?;

        // Get AI response
        let config = self.get_agent_config(conversation.component_type())?;
        let response = self.ai_provider
            .complete(self.build_prompt(config, &conversation))
            .await?;

        // Add assistant message
        let assistant_msg = conversation.add_assistant_message(&response.content);

        // Publish assistant message event
        self.publish_message_sent(
            &conversation,
            &assistant_msg,
            &cycle,
            &metadata,
        ).await?;

        // Publish response completed
        let response_completed = AssistantResponseCompleted {
            event_id: EventId::new(),
            conversation_id: conversation.id(),
            message_id: assistant_msg.id,
            component_id: cmd.component_id,
            session_id: cycle.session_id(),
            tokens_used: response.tokens_used,
            completed_at: Timestamp::now(),
        };

        self.event_publisher.publish(
            EventEnvelope::from_event(&response_completed, "Conversation")
                .with_correlation_id(metadata.correlation_id.clone())
        ).await?;

        // Extract structured data
        if let Some((extracted, summary, count)) = self.extract_structured_data(&response.content, config)? {
            // Update cycle component
            let mut cycle = cycle;
            cycle.update_component_output(conversation.component_type(), extracted)?;
            self.cycle_repo.update(&cycle).await?;

            // Publish extraction event
            let extraction_event = StructuredDataExtracted {
                event_id: EventId::new(),
                conversation_id: conversation.id(),
                component_id: cmd.component_id,
                cycle_id: cycle.id(),
                session_id: cycle.session_id(),
                component_type: conversation.component_type(),
                extraction_summary: summary,
                item_count: count,
                extracted_at: Timestamp::now(),
            };

            self.event_publisher.publish(
                EventEnvelope::from_event(&extraction_event, "Conversation")
                    .with_correlation_id(metadata.correlation_id.clone())
            ).await?;
        }

        // Persist conversation
        self.conversation_repo.save(&conversation).await?;

        Ok(assistant_msg)
    }

    async fn publish_message_sent(
        &self,
        conversation: &Conversation,
        message: &Message,
        cycle: &Cycle,
        metadata: &CommandMetadata,
    ) -> Result<(), DomainError> {
        let event = MessageSent {
            event_id: EventId::new(),
            conversation_id: conversation.id(),
            message_id: message.id,
            component_id: conversation.component_id(),
            cycle_id: cycle.id(),
            session_id: cycle.session_id(),
            component_type: conversation.component_type(),
            role: message.role,
            content_preview: message.content.chars().take(200).collect(),
            sent_at: message.timestamp,
        };

        self.event_publisher.publish(
            EventEnvelope::from_event(&event, "Conversation")
                .with_correlation_id(metadata.correlation_id.clone())
        ).await
    }
}
```

---

## File Structure

```
backend/src/domain/conversation/
├── mod.rs                    # Add events export
├── conversation.rs           # Existing entity
├── agent_state.rs            # Existing
├── events.rs                 # NEW: All conversation events
└── events_test.rs            # NEW: Event unit tests

backend/src/application/commands/
├── send_message.rs           # MODIFY: Add event publishing
├── send_message_test.rs      # MODIFY: Test event publishing
├── regenerate_response.rs    # MODIFY: Add event publishing
└── regenerate_response_test.rs

backend/src/application/handlers/
├── mod.rs                    # Add conversation handlers
├── conversation_init.rs      # NEW: Handle ComponentStarted
└── conversation_init_test.rs # NEW
```

---

## Test Specifications

### Unit Tests: Event Types

```rust
#[test]
fn message_sent_truncates_preview() {
    let long_content = "a".repeat(500);
    let event = MessageSent {
        event_id: EventId::new(),
        conversation_id: ConversationId::new(),
        message_id: MessageId::new(),
        component_id: ComponentId::new(),
        cycle_id: CycleId::new(),
        session_id: SessionId::new(),
        component_type: ComponentType::IssueRaising,
        role: Role::User,
        content_preview: long_content.chars().take(200).collect(),
        sent_at: Timestamp::now(),
    };

    assert_eq!(event.content_preview.len(), 200);
}

#[test]
fn structured_data_extracted_includes_count() {
    let event = StructuredDataExtracted {
        event_id: EventId::new(),
        conversation_id: ConversationId::new(),
        component_id: ComponentId::new(),
        cycle_id: CycleId::new(),
        session_id: SessionId::new(),
        component_type: ComponentType::Objectives,
        extraction_summary: "Extracted 3 fundamental objectives".to_string(),
        item_count: 3,
        extracted_at: Timestamp::now(),
    };

    assert_eq!(event.item_count, 3);
    assert!(event.extraction_summary.contains("3"));
}
```

### Unit Tests: Event Handler

```rust
#[tokio::test]
async fn conversation_init_creates_conversation_on_component_started() {
    let conversation_repo = Arc::new(InMemoryConversationRepository::new());
    let event_bus = Arc::new(InMemoryEventBus::new());
    let agent_configs = create_test_agent_configs();

    let handler = ConversationInitHandler::new(
        conversation_repo.clone(),
        event_bus.clone(),
        agent_configs,
    );

    let component_id = ComponentId::new();
    let event = EventEnvelope {
        event_id: EventId::new(),
        event_type: "component.started".to_string(),
        aggregate_id: CycleId::new().to_string(),
        aggregate_type: "Cycle".to_string(),
        occurred_at: Timestamp::now(),
        payload: json!({
            "cycle_id": CycleId::new().to_string(),
            "session_id": SessionId::new().to_string(),
            "component_id": component_id.to_string(),
            "component_type": "issue_raising",
            "started_at": Timestamp::now().to_string(),
        }),
        metadata: EventMetadata::default(),
    };

    // Act
    handler.handle(event).await.unwrap();

    // Assert - conversation created
    let conversation = conversation_repo.find_by_component(component_id).await.unwrap();
    assert!(conversation.is_some());

    // Assert - event published
    assert!(event_bus.has_event("conversation.started"));
}

#[tokio::test]
async fn conversation_init_is_idempotent() {
    let conversation_repo = Arc::new(InMemoryConversationRepository::new());
    let event_bus = Arc::new(InMemoryEventBus::new());
    let agent_configs = create_test_agent_configs();

    // Pre-create conversation
    let component_id = ComponentId::new();
    let existing = Conversation::new(
        component_id,
        ComponentType::IssueRaising,
        "System prompt".to_string(),
    ).unwrap();
    conversation_repo.save(&existing).await.unwrap();

    let handler = ConversationInitHandler::new(
        conversation_repo.clone(),
        event_bus.clone(),
        agent_configs,
    );

    let event = create_component_started_event(component_id);

    // Act
    handler.handle(event).await.unwrap();

    // Assert - no duplicate conversation
    // (Would fail if repo.save was called again with same component_id)
    assert_eq!(event_bus.event_count(), 0); // No new event
}
```

### Unit Tests: Command Handler

```rust
#[tokio::test]
async fn send_message_publishes_user_and_assistant_events() {
    let conversation_repo = Arc::new(InMemoryConversationRepository::new());
    let cycle_repo = Arc::new(InMemoryCycleRepository::new());
    let ai_provider = Arc::new(MockAIProvider::with_response("AI response"));
    let event_bus = Arc::new(InMemoryEventBus::new());

    // Setup cycle and conversation
    let session_id = SessionId::new();
    let cycle = Cycle::new(session_id).unwrap();
    let cycle_id = cycle.id();
    cycle_repo.save(&cycle).await.unwrap();

    let component_id = ComponentId::new();
    let conversation = Conversation::new(
        component_id,
        ComponentType::IssueRaising,
        "System prompt".to_string(),
    ).unwrap();
    conversation_repo.save(&conversation).await.unwrap();

    let handler = SendMessageHandler::new(
        conversation_repo,
        cycle_repo,
        ai_provider,
        event_bus.clone(),
        create_test_agent_configs(),
    );

    let cmd = SendMessageCommand {
        component_id,
        content: "Hello, I need help with a decision".to_string(),
    };

    // Act
    handler.handle(cmd, CommandMetadata::default()).await.unwrap();

    // Assert - multiple events published
    let message_events = event_bus.events_of_type("message.sent");
    assert_eq!(message_events.len(), 2); // User + Assistant

    // Verify user message event
    let user_event: MessageSent = message_events[0].payload_as().unwrap();
    assert_eq!(user_event.role, Role::User);

    // Verify assistant message event
    let assistant_event: MessageSent = message_events[1].payload_as().unwrap();
    assert_eq!(assistant_event.role, Role::Assistant);

    // Verify response lifecycle events
    assert!(event_bus.has_event("assistant.response_started"));
    assert!(event_bus.has_event("assistant.response_completed"));
}

#[tokio::test]
async fn send_message_publishes_extraction_event_when_data_extracted() {
    // Similar setup...

    let ai_provider = Arc::new(MockAIProvider::with_response(
        "I've identified the following objectives:\n1. Maximize income\n2. Work-life balance\n3. Career growth"
    ));

    // Act
    handler.handle(cmd, CommandMetadata::default()).await.unwrap();

    // Assert - extraction event published
    let extraction_events = event_bus.events_of_type("conversation.data_extracted");
    assert_eq!(extraction_events.len(), 1);

    let payload: StructuredDataExtracted = extraction_events[0].payload_as().unwrap();
    assert!(payload.item_count > 0);
}
```

---

## Event Registration

```rust
// backend/src/main.rs or setup module

fn register_conversation_handlers(event_bus: &impl EventSubscriber, deps: &Dependencies) {
    // Auto-initialize conversations when components start
    event_bus.subscribe(
        "component.started",
        ConversationInitHandler::new(
            deps.conversation_repo.clone(),
            deps.event_publisher.clone(),
            deps.agent_configs.clone(),
        ),
    );
}
```

---

## Dependencies

### Module Dependencies

- `foundation::events` - EventId, EventEnvelope, DomainEvent
- `foundation::ids` - ConversationId, ComponentId, MessageId, CycleId, SessionId
- `foundation::component_type` - ComponentType enum
- `proact-types::message` - Role enum
- `ports::event_publisher` - EventPublisher trait
- `cycle::events` - ComponentStarted (subscribed)

---

## Related Documents

- **Integration Spec:** features/integrations/full-proact-journey.md
- **Phase 3:** features/cycle/cycle-events.md
- **Checklist:** REQUIREMENTS/CHECKLIST-events.md (Phase 4)
- **Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md

---

*Version: 1.0.0*
*Created: 2026-01-07*
*Phase: 4 of 8*
