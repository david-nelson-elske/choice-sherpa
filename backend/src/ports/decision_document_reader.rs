//! Decision Document Reader Port - Read-optimized document queries.
//!
//! This port defines the contract for querying decision document data
//! in various formats optimized for different use cases.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::cycle::{DocumentVersion, SyncSource, UpdatedBy};
use crate::domain::foundation::{
    ComponentType, CycleId, DecisionDocumentId, DomainError, SessionId, Timestamp, UserId,
};

/// Port for read operations on decision documents.
///
/// # Contract
///
/// Implementations must:
/// - Provide efficient read-only access to document data
/// - Support various view formats (full, summary, metadata-only)
/// - Enable search and navigation across documents
///
/// # Usage
///
/// ```rust,ignore
/// let reader: &dyn DecisionDocumentReader = get_reader();
///
/// // Get full document view
/// let view = reader.get_by_cycle(cycle_id).await?;
///
/// // Get just the content
/// let content = reader.get_content(cycle_id).await?;
///
/// // Search user's documents
/// let results = reader.search(&user_id, "career decision").await?;
/// ```
#[async_trait]
pub trait DecisionDocumentReader: Send + Sync {
    /// Get full document view (metadata + content).
    ///
    /// # Arguments
    ///
    /// * `cycle_id` - The cycle to get the document for
    ///
    /// # Returns
    ///
    /// The full document view if found, None otherwise.
    async fn get_by_cycle(&self, cycle_id: CycleId) -> Result<Option<DocumentView>, DomainError>;

    /// Get document by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The document's unique identifier
    ///
    /// # Returns
    ///
    /// The full document view if found, None otherwise.
    async fn get_by_id(
        &self,
        id: DecisionDocumentId,
    ) -> Result<Option<DocumentView>, DomainError>;

    /// Get content only (efficient for large documents).
    ///
    /// Reads directly from filesystem without loading full metadata.
    ///
    /// # Arguments
    ///
    /// * `cycle_id` - The cycle to get content for
    ///
    /// # Returns
    ///
    /// The markdown content if found, None otherwise.
    async fn get_content(&self, cycle_id: CycleId) -> Result<Option<String>, DomainError>;

    /// Get version history for a document.
    ///
    /// Returns metadata-only version entries from the database.
    ///
    /// # Arguments
    ///
    /// * `cycle_id` - The cycle to get history for
    /// * `limit` - Maximum number of versions to return
    ///
    /// # Returns
    ///
    /// List of version info entries, newest first.
    async fn get_version_history(
        &self,
        cycle_id: CycleId,
        limit: i32,
    ) -> Result<Vec<DocumentVersionInfo>, DomainError>;

    /// Search documents by content.
    ///
    /// Uses database full-text search for efficient querying.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user whose documents to search
    /// * `query` - The search query string
    ///
    /// # Returns
    ///
    /// Matching documents with relevance scores.
    async fn search(
        &self,
        user_id: &UserId,
        query: &str,
    ) -> Result<Vec<DocumentSearchResult>, DomainError>;

    /// Get document tree for a session.
    ///
    /// Returns hierarchical view of all documents in a session,
    /// organized by cycle branching relationships.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session to get documents for
    ///
    /// # Returns
    ///
    /// Tree structure of documents.
    async fn get_document_tree(&self, session_id: SessionId)
        -> Result<DocumentTree, DomainError>;

    /// Get document summary (metadata without content).
    ///
    /// Efficient for listing documents without loading full content.
    ///
    /// # Arguments
    ///
    /// * `cycle_id` - The cycle to get summary for
    ///
    /// # Returns
    ///
    /// Document summary if found, None otherwise.
    async fn get_summary(&self, cycle_id: CycleId)
        -> Result<Option<DocumentSummary>, DomainError>;

    /// List all documents for a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user to list documents for
    /// * `options` - Pagination and filter options
    ///
    /// # Returns
    ///
    /// List of document summaries.
    async fn list_by_user(
        &self,
        user_id: &UserId,
        options: DocumentListOptions,
    ) -> Result<Vec<DocumentSummary>, DomainError>;
}

/// Full document view including content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentView {
    /// Document ID.
    pub id: DecisionDocumentId,

    /// Associated cycle ID.
    pub cycle_id: CycleId,

    /// Owner user ID.
    pub user_id: UserId,

    /// Relative file path.
    pub file_path: String,

    /// Full markdown content.
    pub content: String,

    /// Current version number.
    pub version: u32,

    /// PrOACT component progress status.
    pub proact_status: PrOACTStatus,

    /// Overall completion percentage (0-100).
    pub overall_progress: u8,

    /// Decision quality score if assessed.
    pub dq_score: Option<u8>,

    /// What triggered the last sync.
    pub last_sync_source: SyncSource,

    /// When last updated.
    pub updated_at: Timestamp,

    /// Who last updated.
    pub updated_by: UpdatedBy,

    /// Parent document if this is a branch.
    pub parent_document_id: Option<DecisionDocumentId>,

    /// Component where branching occurred.
    pub branch_point: Option<ComponentType>,

    /// Label for this branch.
    pub branch_label: Option<String>,

    /// When document was created.
    pub created_at: Timestamp,
}

