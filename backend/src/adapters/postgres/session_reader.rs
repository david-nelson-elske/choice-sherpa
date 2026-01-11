//! PostgreSQL implementation of SessionReader.
//!
//! Provides read-optimized queries for session data.

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::foundation::{
    DomainError, ErrorCode, SessionId, SessionStatus, Timestamp, UserId,
};
use crate::ports::{ListOptions, SessionList, SessionReader, SessionSummary, SessionView};

/// PostgreSQL implementation of SessionReader.
#[derive(Clone)]
pub struct PostgresSessionReader {
    pool: PgPool,
}

impl PostgresSessionReader {
    /// Creates a new PostgresSessionReader.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionReader for PostgresSessionReader {
    async fn get_by_id(&self, id: &SessionId) -> Result<Option<SessionView>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT s.id, s.user_id, s.title, s.description, s.status,
                   s.created_at, s.updated_at,
                   COUNT(c.id) as cycle_count
            FROM sessions s
            LEFT JOIN cycles c ON c.session_id = s.id
            WHERE s.id = $1
            GROUP BY s.id, s.user_id, s.title, s.description, s.status, s.created_at, s.updated_at
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Failed to fetch session: {}", e),
            )
        })?;

        match row {
            Some(row) => {
                let view = row_to_session_view(row)?;
                Ok(Some(view))
            }
            None => Ok(None),
        }
    }

    async fn list_by_user(
        &self,
        user_id: &UserId,
        options: &ListOptions,
    ) -> Result<SessionList, DomainError> {
        // Build the base query
        let mut query = String::from(
            r#"
            SELECT s.id, s.title, s.status, s.updated_at,
                   COUNT(c.id) as cycle_count
            FROM sessions s
            LEFT JOIN cycles c ON c.session_id = s.id
            WHERE s.user_id = $1
            "#,
        );

        // Add status filter if specified
        if let Some(status) = options.status {
            query.push_str(&format!(" AND s.status = '{}'", session_status_to_str(status)));
        } else if !options.include_archived {
            query.push_str(" AND s.status = 'active'");
        }

        // Group by and order
        query.push_str(
            " GROUP BY s.id, s.title, s.status, s.updated_at ORDER BY s.updated_at DESC",
        );

        // Add limit and offset
        if let Some(limit) = options.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = options.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        // Execute the query
        let rows = sqlx::query(&query)
            .bind(user_id.as_str())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                DomainError::new(
                    ErrorCode::DatabaseError,
                    format!("Failed to list sessions: {}", e),
                )
            })?;

        let items: Result<Vec<SessionSummary>, DomainError> =
            rows.into_iter().map(row_to_session_summary).collect();
        let items = items?;

        // Get total count
        let total = self.count_by_user_with_options(user_id, options).await?;

        // Calculate has_more
        let offset = options.offset.unwrap_or(0) as u64;
        let has_more = offset + (items.len() as u64) < total;

        Ok(SessionList {
            items,
            total,
            has_more,
        })
    }

    async fn search(
        &self,
        user_id: &UserId,
        query: &str,
        options: &ListOptions,
    ) -> Result<SessionList, DomainError> {
        // Build search query with full-text search
        let mut sql = String::from(
            r#"
            SELECT s.id, s.title, s.status, s.updated_at,
                   COUNT(c.id) as cycle_count
            FROM sessions s
            LEFT JOIN cycles c ON c.session_id = s.id
            WHERE s.user_id = $1
              AND to_tsvector('english', COALESCE(s.title, '') || ' ' || COALESCE(s.description, ''))
                  @@ plainto_tsquery('english', $2)
            "#,
        );

        // Add status filter
        if let Some(status) = options.status {
            sql.push_str(&format!(" AND s.status = '{}'", session_status_to_str(status)));
        } else if !options.include_archived {
            sql.push_str(" AND s.status = 'active'");
        }

        // Group by and order
        sql.push_str(
            " GROUP BY s.id, s.title, s.status, s.updated_at ORDER BY s.updated_at DESC",
        );

        // Add limit and offset
        if let Some(limit) = options.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = options.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        // Execute the query
        let rows = sqlx::query(&sql)
            .bind(user_id.as_str())
            .bind(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                DomainError::new(
                    ErrorCode::DatabaseError,
                    format!("Failed to search sessions: {}", e),
                )
            })?;

        let items: Result<Vec<SessionSummary>, DomainError> =
            rows.into_iter().map(row_to_session_summary).collect();
        let items = items?;

        // Get total count (simplified - just use items length for search)
        let total = items.len() as u64;
        let has_more = false; // Simplified for search

        Ok(SessionList {
            items,
            total,
            has_more,
        })
    }

    async fn count_by_status(
        &self,
        user_id: &UserId,
        status: SessionStatus,
    ) -> Result<u64, DomainError> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sessions WHERE user_id = $1 AND status = $2",
        )
        .bind(user_id.as_str())
        .bind(session_status_to_str(status))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Failed to count sessions by status: {}", e),
            )
        })?;

        Ok(result.0 as u64)
    }
}

