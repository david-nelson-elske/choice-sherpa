//! Streaming message handler.
//!
//! Handles user messages in conversations with streaming AI responses.

use crate::domain::conversation::{
    AgentPhase, ConversationSnapshot, ConversationState, ContextMessage, ContextWindowManager,
    PhaseTransitionEngine,
};
use crate::domain::foundation::{ComponentId, ComponentType, SessionId, Timestamp, UserId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

/// Unique identifier for a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageId(Uuid);

impl MessageId {
    /// Creates a new random MessageId.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a MessageId from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the inner UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Command to send a message and receive a streaming response.
#[derive(Debug, Clone)]
pub struct StreamMessageCommand {
    /// The session this message belongs to.
    pub session_id: SessionId,
    /// The component this conversation is for.
    pub component_id: ComponentId,
    /// The user sending the message.
    pub user_id: UserId,
    /// The message content.
    pub content: String,
}

impl StreamMessageCommand {
    /// Creates a new stream message command.
    pub fn new(
        session_id: SessionId,
        component_id: ComponentId,
        user_id: UserId,
        content: impl Into<String>,
    ) -> Self {
        Self {
            session_id,
            component_id,
            user_id,
            content: content.into(),
        }
    }
}

/// Errors that can occur during message handling.
#[derive(Debug, Clone, Error)]
pub enum StreamMessageError {
    #[error("Conversation not found for component {0}")]
    ConversationNotFound(ComponentId),

    #[error("Conversation is not in an active state")]
    ConversationNotActive,

    #[error("User not authorized to access this conversation")]
    Unauthorized,

    #[error("AI provider error: {0}")]
    AIProviderError(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),
}

/// Result of handling a stream message.
#[derive(Debug, Clone)]
pub struct StreamMessageResult {
    /// The ID of the user's message.
    pub user_message_id: MessageId,
    /// The ID of the assistant's response.
    pub assistant_message_id: MessageId,
    /// The new phase after processing.
    pub new_phase: AgentPhase,
    /// The new state after processing.
    pub new_state: ConversationState,
    /// Token usage information.
    pub usage: Option<TokenUsage>,
}

/// Token usage for a completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Tokens used for the prompt.
    pub prompt_tokens: u32,
    /// Tokens used for the completion.
    pub completion_tokens: u32,
    /// Estimated cost in cents.
    pub estimated_cost_cents: u32,
}

/// A stored message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    /// Unique ID of this message.
    pub id: MessageId,
    /// Role of the sender.
    pub role: MessageRole,
    /// Content of the message.
    pub content: String,
    /// When the message was created.
    pub created_at: Timestamp,
}

/// Role for stored messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

impl StoredMessage {
    /// Creates a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            id: MessageId::new(),
            role: MessageRole::User,
            content: content.into(),
            created_at: Timestamp::now(),
        }
    }

    /// Creates a new assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            id: MessageId::new(),
            role: MessageRole::Assistant,
            content: content.into(),
            created_at: Timestamp::now(),
        }
    }

    /// Creates a new assistant message with a specific ID.
    pub fn assistant_with_id(id: MessageId, content: impl Into<String>) -> Self {
        Self {
            id,
            role: MessageRole::Assistant,
            content: content.into(),
            created_at: Timestamp::now(),
        }
    }

    /// Converts to a context message for AI calls.
    pub fn to_context_message(&self) -> ContextMessage {
        match self.role {
            MessageRole::System => ContextMessage::system(&self.content),
            MessageRole::User => ContextMessage::user(&self.content),
            MessageRole::Assistant => ContextMessage::assistant(&self.content),
        }
    }
}

/// WebSocket message types for streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamWebSocketMessage {
    /// A chunk of the streaming response.
    StreamChunk {
        message_id: MessageId,
        delta: String,
        is_final: bool,
    },
    /// An error occurred during streaming.
    StreamError {
        message_id: MessageId,
        error: String,
    },
    /// Streaming completed with full content.
    StreamComplete {
        message_id: MessageId,
        full_content: String,
        usage: Option<TokenUsage>,
    },
}

/// Port for AI completion with streaming support.
pub trait AIProvider: Send + Sync {
    /// Sends a completion request and returns a streaming response.
    fn stream_complete(
        &self,
        request: CompletionRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<StreamingResponse, AIProviderError>> + Send>,
    >;
}

/// Request for AI completion.
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    /// Messages to send to the AI.
    pub messages: Vec<ContextMessage>,
    /// System prompt (if not in messages).
    pub system_prompt: Option<String>,
    /// Temperature for response generation.
    pub temperature: Option<f32>,
    /// Maximum tokens to generate.
    pub max_tokens: Option<u32>,
}

