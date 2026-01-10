//! HTTP handlers for cycle endpoints.
//!
//! These handlers connect Axum routes to application layer command/query handlers.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::application::handlers::cycle::{
    BranchWithDocumentCommand, BranchWithDocumentError, BranchWithDocumentHandler,
    GenerateDocumentCommand, GenerateDocumentError, GenerateDocumentHandler,
    RegenerateDocumentCommand, RegenerateDocumentError, RegenerateDocumentHandler,
    UpdateDocumentFromEditCommand, UpdateDocumentFromEditError, UpdateDocumentFromEditHandler,
};
use crate::domain::foundation::{CommandMetadata, CycleId, DecisionDocumentId, UserId};
use crate::ports::{
    CycleRepository, DecisionDocumentRepository, DocumentExportService, DocumentFormat,
    DocumentGenerator, DocumentParser, ExportFormat, SessionRepository,
};

use super::dto::{
    BranchCycleRequest, BranchCycleResponse, DocumentResponse, ErrorResponse,
    ExportDocumentQuery, GetDocumentQuery, ParseSummaryResponse, RegenerateDocumentResponse,
    UpdateDocumentRequest, UpdateDocumentResponse,
};

// ════════════════════════════════════════════════════════════════════════════════
// Application State
// ════════════════════════════════════════════════════════════════════════════════

/// Shared application state for cycle endpoints.
#[derive(Clone)]
pub struct CycleAppState {
    pub cycle_repository: Arc<dyn CycleRepository>,
    pub session_repository: Arc<dyn SessionRepository>,
    pub document_generator: Arc<dyn DocumentGenerator>,
    pub document_repository: Arc<dyn DecisionDocumentRepository>,
    pub document_parser: Arc<dyn DocumentParser>,
    pub export_service: Arc<dyn DocumentExportService>,
}

impl CycleAppState {
    /// Creates a new app state with all dependencies.
    pub fn new(
        cycle_repository: Arc<dyn CycleRepository>,
        session_repository: Arc<dyn SessionRepository>,
        document_generator: Arc<dyn DocumentGenerator>,
        document_repository: Arc<dyn DecisionDocumentRepository>,
        document_parser: Arc<dyn DocumentParser>,
        export_service: Arc<dyn DocumentExportService>,
    ) -> Self {
        Self {
            cycle_repository,
            session_repository,
            document_generator,
            document_repository,
            document_parser,
            export_service,
        }
    }

    /// Creates the document generation handler.
    pub fn generate_document_handler(&self) -> GenerateDocumentHandler {
        GenerateDocumentHandler::new(
            self.cycle_repository.clone(),
            self.session_repository.clone(),
            self.document_generator.clone(),
        )
    }

    /// Creates the document regeneration handler.
    pub fn regenerate_document_handler(&self) -> RegenerateDocumentHandler {
        RegenerateDocumentHandler::new(
            self.cycle_repository.clone(),
            self.session_repository.clone(),
            self.document_generator.clone(),
            self.document_repository.clone(),
        )
    }

    /// Creates the document update handler.
    pub fn update_document_handler(&self) -> UpdateDocumentFromEditHandler {
        UpdateDocumentFromEditHandler::new(
            self.document_parser.clone(),
            self.document_repository.clone(),
            self.cycle_repository.clone(),
        )
    }

