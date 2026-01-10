//! Route configuration for cycle endpoints.
//!
//! Configures Axum router with cycle-related routes.

use axum::routing::{get, post, put};
use axum::Router;

use super::handlers::{
    branch_cycle, get_document, regenerate_document, update_document, CycleAppState,
};

/// Creates the cycle router with all endpoints.
///
/// Routes:
/// - `GET /api/cycles/:id/document` - Generate decision document
/// - `GET /api/cycles/:id/document?format=summary` - Generate summary document
/// - `GET /api/cycles/:id/document?format=export` - Generate export document
/// - `POST /api/cycles/:id/document/regenerate` - Regenerate and persist document
/// - `POST /api/cycles/:id/branch` - Branch cycle at a component
/// - `PUT /api/documents/:id` - Update document from user edit
pub fn cycle_router() -> Router<CycleAppState> {
    Router::new()
        .route("/api/cycles/:id/document", get(get_document))
        .route("/api/cycles/:id/document/regenerate", post(regenerate_document))
        .route("/api/cycles/:id/branch", post(branch_cycle))
        .route("/api/documents/:id", put(update_document))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cycle::{Cycle, DecisionDocument};
    use crate::domain::foundation::{CycleId, DecisionDocumentId, DomainError, SessionId, UserId};
    use crate::domain::session::Session;
    use crate::ports::{
        CycleRepository, DecisionDocumentRepository, DocumentError, DocumentGenerator,
        DocumentParser, GenerationOptions, IntegrityStatus, SessionRepository, SyncResult,
    };
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::Request;
    use std::sync::Arc;
    use std::sync::Mutex;
    use tower::ServiceExt;

    // ───────────────────────────────────────────────────────────────
    // Mock implementations (minimal for route testing)
    // ───────────────────────────────────────────────────────────────

    struct MockCycleRepository {
        cycles: Mutex<Vec<Cycle>>,
    }

    impl MockCycleRepository {
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

        async fn find_by_id(
            &self,
            id: &crate::domain::foundation::CycleId,
        ) -> Result<Option<Cycle>, DomainError> {
            Ok(self
                .cycles
                .lock()
                .unwrap()
                .iter()
                .find(|c| c.id() == *id)
                .cloned())
        }

        async fn exists(
            &self,
            _id: &crate::domain::foundation::CycleId,
        ) -> Result<bool, DomainError> {
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

        async fn find_branches(
            &self,
            _parent_id: &crate::domain::foundation::CycleId,
        ) -> Result<Vec<Cycle>, DomainError> {
            Ok(vec![])
        }

        async fn count_by_session_id(&self, _session_id: &SessionId) -> Result<u32, DomainError> {
            Ok(0)
        }

        async fn delete(&self, _id: &crate::domain::foundation::CycleId) -> Result<(), DomainError> {
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

    struct MockDocumentGenerator;

    impl DocumentGenerator for MockDocumentGenerator {
        fn generate(
            &self,
            session_title: &str,
            _cycle: &Cycle,
            _options: GenerationOptions,
        ) -> Result<String, DocumentError> {
            Ok(format!("# {}\n\nGenerated content", session_title))
        }

        fn generate_section(
            &self,
            _component_type: crate::domain::foundation::ComponentType,
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

    struct MockDocumentRepository;

    #[async_trait]
    impl DecisionDocumentRepository for MockDocumentRepository {
        async fn save(&self, _document: &DecisionDocument, _content: &str) -> Result<(), DomainError> {
            Ok(())
        }

        async fn update(&self, _document: &DecisionDocument, _content: &str) -> Result<(), DomainError> {
            Ok(())
        }

        async fn find_by_id(
            &self,
            _id: DecisionDocumentId,
        ) -> Result<Option<DecisionDocument>, DomainError> {
            Ok(None)
        }

        async fn find_by_cycle(&self, _cycle_id: CycleId) -> Result<Option<DecisionDocument>, DomainError> {
            Ok(None)
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

    struct MockDocumentParser;

    impl DocumentParser for MockDocumentParser {
        fn parse(&self, _content: &str) -> Result<crate::ports::ParseResult, DocumentError> {
            Ok(crate::ports::ParseResult::empty())
        }

        fn parse_section(
            &self,
            _section_content: &str,
            component_type: crate::domain::foundation::ComponentType,
        ) -> Result<crate::domain::cycle::ParsedSection, DocumentError> {
            Ok(crate::domain::cycle::ParsedSection::success(
                component_type,
                "test".to_string(),
                serde_json::json!({}),
            ))
        }

        fn validate_structure(
            &self,
            _content: &str,
        ) -> Result<Vec<crate::domain::cycle::ParseError>, DocumentError> {
            Ok(vec![])
        }

        fn extract_section_boundaries(
            &self,
            _content: &str,
        ) -> Vec<crate::ports::SectionBoundary> {
            vec![]
        }
    }

    // ───────────────────────────────────────────────────────────────
    // Tests
    // ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn cycle_router_mounts_document_endpoint() {
        let session =
            Session::new(SessionId::new(), UserId::new("test").unwrap(), "Test".to_string())
                .unwrap();
        let session_id = *session.id();
        let cycle = Cycle::new(session_id);
        let cycle_id = cycle.id();

        let state = CycleAppState::new(
            Arc::new(MockCycleRepository::with_cycle(cycle)),
            Arc::new(MockSessionRepository::with_session(session)),
            Arc::new(MockDocumentGenerator),
            Arc::new(MockDocumentRepository),
            Arc::new(MockDocumentParser),
        );

        let app = cycle_router().with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/cycles/{}/document", cycle_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }
}
