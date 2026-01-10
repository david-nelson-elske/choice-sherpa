//! HTTP handlers for cycle endpoints.
//!
//! These handlers connect Axum routes to application layer command/query handlers.

use std::sync::Arc;

use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::application::handlers::cycle::{
    ArchiveCycleCommand, ArchiveCycleError, ArchiveCycleHandler, BranchCycleCommand,
    BranchCycleError, BranchCycleHandler, CompleteComponentCommand, CompleteComponentError,
    CompleteComponentHandler, CompleteCycleCommand, CompleteCycleError, CompleteCycleHandler,
    CreateCycleCommand, CreateCycleError, CreateCycleHandler, GetComponentError,
    GetComponentHandler, GetComponentQuery, GetCycleError, GetCycleHandler, GetCycleQuery,
    GetCycleTreeError, GetCycleTreeHandler, GetCycleTreeQuery, NavigateComponentCommand,
    NavigateComponentError, NavigateComponentHandler, StartComponentCommand, StartComponentError,
    StartComponentHandler, UpdateComponentOutputCommand, UpdateComponentOutputError,
    UpdateComponentOutputHandler,
};
use crate::domain::foundation::{CycleId, SessionId, UserId};
use crate::ports::{AccessChecker, CycleReader, CycleRepository, EventPublisher, SessionRepository};

use super::dto::{
    BranchCycleRequest, CompleteComponentRequest, ComponentCommandResponse, ComponentResponse,
    CreateCycleRequest, CycleCommandResponse, CycleResponse, CycleTreeResponse, ErrorResponse,
    NavigateComponentRequest, StartComponentRequest, UpdateComponentOutputRequest,
};

// ════════════════════════════════════════════════════════════════════════════════
// Application State
// ════════════════════════════════════════════════════════════════════════════════

/// Shared application state containing all dependencies.
#[derive(Clone)]
pub struct CycleAppState {
    pub cycle_repository: Arc<dyn CycleRepository>,
    pub cycle_reader: Arc<dyn CycleReader>,
    pub session_repository: Arc<dyn SessionRepository>,
    pub access_checker: Arc<dyn AccessChecker>,
    pub event_publisher: Arc<dyn EventPublisher>,
}

impl CycleAppState {
    pub fn create_cycle_handler(&self) -> CreateCycleHandler {
        CreateCycleHandler::new(
            self.cycle_repository.clone(),
            self.session_repository.clone(),
            self.access_checker.clone(),
            self.event_publisher.clone(),
        )
    }

    pub fn branch_cycle_handler(&self) -> BranchCycleHandler {
        BranchCycleHandler::new(self.cycle_repository.clone(), self.event_publisher.clone())
    }

    pub fn archive_cycle_handler(&self) -> ArchiveCycleHandler {
        ArchiveCycleHandler::new(self.cycle_repository.clone(), self.event_publisher.clone())
    }

    pub fn complete_cycle_handler(&self) -> CompleteCycleHandler {
        CompleteCycleHandler::new(self.cycle_repository.clone(), self.event_publisher.clone())
    }

    pub fn start_component_handler(&self) -> StartComponentHandler {
        StartComponentHandler::new(self.cycle_repository.clone(), self.event_publisher.clone())
    }

    pub fn complete_component_handler(&self) -> CompleteComponentHandler {
        CompleteComponentHandler::new(self.cycle_repository.clone(), self.event_publisher.clone())
    }

    pub fn update_component_output_handler(&self) -> UpdateComponentOutputHandler {
        UpdateComponentOutputHandler::new(self.cycle_repository.clone(), self.event_publisher.clone())
    }

    pub fn navigate_component_handler(&self) -> NavigateComponentHandler {
        NavigateComponentHandler::new(self.cycle_repository.clone(), self.event_publisher.clone())
    }

    pub fn get_cycle_handler(&self) -> GetCycleHandler {
        GetCycleHandler::new(self.cycle_reader.clone())
    }

