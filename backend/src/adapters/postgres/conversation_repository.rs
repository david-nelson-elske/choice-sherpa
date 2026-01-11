//! PostgreSQL implementation of ConversationRepository.
//!
//! Persists Conversation aggregates with messages to PostgreSQL.

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::conversation::{Conversation, ConversationState, Message, MessageId, Role};
use crate::domain::foundation::{ComponentId, ConversationId, DomainError, ErrorCode, Timestamp};
use crate::ports::ConversationRepository;

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
    async fn save(&self, conversation: &Conversation) -> Result<(), DomainError> {
        let mut tx = self.pool.begin().await.map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Failed to begin transaction: {}", e),
            )
        })?;

        // Insert conversation
        sqlx::query(
            r#"
            INSERT INTO conversations (id, component_id, state, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(conversation.id().as_uuid())
        .bind(conversation.component_id().as_uuid())
        .bind(state_to_str(conversation.state()))
        .bind(conversation.created_at().as_datetime())
        .bind(conversation.updated_at().as_datetime())
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            if e.to_string().contains("unique_component") || e.to_string().contains("duplicate") {
                DomainError::new(
                    ErrorCode::MembershipExists, // Reusing for "already exists"
                    format!(
                        "Conversation already exists for component: {}",
                        conversation.component_id()
                    ),
                )
            } else {
                DomainError::new(
                    ErrorCode::DatabaseError,
                    format!("Failed to insert conversation: {}", e),
                )
            }
        })?;

        // Insert all messages
        for message in conversation.messages() {
            insert_message(&mut tx, conversation.id(), message).await?;
        }

        tx.commit().await.map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Failed to commit transaction: {}", e),
            )
        })?;

        Ok(())
    }

    async fn update(&self, conversation: &Conversation) -> Result<(), DomainError> {
        let result = sqlx::query(
            r#"
            UPDATE conversations SET
                state = $2,
                updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(conversation.id().as_uuid())
        .bind(state_to_str(conversation.state()))
        .bind(conversation.updated_at().as_datetime())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Failed to update conversation: {}", e),
            )
        })?;

        if result.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::ConversationNotFound,
                format!("Conversation not found: {}", conversation.id()),
            ));
        }

        Ok(())
    }

    async fn add_message(
        &self,
        conversation_id: &ConversationId,
        message: &Message,
    ) -> Result<(), DomainError> {
        // First check if conversation exists
        let exists = self.conversation_exists(conversation_id).await?;
        if !exists {
            return Err(DomainError::new(
                ErrorCode::ConversationNotFound,
                format!("Conversation not found: {}", conversation_id),
            ));
        }

        let mut tx = self.pool.begin().await.map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Failed to begin transaction: {}", e),
            )
        })?;

        insert_message(&mut tx, conversation_id, message).await?;

        // Update conversation's updated_at
        sqlx::query("UPDATE conversations SET updated_at = NOW() WHERE id = $1")
            .bind(conversation_id.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                DomainError::new(
                    ErrorCode::DatabaseError,
                    format!("Failed to update conversation timestamp: {}", e),
                )
            })?;

        tx.commit().await.map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Failed to commit transaction: {}", e),
            )
        })?;

        Ok(())
    }

    async fn find_by_id(&self, id: &ConversationId) -> Result<Option<Conversation>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, component_id, state, created_at, updated_at
            FROM conversations WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Failed to fetch conversation: {}", e),
            )
        })?;

        match row {
            Some(row) => {
                let messages = load_messages(&self.pool, id).await?;
                let conversation = row_to_conversation(row, messages)?;
                Ok(Some(conversation))
            }
            None => Ok(None),
        }
    }

    async fn find_by_component(
        &self,
        component_id: &ComponentId,
    ) -> Result<Option<Conversation>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, component_id, state, created_at, updated_at
            FROM conversations WHERE component_id = $1
            "#,
        )
        .bind(component_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Failed to fetch conversation by component: {}", e),
            )
        })?;

        match row {
            Some(row) => {
                let id: uuid::Uuid = row.get("id");
                let conversation_id = ConversationId::from_uuid(id);
                let messages = load_messages(&self.pool, &conversation_id).await?;
                let conversation = row_to_conversation(row, messages)?;
                Ok(Some(conversation))
            }
            None => Ok(None),
        }
    }

    async fn exists_for_component(&self, component_id: &ComponentId) -> Result<bool, DomainError> {
        let result: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM conversations WHERE component_id = $1")
                .bind(component_id.as_uuid())
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    DomainError::new(
                        ErrorCode::DatabaseError,
                        format!("Failed to check conversation existence: {}", e),
                    )
                })?;

        Ok(result.0 > 0)
    }

    async fn delete(&self, id: &ConversationId) -> Result<(), DomainError> {
        let result = sqlx::query("DELETE FROM conversations WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| {
                DomainError::new(
                    ErrorCode::DatabaseError,
                    format!("Failed to delete conversation: {}", e),
                )
            })?;

        if result.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::ConversationNotFound,
                format!("Conversation not found: {}", id),
            ));
        }

        Ok(())
    }
}

