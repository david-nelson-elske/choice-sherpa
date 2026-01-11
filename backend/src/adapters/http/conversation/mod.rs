//! HTTP adapters for conversation endpoints.
//!
//! Provides REST API and WebSocket streaming for conversation access.

pub mod dto;
pub mod handlers;
pub mod routes;
pub mod streaming;

pub use dto::{
    ConversationView, ErrorResponse, MessageRoleDto, MessageView, Page, PaginationParams,
    TokenUsageDto,
};
pub use handlers::{ConversationAppState, ConversationApiError, RateLimiter, RegenerateResponse};
pub use routes::{conversation_router, conversation_routes};
pub use streaming::{
    DataExtractedMessage, PhaseTransition, SendMessageRequest, StreamChunkMessage,
    StreamClientMessage, StreamCompleteMessage, StreamErrorCode, StreamErrorMessage,
    StreamPongMessage, StreamServerMessage, StreamTokenUsage, MAX_MESSAGE_LENGTH,
};
