//! PostgreSQL implementation of DecisionDocumentRepository.
//!
//! This adapter coordinates between PostgreSQL (metadata) and filesystem (content)
//! to provide a unified persistence layer for decision documents.

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::cycle::{
    DecisionDocument, DocumentVersion, MarkdownContent, SyncSource, UpdatedBy,
};
use crate::domain::foundation::{
    ComponentType, CycleId, DecisionDocumentId, DomainError, ErrorCode, Timestamp, UserId,
};
use crate::ports::{
    DecisionDocumentRepository, DocumentFileStorage, IntegrityStatus, StorageError, SyncResult,
};

/// PostgreSQL implementation of the DecisionDocumentRepository port.
///
/// This adapter coordinates writes between the database (metadata/checksums)
/// and the filesystem (actual markdown content). It follows a "file-first"
/// strategy: write the file first, then update the database. This ensures
/// orphaned files can be cleaned up, while the database remains consistent.
///
/// # Dependencies
///
/// - `PgPool` - PostgreSQL connection pool
/// - `DocumentFileStorage` - Filesystem storage adapter
///
/// # Usage
///
/// ```rust,ignore
/// let pool = PgPool::connect("postgres://...").await?;
/// let file_storage = Arc::new(LocalDocumentFileStorage::new("/data/documents"));
/// let repo = PostgresDocumentRepository::new(pool, file_storage);
///
/// // Save a new document
/// repo.save(&document, "# My Decision").await?;
/// ```
#[derive(Clone)]
pub struct PostgresDocumentRepository {
    pool: PgPool,
    file_storage: Arc<dyn DocumentFileStorage>,
}

impl std::fmt::Debug for PostgresDocumentRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresDocumentRepository")
            .field("pool", &"PgPool")
            .field("file_storage", &"Arc<dyn DocumentFileStorage>")
            .finish()
    }
}

impl PostgresDocumentRepository {
    /// Creates a new repository instance.
    pub fn new(pool: PgPool, file_storage: Arc<dyn DocumentFileStorage>) -> Self {
        Self { pool, file_storage }
    }

    /// Maps a database row to a DecisionDocument entity.
    fn row_to_document(&self, row: &DocumentRow) -> Result<DecisionDocument, DomainError> {
        let id = DecisionDocumentId::from_uuid(row.id);
        let cycle_id = CycleId::from_uuid(row.cycle_id);
        let user_id = UserId::new(&row.user_id).map_err(|e| {
            DomainError::new(ErrorCode::InvalidFormat, format!("Invalid user_id: {}", e))
        })?;

        let content = MarkdownContent::new(""); // Content loaded separately from file
        let version = DocumentVersion::from_raw(row.version as u32);

        let last_sync_source: SyncSource = row.last_sync_source.parse().map_err(|e: String| {
            DomainError::new(ErrorCode::InvalidFormat, format!("Invalid sync_source: {}", e))
        })?;

        let last_synced_at = Timestamp::from_datetime(row.last_synced_at);

        let parent_document_id = row.parent_document_id.map(DecisionDocumentId::from_uuid);

        let branch_point = row
            .branch_point
            .as_ref()
            .map(|bp| parse_branch_point(bp))
            .transpose()?;

        let created_at = Timestamp::from_datetime(row.created_at);
        let updated_at = Timestamp::from_datetime(row.updated_at);

        let updated_by = parse_updated_by(&row.updated_by_type, row.updated_by_id.as_deref())?;

        Ok(DecisionDocument::reconstitute(
            id,
            cycle_id,
            user_id,
            content,
            row.file_path.clone(),
            version,
            last_sync_source,
            last_synced_at,
            parent_document_id,
            branch_point,
            row.branch_label.clone(),
            created_at,
            updated_at,
            updated_by,
        ))
    }
}

/// Internal row type for sqlx query mapping.
#[derive(Debug, sqlx::FromRow)]
struct DocumentRow {
    id: uuid::Uuid,
    cycle_id: uuid::Uuid,
    user_id: String,
    file_path: String,
    content_checksum: String,
    file_size_bytes: i32,
    version: i32,
    last_sync_source: String,
    last_synced_at: chrono::DateTime<chrono::Utc>,
    parent_document_id: Option<uuid::Uuid>,
    branch_point: Option<String>,
    branch_label: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    updated_by_type: String,
    updated_by_id: Option<String>,
}

/// Parses a branch point string to ComponentType.
fn parse_branch_point(bp: &str) -> Result<ComponentType, DomainError> {
    match bp {
        "P" => Ok(ComponentType::ProblemFrame),
        "r" => Ok(ComponentType::IssueRaising),
        "O" => Ok(ComponentType::Objectives),
        "A" => Ok(ComponentType::Alternatives),
        "C" => Ok(ComponentType::Consequences),
        "T" => Ok(ComponentType::Tradeoffs),
        _ => Err(DomainError::new(
            ErrorCode::InvalidFormat,
            format!("Invalid branch_point: {}", bp),
        )),
    }
}