    /// Creates the branch with document handler.
    pub fn branch_with_document_handler(&self) -> BranchWithDocumentHandler {
        BranchWithDocumentHandler::new(
            self.cycle_repository.clone(),
            self.session_repository.clone(),
            self.document_repository.clone(),
            self.document_generator.clone(),
        )
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Handlers
// ════════════════════════════════════════════════════════════════════════════════

/// GET /api/cycles/:id/document
///
/// Generates a decision document from the cycle's current state.
///
/// Query parameters:
/// - `format`: "full" (default), "summary", or "export"
///
/// Returns:
/// - 200: Document content with metadata
/// - 400: Invalid format
/// - 404: Cycle not found
/// - 500: Generation failed
pub async fn get_document(
    State(state): State<CycleAppState>,
    Path(cycle_id): Path<String>,
    Query(query): Query<GetDocumentQuery>,
) -> impl IntoResponse {
    // Parse cycle ID
    let cycle_id = match cycle_id.parse::<CycleId>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request("Invalid cycle ID format")),
            )
                .into_response();
        }
    };

    // Parse format
    let format = match query.format.to_lowercase().as_str() {
        "full" => DocumentFormat::Full,
        "summary" => DocumentFormat::Summary,
        "export" => DocumentFormat::Export,
        other => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request(format!(
                    "Invalid format '{}'. Valid formats: full, summary, export",
                    other
                ))),
            )
                .into_response();
        }
    };

    // Create command based on format
    let cmd = match format {
        DocumentFormat::Full => GenerateDocumentCommand::full(cycle_id),
        DocumentFormat::Summary => GenerateDocumentCommand::summary(cycle_id),
        DocumentFormat::Export => GenerateDocumentCommand::export(cycle_id),
    };

    // TODO: Extract user ID from authentication context
    let user_id = UserId::new("system").unwrap();
    let metadata = CommandMetadata::new(user_id);

    // Execute handler
    let handler = state.generate_document_handler();
    match handler.handle(cmd, metadata).await {
        Ok(result) => {
            let response = DocumentResponse {
                content: result.content,
                cycle_id: result.cycle_id.to_string(),
                session_id: result.session_id.to_string(),
                format: format_to_string(&result.format),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(err) => match err {
            GenerateDocumentError::CycleNotFound(id) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::not_found("Cycle", &id.to_string())),
            )
                .into_response(),
            GenerateDocumentError::SessionNotFound(id) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::internal(format!(
                    "Data integrity error: session {} not found for cycle",
                    id
                ))),
            )
                .into_response(),
            GenerateDocumentError::GenerationFailed(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::internal(format!(
                    "Document generation failed: {}",
                    msg
                ))),
            )
                .into_response(),
            GenerateDocumentError::Domain(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::internal(err.to_string())),
            )
                .into_response(),
        },
    }
}

fn format_to_string(format: &DocumentFormat) -> String {
    match format {
        DocumentFormat::Full => "full".to_string(),
        DocumentFormat::Summary => "summary".to_string(),
        DocumentFormat::Export => "export".to_string(),
    }
}

/// POST /api/cycles/:id/document/regenerate
///
/// Regenerates a decision document from the cycle's current state and persists it.
///
/// Returns:
/// - 200: Regenerated document with metadata
/// - 404: Cycle not found
/// - 500: Generation or persistence failed
pub async fn regenerate_document(
    State(state): State<CycleAppState>,
    Path(cycle_id): Path<String>,
) -> impl IntoResponse {
    // Parse cycle ID
    let cycle_id = match cycle_id.parse::<CycleId>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request("Invalid cycle ID format")),
            )
                .into_response();
        }
    };

    // Create command for full regeneration
    let cmd = RegenerateDocumentCommand::full(cycle_id);

    // TODO: Extract user ID from authentication context
    let user_id = UserId::new("system").unwrap();
    let metadata = CommandMetadata::new(user_id);

    // Execute handler
    let handler = state.regenerate_document_handler();
    match handler.handle(cmd, metadata).await {
        Ok(result) => {
            let response = RegenerateDocumentResponse {
                document_id: result.document_id.to_string(),
                cycle_id: result.cycle_id.to_string(),
                session_id: result.session_id.to_string(),
                version: result.version,
                format: format_to_string(&result.format),
                is_new: result.is_new,
                content: result.content,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(err) => match err {
            RegenerateDocumentError::CycleNotFound(id) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::not_found("Cycle", &id.to_string())),
            )
                .into_response(),
            RegenerateDocumentError::SessionNotFound(id) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::internal(format!(
                    "Data integrity error: session {} not found for cycle",
                    id
                ))),
            )
                .into_response(),
            RegenerateDocumentError::GenerationFailed(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::internal(format!(
                    "Document generation failed: {}",
                    msg
                ))),
            )
                .into_response(),
            RegenerateDocumentError::PersistFailed(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::internal(format!(
                    "Failed to persist document: {}",
                    msg
                ))),
            )
                .into_response(),
            RegenerateDocumentError::Domain(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::internal(err.to_string())),
            )
                .into_response(),
        },
    }
}

