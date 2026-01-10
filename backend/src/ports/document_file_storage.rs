//! Document File Storage Port - Filesystem operations interface.
//!
//! This port defines the contract for filesystem operations on decision documents.
//! The domain depends on this trait, while adapters (like LocalFileStorage)
//! provide the implementation.

use async_trait::async_trait;
use std::path::PathBuf;
use thiserror::Error;

use crate::domain::foundation::{DecisionDocumentId, Timestamp, UserId};

/// Port for filesystem operations on decision documents.
///
/// # Contract
///
/// Implementations must:
/// - Organize files by user ID in a hierarchical structure
/// - Support atomic writes (no partial content on failure)
/// - Compute SHA-256 checksums for integrity verification
/// - Handle file metadata (size, modification time)
///
/// # File Organization
///
/// Files are stored in a user-specific directory structure:
/// ```text
/// {base_path}/{user_id}/doc_{document_id}.md
/// ```
///
/// # Usage
///
/// ```rust,ignore
/// let storage: &dyn DocumentFileStorage = get_storage();
///
/// // Write document
/// let path = storage.write(&user_id, doc_id, "# My Decision").await?;
///
/// // Read back
/// let content = storage.read(&user_id, doc_id).await?;
///
/// // Check integrity
/// let checksum = storage.checksum(&user_id, doc_id).await?;
/// ```
#[async_trait]
pub trait DocumentFileStorage: Send + Sync {
    /// Write document content to filesystem.
    ///
    /// Creates the user directory if it doesn't exist.
    /// Writes atomically (using a temp file + rename pattern).
    ///
    /// # Arguments
    ///
    /// * `user_id` - The owner's user ID (determines directory)
    /// * `document_id` - The document's unique identifier
    /// * `content` - The markdown content to write
    ///
    /// # Returns
    ///
    /// The path where the file was written.
    async fn write(
        &self,
        user_id: &UserId,
        document_id: DecisionDocumentId,
        content: &str,
    ) -> Result<FilePath, StorageError>;

    /// Read document content from filesystem.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The owner's user ID
    /// * `document_id` - The document's unique identifier
    ///
    /// # Returns
    ///
    /// The file content as a string.
    ///
    /// # Errors
    ///
    /// Returns `StorageError::NotFound` if the file doesn't exist.
    async fn read(
        &self,
        user_id: &UserId,
        document_id: DecisionDocumentId,
    ) -> Result<String, StorageError>;

    /// Check if document file exists.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The owner's user ID
    /// * `document_id` - The document's unique identifier
    ///
    /// # Returns
    ///
    /// `true` if the file exists, `false` otherwise.
    async fn exists(
        &self,
        user_id: &UserId,
        document_id: DecisionDocumentId,
    ) -> Result<bool, StorageError>;

    /// Delete document file.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The owner's user ID
    /// * `document_id` - The document's unique identifier
    ///
    /// # Errors
    ///
    /// Returns `StorageError::NotFound` if the file doesn't exist.
    async fn delete(
        &self,
        user_id: &UserId,
        document_id: DecisionDocumentId,
    ) -> Result<(), StorageError>;

    /// Get file metadata (size, modified time).
    ///
    /// # Arguments
    ///
    /// * `user_id` - The owner's user ID
    /// * `document_id` - The document's unique identifier
    ///
    /// # Returns
    ///
    /// File metadata including size, modification time, and checksum.
    async fn metadata(
        &self,
        user_id: &UserId,
        document_id: DecisionDocumentId,
    ) -> Result<FileMetadata, StorageError>;

    /// List all document files for a user.
    ///
    /// Used for sync/recovery operations.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The owner's user ID
    ///
    /// # Returns
    ///
    /// A list of file info for all documents belonging to this user.
    async fn list_user_files(&self, user_id: &UserId) -> Result<Vec<FileInfo>, StorageError>;

    /// Compute SHA-256 checksum of file content.
    ///
    /// Reads the file and computes the checksum without loading into memory fully.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The owner's user ID
    /// * `document_id` - The document's unique identifier
    ///
    /// # Returns
    ///
    /// The hex-encoded SHA-256 checksum of the file content.
    async fn checksum(
        &self,
        user_id: &UserId,
        document_id: DecisionDocumentId,
    ) -> Result<String, StorageError>;

    /// Get the full file path for a document.
    ///
    /// Useful for external integrations or debugging.
    fn file_path(&self, user_id: &UserId, document_id: DecisionDocumentId) -> FilePath;
}