    pub fn get_cycle_tree_handler(&self) -> GetCycleTreeHandler {
        GetCycleTreeHandler::new(self.cycle_reader.clone())
    }

    pub fn get_component_handler(&self) -> GetComponentHandler {
        GetComponentHandler::new(self.cycle_repository.clone())
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// User Context
// ════════════════════════════════════════════════════════════════════════════════

/// Authenticated user context extracted from request.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: UserId,
}

/// Rejection type for AuthenticatedUser extraction.
pub struct AuthenticationRequired;

impl IntoResponse for AuthenticationRequired {
    fn into_response(self) -> axum::response::Response {
        let error = ErrorResponse::bad_request("Authentication is required");
        (StatusCode::UNAUTHORIZED, Json(error)).into_response()
    }
}

impl<S> axum::extract::FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthenticationRequired;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut axum::http::request::Parts,
        _state: &'life1 S,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let user_id = parts
                .headers
                .get("X-User-Id")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| UserId::new(s).ok())
                .ok_or(AuthenticationRequired)?;

            Ok(AuthenticatedUser { user_id })
        })
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Query Handlers (GET endpoints)
// ════════════════════════════════════════════════════════════════════════════════

/// GET /api/cycles/:id - Get cycle details
pub async fn get_cycle(
    State(state): State<CycleAppState>,
    Path(cycle_id): Path<String>,
    _user: AuthenticatedUser,
) -> Result<impl IntoResponse, CycleApiError> {
    let cycle_id: CycleId = cycle_id
        .parse()
        .map_err(|_| CycleApiError::BadRequest("Invalid cycle ID format".to_string()))?;

    let handler = state.get_cycle_handler();
    let query = GetCycleQuery { cycle_id };

    let result = handler.handle(query).await?;
    let response = CycleResponse::from(result);

    Ok(Json(response))
}

/// GET /api/sessions/:session_id/cycles/tree - Get cycle tree for session
pub async fn get_cycle_tree(
    State(state): State<CycleAppState>,
    Path(session_id): Path<String>,
    _user: AuthenticatedUser,
) -> Result<impl IntoResponse, CycleApiError> {
    let session_id: SessionId = session_id
        .parse()
        .map_err(|_| CycleApiError::BadRequest("Invalid session ID format".to_string()))?;

    let handler = state.get_cycle_tree_handler();
    let query = GetCycleTreeQuery { session_id };

    let result = handler.handle(query).await?;
    let response = CycleTreeResponse::from(result);

    Ok(Json(response))
}

/// GET /api/cycles/:cycle_id/components/:component_type - Get component details
pub async fn get_component(
    State(state): State<CycleAppState>,
    Path((cycle_id, component_type)): Path<(String, String)>,
    _user: AuthenticatedUser,
) -> Result<impl IntoResponse, CycleApiError> {
    let cycle_id: CycleId = cycle_id
        .parse()
        .map_err(|_| CycleApiError::BadRequest("Invalid cycle ID format".to_string()))?;
    let component_type = serde_json::from_str(&format!("\"{}\"", component_type))
        .map_err(|_| CycleApiError::BadRequest("Invalid component type".to_string()))?;

    let handler = state.get_component_handler();
    let query = GetComponentQuery {
        cycle_id,
        component_type,
    };

    let result = handler.handle(query).await?;
    let response = ComponentResponse {
        cycle_id: result.cycle_id.to_string(),
        component_type: result.component_type,
        status: result.status,
        output: result.output,
    };

    Ok(Json(response))
}

// ════════════════════════════════════════════════════════════════════════════════
// Command Handlers (POST endpoints)
// ════════════════════════════════════════════════════════════════════════════════