impl Default for CompletionRequest {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            system_prompt: None,
            temperature: None,
            max_tokens: None,
        }
    }
}

/// A streaming chunk from the AI.
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// The delta content in this chunk.
    pub delta: String,
    /// The reason for finishing (if this is the last chunk).
    pub finish_reason: Option<String>,
}

/// Streaming response from AI provider.
pub struct StreamingResponse {
    /// Receiver for stream chunks.
    pub receiver: tokio::sync::mpsc::Receiver<Result<StreamChunk, AIProviderError>>,
}

/// Error from AI provider.
#[derive(Debug, Clone, Error)]
pub enum AIProviderError {
    #[error("Rate limited")]
    RateLimited,

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),
}

/// Port for broadcasting WebSocket messages.
pub trait WebSocketBroadcaster: Send + Sync {
    /// Broadcasts a message to all connections in a session.
    fn broadcast_to_session(
        &self,
        session_id: &SessionId,
        message: StreamWebSocketMessage,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), BroadcastError>> + Send>>;
}

/// Error during broadcast.
#[derive(Debug, Clone, Error)]
pub enum BroadcastError {
    #[error("No active connections for session")]
    NoConnections,

    #[error("Send error: {0}")]
    SendError(String),
}

/// Port for conversation persistence.
pub trait ConversationRepository: Send + Sync {
    /// Finds a conversation by component ID.
    fn find_by_component(
        &self,
        component_id: &ComponentId,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Option<ConversationRecord>, RepositoryError>>
                + Send,
        >,
    >;

    /// Saves a conversation.
    fn save(
        &self,
        conversation: &ConversationRecord,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), RepositoryError>> + Send>>;

    /// Adds a message to a conversation.
    fn add_message(
        &self,
        component_id: &ComponentId,
        message: StoredMessage,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), RepositoryError>> + Send>>;
}

/// Repository error.
#[derive(Debug, Clone, Error)]
pub enum RepositoryError {
    #[error("Not found")]
    NotFound,

    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// A conversation record from the repository.
#[derive(Debug, Clone)]
pub struct ConversationRecord {
    /// The component this conversation is for.
    pub component_id: ComponentId,
    /// Type of component.
    pub component_type: ComponentType,
    /// Current state.
    pub state: ConversationState,
    /// Current phase.
    pub phase: AgentPhase,
    /// Messages in the conversation.
    pub messages: Vec<StoredMessage>,
    /// The user who owns this conversation.
    pub user_id: UserId,
    /// System prompt for this conversation.
    pub system_prompt: String,
}

impl ConversationRecord {
    /// Returns the count of user messages.
    pub fn user_message_count(&self) -> usize {
        self.messages
            .iter()
            .filter(|m| m.role == MessageRole::User)
            .count()
    }

    /// Returns the latest user message content.
    pub fn latest_user_message(&self) -> Option<String> {
        self.messages
            .iter()
            .rev()
            .find(|m| m.role == MessageRole::User)
            .map(|m| m.content.clone())
    }

    /// Creates a snapshot for phase transition decisions.
    pub fn to_snapshot(&self) -> ConversationSnapshot {
        ConversationSnapshot::new(
            self.user_message_count(),
            self.latest_user_message(),
            self.component_type,
        )
    }