/// PUT /api/cycles/:id/document
///
/// Updates a decision document from user edits.
///
/// This endpoint receives edited markdown content and:
/// 1. Parses the markdown to extract structured data
/// 2. Optionally updates cycle component outputs (sync_to_components)
/// 3. Persists the updated document
///
/// Returns:
/// - 200: Document updated successfully
/// - 400: Invalid request (missing content, invalid document ID)
/// - 404: Document not found
/// - 409: Version conflict (concurrent edit)
/// - 500: Parse or persistence failed
pub async fn update_document(
    State(state): State<CycleAppState>,
    Path(document_id): Path<String>,
    Json(request): Json<UpdateDocumentRequest>,
) -> impl IntoResponse {
    // Parse document ID
    let document_id = match document_id.parse::<DecisionDocumentId>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request("Invalid document ID format")),
            )
                .into_response();
        }
    };

    // Validate request
    if request.content.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::bad_request("Content cannot be empty")),
        )
            .into_response();
    }

    // Create command
    let cmd = if request.sync_to_components {
        UpdateDocumentFromEditCommand::sync(document_id, request.content)
    } else {
        UpdateDocumentFromEditCommand::document_only(document_id, request.content)
    };

    // TODO: Extract user ID from authentication context
    let user_id = UserId::new("system").unwrap();
    let metadata = CommandMetadata::new(user_id);

    // Execute handler
    let handler = state.update_document_handler();
    match handler.handle(cmd, metadata).await {
        Ok(result) => {
            let response = UpdateDocumentResponse {
                document_id: result.document_id.to_string(),
                cycle_id: result.cycle_id.to_string(),
                version: result.version,
                components_updated: result.components_updated,
                parse_summary: ParseSummaryResponse {
                    sections_parsed: result.parse_result.sections_parsed,
                    warnings: result.parse_result.warnings,
                    errors: result.parse_result.errors,
                },
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(err) => match err {
            UpdateDocumentFromEditError::DocumentNotFound(id) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::not_found("Document", &id.to_string())),
            )
                .into_response(),
            UpdateDocumentFromEditError::CycleNotFound(id) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::internal(format!(
                    "Data integrity error: cycle {} not found for document",
                    id
                ))),
            )
                .into_response(),
            UpdateDocumentFromEditError::ParseFailed(msg) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request(format!(
                    "Document parse failed: {}",
                    msg
                ))),
            )
                .into_response(),
            UpdateDocumentFromEditError::PersistFailed(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::internal(format!(
                    "Failed to persist document: {}",
                    msg
                ))),
            )
                .into_response(),
            UpdateDocumentFromEditError::VersionConflict { expected, actual } => (
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    code: "VERSION_CONFLICT".to_string(),
                    message: format!(
                        "Document was modified: expected version {}, found {}",
                        expected, actual
                    ),
                    details: None,
                }),
            )
                .into_response(),
            UpdateDocumentFromEditError::Domain(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::internal(err.to_string())),
            )
                .into_response(),
        },
    }
}

/// POST /api/cycles/:id/branch
///
/// Branches a cycle at a specified component and creates a document for the branch.
///
/// Request body:
/// - `branch_point`: The component type where branching occurs
///
/// Returns:
/// - 201: Branch created successfully with document
/// - 400: Invalid request (bad cycle ID or branch point)
/// - 404: Cycle not found
/// - 500: Branching or document creation failed
pub async fn branch_cycle(
    State(state): State<CycleAppState>,
    Path(cycle_id): Path<String>,
    Json(request): Json<BranchCycleRequest>,
) -> impl IntoResponse {
    // Parse cycle ID
    let cycle_id = match cycle_id.parse::<CycleId>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request("Invalid cycle ID format")),
            )
                .into_response();
        }
    };

    // TODO: Extract user ID from authentication context
    let user_id = UserId::new("system").unwrap();

    // Create command
    let cmd = BranchWithDocumentCommand {
        parent_cycle_id: cycle_id,
        branch_point: request.branch_point,
        branch_label: None, // Could be added to request DTO later
        user_id,
    };

    // Execute handler
    let handler = state.branch_with_document_handler();
    match handler.handle(cmd).await {
        Ok(result) => {
            let branch_label = result
                .document
                .branch_label()
                .unwrap_or("Branch")
                .to_string();

            let response = BranchCycleResponse {
                cycle_id: result.branch_cycle.id().to_string(),
                parent_cycle_id: cycle_id.to_string(),
                document_id: result.document.id().to_string(),
                branch_point: request.branch_point,
                branch_label,
                content: result.content,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(err) => match err {
            BranchWithDocumentError::CycleNotFound(id) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::not_found("Cycle", &id.to_string())),
            )
                .into_response(),
            BranchWithDocumentError::ParentDocumentNotFound(id) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::internal(format!(
                    "Parent document not found for cycle: {}",
                    id
                ))),
            )
                .into_response(),
            BranchWithDocumentError::SessionNotFound => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::internal("Session not found for cycle")),
            )
                .into_response(),
            BranchWithDocumentError::Domain(err) => {
                // Domain errors are typically validation failures (e.g., branch point not started)
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::bad_request(err.to_string())),
                )
                    .into_response()
            }
            BranchWithDocumentError::GenerationFailed(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::internal(format!(
                    "Document generation failed: {}",
                    msg
                ))),
            )
                .into_response(),
        },
    }
}

