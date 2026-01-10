//! BranchWithDocumentHandler - Branches a cycle and creates its document.
//!
//! This handler coordinates the cycle branching with document creation,
//! ensuring that when a user branches at a component, they get both
//! a new cycle and a corresponding document artifact.

use std::sync::Arc;

use crate::domain::cycle::{Cycle, DecisionDocument};
use crate::domain::foundation::{ComponentType, CycleId, DomainError, UserId};
use crate::ports::{
    CycleRepository, DecisionDocumentRepository, DocumentGenerator, GenerationOptions,
    SessionRepository,
};

/// Command to branch a cycle and create its document.
#[derive(Debug, Clone)]
pub struct BranchWithDocumentCommand {
    /// The cycle to branch from.
    pub parent_cycle_id: CycleId,
    /// The component where branching occurs.
    pub branch_point: ComponentType,
    /// Optional label for the branch (e.g., "Remote Option").
    pub branch_label: Option<String>,
    /// The user performing the branch.
    pub user_id: UserId,
}

/// Result of successful branch with document creation.
#[derive(Debug, Clone)]
pub struct BranchWithDocumentResult {
    /// The newly created branch cycle.
    pub branch_cycle: Cycle,
    /// The document created for the branch.
    pub document: DecisionDocument,
    /// The generated document content.
    pub content: String,
}

/// Error type for branch with document operations.
#[derive(Debug, Clone)]
pub enum BranchWithDocumentError {
    /// Parent cycle not found.
    CycleNotFound(CycleId),
    /// Parent document not found.
    ParentDocumentNotFound(CycleId),
    /// Session not found.
    SessionNotFound,
    /// Domain error (e.g., invalid branch point).
    Domain(DomainError),
    /// Document generation failed.
    GenerationFailed(String),
}

