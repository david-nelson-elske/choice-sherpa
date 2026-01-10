//! RegenerateDocumentHandler - Command handler for regenerating and persisting decision documents.
//!
//! Regenerates a markdown decision document from a cycle's current state and
//! persists it to both the database and filesystem.

use std::sync::Arc;

use crate::domain::cycle::DecisionDocument;
use crate::domain::foundation::{CommandMetadata, CycleId, DecisionDocumentId, DomainError, SessionId};
use crate::ports::{
    CycleRepository, DecisionDocumentRepository, DocumentError, DocumentFormat, DocumentGenerator,
    GenerationOptions, SessionRepository,
};

/// Command to regenerate a decision document.
#[derive(Debug, Clone)]
pub struct RegenerateDocumentCommand {
    /// Cycle to regenerate document for.
    pub cycle_id: CycleId,
    /// Output format.
    pub format: DocumentFormat,
}

impl RegenerateDocumentCommand {
    /// Creates a command for full document regeneration.
    pub fn full(cycle_id: CycleId) -> Self {
        Self {
            cycle_id,
            format: DocumentFormat::Full,
        }
    }

    /// Creates a command for summary document regeneration.
    pub fn summary(cycle_id: CycleId) -> Self {
        Self {
            cycle_id,
            format: DocumentFormat::Summary,
        }
    }
}

/// Result of successful document regeneration.
#[derive(Debug, Clone)]
pub struct RegenerateDocumentResult {
    /// The regenerated document ID.
    pub document_id: DecisionDocumentId,
    /// The cycle ID the document was regenerated from.
    pub cycle_id: CycleId,
    /// The session ID.
    pub session_id: SessionId,
    /// The new version number.
    pub version: u32,
    /// The regenerated markdown content.
    pub content: String,
    /// The format used.
    pub format: DocumentFormat,
    /// Whether this was a new document or an update.
    pub is_new: bool,
}

/// Error type for document regeneration.
#[derive(Debug, Clone)]
pub enum RegenerateDocumentError {
    /// Cycle not found.
    CycleNotFound(CycleId),
    /// Session not found (data integrity issue).
    SessionNotFound(SessionId),
    /// Document generation failed.
    GenerationFailed(String),
    /// Failed to persist document.
    PersistFailed(String),
    /// Domain error.
    Domain(DomainError),
}

