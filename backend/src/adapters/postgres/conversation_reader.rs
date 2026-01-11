//! PostgreSQL implementation of ConversationReader.
//!
//! Provides read-optimized queries for conversation data.

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::conversation::{ConversationState, Role};
use crate::domain::foundation::{ComponentId, ConversationId, DomainError, ErrorCode, Timestamp};
use crate::ports::{ConversationReader, ConversationView, MessageList, MessageListOptions, MessageView};

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
    async fn get(&self, id: &ConversationId) -> Result<Option<ConversationView>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT
                c.id, c.component_id, c.state, c.created_at, c.updated_at,
                COUNT(m.id)::int as message_count
            FROM conversations c
            LEFT JOIN messages m ON c.id = m.conversation_id
            WHERE c.id = $1
            GROUP BY c.id, c.component_id, c.state, c.created_at, c.updated_at
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

        row.map(row_to_view).transpose()
    }

    async fn get_by_component(
        &self,
        component_id: &ComponentId,
    ) -> Result<Option<ConversationView>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT
                c.id, c.component_id, c.state, c.created_at, c.updated_at,
                COUNT(m.id)::int as message_count
            FROM conversations c
            LEFT JOIN messages m ON c.id = m.conversation_id
            WHERE c.component_id = $1
            GROUP BY c.id, c.component_id, c.state, c.created_at, c.updated_at
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

        row.map(row_to_view).transpose()
    }

    async fn get_messages(
        &self,
        conversation_id: &ConversationId,
        options: &MessageListOptions,
    ) -> Result<MessageList, DomainError> {
        let limit = options.effective_limit() as i64;
        let offset = options.effective_offset() as i64;

        // Build query based on filter options
        let (messages_query, count_query) = if options.user_visible_only {
            (
                r#"
                SELECT id, role, content, created_at
                FROM messages
                WHERE conversation_id = $1 AND role IN ('user', 'assistant')
                ORDER BY created_at ASC
                LIMIT $2 OFFSET $3
                "#,
                r#"
                SELECT COUNT(*)::bigint as total
                FROM messages
                WHERE conversation_id = $1 AND role IN ('user', 'assistant')
                "#,
            )
        } else {
            (
                r#"
                SELECT id, role, content, created_at
                FROM messages
                WHERE conversation_id = $1
                ORDER BY created_at ASC
                LIMIT $2 OFFSET $3
                "#,
                r#"
                SELECT COUNT(*)::bigint as total
                FROM messages
                WHERE conversation_id = $1
                "#,
            )
        };

        // Fetch messages
        let rows = sqlx::query(messages_query)
            .bind(conversation_id.as_uuid())
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                DomainError::new(
                    ErrorCode::DatabaseError,
                    format!("Failed to fetch messages: {}", e),
                )
            })?;

        // Fetch total count
        let count_row = sqlx::query(count_query)
            .bind(conversation_id.as_uuid())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                DomainError::new(
                    ErrorCode::DatabaseError,
                    format!("Failed to count messages: {}", e),
                )
            })?;

        let total: i64 = count_row.get("total");

        let items: Result<Vec<MessageView>, DomainError> = rows
            .into_iter()
            .map(|row| {
                let id: uuid::Uuid = row.get("id");
                let role: String = row.get("role");
                let content: String = row.get("content");
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

                Ok(MessageView {
                    id: id.to_string(),
                    role: str_to_role(&role)?,
                    content,
                    created_at: Timestamp::from_datetime(created_at),
                })
            })
            .collect();

        let items = items?;
        let has_more = (offset + items.len() as i64) < total;

        Ok(MessageList {
            items,
            total: total as u64,
            has_more,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper functions
// ─────────────────────────────────────────────────────────────────────────────

fn row_to_view(row: sqlx::postgres::PgRow) -> Result<ConversationView, DomainError> {
    let id: uuid::Uuid = row.get("id");
    let component_id: uuid::Uuid = row.get("component_id");
    let state: String = row.get("state");
    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
    let message_count: i32 = row.get("message_count");

    Ok(ConversationView {
        id: ConversationId::from_uuid(id),
        component_id: ComponentId::from_uuid(component_id),
        state: str_to_state(&state)?,
        message_count: message_count as u32,
        created_at: Timestamp::from_datetime(created_at),
        updated_at: Timestamp::from_datetime(updated_at),
    })
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