/// Converts ComponentType to database branch point string.
fn component_to_branch_point(ct: &ComponentType) -> &'static str {
    match ct {
        ComponentType::ProblemFrame => "P",
        ComponentType::IssueRaising => "r",
        ComponentType::Objectives => "O",
        ComponentType::Alternatives => "A",
        ComponentType::Consequences => "C",
        ComponentType::Tradeoffs => "T",
        ComponentType::Recommendation => "R",
        ComponentType::DecisionQuality => "D",
        ComponentType::NotesNextSteps => "N",
    }
}

/// Parses UpdatedBy from database columns.
fn parse_updated_by(
    updated_by_type: &str,
    updated_by_id: Option<&str>,
) -> Result<UpdatedBy, DomainError> {
    match updated_by_type {
        "system" => Ok(UpdatedBy::System),
        "agent" => Ok(UpdatedBy::Agent),
        "user" => {
            let user_id_str = updated_by_id.ok_or_else(|| {
                DomainError::new(
                    ErrorCode::InvalidFormat,
                    "User update requires updated_by_id".to_string(),
                )
            })?;
            let user_id = UserId::new(user_id_str).map_err(|e| {
                DomainError::new(ErrorCode::InvalidFormat, format!("Invalid updated_by_id: {}", e))
            })?;
            Ok(UpdatedBy::User { user_id })
        }
        _ => Err(DomainError::new(
            ErrorCode::InvalidFormat,
            format!("Invalid updated_by_type: {}", updated_by_type),
        )),
    }
}

/// Maps StorageError to DomainError.
fn storage_to_domain_error(err: StorageError) -> DomainError {
    match err {
        StorageError::NotFound { path } => {
            DomainError::new(ErrorCode::NotFound, format!("File not found: {}", path))
        }
        StorageError::PermissionDenied { path } => DomainError::new(
            ErrorCode::Forbidden,
            format!("Permission denied: {}", path),
        ),
        StorageError::Io { message } => {
            DomainError::new(ErrorCode::InternalError, format!("IO error: {}", message))
        }
        StorageError::UserDirectoryNotFound { user_id } => DomainError::new(
            ErrorCode::NotFound,
            format!("User directory not found: {}", user_id),
        ),
        StorageError::InvalidDocumentId { filename } => DomainError::new(
            ErrorCode::InvalidFormat,
            format!("Invalid document ID in filename: {}", filename),
        ),
        StorageError::FileTooLarge {
            size_bytes,
            max_bytes,
        } => DomainError::new(
            ErrorCode::ValidationFailed,
            format!(
                "File too large: {} bytes (max: {} bytes)",
                size_bytes, max_bytes
            ),
        ),
    }
}