    /// Converts messages to context messages for AI calls.
    pub fn to_context_messages(&self) -> Vec<ContextMessage> {
        self.messages
            .iter()
            .map(|m| m.to_context_message())
            .collect()
    }
}

/// Configuration for the streaming handler.
#[derive(Debug, Clone)]
pub struct StreamingHandlerConfig {
    /// Default temperature for completions.
    pub default_temperature: f32,
    /// Default max tokens for responses.
    pub default_max_tokens: u32,
}

impl Default for StreamingHandlerConfig {
    fn default() -> Self {
        Self {
            default_temperature: 0.7,
            default_max_tokens: 2000,
        }
    }
}

/// Handler for streaming message interactions.
pub struct StreamingMessageHandler<A, W, R>
where
    A: AIProvider,
    W: WebSocketBroadcaster,
    R: ConversationRepository,
{
    ai_provider: Arc<A>,
    ws_broadcaster: Arc<W>,
    conversation_repo: Arc<R>,
    config: StreamingHandlerConfig,
}

impl<A, W, R> StreamingMessageHandler<A, W, R>
where
    A: AIProvider,
    W: WebSocketBroadcaster,
    R: ConversationRepository,
{
    /// Creates a new handler with the given dependencies.
    pub fn new(ai_provider: Arc<A>, ws_broadcaster: Arc<W>, conversation_repo: Arc<R>) -> Self {
        Self {
            ai_provider,
            ws_broadcaster,
            conversation_repo,
            config: StreamingHandlerConfig::default(),
        }
    }

    /// Creates a handler with custom configuration.
    pub fn with_config(
        ai_provider: Arc<A>,
        ws_broadcaster: Arc<W>,
        conversation_repo: Arc<R>,
        config: StreamingHandlerConfig,
    ) -> Self {
        Self {
            ai_provider,
            ws_broadcaster,
            conversation_repo,
            config,
        }
    }

    /// Handles a streaming message command.
    pub async fn handle(
        &self,
        cmd: StreamMessageCommand,
    ) -> Result<StreamMessageResult, StreamMessageError> {
        // 1. Load conversation
        let mut conversation = self
            .conversation_repo
            .find_by_component(&cmd.component_id)
            .await
            .map_err(|e| StreamMessageError::RepositoryError(e.to_string()))?
            .ok_or(StreamMessageError::ConversationNotFound(cmd.component_id))?;

        // 2. Verify authorization
        if conversation.user_id != cmd.user_id {
            return Err(StreamMessageError::Unauthorized);
        }

        // 3. Verify conversation is active
        if !conversation.state.accepts_user_input() {
            return Err(StreamMessageError::ConversationNotActive);
        }

        // 4. Add user message
        let user_message = StoredMessage::user(&cmd.content);
        let user_message_id = user_message.id;
        conversation.messages.push(user_message.clone());

        // Save user message
        self.conversation_repo
            .add_message(&cmd.component_id, user_message)
            .await
            .map_err(|e| StreamMessageError::RepositoryError(e.to_string()))?;

        // 5. Build context for AI
        let context_manager = ContextWindowManager::for_component(conversation.component_type);
        let context_messages = conversation.to_context_messages();
        let built_context =
            context_manager.build_context(&conversation.system_prompt, &context_messages);

        // 6. Start streaming response
        let assistant_message_id = MessageId::new();
        let request = CompletionRequest {
            messages: built_context.messages,
            system_prompt: None, // Already in messages
            temperature: Some(self.config.default_temperature),
            max_tokens: Some(self.config.default_max_tokens),
        };

        let mut stream = self
            .ai_provider
            .stream_complete(request)
            .await
            .map_err(|e| StreamMessageError::AIProviderError(e.to_string()))?;

        // 7. Stream chunks to WebSocket
        let mut full_response = String::new();

        while let Some(chunk_result) = stream.receiver.recv().await {
            match chunk_result {
                Ok(chunk) => {
                    full_response.push_str(&chunk.delta);

                    // Broadcast chunk to client
                    let _ = self
                        .ws_broadcaster
                        .broadcast_to_session(
                            &cmd.session_id,
                            StreamWebSocketMessage::StreamChunk {
                                message_id: assistant_message_id,
                                delta: chunk.delta,
                                is_final: chunk.finish_reason.is_some(),
                            },
                        )
                        .await;
                }
                Err(e) => {
                    // Broadcast error
                    let _ = self
                        .ws_broadcaster
                        .broadcast_to_session(
                            &cmd.session_id,
                            StreamWebSocketMessage::StreamError {
                                message_id: assistant_message_id,
                                error: e.to_string(),
                            },
                        )
                        .await;

                    return Err(StreamMessageError::AIProviderError(e.to_string()));
                }
            }
        }

        // 8. Add complete assistant message
        let assistant_message =
            StoredMessage::assistant_with_id(assistant_message_id, &full_response);
        conversation.messages.push(assistant_message.clone());

        self.conversation_repo
            .add_message(&cmd.component_id, assistant_message)
            .await
            .map_err(|e| StreamMessageError::RepositoryError(e.to_string()))?;

        // 9. Determine new phase
        let transition_engine = PhaseTransitionEngine::for_component(conversation.component_type);
        let snapshot = conversation.to_snapshot();
        let new_phase = transition_engine.next_phase(conversation.phase, &snapshot);

        // 10. Update state if needed
        let new_state = if conversation.state == ConversationState::Ready {
            // First user message moves to InProgress
            ConversationState::InProgress
        } else {
            conversation.state
        };

        // 11. Send completion message
        let _ = self
            .ws_broadcaster
            .broadcast_to_session(
                &cmd.session_id,
                StreamWebSocketMessage::StreamComplete {
                    message_id: assistant_message_id,
                    full_content: full_response,
                    usage: None, // TODO: Add actual usage tracking
                },
            )
            .await;

        Ok(StreamMessageResult {
            user_message_id,
            assistant_message_id,
            new_phase,
            new_state,
            usage: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod message_id {
        use super::*;

        #[test]
        fn generates_unique_ids() {
            let id1 = MessageId::new();
            let id2 = MessageId::new();
            assert_ne!(id1, id2);
        }

        #[test]
        fn displays_as_uuid() {
            let id = MessageId::new();
            let display = format!("{}", id);
            assert!(display.len() == 36); // UUID format
        }
    }

    mod stored_message {
        use super::*;

        #[test]
        fn creates_user_message() {
            let msg = StoredMessage::user("Hello");
            assert_eq!(msg.role, MessageRole::User);
            assert_eq!(msg.content, "Hello");
        }

        #[test]
        fn creates_assistant_message() {
            let msg = StoredMessage::assistant("Hi there!");
            assert_eq!(msg.role, MessageRole::Assistant);
            assert_eq!(msg.content, "Hi there!");
        }

        #[test]
        fn converts_to_context_message() {
            let msg = StoredMessage::user("Test");
            let ctx = msg.to_context_message();
            assert_eq!(
                ctx.role,
                crate::domain::conversation::MessageRole::User
            );
            assert_eq!(ctx.content, "Test");
        }
    }

    mod stream_websocket_message {
        use super::*;

        #[test]
        fn serializes_chunk() {
            let msg = StreamWebSocketMessage::StreamChunk {
                message_id: MessageId::new(),
                delta: "Hello".to_string(),
                is_final: false,
            };
            let json = serde_json::to_string(&msg).unwrap();
            assert!(json.contains("stream_chunk"));
            assert!(json.contains("Hello"));
        }

        #[test]
        fn serializes_error() {
            let msg = StreamWebSocketMessage::StreamError {
                message_id: MessageId::new(),
                error: "Connection lost".to_string(),
            };
            let json = serde_json::to_string(&msg).unwrap();
            assert!(json.contains("stream_error"));
            assert!(json.contains("Connection lost"));
        }

        #[test]
        fn serializes_complete() {
            let msg = StreamWebSocketMessage::StreamComplete {
                message_id: MessageId::new(),
                full_content: "Full response".to_string(),
                usage: Some(TokenUsage {
                    prompt_tokens: 100,
                    completion_tokens: 50,
                    estimated_cost_cents: 1,
                }),
            };
            let json = serde_json::to_string(&msg).unwrap();
            assert!(json.contains("stream_complete"));
            assert!(json.contains("Full response"));
        }
    }

    mod conversation_record {
        use super::*;

        fn sample_record() -> ConversationRecord {
            ConversationRecord {
                component_id: ComponentId::new(),
                component_type: ComponentType::IssueRaising,
                state: ConversationState::InProgress,
                phase: AgentPhase::Gather,
                messages: vec![
                    StoredMessage::user("First message"),
                    StoredMessage::assistant("Response"),
                    StoredMessage::user("Second message"),
                ],
                user_id: UserId::new("user-123").unwrap(),
                system_prompt: "You are helpful.".to_string(),
            }
        }

        #[test]
        fn counts_user_messages() {
            let record = sample_record();
            assert_eq!(record.user_message_count(), 2);
        }

        #[test]
        fn gets_latest_user_message() {
            let record = sample_record();
            assert_eq!(
                record.latest_user_message(),
                Some("Second message".to_string())
            );
        }

        #[test]
        fn creates_snapshot() {
            let record = sample_record();
            let snapshot = record.to_snapshot();
            assert_eq!(snapshot.user_message_count, 2);
            assert_eq!(
                snapshot.latest_user_message,
                Some("Second message".to_string())
            );
            assert_eq!(snapshot.component_type, ComponentType::IssueRaising);
        }

        #[test]
        fn converts_to_context_messages() {
            let record = sample_record();
            let context = record.to_context_messages();
            assert_eq!(context.len(), 3);
        }
    }

    mod stream_message_command {
        use super::*;

        #[test]
        fn creates_command() {
            let cmd = StreamMessageCommand::new(
                SessionId::new(),
                ComponentId::new(),
                UserId::new("user-1").unwrap(),
                "Test message",
            );
            assert_eq!(cmd.content, "Test message");
        }
    }
}