/// POST /api/cycles - Create a new cycle
pub async fn create_cycle(
    State(state): State<CycleAppState>,
    _user: AuthenticatedUser,
    Json(request): Json<CreateCycleRequest>,
) -> Result<impl IntoResponse, CycleApiError> {
    let session_id: SessionId = request
        .session_id
        .parse()
        .map_err(|_| CycleApiError::BadRequest("Invalid session ID format".to_string()))?;

    let handler = state.create_cycle_handler();
    let cmd = CreateCycleCommand { session_id };

    let result = handler.handle(cmd).await?;

    let response = CycleCommandResponse {
        cycle_id: result.cycle.id().to_string(),
        message: "Cycle created successfully".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// POST /api/cycles/:id/branch - Branch a cycle
pub async fn branch_cycle(
    State(state): State<CycleAppState>,
    Path(cycle_id): Path<String>,
    _user: AuthenticatedUser,
    Json(request): Json<BranchCycleRequest>,
) -> Result<impl IntoResponse, CycleApiError> {
    let cycle_id: CycleId = cycle_id
        .parse()
        .map_err(|_| CycleApiError::BadRequest("Invalid cycle ID format".to_string()))?;

    let handler = state.branch_cycle_handler();
    let cmd = BranchCycleCommand {
        parent_cycle_id: cycle_id,
        branch_point: request.branch_point,
    };

    let result = handler.handle(cmd).await?;

    let response = CycleCommandResponse {
        cycle_id: result.branch.id().to_string(),
        message: format!("Branched at {:?}", result.event.branch_point),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// POST /api/cycles/:id/archive - Archive a cycle
pub async fn archive_cycle(
    State(state): State<CycleAppState>,
    Path(cycle_id): Path<String>,
    _user: AuthenticatedUser,
) -> Result<impl IntoResponse, CycleApiError> {
    let cycle_id: CycleId = cycle_id
        .parse()
        .map_err(|_| CycleApiError::BadRequest("Invalid cycle ID format".to_string()))?;

    let handler = state.archive_cycle_handler();
    let cmd = ArchiveCycleCommand { cycle_id };

    handler.handle(cmd).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/cycles/:id/complete - Complete a cycle
pub async fn complete_cycle(
    State(state): State<CycleAppState>,
    Path(cycle_id): Path<String>,
    _user: AuthenticatedUser,
) -> Result<impl IntoResponse, CycleApiError> {
    let cycle_id: CycleId = cycle_id
        .parse()
        .map_err(|_| CycleApiError::BadRequest("Invalid cycle ID format".to_string()))?;

    let handler = state.complete_cycle_handler();
    let cmd = CompleteCycleCommand { cycle_id };

    handler.handle(cmd).await?;

    let response = CycleCommandResponse {
        cycle_id: cycle_id.to_string(),
        message: "Cycle completed successfully".to_string(),
    };

    Ok(Json(response))
}

/// POST /api/cycles/:id/components/start - Start a component
pub async fn start_component(
    State(state): State<CycleAppState>,
    Path(cycle_id): Path<String>,
    _user: AuthenticatedUser,
    Json(request): Json<StartComponentRequest>,
) -> Result<impl IntoResponse, CycleApiError> {
    let cycle_id: CycleId = cycle_id
        .parse()
        .map_err(|_| CycleApiError::BadRequest("Invalid cycle ID format".to_string()))?;

    let handler = state.start_component_handler();
    let cmd = StartComponentCommand {
        cycle_id,
        component_type: request.component_type,
    };

    let result = handler.handle(cmd).await?;

    let response = ComponentCommandResponse {
        cycle_id: cycle_id.to_string(),
        component_type: result.event.component_type,
        message: format!("{:?} started", result.event.component_type),
    };

    Ok(Json(response))
}

/// POST /api/cycles/:id/components/complete - Complete a component
pub async fn complete_component(
    State(state): State<CycleAppState>,
    Path(cycle_id): Path<String>,
    _user: AuthenticatedUser,
    Json(request): Json<CompleteComponentRequest>,
) -> Result<impl IntoResponse, CycleApiError> {
    let cycle_id: CycleId = cycle_id
        .parse()
        .map_err(|_| CycleApiError::BadRequest("Invalid cycle ID format".to_string()))?;

    let handler = state.complete_component_handler();
    let cmd = CompleteComponentCommand {
        cycle_id,
        component_type: request.component_type,
    };

    let result = handler.handle(cmd).await?;

    let response = ComponentCommandResponse {
        cycle_id: cycle_id.to_string(),
        component_type: result.event.component_type,
        message: format!("{:?} completed", result.event.component_type),
    };

    Ok(Json(response))
}

/// POST /api/cycles/:id/components/output - Update component output
pub async fn update_component_output(
    State(state): State<CycleAppState>,
    Path(cycle_id): Path<String>,
    _user: AuthenticatedUser,
    Json(request): Json<UpdateComponentOutputRequest>,
) -> Result<impl IntoResponse, CycleApiError> {
    let cycle_id: CycleId = cycle_id
        .parse()
        .map_err(|_| CycleApiError::BadRequest("Invalid cycle ID format".to_string()))?;

    let handler = state.update_component_output_handler();
    let cmd = UpdateComponentOutputCommand {
        cycle_id,
        component_type: request.component_type,
        output: request.output,
    };

    let result = handler.handle(cmd).await?;

    let response = ComponentCommandResponse {
        cycle_id: cycle_id.to_string(),
        component_type: result.event.component_type,
        message: format!("{:?} output updated", result.event.component_type),
    };

    Ok(Json(response))
}

/// POST /api/cycles/:id/components/navigate - Navigate to a component
pub async fn navigate_component(
    State(state): State<CycleAppState>,
    Path(cycle_id): Path<String>,
    _user: AuthenticatedUser,
    Json(request): Json<NavigateComponentRequest>,
) -> Result<impl IntoResponse, CycleApiError> {
    let cycle_id: CycleId = cycle_id
        .parse()
        .map_err(|_| CycleApiError::BadRequest("Invalid cycle ID format".to_string()))?;

    let handler = state.navigate_component_handler();
    let cmd = NavigateComponentCommand {
        cycle_id,
        target: request.target,
    };

    let result = handler.handle(cmd).await?;

    let response = ComponentCommandResponse {
        cycle_id: cycle_id.to_string(),
        component_type: result.event.target,
        message: format!("Navigated to {:?}", result.event.target),
    };

    Ok(Json(response))
}

// ════════════════════════════════════════════════════════════════════════════════
// Error Handling
// ════════════════════════════════════════════════════════════════════════════════

/// API error type that converts domain errors to HTTP responses.
#[derive(Debug)]
pub enum CycleApiError {
    BadRequest(String),
    NotFound(String),
    Forbidden(String),
    Conflict(String),
    Internal(String),
}

impl From<CreateCycleError> for CycleApiError {
    fn from(err: CreateCycleError) -> Self {
        match err {
            CreateCycleError::SessionNotFound(id) => {
                CycleApiError::NotFound(format!("Session not found: {}", id))
            }
            CreateCycleError::AccessDenied(reason) => {
                CycleApiError::Forbidden(format!("Access denied: {:?}", reason))
            }
            CreateCycleError::Domain(e) => CycleApiError::Internal(e.to_string()),
        }
    }
}

impl From<BranchCycleError> for CycleApiError {
    fn from(err: BranchCycleError) -> Self {
        match err {
            BranchCycleError::CycleNotFound(id) => {
                CycleApiError::NotFound(format!("Cycle not found: {}", id))
            }
            BranchCycleError::AccessDenied(reason) => {
                CycleApiError::Forbidden(format!("Access denied: {:?}", reason))
            }
            BranchCycleError::Domain(e) => CycleApiError::BadRequest(e.to_string()),
        }
    }
}

impl From<ArchiveCycleError> for CycleApiError {
    fn from(err: ArchiveCycleError) -> Self {
        match err {
            ArchiveCycleError::CycleNotFound(id) => {
                CycleApiError::NotFound(format!("Cycle not found: {}", id))
            }
            ArchiveCycleError::Domain(e) => CycleApiError::Conflict(e.to_string()),
        }
    }
}

impl From<CompleteCycleError> for CycleApiError {
    fn from(err: CompleteCycleError) -> Self {
        match err {
            CompleteCycleError::CycleNotFound(id) => {
                CycleApiError::NotFound(format!("Cycle not found: {}", id))
            }
            CompleteCycleError::Domain(e) => CycleApiError::BadRequest(e.to_string()),
        }
    }
}

impl From<StartComponentError> for CycleApiError {
    fn from(err: StartComponentError) -> Self {
        match err {
            StartComponentError::CycleNotFound(id) => {
                CycleApiError::NotFound(format!("Cycle not found: {}", id))
            }
            StartComponentError::Domain(e) => CycleApiError::Conflict(e.to_string()),
        }
    }
}

impl From<CompleteComponentError> for CycleApiError {
    fn from(err: CompleteComponentError) -> Self {
        match err {
            CompleteComponentError::CycleNotFound(id) => {
                CycleApiError::NotFound(format!("Cycle not found: {}", id))
            }
            CompleteComponentError::Domain(e) => CycleApiError::Conflict(e.to_string()),
        }
    }
}

impl From<UpdateComponentOutputError> for CycleApiError {
    fn from(err: UpdateComponentOutputError) -> Self {
        match err {
            UpdateComponentOutputError::CycleNotFound(id) => {
                CycleApiError::NotFound(format!("Cycle not found: {}", id))
            }
            UpdateComponentOutputError::Domain(e) => CycleApiError::BadRequest(e.to_string()),
        }
    }
}

impl From<NavigateComponentError> for CycleApiError {
    fn from(err: NavigateComponentError) -> Self {
        match err {
            NavigateComponentError::CycleNotFound(id) => {
                CycleApiError::NotFound(format!("Cycle not found: {}", id))
            }
            NavigateComponentError::Domain(e) => CycleApiError::BadRequest(e.to_string()),
        }
    }
}

impl From<GetCycleError> for CycleApiError {
    fn from(err: GetCycleError) -> Self {
        match err {
            GetCycleError::NotFound(id) => {
                CycleApiError::NotFound(format!("Cycle not found: {}", id))
            }
            GetCycleError::Infrastructure(msg) => CycleApiError::Internal(msg),
        }
    }
}

impl From<GetCycleTreeError> for CycleApiError {
    fn from(err: GetCycleTreeError) -> Self {
        match err {
            GetCycleTreeError::NoCycles(id) => {
                CycleApiError::NotFound(format!("No cycles found for session: {}", id))
            }
            GetCycleTreeError::Infrastructure(msg) => CycleApiError::Internal(msg),
        }
    }
}

impl From<GetComponentError> for CycleApiError {
    fn from(err: GetComponentError) -> Self {
        match err {
            GetComponentError::CycleNotFound(id) => {
                CycleApiError::NotFound(format!("Cycle not found: {}", id))
            }
            GetComponentError::ComponentNotFound(cycle_id, ct) => {
                CycleApiError::NotFound(format!("Component {:?} not found in cycle {}", ct, cycle_id))
            }
            GetComponentError::Infrastructure(msg) => CycleApiError::Internal(msg),
        }
    }
}

impl IntoResponse for CycleApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error) = match self {
            CycleApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, ErrorResponse::bad_request(msg)),
            CycleApiError::NotFound(msg) => (StatusCode::NOT_FOUND, ErrorResponse::not_found("Resource", &msg)),
            CycleApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, ErrorResponse::forbidden(msg)),
            CycleApiError::Conflict(msg) => (StatusCode::CONFLICT, ErrorResponse {
                code: "CONFLICT".to_string(),
                message: msg,
                details: None,
            }),
            CycleApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, ErrorResponse::internal(msg)),
        };

        (status, Json(error)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cycle::Cycle;
    use crate::domain::foundation::{ComponentStatus, ComponentType, CycleStatus, DomainError, Timestamp};
    use crate::domain::membership::TierLimits;
    use crate::domain::session::Session;
    use crate::ports::{
        AccessResult, ComponentStatusItem, CycleProgressView, CycleSummary,
        CycleTreeNode, CycleView, UsageStats,
    };
    use async_trait::async_trait;
    use std::sync::Mutex;

    // ════════════════════════════════════════════════════════════════════════════
    // Mock Implementations
    // ════════════════════════════════════════════════════════════════════════════

    struct MockCycleRepository {
        cycles: Mutex<Vec<Cycle>>,
    }

    impl MockCycleRepository {
        fn new() -> Self {
            Self { cycles: Mutex::new(Vec::new()) }
        }
    }

    #[async_trait]
    impl CycleRepository for MockCycleRepository {
        async fn save(&self, cycle: &Cycle) -> Result<(), DomainError> {
            self.cycles.lock().unwrap().push(cycle.clone());
            Ok(())
        }
        async fn update(&self, _cycle: &Cycle) -> Result<(), DomainError> { Ok(()) }
        async fn find_by_id(&self, id: &CycleId) -> Result<Option<Cycle>, DomainError> {
            Ok(self.cycles.lock().unwrap().iter().find(|c| c.id() == *id).cloned())
        }
        async fn exists(&self, _id: &CycleId) -> Result<bool, DomainError> { Ok(false) }
        async fn find_by_session_id(&self, _session_id: &SessionId) -> Result<Vec<Cycle>, DomainError> { Ok(vec![]) }
        async fn find_primary_by_session_id(&self, _session_id: &SessionId) -> Result<Option<Cycle>, DomainError> { Ok(None) }
        async fn find_branches(&self, _parent_id: &CycleId) -> Result<Vec<Cycle>, DomainError> { Ok(vec![]) }
        async fn count_by_session_id(&self, _session_id: &SessionId) -> Result<u32, DomainError> { Ok(0) }
        async fn delete(&self, _id: &CycleId) -> Result<(), DomainError> { Ok(()) }
    }

    struct MockCycleReader {
        views: Mutex<Vec<CycleView>>,
    }

    impl MockCycleReader {
        fn new() -> Self {
            Self { views: Mutex::new(Vec::new()) }
        }

        fn with_view(view: CycleView) -> Self {
            Self { views: Mutex::new(vec![view]) }
        }
    }

    #[async_trait]
    impl CycleReader for MockCycleReader {
        async fn get_by_id(&self, id: &CycleId) -> Result<Option<CycleView>, DomainError> {
            Ok(self.views.lock().unwrap().iter().find(|v| v.id == *id).cloned())
        }
        async fn list_by_session_id(&self, _session_id: &SessionId) -> Result<Vec<CycleSummary>, DomainError> { Ok(vec![]) }
        async fn get_tree(&self, _session_id: &SessionId) -> Result<Option<CycleTreeNode>, DomainError> { Ok(None) }
        async fn get_progress(&self, _id: &CycleId) -> Result<Option<CycleProgressView>, DomainError> { Ok(None) }
        async fn get_lineage(&self, _id: &CycleId) -> Result<Vec<CycleSummary>, DomainError> { Ok(vec![]) }
    }

    struct MockSessionRepository;

    #[async_trait]
    impl SessionRepository for MockSessionRepository {
        async fn save(&self, _session: &Session) -> Result<(), DomainError> { Ok(()) }
        async fn update(&self, _session: &Session) -> Result<(), DomainError> { Ok(()) }
        async fn find_by_id(&self, _id: &SessionId) -> Result<Option<Session>, DomainError> { Ok(None) }
        async fn find_by_user_id(&self, _user_id: &UserId) -> Result<Vec<Session>, DomainError> { Ok(vec![]) }
        async fn count_by_user_id(&self, _user_id: &UserId) -> Result<u32, DomainError> { Ok(0) }
        async fn delete(&self, _id: &SessionId) -> Result<(), DomainError> { Ok(()) }
    }

    struct MockAccessChecker;

    #[async_trait]
    impl AccessChecker for MockAccessChecker {
        async fn can_create_session(&self, _user_id: &UserId) -> Result<AccessResult, DomainError> { Ok(AccessResult::Allowed) }
        async fn can_create_cycle(&self, _user_id: &UserId, _session_id: &SessionId) -> Result<AccessResult, DomainError> { Ok(AccessResult::Allowed) }
        async fn can_export(&self, _user_id: &UserId) -> Result<AccessResult, DomainError> { Ok(AccessResult::Allowed) }
        async fn get_tier_limits(&self, _user_id: &UserId) -> Result<TierLimits, DomainError> { Ok(TierLimits::default()) }
        async fn get_usage(&self, _user_id: &UserId) -> Result<UsageStats, DomainError> { Ok(UsageStats::new()) }
    }

    struct MockEventPublisher;

    #[async_trait]
    impl EventPublisher for MockEventPublisher {
        async fn publish(&self, _event: crate::domain::foundation::EventEnvelope) -> Result<(), DomainError> { Ok(()) }
        async fn publish_all(&self, _events: Vec<crate::domain::foundation::EventEnvelope>) -> Result<(), DomainError> { Ok(()) }
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Test Helpers
    // ════════════════════════════════════════════════════════════════════════════

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    fn test_user() -> AuthenticatedUser {
        AuthenticatedUser { user_id: test_user_id() }
    }

    fn test_cycle_view() -> CycleView {
        CycleView {
            id: CycleId::new(),
            session_id: SessionId::new(),
            parent_cycle_id: None,
            branch_point: None,
            status: CycleStatus::Active,
            current_step: ComponentType::IssueRaising,
            component_statuses: vec![ComponentStatusItem {
                component_type: ComponentType::IssueRaising,
                status: ComponentStatus::NotStarted,
                is_current: true,
            }],
            progress_percent: 0,
            is_complete: false,
            branch_count: 0,
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        }
    }

    fn test_state() -> CycleAppState {
        let view = test_cycle_view();
        CycleAppState {
            cycle_repository: Arc::new(MockCycleRepository::new()),
            cycle_reader: Arc::new(MockCycleReader::with_view(view)),
            session_repository: Arc::new(MockSessionRepository),
            access_checker: Arc::new(MockAccessChecker),
            event_publisher: Arc::new(MockEventPublisher),
        }
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn get_cycle_returns_cycle_when_exists() {
        let view = test_cycle_view();
        let cycle_id = view.id.to_string();
        let state = CycleAppState {
            cycle_reader: Arc::new(MockCycleReader::with_view(view)),
            ..test_state()
        };
        let user = test_user();

        let result = get_cycle(State(state), Path(cycle_id), user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn get_cycle_returns_not_found_when_missing() {
        let state = CycleAppState {
            cycle_reader: Arc::new(MockCycleReader::new()),
            ..test_state()
        };
        let user = test_user();

        let result = get_cycle(State(state), Path(CycleId::new().to_string()), user).await;
        assert!(matches!(result, Err(CycleApiError::NotFound(_))));
    }

    #[tokio::test]
    async fn get_cycle_returns_bad_request_for_invalid_id() {
        let state = test_state();
        let user = test_user();

        let result = get_cycle(State(state), Path("not-a-uuid".to_string()), user).await;
        assert!(matches!(result, Err(CycleApiError::BadRequest(_))));
    }

    #[test]
    fn cycle_api_error_maps_bad_request_to_400() {
        let err = CycleApiError::BadRequest("test".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn cycle_api_error_maps_not_found_to_404() {
        let err = CycleApiError::NotFound("test".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn cycle_api_error_maps_forbidden_to_403() {
        let err = CycleApiError::Forbidden("test".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn cycle_api_error_maps_conflict_to_409() {
        let err = CycleApiError::Conflict("test".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn cycle_api_error_maps_internal_to_500() {
        let err = CycleApiError::Internal("test".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
