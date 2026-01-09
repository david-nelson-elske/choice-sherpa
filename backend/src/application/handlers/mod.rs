//! Application handlers.
//!
//! Command and query handlers that orchestrate domain operations.

pub mod stream_message;

pub use stream_message::{
    // Handler
    StreamingMessageHandler,
    StreamingHandlerConfig,
    // Commands and Results
    StreamMessageCommand,
    StreamMessageResult,
    StreamMessageError,
    // Types
    MessageId,
    MessageRole,
    StoredMessage,
    TokenUsage,
    // WebSocket messages
    StreamWebSocketMessage,
    // Ports
    AIProvider,
    AIProviderError,
    CompletionRequest,
    StreamChunk,
    StreamingResponse,
    WebSocketBroadcaster,
    BroadcastError,
    ConversationRepository,
    ConversationRecord,
    RepositoryError,
};