/// GET /api/cycles/:id/document/export
///
/// Exports the decision document in various formats (markdown, PDF, HTML).
///
/// Query parameters:
/// - `format`: "markdown" (default), "pdf", or "html"
///
/// Returns:
/// - 200: Exported file content with appropriate Content-Type header
/// - 400: Invalid format or cycle ID
/// - 404: Cycle not found
/// - 500: Export conversion failed
/// - 503: Export service unavailable (e.g., Pandoc not installed for PDF)
pub async fn export_document(
    State(state): State<CycleAppState>,
    Path(cycle_id): Path<String>,
    Query(query): Query<ExportDocumentQuery>,
) -> impl IntoResponse {
    use axum::http::header;
    use crate::ports::ExportedDocument;

    // Parse cycle ID
    let cycle_id = match cycle_id.parse::<CycleId>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request("Invalid cycle ID format")),
            )
                .into_response();
        }
    };

    // Parse export format
    let format = match query.format.parse::<ExportFormat>() {
        Ok(f) => f,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request(format!(
                    "Invalid export format '{}'. Valid formats: markdown, pdf, html",
                    query.format
                ))),
            )
                .into_response();
        }
    };

    // First, generate the markdown content
    let cmd = GenerateDocumentCommand::export(cycle_id);
    let user_id = UserId::new("system").unwrap();
    let metadata = CommandMetadata::new(user_id);

    let handler = state.generate_document_handler();
    let generate_result = match handler.handle(cmd, metadata).await {
        Ok(result) => result,
        Err(err) => {
            return match err {
                GenerateDocumentError::CycleNotFound(id) => (
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse::not_found("Cycle", &id.to_string())),
                )
                    .into_response(),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::internal(format!(
                        "Document generation failed: {}",
                        err
                    ))),
                )
                    .into_response(),
            };
        }
    };

    // Convert to requested format
    let base_filename = format!("decision-{}", cycle_id);
    let exported = match format {
        ExportFormat::Markdown => {
            ExportedDocument::from_markdown(generate_result.content, &base_filename)
        }
        ExportFormat::Html => {
            match state.export_service.to_html(&generate_result.content).await {
                Ok(html) => ExportedDocument::from_html(html, &base_filename),
                Err(err) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::internal(format!(
                            "HTML conversion failed: {}",
                            err
                        ))),
                    )
                        .into_response();
                }
            }
        }
        ExportFormat::Pdf => {
            match state.export_service.to_pdf(&generate_result.content).await {
                Ok(pdf) => ExportedDocument::from_pdf(pdf, &base_filename),
                Err(crate::ports::ExportError::ServiceUnavailable(msg)) => {
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        Json(ErrorResponse {
                            code: "SERVICE_UNAVAILABLE".to_string(),
                            message: msg,
                            details: None,
                        }),
                    )
                        .into_response();
                }
                Err(err) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::internal(format!(
                            "PDF conversion failed: {}",
                            err
                        ))),
                    )
                        .into_response();
                }
            }
        }
    };

    // Return file with appropriate headers
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, exported.content_type.as_str()),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{}\"", exported.filename),
            ),
        ],
        exported.content,
    )
        .into_response()
}

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cycle::{Cycle, DecisionDocument};
    use crate::domain::foundation::{DecisionDocumentId, DomainError, SessionId};
    use crate::domain::session::Session;
    use crate::ports::{DocumentError, GenerationOptions, IntegrityStatus, SyncResult};
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get;
    use axum::Router;
    use std::sync::Mutex;
    use tower::ServiceExt;

    // ───────────────────────────────────────────────────────────────
    // Mock implementations
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

        fn empty() -> Self {
            Self {
                cycles: Mutex::new(vec![]),
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
        fn with_session(session: Session) -> Self {
            Self {
                sessions: Mutex::new(vec![session]),
            }
        }

        fn empty() -> Self {
            Self {
                sessions: Mutex::new(vec![]),
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
    }

    impl MockDocumentGenerator {
        fn new(content: &str) -> Self {
            Self {
                content: content.to_string(),
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

    struct MockExportService;

    #[async_trait]
    impl DocumentExportService for MockExportService {
        async fn to_pdf(&self, _markdown: &str) -> Result<Vec<u8>, crate::ports::ExportError> {
            Ok(vec![0x25, 0x50, 0x44, 0x46]) // PDF magic bytes
        }

        async fn to_html(&self, markdown: &str) -> Result<String, crate::ports::ExportError> {
            Ok(format!("<html><body>{}</body></html>", markdown))
        }

        async fn is_available(&self) -> bool {
            true
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

    fn create_app(state: CycleAppState) -> Router {
        Router::new()
            .route("/api/cycles/:id/document", get(get_document))
            .with_state(state)
    }

    // ───────────────────────────────────────────────────────────────
    // Tests
    // ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn get_document_returns_200_for_valid_cycle() {
        let session = test_session();
        let session_id = *session.id();
        let cycle = test_cycle(session_id);
        let cycle_id = cycle.id();

        let state = CycleAppState::new(
            Arc::new(MockCycleRepository::with_cycle(cycle)),
            Arc::new(MockSessionRepository::with_session(session)),
            Arc::new(MockDocumentGenerator::new("Test content")),
            Arc::new(MockDocumentRepository),
            Arc::new(MockDocumentParser),
            Arc::new(MockExportService),
        );

        let app = create_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/cycles/{}/document", cycle_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn get_document_returns_404_for_missing_cycle() {
        let session = test_session();
        let missing_id = CycleId::new();

        let state = CycleAppState::new(
            Arc::new(MockCycleRepository::empty()),
            Arc::new(MockSessionRepository::with_session(session)),
            Arc::new(MockDocumentGenerator::new("Test")),
            Arc::new(MockDocumentRepository),
            Arc::new(MockDocumentParser),
            Arc::new(MockExportService),
        );

        let app = create_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/cycles/{}/document", missing_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn get_document_returns_400_for_invalid_format() {
        let session = test_session();
        let session_id = *session.id();
        let cycle = test_cycle(session_id);
        let cycle_id = cycle.id();

        let state = CycleAppState::new(
            Arc::new(MockCycleRepository::with_cycle(cycle)),
            Arc::new(MockSessionRepository::with_session(session)),
            Arc::new(MockDocumentGenerator::new("Test")),
            Arc::new(MockDocumentRepository),
            Arc::new(MockDocumentParser),
            Arc::new(MockExportService),
        );

        let app = create_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/cycles/{}/document?format=invalid", cycle_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_document_accepts_summary_format() {
        let session = test_session();
        let session_id = *session.id();
        let cycle = test_cycle(session_id);
        let cycle_id = cycle.id();

        let state = CycleAppState::new(
            Arc::new(MockCycleRepository::with_cycle(cycle)),
            Arc::new(MockSessionRepository::with_session(session)),
            Arc::new(MockDocumentGenerator::new("Test")),
            Arc::new(MockDocumentRepository),
            Arc::new(MockDocumentParser),
            Arc::new(MockExportService),
        );

        let app = create_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/cycles/{}/document?format=summary", cycle_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn get_document_accepts_export_format() {
        let session = test_session();
        let session_id = *session.id();
        let cycle = test_cycle(session_id);
        let cycle_id = cycle.id();

        let state = CycleAppState::new(
            Arc::new(MockCycleRepository::with_cycle(cycle)),
            Arc::new(MockSessionRepository::with_session(session)),
            Arc::new(MockDocumentGenerator::new("Test")),
            Arc::new(MockDocumentRepository),
            Arc::new(MockDocumentParser),
            Arc::new(MockExportService),
        );

        let app = create_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/cycles/{}/document?format=export", cycle_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
