//! Conversation reader port (read side / CQRS queries).
//!
//! Defines the contract for conversation queries and read operations.
//! Optimized for UI display and message pagination.
//!
//! # Design
//!
//! - **Read-optimized**: Can use caching, denormalized views
//! - **Separated from write**: CQRS pattern for scalability
//! - **Pagination support**: For message history

use crate::domain::conversation::{ConversationState, Role};
use crate::domain::foundation::{ComponentId, ConversationId, DomainError, Timestamp};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Reader port for conversation queries.
///
/// Provides read-optimized views of conversation data.
/// Implementations may use caching for frequently-accessed data.
#[async_trait]
pub trait ConversationReader: Send + Sync {
    /// Get a conversation view by ID.
    ///
    /// Returns `None` if not found.
    async fn get(&self, id: &ConversationId) -> Result<Option<ConversationView>, DomainError>;

    /// Get a conversation view by component ID.
    ///
    /// Returns `None` if no conversation exists for the component.
    async fn get_by_component(
        &self,
        component_id: &ComponentId,
    ) -> Result<Option<ConversationView>, DomainError>;

    /// Get paginated messages for a conversation.
    ///
    /// Messages are ordered by `created_at` ascending (oldest first).
    ///
    /// # Arguments
    ///
    /// * `conversation_id` - The conversation to get messages for
    /// * `options` - Pagination options (limit, offset)
    ///
    /// # Returns
    ///
    /// A `MessageList` containing the messages and pagination metadata.
    async fn get_messages(
        &self,
        conversation_id: &ConversationId,
        options: &MessageListOptions,
    ) -> Result<MessageList, DomainError>;
}

/// Options for listing messages.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageListOptions {
    /// Maximum number of messages to return.
    pub limit: Option<u32>,

    /// Number of messages to skip.
    pub offset: Option<u32>,

    /// Filter to user-visible messages only (User/Assistant).
    pub user_visible_only: bool,
}

impl MessageListOptions {
    /// Create options for a paginated query.
    pub fn paginated(limit: u32, offset: u32) -> Self {
        Self {
            limit: Some(limit),
            offset: Some(offset),
            user_visible_only: false,
        }
    }

    /// Create options with just a limit (no offset).
    pub fn with_limit(limit: u32) -> Self {
        Self {
            limit: Some(limit),
            offset: None,
            user_visible_only: false,
        }
    }

    /// Filter to only user-visible messages.
    pub fn visible_only(mut self) -> Self {
        self.user_visible_only = true;
        self
    }

    /// Returns the effective limit (defaults to 50).
    pub fn effective_limit(&self) -> u32 {
        self.limit.unwrap_or(50).min(100)
    }

    /// Returns the effective offset (defaults to 0).
    pub fn effective_offset(&self) -> u32 {
        self.offset.unwrap_or(0)
    }
}

/// Paginated list of messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageList {
    /// Messages in this page.
    pub items: Vec<MessageView>,

    /// Total number of messages in the conversation.
    pub total: u64,

    /// Whether there are more messages after this page.
    pub has_more: bool,
}

/// View of a conversation for UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationView {
    /// Conversation ID.
    pub id: ConversationId,

    /// Component ID this conversation belongs to.
    pub component_id: ComponentId,

    /// Current state.
    pub state: ConversationState,

    /// Number of messages.
    pub message_count: u32,

    /// When the conversation was created.
    pub created_at: Timestamp,

    /// When the conversation was last updated.
    pub updated_at: Timestamp,
}

/// View of a message for UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageView {
    /// Message ID.
    pub id: String,

    /// Role (user, assistant, system).
    pub role: Role,

    /// Message content.
    pub content: String,

    /// When the message was created.
    pub created_at: Timestamp,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trait object safety test
    #[test]
    fn conversation_reader_is_object_safe() {
        fn _accepts_dyn(_reader: &dyn ConversationReader) {}
    }

    mod message_list_options {
        use super::*;

        #[test]
        fn default_has_no_limits() {
            let options = MessageListOptions::default();
            assert!(options.limit.is_none());
            assert!(options.offset.is_none());
            assert!(!options.user_visible_only);
        }

        #[test]
        fn paginated_sets_limit_and_offset() {
            let options = MessageListOptions::paginated(10, 20);
            assert_eq!(options.limit, Some(10));
            assert_eq!(options.offset, Some(20));
        }

        #[test]
        fn effective_limit_defaults_to_50() {
            let options = MessageListOptions::default();
            assert_eq!(options.effective_limit(), 50);
        }

        #[test]
        fn effective_limit_caps_at_100() {
            let options = MessageListOptions { limit: Some(200), ..Default::default() };
            assert_eq!(options.effective_limit(), 100);
        }

        #[test]
        fn effective_offset_defaults_to_0() {
            let options = MessageListOptions::default();
            assert_eq!(options.effective_offset(), 0);
        }

        #[test]
        fn visible_only_filters_system_messages() {
            let options = MessageListOptions::default().visible_only();
            assert!(options.user_visible_only);
        }
    }
}
