//! GenerateDocumentHandler - Command handler for generating decision documents.
//!
//! Generates a markdown decision document from a cycle's current state.
//! This is a query-like command that produces content without side effects.

use std::sync::Arc;

use crate::domain::foundation::{CommandMetadata, CycleId, DomainError, SessionId};
use crate::ports::{
    CycleRepository, DocumentError, DocumentFormat, DocumentGenerator, GenerationOptions,
    SessionRepository,
};

/// Command to generate a decision document.
#[derive(Debug, Clone)]
pub struct GenerateDocumentCommand {
    /// Cycle to generate document for.
    pub cycle_id: CycleId,
    /// Output format.
    pub format: DocumentFormat,
    /// Include empty sections.
    pub include_empty_sections: bool,
    /// Include metadata block.
    pub include_metadata: bool,
    /// Include version info in footer.
    pub include_version_info: bool,
}

impl GenerateDocumentCommand {
    /// Creates a command for full document generation.
    pub fn full(cycle_id: CycleId) -> Self {
        Self {
            cycle_id,
            format: DocumentFormat::Full,
            include_empty_sections: true,
            include_metadata: true,
            include_version_info: true,
        }
    }

    /// Creates a command for summary document generation.
    pub fn summary(cycle_id: CycleId) -> Self {
        Self {
            cycle_id,
            format: DocumentFormat::Summary,
            include_empty_sections: false,
            include_metadata: false,
            include_version_info: false,
        }
    }

    /// Creates a command for export document generation.
    pub fn export(cycle_id: CycleId) -> Self {
        Self {
            cycle_id,
            format: DocumentFormat::Export,
            include_empty_sections: false,
            include_metadata: false,
            include_version_info: true,
        }
    }
}

/// Result of successful document generation.
#[derive(Debug, Clone)]
pub struct GenerateDocumentResult {
    /// The generated markdown content.
    pub content: String,
    /// The cycle ID the document was generated from.
    pub cycle_id: CycleId,
    /// The session ID.
    pub session_id: SessionId,
    /// The format used.
    pub format: DocumentFormat,
}

/// Error type for document generation.
#[derive(Debug, Clone)]
pub enum GenerateDocumentError {
    /// Cycle not found.
    CycleNotFound(CycleId),
    /// Session not found (data integrity issue).
    SessionNotFound(SessionId),
    /// Document generation failed.
    GenerationFailed(String),
    /// Domain error.
    Domain(DomainError),
}

