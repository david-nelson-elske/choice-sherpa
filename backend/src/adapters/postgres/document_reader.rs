//! PostgreSQL implementation of DecisionDocumentReader.
//!
//! This adapter provides read-optimized queries for decision documents,
//! combining database metadata with filesystem content as needed.

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::cycle::{SyncSource, UpdatedBy};
use crate::domain::foundation::{
    ComponentType, CycleId, DecisionDocumentId, DomainError, ErrorCode, SessionId, Timestamp,
    UserId,
};
use crate::ports::{
    ComponentStatus, DecisionDocumentReader, DocumentFileStorage, DocumentListOptions,
    DocumentSearchResult, DocumentSummary, DocumentTree, DocumentTreeNode, DocumentVersionInfo,
    DocumentView, OrderBy, PrOACTStatus, StorageError,
};

/// PostgreSQL implementation of the DecisionDocumentReader port.
///
/// This adapter provides efficient read operations by:
/// - Querying PostgreSQL for metadata and indexes
/// - Reading content from filesystem via DocumentFileStorage
/// - Using database features like full-text search and JSONB queries
///
/// # Dependencies
///
/// - `PgPool` - PostgreSQL connection pool
/// - `DocumentFileStorage` - Filesystem storage adapter
#[derive(Clone)]
pub struct PostgresDocumentReader {
    pool: PgPool,
    file_storage: Arc<dyn DocumentFileStorage>,
}

impl std::fmt::Debug for PostgresDocumentReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresDocumentReader")
            .field("pool", &"PgPool")
            .field("file_storage", &"Arc<dyn DocumentFileStorage>")
            .finish()
    }
}

impl PostgresDocumentReader {
    /// Creates a new reader instance.
    pub fn new(pool: PgPool, file_storage: Arc<dyn DocumentFileStorage>) -> Self {
        Self { pool, file_storage }
    }
}

/// Database row for document queries.
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
    proact_status: serde_json::Value,
    overall_progress: i32,
    dq_score: Option<i32>,
    parent_document_id: Option<uuid::Uuid>,
    branch_point: Option<String>,
    branch_label: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    updated_by_type: String,
    updated_by_id: Option<String>,
}

/// Database row for document summary.
#[derive(Debug, sqlx::FromRow)]
struct SummaryRow {
    id: uuid::Uuid,
    cycle_id: uuid::Uuid,
    version: i32,
    overall_progress: i32,
    dq_score: Option<i32>,
    updated_at: chrono::DateTime<chrono::Utc>,
    file_size_bytes: i32,
    extracted_json: Option<serde_json::Value>,
}

/// Database row for version history.
#[derive(Debug, sqlx::FromRow)]
struct VersionRow {
    version: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    sync_source: String,
    updated_by_type: String,
    updated_by_id: Option<String>,
    content_checksum: String,
    proact_status: serde_json::Value,
    change_summary: Option<String>,
}

/// Database row for tree building.
#[derive(Debug, sqlx::FromRow)]
struct TreeRow {
    id: uuid::Uuid,
    cycle_id: uuid::Uuid,
    parent_document_id: Option<uuid::Uuid>,
    branch_point: Option<String>,
    branch_label: Option<String>,
    proact_status: serde_json::Value,
}

/// Parse SyncSource from string.
fn parse_sync_source(s: &str) -> SyncSource {
    s.parse().unwrap_or(SyncSource::Initial)
}

/// Parse UpdatedBy from database columns.
fn parse_updated_by(updated_by_type: &str, updated_by_id: Option<&str>) -> UpdatedBy {
    match updated_by_type {
        "user" => {
            if let Some(id) = updated_by_id {
                if let Ok(user_id) = UserId::new(id) {
                    return UpdatedBy::User { user_id };
                }
            }
            UpdatedBy::System
        }
        "agent" => UpdatedBy::Agent,
        _ => UpdatedBy::System,
    }
}

