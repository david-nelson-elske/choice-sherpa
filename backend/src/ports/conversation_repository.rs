//! Conversation Repository Port - Write-side persistence for conversations.

use async_trait::async_trait;
use crate::domain::conversation::Conversation;
use crate::domain::foundation::{ComponentId, ConversationId};
use crate::domain::proact::Message;

/// Port for persisting conversations (write-side).
///
/// This port handles all write operations for conversations,
/// including creation, updates, and message appending.
#[async_trait::async_trait]
pub trait ConversationRepository: Send + Sync {
    /// Persists a new conversation.
    async fn save(&self, conversation: &Conversation) -> Result<(), RepositoryError>;

    /// Updates an existing conversation.
    async fn update(&self, conversation: &Conversation) -> Result<(), RepositoryError>;

    /// Finds a conversation by component ID.
    async fn find_by_component(
        &self,
        component_id: ComponentId,
    ) -> Result<Option<Conversation>, RepositoryError>;

    /// Finds a conversation by ID.
    async fn find_by_id(
        &self,
        id: ConversationId,
    ) -> Result<Option<Conversation>, RepositoryError>;

    /// Appends a message to a conversation (optimized for append-only).
    async fn append_message(
        &self,
        conversation_id: ConversationId,
        message: &Message,
    ) -> Result<(), RepositoryError>;

    /// Deletes a conversation.
    async fn delete(&self, id: ConversationId) -> Result<(), RepositoryError>;
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Conversation not found: {0}")]
    NotFound(ConversationId),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
