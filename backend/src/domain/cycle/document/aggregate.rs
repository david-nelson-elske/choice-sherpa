//! DecisionDocument aggregate - The live markdown artifact for a decision cycle.
//!
//! This entity represents the primary working document for a decision cycle.
//! Both users and AI agents operate on this document, and changes are synchronized
//! with the structured component data.

use crate::domain::foundation::{
    ComponentType, CycleId, DecisionDocumentId, DomainError, ErrorCode, Timestamp, UserId,
};

use super::{DocumentVersion, MarkdownContent, SyncSource, UpdatedBy};

/// The DecisionDocument aggregate root.
///
/// A DecisionDocument is the live markdown artifact for a decision cycle.
/// It serves as the human-readable, editable interface to the structured
/// PrOACT component data.
#[derive(Debug, Clone)]
pub struct DecisionDocument {
    // Identity
    id: DecisionDocumentId,
    cycle_id: CycleId,
    user_id: UserId,

    // Content
    content: MarkdownContent,
    file_path: String,

    // Versioning
    version: DocumentVersion,
    last_sync_source: SyncSource,
    last_synced_at: Timestamp,

    // Branch metadata
    parent_document_id: Option<DecisionDocumentId>,
    branch_point: Option<ComponentType>,
    branch_label: Option<String>,

    // Timestamps
    created_at: Timestamp,
    updated_at: Timestamp,
    updated_by: UpdatedBy,
}

impl DecisionDocument {
    // ════════════════════════════════════════════════════════════════════════════════
    // Construction
    // ════════════════════════════════════════════════════════════════════════════════

    /// Creates a new decision document for a cycle.
    pub fn new(cycle_id: CycleId, user_id: UserId, initial_content: impl Into<String>) -> Self {
        let id = DecisionDocumentId::new();
        let now = Timestamp::now();
        let content = MarkdownContent::new(initial_content);

        Self {
            id,
            cycle_id,
            user_id: user_id.clone(),
            content,
            file_path: Self::compute_file_path(&user_id, &id),
            version: DocumentVersion::initial(),
            last_sync_source: SyncSource::Initial,
            last_synced_at: now,
            parent_document_id: None,
            branch_point: None,
            branch_label: None,
            created_at: now,
            updated_at: now,
            updated_by: UpdatedBy::System,
        }
    }

    /// Creates a branched document from a parent.
    pub fn new_branch(
        cycle_id: CycleId,
        user_id: UserId,
        parent_document_id: DecisionDocumentId,
        branch_point: ComponentType,
        branch_label: impl Into<String>,
        initial_content: impl Into<String>,
    ) -> Self {
        let id = DecisionDocumentId::new();
        let now = Timestamp::now();
        let content = MarkdownContent::new(initial_content);

        Self {
            id,
            cycle_id,
            user_id: user_id.clone(),
            content,
            file_path: Self::compute_file_path(&user_id, &id),
            version: DocumentVersion::initial(),
            last_sync_source: SyncSource::Initial,
            last_synced_at: now,
            parent_document_id: Some(parent_document_id),
            branch_point: Some(branch_point),
            branch_label: Some(branch_label.into()),
            created_at: now,
            updated_at: now,
            updated_by: UpdatedBy::System,
        }
    }

