//! Message entity for conversations.
//!
//! Messages are immutable records of user/assistant exchanges within a conversation.
//! Each message has a role (user/assistant/system), content, and timestamp.

use crate::domain::foundation::{DomainError, Timestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a message within a conversation.
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

impl std::str::FromStr for MessageId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// Role of a message sender in a conversation.
///
/// Mirrors the AI provider message roles for consistency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// System instructions (typically invisible to user).
    System,
    /// User input.
    User,
    /// AI assistant response.
    Assistant,
}

impl Role {
    /// Returns true if this is a user-visible role.
    pub fn is_user_visible(&self) -> bool {
        matches!(self, Self::User | Self::Assistant)
    }
}

/// An immutable message within a conversation.
///
/// # Invariants
///
/// - `id` is globally unique
/// - `content` is non-empty (validated at construction)
/// - `created_at` is set at construction and never changes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Unique identifier for this message.
    id: MessageId,

    /// The role of the message sender.
    role: Role,

    /// The content of the message.
    content: String,

    /// When the message was created.
    created_at: Timestamp,
}

impl Message {
    /// Creates a new message with the given role and content.
    ///
    /// # Errors
    ///
    /// - `ValidationFailed` if content is empty
    pub fn new(role: Role, content: impl Into<String>) -> Result<Self, DomainError> {
        let content = content.into();
        Self::validate_content(&content)?;

        Ok(Self {
            id: MessageId::new(),
            role,
            content,
            created_at: Timestamp::now(),
        })
    }

    /// Creates a user message.
    ///
    /// # Errors
    ///
    /// - `ValidationFailed` if content is empty
    pub fn user(content: impl Into<String>) -> Result<Self, DomainError> {
        Self::new(Role::User, content)
    }

    /// Creates an assistant message.
    ///
    /// # Errors
    ///
    /// - `ValidationFailed` if content is empty
    pub fn assistant(content: impl Into<String>) -> Result<Self, DomainError> {
        Self::new(Role::Assistant, content)
    }

    /// Creates a system message.
    ///
    /// # Errors
    ///
    /// - `ValidationFailed` if content is empty
    pub fn system(content: impl Into<String>) -> Result<Self, DomainError> {
        Self::new(Role::System, content)
    }

    /// Reconstitutes a message from persistence (no validation).
    pub fn reconstitute(
        id: MessageId,
        role: Role,
        content: String,
        created_at: Timestamp,
    ) -> Self {
        Self {
            id,
            role,
            content,
            created_at,
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Accessors
    // ─────────────────────────────────────────────────────────────────────────

    /// Returns the message ID.
    pub fn id(&self) -> &MessageId {
        &self.id
    }

    /// Returns the role.
    pub fn role(&self) -> Role {
        self.role
    }

    /// Returns the content.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns when the message was created.
    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    /// Returns true if this message is from the user.
    pub fn is_user(&self) -> bool {
        self.role == Role::User
    }

    /// Returns true if this message is from the assistant.
    pub fn is_assistant(&self) -> bool {
        self.role == Role::Assistant
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Private helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn validate_content(content: &str) -> Result<(), DomainError> {
        if content.trim().is_empty() {
            return Err(DomainError::validation(
                "content",
                "Message content cannot be empty",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod message_id {
        use super::*;

        #[test]
        fn generates_unique_values() {
            let id1 = MessageId::new();
            let id2 = MessageId::new();
            assert_ne!(id1, id2);
        }

        #[test]
        fn parses_from_valid_string() {
            let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
            let id: MessageId = uuid_str.parse().unwrap();
            assert_eq!(id.to_string(), uuid_str);
        }

        #[test]
        fn from_uuid_preserves_value() {
            let uuid = Uuid::new_v4();
            let id = MessageId::from_uuid(uuid);
            assert_eq!(id.as_uuid(), &uuid);
        }
    }

    mod role {
        use super::*;

        #[test]
        fn user_is_visible() {
            assert!(Role::User.is_user_visible());
        }

        #[test]
        fn assistant_is_visible() {
            assert!(Role::Assistant.is_user_visible());
        }

        #[test]
        fn system_is_not_visible() {
            assert!(!Role::System.is_user_visible());
        }

        #[test]
        fn serializes_to_snake_case() {
            let json = serde_json::to_string(&Role::User).unwrap();
            assert_eq!(json, "\"user\"");
        }
    }

    mod message_construction {
        use super::*;

        #[test]
        fn new_creates_message_with_role() {
            let msg = Message::new(Role::User, "Hello").unwrap();
            assert_eq!(msg.role(), Role::User);
            assert_eq!(msg.content(), "Hello");
        }

        #[test]
        fn user_creates_user_message() {
            let msg = Message::user("Hello").unwrap();
            assert!(msg.is_user());
            assert!(!msg.is_assistant());
        }

        #[test]
        fn assistant_creates_assistant_message() {
            let msg = Message::assistant("Hi there").unwrap();
            assert!(msg.is_assistant());
            assert!(!msg.is_user());
        }

        #[test]
        fn system_creates_system_message() {
            let msg = Message::system("You are helpful").unwrap();
            assert_eq!(msg.role(), Role::System);
        }

        #[test]
        fn rejects_empty_content() {
            let result = Message::new(Role::User, "");
            assert!(result.is_err());
        }

        #[test]
        fn rejects_whitespace_only_content() {
            let result = Message::new(Role::User, "   ");
            assert!(result.is_err());
        }

        #[test]
        fn sets_created_at() {
            let msg = Message::user("Hello").unwrap();
            // Should be very recent (within last second)
            let now = Timestamp::now();
            assert!(msg.created_at().as_datetime() <= now.as_datetime());
        }
    }

    mod message_reconstitute {
        use super::*;

        #[test]
        fn reconstitute_preserves_all_fields() {
            let id = MessageId::new();
            let created_at = Timestamp::now();

            let msg = Message::reconstitute(
                id,
                Role::User,
                "Test content".to_string(),
                created_at,
            );

            assert_eq!(msg.id(), &id);
            assert_eq!(msg.role(), Role::User);
            assert_eq!(msg.content(), "Test content");
            assert_eq!(msg.created_at(), &created_at);
        }
    }
}