/// Parse PrOACT status from JSONB.
fn parse_proact_status(json: &serde_json::Value) -> PrOACTStatus {
    fn parse_component(val: Option<&serde_json::Value>) -> ComponentStatus {
        match val.and_then(|v| v.as_str()) {
            Some("completed") => ComponentStatus::Completed,
            Some("in_progress") => ComponentStatus::InProgress,
            Some("needs_revision") => ComponentStatus::NeedsRevision,
            _ => ComponentStatus::NotStarted,
        }
    }

    PrOACTStatus {
        issue_raising: parse_component(json.get("r")),
        problem_frame: parse_component(json.get("P")),
        objectives: parse_component(json.get("O")),
        alternatives: parse_component(json.get("A")),
        consequences: parse_component(json.get("C")),
        tradeoffs: parse_component(json.get("T")),
        recommendation: parse_component(json.get("R")),
        decision_quality: parse_component(json.get("D")),
    }
}

/// Parse branch point string to ComponentType.
fn parse_branch_point(bp: &str) -> Option<ComponentType> {
    match bp {
        "P" => Some(ComponentType::ProblemFrame),
        "r" => Some(ComponentType::IssueRaising),
        "O" => Some(ComponentType::Objectives),
        "A" => Some(ComponentType::Alternatives),
        "C" => Some(ComponentType::Consequences),
        "T" => Some(ComponentType::Tradeoffs),
        "R" => Some(ComponentType::Recommendation),
        "D" => Some(ComponentType::DecisionQuality),
        "N" => Some(ComponentType::NotesNextSteps),
        _ => None,
    }
}

/// Map StorageError to DomainError.
fn storage_to_domain_error(err: StorageError) -> DomainError {
    match err {
        StorageError::NotFound { path } => {
            DomainError::new(ErrorCode::NotFound, format!("File not found: {}", path))
        }
        _ => DomainError::new(ErrorCode::InternalError, format!("Storage error: {}", err)),
    }
}

#[async_trait]
impl DecisionDocumentReader for PostgresDocumentReader {
    async fn get_by_cycle(&self, cycle_id: CycleId) -> Result<Option<DocumentView>, DomainError> {
        let row = sqlx::query_as::<_, DocumentRow>(
            r#"
            SELECT
                id, cycle_id, user_id, file_path, content_checksum, file_size_bytes,
                version, last_sync_source, last_synced_at,
                proact_status, overall_progress, dq_score,
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

        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };

        // Load content from filesystem
        let user_id = UserId::new(&row.user_id).map_err(|e| {
            DomainError::new(ErrorCode::InvalidFormat, format!("Invalid user_id: {}", e))
        })?;
        let document_id = DecisionDocumentId::from_uuid(row.id);

        let content = self
            .file_storage
            .read(&user_id, document_id)
            .await
            .map_err(storage_to_domain_error)?;

        Ok(Some(DocumentView {
            id: document_id,
            cycle_id: CycleId::from_uuid(row.cycle_id),
            user_id,
            file_path: row.file_path,
            content,
            version: row.version as u32,
            proact_status: parse_proact_status(&row.proact_status),
            overall_progress: row.overall_progress as u8,
            dq_score: row.dq_score.map(|s| s as u8),
            last_sync_source: parse_sync_source(&row.last_sync_source),
            updated_at: Timestamp::from_datetime(row.updated_at),
            updated_by: parse_updated_by(&row.updated_by_type, row.updated_by_id.as_deref()),
            parent_document_id: row.parent_document_id.map(DecisionDocumentId::from_uuid),
            branch_point: row.branch_point.as_ref().and_then(|bp| parse_branch_point(bp)),
            branch_label: row.branch_label,
            created_at: Timestamp::from_datetime(row.created_at),
        }))
    }

    async fn get_by_id(
        &self,
        id: DecisionDocumentId,
    ) -> Result<Option<DocumentView>, DomainError> {
        let row = sqlx::query_as::<_, DocumentRow>(
            r#"
            SELECT
                id, cycle_id, user_id, file_path, content_checksum, file_size_bytes,
                version, last_sync_source, last_synced_at,
                proact_status, overall_progress, dq_score,
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

        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };

        // Load content from filesystem
        let user_id = UserId::new(&row.user_id).map_err(|e| {
            DomainError::new(ErrorCode::InvalidFormat, format!("Invalid user_id: {}", e))
        })?;

        let content = self
            .file_storage
            .read(&user_id, id)
            .await
            .map_err(storage_to_domain_error)?;

        Ok(Some(DocumentView {
            id,
            cycle_id: CycleId::from_uuid(row.cycle_id),
            user_id,
            file_path: row.file_path,
            content,
            version: row.version as u32,
            proact_status: parse_proact_status(&row.proact_status),
            overall_progress: row.overall_progress as u8,
            dq_score: row.dq_score.map(|s| s as u8),
            last_sync_source: parse_sync_source(&row.last_sync_source),
            updated_at: Timestamp::from_datetime(row.updated_at),
            updated_by: parse_updated_by(&row.updated_by_type, row.updated_by_id.as_deref()),
            parent_document_id: row.parent_document_id.map(DecisionDocumentId::from_uuid),
            branch_point: row.branch_point.as_ref().and_then(|bp| parse_branch_point(bp)),
            branch_label: row.branch_label,
            created_at: Timestamp::from_datetime(row.created_at),
        }))
    }

