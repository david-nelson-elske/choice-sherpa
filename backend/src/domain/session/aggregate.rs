//! Session aggregate entity.
//!
//! Sessions are the top-level container for decision contexts.
//! Each session belongs to one user and can contain multiple cycles.
//!
//! # Ownership
//!
//! Sessions reference cycles by ID but do NOT own them.
//! Cycles are managed by the Cycle module.

use crate::domain::foundation::{
    CycleId, DomainError, ErrorCode, SessionId, SessionStatus, Timestamp, UserId,
};
use serde::{Deserialize, Serialize};

/// Maximum length for session title.
pub const MAX_TITLE_LENGTH: usize = 500;

/// Session aggregate - top-level container for a decision context.
///
/// # Invariants
///
/// - `id` is globally unique
/// - `title` is 1-500 characters, non-empty
/// - `cycle_ids` contains no duplicates
/// - Archived sessions cannot be modified
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    /// Unique identifier for this session.
    id: SessionId,

    /// User who owns this session.
    user_id: UserId,

    /// Session title.
    title: String,

    /// Optional description.
    description: Option<String>,

    /// Current status (Active or Archived).
    status: SessionStatus,

    /// IDs of cycles in this session (not owned).
    cycle_ids: Vec<CycleId>,

    /// When the session was created.
    created_at: Timestamp,

    /// When the session was last updated.
    updated_at: Timestamp,
}

impl Session {
    /// Create a new active session.
    ///
    /// # Errors
    ///
    /// - `ValidationFailed` if title is empty or too long
    pub fn new(id: SessionId, user_id: UserId, title: String) -> Result<Self, DomainError> {
        Self::validate_title(&title)?;

        let now = Timestamp::now();
        Ok(Self {
            id,
            user_id,
            title,
            description: None,
            status: SessionStatus::Active,
            cycle_ids: Vec::new(),
            created_at: now,
            updated_at: now,
        })
    }