/// Document summary without content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSummary {
    /// Document ID.
    pub id: DecisionDocumentId,

    /// Associated cycle ID.
    pub cycle_id: CycleId,

    /// Document title (extracted from content).
    pub title: Option<String>,

    /// Current version number.
    pub version: u32,

    /// Overall completion percentage.
    pub overall_progress: u8,

    /// Decision quality score if assessed.
    pub dq_score: Option<u8>,

    /// When last updated.
    pub updated_at: Timestamp,

    /// File size in bytes.
    pub file_size_bytes: i32,
}

/// Version history entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentVersionInfo {
    /// Version number.
    pub version: u32,

    /// When this version was created.
    pub updated_at: Timestamp,

    /// Who made the change.
    pub updated_by: UpdatedBy,

    /// What triggered the update.
    pub sync_source: SyncSource,

    /// Content checksum at this version.
    pub checksum: String,

    /// PrOACT status at this version.
    pub proact_status: PrOACTStatus,

    /// Optional summary of changes.
    pub change_summary: Option<String>,
}

/// Search result entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSearchResult {
    /// Document ID.
    pub document_id: DecisionDocumentId,

    /// Associated cycle ID.
    pub cycle_id: CycleId,

    /// Document title.
    pub title: String,

    /// Content snippet with highlighted match.
    pub snippet: String,

    /// Relevance score (0.0 to 1.0).
    pub relevance: f32,
}

/// Document tree for session visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTree {
    /// Session this tree belongs to.
    pub session_id: SessionId,

    /// Root documents (non-branched).
    pub documents: Vec<DocumentTreeNode>,
}

impl DocumentTree {
    /// Creates an empty document tree.
    pub fn empty(session_id: SessionId) -> Self {
        Self {
            session_id,
            documents: Vec::new(),
        }
    }

    /// Returns the total number of documents in the tree.
    pub fn total_count(&self) -> usize {
        fn count_nodes(nodes: &[DocumentTreeNode]) -> usize {
            nodes
                .iter()
                .map(|n| 1 + count_nodes(&n.children))
                .sum()
        }
        count_nodes(&self.documents)
    }
}

/// Node in the document tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTreeNode {
    /// Document ID.
    pub document_id: DecisionDocumentId,

    /// Associated cycle ID.
    pub cycle_id: CycleId,

    /// Display label.
    pub label: String,

    /// PrOACT component progress.
    pub proact_status: PrOACTStatus,

    /// Branch point (if this is a branched document).
    pub branch_point: Option<ComponentType>,

    /// Child branches.
    pub children: Vec<DocumentTreeNode>,
}

impl DocumentTreeNode {
    /// Creates a new tree node.
    pub fn new(
        document_id: DecisionDocumentId,
        cycle_id: CycleId,
        label: impl Into<String>,
        proact_status: PrOACTStatus,
    ) -> Self {
        Self {
            document_id,
            cycle_id,
            label: label.into(),
            proact_status,
            branch_point: None,
            children: Vec::new(),
        }
    }

    /// Sets the branch point.
    pub fn with_branch_point(mut self, branch_point: ComponentType) -> Self {
        self.branch_point = Some(branch_point);
        self
    }

    /// Adds a child node.
    pub fn with_child(mut self, child: DocumentTreeNode) -> Self {
        self.children.push(child);
        self
    }
}

/// PrOACT component status for all 8 components.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrOACTStatus {
    /// Issue Raising status.
    pub issue_raising: ComponentStatus,
    /// Problem Frame status.
    pub problem_frame: ComponentStatus,
    /// Objectives status.
    pub objectives: ComponentStatus,
    /// Alternatives status.
    pub alternatives: ComponentStatus,
    /// Consequences status.
    pub consequences: ComponentStatus,
    /// Tradeoffs status.
    pub tradeoffs: ComponentStatus,
    /// Recommendation status.
    pub recommendation: ComponentStatus,
    /// Decision Quality status.
    pub decision_quality: ComponentStatus,
}

impl PrOACTStatus {
    /// Returns the number of completed components.
    pub fn completed_count(&self) -> u8 {
        let statuses = [
            &self.issue_raising,
            &self.problem_frame,
            &self.objectives,
            &self.alternatives,
            &self.consequences,
            &self.tradeoffs,
            &self.recommendation,
            &self.decision_quality,
        ];
        statuses
            .iter()
            .filter(|s| ***s == ComponentStatus::Completed)
            .count() as u8
    }

    /// Returns overall progress as a percentage.
    pub fn progress_percentage(&self) -> u8 {
        (self.completed_count() as u16 * 100 / 8) as u8
    }
}

/// Status of a single PrOACT component.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentStatus {
    /// Component hasn't been started.
    #[default]
    NotStarted,
    /// Component is in progress.
    InProgress,
    /// Component is completed.
    Completed,
    /// Component marked for revision.
    NeedsRevision,
}