    /// Reconstitutes a document from persistence.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: DecisionDocumentId,
        cycle_id: CycleId,
        user_id: UserId,
        content: MarkdownContent,
        file_path: String,
        version: DocumentVersion,
        last_sync_source: SyncSource,
        last_synced_at: Timestamp,
        parent_document_id: Option<DecisionDocumentId>,
        branch_point: Option<ComponentType>,
        branch_label: Option<String>,
        created_at: Timestamp,
        updated_at: Timestamp,
        updated_by: UpdatedBy,
    ) -> Self {
        Self {
            id,
            cycle_id,
            user_id,
            content,
            file_path,
            version,
            last_sync_source,
            last_synced_at,
            parent_document_id,
            branch_point,
            branch_label,
            created_at,
            updated_at,
            updated_by,
        }
    }

    /// Computes the file path for a document.
    fn compute_file_path(user_id: &UserId, id: &DecisionDocumentId) -> String {
        format!("{}/doc_{}.md", user_id, id)
    }

    // ════════════════════════════════════════════════════════════════════════════════
    // Accessors
    // ════════════════════════════════════════════════════════════════════════════════

    /// Returns the document ID.
    pub fn id(&self) -> DecisionDocumentId {
        self.id
    }

    /// Returns the cycle ID this document belongs to.
    pub fn cycle_id(&self) -> CycleId {
        self.cycle_id
    }

    /// Returns the user ID who owns this document.
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    /// Returns the markdown content.
    pub fn content(&self) -> &MarkdownContent {
        &self.content
    }

    /// Returns the raw markdown string.
    pub fn raw_content(&self) -> &str {
        self.content.raw()
    }

    /// Returns the content checksum.
    pub fn content_checksum(&self) -> &str {
        self.content.checksum()
    }

    /// Returns the file path (relative).
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Returns the current version.
    pub fn version(&self) -> DocumentVersion {
        self.version
    }

    /// Returns what triggered the last sync.
    pub fn last_sync_source(&self) -> SyncSource {
        self.last_sync_source
    }

    /// Returns when the last sync occurred.
    pub fn last_synced_at(&self) -> Timestamp {
        self.last_synced_at
    }

    /// Returns the parent document ID if this is a branch.
    pub fn parent_document_id(&self) -> Option<DecisionDocumentId> {
        self.parent_document_id
    }

    /// Returns the component where branching occurred.
    pub fn branch_point(&self) -> Option<ComponentType> {
        self.branch_point
    }

    /// Returns the branch label.
    pub fn branch_label(&self) -> Option<&str> {
        self.branch_label.as_deref()
    }

    /// Returns when this document was created.
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    /// Returns when this document was last updated.
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    /// Returns who last updated this document.
    pub fn updated_by(&self) -> &UpdatedBy {
        &self.updated_by
    }

    /// Returns true if this is a branched document.
    pub fn is_branch(&self) -> bool {
        self.parent_document_id.is_some()
    }

    /// Returns the content size in bytes.
    pub fn content_size_bytes(&self) -> usize {
        self.content.size_bytes()
    }

    // ════════════════════════════════════════════════════════════════════════════════
    // Mutations
    // ════════════════════════════════════════════════════════════════════════════════

    /// Updates the document from component output (system/agent generated).
    ///
    /// This is called when component data changes and the document needs to reflect that.
    pub fn update_from_components(&mut self, new_content: impl Into<String>) {
        let now = Timestamp::now();

        self.content.update(new_content);
        self.version = self.version.increment();
        self.last_sync_source = SyncSource::ComponentUpdate;
        self.last_synced_at = now;
        self.updated_at = now;
        self.updated_by = UpdatedBy::System;
    }

    /// Updates the document from agent-generated content.
    pub fn update_from_agent(&mut self, new_content: impl Into<String>) {
        let now = Timestamp::now();

        self.content.update(new_content);
        self.version = self.version.increment();
        self.last_sync_source = SyncSource::ComponentUpdate;
        self.last_synced_at = now;
        self.updated_at = now;
        self.updated_by = UpdatedBy::Agent;
    }

    /// Updates the document from a user edit (no version check).
    ///
    /// Use this when the handler has already loaded the document and
    /// version conflicts aren't a concern (e.g., single-user editing).
    pub fn update_from_user_edit(&mut self, new_content: impl Into<String>) {
        let now = Timestamp::now();

        self.content.update(new_content);
        self.version = self.version.increment();
        self.last_sync_source = SyncSource::UserEdit;
        self.last_synced_at = now;
        self.updated_at = now;
        self.updated_by = UpdatedBy::System; // System because we don't have user context here
    }

    /// Applies a user edit to the document.
    ///
    /// Validates version for optimistic locking.
    pub fn apply_user_edit(
        &mut self,
        new_content: impl Into<String>,
        user_id: UserId,
        expected_version: DocumentVersion,
    ) -> Result<(), DomainError> {
        // Optimistic locking check
        if self.version != expected_version {
            return Err(DomainError::new(
                ErrorCode::ConcurrencyConflict,
                format!(
                    "Document version mismatch: expected {}, found {}",
                    expected_version, self.version
                ),
            ));
        }

        let now = Timestamp::now();

        self.content.update(new_content);
        self.version = self.version.increment();
        self.last_sync_source = SyncSource::UserEdit;
        self.last_synced_at = now;
        self.updated_at = now;
        self.updated_by = UpdatedBy::User { user_id };

        Ok(())
    }

    /// Syncs the document from filesystem changes.
    ///
    /// Used when external tools modify the file directly.
    pub fn sync_from_file(&mut self, new_content: impl Into<String>, new_checksum: String) {
        let now = Timestamp::now();

        // Only update if content actually changed
        let new_content = new_content.into();
        if self.content.checksum() != new_checksum {
            self.content = MarkdownContent::new(new_content);
            self.version = self.version.increment();
            self.last_sync_source = SyncSource::FileSync;
            self.last_synced_at = now;
            self.updated_at = now;
            // Keep updated_by as is - we don't know who made the external edit
        }
    }

    /// Checks if the content has changed compared to another string.
    pub fn has_content_changed(&self, other: &str) -> bool {
        self.content.has_changed(other)
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    fn test_cycle_id() -> CycleId {
        CycleId::new()
    }

    // ───────────────────────────────────────────────────────────────
    // Creation Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn new_document_has_correct_initial_state() {
        let cycle_id = test_cycle_id();
        let user_id = test_user_id();

        let doc = DecisionDocument::new(cycle_id, user_id.clone(), "# My Decision");

        assert_eq!(doc.cycle_id(), cycle_id);
        assert_eq!(doc.user_id(), &user_id);
        assert_eq!(doc.raw_content(), "# My Decision");
        assert_eq!(doc.version(), DocumentVersion::initial());
        assert_eq!(doc.last_sync_source(), SyncSource::Initial);
        assert!(!doc.is_branch());
    }

    #[test]
    fn new_document_computes_file_path() {
        let cycle_id = test_cycle_id();
        let user_id = test_user_id();

        let doc = DecisionDocument::new(cycle_id, user_id.clone(), "# Content");

        assert!(doc.file_path().starts_with("test-user-123/doc_"));
        assert!(doc.file_path().ends_with(".md"));
    }

    #[test]
    fn new_branch_has_parent_reference() {
        let cycle_id = test_cycle_id();
        let user_id = test_user_id();
        let parent_id = DecisionDocumentId::new();

        let doc = DecisionDocument::new_branch(
            cycle_id,
            user_id,
            parent_id,
            ComponentType::Alternatives,
            "Remote Option",
            "# Branched Decision",
        );

        assert!(doc.is_branch());
        assert_eq!(doc.parent_document_id(), Some(parent_id));
        assert_eq!(doc.branch_point(), Some(ComponentType::Alternatives));
        assert_eq!(doc.branch_label(), Some("Remote Option"));
    }

    // ───────────────────────────────────────────────────────────────
    // Update Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn update_from_components_increments_version() {
        let mut doc = DecisionDocument::new(test_cycle_id(), test_user_id(), "# Original");

        doc.update_from_components("# Updated");

        assert_eq!(doc.version(), DocumentVersion::from_raw(2));
        assert_eq!(doc.raw_content(), "# Updated");
        assert_eq!(doc.last_sync_source(), SyncSource::ComponentUpdate);
        assert!(matches!(doc.updated_by(), UpdatedBy::System));
    }

    #[test]
    fn update_from_agent_sets_agent_updated_by() {
        let mut doc = DecisionDocument::new(test_cycle_id(), test_user_id(), "# Original");

        doc.update_from_agent("# Agent Updated");

        assert!(matches!(doc.updated_by(), UpdatedBy::Agent));
    }

    #[test]
    fn apply_user_edit_with_correct_version_succeeds() {
        let user_id = test_user_id();
        let mut doc = DecisionDocument::new(test_cycle_id(), user_id.clone(), "# Original");
        let expected_version = doc.version();

        let result = doc.apply_user_edit("# User Edited", user_id.clone(), expected_version);

        assert!(result.is_ok());
        assert_eq!(doc.raw_content(), "# User Edited");
        assert_eq!(doc.last_sync_source(), SyncSource::UserEdit);
        assert!(matches!(doc.updated_by(), UpdatedBy::User { .. }));
    }

    #[test]
    fn apply_user_edit_with_wrong_version_fails() {
        let user_id = test_user_id();
        let mut doc = DecisionDocument::new(test_cycle_id(), user_id.clone(), "# Original");
        let wrong_version = DocumentVersion::from_raw(999);

        let result = doc.apply_user_edit("# User Edited", user_id, wrong_version);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::ConcurrencyConflict);
    }

    #[test]
    fn sync_from_file_only_updates_when_checksum_differs() {
        let mut doc = DecisionDocument::new(test_cycle_id(), test_user_id(), "# Original");
        let original_checksum = doc.content_checksum().to_string();
        let original_version = doc.version();

        // Same content, same checksum - no update
        doc.sync_from_file("# Original", original_checksum.clone());
        assert_eq!(doc.version(), original_version);

        // Different checksum - update
        doc.sync_from_file("# Updated", "different_checksum".to_string());
        assert_eq!(doc.version(), DocumentVersion::from_raw(2));
        assert_eq!(doc.last_sync_source(), SyncSource::FileSync);
    }

    // ───────────────────────────────────────────────────────────────
    // Content Change Detection Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn has_content_changed_detects_differences() {
        let doc = DecisionDocument::new(test_cycle_id(), test_user_id(), "# Original");

        assert!(!doc.has_content_changed("# Original"));
        assert!(doc.has_content_changed("# Different"));
    }
}