    async fn get_content(&self, cycle_id: CycleId) -> Result<Option<String>, DomainError> {
        // Get minimal info needed to read file
        let row = sqlx::query_as::<_, (uuid::Uuid, String)>(
            "SELECT id, user_id FROM decision_documents WHERE cycle_id = $1",
        )
        .bind(cycle_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        let (id, user_id_str) = match row {
            Some(r) => r,
            None => return Ok(None),
        };

        let user_id = UserId::new(&user_id_str).map_err(|e| {
            DomainError::new(ErrorCode::InvalidFormat, format!("Invalid user_id: {}", e))
        })?;
        let document_id = DecisionDocumentId::from_uuid(id);

        let content = self
            .file_storage
            .read(&user_id, document_id)
            .await
            .map_err(storage_to_domain_error)?;

        Ok(Some(content))
    }

    async fn get_version_history(
        &self,
        cycle_id: CycleId,
        limit: i32,
    ) -> Result<Vec<DocumentVersionInfo>, DomainError> {
        let rows = sqlx::query_as::<_, VersionRow>(
            r#"
            SELECT
                v.version, v.created_at, v.sync_source,
                v.updated_by_type, v.updated_by_id,
                v.content_checksum, v.proact_status, v.change_summary
            FROM decision_document_versions v
            JOIN decision_documents d ON d.id = v.document_id
            WHERE d.cycle_id = $1
            ORDER BY v.version DESC
            LIMIT $2
            "#,
        )
        .bind(cycle_id.as_uuid())
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|row| DocumentVersionInfo {
                version: row.version as u32,
                updated_at: Timestamp::from_datetime(row.created_at),
                updated_by: parse_updated_by(&row.updated_by_type, row.updated_by_id.as_deref()),
                sync_source: parse_sync_source(&row.sync_source),
                checksum: row.content_checksum,
                proact_status: parse_proact_status(&row.proact_status),
                change_summary: row.change_summary,
            })
            .collect())
    }

    async fn search(
        &self,
        user_id: &UserId,
        query: &str,
    ) -> Result<Vec<DocumentSearchResult>, DomainError> {
        // Use PostgreSQL full-text search
        let rows = sqlx::query_as::<_, (uuid::Uuid, uuid::Uuid, serde_json::Value, f32)>(
            r#"
            SELECT
                id, cycle_id, extracted_json,
                ts_rank(
                    to_tsvector('english', COALESCE(extracted_json->>'title', '') || ' ' ||
                                           COALESCE(extracted_json->>'focal_decision', '')),
                    plainto_tsquery('english', $2)
                ) as rank
            FROM decision_documents
            WHERE user_id = $1
              AND to_tsvector('english', COALESCE(extracted_json->>'title', '') || ' ' ||
                                         COALESCE(extracted_json->>'focal_decision', ''))
                  @@ plainto_tsquery('english', $2)
            ORDER BY rank DESC
            LIMIT 20
            "#,
        )
        .bind(user_id.as_str())
        .bind(query)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|(id, cycle_id, extracted_json, rank)| {
                let title = extracted_json
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled")
                    .to_string();
                let focal_decision = extracted_json
                    .get("focal_decision")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                DocumentSearchResult {
                    document_id: DecisionDocumentId::from_uuid(id),
                    cycle_id: CycleId::from_uuid(cycle_id),
                    title,
                    snippet: focal_decision,
                    relevance: rank,
                }
            })
            .collect())
    }

    async fn get_document_tree(
        &self,
        session_id: SessionId,
    ) -> Result<DocumentTree, DomainError> {
        // Get all documents for this session's cycles
        // Note: This assumes there's a join path from session to cycles to documents
        // For now, we'll return an empty tree as the actual join depends on session/cycle tables
        let rows = sqlx::query_as::<_, TreeRow>(
            r#"
            SELECT
                d.id, d.cycle_id, d.parent_document_id,
                d.branch_point, d.branch_label, d.proact_status
            FROM decision_documents d
            JOIN cycles c ON c.id = d.cycle_id
            WHERE c.session_id = $1
            ORDER BY d.created_at ASC
            "#,
        )
        .bind(session_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default(); // Return empty if table doesn't exist yet

        // Build tree from flat list
        let mut tree = DocumentTree::empty(session_id);

        if rows.is_empty() {
            return Ok(tree);
        }

        // Group by parent relationship
        let mut nodes: std::collections::HashMap<uuid::Uuid, DocumentTreeNode> =
            std::collections::HashMap::new();
        let mut children: std::collections::HashMap<uuid::Uuid, Vec<uuid::Uuid>> =
            std::collections::HashMap::new();
        let mut roots: Vec<uuid::Uuid> = Vec::new();

        for row in &rows {
            let node = DocumentTreeNode {
                document_id: DecisionDocumentId::from_uuid(row.id),
                cycle_id: CycleId::from_uuid(row.cycle_id),
                label: row
                    .branch_label
                    .clone()
                    .unwrap_or_else(|| "Main".to_string()),
                proact_status: parse_proact_status(&row.proact_status),
                branch_point: row
                    .branch_point
                    .as_ref()
                    .and_then(|bp| parse_branch_point(bp)),
                children: Vec::new(),
            };

            nodes.insert(row.id, node);

            match row.parent_document_id {
                Some(parent_id) => {
                    children.entry(parent_id).or_default().push(row.id);
                }
                None => {
                    roots.push(row.id);
                }
            }
        }

        // Build tree recursively
        fn build_node(
            id: uuid::Uuid,
            nodes: &mut std::collections::HashMap<uuid::Uuid, DocumentTreeNode>,
            children: &std::collections::HashMap<uuid::Uuid, Vec<uuid::Uuid>>,
        ) -> Option<DocumentTreeNode> {
            let mut node = nodes.remove(&id)?;
            if let Some(child_ids) = children.get(&id) {
                for child_id in child_ids {
                    if let Some(child_node) = build_node(*child_id, nodes, children) {
                        node.children.push(child_node);
                    }
                }
            }
            Some(node)
        }

        for root_id in roots {
            if let Some(root_node) = build_node(root_id, &mut nodes, &children) {
                tree.documents.push(root_node);
            }
        }

        Ok(tree)
    }

    async fn get_summary(&self, cycle_id: CycleId) -> Result<Option<DocumentSummary>, DomainError> {
        let row = sqlx::query_as::<_, SummaryRow>(
            r#"
            SELECT
                id, cycle_id, version, overall_progress, dq_score,
                updated_at, file_size_bytes, extracted_json
            FROM decision_documents
            WHERE cycle_id = $1
            "#,
        )
        .bind(cycle_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        Ok(row.map(|r| DocumentSummary {
            id: DecisionDocumentId::from_uuid(r.id),
            cycle_id: CycleId::from_uuid(r.cycle_id),
            title: r
                .extracted_json
                .as_ref()
                .and_then(|j| j.get("title"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            version: r.version as u32,
            overall_progress: r.overall_progress as u8,
            dq_score: r.dq_score.map(|s| s as u8),
            updated_at: Timestamp::from_datetime(r.updated_at),
            file_size_bytes: r.file_size_bytes,
        }))
    }

    async fn list_by_user(
        &self,
        user_id: &UserId,
        options: DocumentListOptions,
    ) -> Result<Vec<DocumentSummary>, DomainError> {
        let limit = options.limit.unwrap_or(50);
        let offset = options.offset.unwrap_or(0);

        let order_clause = match options.order_by.unwrap_or_default() {
            OrderBy::UpdatedAtDesc => "ORDER BY updated_at DESC",
            OrderBy::UpdatedAtAsc => "ORDER BY updated_at ASC",
            OrderBy::CreatedAtDesc => "ORDER BY created_at DESC",
            OrderBy::CreatedAtAsc => "ORDER BY created_at ASC",
        };

        // Build dynamic query with ordering
        let query = format!(
            r#"
            SELECT
                id, cycle_id, version, overall_progress, dq_score,
                updated_at, file_size_bytes, extracted_json
            FROM decision_documents
            WHERE user_id = $1
            {}
            LIMIT $2 OFFSET $3
            "#,
            order_clause
        );

        let rows = sqlx::query_as::<_, SummaryRow>(&query)
            .bind(user_id.as_str())
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e))
            })?;

        Ok(rows
            .into_iter()
            .map(|r| DocumentSummary {
                id: DecisionDocumentId::from_uuid(r.id),
                cycle_id: CycleId::from_uuid(r.cycle_id),
                title: r
                    .extracted_json
                    .as_ref()
                    .and_then(|j| j.get("title"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                version: r.version as u32,
                overall_progress: r.overall_progress as u8,
                dq_score: r.dq_score.map(|s| s as u8),
                updated_at: Timestamp::from_datetime(r.updated_at),
                file_size_bytes: r.file_size_bytes,
            })
            .collect())
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
    fn parse_sync_source_valid() {
        assert_eq!(parse_sync_source("initial"), SyncSource::Initial);
        assert_eq!(parse_sync_source("component_update"), SyncSource::ComponentUpdate);
        assert_eq!(parse_sync_source("user_edit"), SyncSource::UserEdit);
        assert_eq!(parse_sync_source("file_sync"), SyncSource::FileSync);
    }

    #[test]
    fn parse_sync_source_invalid_returns_default() {
        assert_eq!(parse_sync_source("invalid"), SyncSource::Initial);
    }

    #[test]
    fn parse_updated_by_system() {
        let result = parse_updated_by("system", None);
        assert!(matches!(result, UpdatedBy::System));
    }

    #[test]
    fn parse_updated_by_agent() {
        let result = parse_updated_by("agent", None);
        assert!(matches!(result, UpdatedBy::Agent));
    }

    #[test]
    fn parse_updated_by_user() {
        let result = parse_updated_by("user", Some("user-123"));
        match result {
            UpdatedBy::User { user_id } => assert_eq!(user_id.as_str(), "user-123"),
            _ => panic!("Expected User variant"),
        }
    }

    #[test]
    fn parse_updated_by_user_invalid_falls_back_to_system() {
        let result = parse_updated_by("user", None);
        assert!(matches!(result, UpdatedBy::System));
    }

    #[test]
    fn parse_proact_status_all_not_started() {
        let json = serde_json::json!({});
        let status = parse_proact_status(&json);
        assert_eq!(status.completed_count(), 0);
    }

    #[test]
    fn parse_proact_status_mixed() {
        let json = serde_json::json!({
            "P": "completed",
            "r": "in_progress",
            "O": "not_started",
            "A": "completed",
            "C": "needs_revision"
        });
        let status = parse_proact_status(&json);
        assert_eq!(status.problem_frame, ComponentStatus::Completed);
        assert_eq!(status.issue_raising, ComponentStatus::InProgress);
        assert_eq!(status.objectives, ComponentStatus::NotStarted);
        assert_eq!(status.alternatives, ComponentStatus::Completed);
        assert_eq!(status.consequences, ComponentStatus::NeedsRevision);
    }

    #[test]
    fn parse_branch_point_valid() {
        assert_eq!(parse_branch_point("P"), Some(ComponentType::ProblemFrame));
        assert_eq!(parse_branch_point("r"), Some(ComponentType::IssueRaising));
        assert_eq!(parse_branch_point("O"), Some(ComponentType::Objectives));
        assert_eq!(parse_branch_point("A"), Some(ComponentType::Alternatives));
        assert_eq!(parse_branch_point("C"), Some(ComponentType::Consequences));
        assert_eq!(parse_branch_point("T"), Some(ComponentType::Tradeoffs));
        assert_eq!(parse_branch_point("R"), Some(ComponentType::Recommendation));
        assert_eq!(parse_branch_point("D"), Some(ComponentType::DecisionQuality));
    }

    #[test]
    fn parse_branch_point_invalid() {
        assert_eq!(parse_branch_point("X"), None);
        assert_eq!(parse_branch_point(""), None);
    }
}
