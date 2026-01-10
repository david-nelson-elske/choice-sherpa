//! Decision Document Repository Port - Coordinated DB + filesystem operations.
//!
//! This port defines the contract for persisting decision documents,
//! coordinating between database metadata storage and filesystem content storage.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::cycle::DecisionDocument;
use crate::domain::foundation::{CycleId, DecisionDocumentId, DomainError};

/// Port for decision document persistence operations.
///
/// # Contract
///
/// Implementations must:
/// - Coordinate database and filesystem operations atomically
/// - Store metadata in database, content in filesystem
/// - Maintain consistency between storage layers
/// - Support synchronization from external file edits
///
/// # Dual Storage
///
/// Documents are stored in two places:
/// - **Database**: Metadata, version, checksums, progress tracking
/// - **Filesystem**: Full markdown content (for agent access)
///
/// The repository ensures these stay synchronized.
///
/// # Usage
///
/// ```rust,ignore
/// let repo: &dyn DecisionDocumentRepository = get_repo();
///
/// // Save new document (creates both DB record and file)
/// repo.save(&document, "# My Decision...").await?;
///
/// // Update existing (updates both)
/// repo.update(&document, "# Updated content...").await?;
///
/// // Find by cycle
/// if let Some(doc) = repo.find_by_cycle(cycle_id).await? {
///     // Document loaded from DB, content from file
/// }
/// ```
#[async_trait]
pub trait DecisionDocumentRepository: Send + Sync {
    /// Save a new document (creates file + DB record).
    ///
    /// # Arguments
    ///
    /// * `document` - The document entity to save
    /// * `content` - The markdown content to write to filesystem
    ///
    /// # Errors
    ///
    /// Returns `DomainError` if:
    /// - A document already exists for this cycle
    /// - Database or filesystem write fails
    async fn save(&self, document: &DecisionDocument, content: &str) -> Result<(), DomainError>;

    /// Update an existing document (updates file + DB record).
    ///
    /// # Arguments
    ///
    /// * `document` - The document entity with updated state
    /// * `content` - The new markdown content
    ///
    /// # Errors
    ///
    /// Returns `DomainError` if:
    /// - Document doesn't exist
    /// - Version conflict (optimistic locking)
    /// - Database or filesystem write fails
    async fn update(&self, document: &DecisionDocument, content: &str) -> Result<(), DomainError>;

    /// Find document by ID.
    ///
    /// Loads metadata from database, content can be loaded separately.
    ///
    /// # Arguments
    ///
    /// * `id` - The document's unique identifier
    ///
    /// # Returns
    ///
    /// The document if found, None otherwise.
    async fn find_by_id(
        &self,
        id: DecisionDocumentId,
    ) -> Result<Option<DecisionDocument>, DomainError>;

    /// Find document by cycle ID.
    ///
    /// Each cycle has at most one document.
    ///
    /// # Arguments
    ///
    /// * `cycle_id` - The cycle's unique identifier
    ///
    /// # Returns
    ///
    /// The document if found, None otherwise.
    async fn find_by_cycle(&self, cycle_id: CycleId) -> Result<Option<DecisionDocument>, DomainError>;

    /// Sync document from filesystem changes.
    ///
    /// Reads the file, computes checksum, and updates DB if changed.
    /// Used when external tools (agents) modify the file directly.
    ///
    /// # Arguments
    ///
    /// * `document_id` - The document to sync
    ///
    /// # Returns
    ///
    /// Sync result indicating what changed.
    async fn sync_from_file(
        &self,
        document_id: DecisionDocumentId,
    ) -> Result<SyncResult, DomainError>;

    /// Verify file and database are in sync.
    ///
    /// Compares checksums without modifying anything.
    ///
    /// # Arguments
    ///
    /// * `document_id` - The document to verify
    ///
    /// # Returns
    ///
    /// The integrity status.
    async fn verify_integrity(
        &self,
        document_id: DecisionDocumentId,
    ) -> Result<IntegrityStatus, DomainError>;

    /// Delete a document (removes both DB record and file).
    ///
    /// # Arguments
    ///
    /// * `document_id` - The document to delete
    ///
    /// # Errors
    ///
    /// Returns `DomainError` if document doesn't exist.
    async fn delete(&self, document_id: DecisionDocumentId) -> Result<(), DomainError>;
}

