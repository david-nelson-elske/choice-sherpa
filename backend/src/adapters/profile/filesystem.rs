//! Filesystem storage adapter for profile markdown files

use async_trait::async_trait;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::domain::foundation::UserId;
use crate::ports::{ProfileFileStorage, StorageError};

/// Filesystem-based profile storage
///
/// Stores profile markdown files in a configurable base directory
/// organized by user ID: {base_dir}/profiles/{user_id}/profile.md
pub struct FsProfileStorage {
    base_dir: PathBuf,
}

impl FsProfileStorage {
    /// Create new filesystem storage with base directory
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Get profile directory for a user
    fn get_user_dir(&self, user_id: &UserId) -> PathBuf {
        self.base_dir.join("profiles").join(user_id.as_str())
    }

    /// Get full path to profile file
    fn get_file_path(&self, user_id: &UserId) -> PathBuf {
        self.get_user_dir(user_id).join("profile.md")
    }

    /// Ensure parent directory exists
    async fn ensure_dir_exists(&self, path: &Path) -> Result<(), StorageError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError::IoError(format!("Failed to create directory: {}", e)))?;
        }
        Ok(())
    }
}

#[async_trait]
impl ProfileFileStorage for FsProfileStorage {
    async fn write(&self, user_id: &UserId, content: &str) -> Result<PathBuf, StorageError> {
        let file_path = self.get_file_path(user_id);

        // Ensure directory exists
        self.ensure_dir_exists(&file_path).await?;

        // Write file atomically using a temporary file
        let temp_path = file_path.with_extension("tmp");
        fs::write(&temp_path, content).await.map_err(|e| {
            StorageError::IoError(format!("Failed to write temporary file: {}", e))
        })?;

        // Rename to final location (atomic operation on Unix)
        fs::rename(&temp_path, &file_path).await.map_err(|e| {
            StorageError::IoError(format!("Failed to rename file: {}", e))
        })?;

        Ok(file_path)
    }

    async fn read(&self, user_id: &UserId) -> Result<String, StorageError> {
        let file_path = self.get_file_path(user_id);

        if !file_path.exists() {
            return Err(StorageError::NotFound(format!(
                "Profile not found for user: {}",
                user_id.as_str()
            )));
        }

        fs::read_to_string(&file_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                StorageError::PermissionDenied(format!("Cannot read file: {}", e))
            } else {
                StorageError::IoError(format!("Failed to read file: {}", e))
            }
        })
    }

    async fn exists(&self, user_id: &UserId) -> Result<bool, StorageError> {
        let file_path = self.get_file_path(user_id);
        Ok(file_path.exists())
    }

    async fn delete(&self, user_id: &UserId) -> Result<(), StorageError> {
        let file_path = self.get_file_path(user_id);

        if !file_path.exists() {
            // Not an error - idempotent delete
            return Ok(());
        }

        fs::remove_file(&file_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                StorageError::PermissionDenied(format!("Cannot delete file: {}", e))
            } else {
                StorageError::IoError(format!("Failed to delete file: {}", e))
            }
        })?;

        // Try to remove parent directory if empty
        let user_dir = self.get_user_dir(user_id);
        let _ = fs::remove_dir(&user_dir).await; // Ignore errors (directory might not be empty)

        Ok(())
    }

    fn compute_checksum(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn get_profile_path(&self, user_id: &UserId) -> PathBuf {
        self.get_file_path(user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_user_id() -> UserId {
        UserId::new("test-user@example.com".to_string()).unwrap()
    }

    #[tokio::test]
    async fn test_write_and_read() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FsProfileStorage::new(temp_dir.path());
        let user_id = test_user_id();

        let content = "# Test Profile\n\nThis is a test.";

        // Write
        let path = storage.write(&user_id, content).await.unwrap();
        assert!(path.exists());

        // Read
        let read_content = storage.read(&user_id).await.unwrap();
        assert_eq!(read_content, content);
    }

    #[tokio::test]
    async fn test_exists() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FsProfileStorage::new(temp_dir.path());
        let user_id = test_user_id();

        // Should not exist initially
        assert!(!storage.exists(&user_id).await.unwrap());

        // Write file
        storage.write(&user_id, "test").await.unwrap();

        // Should exist now
        assert!(storage.exists(&user_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_delete() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FsProfileStorage::new(temp_dir.path());
        let user_id = test_user_id();

        // Write file
        storage.write(&user_id, "test").await.unwrap();
        assert!(storage.exists(&user_id).await.unwrap());

        // Delete
        storage.delete(&user_id).await.unwrap();
        assert!(!storage.exists(&user_id).await.unwrap());

        // Delete again (should be idempotent)
        storage.delete(&user_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_read_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FsProfileStorage::new(temp_dir.path());
        let user_id = test_user_id();

        let result = storage.read(&user_id).await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_compute_checksum() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FsProfileStorage::new(temp_dir.path());

        let content = "test content";
        let checksum1 = storage.compute_checksum(content);
        let checksum2 = storage.compute_checksum(content);

        // Same content should produce same checksum
        assert_eq!(checksum1, checksum2);

        // Different content should produce different checksum
        let different_checksum = storage.compute_checksum("different content");
        assert_ne!(checksum1, different_checksum);

        // Checksum should be 64 hex characters (SHA-256)
        assert_eq!(checksum1.len(), 64);
    }

    #[tokio::test]
    async fn test_get_profile_path() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FsProfileStorage::new(temp_dir.path());
        let user_id = test_user_id();

        let path = storage.get_profile_path(&user_id);
        assert!(path.to_str().unwrap().contains("test-user@example.com"));
        assert!(path.to_str().unwrap().ends_with("profile.md"));
    }

    #[tokio::test]
    async fn test_atomic_write() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FsProfileStorage::new(temp_dir.path());
        let user_id = test_user_id();

        // Write initial content
        storage.write(&user_id, "version 1").await.unwrap();

        // Write new content (should be atomic)
        storage.write(&user_id, "version 2").await.unwrap();

        // Read should get latest version
        let content = storage.read(&user_id).await.unwrap();
        assert_eq!(content, "version 2");
    }
}
