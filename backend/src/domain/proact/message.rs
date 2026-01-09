//! Message types for component conversations.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::foundation::Timestamp;

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

/// Role of the message sender.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
    System,
}

/// Metadata associated with a message.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Approximate token count.
    #[serde(default)]
    pub token_count: Option<u32>,

    /// Data extracted from this message.
    #[serde(default)]
    pub extracted_data: serde_json::Value,
}

/// A single message in a component conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub role: Role,
    pub content: String,
    pub metadata: MessageMetadata,
    pub timestamp: Timestamp,
}

impl Message {
    /// Creates a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            id: MessageId::new(),
            role: Role::User,
            content: content.into(),
            metadata: MessageMetadata::default(),
            timestamp: Timestamp::now(),
        }
    }

    /// Creates an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            id: MessageId::new(),
            role: Role::Assistant,
            content: content.into(),
            metadata: MessageMetadata::default(),
            timestamp: Timestamp::now(),
        }
    }

    /// Creates a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            id: MessageId::new(),
            role: Role::System,
            content: content.into(),
            metadata: MessageMetadata::default(),
            timestamp: Timestamp::now(),
        }
    }

    /// Sets the token count metadata.
    pub fn with_token_count(mut self, count: u32) -> Self {
        self.metadata.token_count = Some(count);
        self
    }

    /// Sets the extracted data metadata.
    pub fn with_extracted_data(mut self, data: serde_json::Value) -> Self {
        self.metadata.extracted_data = data;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn message_id_generates_unique_values() {
        let id1 = MessageId::new();
        let id2 = MessageId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn user_message_has_user_role() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn assistant_message_has_assistant_role() {
        let msg = Message::assistant("Hi there!");
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.content, "Hi there!");
    }

    #[test]
    fn system_message_has_system_role() {
        let msg = Message::system("You are a helpful assistant.");
        assert_eq!(msg.role, Role::System);
        assert_eq!(msg.content, "You are a helpful assistant.");
    }

    #[test]
    fn message_with_token_count_sets_metadata() {
        let msg = Message::user("Hello").with_token_count(5);
        assert_eq!(msg.metadata.token_count, Some(5));
    }

    #[test]
    fn message_with_extracted_data_sets_metadata() {
        let data = json!({"intent": "greeting"});
        let msg = Message::user("Hello").with_extracted_data(data.clone());
        assert_eq!(msg.metadata.extracted_data, data);
    }

    #[test]
    fn role_serializes_to_snake_case() {
        assert_eq!(serde_json::to_string(&Role::User).unwrap(), "\"user\"");
        assert_eq!(serde_json::to_string(&Role::Assistant).unwrap(), "\"assistant\"");
        assert_eq!(serde_json::to_string(&Role::System).unwrap(), "\"system\"");
    }

    #[test]
    fn role_deserializes_from_snake_case() {
        let role: Role = serde_json::from_str("\"user\"").unwrap();
        assert_eq!(role, Role::User);
    }

    #[test]
    fn message_metadata_defaults_correctly() {
        let metadata = MessageMetadata::default();
        assert_eq!(metadata.token_count, None);
        assert_eq!(metadata.extracted_data, serde_json::Value::Null);
    }
}