/// Represents a file path (absolute or relative).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePath(PathBuf);

impl FilePath {
    /// Creates a new file path from a PathBuf.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    /// Returns the path as a string (lossy conversion for non-UTF8 paths).
    pub fn to_string_lossy(&self) -> String {
        self.0.to_string_lossy().to_string()
    }

    /// Returns the relative path as a string (for database storage).
    pub fn relative(&self) -> String {
        self.to_string_lossy()
    }

    /// Returns a reference to the inner PathBuf.
    pub fn as_path(&self) -> &std::path::Path {
        &self.0
    }

    /// Returns the inner PathBuf.
    pub fn into_inner(self) -> PathBuf {
        self.0
    }

    /// Returns the file name without the directory.
    pub fn file_name(&self) -> Option<String> {
        self.0
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
    }

    /// Returns the parent directory.
    pub fn parent(&self) -> Option<FilePath> {
        self.0.parent().map(|p| FilePath::new(p.to_path_buf()))
    }
}

impl std::fmt::Display for FilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_lossy())
    }
}

impl From<PathBuf> for FilePath {
    fn from(path: PathBuf) -> Self {
        Self::new(path)
    }
}

impl From<&str> for FilePath {
    fn from(s: &str) -> Self {
        Self::new(PathBuf::from(s))
    }
}

impl From<String> for FilePath {
    fn from(s: String) -> Self {
        Self::new(PathBuf::from(s))
    }
}

/// File metadata information.
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// File size in bytes.
    pub size_bytes: u64,

    /// Last modification timestamp.
    pub modified_at: Timestamp,

    /// SHA-256 checksum of the content.
    pub checksum: String,
}

impl FileMetadata {
    /// Creates new file metadata.
    pub fn new(size_bytes: u64, modified_at: Timestamp, checksum: impl Into<String>) -> Self {
        Self {
            size_bytes,
            modified_at,
            checksum: checksum.into(),
        }
    }
}

/// Information about a stored file.
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// The document ID extracted from the filename.
    pub document_id: DecisionDocumentId,

    /// The full file path.
    pub path: FilePath,

    /// File size in bytes.
    pub size_bytes: u64,

    /// Last modification timestamp.
    pub modified_at: Timestamp,
}

impl FileInfo {
    /// Creates new file info.
    pub fn new(
        document_id: DecisionDocumentId,
        path: impl Into<FilePath>,
        size_bytes: u64,
        modified_at: Timestamp,
    ) -> Self {
        Self {
            document_id,
            path: path.into(),
            size_bytes,
            modified_at,
        }
    }
}

/// Errors that can occur during file storage operations.
#[derive(Debug, Clone, Error)]
pub enum StorageError {
    /// File was not found.
    #[error("File not found: {path}")]
    NotFound { path: String },

    /// Permission denied accessing the file.
    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    /// IO error during file operation.
    #[error("IO error: {message}")]
    Io { message: String },

    /// User directory doesn't exist and couldn't be created.
    #[error("User directory not found: {user_id}")]
    UserDirectoryNotFound { user_id: String },

    /// Invalid document ID in filename.
    #[error("Invalid document ID in filename: {filename}")]
    InvalidDocumentId { filename: String },

    /// File is too large.
    #[error("File too large: {size_bytes} bytes (max: {max_bytes})")]
    FileTooLarge { size_bytes: u64, max_bytes: u64 },
}

impl StorageError {
    /// Creates a not found error.
    pub fn not_found(path: impl Into<String>) -> Self {
        Self::NotFound { path: path.into() }
    }

    /// Creates a permission denied error.
    pub fn permission_denied(path: impl Into<String>) -> Self {
        Self::PermissionDenied { path: path.into() }
    }

