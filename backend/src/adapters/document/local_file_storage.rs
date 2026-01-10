//! Local Filesystem Storage Adapter - Implementation of DocumentFileStorage.
//!
//! Stores decision documents as markdown files in a user-organized directory structure.
//! Uses atomic writes and SHA-256 checksums for data integrity.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::SystemTime;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::domain::foundation::{DecisionDocumentId, Timestamp, UserId};
use crate::ports::{
    DocumentFileStorage, FileInfo, FileMetadata, FilePath, StorageError,
};

/// Maximum file size allowed (10 MB).
const MAX_FILE_SIZE_BYTES: u64 = 10 * 1024 * 1024;

/// Local filesystem storage for decision documents.
///
/// # Directory Structure
///
/// ```text
/// {base_path}/
/// ├── user_abc123/
/// │   ├── doc_def456.md
/// │   └── doc_ghi789.md
/// └── user_xyz987/
///     └── doc_aaa111.md
/// ```
///
/// # Atomic Writes
///
/// Uses a write-to-temp-then-rename pattern to ensure atomic writes:
/// 1. Write content to `doc_{id}.md.tmp`
/// 2. Sync to disk
/// 3. Rename to `doc_{id}.md`
///
/// This prevents partial writes if the process crashes during write.
///
/// # Usage
///
/// ```rust,ignore
/// let storage = LocalDocumentFileStorage::new("/var/decisions");
///
/// // Write a document
/// let path = storage.write(&user_id, doc_id, "# My Decision").await?;
///
/// // Read it back
/// let content = storage.read(&user_id, doc_id).await?;
///
/// // Get metadata
/// let meta = storage.metadata(&user_id, doc_id).await?;
/// println!("Size: {} bytes, Checksum: {}", meta.size_bytes, meta.checksum);
/// ```
#[derive(Debug, Clone)]
pub struct LocalDocumentFileStorage {
    /// Base directory for all document storage.
    base_path: PathBuf,
}

impl LocalDocumentFileStorage {
    /// Creates a new local file storage with the given base path.
    ///
    /// # Arguments
    ///
    /// * `base_path` - The root directory for document storage
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let storage = LocalDocumentFileStorage::new("/var/decisions");
    /// ```
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// Returns the user directory path.
    fn user_dir(&self, user_id: &UserId) -> PathBuf {
        self.base_path.join(format!("user_{}", user_id.as_str()))
    }

    /// Returns the full file path for a document.
    fn document_path(&self, user_id: &UserId, document_id: DecisionDocumentId) -> PathBuf {
        self.user_dir(user_id)
            .join(format!("doc_{}.md", document_id))
    }

    /// Returns the temporary file path for atomic writes.
    fn temp_path(&self, user_id: &UserId, document_id: DecisionDocumentId) -> PathBuf {
        self.user_dir(user_id)
            .join(format!("doc_{}.md.tmp", document_id))
    }

    /// Ensures the user directory exists.
    async fn ensure_user_dir(&self, user_id: &UserId) -> Result<(), StorageError> {
        let dir = self.user_dir(user_id);
        fs::create_dir_all(&dir).await.map_err(|e| {
            StorageError::io(format!(
                "Failed to create user directory {}: {}",
                dir.display(),
                e
            ))
        })
    }

