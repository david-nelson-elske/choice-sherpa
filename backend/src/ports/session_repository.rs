//! Session repository port (write side).
//!
//! Defines the contract for persisting and retrieving Session aggregates.
//! Implementations handle the actual database operations.
//!
//! # Design
//!
//! - **Write-focused**: Optimized for aggregate persistence
//! - **Event publishing**: Implementations should publish domain events
//! - **User-scoped**: Most queries are by user_id

use crate::domain::foundation::{DomainError, SessionId, UserId};
use crate::domain::session::Session;
use async_trait::async_trait;

/// Repository port for Session aggregate persistence.
///
/// Handles write operations for session lifecycle management.
/// Implementations must ensure:
/// - Domain event publication on state changes
/// - Proper indexing for user-based queries
#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Save a new session.
    ///
    /// # Errors
    ///
    /// - `DatabaseError` on persistence failure
    async fn save(&self, session: &Session) -> Result<(), DomainError>;

    /// Update an existing session.
    ///
    /// # Errors
    ///
    /// - `SessionNotFound` if session doesn't exist
    /// - `DatabaseError` on persistence failure
    async fn update(&self, session: &Session) -> Result<(), DomainError>;

    /// Find a session by its ID.
    ///
    /// Returns `None` if not found.
    async fn find_by_id(&self, id: &SessionId) -> Result<Option<Session>, DomainError>;

    /// Check if a session exists.
    async fn exists(&self, id: &SessionId) -> Result<bool, DomainError>;

    /// Find all sessions owned by a user.
    ///
    /// Returns sessions ordered by updated_at descending.
    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Vec<Session>, DomainError>;

    /// Count sessions for a user (for access control checks).
    ///
    /// Only counts active (non-archived) sessions by default.
    async fn count_active_by_user(&self, user_id: &UserId) -> Result<u32, DomainError>;

    /// Delete a session (primarily for testing).
    ///
    /// In production, sessions should be archived rather than deleted.
    ///
    /// # Errors
    ///
    /// - `SessionNotFound` if session doesn't exist
    /// - `DatabaseError` on persistence failure
    async fn delete(&self, id: &SessionId) -> Result<(), DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trait object safety test
    #[test]
    fn session_repository_is_object_safe() {
        fn _accepts_dyn(_repo: &dyn SessionRepository) {}
    }
}
