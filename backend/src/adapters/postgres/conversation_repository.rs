//! PostgreSQL implementation of ConversationRepository.
//!
//! Persists Conversation aggregates with messages to PostgreSQL.

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::conversation::{AgentPhase, Conversation, ConversationState};
use crate::domain::foundation::{
    ComponentId, ComponentType, ConversationId, Timestamp,
};
use crate::domain::proact::{Message, MessageId, MessageMetadata, Role};
use crate::ports::{ConversationRepository, ConversationRepositoryError as RepositoryError};

/// PostgreSQL implementation of ConversationRepository.
#[derive(Clone)]
pub struct PostgresConversationRepository {
    pool: PgPool,
}

impl PostgresConversationRepository {
    /// Creates a new PostgresConversationRepository.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConversationRepository for PostgresConversationRepository {
    async fn save(&self, conversation: &Conversation) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(|e| {
            RepositoryError::Database(format!("Failed to start transaction: {}", e))
        })?;

        // Insert conversation
        sqlx::query(
            r#"
            INSERT INTO conversations (
                id, component_id, component_type, state, current_phase,
                pending_extraction, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(conversation.id().as_uuid())
        .bind(conversation.component_id().as_uuid())
        .bind(component_type_to_str(conversation.component_type()))
        .bind(conversation_state_to_str(conversation.state()))
        .bind(agent_phase_to_str(conversation.current_phase()))
        .bind(conversation.pending_extraction())
        .bind(conversation.created_at().as_datetime())
        .bind(conversation.updated_at().as_datetime())
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            RepositoryError::Database(format!("Failed to insert conversation: {}", e))
        })?;

        // Insert messages
        for message in conversation.messages() {
            sqlx::query(
                r#"
                INSERT INTO messages (id, conversation_id, role, content, created_at)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(message.id.as_uuid())
            .bind(conversation.id().as_uuid())
            .bind(role_to_str(message.role))
            .bind(&message.content)
            .bind(message.timestamp.as_datetime())
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                RepositoryError::Database(format!("Failed to insert message: {}", e))
            })?;
        }

        tx.commit().await.map_err(|e| {
            RepositoryError::Database(format!("Failed to commit transaction: {}", e))
        })?;

        Ok(())
    }

    async fn update(&self, conversation: &Conversation) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(|e| {
            RepositoryError::Database(format!("Failed to start transaction: {}", e))
        })?;

        // Update conversation
        let result = sqlx::query(
            r#"
            UPDATE conversations SET
                state = $2,
                current_phase = $3,
                pending_extraction = $4,
                updated_at = $5
            WHERE id = $1
            "#,
        )
        .bind(conversation.id().as_uuid())
        .bind(conversation_state_to_str(conversation.state()))
        .bind(agent_phase_to_str(conversation.current_phase()))
        .bind(conversation.pending_extraction())
        .bind(conversation.updated_at().as_datetime())
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            RepositoryError::Database(format!("Failed to update conversation: {}", e))
        })?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(conversation.id()));
        }

        // Delete existing messages
        sqlx::query("DELETE FROM messages WHERE conversation_id = $1")
            .bind(conversation.id().as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                RepositoryError::Database(format!("Failed to delete messages: {}", e))
            })?;

        // Insert all messages
        for message in conversation.messages() {
            sqlx::query(
                r#"
                INSERT INTO messages (id, conversation_id, role, content, created_at)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(message.id.as_uuid())
            .bind(conversation.id().as_uuid())
            .bind(role_to_str(message.role))
            .bind(&message.content)
            .bind(message.timestamp.as_datetime())
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                RepositoryError::Database(format!("Failed to insert message: {}", e))
            })?;
        }

        tx.commit().await.map_err(|e| {
            RepositoryError::Database(format!("Failed to commit transaction: {}", e))
        })?;

        Ok(())
    }

    async fn find_by_id(
        &self,
        id: ConversationId,
    ) -> Result<Option<Conversation>, RepositoryError> {
        // Fetch conversation
        let conv_row = sqlx::query(
            r#"
            SELECT id, component_id, component_type, state, current_phase,
                   pending_extraction, created_at, updated_at
            FROM conversations
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            RepositoryError::Database(format!(
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
        .bind(id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            RepositoryError::Database(format!("Failed to fetch messages: {}", e))
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

        // Reconstruct conversation
        let id_uuid: uuid::Uuid = conv_row.get("id");
        let component_id_uuid: uuid::Uuid = conv_row.get("component_id");
        let component_type_str: &str = conv_row.get("component_type");
        let state_str: &str = conv_row.get("state");
        let phase_str: &str = conv_row.get("current_phase");
        let pending_extraction: Option<serde_json::Value> = conv_row.get("pending_extraction");
        let created_at: chrono::DateTime<chrono::Utc> = conv_row.get("created_at");
        let updated_at: chrono::DateTime<chrono::Utc> = conv_row.get("updated_at");

        let conversation = Conversation::reconstitute(
            ConversationId::from_uuid(id_uuid),
            ComponentId::from_uuid(component_id_uuid),
            str_to_component_type(component_type_str)?,
            messages,
            str_to_conversation_state(state_str)?,
            str_to_agent_phase(phase_str)?,
            pending_extraction,
            Timestamp::from_datetime(created_at),
            Timestamp::from_datetime(updated_at),
        );

        Ok(Some(conversation))
    }

    async fn find_by_component(
        &self,
        component_id: ComponentId,
    ) -> Result<Option<Conversation>, RepositoryError> {
        // Fetch conversation by component_id
        let conv_row = sqlx::query(
            r#"
            SELECT id
            FROM conversations
            WHERE component_id = $1
            "#,
        )
        .bind(component_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            RepositoryError::Database(format!(
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

        // Reuse find_by_id
        self.find_by_id(conversation_id).await
    }

    async fn append_message(
        &self,
        conversation_id: ConversationId,
        message: &Message,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(|e| {
            RepositoryError::Database(format!("Failed to start transaction: {}", e))
        })?;

        // Insert message
        sqlx::query(
            r#"
            INSERT INTO messages (id, conversation_id, role, content, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(message.id.as_uuid())
        .bind(conversation_id.as_uuid())
        .bind(role_to_str(message.role))
        .bind(&message.content)
        .bind(message.timestamp.as_datetime())
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            RepositoryError::Database(format!("Failed to insert message: {}", e))
        })?;

        // Update conversation updated_at
        sqlx::query(
            r#"
            UPDATE conversations SET updated_at = $2
            WHERE id = $1
            "#,
        )
        .bind(conversation_id.as_uuid())
        .bind(chrono::Utc::now())
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            RepositoryError::Database(format!("Failed to update conversation timestamp: {}", e))
        })?;

        tx.commit().await.map_err(|e| {
            RepositoryError::Database(format!("Failed to commit transaction: {}", e))
        })?;

        Ok(())
    }

    async fn delete(&self, id: ConversationId) -> Result<(), RepositoryError> {
        let result = sqlx::query("DELETE FROM conversations WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| {
                RepositoryError::Database(format!(
                    "Failed to delete conversation: {}",
                    e
                ))
            })?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(id));
        }

        Ok(())
    }
}

// === Helper Functions ===

fn conversation_state_to_str(state: ConversationState) -> &'static str {
    match state {
        ConversationState::Initializing => "initializing",
        ConversationState::Ready => "ready",
        ConversationState::InProgress => "in_progress",
        ConversationState::Confirmed => "confirmed",
        ConversationState::Complete => "complete",
    }
}

