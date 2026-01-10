//! HTTP handlers for cycle endpoints.
//!
//! These handlers connect Axum routes to application layer command/query handlers.
//!
//! Currently implements handlers for:
//! - Create cycle
//! - Branch cycle
//!
//! Additional handlers (archive, complete, component operations, queries) will be
//! added as the corresponding application layer handlers are implemented.

use std::sync::Arc;

use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::application::handlers::cycle::{
    BranchCycleCommand, BranchCycleError, BranchCycleHandler, CreateCycleCommand, CreateCycleError,
    CreateCycleHandler,
};
use crate::domain::foundation::{CommandMetadata, CycleId, SessionId, UserId};
use crate::ports::{AccessChecker, CycleRepository, EventPublisher, SessionRepository};

use super::dto::{
    BranchCycleRequest, CreateCycleRequest, CycleCommandResponse, ErrorResponse,
};

// ════════════════════════════════════════════════════════════════════════════════
// Application State
// ════════════════════════════════════════════════════════════════════════════════

/// Shared application state containing all dependencies.
#[derive(Clone)]
pub struct CycleAppState {
    pub cycle_repository: Arc<dyn CycleRepository>,
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
        BranchCycleHandler::new(
            self.cycle_repository.clone(),
            self.access_checker.clone(),
            self.event_publisher.clone(),
        )
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
// Command Handlers (POST endpoints)
// ════════════════════════════════════════════════════════════════════════════════

/// POST /api/cycles - Create a new cycle
pub async fn create_cycle(
    State(state): State<CycleAppState>,
    user: AuthenticatedUser,
    Json(request): Json<CreateCycleRequest>,
) -> Result<impl IntoResponse, CycleApiError> {
    let session_id: SessionId = request
        .session_id
        .parse()
        .map_err(|_| CycleApiError::BadRequest("Invalid session ID format".to_string()))?;

    let handler = state.create_cycle_handler();
    let cmd = CreateCycleCommand { session_id };
    let metadata = CommandMetadata::new(user.user_id);

    let result = handler.handle(cmd, metadata).await?;

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
    user: AuthenticatedUser,
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
    let metadata = CommandMetadata::new(user.user_id);

    let result = handler.handle(cmd, metadata).await?;

    let response = CycleCommandResponse {
        cycle_id: result.branch.id().to_string(),
        message: format!("Branched at {:?}", result.event.branch_point),
    };

    Ok((StatusCode::CREATED, Json(response)))
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

impl IntoResponse for CycleApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error) = match self {
            CycleApiError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, ErrorResponse::bad_request(msg))
            }
            CycleApiError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, ErrorResponse::not_found("Resource", &msg))
            }
            CycleApiError::Forbidden(msg) => {
                (StatusCode::FORBIDDEN, ErrorResponse::forbidden(msg))
            }
            CycleApiError::Conflict(msg) => (
                StatusCode::CONFLICT,
                ErrorResponse {
                    code: "CONFLICT".to_string(),
                    message: msg,
                    details: None,
                },
            ),
            CycleApiError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ErrorResponse::internal(msg))
            }
        };

        (status, Json(error)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cycle::Cycle;
    use crate::domain::foundation::DomainError;
    use crate::domain::membership::{MembershipTier, TierLimits};
    use crate::domain::session::Session;
    use crate::ports::{AccessResult, UsageStats};
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

    struct MockSessionRepository;

    #[async_trait]
    impl SessionRepository for MockSessionRepository {
        async fn save(&self, _session: &Session) -> Result<(), DomainError> {
            Ok(())
        }
        async fn update(&self, _session: &Session) -> Result<(), DomainError> {
            Ok(())
        }
        async fn find_by_id(&self, _id: &SessionId) -> Result<Option<Session>, DomainError> {
            Ok(None)
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

    struct MockAccessChecker;

    #[async_trait]
    impl AccessChecker for MockAccessChecker {
        async fn can_create_session(&self, _user_id: &UserId) -> Result<AccessResult, DomainError> {
            Ok(AccessResult::Allowed)
        }
        async fn can_create_cycle(
            &self,
            _user_id: &UserId,
            _session_id: &SessionId,
        ) -> Result<AccessResult, DomainError> {
            Ok(AccessResult::Allowed)
        }
        async fn can_export(&self, _user_id: &UserId) -> Result<AccessResult, DomainError> {
            Ok(AccessResult::Allowed)
        }
        async fn get_tier_limits(&self, _user_id: &UserId) -> Result<TierLimits, DomainError> {
            Ok(TierLimits::for_tier(MembershipTier::Free))
        }
        async fn get_usage(&self, _user_id: &UserId) -> Result<UsageStats, DomainError> {
            Ok(UsageStats::new())
        }
    }

    struct MockEventPublisher;

    #[async_trait]
    impl EventPublisher for MockEventPublisher {
        async fn publish(
            &self,
            _event: crate::domain::foundation::EventEnvelope,
        ) -> Result<(), DomainError> {
            Ok(())
        }
        async fn publish_all(
            &self,
            _events: Vec<crate::domain::foundation::EventEnvelope>,
        ) -> Result<(), DomainError> {
            Ok(())
        }
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Test Helpers
    // ════════════════════════════════════════════════════════════════════════════

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    fn _test_user() -> AuthenticatedUser {
        AuthenticatedUser {
            user_id: test_user_id(),
        }
    }

    fn test_state() -> CycleAppState {
        CycleAppState {
            cycle_repository: Arc::new(MockCycleRepository::new()),
            session_repository: Arc::new(MockSessionRepository),
            access_checker: Arc::new(MockAccessChecker),
            event_publisher: Arc::new(MockEventPublisher),
        }
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Tests
    // ════════════════════════════════════════════════════════════════════════════

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

    #[test]
    fn state_creates_handlers() {
        let state = test_state();
        let _ = state.create_cycle_handler();
        let _ = state.branch_cycle_handler();
    }
}
