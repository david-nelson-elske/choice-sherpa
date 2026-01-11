//! GetConversationHandler - Query handler for retrieving conversation data.

use std::sync::Arc;

use crate::domain::foundation::{ComponentId, ConversationId, DomainError, ErrorCode};
use crate::ports::{ConversationReader, ConversationReaderError, ConversationView};

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
            Err(e) => Err(map_reader_error(e)),
        }
    }

    pub async fn get_by_id(&self, conversation_id: &ConversationId) -> Result<ConversationView, DomainError> {
        match self.reader.get_by_id(conversation_id).await {
            Ok(Some(view)) => Ok(view),
            Ok(None) => Err(DomainError::new(
                ErrorCode::ConversationNotFound,
                format!("Conversation not found: {}", conversation_id),
            )),
            Err(e) => Err(map_reader_error(e)),
        }
    }
}

fn map_reader_error(e: ConversationReaderError) -> DomainError {
    match e {
        ConversationReaderError::Database(msg) => {
            DomainError::new(ErrorCode::DatabaseError, msg)
        }
        ConversationReaderError::Serialization(msg) => {
            DomainError::new(ErrorCode::InternalError, format!("Serialization error: {}", msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::conversation::{AgentPhase, ConversationState};
    use crate::domain::foundation::{ComponentType, Timestamp};
    use crate::domain::proact::Message;
    use crate::ports::ConversationReader;
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
        async fn get_by_component(
            &self,
            _component_id: &ComponentId,
        ) -> Result<Option<ConversationView>, ConversationReaderError> {
            Ok(self.view.clone())
        }

        async fn get_by_id(
            &self,
            _conversation_id: &ConversationId,
        ) -> Result<Option<ConversationView>, ConversationReaderError> {
            Ok(self.view.clone())
        }

        async fn get_message_count(
            &self,
            _conversation_id: ConversationId,
        ) -> Result<usize, ConversationReaderError> {
            Ok(self.view.as_ref().map(|v| v.message_count).unwrap_or(0))
        }

        async fn get_recent_messages(
            &self,
            _conversation_id: ConversationId,
            _limit: usize,
        ) -> Result<Vec<Message>, ConversationReaderError> {
            Ok(self.view.as_ref().map(|v| v.messages.clone()).unwrap_or_default())
        }
    }

    #[tokio::test]
    async fn get_conversation_returns_view_when_found() {
        let view = ConversationView {
            id: ConversationId::new(),
            component_id: ComponentId::new(),
            component_type: ComponentType::IssueRaising,
            messages: vec![],
            state: ConversationState::Ready,
            current_phase: AgentPhase::Intro,
            pending_extraction: None,
            message_count: 0,
            last_message_at: None,
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
