//! PostgreSQL implementation of ConversationReader.
//!
//! Provides optimized read access to conversation data.

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::conversation::{AgentPhase, ConversationState};
use crate::domain::foundation::{ComponentId, ComponentType, ConversationId, Timestamp};
use crate::domain::proact::{Message, MessageId, MessageMetadata, Role};
use crate::ports::{ConversationReader, ConversationReaderError, ConversationView};

/// PostgreSQL implementation of ConversationReader.
#[derive(Clone)]
pub struct PostgresConversationReader {
    pool: PgPool,
}

impl PostgresConversationReader {
    /// Creates a new PostgresConversationReader.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConversationReader for PostgresConversationReader {
    async fn get_by_component(
        &self,
        component_id: &ComponentId,
    ) -> Result<Option<ConversationView>, ConversationReaderError> {
        // Fetch conversation
        let conv_row = sqlx::query(
            r#"
            SELECT id, component_id, component_type, state, current_phase,
                   pending_extraction, created_at, updated_at
            FROM conversations
            WHERE component_id = $1
            "#,
        )
        .bind(component_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            ConversationReaderError::Database(format!(
                "Failed to fetch conversation by component: {}",
                e
            ))
        })?;

        let conv_row = match conv_row {
            Some(row) => row,
            None => return Ok(None),
        };

        let id_uuid: uuid::Uuid = conv_row.get("id");
        let conversation_id = ConversationId::from_uuid(id_uuid);

        // Reuse get_by_id
        self.get_by_id(&conversation_id).await
    }

    async fn get_by_id(
        &self,
        conversation_id: &ConversationId,
    ) -> Result<Option<ConversationView>, ConversationReaderError> {
        // Fetch conversation with message count and last message timestamp
        let conv_row = sqlx::query(
            r#"
            SELECT c.id, c.component_id, c.component_type, c.state, c.current_phase,
                   c.pending_extraction, c.created_at, c.updated_at,
                   COUNT(m.id) as message_count,
                   MAX(m.created_at) as last_message_at
            FROM conversations c
            LEFT JOIN messages m ON m.conversation_id = c.id
            WHERE c.id = $1
            GROUP BY c.id, c.component_id, c.component_type, c.state, c.current_phase,
                     c.pending_extraction, c.created_at, c.updated_at
            "#,
        )
        .bind(conversation_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            ConversationReaderError::Database(format!(
                "Failed to fetch conversation: {}",
                e
            ))
        })?;

        let conv_row = match conv_row {
            Some(row) => row,
            None => return Ok(None),
        };

        // Fetch messages
        let message_rows = sqlx::query(
            r#"
            SELECT id, role, content, created_at
            FROM messages
            WHERE conversation_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(conversation_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            ConversationReaderError::Database(format!("Failed to fetch messages: {}", e))
        })?;

        // Reconstruct messages
        let messages: Vec<Message> = message_rows
            .iter()
            .map(|row| {
                let id: uuid::Uuid = row.get("id");
                let role_str: &str = row.get("role");
                let content: String = row.get("content");
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

                Message {
                    id: MessageId::from_uuid(id),
                    role: str_to_role(role_str),
                    content,
                    metadata: MessageMetadata::default(),
                    timestamp: Timestamp::from_datetime(created_at),
                }
            })
            .collect();

        // Build view
        let id_uuid: uuid::Uuid = conv_row.get("id");
        let component_id_uuid: uuid::Uuid = conv_row.get("component_id");
        let component_type_str: &str = conv_row.get("component_type");
        let state_str: &str = conv_row.get("state");
        let phase_str: &str = conv_row.get("current_phase");
        let pending_extraction: Option<serde_json::Value> = conv_row.get("pending_extraction");
        let created_at: chrono::DateTime<chrono::Utc> = conv_row.get("created_at");
        let updated_at: chrono::DateTime<chrono::Utc> = conv_row.get("updated_at");
        let message_count: i64 = conv_row.get("message_count");
        let last_message_at: Option<chrono::DateTime<chrono::Utc>> = conv_row.get("last_message_at");

        let view = ConversationView {
            id: ConversationId::from_uuid(id_uuid),
            component_id: ComponentId::from_uuid(component_id_uuid),
            component_type: str_to_component_type(component_type_str)?,
            messages,
            state: str_to_conversation_state(state_str)?,
            current_phase: str_to_agent_phase(phase_str)?,
            pending_extraction,
            message_count: message_count as usize,
            last_message_at: last_message_at.map(Timestamp::from_datetime),
        };

        Ok(Some(view))
    }

    async fn get_message_count(
        &self,
        conversation_id: ConversationId,
    ) -> Result<usize, ConversationReaderError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM messages
            WHERE conversation_id = $1
            "#,
        )
        .bind(conversation_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            ConversationReaderError::Database(format!("Failed to count messages: {}", e))
        })?;

        let count: i64 = row.get("count");
        Ok(count as usize)
    }

    async fn get_recent_messages(
        &self,
        conversation_id: ConversationId,
        limit: usize,
    ) -> Result<Vec<Message>, ConversationReaderError> {
        let message_rows = sqlx::query(
            r#"
            SELECT id, role, content, created_at
            FROM messages
            WHERE conversation_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(conversation_id.as_uuid())
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            ConversationReaderError::Database(format!("Failed to fetch recent messages: {}", e))
        })?;

        let mut messages: Vec<Message> = message_rows
            .iter()
            .map(|row| {
                let id: uuid::Uuid = row.get("id");
                let role_str: &str = row.get("role");
                let content: String = row.get("content");
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

                Message {
                    id: MessageId::from_uuid(id),
                    role: str_to_role(role_str),
                    content,
                    metadata: MessageMetadata::default(),
                    timestamp: Timestamp::from_datetime(created_at),
                }
            })
            .collect();

        // Reverse to get chronological order
        messages.reverse();

        Ok(messages)
    }
}

