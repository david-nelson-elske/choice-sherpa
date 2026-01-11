//! SendMessageHandler - Command handler for sending messages in conversations.

use std::sync::Arc;

use crate::domain::conversation::Conversation;
use crate::domain::foundation::{ComponentId, ConversationId, DomainError, ErrorCode};
use crate::domain::proact::Message;
use crate::ports::{ConversationRepository, ConversationRepositoryError};

/// Command to send a message in a conversation.
#[derive(Debug, Clone)]
pub struct SendMessageCommand {
    pub component_id: ComponentId,
    pub content: String,
}

/// Result of sending a message.
#[derive(Debug, Clone)]
pub struct SendMessageResult {
    pub conversation_id: ConversationId,
    pub message: Message,
}

/// Handler for sending messages.
pub struct SendMessageHandler {
    repository: Arc<dyn ConversationRepository>,
}

impl SendMessageHandler {
    pub fn new(repository: Arc<dyn ConversationRepository>) -> Self {
        Self { repository }
    }

    pub async fn handle(
        &self,
        cmd: SendMessageCommand,
    ) -> Result<SendMessageResult, DomainError> {
        // 1. Find or create conversation
        let mut conversation = match self.repository.find_by_component(cmd.component_id).await {
            Ok(Some(conv)) => conv,
            Ok(None) => {
                return Err(DomainError::new(
                    ErrorCode::ConversationNotFound,
                    format!("No conversation found for component {}", cmd.component_id),
                ));
            }
            Err(e) => {
                return Err(map_repository_error(e));
            }
        };

        // 2. Add user message
        conversation.add_user_message(&cmd.content).map_err(|e| e)?;

        // 3. Get the message that was just added
        let message = conversation
            .last_message()
            .expect("Message should exist after adding")
            .clone();

        // 4. Persist updated conversation
        self.repository
            .update(&conversation)
            .await
            .map_err(map_repository_error)?;

        Ok(SendMessageResult {
            conversation_id: conversation.id(),
            message,
        })
    }
}

fn map_repository_error(e: ConversationRepositoryError) -> DomainError {
    match e {
        ConversationRepositoryError::NotFound(id) => {
            DomainError::new(ErrorCode::ConversationNotFound, format!("Conversation not found: {}", id))
        }
        ConversationRepositoryError::Database(msg) => {
            DomainError::new(ErrorCode::DatabaseError, msg)
        }
        ConversationRepositoryError::Serialization(msg) => {
            DomainError::new(ErrorCode::InternalError, format!("Serialization error: {}", msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::conversation::ConversationState;
    use crate::domain::foundation::ComponentType;
    use crate::ports::ConversationRepository;
    use async_trait::async_trait;
    use std::sync::Mutex;

    #[derive(Clone)]
    struct MockRepository {
        conversations: Arc<Mutex<Vec<Conversation>>>,
    }

    impl MockRepository {
        fn new() -> Self {
            Self {
                conversations: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn add_conversation(&self, conv: Conversation) {
            self.conversations.lock().unwrap().push(conv);
        }
    }

    #[async_trait]
    impl ConversationRepository for MockRepository {
        async fn save(&self, conversation: &Conversation) -> Result<(), ConversationRepositoryError> {
            self.conversations.lock().unwrap().push(conversation.clone());
            Ok(())
        }

        async fn update(&self, conversation: &Conversation) -> Result<(), ConversationRepositoryError> {
            let mut convs = self.conversations.lock().unwrap();
            if let Some(pos) = convs.iter().position(|c| c.id() == conversation.id()) {
                convs[pos] = conversation.clone();
                Ok(())
            } else {
                Err(ConversationRepositoryError::NotFound(conversation.id()))
            }
        }

        async fn find_by_id(
            &self,
            id: ConversationId,
        ) -> Result<Option<Conversation>, ConversationRepositoryError> {
            let convs = self.conversations.lock().unwrap();
            Ok(convs.iter().find(|c| c.id() == id).cloned())
        }

        async fn find_by_component(
            &self,
            component_id: ComponentId,
        ) -> Result<Option<Conversation>, ConversationRepositoryError> {
            let convs = self.conversations.lock().unwrap();
            Ok(convs.iter().find(|c| c.component_id() == component_id).cloned())
        }

        async fn append_message(
            &self,
            _conversation_id: ConversationId,
            _message: &Message,
        ) -> Result<(), ConversationRepositoryError> {
            Ok(())
        }

        async fn delete(&self, _id: ConversationId) -> Result<(), ConversationRepositoryError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn send_message_adds_user_message_to_conversation() {
        let repo = Arc::new(MockRepository::new());
        let handler = SendMessageHandler::new(repo.clone());

        // Create a conversation
        let component_id = ComponentId::new();
        let mut conversation = Conversation::new(component_id, ComponentType::IssueRaising);
        conversation.mark_ready().unwrap();
        repo.add_conversation(conversation.clone());

        // Send message
        let cmd = SendMessageCommand {
            component_id,
            content: "Hello, I need help with a decision".to_string(),
        };

        let result = handler.handle(cmd).await.unwrap();

        assert_eq!(result.conversation_id, conversation.id());
        assert_eq!(result.message.content, "Hello, I need help with a decision");

        // Verify conversation was updated
        let updated = repo.find_by_component(component_id).await.unwrap().unwrap();
        assert_eq!(updated.message_count(), 1);
        assert_eq!(updated.state(), ConversationState::InProgress);
    }

    #[tokio::test]
    async fn send_message_fails_if_conversation_not_found() {
        let repo = Arc::new(MockRepository::new());
        let handler = SendMessageHandler::new(repo);

        let cmd = SendMessageCommand {
            component_id: ComponentId::new(),
            content: "Hello".to_string(),
        };

        let result = handler.handle(cmd).await;
        assert!(result.is_err());
    }
}
