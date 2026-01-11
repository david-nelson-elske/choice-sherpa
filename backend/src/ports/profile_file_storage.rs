//! ProfileFileStorage port for filesystem operations

use async_trait::async_trait;
use std::path::PathBuf;

use crate::domain::foundation::UserId;

/// Errors that can occur during file storage operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    /// File not found
    NotFound(String),
    /// Permission denied
    PermissionDenied(String),
    /// IO error
    IoError(String),
    /// Invalid path
    InvalidPath(String),
    /// Checksum mismatch
    ChecksumMismatch { expected: String, actual: String },
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "File not found: {}", msg),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
            Self::InvalidPath(msg) => write!(f, "Invalid path: {}", msg),
            Self::ChecksumMismatch { expected, actual } => {
                write!(
                    f,
                    "Checksum mismatch: expected {}, got {}",
                    expected, actual
                )
            }
        }
    }
}

impl std::error::Error for StorageError {}

/// File storage operations for profile markdown files
#[async_trait]
pub trait ProfileFileStorage: Send + Sync {
    /// Write profile markdown to filesystem
    ///
    /// Returns the file path where the profile was written
    async fn write(&self, user_id: &UserId, content: &str) -> Result<PathBuf, StorageError>;

    /// Read profile markdown from filesystem
    async fn read(&self, user_id: &UserId) -> Result<String, StorageError>;

    /// Check if profile file exists for user
    async fn exists(&self, user_id: &UserId) -> Result<bool, StorageError>;

    /// Delete profile file from filesystem
    async fn delete(&self, user_id: &UserId) -> Result<(), StorageError>;

    /// Compute checksum for content
    fn compute_checksum(&self, content: &str) -> String;

    /// Get file path for user's profile
    fn get_profile_path(&self, user_id: &UserId) -> PathBuf;
}