impl std::fmt::Display for BranchWithDocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BranchWithDocumentError::CycleNotFound(id) => {
                write!(f, "Cycle not found: {}", id)
            }
            BranchWithDocumentError::ParentDocumentNotFound(id) => {
                write!(f, "Parent document not found for cycle: {}", id)
            }
            BranchWithDocumentError::SessionNotFound => {
                write!(f, "Session not found")
            }
            BranchWithDocumentError::Domain(err) => write!(f, "{}", err),
            BranchWithDocumentError::GenerationFailed(msg) => {
                write!(f, "Document generation failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for BranchWithDocumentError {}

impl From<DomainError> for BranchWithDocumentError {
    fn from(err: DomainError) -> Self {
        BranchWithDocumentError::Domain(err)
    }
}

/// Handler for branching cycles with document creation.
pub struct BranchWithDocumentHandler {
    cycle_repository: Arc<dyn CycleRepository>,
    session_repository: Arc<dyn SessionRepository>,
    document_repository: Arc<dyn DecisionDocumentRepository>,
    document_generator: Arc<dyn DocumentGenerator>,
}

impl BranchWithDocumentHandler {
    pub fn new(
        cycle_repository: Arc<dyn CycleRepository>,
        session_repository: Arc<dyn SessionRepository>,
        document_repository: Arc<dyn DecisionDocumentRepository>,
        document_generator: Arc<dyn DocumentGenerator>,
    ) -> Self {
        Self {
            cycle_repository,
            session_repository,
            document_repository,
            document_generator,
        }
    }

    pub async fn handle(
        &self,
        cmd: BranchWithDocumentCommand,
    ) -> Result<BranchWithDocumentResult, BranchWithDocumentError> {
        // 1. Find the parent cycle
        let parent_cycle = self
            .cycle_repository
            .find_by_id(&cmd.parent_cycle_id)
            .await?
            .ok_or(BranchWithDocumentError::CycleNotFound(cmd.parent_cycle_id))?;

        // 2. Find the session for the title
        let session = self
            .session_repository
            .find_by_id(&parent_cycle.session_id())
            .await?
            .ok_or(BranchWithDocumentError::SessionNotFound)?;

        // 3. Get the parent document (if it exists)
        let parent_document = self
            .document_repository
            .find_by_cycle(parent_cycle.id())
            .await?;

        // 4. Branch the cycle (domain logic handles validation)
        let branch_cycle = parent_cycle.branch_at(cmd.branch_point)?;

        // 5. Persist the branched cycle
        self.cycle_repository.save(&branch_cycle).await?;

        // 6. Generate document content for the branch
        let branch_label = cmd.branch_label.clone().unwrap_or_else(|| {
            format!("Branch at {}", cmd.branch_point.display_name())
        });

        let options = GenerationOptions::full();
        let content = self
            .document_generator
            .generate(session.title(), &branch_cycle, options)
            .map_err(|e| BranchWithDocumentError::GenerationFailed(e.to_string()))?;

        // 7. Create the branched document
        let document = if let Some(ref parent_doc) = parent_document {
            DecisionDocument::new_branch(
                branch_cycle.id(),
                cmd.user_id.clone(),
                parent_doc.id(),
                cmd.branch_point,
                &branch_label,
                &content,
            )
        } else {
            // No parent document - create a new document for the branch
            DecisionDocument::new(branch_cycle.id(), cmd.user_id.clone(), &content)
        };

        // 8. Persist the document
        self.document_repository.save(&document, &content).await?;

        Ok(BranchWithDocumentResult {
            branch_cycle,
            document,
            content,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{DecisionDocumentId, SessionId};
    use crate::domain::session::Session;
    use crate::ports::{DocumentError, IntegrityStatus, SyncResult};
    use async_trait::async_trait;
    use std::sync::Mutex;

    // ─────────────────────────────────────────────────────────────────────
    // Mock implementations
    // ─────────────────────────────────────────────────────────────────────

    struct MockCycleRepository {
        cycles: Mutex<Vec<Cycle>>,
        saved: Mutex<Vec<Cycle>>,
    }

    impl MockCycleRepository {
        fn with_cycle(cycle: Cycle) -> Self {
            Self {
                cycles: Mutex::new(vec![cycle]),
                saved: Mutex::new(Vec::new()),
            }
        }

        fn saved_cycles(&self) -> Vec<Cycle> {
            self.saved.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl CycleRepository for MockCycleRepository {
        async fn save(&self, cycle: &Cycle) -> Result<(), DomainError> {
            self.saved.lock().unwrap().push(cycle.clone());
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

        async fn find_by_session_id(&self, _session_id: &SessionId) -> Result<Vec<Cycle>, DomainError> {
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
        fn with_session(session: Session) -> Self {
            Self {
                sessions: Mutex::new(vec![session]),
            }
        }
    }

    #[async_trait]
    impl SessionRepository for MockSessionRepository {
        async fn save(&self, _session: &Session) -> Result<(), DomainError> {
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

    struct MockDocumentRepository {
        documents: Mutex<Vec<(DecisionDocument, String)>>,
    }

    impl MockDocumentRepository {
        fn empty() -> Self {
            Self {
                documents: Mutex::new(Vec::new()),
            }
        }

        fn with_document(doc: DecisionDocument, content: String) -> Self {
            Self {
                documents: Mutex::new(vec![(doc, content)]),
            }
        }

        fn saved_documents(&self) -> Vec<DecisionDocument> {
            self.documents.lock().unwrap().iter().map(|(d, _)| d.clone()).collect()
        }
    }

    #[async_trait]
    impl DecisionDocumentRepository for MockDocumentRepository {
        async fn save(&self, document: &DecisionDocument, content: &str) -> Result<(), DomainError> {
            self.documents
                .lock()
                .unwrap()
                .push((document.clone(), content.to_string()));
            Ok(())
        }

        async fn update(&self, _document: &DecisionDocument, _content: &str) -> Result<(), DomainError> {
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
                .find(|(d, _)| d.id() == id)
                .map(|(d, _)| d.clone()))
        }

        async fn find_by_cycle(&self, cycle_id: CycleId) -> Result<Option<DecisionDocument>, DomainError> {
            Ok(self
                .documents
                .lock()
                .unwrap()
                .iter()
                .find(|(d, _)| d.cycle_id() == cycle_id)
                .map(|(d, _)| d.clone()))
        }

        async fn sync_from_file(
            &self,
            _document_id: DecisionDocumentId,
        ) -> Result<SyncResult, DomainError> {
            Ok(SyncResult::unchanged("abc", 1))
        }

        async fn verify_integrity(
            &self,
            _document_id: DecisionDocumentId,
        ) -> Result<IntegrityStatus, DomainError> {
            Ok(IntegrityStatus::InSync)
        }

        async fn delete(&self, _document_id: DecisionDocumentId) -> Result<(), DomainError> {
            Ok(())
        }
    }

    struct MockDocumentGenerator;

    impl DocumentGenerator for MockDocumentGenerator {
        fn generate(
            &self,
            session_title: &str,
            _cycle: &Cycle,
            _options: GenerationOptions,
        ) -> Result<String, DocumentError> {
            Ok(format!("# {}\n\nBranched document content", session_title))
        }

        fn generate_section(
            &self,
            _component_type: ComponentType,
            _output: &serde_json::Value,
        ) -> Result<String, DocumentError> {
            Ok("Section content".to_string())
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
            Ok("---\n".to_string())
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Test helpers
    // ─────────────────────────────────────────────────────────────────────

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    fn create_parent_cycle_with_started_component(session_id: SessionId) -> Cycle {
        let mut cycle = Cycle::new(session_id);
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle.take_events(); // Clear events from setup
        cycle
    }

    // ─────────────────────────────────────────────────────────────────────
    // Tests
    // ─────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn branches_cycle_and_creates_document() {
        let session = Session::new(SessionId::new(), test_user_id(), "Test Decision".to_string()).unwrap();
        let session_id = *session.id();
        let parent_cycle = create_parent_cycle_with_started_component(session_id);
        let parent_id = parent_cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(parent_cycle));
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let doc_repo = Arc::new(MockDocumentRepository::empty());
        let doc_gen = Arc::new(MockDocumentGenerator);

        let handler = BranchWithDocumentHandler::new(
            cycle_repo.clone(),
            session_repo,
            doc_repo.clone(),
            doc_gen,
        );

        let cmd = BranchWithDocumentCommand {
            parent_cycle_id: parent_id,
            branch_point: ComponentType::IssueRaising,
            branch_label: Some("Alternative Path".to_string()),
            user_id: test_user_id(),
        };

        let result = handler.handle(cmd).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.branch_cycle.is_branch());
        assert_eq!(result.branch_cycle.parent_cycle_id(), Some(parent_id));
        assert!(result.content.contains("Test Decision"));

        // Verify cycle was saved
        assert_eq!(cycle_repo.saved_cycles().len(), 1);

        // Verify document was saved
        let saved_docs = doc_repo.saved_documents();
        assert_eq!(saved_docs.len(), 1);
        assert!(!saved_docs[0].is_branch()); // No parent doc, so not a branch document
    }

    #[tokio::test]
    async fn creates_branch_document_when_parent_document_exists() {
        let session = Session::new(SessionId::new(), test_user_id(), "Test Decision".to_string()).unwrap();
        let session_id = *session.id();
        let parent_cycle = create_parent_cycle_with_started_component(session_id);
        let parent_id = parent_cycle.id();

        // Create parent document
        let parent_doc = DecisionDocument::new(parent_id, test_user_id(), "# Parent Document");

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(parent_cycle));
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let doc_repo = Arc::new(MockDocumentRepository::with_document(parent_doc.clone(), "# Parent".to_string()));
        let doc_gen = Arc::new(MockDocumentGenerator);

        let handler = BranchWithDocumentHandler::new(
            cycle_repo.clone(),
            session_repo,
            doc_repo.clone(),
            doc_gen,
        );

        let cmd = BranchWithDocumentCommand {
            parent_cycle_id: parent_id,
            branch_point: ComponentType::IssueRaising,
            branch_label: Some("Alternative Path".to_string()),
            user_id: test_user_id(),
        };

        let result = handler.handle(cmd).await.unwrap();

        // Verify document is a branch with parent reference
        assert!(result.document.is_branch());
        assert_eq!(result.document.parent_document_id(), Some(parent_doc.id()));
        assert_eq!(result.document.branch_point(), Some(ComponentType::IssueRaising));
        assert_eq!(result.document.branch_label(), Some("Alternative Path"));
    }

    #[tokio::test]
    async fn fails_when_parent_cycle_not_found() {
        let session = Session::new(SessionId::new(), test_user_id(), "Test".to_string()).unwrap();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(Cycle::new(*session.id())));
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let doc_repo = Arc::new(MockDocumentRepository::empty());
        let doc_gen = Arc::new(MockDocumentGenerator);

        let handler = BranchWithDocumentHandler::new(cycle_repo, session_repo, doc_repo, doc_gen);

        let cmd = BranchWithDocumentCommand {
            parent_cycle_id: CycleId::new(), // Non-existent
            branch_point: ComponentType::IssueRaising,
            branch_label: None,
            user_id: test_user_id(),
        };

        let result = handler.handle(cmd).await;

        assert!(matches!(result, Err(BranchWithDocumentError::CycleNotFound(_))));
    }

    #[tokio::test]
    async fn fails_when_branch_point_not_started() {
        let session = Session::new(SessionId::new(), test_user_id(), "Test Decision".to_string()).unwrap();
        let session_id = *session.id();
        let parent_cycle = Cycle::new(session_id); // No components started
        let parent_id = parent_cycle.id();

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(parent_cycle));
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let doc_repo = Arc::new(MockDocumentRepository::empty());
        let doc_gen = Arc::new(MockDocumentGenerator);

        let handler = BranchWithDocumentHandler::new(cycle_repo, session_repo, doc_repo, doc_gen);

        let cmd = BranchWithDocumentCommand {
            parent_cycle_id: parent_id,
            branch_point: ComponentType::IssueRaising,
            branch_label: None,
            user_id: test_user_id(),
        };

        let result = handler.handle(cmd).await;

        assert!(matches!(result, Err(BranchWithDocumentError::Domain(_))));
    }

    #[tokio::test]
    async fn uses_default_branch_label_when_not_provided() {
        let session = Session::new(SessionId::new(), test_user_id(), "Test Decision".to_string()).unwrap();
        let session_id = *session.id();
        let mut parent_cycle = Cycle::new(session_id);
        parent_cycle.start_component(ComponentType::IssueRaising).unwrap();
        parent_cycle.complete_component(ComponentType::IssueRaising).unwrap();
        parent_cycle.start_component(ComponentType::ProblemFrame).unwrap();
        let parent_id = parent_cycle.id();

        let parent_doc = DecisionDocument::new(parent_id, test_user_id(), "# Parent");

        let cycle_repo = Arc::new(MockCycleRepository::with_cycle(parent_cycle));
        let session_repo = Arc::new(MockSessionRepository::with_session(session));
        let doc_repo = Arc::new(MockDocumentRepository::with_document(parent_doc, "# Parent".to_string()));
        let doc_gen = Arc::new(MockDocumentGenerator);

        let handler = BranchWithDocumentHandler::new(cycle_repo, session_repo, doc_repo, doc_gen);

        let cmd = BranchWithDocumentCommand {
            parent_cycle_id: parent_id,
            branch_point: ComponentType::ProblemFrame,
            branch_label: None, // No label provided
            user_id: test_user_id(),
        };

        let result = handler.handle(cmd).await.unwrap();

        // Should use default label
        assert_eq!(result.document.branch_label(), Some("Branch at Problem Frame"));
    }
}