    /// Computes SHA-256 checksum of the given content.
    fn compute_checksum(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Extracts document ID from a filename like "doc_{id}.md".
    fn parse_document_id(filename: &str) -> Option<DecisionDocumentId> {
        if !filename.starts_with("doc_") || !filename.ends_with(".md") {
            return None;
        }
        // Ignore temp files
        if filename.ends_with(".tmp") {
            return None;
        }
        let id_str = &filename[4..filename.len() - 3]; // Remove "doc_" and ".md"
        DecisionDocumentId::from_str(id_str).ok()
    }

    /// Converts SystemTime to Timestamp.
    fn system_time_to_timestamp(system_time: SystemTime) -> Timestamp {
        let datetime: DateTime<Utc> = system_time.into();
        Timestamp::from_datetime(datetime)
    }
}

#[async_trait]
impl DocumentFileStorage for LocalDocumentFileStorage {
    async fn write(
        &self,
        user_id: &UserId,
        document_id: DecisionDocumentId,
        content: &str,
    ) -> Result<FilePath, StorageError> {
        // Check file size
        let size = content.len() as u64;
        if size > MAX_FILE_SIZE_BYTES {
            return Err(StorageError::file_too_large(size, MAX_FILE_SIZE_BYTES));
        }

        // Ensure user directory exists
        self.ensure_user_dir(user_id).await?;

        let temp_path = self.temp_path(user_id, document_id);
        let final_path = self.document_path(user_id, document_id);

        // Write to temp file
        let mut file = fs::File::create(&temp_path).await.map_err(|e| {
            StorageError::io(format!(
                "Failed to create temp file {}: {}",
                temp_path.display(),
                e
            ))
        })?;

        file.write_all(content.as_bytes()).await.map_err(|e| {
            StorageError::io(format!(
                "Failed to write to temp file {}: {}",
                temp_path.display(),
                e
            ))
        })?;

        // Sync to disk
        file.sync_all().await.map_err(|e| {
            StorageError::io(format!(
                "Failed to sync temp file {}: {}",
                temp_path.display(),
                e
            ))
        })?;

        // Atomic rename
        fs::rename(&temp_path, &final_path).await.map_err(|e| {
            StorageError::io(format!(
                "Failed to rename {} to {}: {}",
                temp_path.display(),
                final_path.display(),
                e
            ))
        })?;

        Ok(FilePath::new(final_path))
    }

