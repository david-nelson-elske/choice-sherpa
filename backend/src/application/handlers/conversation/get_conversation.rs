//! GetConversationHandler - Query handler for retrieving conversation data.

use std::sync::Arc;

use crate::domain::foundation::{ComponentId, ConversationId, DomainError, ErrorCode};
use crate::ports::{ConversationReader, ConversationView};

/// Query to get a conversation.
#[derive(Debug, Clone)]
pub struct GetConversationQuery {
    pub component_id: ComponentId,
}

/// Handler for getting conversations.
pub struct GetConversationHandler {
    reader: Arc<dyn ConversationReader>,
}

impl GetConversationHandler {
    pub fn new(reader: Arc<dyn ConversationReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(
        &self,
        query: GetConversationQuery,
    ) -> Result<ConversationView, DomainError> {
        match self.reader.get_by_component(&query.component_id).await {
            Ok(Some(view)) => Ok(view),
            Ok(None) => Err(DomainError::new(
                ErrorCode::ConversationNotFound,
                format!("No conversation found for component {}", query.component_id),
            )),
            Err(e) => Err(e),
        }
    }

    pub async fn get_by_id(&self, conversation_id: &ConversationId) -> Result<ConversationView, DomainError> {
        match self.reader.get(conversation_id).await {
            Ok(Some(view)) => Ok(view),
            Ok(None) => Err(DomainError::new(
                ErrorCode::ConversationNotFound,
                format!("Conversation not found: {}", conversation_id),
            )),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::conversation::ConversationState;
    use crate::domain::foundation::Timestamp;
    use crate::ports::{ConversationReader, MessageList, MessageListOptions};
    use async_trait::async_trait;

    #[derive(Clone)]
    struct MockReader {
        view: Option<ConversationView>,
    }

    impl MockReader {
        fn with_view(view: ConversationView) -> Self {
            Self { view: Some(view) }
        }

        fn empty() -> Self {
            Self { view: None }
        }
    }

    #[async_trait]
    impl ConversationReader for MockReader {
        async fn get(
            &self,
            _id: &ConversationId,
        ) -> Result<Option<ConversationView>, DomainError> {
            Ok(self.view.clone())
        }

        async fn get_by_component(
            &self,
            _component_id: &ComponentId,
        ) -> Result<Option<ConversationView>, DomainError> {
            Ok(self.view.clone())
        }

        async fn get_messages(
            &self,
            _conversation_id: &ConversationId,
            _options: &MessageListOptions,
        ) -> Result<MessageList, DomainError> {
            Ok(MessageList {
                items: vec![],
                total: 0,
                has_more: false,
            })
        }
    }

    #[tokio::test]
    async fn get_conversation_returns_view_when_found() {
        let now = Timestamp::now();
        let view = ConversationView {
            id: ConversationId::new(),
            component_id: ComponentId::new(),
            state: ConversationState::Ready,
            message_count: 0,
            created_at: now,
            updated_at: now,
        };

        let reader = Arc::new(MockReader::with_view(view.clone()));
        let handler = GetConversationHandler::new(reader);

        let query = GetConversationQuery {
            component_id: view.component_id,
        };

        let result = handler.handle(query).await.unwrap();
        assert_eq!(result.id, view.id);
        assert_eq!(result.component_id, view.component_id);
    }

    #[tokio::test]
    async fn get_conversation_returns_error_when_not_found() {
        let reader = Arc::new(MockReader::empty());
        let handler = GetConversationHandler::new(reader);

        let query = GetConversationQuery {
            component_id: ComponentId::new(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_err());
    }
}
