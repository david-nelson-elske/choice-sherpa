//! UpdateDocumentFromEditHandler - Command handler for bidirectional document sync.
//!
//! Parses edited markdown content and updates both the document and
//! the cycle's component outputs, enabling bidirectional synchronization.

use std::sync::Arc;

use crate::domain::foundation::{
    CommandMetadata, ComponentType, CycleId, DecisionDocumentId, DomainError,
};
use crate::ports::{
    CycleRepository, DecisionDocumentRepository, DocumentError, DocumentParser, ParseResult,
};

/// Command to update a document from user edits.
#[derive(Debug, Clone)]
pub struct UpdateDocumentFromEditCommand {
    /// Document to update.
    pub document_id: DecisionDocumentId,
    /// Edited markdown content.
    pub content: String,
    /// Whether to update component outputs from parsed data.
    pub sync_to_components: bool,
}

impl UpdateDocumentFromEditCommand {
    /// Creates a command for full bidirectional sync.
    pub fn sync(document_id: DecisionDocumentId, content: String) -> Self {
        Self {
            document_id,
            content,
            sync_to_components: true,
        }
    }

    /// Creates a command for document-only update (no component sync).
    pub fn document_only(document_id: DecisionDocumentId, content: String) -> Self {
        Self {
            document_id,
            content,
            sync_to_components: false,
        }
    }
}

/// Result of successful document update.
#[derive(Debug, Clone)]
pub struct UpdateDocumentFromEditResult {
    /// The updated document ID.
    pub document_id: DecisionDocumentId,
    /// The cycle ID.
    pub cycle_id: CycleId,
    /// The new version number.
    pub version: u32,
    /// Parse result with any warnings.
    pub parse_result: ParseResultSummary,
    /// Number of components updated.
    pub components_updated: usize,
}

/// Summary of parse result for the response.
#[derive(Debug, Clone)]
pub struct ParseResultSummary {
    /// Number of sections successfully parsed.
    pub sections_parsed: usize,
    /// Number of warnings generated.
    pub warnings: usize,
    /// Number of errors generated.
    pub errors: usize,
}

impl From<&ParseResult> for ParseResultSummary {
    fn from(result: &ParseResult) -> Self {
        Self {
            sections_parsed: result.successful_section_count(),
            warnings: result.warnings.len(),
            errors: result.errors.len(),
        }
    }
}

/// Error type for document update.
#[derive(Debug, Clone)]
pub enum UpdateDocumentFromEditError {
    /// Document not found.
    DocumentNotFound(DecisionDocumentId),
    /// Cycle not found (data integrity issue).
    CycleNotFound(CycleId),
    /// Parse failed.
    ParseFailed(String),
    /// Failed to persist document.
    PersistFailed(String),
    /// Optimistic locking conflict.
    VersionConflict { expected: u32, actual: u32 },
    /// Domain error.
    Domain(DomainError),
}

