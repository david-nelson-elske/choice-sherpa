//! Session reader port (read side / CQRS queries).
//!
//! Defines the contract for session queries and read operations.
//! Optimized for UI display, search, and listing.
//!
//! # Design
//!
//! - **Read-optimized**: Can use caching, denormalized views
//! - **Separated from write**: CQRS pattern for scalability
//! - **Search support**: Full-text search on title and description

use crate::domain::foundation::{DomainError, SessionId, SessionStatus, Timestamp, UserId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Reader port for session queries.
///
/// Provides read-optimized views of session data.
/// Implementations may use caching for frequently-accessed data.
#[async_trait]
pub trait SessionReader: Send + Sync {
    /// Get detailed session view by ID.
    ///
    /// Returns `None` if not found.
    async fn get_by_id(&self, id: &SessionId) -> Result<Option<SessionView>, DomainError>;

    /// List sessions for a user with pagination.
    ///
    /// Returns sessions ordered by updated_at descending.
    async fn list_by_user(
        &self,
        user_id: &UserId,
        options: &ListOptions,
    ) -> Result<SessionList, DomainError>;

    /// Search sessions by title/description.
    ///
    /// Performs full-text search across title and description fields.
    async fn search(
        &self,
        user_id: &UserId,
        query: &str,
        options: &ListOptions,
    ) -> Result<SessionList, DomainError>;

    /// Count sessions for a user by status.
    async fn count_by_status(
        &self,
        user_id: &UserId,
        status: SessionStatus,
    ) -> Result<u64, DomainError>;
}

/// Options for listing sessions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListOptions {
    /// Maximum number of results to return.
    pub limit: Option<u32>,

    /// Number of results to skip.
    pub offset: Option<u32>,

    /// Filter by status (None = all statuses).
    pub status: Option<SessionStatus>,

    /// Include archived sessions.
    pub include_archived: bool,
}

impl ListOptions {
    /// Create options for a paginated query.
    pub fn paginated(page: u32, per_page: u32) -> Self {
        Self {
            limit: Some(per_page),
            offset: Some((page.saturating_sub(1)) * per_page),
            status: None,
            include_archived: false,
        }
    }

    /// Include archived sessions in results.
    pub fn with_archived(mut self) -> Self {
        self.include_archived = true;
        self
    }

    /// Filter to a specific status.
    pub fn with_status(mut self, status: SessionStatus) -> Self {
        self.status = Some(status);
        self
    }
}

/// Paginated list of sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionList {
    /// Sessions in this page.
    pub items: Vec<SessionSummary>,

    /// Total number of matching sessions.
    pub total: u64,

    /// Whether there are more results.
    pub has_more: bool,
}

/// Detailed view of a session for UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionView {
    /// Session ID.
    pub id: SessionId,

    /// Owner's user ID.
    pub user_id: UserId,

    /// Session title.
    pub title: String,

    /// Optional description.
    pub description: Option<String>,

    /// Current status.
    pub status: SessionStatus,

    /// Number of cycles in this session.
    pub cycle_count: u32,

    /// When the session was created.
    pub created_at: Timestamp,

    /// When the session was last updated.
    pub updated_at: Timestamp,
}

/// Summary view of a session for lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// Session ID.
    pub id: SessionId,

    /// Session title.
    pub title: String,

    /// Current status.
    pub status: SessionStatus,

    /// Number of cycles.
    pub cycle_count: u32,

    /// When the session was last updated.
    pub updated_at: Timestamp,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trait object safety test
    #[test]
    fn session_reader_is_object_safe() {
        fn _accepts_dyn(_reader: &dyn SessionReader) {}
    }

    #[test]
    fn list_options_pagination_calculates_offset() {
        let options = ListOptions::paginated(1, 10);
        assert_eq!(options.offset, Some(0));
        assert_eq!(options.limit, Some(10));

        let options = ListOptions::paginated(3, 25);
        assert_eq!(options.offset, Some(50));
        assert_eq!(options.limit, Some(25));
    }

    #[test]
    fn list_options_default_excludes_archived() {
        let options = ListOptions::default();
        assert!(!options.include_archived);
    }

    #[test]
    fn list_options_can_include_archived() {
        let options = ListOptions::default().with_archived();
        assert!(options.include_archived);
    }
}