fn str_to_conversation_state(s: &str) -> Result<ConversationState, RepositoryError> {
    match s {
        "initializing" => Ok(ConversationState::Initializing),
        "ready" => Ok(ConversationState::Ready),
        "in_progress" => Ok(ConversationState::InProgress),
        "confirmed" => Ok(ConversationState::Confirmed),
        "complete" => Ok(ConversationState::Complete),
        _ => Err(RepositoryError::Serialization(format!(
            "Invalid conversation state: {}",
            s
        ))),
    }
}

fn agent_phase_to_str(phase: AgentPhase) -> &'static str {
    match phase {
        AgentPhase::Intro => "intro",
        AgentPhase::Gather => "gather",
        AgentPhase::Clarify => "clarify",
        AgentPhase::Extract => "extract",
        AgentPhase::Confirm => "confirm",
    }
}

fn str_to_agent_phase(s: &str) -> Result<AgentPhase, RepositoryError> {
    match s {
        "intro" => Ok(AgentPhase::Intro),
        "gather" => Ok(AgentPhase::Gather),
        "clarify" => Ok(AgentPhase::Clarify),
        "extract" => Ok(AgentPhase::Extract),
        "confirm" => Ok(AgentPhase::Confirm),
        _ => Err(RepositoryError::Serialization(format!(
            "Invalid agent phase: {}",
            s
        ))),
    }
}

fn component_type_to_str(component_type: ComponentType) -> &'static str {
    match component_type {
        ComponentType::IssueRaising => "issue_raising",
        ComponentType::ProblemFrame => "problem_frame",
        ComponentType::Objectives => "objectives",
        ComponentType::Alternatives => "alternatives",
        ComponentType::Consequences => "consequences",
        ComponentType::Tradeoffs => "tradeoffs",
        ComponentType::Recommendation => "recommendation",
        ComponentType::DecisionQuality => "decision_quality",
        ComponentType::NotesNextSteps => "notes_next_steps",
    }
}

fn str_to_component_type(s: &str) -> Result<ComponentType, RepositoryError> {
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
        _ => Err(RepositoryError::Serialization(format!(
            "Invalid component type: {}",
            s
        ))),
    }
}

fn role_to_str(role: Role) -> &'static str {
    match role {
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::System => "system",
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