impl PostgresSessionReader {
    /// Helper to count sessions with options applied.
    async fn count_by_user_with_options(
        &self,
        user_id: &UserId,
        options: &ListOptions,
    ) -> Result<u64, DomainError> {
        let mut query = String::from("SELECT COUNT(*) FROM sessions WHERE user_id = $1");

        if let Some(status) = options.status {
            query.push_str(&format!(" AND status = '{}'", session_status_to_str(status)));
        } else if !options.include_archived {
            query.push_str(" AND status = 'active'");
        }

        let result: (i64,) = sqlx::query_as(&query)
            .bind(user_id.as_str())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                DomainError::new(
                    ErrorCode::DatabaseError,
                    format!("Failed to count sessions: {}", e),
                )
            })?;

        Ok(result.0 as u64)
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Helper functions
// ════════════════════════════════════════════════════════════════════════════

fn session_status_to_str(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Active => "active",
        SessionStatus::Archived => "archived",
    }
}

fn str_to_session_status(s: &str) -> Result<SessionStatus, DomainError> {
    match s {
        "active" => Ok(SessionStatus::Active),
        "archived" => Ok(SessionStatus::Archived),
        _ => Err(DomainError::new(
            ErrorCode::DatabaseError,
            format!("Invalid session status: {}", s),
        )),
    }
}

fn row_to_session_view(row: sqlx::postgres::PgRow) -> Result<SessionView, DomainError> {
    let id: uuid::Uuid = row.try_get("id").map_err(|e| {
        DomainError::new(
            ErrorCode::DatabaseError,
            format!("Failed to get id: {}", e),
        )
    })?;

    let user_id: String = row.try_get("user_id").map_err(|e| {
        DomainError::new(
            ErrorCode::DatabaseError,
            format!("Failed to get user_id: {}", e),
        )
    })?;

    let title: String = row.try_get("title").map_err(|e| {
        DomainError::new(
            ErrorCode::DatabaseError,
            format!("Failed to get title: {}", e),
        )
    })?;

    let description: Option<String> = row.try_get("description").map_err(|e| {
        DomainError::new(
            ErrorCode::DatabaseError,
            format!("Failed to get description: {}", e),
        )
    })?;

    let status_str: String = row.try_get("status").map_err(|e| {
        DomainError::new(
            ErrorCode::DatabaseError,
            format!("Failed to get status: {}", e),
        )
    })?;
    let status = str_to_session_status(&status_str)?;

    let cycle_count: i64 = row.try_get("cycle_count").map_err(|e| {
        DomainError::new(
            ErrorCode::DatabaseError,
            format!("Failed to get cycle_count: {}", e),
        )
    })?;

    let created_at: chrono::DateTime<chrono::Utc> = row.try_get("created_at").map_err(|e| {
        DomainError::new(
            ErrorCode::DatabaseError,
            format!("Failed to get created_at: {}", e),
        )
    })?;

    let updated_at: chrono::DateTime<chrono::Utc> = row.try_get("updated_at").map_err(|e| {
        DomainError::new(
            ErrorCode::DatabaseError,
            format!("Failed to get updated_at: {}", e),
        )
    })?;

    Ok(SessionView {
        id: SessionId::from_uuid(id),
        user_id: UserId::new(user_id).map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Invalid user_id: {}", e),
            )
        })?,
        title,
        description,
        status,
        cycle_count: cycle_count as u32,
        created_at: Timestamp::from_datetime(created_at),
        updated_at: Timestamp::from_datetime(updated_at),
    })
}

fn row_to_session_summary(row: sqlx::postgres::PgRow) -> Result<SessionSummary, DomainError> {
    let id: uuid::Uuid = row.try_get("id").map_err(|e| {
        DomainError::new(
            ErrorCode::DatabaseError,
            format!("Failed to get id: {}", e),
        )
    })?;

    let title: String = row.try_get("title").map_err(|e| {
        DomainError::new(
            ErrorCode::DatabaseError,
            format!("Failed to get title: {}", e),
        )
    })?;

    let status_str: String = row.try_get("status").map_err(|e| {
        DomainError::new(
            ErrorCode::DatabaseError,
            format!("Failed to get status: {}", e),
        )
    })?;
    let status = str_to_session_status(&status_str)?;

    let cycle_count: i64 = row.try_get("cycle_count").map_err(|e| {
        DomainError::new(
            ErrorCode::DatabaseError,
            format!("Failed to get cycle_count: {}", e),
        )
    })?;

    let updated_at: chrono::DateTime<chrono::Utc> = row.try_get("updated_at").map_err(|e| {
        DomainError::new(
            ErrorCode::DatabaseError,
            format!("Failed to get updated_at: {}", e),
        )
    })?;

    Ok(SessionSummary {
        id: SessionId::from_uuid(id),
        title,
        status,
        cycle_count: cycle_count as u32,
        updated_at: Timestamp::from_datetime(updated_at),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_status_conversion_roundtrips() {
        let active = SessionStatus::Active;
        assert_eq!(
            str_to_session_status(session_status_to_str(active)).unwrap(),
            active
        );

        let archived = SessionStatus::Archived;
        assert_eq!(
            str_to_session_status(session_status_to_str(archived)).unwrap(),
            archived
        );
    }

    #[test]
    fn str_to_session_status_rejects_invalid() {
        assert!(str_to_session_status("invalid").is_err());
    }
}