impl std::fmt::Display for UpdateDocumentFromEditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateDocumentFromEditError::DocumentNotFound(id) => {
                write!(f, "Document not found: {}", id)
            }
            UpdateDocumentFromEditError::CycleNotFound(id) => {
                write!(f, "Cycle not found (data integrity issue): {}", id)
            }
            UpdateDocumentFromEditError::ParseFailed(msg) => write!(f, "Parse failed: {}", msg),
            UpdateDocumentFromEditError::PersistFailed(msg) => {
                write!(f, "Failed to persist: {}", msg)
            }
            UpdateDocumentFromEditError::VersionConflict { expected, actual } => {
                write!(
                    f,
                    "Version conflict: expected v{}, found v{}",
                    expected, actual
                )
            }
            UpdateDocumentFromEditError::Domain(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for UpdateDocumentFromEditError {}

impl From<DomainError> for UpdateDocumentFromEditError {
    fn from(err: DomainError) -> Self {
        UpdateDocumentFromEditError::Domain(err)
    }
}

impl From<DocumentError> for UpdateDocumentFromEditError {
    fn from(err: DocumentError) -> Self {
        UpdateDocumentFromEditError::ParseFailed(err.to_string())
    }
}

/// Handler for updating documents from user edits.
///
/// # Dependencies
///
/// - `DocumentParser`: Parse markdown back to structured data
/// - `DecisionDocumentRepository`: Persist document changes
/// - `CycleRepository`: Update component outputs (if syncing)
///
/// # Update Flow
///
/// 1. Find the existing document
/// 2. Parse the new content
/// 3. If sync enabled, update cycle components
/// 4. Update document content and version
/// 5. Persist changes
///
/// # Usage
///
/// ```rust,ignore
/// let handler = UpdateDocumentFromEditHandler::new(parser, doc_repo, cycle_repo);
/// let cmd = UpdateDocumentFromEditCommand::sync(doc_id, edited_content);
/// let result = handler.handle(cmd, metadata).await?;
/// println!("Updated {} components", result.components_updated);
/// ```
pub struct UpdateDocumentFromEditHandler {
    document_parser: Arc<dyn DocumentParser>,
    document_repository: Arc<dyn DecisionDocumentRepository>,
    cycle_repository: Arc<dyn CycleRepository>,
}

impl UpdateDocumentFromEditHandler {
    pub fn new(
        document_parser: Arc<dyn DocumentParser>,
        document_repository: Arc<dyn DecisionDocumentRepository>,
        cycle_repository: Arc<dyn CycleRepository>,
    ) -> Self {
        Self {
            document_parser,
            document_repository,
            cycle_repository,
        }
    }

    pub async fn handle(
        &self,
        cmd: UpdateDocumentFromEditCommand,
        _metadata: CommandMetadata,
    ) -> Result<UpdateDocumentFromEditResult, UpdateDocumentFromEditError> {
        // 1. Find the document
        let mut document = self
            .document_repository
            .find_by_id(cmd.document_id)
            .await?
            .ok_or(UpdateDocumentFromEditError::DocumentNotFound(cmd.document_id))?;

        let cycle_id = document.cycle_id();

        // 2. Parse the new content
        let parse_result = self.document_parser.parse(&cmd.content)?;

        // Check for critical parse errors
        if parse_result.has_errors() {
            return Err(UpdateDocumentFromEditError::ParseFailed(format!(
                "Document has {} parse errors",
                parse_result.errors.len()
            )));
        }

        // 3. If sync to components is enabled, update the cycle
        let mut components_updated = 0;

        if cmd.sync_to_components {
            let mut cycle = self
                .cycle_repository
                .find_by_id(&cycle_id)
                .await?
                .ok_or(UpdateDocumentFromEditError::CycleNotFound(cycle_id))?;

            // Update each successfully parsed section
            for section in &parse_result.sections {
                if section.is_successful() {
                    if let Some(data) = &section.parsed_data {
                        // Update the component output in the cycle
                        if self.update_component_output(&mut cycle, section.component_type, data) {
                            components_updated += 1;
                        }
                    }
                }
            }

            // Persist cycle changes
            if components_updated > 0 {
                self.cycle_repository
                    .update(&cycle)
                    .await
                    .map_err(|e| UpdateDocumentFromEditError::PersistFailed(e.to_string()))?;
            }
        }

        // 4. Update document
        document.update_from_user_edit(&cmd.content);

        // 5. Persist document
        self.document_repository
            .update(&document, &cmd.content)
            .await
            .map_err(|e| UpdateDocumentFromEditError::PersistFailed(e.to_string()))?;

        Ok(UpdateDocumentFromEditResult {
            document_id: cmd.document_id,
            cycle_id,
            version: document.version().as_u32(),
            parse_result: ParseResultSummary::from(&parse_result),
            components_updated,
        })
    }

    /// Updates a component's output with parsed data.
    ///
    /// Returns true if the component was updated.
    fn update_component_output(
        &self,
        cycle: &mut crate::domain::cycle::Cycle,
        component_type: ComponentType,
        data: &serde_json::Value,
    ) -> bool {
        // Skip empty data
        if data.is_null() || (data.is_object() && data.as_object().unwrap().is_empty()) {
            return false;
        }

        // Get the component if it exists
        if let Some(component) = cycle.component_mut(component_type) {
            // Update the output - ignore errors for now (best effort)
            if component.set_output_from_value(data.clone()).is_ok() {
                return true;
            }
        }

        // Component doesn't exist or update failed
        false
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cycle::{Cycle, DecisionDocument, ParseError, ParsedMetadata, ParsedSection};
    use crate::domain::foundation::{ErrorCode, SessionId, UserId};
    use crate::ports::SectionBoundary;
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::Mutex;

    // ───────────────────────────────────────────────────────────────
    // Mock implementations
    // ───────────────────────────────────────────────────────────────

    struct MockDocumentParser {
        result: Mutex<Option<ParseResult>>,
        fail: bool,
    }

    impl MockDocumentParser {
        fn new(result: ParseResult) -> Self {
            Self {
                result: Mutex::new(Some(result)),
                fail: false,
            }
        }

        fn failing() -> Self {
            Self {
                result: Mutex::new(None),
                fail: true,
            }
        }
    }

    impl DocumentParser for MockDocumentParser {
        fn parse(&self, _content: &str) -> Result<ParseResult, DocumentError> {
            if self.fail {
                return Err(DocumentError::internal("Simulated parse failure"));
            }
            Ok(self.result.lock().unwrap().take().unwrap_or_else(ParseResult::empty))
        }

        fn parse_section(
            &self,
            _section_content: &str,
            expected_type: ComponentType,
        ) -> Result<ParsedSection, DocumentError> {
            Ok(ParsedSection::success(
                expected_type,
                "test".to_string(),
                json!({}),
            ))
        }

        fn validate_structure(&self, _content: &str) -> Result<Vec<ParseError>, DocumentError> {
            Ok(vec![])
        }

        fn extract_section_boundaries(&self, _content: &str) -> Vec<SectionBoundary> {
            vec![]
        }
    }

    struct MockDocumentRepository {
        documents: Mutex<Vec<DecisionDocument>>,
        fail_update: bool,
    }

    impl MockDocumentRepository {
        fn with_document(doc: DecisionDocument) -> Self {
            Self {
                documents: Mutex::new(vec![doc]),
                fail_update: false,
            }
        }

        fn failing() -> Self {
            Self {
                documents: Mutex::new(Vec::new()),
                fail_update: true,
            }
        }
    }

    #[async_trait]
    impl DecisionDocumentRepository for MockDocumentRepository {
        async fn save(&self, document: &DecisionDocument, _content: &str) -> Result<(), DomainError> {
            self.documents.lock().unwrap().push(document.clone());
            Ok(())
        }

        async fn update(&self, document: &DecisionDocument, _content: &str) -> Result<(), DomainError> {
            if self.fail_update {
                return Err(DomainError::new(
                    ErrorCode::InternalError,
                    "Simulated update failure",
                ));
            }
            let mut docs = self.documents.lock().unwrap();
            if let Some(pos) = docs.iter().position(|d| d.id() == document.id()) {
                docs[pos] = document.clone();
            }
            Ok(())
        }

        async fn find_by_id(
            &self,
            id: DecisionDocumentId,
        ) -> Result<Option<DecisionDocument>, DomainError> {
            Ok(self
                .documents
                .lock()
                .unwrap()
                .iter()
                .find(|d| d.id() == id)
                .cloned())
        }

        async fn find_by_cycle(&self, cycle_id: CycleId) -> Result<Option<DecisionDocument>, DomainError> {
            Ok(self
                .documents
                .lock()
                .unwrap()
                .iter()
                .find(|d| d.cycle_id() == cycle_id)
                .cloned())
        }

        async fn sync_from_file(
            &self,
            _document_id: DecisionDocumentId,
        ) -> Result<crate::ports::SyncResult, DomainError> {
            Ok(crate::ports::SyncResult::unchanged("abc", 1))
        }

        async fn verify_integrity(
            &self,
            _document_id: DecisionDocumentId,
        ) -> Result<crate::ports::IntegrityStatus, DomainError> {
            Ok(crate::ports::IntegrityStatus::InSync)
        }

        async fn delete(&self, document_id: DecisionDocumentId) -> Result<(), DomainError> {
            let mut docs = self.documents.lock().unwrap();
            docs.retain(|d| d.id() != document_id);
            Ok(())
        }
    }

    struct MockCycleRepository {
        cycles: Mutex<Vec<Cycle>>,
    }

    impl MockCycleRepository {
        fn with_cycle(cycle: Cycle) -> Self {
            Self {
                cycles: Mutex::new(vec![cycle]),
            }
        }

        fn empty() -> Self {
            Self {
                cycles: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl CycleRepository for MockCycleRepository {
        async fn save(&self, cycle: &Cycle) -> Result<(), DomainError> {
            self.cycles.lock().unwrap().push(cycle.clone());
            Ok(())
        }

        async fn update(&self, cycle: &Cycle) -> Result<(), DomainError> {
            let mut cycles = self.cycles.lock().unwrap();
            if let Some(pos) = cycles.iter().position(|c| c.id() == cycle.id()) {
                cycles[pos] = cycle.clone();
            }
            Ok(())
        }

        async fn find_by_id(&self, id: &CycleId) -> Result<Option<Cycle>, DomainError> {
            Ok(self
                .cycles
                .lock()
                .unwrap()
                .iter()
                .find(|c| c.id() == *id)
                .cloned())
        }

        async fn exists(&self, _id: &CycleId) -> Result<bool, DomainError> {
            Ok(false)
        }

        async fn find_by_session_id(
            &self,
            _session_id: &SessionId,
        ) -> Result<Vec<Cycle>, DomainError> {
            Ok(vec![])
        }

        async fn find_primary_by_session_id(
            &self,
            _session_id: &SessionId,
        ) -> Result<Option<Cycle>, DomainError> {
            Ok(None)
        }

        async fn find_branches(&self, _parent_id: &CycleId) -> Result<Vec<Cycle>, DomainError> {
            Ok(vec![])
        }

        async fn count_by_session_id(&self, _session_id: &SessionId) -> Result<u32, DomainError> {
            Ok(0)
        }

        async fn delete(&self, _id: &CycleId) -> Result<(), DomainError> {
            Ok(())
        }
    }

    // ───────────────────────────────────────────────────────────────
    // Test helpers
    // ───────────────────────────────────────────────────────────────

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    fn test_metadata() -> CommandMetadata {
        CommandMetadata::new(test_user_id())
    }

    fn test_cycle() -> Cycle {
        let session_id = SessionId::new();
        let mut cycle = Cycle::new(session_id);
        // Start a component so we can update it
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle
    }

    fn test_document(cycle_id: CycleId) -> DecisionDocument {
        DecisionDocument::new(cycle_id, test_user_id(), "# Test Document")
    }

    fn success_parse_result() -> ParseResult {
        let mut result = ParseResult::empty();
        result.metadata = ParsedMetadata {
            title: Some("Test Decision".to_string()),
            ..Default::default()
        };
        // Use domain-compatible structure for IssueRaisingOutput
        result.sections.push(ParsedSection::success(
            ComponentType::IssueRaising,
            "## 1. Issue Raising".to_string(),
            json!({
                "potential_decisions": ["Option A", "Option B"],
                "objectives": [],
                "uncertainties": [],
                "considerations": [],
                "user_confirmed": false
            }),
        ));
        result
    }

    fn create_handler(
        parser: Arc<dyn DocumentParser>,
        doc_repo: Arc<dyn DecisionDocumentRepository>,
        cycle_repo: Arc<dyn CycleRepository>,
    ) -> UpdateDocumentFromEditHandler {
        UpdateDocumentFromEditHandler::new(parser, doc_repo, cycle_repo)
    }

    // ───────────────────────────────────────────────────────────────
    // Tests
    // ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn updates_document_only_when_sync_disabled() {
        let cycle = test_cycle();
        let cycle_id = cycle.id();
        let doc = test_document(cycle_id);
        let doc_id = doc.id();

        let parser = Arc::new(MockDocumentParser::new(success_parse_result()));
        let doc_repo = Arc::new(MockDocumentRepository::with_document(doc));
        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));

        let handler = create_handler(parser, doc_repo, cycle_repo);
        let cmd = UpdateDocumentFromEditCommand::document_only(doc_id, "# Updated".to_string());
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.components_updated, 0);
        assert_eq!(result.version, 2); // Version incremented
    }

    #[tokio::test]
    async fn updates_components_when_sync_enabled() {
        let cycle = test_cycle();
        let cycle_id = cycle.id();
        let doc = test_document(cycle_id);
        let doc_id = doc.id();

        let parser = Arc::new(MockDocumentParser::new(success_parse_result()));
        let doc_repo = Arc::new(MockDocumentRepository::with_document(doc));
        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));

        let handler = create_handler(parser, doc_repo, cycle_repo);
        let cmd = UpdateDocumentFromEditCommand::sync(doc_id, "# Updated".to_string());
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.components_updated, 1);
        assert_eq!(result.parse_result.sections_parsed, 1);
    }

    #[tokio::test]
    async fn fails_when_document_not_found() {
        let cycle = test_cycle();
        let doc_id = DecisionDocumentId::new();

        let parser = Arc::new(MockDocumentParser::new(success_parse_result()));
        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));

        // Need to create a repo that returns None
        struct EmptyDocRepo;
        #[async_trait]
        impl DecisionDocumentRepository for EmptyDocRepo {
            async fn save(&self, _: &DecisionDocument, _: &str) -> Result<(), DomainError> {
                Ok(())
            }
            async fn update(&self, _: &DecisionDocument, _: &str) -> Result<(), DomainError> {
                Ok(())
            }
            async fn find_by_id(&self, _: DecisionDocumentId) -> Result<Option<DecisionDocument>, DomainError> {
                Ok(None)
            }
            async fn find_by_cycle(&self, _: CycleId) -> Result<Option<DecisionDocument>, DomainError> {
                Ok(None)
            }
            async fn sync_from_file(&self, _: DecisionDocumentId) -> Result<crate::ports::SyncResult, DomainError> {
                Ok(crate::ports::SyncResult::unchanged("", 0))
            }
            async fn verify_integrity(&self, _: DecisionDocumentId) -> Result<crate::ports::IntegrityStatus, DomainError> {
                Ok(crate::ports::IntegrityStatus::InSync)
            }
            async fn delete(&self, _: DecisionDocumentId) -> Result<(), DomainError> {
                Ok(())
            }
        }

        let handler = create_handler(parser, Arc::new(EmptyDocRepo), cycle_repo);
        let cmd = UpdateDocumentFromEditCommand::sync(doc_id, "# Updated".to_string());
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(
            result,
            Err(UpdateDocumentFromEditError::DocumentNotFound(_))
        ));
    }

    #[tokio::test]
    async fn fails_when_cycle_not_found_during_sync() {
        let cycle = test_cycle();
        let cycle_id = cycle.id();
        let doc = test_document(cycle_id);
        let doc_id = doc.id();

        let parser = Arc::new(MockDocumentParser::new(success_parse_result()));
        let doc_repo = Arc::new(MockDocumentRepository::with_document(doc));
        let cycle_repo = Arc::new(MockCycleRepository::empty());

        let handler = create_handler(parser, doc_repo, cycle_repo);
        let cmd = UpdateDocumentFromEditCommand::sync(doc_id, "# Updated".to_string());
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(
            result,
            Err(UpdateDocumentFromEditError::CycleNotFound(_))
        ));
    }

    #[tokio::test]
    async fn fails_when_parse_fails() {
        let cycle = test_cycle();
        let cycle_id = cycle.id();
        let doc = test_document(cycle_id);
        let doc_id = doc.id();

        let parser = Arc::new(MockDocumentParser::failing());
        let doc_repo = Arc::new(MockDocumentRepository::with_document(doc));
        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));

        let handler = create_handler(parser, doc_repo, cycle_repo);
        let cmd = UpdateDocumentFromEditCommand::sync(doc_id, "# Updated".to_string());
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(
            result,
            Err(UpdateDocumentFromEditError::ParseFailed(_))
        ));
    }

    #[tokio::test]
    async fn returns_parse_summary() {
        let cycle = test_cycle();
        let cycle_id = cycle.id();
        let doc = test_document(cycle_id);
        let doc_id = doc.id();

        let mut parse_result = success_parse_result();
        parse_result.warnings.push(ParseError::warning(10, "Minor issue"));

        let parser = Arc::new(MockDocumentParser::new(parse_result));
        let doc_repo = Arc::new(MockDocumentRepository::with_document(doc));
        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));

        let handler = create_handler(parser, doc_repo, cycle_repo);
        let cmd = UpdateDocumentFromEditCommand::sync(doc_id, "# Updated".to_string());
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.parse_result.warnings, 1);
        assert_eq!(result.parse_result.errors, 0);
    }
}