#[async_trait]
impl DecisionDocumentRepository for PostgresDocumentRepository {
    async fn save(&self, document: &DecisionDocument, content: &str) -> Result<(), DomainError> {
        // Strategy: Write file first, then DB record.
        // Orphaned files can be cleaned up; orphaned DB records cause integrity issues.

        // 1. Write content to filesystem
        self.file_storage
            .write(document.user_id(), document.id(), content)
            .await
            .map_err(storage_to_domain_error)?;

        // 2. Insert database record
        let result = sqlx::query(
            r#"
            INSERT INTO decision_documents (
                id, cycle_id, user_id, file_path, content_checksum, file_size_bytes,
                version, last_sync_source, last_synced_at,
                parent_document_id, branch_point, branch_label,
                created_at, updated_at, updated_by_type, updated_by_id
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9,
                $10, $11, $12,
                $13, $14, $15, $16
            )
            "#,
        )
        .bind(document.id().as_uuid())
        .bind(document.cycle_id().as_uuid())
        .bind(document.user_id().as_str())
        .bind(document.file_path())
        .bind(document.content_checksum())
        .bind(content.len() as i32)
        .bind(document.version().as_u32() as i32)
        .bind(document.last_sync_source().as_str())
        .bind(document.last_synced_at().as_datetime())
        .bind(document.parent_document_id().map(|id| *id.as_uuid()))
        .bind(document.branch_point().map(|ct| component_to_branch_point(&ct)))
        .bind(document.branch_label())
        .bind(document.created_at().as_datetime())
        .bind(document.updated_at().as_datetime())
        .bind(document.updated_by().type_str())
        .bind(document.updated_by().user_id().map(|u| u.as_str().to_string()))
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                // Clean up orphaned file
                let _ = self
                    .file_storage
                    .delete(document.user_id(), document.id())
                    .await;
                Err(DomainError::new(
                    ErrorCode::ConcurrencyConflict,
                    format!("Document already exists for cycle {}", document.cycle_id()),
                ))
            }
            Err(e) => {
                // Clean up orphaned file
                let _ = self
                    .file_storage
                    .delete(document.user_id(), document.id())
                    .await;
                Err(DomainError::new(
                    ErrorCode::InternalError,
                    format!("Database error: {}", e),
                ))
            }
        }
    }

    async fn update(&self, document: &DecisionDocument, content: &str) -> Result<(), DomainError> {
        // Strategy: Update file first, then DB.
        // Use optimistic locking on version.

        // 1. Update filesystem
        self.file_storage
            .write(document.user_id(), document.id(), content)
            .await
            .map_err(storage_to_domain_error)?;

        // 2. Update database with version check
        let result = sqlx::query(
            r#"
            UPDATE decision_documents
            SET
                content_checksum = $1,
                file_size_bytes = $2,
                version = $3,
                last_sync_source = $4,
                last_synced_at = $5,
                updated_at = $6,
                updated_by_type = $7,
                updated_by_id = $8
            WHERE id = $9 AND version = $10
            "#,
        )
        .bind(document.content_checksum())
        .bind(content.len() as i32)
        .bind(document.version().as_u32() as i32)
        .bind(document.last_sync_source().as_str())
        .bind(document.last_synced_at().as_datetime())
        .bind(document.updated_at().as_datetime())
        .bind(document.updated_by().type_str())
        .bind(document.updated_by().user_id().map(|u| u.as_str().to_string()))
        .bind(document.id().as_uuid())
        .bind((document.version().as_u32() - 1) as i32) // Expect previous version
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            // Check if document exists at all
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM decision_documents WHERE id = $1)",
            )
            .bind(document.id().as_uuid())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e))
            })?;

            if exists {
                return Err(DomainError::new(
                    ErrorCode::ConcurrencyConflict,
                    "Document version mismatch - another update occurred".to_string(),
                ));
            } else {
                return Err(DomainError::new(
                    ErrorCode::NotFound,
                    format!("Document {} not found", document.id()),
                ));
            }
        }

        Ok(())
    }

    async fn find_by_id(
        &self,
        id: DecisionDocumentId,
    ) -> Result<Option<DecisionDocument>, DomainError> {
        let row = sqlx::query_as::<_, DocumentRow>(
            r#"
            SELECT
                id, cycle_id, user_id, file_path, content_checksum, file_size_bytes,
                version, last_sync_source, last_synced_at,
                parent_document_id, branch_point, branch_label,
                created_at, updated_at, updated_by_type, updated_by_id
            FROM decision_documents
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        match row {
            Some(r) => Ok(Some(self.row_to_document(&r)?)),
            None => Ok(None),
        }
    }

    async fn find_by_cycle(&self, cycle_id: CycleId) -> Result<Option<DecisionDocument>, DomainError> {
        let row = sqlx::query_as::<_, DocumentRow>(
            r#"
            SELECT
                id, cycle_id, user_id, file_path, content_checksum, file_size_bytes,
                version, last_sync_source, last_synced_at,
                parent_document_id, branch_point, branch_label,
                created_at, updated_at, updated_by_type, updated_by_id
            FROM decision_documents
            WHERE cycle_id = $1
            "#,
        )
        .bind(cycle_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        match row {
            Some(r) => Ok(Some(self.row_to_document(&r)?)),
            None => Ok(None),
        }
    }

    async fn sync_from_file(&self, document_id: DecisionDocumentId) -> Result<SyncResult, DomainError> {
        // 1. Get current document from DB
        let document = self
            .find_by_id(document_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::NotFound, "Document not found"))?;

        // 2. Read current file checksum
        let file_checksum = self
            .file_storage
            .checksum(document.user_id(), document_id)
            .await
            .map_err(storage_to_domain_error)?;

        // 3. Compare checksums
        if file_checksum == document.content_checksum() {
            return Ok(SyncResult::unchanged(
                file_checksum,
                document.version().as_u32(),
            ));
        }

        // 4. File changed - update database
        let new_version = document.version().increment();
        let metadata = self
            .file_storage
            .metadata(document.user_id(), document_id)
            .await
            .map_err(storage_to_domain_error)?;

        sqlx::query(
            r#"
            UPDATE decision_documents
            SET
                content_checksum = $1,
                file_size_bytes = $2,
                version = $3,
                last_sync_source = 'file_sync',
                last_synced_at = NOW(),
                updated_at = NOW()
            WHERE id = $4
            "#,
        )
        .bind(&file_checksum)
        .bind(metadata.size_bytes as i32)
        .bind(new_version.as_u32() as i32)
        .bind(document_id.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        Ok(SyncResult::changed(file_checksum, new_version.as_u32()))
    }

    async fn verify_integrity(
        &self,
        document_id: DecisionDocumentId,
    ) -> Result<IntegrityStatus, DomainError> {
        // 1. Get document from DB
        let document = match self.find_by_id(document_id).await? {
            Some(doc) => doc,
            None => return Ok(IntegrityStatus::DbRecordMissing),
        };

        // 2. Check if file exists
        let file_exists = self
            .file_storage
            .exists(document.user_id(), document_id)
            .await
            .map_err(storage_to_domain_error)?;

        if !file_exists {
            return Ok(IntegrityStatus::FileMissing);
        }

        // 3. Compare checksums
        let file_checksum = self
            .file_storage
            .checksum(document.user_id(), document_id)
            .await
            .map_err(storage_to_domain_error)?;

        if file_checksum == document.content_checksum() {
            Ok(IntegrityStatus::InSync)
        } else {
            Ok(IntegrityStatus::FileModified {
                file_checksum,
                db_checksum: document.content_checksum().to_string(),
            })
        }
    }

    async fn delete(&self, document_id: DecisionDocumentId) -> Result<(), DomainError> {
        // 1. Get document first (need user_id for file deletion)
        let document = self
            .find_by_id(document_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::NotFound, "Document not found"))?;

        // 2. Delete from database first (authoritative)
        let result = sqlx::query("DELETE FROM decision_documents WHERE id = $1")
            .bind(document_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::NotFound,
                format!("Document {} not found", document_id),
            ));
        }

        // 3. Delete file (best effort - orphaned files can be cleaned up later)
        let _ = self
            .file_storage
            .delete(document.user_id(), document_id)
            .await;

        Ok(())
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ───────────────────────────────────────────────────────────────
    // Helper function tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn parse_branch_point_valid() {
        assert_eq!(parse_branch_point("P").unwrap(), ComponentType::ProblemFrame);
        assert_eq!(parse_branch_point("r").unwrap(), ComponentType::IssueRaising);
        assert_eq!(parse_branch_point("O").unwrap(), ComponentType::Objectives);
        assert_eq!(parse_branch_point("A").unwrap(), ComponentType::Alternatives);
        assert_eq!(parse_branch_point("C").unwrap(), ComponentType::Consequences);
        assert_eq!(parse_branch_point("T").unwrap(), ComponentType::Tradeoffs);
    }

    #[test]
    fn parse_branch_point_invalid() {
        assert!(parse_branch_point("X").is_err());
        assert!(parse_branch_point("").is_err());
    }

    #[test]
    fn component_to_branch_point_roundtrip() {
        let types = [
            ComponentType::ProblemFrame,
            ComponentType::IssueRaising,
            ComponentType::Objectives,
            ComponentType::Alternatives,
            ComponentType::Consequences,
            ComponentType::Tradeoffs,
        ];

        for ct in types {
            let bp = component_to_branch_point(&ct);
            let parsed = parse_branch_point(bp).unwrap();
            assert_eq!(parsed, ct);
        }
    }

    #[test]
    fn parse_updated_by_system() {
        let result = parse_updated_by("system", None).unwrap();
        assert!(matches!(result, UpdatedBy::System));
    }

    #[test]
    fn parse_updated_by_agent() {
        let result = parse_updated_by("agent", None).unwrap();
        assert!(matches!(result, UpdatedBy::Agent));
    }

    #[test]
    fn parse_updated_by_user() {
        let result = parse_updated_by("user", Some("user-123")).unwrap();
        match result {
            UpdatedBy::User { user_id } => assert_eq!(user_id.as_str(), "user-123"),
            _ => panic!("Expected User variant"),
        }
    }

    #[test]
    fn parse_updated_by_user_without_id_fails() {
        let result = parse_updated_by("user", None);
        assert!(result.is_err());
    }

    #[test]
    fn parse_updated_by_invalid_type_fails() {
        let result = parse_updated_by("invalid", None);
        assert!(result.is_err());
    }

    // ───────────────────────────────────────────────────────────────
    // StorageError mapping tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn storage_error_maps_to_domain_error() {
        let not_found = storage_to_domain_error(StorageError::not_found("/path"));
        assert_eq!(not_found.code, ErrorCode::NotFound);

        let permission = storage_to_domain_error(StorageError::permission_denied("/path"));
        assert_eq!(permission.code, ErrorCode::Forbidden);

        let io = storage_to_domain_error(StorageError::io("disk full"));
        assert_eq!(io.code, ErrorCode::InternalError);

        let file_large = storage_to_domain_error(StorageError::file_too_large(100, 50));
        assert_eq!(file_large.code, ErrorCode::ValidationFailed);
    }
}