impl std::fmt::Display for RegenerateDocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegenerateDocumentError::CycleNotFound(id) => write!(f, "Cycle not found: {}", id),
            RegenerateDocumentError::SessionNotFound(id) => {
                write!(f, "Session not found (data integrity issue): {}", id)
            }
            RegenerateDocumentError::GenerationFailed(msg) => {
                write!(f, "Document generation failed: {}", msg)
            }
            RegenerateDocumentError::PersistFailed(msg) => {
                write!(f, "Failed to persist document: {}", msg)
            }
            RegenerateDocumentError::Domain(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for RegenerateDocumentError {}

impl From<DomainError> for RegenerateDocumentError {
    fn from(err: DomainError) -> Self {
        RegenerateDocumentError::Domain(err)
    }
}

impl From<DocumentError> for RegenerateDocumentError {
    fn from(err: DocumentError) -> Self {
        RegenerateDocumentError::GenerationFailed(err.to_string())
    }
}

/// Handler for regenerating and persisting decision documents.
///
/// # Dependencies
///
/// - `CycleRepository`: Read cycle state
/// - `SessionRepository`: Read session title and owner
/// - `DocumentGenerator`: Generate markdown content
/// - `DecisionDocumentRepository`: Persist documents
///
/// # Regeneration Flow
///
/// 1. Find the cycle and session
/// 2. Generate fresh markdown content from cycle state
/// 3. Find existing document or create new one
/// 4. Update content and version
/// 5. Persist to dual storage (DB + filesystem)
///
/// # Usage
///
/// ```rust,ignore
/// let handler = RegenerateDocumentHandler::new(cycle_repo, session_repo, generator, doc_repo);
/// let cmd = RegenerateDocumentCommand::full(cycle_id);
/// let result = handler.handle(cmd, metadata).await?;
/// println!("Regenerated document v{}", result.version);
/// ```
pub struct RegenerateDocumentHandler {
    cycle_repository: Arc<dyn CycleRepository>,
    session_repository: Arc<dyn SessionRepository>,
    document_generator: Arc<dyn DocumentGenerator>,
    document_repository: Arc<dyn DecisionDocumentRepository>,
}

impl RegenerateDocumentHandler {
    pub fn new(
        cycle_repository: Arc<dyn CycleRepository>,
        session_repository: Arc<dyn SessionRepository>,
        document_generator: Arc<dyn DocumentGenerator>,
        document_repository: Arc<dyn DecisionDocumentRepository>,
    ) -> Self {
        Self {
            cycle_repository,
            session_repository,
            document_generator,
            document_repository,
        }
    }

    pub async fn handle(
        &self,
        cmd: RegenerateDocumentCommand,
        _metadata: CommandMetadata,
    ) -> Result<RegenerateDocumentResult, RegenerateDocumentError> {
        // 1. Find the cycle
        let cycle = self
            .cycle_repository
            .find_by_id(&cmd.cycle_id)
            .await?
            .ok_or(RegenerateDocumentError::CycleNotFound(cmd.cycle_id))?;

        // 2. Find the session (for title and user)
        let session_id = cycle.session_id();
        let session = self
            .session_repository
            .find_by_id(&session_id)
            .await?
            .ok_or(RegenerateDocumentError::SessionNotFound(session_id))?;

        // 3. Build generation options
        let options = GenerationOptions {
            format: cmd.format.clone(),
            include_empty_sections: matches!(cmd.format, DocumentFormat::Full),
            include_metadata: matches!(cmd.format, DocumentFormat::Full),
            include_version_info: true,
        };

        // 4. Generate the document
        let content = self
            .document_generator
            .generate(session.title(), &cycle, options)?;

        // 5. Check if document exists for this cycle
        let existing_doc = self.document_repository.find_by_cycle(cmd.cycle_id).await?;

        let (document_id, version, is_new) = match existing_doc {
            Some(mut doc) => {
                // Update existing document
                doc.update_from_components(&content);
                let doc_id = doc.id();
                let version = doc.version().as_u32();

                self.document_repository
                    .update(&doc, &content)
                    .await
                    .map_err(|e| RegenerateDocumentError::PersistFailed(e.to_string()))?;

                (doc_id, version, false)
            }
            None => {
                // Create new document
                let user_id = session.user_id().clone();
                let doc = DecisionDocument::new(cmd.cycle_id, user_id, &content);
                let doc_id = doc.id();
                let version = doc.version().as_u32();

                self.document_repository
                    .save(&doc, &content)
                    .await
                    .map_err(|e| RegenerateDocumentError::PersistFailed(e.to_string()))?;

                (doc_id, version, true)
            }
        };

        Ok(RegenerateDocumentResult {
            document_id,
            cycle_id: cmd.cycle_id,
            session_id,
            version,
            content,
            format: cmd.format,
            is_new,
        })
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cycle::Cycle;
    use crate::domain::foundation::{ErrorCode, UserId};
    use crate::domain::session::Session;
    use crate::ports::GenerationOptions;
    use async_trait::async_trait;
    use std::sync::Mutex;

    // ───────────────────────────────────────────────────────────────
    // Mock implementations
    // ───────────────────────────────────────────────────────────────

    struct MockCycleRepository {
        cycles: Mutex<Vec<Cycle>>,
    }

    impl MockCycleRepository {
        fn new() -> Self {
            Self {
                cycles: Mutex::new(Vec::new()),
            }
        }

        fn with_cycle(cycle: Cycle) -> Self {
            Self {
                cycles: Mutex::new(vec![cycle]),
            }
        }
    }

    #[async_trait]
    impl CycleRepository for MockCycleRepository {
        async fn save(&self, cycle: &Cycle) -> Result<(), DomainError> {
            self.cycles.lock().unwrap().push(cycle.clone());
            Ok(())
        }

        async fn update(&self, _cycle: &Cycle) -> Result<(), DomainError> {
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

    struct MockSessionRepository {
        sessions: Mutex<Vec<Session>>,
    }

    impl MockSessionRepository {
        fn new() -> Self {
            Self {
                sessions: Mutex::new(Vec::new()),
            }
        }

        fn with_session(session: Session) -> Self {
            Self {
                sessions: Mutex::new(vec![session]),
            }
        }
    }

    #[async_trait]
    impl SessionRepository for MockSessionRepository {
        async fn save(&self, session: &Session) -> Result<(), DomainError> {
            self.sessions.lock().unwrap().push(session.clone());
            Ok(())
        }

        async fn update(&self, _session: &Session) -> Result<(), DomainError> {
            Ok(())
        }

        async fn find_by_id(&self, id: &SessionId) -> Result<Option<Session>, DomainError> {
            Ok(self
                .sessions
                .lock()
                .unwrap()
                .iter()
                .find(|s| s.id() == id)
                .cloned())
        }

        async fn exists(&self, _id: &SessionId) -> Result<bool, DomainError> {
            Ok(false)
        }

        async fn find_by_user_id(&self, _user_id: &UserId) -> Result<Vec<Session>, DomainError> {
            Ok(vec![])
        }

        async fn count_active_by_user(&self, _user_id: &UserId) -> Result<u32, DomainError> {
            Ok(0)
        }

        async fn delete(&self, _id: &SessionId) -> Result<(), DomainError> {
            Ok(())
        }
    }

    struct MockDocumentGenerator {
        content: String,
        fail: bool,
    }

    impl MockDocumentGenerator {
        fn new(content: &str) -> Self {
            Self {
                content: content.to_string(),
                fail: false,
            }
        }

        fn failing() -> Self {
            Self {
                content: String::new(),
                fail: true,
            }
        }
    }

    impl DocumentGenerator for MockDocumentGenerator {
        fn generate(
            &self,
            session_title: &str,
            _cycle: &Cycle,
            _options: GenerationOptions,
        ) -> Result<String, DocumentError> {
            if self.fail {
                return Err(DocumentError::internal("Simulated failure"));
            }
            Ok(format!("# {}\n\n{}", session_title, self.content))
        }

        fn generate_section(
            &self,
            _component_type: crate::domain::foundation::ComponentType,
            _output: &serde_json::Value,
        ) -> Result<String, DocumentError> {
            Ok(self.content.clone())
        }

        fn generate_header(
            &self,
            session_title: &str,
            _options: &GenerationOptions,
        ) -> Result<String, DocumentError> {
            Ok(format!("# {}\n", session_title))
        }

        fn generate_footer(
            &self,
            _cycle: &Cycle,
            _options: &GenerationOptions,
        ) -> Result<String, DocumentError> {
            Ok("---\n*Generated*\n".to_string())
        }
    }

    struct MockDocumentRepository {
        documents: Mutex<Vec<DecisionDocument>>,
        fail_save: bool,
    }

    impl MockDocumentRepository {
        fn new() -> Self {
            Self {
                documents: Mutex::new(Vec::new()),
                fail_save: false,
            }
        }

        fn with_document(doc: DecisionDocument) -> Self {
            Self {
                documents: Mutex::new(vec![doc]),
                fail_save: false,
            }
        }

        fn failing() -> Self {
            Self {
                documents: Mutex::new(Vec::new()),
                fail_save: true,
            }
        }
    }

    #[async_trait]
    impl DecisionDocumentRepository for MockDocumentRepository {
        async fn save(&self, document: &DecisionDocument, _content: &str) -> Result<(), DomainError> {
            if self.fail_save {
                return Err(DomainError::new(ErrorCode::InternalError, "Simulated save failure"));
            }
            self.documents.lock().unwrap().push(document.clone());
            Ok(())
        }

        async fn update(&self, document: &DecisionDocument, _content: &str) -> Result<(), DomainError> {
            if self.fail_save {
                return Err(DomainError::new(ErrorCode::InternalError, "Simulated update failure"));
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

    // ───────────────────────────────────────────────────────────────
    // Test helpers
    // ───────────────────────────────────────────────────────────────

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    fn test_session() -> Session {
        Session::new(SessionId::new(), test_user_id(), "Career Decision".to_string()).unwrap()
    }

    fn test_cycle(session_id: SessionId) -> Cycle {
        Cycle::new(session_id)
    }

    fn test_metadata() -> CommandMetadata {
        CommandMetadata::new(test_user_id())
    }

    fn create_handler(
        cycle_repo: Arc<dyn CycleRepository>,
        session_repo: Arc<dyn SessionRepository>,
        generator: Arc<dyn DocumentGenerator>,
        doc_repo: Arc<dyn DecisionDocumentRepository>,
    ) -> RegenerateDocumentHandler {
        RegenerateDocumentHandler::new(cycle_repo, session_repo, generator, doc_repo)
    }

    // ───────────────────────────────────────────────────────────────
    // Tests
    // ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn regenerates_new_document_when_none_exists() {
        let session = test_session();
        let session_id = *session.id();
        let cycle = test_cycle(session_id);
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let generator = Arc::new(MockDocumentGenerator::new("Test content"));
        let doc_repo = Arc::new(MockDocumentRepository::new());

        let handler = create_handler(cycle_repo, session_repo, generator, doc_repo);
        let cmd = RegenerateDocumentCommand::full(cycle_id);
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_new);
        assert_eq!(result.cycle_id, cycle_id);
        assert_eq!(result.session_id, session_id);
        assert!(result.content.contains("Career Decision"));
    }

    #[tokio::test]
    async fn regenerates_existing_document() {
        let session = test_session();
        let session_id = *session.id();
        let cycle = test_cycle(session_id);
        let cycle_id = cycle.id();

        let existing_doc = DecisionDocument::new(cycle_id, test_user_id(), "# Old content");

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let generator = Arc::new(MockDocumentGenerator::new("New content"));
        let doc_repo = Arc::new(MockDocumentRepository::with_document(existing_doc));

        let handler = create_handler(cycle_repo, session_repo, generator, doc_repo);
        let cmd = RegenerateDocumentCommand::full(cycle_id);
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.is_new);
        assert_eq!(result.version, 2); // Version incremented
        assert!(result.content.contains("New content"));
    }

    #[tokio::test]
    async fn fails_when_cycle_not_found() {
        let session = test_session();
        let cycle_id = CycleId::new();

        let cycle_repo = Arc::new(MockCycleRepository::new());
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let generator = Arc::new(MockDocumentGenerator::new("Test"));
        let doc_repo = Arc::new(MockDocumentRepository::new());

        let handler = create_handler(cycle_repo, session_repo, generator, doc_repo);
        let cmd = RegenerateDocumentCommand::full(cycle_id);
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(
            result,
            Err(RegenerateDocumentError::CycleNotFound(_))
        ));
    }

    #[tokio::test]
    async fn fails_when_session_not_found() {
        let session_id = SessionId::new();
        let cycle = test_cycle(session_id);
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let session_repo = Arc::new(MockSessionRepository::new());
        let generator = Arc::new(MockDocumentGenerator::new("Test"));
        let doc_repo = Arc::new(MockDocumentRepository::new());

        let handler = create_handler(cycle_repo, session_repo, generator, doc_repo);
        let cmd = RegenerateDocumentCommand::full(cycle_id);
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(
            result,
            Err(RegenerateDocumentError::SessionNotFound(_))
        ));
    }

    #[tokio::test]
    async fn fails_when_generation_fails() {
        let session = test_session();
        let session_id = *session.id();
        let cycle = test_cycle(session_id);
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let generator = Arc::new(MockDocumentGenerator::failing());
        let doc_repo = Arc::new(MockDocumentRepository::new());

        let handler = create_handler(cycle_repo, session_repo, generator, doc_repo);
        let cmd = RegenerateDocumentCommand::full(cycle_id);
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(
            result,
            Err(RegenerateDocumentError::GenerationFailed(_))
        ));
    }

    #[tokio::test]
    async fn fails_when_persist_fails() {
        let session = test_session();
        let session_id = *session.id();
        let cycle = test_cycle(session_id);
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let generator = Arc::new(MockDocumentGenerator::new("Test"));
        let doc_repo = Arc::new(MockDocumentRepository::failing());

        let handler = create_handler(cycle_repo, session_repo, generator, doc_repo);
        let cmd = RegenerateDocumentCommand::full(cycle_id);
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(
            result,
            Err(RegenerateDocumentError::PersistFailed(_))
        ));
    }

    #[tokio::test]
    async fn full_command_creates_correct_options() {
        let cmd = RegenerateDocumentCommand::full(CycleId::new());
        assert_eq!(cmd.format, DocumentFormat::Full);
    }

    #[tokio::test]
    async fn summary_command_creates_correct_options() {
        let cmd = RegenerateDocumentCommand::summary(CycleId::new());
        assert_eq!(cmd.format, DocumentFormat::Summary);
    }
}