    /// Creates an IO error.
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            message: message.into(),
        }
    }

    /// Creates a user directory not found error.
    pub fn user_directory_not_found(user_id: impl Into<String>) -> Self {
        Self::UserDirectoryNotFound {
            user_id: user_id.into(),
        }
    }

    /// Creates an invalid document ID error.
    pub fn invalid_document_id(filename: impl Into<String>) -> Self {
        Self::InvalidDocumentId {
            filename: filename.into(),
        }
    }

    /// Creates a file too large error.
    pub fn file_too_large(size_bytes: u64, max_bytes: u64) -> Self {
        Self::FileTooLarge {
            size_bytes,
            max_bytes,
        }
    }
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => StorageError::not_found(err.to_string()),
            std::io::ErrorKind::PermissionDenied => {
                StorageError::permission_denied(err.to_string())
            }
            _ => StorageError::io(err.to_string()),
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ───────────────────────────────────────────────────────────────
    // FilePath tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn file_path_from_string() {
        let path: FilePath = "/home/user/doc.md".into();
        assert_eq!(path.to_string_lossy(), "/home/user/doc.md");
    }

    #[test]
    fn file_path_from_pathbuf() {
        let pathbuf = PathBuf::from("/tmp/test.md");
        let path: FilePath = pathbuf.into();
        assert_eq!(path.to_string_lossy(), "/tmp/test.md");
    }

    #[test]
    fn file_path_relative_returns_string() {
        let path = FilePath::new("/users/alice/docs/decision.md");
        assert_eq!(path.relative(), "/users/alice/docs/decision.md");
    }

    #[test]
    fn file_path_file_name_extracts_name() {
        let path = FilePath::new("/path/to/document.md");
        assert_eq!(path.file_name(), Some("document.md".to_string()));
    }

    #[test]
    fn file_path_parent_returns_parent_dir() {
        let path = FilePath::new("/path/to/document.md");
        let parent = path.parent().unwrap();
        assert_eq!(parent.to_string_lossy(), "/path/to");
    }

    #[test]
    fn file_path_display_works() {
        let path = FilePath::new("/test/path.md");
        assert_eq!(format!("{}", path), "/test/path.md");
    }

    // ───────────────────────────────────────────────────────────────
    // FileMetadata tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn file_metadata_creation() {
        let now = Timestamp::now();
        let metadata = FileMetadata::new(1024, now, "abc123");

        assert_eq!(metadata.size_bytes, 1024);
        assert_eq!(metadata.modified_at, now);
        assert_eq!(metadata.checksum, "abc123");
    }

    // ───────────────────────────────────────────────────────────────
    // FileInfo tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn file_info_creation() {
        let doc_id = DecisionDocumentId::new();
        let now = Timestamp::now();
        let info = FileInfo::new(doc_id, "/path/to/doc.md", 2048, now);

        assert_eq!(info.document_id, doc_id);
        assert_eq!(info.size_bytes, 2048);
        assert_eq!(info.path.to_string_lossy(), "/path/to/doc.md");
    }

    // ───────────────────────────────────────────────────────────────
    // StorageError tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn storage_error_not_found_displays_path() {
        let err = StorageError::not_found("/missing/file.md");
        assert!(err.to_string().contains("/missing/file.md"));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn storage_error_permission_denied_displays_path() {
        let err = StorageError::permission_denied("/protected/file.md");
        assert!(err.to_string().contains("/protected/file.md"));
        assert!(err.to_string().contains("Permission denied"));
    }

    #[test]
    fn storage_error_io_displays_message() {
        let err = StorageError::io("disk full");
        assert!(err.to_string().contains("disk full"));
    }

    #[test]
    fn storage_error_user_directory_displays_user_id() {
        let err = StorageError::user_directory_not_found("user-123");
        assert!(err.to_string().contains("user-123"));
    }

    #[test]
    fn storage_error_file_too_large_displays_sizes() {
        let err = StorageError::file_too_large(10_000_000, 5_000_000);
        assert!(err.to_string().contains("10000000"));
        assert!(err.to_string().contains("5000000"));
    }

    #[test]
    fn storage_error_from_io_error_not_found() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let storage_err: StorageError = io_err.into();
        assert!(matches!(storage_err, StorageError::NotFound { .. }));
    }

    #[test]
    fn storage_error_from_io_error_permission_denied() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let storage_err: StorageError = io_err.into();
        assert!(matches!(storage_err, StorageError::PermissionDenied { .. }));
    }

    #[test]
    fn storage_error_from_io_error_other() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "unknown error");
        let storage_err: StorageError = io_err.into();
        assert!(matches!(storage_err, StorageError::Io { .. }));
    }

    // ───────────────────────────────────────────────────────────────
    // Trait object safety test
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn document_file_storage_is_object_safe() {
        fn check<T: DocumentFileStorage + ?Sized>() {}
        // This compiles only if the trait is object-safe
        check::<dyn DocumentFileStorage>();
    }
}
