//! PostgreSQL implementation of SessionRepository.
//!
//! Persists Session aggregates to PostgreSQL.

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::foundation::{
    CycleId, DomainError, ErrorCode, SessionId, SessionStatus, Timestamp, UserId,
};
use crate::domain::session::Session;
use crate::ports::SessionRepository;

/// PostgreSQL implementation of SessionRepository.
#[derive(Clone)]
pub struct PostgresSessionRepository {
    pool: PgPool,
}

impl PostgresSessionRepository {
    /// Creates a new PostgresSessionRepository.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for PostgresSessionRepository {
    async fn save(&self, session: &Session) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, user_id, title, description, status, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(session.id().as_uuid())
        .bind(session.user_id().as_str())
        .bind(session.title())
        .bind(session.description())
        .bind(session_status_to_str(session.status()))
        .bind(session.created_at().as_datetime())
        .bind(session.updated_at().as_datetime())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Failed to insert session: {}", e),
            )
        })?;

        Ok(())
    }

    async fn update(&self, session: &Session) -> Result<(), DomainError> {
        let result = sqlx::query(
            r#"
            UPDATE sessions SET
                title = $2,
                description = $3,
                status = $4,
                updated_at = $5
            WHERE id = $1
            "#,
        )
        .bind(session.id().as_uuid())
        .bind(session.title())
        .bind(session.description())
        .bind(session_status_to_str(session.status()))
        .bind(session.updated_at().as_datetime())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Failed to update session: {}", e),
            )
        })?;

        if result.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::SessionNotFound,
                format!("Session not found: {}", session.id()),
            ));
        }

        Ok(())
    }

    async fn find_by_id(&self, id: &SessionId) -> Result<Option<Session>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT s.id, s.user_id, s.title, s.description, s.status,
                   s.created_at, s.updated_at,
                   COALESCE(array_agg(c.id) FILTER (WHERE c.id IS NOT NULL), '{}') as cycle_ids
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
                let session = row_to_session(row)?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    async fn exists(&self, id: &SessionId) -> Result<bool, DomainError> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE id = $1")
            .bind(id.as_uuid())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                DomainError::new(
                    ErrorCode::DatabaseError,
                    format!("Failed to check session existence: {}", e),
                )
            })?;

        Ok(result.0 > 0)
    }

    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Vec<Session>, DomainError> {
        let rows = sqlx::query(
            r#"
            SELECT s.id, s.user_id, s.title, s.description, s.status,
                   s.created_at, s.updated_at,
                   COALESCE(array_agg(c.id) FILTER (WHERE c.id IS NOT NULL), '{}') as cycle_ids
            FROM sessions s
            LEFT JOIN cycles c ON c.session_id = s.id
            WHERE s.user_id = $1
            GROUP BY s.id, s.user_id, s.title, s.description, s.status, s.created_at, s.updated_at
            ORDER BY s.updated_at DESC
            "#,
        )
        .bind(user_id.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Failed to fetch sessions by user: {}", e),
            )
        })?;

        let sessions: Result<Vec<Session>, DomainError> =
            rows.into_iter().map(row_to_session).collect();

        sessions
    }

    async fn count_active_by_user(&self, user_id: &UserId) -> Result<u32, DomainError> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sessions WHERE user_id = $1 AND status = 'active'",
        )
        .bind(user_id.as_str())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Failed to count active sessions: {}", e),
            )
        })?;

        Ok(result.0 as u32)
    }

    async fn delete(&self, id: &SessionId) -> Result<(), DomainError> {
        let result = sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| {
                DomainError::new(
                    ErrorCode::DatabaseError,
                    format!("Failed to delete session: {}", e),
                )
            })?;

        if result.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::SessionNotFound,
                format!("Session not found: {}", id),
            ));
        }

        Ok(())
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

fn row_to_session(row: sqlx::postgres::PgRow) -> Result<Session, DomainError> {
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

    let cycle_uuids: Vec<uuid::Uuid> = row.try_get("cycle_ids").map_err(|e| {
        DomainError::new(
            ErrorCode::DatabaseError,
            format!("Failed to get cycle_ids: {}", e),
        )
    })?;

    let cycle_ids: Vec<CycleId> = cycle_uuids.into_iter().map(CycleId::from_uuid).collect();

    Ok(Session::reconstitute(
        SessionId::from_uuid(id),
        UserId::new(user_id).map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Invalid user_id: {}", e),
            )
        })?,
        title,
        description,
        status,
        cycle_ids,
        Timestamp::from_datetime(created_at),
        Timestamp::from_datetime(updated_at),
    ))
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
