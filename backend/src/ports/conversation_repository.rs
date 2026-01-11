//! Conversation repository port (write side).
//!
//! Defines the contract for persisting and retrieving Conversation aggregates.
//! Implementations handle the actual database operations.
//!
//! # Design
//!
//! - **Write-focused**: Optimized for aggregate persistence
//! - **Component-scoped**: One conversation per component (unique constraint)
//! - **Message ownership**: Messages are owned by Conversation

use crate::domain::conversation::{Conversation, Message};
use crate::domain::foundation::{ComponentId, ConversationId, DomainError};
use async_trait::async_trait;

/// Repository port for Conversation aggregate persistence.
///
/// Handles write operations for conversation lifecycle management.
/// Implementations must ensure:
/// - One conversation per component (unique constraint)
/// - Messages are persisted in order
/// - Domain event publication on state changes
#[async_trait]
pub trait ConversationRepository: Send + Sync {
    /// Save a new conversation.
    ///
    /// # Errors
    ///
    /// - `AlreadyExists` if component already has a conversation
    /// - `DatabaseError` on persistence failure
    async fn save(&self, conversation: &Conversation) -> Result<(), DomainError>;

    /// Update an existing conversation.
    ///
    /// Updates the conversation state and metadata. Does NOT update messages
    /// (use `add_message` for that).
    ///
    /// # Errors
    ///
    /// - `ConversationNotFound` if conversation doesn't exist
    /// - `DatabaseError` on persistence failure
    async fn update(&self, conversation: &Conversation) -> Result<(), DomainError>;

    /// Add a message to a conversation.
    ///
    /// Messages are appended in order. The conversation must exist.
    ///
    /// # Errors
    ///
    /// - `ConversationNotFound` if conversation doesn't exist
    /// - `DatabaseError` on persistence failure
    async fn add_message(
        &self,
        conversation_id: &ConversationId,
        message: &Message,
    ) -> Result<(), DomainError>;

    /// Find a conversation by its ID.
    ///
    /// Returns the full conversation including all messages.
    ///
    /// Returns `None` if not found.
    async fn find_by_id(
        &self,
        id: &ConversationId,
    ) -> Result<Option<Conversation>, DomainError>;

    /// Find a conversation by component ID.
    ///
    /// Each component has at most one conversation.
    ///
    /// Returns `None` if no conversation exists for the component.
    async fn find_by_component(
        &self,
        component_id: &ComponentId,
    ) -> Result<Option<Conversation>, DomainError>;

    /// Check if a conversation exists for a component.
    async fn exists_for_component(&self, component_id: &ComponentId) -> Result<bool, DomainError>;

    /// Delete a conversation (primarily for testing).
    ///
    /// In production, conversations should generally not be deleted.
    ///
    /// # Errors
    ///
    /// - `ConversationNotFound` if conversation doesn't exist
    /// - `DatabaseError` on persistence failure
    async fn delete(&self, id: &ConversationId) -> Result<(), DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trait object safety test
    #[test]
    fn conversation_repository_is_object_safe() {
        fn _accepts_dyn(_repo: &dyn ConversationRepository) {}
    }
}