    /// Reconstitute a session from persistence (no validation, no events).
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: SessionId,
        user_id: UserId,
        title: String,
        description: Option<String>,
        status: SessionStatus,
        cycle_ids: Vec<CycleId>,
        created_at: Timestamp,
        updated_at: Timestamp,
    ) -> Self {
        Self {
            id,
            user_id,
            title,
            description,
            status,
            cycle_ids,
            created_at,
            updated_at,
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Accessors
    // ─────────────────────────────────────────────────────────────────────────

    /// Returns the session ID.
    pub fn id(&self) -> &SessionId {
        &self.id
    }

    /// Returns the owner's user ID.
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    /// Returns the session title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the session description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the current status.
    pub fn status(&self) -> SessionStatus {
        self.status
    }

    /// Returns the cycle IDs.
    pub fn cycle_ids(&self) -> &[CycleId] {
        &self.cycle_ids
    }

    /// Returns the number of cycles.
    pub fn cycle_count(&self) -> usize {
        self.cycle_ids.len()
    }

    /// Returns when the session was created.
    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    /// Returns when the session was last updated.
    pub fn updated_at(&self) -> &Timestamp {
        &self.updated_at
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Authorization
    // ─────────────────────────────────────────────────────────────────────────

    /// Checks if the given user owns this session.
    pub fn is_owner(&self, user_id: &UserId) -> bool {
        &self.user_id == user_id
    }

    /// Validates that the user can access this session.
    ///
    /// # Errors
    ///
    /// - `Forbidden` if user is not the owner
    pub fn authorize(&self, user_id: &UserId) -> Result<(), DomainError> {
        if self.is_owner(user_id) {
            Ok(())
        } else {
            Err(DomainError::new(
                ErrorCode::Forbidden,
                "User is not authorized to access this session",
            ))
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Mutations
    // ─────────────────────────────────────────────────────────────────────────

    /// Rename the session.
    ///
    /// # Errors
    ///
    /// - `SessionArchived` if session is archived
    /// - `ValidationFailed` if title is empty or too long
    pub fn rename(&mut self, new_title: String) -> Result<String, DomainError> {
        self.ensure_mutable()?;
        Self::validate_title(&new_title)?;

        let old_title = std::mem::replace(&mut self.title, new_title);
        self.updated_at = Timestamp::now();
        Ok(old_title)
    }

    /// Update the session description.
    ///
    /// # Errors
    ///
    /// - `SessionArchived` if session is archived
    pub fn update_description(
        &mut self,
        description: Option<String>,
    ) -> Result<Option<String>, DomainError> {
        self.ensure_mutable()?;

        let old_description = std::mem::replace(&mut self.description, description);
        self.updated_at = Timestamp::now();
        Ok(old_description)
    }

    /// Add a cycle to this session.
    ///
    /// # Errors
    ///
    /// - `SessionArchived` if session is archived
    /// - `ValidationFailed` if cycle is already in session
    pub fn add_cycle(&mut self, cycle_id: CycleId) -> Result<bool, DomainError> {
        self.ensure_mutable()?;

        if self.cycle_ids.contains(&cycle_id) {
            return Ok(false); // Already exists
        }

        let is_root = self.cycle_ids.is_empty();
        self.cycle_ids.push(cycle_id);
        self.updated_at = Timestamp::now();
        Ok(is_root)
    }

    /// Archive the session (soft delete).
    ///
    /// # Errors
    ///
    /// - `InvalidStateTransition` if already archived
    pub fn archive(&mut self) -> Result<(), DomainError> {
        if !self.status.can_transition_to(&SessionStatus::Archived) {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                "Session is already archived",
            ));
        }

        self.status = SessionStatus::Archived;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Private helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Validates that the session can be modified.
    fn ensure_mutable(&self) -> Result<(), DomainError> {
        if self.status.is_mutable() {
            Ok(())
        } else {
            Err(DomainError::new(
                ErrorCode::SessionArchived,
                "Cannot modify an archived session",
            ))
        }
    }

    /// Validates the session title.
    fn validate_title(title: &str) -> Result<(), DomainError> {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            return Err(DomainError::validation("title", "Title cannot be empty"));
        }
        if trimmed.len() > MAX_TITLE_LENGTH {
            return Err(DomainError::validation(
                "title",
                format!("Title must be {} characters or less", MAX_TITLE_LENGTH),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_user_id() -> UserId {
        UserId::new("user-123".to_string()).unwrap()
    }

    fn test_session() -> Session {
        Session::new(SessionId::new(), test_user_id(), "Test Session".to_string()).unwrap()
    }

    // Construction tests

    #[test]
    fn new_session_is_active() {
        let session = test_session();
        assert_eq!(session.status(), SessionStatus::Active);
    }

    #[test]
    fn new_session_has_no_cycles() {
        let session = test_session();
        assert!(session.cycle_ids().is_empty());
        assert_eq!(session.cycle_count(), 0);
    }

    #[test]
    fn new_session_rejects_empty_title() {
        let result = Session::new(SessionId::new(), test_user_id(), "".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn new_session_rejects_whitespace_title() {
        let result = Session::new(SessionId::new(), test_user_id(), "   ".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn new_session_rejects_too_long_title() {
        let long_title = "x".repeat(MAX_TITLE_LENGTH + 1);
        let result = Session::new(SessionId::new(), test_user_id(), long_title);
        assert!(result.is_err());
    }

    // Rename tests

    #[test]
    fn rename_returns_old_title() {
        let mut session = test_session();
        let old = session.rename("New Title".to_string()).unwrap();
        assert_eq!(old, "Test Session");
        assert_eq!(session.title(), "New Title");
    }

    #[test]
    fn rename_fails_when_archived() {
        let mut session = test_session();
        session.archive().unwrap();
        let result = session.rename("New Title".to_string());
        assert!(result.is_err());
    }

    // Description tests

    #[test]
    fn update_description_returns_old() {
        let mut session = test_session();
        let old = session
            .update_description(Some("New description".to_string()))
            .unwrap();
        assert!(old.is_none());
        assert_eq!(session.description(), Some("New description"));
    }

    // Cycle management tests

    #[test]
    fn add_cycle_first_is_root() {
        let mut session = test_session();
        let is_root = session.add_cycle(CycleId::new()).unwrap();
        assert!(is_root);
    }

    #[test]
    fn add_cycle_second_is_not_root() {
        let mut session = test_session();
        session.add_cycle(CycleId::new()).unwrap();
        let is_root = session.add_cycle(CycleId::new()).unwrap();
        assert!(!is_root);
    }

    #[test]
    fn add_cycle_duplicate_returns_false() {
        let mut session = test_session();
        let cycle_id = CycleId::new();
        session.add_cycle(cycle_id).unwrap();
        let result = session.add_cycle(cycle_id).unwrap();
        assert!(!result); // Not root, already existed
    }

    // Archive tests

    #[test]
    fn archive_changes_status() {
        let mut session = test_session();
        session.archive().unwrap();
        assert_eq!(session.status(), SessionStatus::Archived);
    }

    #[test]
    fn archive_twice_fails() {
        let mut session = test_session();
        session.archive().unwrap();
        let result = session.archive();
        assert!(result.is_err());
    }

    // Authorization tests

    #[test]
    fn owner_is_authorized() {
        let session = test_session();
        assert!(session.authorize(&test_user_id()).is_ok());
    }

    #[test]
    fn non_owner_is_forbidden() {
        let session = test_session();
        let other_user = UserId::new("other-user".to_string()).unwrap();
        let result = session.authorize(&other_user);
        assert!(result.is_err());
    }
}