/// Result of syncing a document from file changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Whether the file had changes.
    pub changed: bool,

    /// The new checksum (after sync).
    pub new_checksum: String,

    /// The new version number.
    pub new_version: u32,
}

impl SyncResult {
    /// Creates a result indicating no changes.
    pub fn unchanged(checksum: impl Into<String>, version: u32) -> Self {
        Self {
            changed: false,
            new_checksum: checksum.into(),
            new_version: version,
        }
    }

    /// Creates a result indicating changes were synced.
    pub fn changed(checksum: impl Into<String>, version: u32) -> Self {
        Self {
            changed: true,
            new_checksum: checksum.into(),
            new_version: version,
        }
    }
}

/// Status of document integrity between file and database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum IntegrityStatus {
    /// File and database are in sync.
    InSync,

    /// File has been modified externally.
    FileModified {
        /// Checksum from the file.
        file_checksum: String,
        /// Checksum stored in database.
        db_checksum: String,
    },

    /// File is missing from filesystem.
    FileMissing,

    /// Database record is missing.
    DbRecordMissing,
}

impl IntegrityStatus {
    /// Returns true if file and database are in sync.
    pub fn is_in_sync(&self) -> bool {
        matches!(self, IntegrityStatus::InSync)
    }

    /// Returns true if file was modified externally.
    pub fn is_file_modified(&self) -> bool {
        matches!(self, IntegrityStatus::FileModified { .. })
    }

    /// Returns true if the file is missing.
    pub fn is_file_missing(&self) -> bool {
        matches!(self, IntegrityStatus::FileMissing)
    }

    /// Returns true if the database record is missing.
    pub fn is_db_missing(&self) -> bool {
        matches!(self, IntegrityStatus::DbRecordMissing)
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ───────────────────────────────────────────────────────────────
    // SyncResult tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn sync_result_unchanged() {
        let result = SyncResult::unchanged("abc123", 5);
        assert!(!result.changed);
        assert_eq!(result.new_checksum, "abc123");
        assert_eq!(result.new_version, 5);
    }

    #[test]
    fn sync_result_changed() {
        let result = SyncResult::changed("def456", 6);
        assert!(result.changed);
        assert_eq!(result.new_checksum, "def456");
        assert_eq!(result.new_version, 6);
    }

    // ───────────────────────────────────────────────────────────────
    // IntegrityStatus tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn integrity_status_in_sync() {
        let status = IntegrityStatus::InSync;
        assert!(status.is_in_sync());
        assert!(!status.is_file_modified());
        assert!(!status.is_file_missing());
        assert!(!status.is_db_missing());
    }

    #[test]
    fn integrity_status_file_modified() {
        let status = IntegrityStatus::FileModified {
            file_checksum: "new".to_string(),
            db_checksum: "old".to_string(),
        };
        assert!(!status.is_in_sync());
        assert!(status.is_file_modified());
        assert!(!status.is_file_missing());
        assert!(!status.is_db_missing());
    }

    #[test]
    fn integrity_status_file_missing() {
        let status = IntegrityStatus::FileMissing;
        assert!(!status.is_in_sync());
        assert!(!status.is_file_modified());
        assert!(status.is_file_missing());
        assert!(!status.is_db_missing());
    }

    #[test]
    fn integrity_status_db_missing() {
        let status = IntegrityStatus::DbRecordMissing;
        assert!(!status.is_in_sync());
        assert!(!status.is_file_modified());
        assert!(!status.is_file_missing());
        assert!(status.is_db_missing());
    }

    #[test]
    fn integrity_status_serializes_correctly() {
        let in_sync = IntegrityStatus::InSync;
        let json = serde_json::to_string(&in_sync).unwrap();
        assert!(json.contains("in_sync"));

        let file_modified = IntegrityStatus::FileModified {
            file_checksum: "abc".to_string(),
            db_checksum: "def".to_string(),
        };
        let json = serde_json::to_string(&file_modified).unwrap();
        assert!(json.contains("file_modified"));
        assert!(json.contains("file_checksum"));
    }

    // ───────────────────────────────────────────────────────────────
    // Trait object safety test
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn decision_document_repository_is_object_safe() {
        fn check<T: DecisionDocumentRepository + ?Sized>() {}
        // This compiles only if the trait is object-safe
        check::<dyn DecisionDocumentRepository>();
    }
}
