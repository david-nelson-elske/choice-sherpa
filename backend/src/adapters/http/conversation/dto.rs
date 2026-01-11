//! HTTP DTOs for conversation endpoints.
//!
//! These types decouple the HTTP API from domain types, allowing independent evolution.

use serde::{Deserialize, Serialize};

use crate::domain::conversation::{AgentPhase, ConversationState};
use crate::domain::foundation::{ComponentType, Timestamp};
use crate::domain::proact::{Message as DomainMessage, Role};
use crate::ports::ConversationView as DomainConversationView;

// ════════════════════════════════════════════════════════════════════════════
// Request DTOs
// ════════════════════════════════════════════════════════════════════════════

/// Request to send a message in a conversation.
#[derive(Debug, Clone, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

// ════════════════════════════════════════════════════════════════════════════
// Response DTOs
// ════════════════════════════════════════════════════════════════════════════

/// Response for message sent successfully.
#[derive(Debug, Clone, Serialize)]
pub struct SendMessageResponse {
    pub conversation_id: String,
    pub message: MessageResponse,
}

/// Message representation for API responses.
#[derive(Debug, Clone, Serialize)]
pub struct MessageResponse {
    pub id: String,
    pub role: Role,
    pub content: String,
    pub timestamp: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_count: Option<u32>,
}

impl From<&DomainMessage> for MessageResponse {
    fn from(message: &DomainMessage) -> Self {
        Self {
            id: message.id.as_uuid().to_string(),
            role: message.role,
            content: message.content.clone(),
            timestamp: message.timestamp,
            token_count: message.metadata.token_count,
        }
    }
}

/// Conversation view for API responses.
#[derive(Debug, Clone, Serialize)]
pub struct ConversationResponse {
    pub id: String,
    pub component_id: String,
    pub component_type: ComponentType,
    pub state: ConversationState,
    pub current_phase: AgentPhase,
    pub messages: Vec<MessageResponse>,
    pub message_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_extraction: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_at: Option<Timestamp>,
}

impl From<DomainConversationView> for ConversationResponse {
    fn from(view: DomainConversationView) -> Self {
        Self {
            id: view.id.to_string(),
            component_id: view.component_id.to_string(),
            component_type: view.component_type,
            state: view.state,
            current_phase: view.current_phase,
            messages: view.messages.iter().map(MessageResponse::from).collect(),
            message_count: view.message_count,
            pending_extraction: view.pending_extraction,
            last_message_at: view.last_message_at,
        }
    }
}

/// Streaming message chunk for WebSocket.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamChunk {
    /// Content delta (incremental text).
    Delta { content: String },
    /// Message completed.
    Done {
        message_id: String,
        token_count: Option<u32>,
    },
    /// Error occurred during streaming.
    Error { message: String },
}