    async fn read(
        &self,
        user_id: &UserId,
        document_id: DecisionDocumentId,
    ) -> Result<String, StorageError> {
        let path = self.document_path(user_id, document_id);

        fs::read_to_string(&path)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => StorageError::not_found(path.display().to_string()),
                std::io::ErrorKind::PermissionDenied => {
                    StorageError::permission_denied(path.display().to_string())
                }
                _ => StorageError::io(format!("Failed to read {}: {}", path.display(), e)),
            })
    }

    async fn exists(
        &self,
        user_id: &UserId,
        document_id: DecisionDocumentId,
    ) -> Result<bool, StorageError> {
        let path = self.document_path(user_id, document_id);
        Ok(path.exists())
    }

    async fn delete(
        &self,
        user_id: &UserId,
        document_id: DecisionDocumentId,
    ) -> Result<(), StorageError> {
        let path = self.document_path(user_id, document_id);

        fs::remove_file(&path).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => StorageError::not_found(path.display().to_string()),
            std::io::ErrorKind::PermissionDenied => {
                StorageError::permission_denied(path.display().to_string())
            }
            _ => StorageError::io(format!("Failed to delete {}: {}", path.display(), e)),
        })
    }

    async fn metadata(
        &self,
        user_id: &UserId,
        document_id: DecisionDocumentId,
    ) -> Result<FileMetadata, StorageError> {
        let path = self.document_path(user_id, document_id);

        // Get file metadata
        let file_meta = fs::metadata(&path).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => StorageError::not_found(path.display().to_string()),
            std::io::ErrorKind::PermissionDenied => {
                StorageError::permission_denied(path.display().to_string())
            }
            _ => StorageError::io(format!("Failed to get metadata for {}: {}", path.display(), e)),
        })?;

        // Get modification time
        let modified = file_meta.modified().map_err(|e| {
            StorageError::io(format!(
                "Failed to get modification time for {}: {}",
                path.display(),
                e
            ))
        })?;

        // Read content for checksum
        let content = self.read(user_id, document_id).await?;
        let checksum = Self::compute_checksum(&content);

        Ok(FileMetadata::new(
            file_meta.len(),
            Self::system_time_to_timestamp(modified),
            checksum,
        ))
    }

    async fn list_user_files(&self, user_id: &UserId) -> Result<Vec<FileInfo>, StorageError> {
        let user_dir = self.user_dir(user_id);

        // Check if directory exists
        if !user_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = fs::read_dir(&user_dir).await.map_err(|e| {
            StorageError::io(format!(
                "Failed to read user directory {}: {}",
                user_dir.display(),
                e
            ))
        })?;

        let mut files = Vec::new();

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            StorageError::io(format!("Failed to read directory entry: {}", e))
        })? {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Skip non-document files
            if let Some(document_id) = Self::parse_document_id(&file_name_str) {
                let path = entry.path();
                let meta = entry.metadata().await.map_err(|e| {
                    StorageError::io(format!(
                        "Failed to get metadata for {}: {}",
                        path.display(),
                        e
                    ))
                })?;

                let modified = meta.modified().map_err(|e| {
                    StorageError::io(format!(
                        "Failed to get modification time for {}: {}",
                        path.display(),
                        e
                    ))
                })?;

                files.push(FileInfo::new(
                    document_id,
                    FilePath::new(path),
                    meta.len(),
                    Self::system_time_to_timestamp(modified),
                ));
            }
        }

        // Sort by modification time, newest first
        files.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

        Ok(files)
    }

    async fn checksum(
        &self,
        user_id: &UserId,
        document_id: DecisionDocumentId,
    ) -> Result<String, StorageError> {
        let content = self.read(user_id, document_id).await?;
        Ok(Self::compute_checksum(&content))
    }

    fn file_path(&self, user_id: &UserId, document_id: DecisionDocumentId) -> FilePath {
        FilePath::new(self.document_path(user_id, document_id))
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ───────────────────────────────────────────────────────────────
    // Test helpers
    // ───────────────────────────────────────────────────────────────

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    fn test_document_id() -> DecisionDocumentId {
        DecisionDocumentId::new()
    }

    fn test_content() -> &'static str {
        "# My Decision\n\nThis is a test document.\n"
    }

    fn create_storage() -> (LocalDocumentFileStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalDocumentFileStorage::new(temp_dir.path());
        (storage, temp_dir)
    }

    // ───────────────────────────────────────────────────────────────
    // Write tests
    // ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn write_creates_file() {
        let (storage, _temp) = create_storage();
        let user_id = test_user_id();
        let doc_id = test_document_id();

        let result = storage.write(&user_id, doc_id, test_content()).await;

        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.as_path().exists());
    }

    #[tokio::test]
    async fn write_creates_user_directory() {
        let (storage, temp) = create_storage();
        let user_id = test_user_id();
        let doc_id = test_document_id();

        storage.write(&user_id, doc_id, test_content()).await.unwrap();

        let user_dir = temp.path().join(format!("user_{}", user_id.as_str()));
        assert!(user_dir.exists());
        assert!(user_dir.is_dir());
    }

    #[tokio::test]
    async fn write_content_is_correct() {
        let (storage, _temp) = create_storage();
        let user_id = test_user_id();
        let doc_id = test_document_id();
        let content = "# Test Content\n\nWith multiple lines.\n";

        storage.write(&user_id, doc_id, content).await.unwrap();

        let read_content = storage.read(&user_id, doc_id).await.unwrap();
        assert_eq!(read_content, content);
    }

    #[tokio::test]
    async fn write_overwrites_existing() {
        let (storage, _temp) = create_storage();
        let user_id = test_user_id();
        let doc_id = test_document_id();

        storage.write(&user_id, doc_id, "Original").await.unwrap();
        storage.write(&user_id, doc_id, "Updated").await.unwrap();

        let content = storage.read(&user_id, doc_id).await.unwrap();
        assert_eq!(content, "Updated");
    }

    #[tokio::test]
    async fn write_rejects_oversized_content() {
        let (storage, _temp) = create_storage();
        let user_id = test_user_id();
        let doc_id = test_document_id();
        let large_content = "x".repeat((MAX_FILE_SIZE_BYTES + 1) as usize);

        let result = storage.write(&user_id, doc_id, &large_content).await;

        assert!(matches!(result, Err(StorageError::FileTooLarge { .. })));
    }

    // ───────────────────────────────────────────────────────────────
    // Read tests
    // ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn read_returns_content() {
        let (storage, _temp) = create_storage();
        let user_id = test_user_id();
        let doc_id = test_document_id();
        let content = "# Reading Test\n";

        storage.write(&user_id, doc_id, content).await.unwrap();

        let result = storage.read(&user_id, doc_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), content);
    }

    #[tokio::test]
    async fn read_returns_not_found_for_missing() {
        let (storage, _temp) = create_storage();
        let user_id = test_user_id();
        let doc_id = test_document_id();

        let result = storage.read(&user_id, doc_id).await;

        assert!(matches!(result, Err(StorageError::NotFound { .. })));
    }

    // ───────────────────────────────────────────────────────────────
    // Exists tests
    // ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn exists_returns_true_when_file_exists() {
        let (storage, _temp) = create_storage();
        let user_id = test_user_id();
        let doc_id = test_document_id();

        storage.write(&user_id, doc_id, test_content()).await.unwrap();

        let result = storage.exists(&user_id, doc_id).await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn exists_returns_false_when_file_missing() {
        let (storage, _temp) = create_storage();
        let user_id = test_user_id();
        let doc_id = test_document_id();

        let result = storage.exists(&user_id, doc_id).await.unwrap();
        assert!(!result);
    }

    // ───────────────────────────────────────────────────────────────
    // Delete tests
    // ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn delete_removes_file() {
        let (storage, _temp) = create_storage();
        let user_id = test_user_id();
        let doc_id = test_document_id();

        storage.write(&user_id, doc_id, test_content()).await.unwrap();
        storage.delete(&user_id, doc_id).await.unwrap();

        let exists = storage.exists(&user_id, doc_id).await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    async fn delete_returns_not_found_for_missing() {
        let (storage, _temp) = create_storage();
        let user_id = test_user_id();
        let doc_id = test_document_id();

        let result = storage.delete(&user_id, doc_id).await;

        assert!(matches!(result, Err(StorageError::NotFound { .. })));
    }

    // ───────────────────────────────────────────────────────────────
    // Metadata tests
    // ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn metadata_returns_correct_size() {
        let (storage, _temp) = create_storage();
        let user_id = test_user_id();
        let doc_id = test_document_id();
        let content = "12345"; // 5 bytes

        storage.write(&user_id, doc_id, content).await.unwrap();

        let meta = storage.metadata(&user_id, doc_id).await.unwrap();
        assert_eq!(meta.size_bytes, 5);
    }

    #[tokio::test]
    async fn metadata_returns_valid_checksum() {
        let (storage, _temp) = create_storage();
        let user_id = test_user_id();
        let doc_id = test_document_id();
        let content = "test content";

        storage.write(&user_id, doc_id, content).await.unwrap();

        let meta = storage.metadata(&user_id, doc_id).await.unwrap();
        // Checksum should be 64 hex characters (SHA-256)
        assert_eq!(meta.checksum.len(), 64);
        // Same content should produce same checksum
        let expected = LocalDocumentFileStorage::compute_checksum(content);
        assert_eq!(meta.checksum, expected);
    }

    #[tokio::test]
    async fn metadata_returns_not_found_for_missing() {
        let (storage, _temp) = create_storage();
        let user_id = test_user_id();
        let doc_id = test_document_id();

        let result = storage.metadata(&user_id, doc_id).await;

        assert!(matches!(result, Err(StorageError::NotFound { .. })));
    }

    // ───────────────────────────────────────────────────────────────
    // List tests
    // ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn list_returns_empty_for_new_user() {
        let (storage, _temp) = create_storage();
        let user_id = test_user_id();

        let files = storage.list_user_files(&user_id).await.unwrap();

        assert!(files.is_empty());
    }

    #[tokio::test]
    async fn list_returns_all_user_documents() {
        let (storage, _temp) = create_storage();
        let user_id = test_user_id();
        let doc1 = DecisionDocumentId::new();
        let doc2 = DecisionDocumentId::new();

        storage.write(&user_id, doc1, "Doc 1").await.unwrap();
        storage.write(&user_id, doc2, "Doc 2").await.unwrap();

        let files = storage.list_user_files(&user_id).await.unwrap();

        assert_eq!(files.len(), 2);
        let ids: Vec<_> = files.iter().map(|f| f.document_id).collect();
        assert!(ids.contains(&doc1));
        assert!(ids.contains(&doc2));
    }

    #[tokio::test]
    async fn list_ignores_other_users() {
        let (storage, _temp) = create_storage();
        let user1 = UserId::new("user-1").unwrap();
        let user2 = UserId::new("user-2").unwrap();
        let doc1 = DecisionDocumentId::new();
        let doc2 = DecisionDocumentId::new();

        storage.write(&user1, doc1, "User 1 doc").await.unwrap();
        storage.write(&user2, doc2, "User 2 doc").await.unwrap();

        let files = storage.list_user_files(&user1).await.unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].document_id, doc1);
    }

    // ───────────────────────────────────────────────────────────────
    // Checksum tests
    // ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn checksum_returns_consistent_hash() {
        let (storage, _temp) = create_storage();
        let user_id = test_user_id();
        let doc_id = test_document_id();
        let content = "consistent content";

        storage.write(&user_id, doc_id, content).await.unwrap();

        let checksum1 = storage.checksum(&user_id, doc_id).await.unwrap();
        let checksum2 = storage.checksum(&user_id, doc_id).await.unwrap();

        assert_eq!(checksum1, checksum2);
    }

    #[tokio::test]
    async fn checksum_changes_with_content() {
        let (storage, _temp) = create_storage();
        let user_id = test_user_id();
        let doc_id = test_document_id();

        storage.write(&user_id, doc_id, "content v1").await.unwrap();
        let checksum1 = storage.checksum(&user_id, doc_id).await.unwrap();

        storage.write(&user_id, doc_id, "content v2").await.unwrap();
        let checksum2 = storage.checksum(&user_id, doc_id).await.unwrap();

        assert_ne!(checksum1, checksum2);
    }

    // ───────────────────────────────────────────────────────────────
    // File path tests
    // ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn file_path_returns_expected_structure() {
        let (storage, temp) = create_storage();
        let user_id = test_user_id();
        let doc_id = test_document_id();

        let path = storage.file_path(&user_id, doc_id);

        let expected = temp
            .path()
            .join(format!("user_{}", user_id.as_str()))
            .join(format!("doc_{}.md", doc_id));
        assert_eq!(path.as_path(), expected.as_path());
    }

    // ───────────────────────────────────────────────────────────────
    // Document ID parsing tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn parse_document_id_valid() {
        let doc_id = DecisionDocumentId::new();
        let filename = format!("doc_{}.md", doc_id);

        let parsed = LocalDocumentFileStorage::parse_document_id(&filename);

        assert!(parsed.is_some());
        assert_eq!(parsed.unwrap(), doc_id);
    }

    #[test]
    fn parse_document_id_ignores_tmp_files() {
        let filename = "doc_abc123.md.tmp";

        let parsed = LocalDocumentFileStorage::parse_document_id(filename);

        assert!(parsed.is_none());
    }

    #[test]
    fn parse_document_id_ignores_non_doc_files() {
        let filenames = ["readme.md", "config.json", "backup_doc.md"];

        for filename in filenames {
            let parsed = LocalDocumentFileStorage::parse_document_id(filename);
            assert!(parsed.is_none(), "Should ignore: {}", filename);
        }
    }
}