impl std::fmt::Display for GenerateDocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenerateDocumentError::CycleNotFound(id) => write!(f, "Cycle not found: {}", id),
            GenerateDocumentError::SessionNotFound(id) => {
                write!(f, "Session not found (data integrity issue): {}", id)
            }
            GenerateDocumentError::GenerationFailed(msg) => {
                write!(f, "Document generation failed: {}", msg)
            }
            GenerateDocumentError::Domain(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for GenerateDocumentError {}

impl From<DomainError> for GenerateDocumentError {
    fn from(err: DomainError) -> Self {
        GenerateDocumentError::Domain(err)
    }
}

impl From<DocumentError> for GenerateDocumentError {
    fn from(err: DocumentError) -> Self {
        GenerateDocumentError::GenerationFailed(err.to_string())
    }
}

/// Handler for generating decision documents.
///
/// # Dependencies
///
/// - `CycleRepository`: Read cycle state
/// - `SessionRepository`: Read session title
/// - `DocumentGenerator`: Generate markdown content
///
/// # Usage
///
/// ```rust,ignore
/// let handler = GenerateDocumentHandler::new(cycle_repo, session_repo, generator);
/// let cmd = GenerateDocumentCommand::full(cycle_id);
/// let result = handler.handle(cmd, metadata).await?;
/// println!("{}", result.content);
/// ```
pub struct GenerateDocumentHandler {
    cycle_repository: Arc<dyn CycleRepository>,
    session_repository: Arc<dyn SessionRepository>,
    document_generator: Arc<dyn DocumentGenerator>,
}

impl GenerateDocumentHandler {
    pub fn new(
        cycle_repository: Arc<dyn CycleRepository>,
        session_repository: Arc<dyn SessionRepository>,
        document_generator: Arc<dyn DocumentGenerator>,
    ) -> Self {
        Self {
            cycle_repository,
            session_repository,
            document_generator,
        }
    }

    pub async fn handle(
        &self,
        cmd: GenerateDocumentCommand,
        _metadata: CommandMetadata,
    ) -> Result<GenerateDocumentResult, GenerateDocumentError> {
        // 1. Find the cycle
        let cycle = self
            .cycle_repository
            .find_by_id(&cmd.cycle_id)
            .await?
            .ok_or(GenerateDocumentError::CycleNotFound(cmd.cycle_id))?;

        // 2. Find the session (for title)
        let session_id = cycle.session_id();
        let session = self
            .session_repository
            .find_by_id(&session_id)
            .await?
            .ok_or(GenerateDocumentError::SessionNotFound(session_id))?;

        // 3. Build generation options
        let options = GenerationOptions {
            format: cmd.format.clone(),
            include_empty_sections: cmd.include_empty_sections,
            include_metadata: cmd.include_metadata,
            include_version_info: cmd.include_version_info,
        };

        // 4. Generate the document
        let content = self
            .document_generator
            .generate(session.title(), &cycle, options)?;

        Ok(GenerateDocumentResult {
            content,
            cycle_id: cmd.cycle_id,
            session_id,
            format: cmd.format,
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
    use crate::domain::foundation::UserId;
    use crate::domain::session::Session;
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
    ) -> GenerateDocumentHandler {
        GenerateDocumentHandler::new(cycle_repo, session_repo, generator)
    }

    // ───────────────────────────────────────────────────────────────
    // Tests
    // ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn generates_document_for_valid_cycle() {
        let session = test_session();
        let session_id = *session.id();
        let cycle = test_cycle(session_id);
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let generator = Arc::new(MockDocumentGenerator::new("Test content"));

        let handler = create_handler(cycle_repo, session_repo, generator);
        let cmd = GenerateDocumentCommand::full(cycle_id);
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("Career Decision"));
        assert!(result.content.contains("Test content"));
        assert_eq!(result.cycle_id, cycle_id);
        assert_eq!(result.session_id, session_id);
    }

    #[tokio::test]
    async fn fails_when_cycle_not_found() {
        let session = test_session();
        let cycle_id = CycleId::new();

        let cycle_repo = Arc::new(MockCycleRepository::new());
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let generator = Arc::new(MockDocumentGenerator::new("Test"));

        let handler = create_handler(cycle_repo, session_repo, generator);
        let cmd = GenerateDocumentCommand::full(cycle_id);
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(
            result,
            Err(GenerateDocumentError::CycleNotFound(_))
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

        let handler = create_handler(cycle_repo, session_repo, generator);
        let cmd = GenerateDocumentCommand::full(cycle_id);
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(
            result,
            Err(GenerateDocumentError::SessionNotFound(_))
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

        let handler = create_handler(cycle_repo, session_repo, generator);
        let cmd = GenerateDocumentCommand::full(cycle_id);
        let result = handler.handle(cmd, test_metadata()).await;

        assert!(matches!(
            result,
            Err(GenerateDocumentError::GenerationFailed(_))
        ));
    }

    #[tokio::test]
    async fn full_command_sets_correct_options() {
        let cmd = GenerateDocumentCommand::full(CycleId::new());
        assert_eq!(cmd.format, DocumentFormat::Full);
        assert!(cmd.include_empty_sections);
        assert!(cmd.include_metadata);
        assert!(cmd.include_version_info);
    }

    #[tokio::test]
    async fn summary_command_sets_correct_options() {
        let cmd = GenerateDocumentCommand::summary(CycleId::new());
        assert_eq!(cmd.format, DocumentFormat::Summary);
        assert!(!cmd.include_empty_sections);
        assert!(!cmd.include_metadata);
        assert!(!cmd.include_version_info);
    }

    #[tokio::test]
    async fn export_command_sets_correct_options() {
        let cmd = GenerateDocumentCommand::export(CycleId::new());
        assert_eq!(cmd.format, DocumentFormat::Export);
        assert!(!cmd.include_empty_sections);
        assert!(!cmd.include_metadata);
        assert!(cmd.include_version_info);
    }

    #[tokio::test]
    async fn result_includes_format() {
        let session = test_session();
        let session_id = *session.id();
        let cycle = test_cycle(session_id);
        let cycle_id = cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(cycle));
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let generator = Arc::new(MockDocumentGenerator::new("Test"));

        let handler = create_handler(cycle_repo, session_repo, generator);
        let cmd = GenerateDocumentCommand::summary(cycle_id);
        let result = handler.handle(cmd, test_metadata()).await.unwrap();

        assert_eq!(result.format, DocumentFormat::Summary);
    }
}