impl PostgresConversationRepository {
    async fn conversation_exists(&self, id: &ConversationId) -> Result<bool, DomainError> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM conversations WHERE id = $1")
            .bind(id.as_uuid())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                DomainError::new(
                    ErrorCode::DatabaseError,
                    format!("Failed to check conversation existence: {}", e),
                )
            })?;

        Ok(result.0 > 0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper functions
// ─────────────────────────────────────────────────────────────────────────────

async fn insert_message(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    conversation_id: &ConversationId,
    message: &Message,
) -> Result<(), DomainError> {
    sqlx::query(
        r#"
        INSERT INTO messages (id, conversation_id, role, content, created_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(message.id().as_uuid())
    .bind(conversation_id.as_uuid())
    .bind(role_to_str(message.role()))
    .bind(message.content())
    .bind(message.created_at().as_datetime())
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        DomainError::new(
            ErrorCode::DatabaseError,
            format!("Failed to insert message: {}", e),
        )
    })?;

    Ok(())
}

async fn load_messages(
    pool: &PgPool,
    conversation_id: &ConversationId,
) -> Result<Vec<Message>, DomainError> {
    let rows = sqlx::query(
        r#"
        SELECT id, role, content, created_at
        FROM messages
        WHERE conversation_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(conversation_id.as_uuid())
    .fetch_all(pool)
    .await
    .map_err(|e| {
        DomainError::new(
            ErrorCode::DatabaseError,
            format!("Failed to load messages: {}", e),
        )
    })?;

    let messages: Result<Vec<Message>, DomainError> = rows
        .into_iter()
        .map(|row| {
            let id: uuid::Uuid = row.get("id");
            let role: String = row.get("role");
            let content: String = row.get("content");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

            Ok(Message::reconstitute(
                MessageId::from_uuid(id),
                str_to_role(&role)?,
                content,
                Timestamp::from_datetime(created_at),
            ))
        })
        .collect();

    messages
}

fn row_to_conversation(
    row: sqlx::postgres::PgRow,
    messages: Vec<Message>,
) -> Result<Conversation, DomainError> {
    let id: uuid::Uuid = row.get("id");
    let component_id: uuid::Uuid = row.get("component_id");
    let state: String = row.get("state");
    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

    Ok(Conversation::reconstitute(
        ConversationId::from_uuid(id),
        ComponentId::from_uuid(component_id),
        str_to_state(&state)?,
        messages,
        Timestamp::from_datetime(created_at),
        Timestamp::from_datetime(updated_at),
    ))
}

fn state_to_str(state: ConversationState) -> &'static str {
    match state {
        ConversationState::Initializing => "initializing",
        ConversationState::Ready => "ready",
        ConversationState::InProgress => "in_progress",
        ConversationState::Confirmed => "confirmed",
        ConversationState::Complete => "complete",
    }
}

fn str_to_state(s: &str) -> Result<ConversationState, DomainError> {
    match s {
        "initializing" => Ok(ConversationState::Initializing),
        "ready" => Ok(ConversationState::Ready),
        "in_progress" => Ok(ConversationState::InProgress),
        "confirmed" => Ok(ConversationState::Confirmed),
        "complete" => Ok(ConversationState::Complete),
        _ => Err(DomainError::new(
            ErrorCode::DatabaseError,
            format!("Invalid conversation state: {}", s),
        )),
    }
}

fn role_to_str(role: Role) -> &'static str {
    match role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
    }
}

fn str_to_role(s: &str) -> Result<Role, DomainError> {
    match s {
        "system" => Ok(Role::System),
        "user" => Ok(Role::User),
        "assistant" => Ok(Role::Assistant),
        _ => Err(DomainError::new(
            ErrorCode::DatabaseError,
            format!("Invalid message role: {}", s),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod state_conversion {
        use super::*;

        #[test]
        fn all_states_round_trip() {
            for state in [
                ConversationState::Initializing,
                ConversationState::Ready,
                ConversationState::InProgress,
                ConversationState::Confirmed,
                ConversationState::Complete,
            ] {
                let s = state_to_str(state);
                let recovered = str_to_state(s).unwrap();
                assert_eq!(recovered, state);
            }
        }
    }

    mod role_conversion {
        use super::*;

        #[test]
        fn all_roles_round_trip() {
            for role in [Role::System, Role::User, Role::Assistant] {
                let s = role_to_str(role);
                let recovered = str_to_role(s).unwrap();
                assert_eq!(recovered, role);
            }
        }
    }
}