// === Helper Functions ===

fn str_to_conversation_state(s: &str) -> Result<ConversationState, ConversationReaderError> {
    match s {
        "initializing" => Ok(ConversationState::Initializing),
        "ready" => Ok(ConversationState::Ready),
        "in_progress" => Ok(ConversationState::InProgress),
        "confirmed" => Ok(ConversationState::Confirmed),
        "complete" => Ok(ConversationState::Complete),
        _ => Err(ConversationReaderError::Serialization(format!(
            "Invalid conversation state: {}",
            s
        ))),
    }
}

fn str_to_agent_phase(s: &str) -> Result<AgentPhase, ConversationReaderError> {
    match s {
        "intro" => Ok(AgentPhase::Intro),
        "gather" => Ok(AgentPhase::Gather),
        "clarify" => Ok(AgentPhase::Clarify),
        "extract" => Ok(AgentPhase::Extract),
        "confirm" => Ok(AgentPhase::Confirm),
        _ => Err(ConversationReaderError::Serialization(format!(
            "Invalid agent phase: {}",
            s
        ))),
    }
}

fn str_to_component_type(s: &str) -> Result<ComponentType, ConversationReaderError> {
    match s {
        "issue_raising" => Ok(ComponentType::IssueRaising),
        "problem_frame" => Ok(ComponentType::ProblemFrame),
        "objectives" => Ok(ComponentType::Objectives),
        "alternatives" => Ok(ComponentType::Alternatives),
        "consequences" => Ok(ComponentType::Consequences),
        "tradeoffs" => Ok(ComponentType::Tradeoffs),
        "recommendation" => Ok(ComponentType::Recommendation),
        "decision_quality" => Ok(ComponentType::DecisionQuality),
        "notes_next_steps" => Ok(ComponentType::NotesNextSteps),
        _ => Err(ConversationReaderError::Serialization(format!(
            "Invalid component type: {}",
            s
        ))),
    }
}

fn str_to_role(s: &str) -> Role {
    match s {
        "user" => Role::User,
        "assistant" => Role::Assistant,
        "system" => Role::System,
        _ => Role::System, // Default fallback
    }
}