/// Options for listing documents.
#[derive(Debug, Clone, Default)]
pub struct DocumentListOptions {
    /// Maximum number of results.
    pub limit: Option<i32>,
    /// Offset for pagination.
    pub offset: Option<i32>,
    /// Order by field.
    pub order_by: Option<OrderBy>,
}

impl DocumentListOptions {
    /// Creates default list options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the limit.
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the offset.
    pub fn offset(mut self, offset: i32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Sets the order.
    pub fn order_by(mut self, order: OrderBy) -> Self {
        self.order_by = Some(order);
        self
    }
}

/// Ordering options for document lists.
#[derive(Debug, Clone, Copy, Default)]
pub enum OrderBy {
    /// Order by update time (newest first).
    #[default]
    UpdatedAtDesc,
    /// Order by update time (oldest first).
    UpdatedAtAsc,
    /// Order by creation time (newest first).
    CreatedAtDesc,
    /// Order by creation time (oldest first).
    CreatedAtAsc,
}

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn test_session_id() -> SessionId {
        SessionId::new()
    }

    // ───────────────────────────────────────────────────────────────
    // DocumentTree tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn document_tree_empty() {
        let tree = DocumentTree::empty(test_session_id());
        assert_eq!(tree.total_count(), 0);
        assert!(tree.documents.is_empty());
    }

    #[test]
    fn document_tree_total_count_includes_children() {
        let session_id = test_session_id();
        let mut tree = DocumentTree::empty(session_id);

        // Add root with 2 children
        let child1 = DocumentTreeNode::new(
            DecisionDocumentId::new(),
            CycleId::new(),
            "Child 1",
            PrOACTStatus::default(),
        );
        let child2 = DocumentTreeNode::new(
            DecisionDocumentId::new(),
            CycleId::new(),
            "Child 2",
            PrOACTStatus::default(),
        );
        let root = DocumentTreeNode::new(
            DecisionDocumentId::new(),
            CycleId::new(),
            "Root",
            PrOACTStatus::default(),
        )
        .with_child(child1)
        .with_child(child2);

        tree.documents.push(root);

        assert_eq!(tree.total_count(), 3); // 1 root + 2 children
    }

    // ───────────────────────────────────────────────────────────────
    // DocumentTreeNode tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn tree_node_with_branch_point() {
        let node = DocumentTreeNode::new(
            DecisionDocumentId::new(),
            CycleId::new(),
            "Branched",
            PrOACTStatus::default(),
        )
        .with_branch_point(ComponentType::Alternatives);

        assert_eq!(node.branch_point, Some(ComponentType::Alternatives));
    }

    // ───────────────────────────────────────────────────────────────
    // PrOACTStatus tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn proact_status_default_all_not_started() {
        let status = PrOACTStatus::default();
        assert_eq!(status.completed_count(), 0);
        assert_eq!(status.progress_percentage(), 0);
    }

    #[test]
    fn proact_status_counts_completed() {
        let mut status = PrOACTStatus::default();
        status.issue_raising = ComponentStatus::Completed;
        status.problem_frame = ComponentStatus::Completed;
        status.objectives = ComponentStatus::InProgress;

        assert_eq!(status.completed_count(), 2);
        assert_eq!(status.progress_percentage(), 25); // 2/8 = 25%
    }

    #[test]
    fn proact_status_all_completed_is_100() {
        let status = PrOACTStatus {
            issue_raising: ComponentStatus::Completed,
            problem_frame: ComponentStatus::Completed,
            objectives: ComponentStatus::Completed,
            alternatives: ComponentStatus::Completed,
            consequences: ComponentStatus::Completed,
            tradeoffs: ComponentStatus::Completed,
            recommendation: ComponentStatus::Completed,
            decision_quality: ComponentStatus::Completed,
        };

        assert_eq!(status.completed_count(), 8);
        assert_eq!(status.progress_percentage(), 100);
    }

    // ───────────────────────────────────────────────────────────────
    // ComponentStatus tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn component_status_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&ComponentStatus::NotStarted).unwrap(),
            "\"not_started\""
        );
        assert_eq!(
            serde_json::to_string(&ComponentStatus::InProgress).unwrap(),
            "\"in_progress\""
        );
        assert_eq!(
            serde_json::to_string(&ComponentStatus::Completed).unwrap(),
            "\"completed\""
        );
        assert_eq!(
            serde_json::to_string(&ComponentStatus::NeedsRevision).unwrap(),
            "\"needs_revision\""
        );
    }

    // ───────────────────────────────────────────────────────────────
    // ListOptions tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn list_options_builder_pattern() {
        let opts = DocumentListOptions::new()
            .limit(10)
            .offset(20)
            .order_by(OrderBy::CreatedAtDesc);

        assert_eq!(opts.limit, Some(10));
        assert_eq!(opts.offset, Some(20));
        assert!(matches!(opts.order_by, Some(OrderBy::CreatedAtDesc)));
    }

    // ───────────────────────────────────────────────────────────────
    // Trait object safety test
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn decision_document_reader_is_object_safe() {
        fn check<T: DecisionDocumentReader + ?Sized>() {}
        // This compiles only if the trait is object-safe
        check::<dyn DecisionDocumentReader>();
    }
}
