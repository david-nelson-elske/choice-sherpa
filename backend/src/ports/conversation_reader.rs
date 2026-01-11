//! ConversationReader port - Query interface for conversation data.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::conversation::{AgentPhase, ConversationState};
use crate::domain::foundation::{ComponentId, ComponentType, ConversationId, Timestamp};
use crate::domain::proact::Message;

/// Port for reading conversation data (CQRS query side).
///
/// This port is optimized for read operations and provides
/// simplified view models for the conversation UI.
#[async_trait::async_trait]
pub trait ConversationReader: Send + Sync {
    /// Gets a conversation view by component ID.
    async fn get_by_component(
        &self,
        component_id: &ComponentId,
    ) -> Result<Option<ConversationView>, ReaderError>;

    /// Gets a conversation view by conversation ID.
    async fn get_by_id(
        &self,
        conversation_id: &ConversationId,
    ) -> Result<Option<ConversationView>, ReaderError>;

    /// Gets message count for a conversation.
    async fn get_message_count(&self, conversation_id: ConversationId) -> Result<usize, ReaderError>;

    /// Gets recent messages (for context).
    async fn get_recent_messages(
        &self,
        conversation_id: ConversationId,
        limit: usize,
    ) -> Result<Vec<Message>, ReaderError>;
}

/// View model for displaying a conversation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConversationView {
    pub id: ConversationId,
    pub component_id: ComponentId,
    pub component_type: ComponentType,
    pub messages: Vec<Message>,
    pub state: ConversationState,
    pub current_phase: AgentPhase,
    pub pending_extraction: Option<serde_json::Value>,
    pub message_count: usize,
    pub last_message_at: Option<Timestamp>,
}

#[derive(Debug, thiserror::Error)]
pub enum ReaderError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
