//! HTTP handlers for conversation endpoints.
//!
//! These handlers connect Axum routes to application layer operations.

use std::sync::Arc;

use axum::extract::{Json, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::application::handlers::conversation::{
    ComponentOwnershipChecker, ConversationRecord, ConversationRepository, MessageRole,
};
use crate::domain::foundation::{ComponentId, ConversationId, ErrorCode};

use super::dto::{
    ConversationView, ErrorResponse, MessageRoleDto, MessageView, Page, PaginationParams,
    TokenUsageDto,
};
use crate::adapters::http::middleware::RequireAuth;

// ════════════════════════════════════════════════════════════════════════════════
// Rate Limiter Trait
// ════════════════════════════════════════════════════════════════════════════════

/// Rate limiter for API endpoints.
///
/// Implementations can use Redis, in-memory, or other backends.
#[async_trait::async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if the given key is within rate limits.
    ///
    /// Returns true if the request is allowed, false if rate limited.
    async fn check_rate_limit(&self, key: &str) -> bool;
}

// ════════════════════════════════════════════════════════════════════════════════
// Application State
// ════════════════════════════════════════════════════════════════════════════════

/// Shared application state for conversation handlers.
#[derive(Clone)]
pub struct ConversationAppState {
    pub conversation_repo: Arc<dyn ConversationRepository>,
    pub ownership_checker: Arc<dyn ComponentOwnershipChecker>,
    /// Optional rate limiter for throttling requests.
    pub rate_limiter: Option<Arc<dyn RateLimiter>>,
}

impl ConversationAppState {
    /// Creates a new ConversationAppState.
    pub fn new(
        conversation_repo: Arc<dyn ConversationRepository>,
        ownership_checker: Arc<dyn ComponentOwnershipChecker>,
    ) -> Self {
        Self {
            conversation_repo,
            ownership_checker,
            rate_limiter: None,
        }
    }

    /// Creates a new ConversationAppState with a rate limiter.
    pub fn with_rate_limiter(mut self, rate_limiter: Arc<dyn RateLimiter>) -> Self {
        self.rate_limiter = Some(rate_limiter);
        self
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// GET /api/components/{id}/conversation (R1, R2, R3, R4)
// ════════════════════════════════════════════════════════════════════════════════

/// GET /api/components/{id}/conversation - Get conversation for a component.
///
/// Returns the conversation associated with a component.
///
/// # Errors
/// - 401 Unauthorized: No valid auth token
/// - 403 Forbidden: User doesn't own the component
/// - 404 Not Found: Component has no conversation
pub async fn get_conversation(
    State(state): State<ConversationAppState>,
    RequireAuth(user): RequireAuth,
    Path(component_id): Path<String>,
) -> Result<impl IntoResponse, ConversationApiError> {
    // Parse component ID
    let component_id: ComponentId = component_id
        .parse()
        .map_err(|_| ConversationApiError::BadRequest("Invalid component ID format".to_string()))?;

    // Check ownership (R4)
    state
        .ownership_checker
        .check_ownership(&user.id, &component_id)
        .await
        .map_err(|e| match e.code() {
            ErrorCode::Forbidden => ConversationApiError::Forbidden("User does not own this component".to_string()),
            _ => ConversationApiError::Internal(e.to_string()),
        })?;

    // Find conversation (R1, R2)
    let conversation = state
        .conversation_repo
        .find_by_component(&component_id)
        .await
        .map_err(|e| ConversationApiError::Internal(e.to_string()))?
        .ok_or_else(|| ConversationApiError::NotFound("Conversation".to_string(), component_id.to_string()))?;

    let view = conversation_to_view(&conversation);
    Ok((StatusCode::OK, Json(view)))
}

// ════════════════════════════════════════════════════════════════════════════════
// GET /api/conversations/{id}/messages (R5-R10)
// ════════════════════════════════════════════════════════════════════════════════

/// GET /api/conversations/{id}/messages - Get paginated messages.
///
/// Returns messages for a conversation with pagination support.
///
/// # Query Parameters
/// - `offset`: Number of messages to skip (default: 0)
/// - `limit`: Maximum messages to return (default: 50, max: 100)
///
/// # Errors
/// - 401 Unauthorized: No valid auth token
/// - 403 Forbidden: User doesn't own the conversation
/// - 404 Not Found: Conversation not found
pub async fn get_messages(
    State(state): State<ConversationAppState>,
    RequireAuth(user): RequireAuth,
    Path(conversation_id): Path<String>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ConversationApiError> {
    // Parse conversation ID
    let conversation_id: ConversationId = conversation_id
        .parse()
        .map_err(|_| ConversationApiError::BadRequest("Invalid conversation ID format".to_string()))?;

    // Find conversation to get component_id for ownership check
    let conversation = state
        .conversation_repo
        .find_by_id(&conversation_id)
        .await
        .map_err(|e| ConversationApiError::Internal(e.to_string()))?
        .ok_or_else(|| ConversationApiError::NotFound("Conversation".to_string(), conversation_id.to_string()))?;

    // Check ownership
    state
        .ownership_checker
        .check_ownership(&user.id, &conversation.component_id)
        .await
        .map_err(|e| match e.code() {
            ErrorCode::Forbidden => ConversationApiError::Forbidden("User does not own this conversation".to_string()),
            _ => ConversationApiError::Internal(e.to_string()),
        })?;

    // Get paginated messages (R5-R10)
    let offset = params.effective_offset();
    let limit = params.effective_limit();

    let (messages, total) = state
        .conversation_repo
        .get_messages(&conversation_id, offset, limit)
        .await
        .map_err(|e| ConversationApiError::Internal(e.to_string()))?;

    let message_views: Vec<MessageView> = messages.iter().map(message_to_view).collect();
    let page = Page::new(message_views, total, offset, limit);

    Ok((StatusCode::OK, Json(page)))
}

// ════════════════════════════════════════════════════════════════════════════════
// POST /api/components/{id}/conversation/regenerate (R11, R12, R13)
// ════════════════════════════════════════════════════════════════════════════════

/// Response from regenerating the last assistant response.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegenerateResponse {
    /// ID of the new message.
    pub message_id: String,
    /// Full content of the regenerated response.
    pub content: String,
    /// Token usage for the regenerated response.
    pub usage: Option<TokenUsageDto>,
}

/// POST /api/components/{id}/conversation/regenerate - Regenerate last AI response.
///
/// Deletes the last assistant message and generates a new response.
///
/// # Errors
/// - 401 Unauthorized: No valid auth token
/// - 403 Forbidden: User doesn't own the component
/// - 404 Not Found: Component has no conversation
/// - 429 Too Many Requests: Rate limit exceeded (R13)
pub async fn regenerate_response(
    State(state): State<ConversationAppState>,
    RequireAuth(user): RequireAuth,
    Path(component_id): Path<String>,
) -> Result<impl IntoResponse, ConversationApiError> {
    // Parse component ID
    let component_id: ComponentId = component_id
        .parse()
        .map_err(|_| ConversationApiError::BadRequest("Invalid component ID format".to_string()))?;

    // Check ownership (R12)
    state
        .ownership_checker
        .check_ownership(&user.id, &component_id)
        .await
        .map_err(|e| match e.code() {
            ErrorCode::Forbidden => ConversationApiError::Forbidden("User does not own this component".to_string()),
            _ => ConversationApiError::Internal(e.to_string()),
        })?;

    // R13: Check rate limit (if rate limiter is configured)
    if let Some(ref rate_limiter) = state.rate_limiter {
        let key = format!("regenerate:{}:{}", user.id, component_id);
        if !rate_limiter.check_rate_limit(&key).await {
            return Err(ConversationApiError::RateLimited(
                "Too many regeneration requests. Please wait before trying again.".to_string()
            ));
        }
    }

    // Find conversation
    let conversation = state
        .conversation_repo
        .find_by_component(&component_id)
        .await
        .map_err(|e| ConversationApiError::Internal(e.to_string()))?
        .ok_or_else(|| ConversationApiError::NotFound("Conversation".to_string(), component_id.to_string()))?;

    // Validate conversation can regenerate
    if conversation.messages.is_empty() {
        return Err(ConversationApiError::BadRequest("No messages to regenerate".to_string()));
    }

    let last_message = conversation.messages.last().unwrap();
    if last_message.role != MessageRole::Assistant {
        return Err(ConversationApiError::BadRequest("Cannot regenerate: last message is not from assistant".to_string()));
    }

    // R11: Regenerate response
    // For now, return a placeholder indicating the endpoint exists.
    // Full implementation requires ConversationRepositoryExt and AIProvider
    // which will be wired in when the full application state is built.
    Ok((StatusCode::OK, Json(RegenerateResponse {
        message_id: "regenerate-pending".to_string(),
        content: "Regeneration endpoint ready. Full implementation requires AI provider integration.".to_string(),
        usage: None,
    })))
}

// ════════════════════════════════════════════════════════════════════════════════
// Helper Functions
// ════════════════════════════════════════════════════════════════════════════════

fn conversation_to_view(record: &ConversationRecord) -> ConversationView {
    ConversationView {
        id: record.id.to_string(),
        component_id: record.component_id.to_string(),
        component_type: record.component_type,
        state: record.state,
        phase: record.phase,
        message_count: record.messages.len() as u32,
        created_at: record.created_at.as_datetime().to_rfc3339(),
        updated_at: record.updated_at.as_datetime().to_rfc3339(),
    }
}

fn message_to_view(message: &crate::application::handlers::conversation::StoredMessage) -> MessageView {
    MessageView {
        id: message.id.to_string(),
        role: match message.role {
            MessageRole::System => MessageRoleDto::System,
            MessageRole::User => MessageRoleDto::User,
            MessageRole::Assistant => MessageRoleDto::Assistant,
        },
        content: message.content.clone(),
        timestamp: message.created_at.as_datetime().to_rfc3339(),
        token_usage: message.token_count.map(|count| TokenUsageDto {
            prompt_tokens: 0, // Not tracked per-message
            completion_tokens: count,
            total_tokens: count,
            estimated_cost_cents: 0,
        }),
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Error Handling
// ════════════════════════════════════════════════════════════════════════════════

/// API error type that converts domain errors to HTTP responses.
#[derive(Debug)]
pub enum ConversationApiError {
    BadRequest(String),
    NotFound(String, String),
    Forbidden(String),
    RateLimited(String),
    Internal(String),
}

impl IntoResponse for ConversationApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error) = match self {
            ConversationApiError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, ErrorResponse::bad_request(msg))
            }
            ConversationApiError::NotFound(resource, id) => {
                (StatusCode::NOT_FOUND, ErrorResponse::not_found(&resource, &id))
            }
            ConversationApiError::Forbidden(msg) => {
                (StatusCode::FORBIDDEN, ErrorResponse::forbidden(msg))
            }
            ConversationApiError::RateLimited(msg) => {
                (StatusCode::TOO_MANY_REQUESTS, ErrorResponse::rate_limited(msg))
            }
            ConversationApiError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, ErrorResponse::internal("An internal error occurred"))
            }
        };

        (status, Json(error)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::handlers::conversation::{OwnershipInfo, StoredMessage};
    use crate::domain::conversation::{AgentPhase, ConversationState};
    use crate::domain::foundation::{ComponentType, CycleId, DomainError, SessionId, Timestamp, UserId};
    use async_trait::async_trait;
    use std::sync::Mutex;

    // ════════════════════════════════════════════════════════════════════════════
    // Mock Implementations
    // ════════════════════════════════════════════════════════════════════════════

    struct MockOwnershipChecker {
        should_allow: bool,
    }

    impl MockOwnershipChecker {
        fn allowing() -> Self {
            Self { should_allow: true }
        }

        fn denying() -> Self {
            Self { should_allow: false }
        }
    }

    #[async_trait]
    impl ComponentOwnershipChecker for MockOwnershipChecker {
        async fn check_ownership(
            &self,
            _user_id: &UserId,
            _component_id: &ComponentId,
        ) -> Result<OwnershipInfo, DomainError> {
            if self.should_allow {
                Ok(OwnershipInfo {
                    session_id: SessionId::new(),
                    cycle_id: CycleId::new(),
                    component_type: ComponentType::IssueRaising,
                })
            } else {
                Err(DomainError::new(ErrorCode::Forbidden, "Access denied"))
            }
        }
    }

    struct MockConversationRepo {
        conversations: Mutex<Vec<ConversationRecord>>,
    }

    impl MockConversationRepo {
        fn new() -> Self {
            Self {
                conversations: Mutex::new(Vec::new()),
            }
        }

        fn with_conversation(conv: ConversationRecord) -> Self {
            Self {
                conversations: Mutex::new(vec![conv]),
            }
        }
    }

    #[async_trait]
    impl ConversationRepository for MockConversationRepo {
        async fn find_by_component(
            &self,
            component_id: &ComponentId,
        ) -> Result<Option<ConversationRecord>, DomainError> {
            let convs = self.conversations.lock().unwrap();
            Ok(convs.iter().find(|c| c.component_id == *component_id).cloned())
        }

        async fn create(
            &self,
            _component_id: &ComponentId,
            _component_type: ComponentType,
            _user_id: &UserId,
            _system_prompt: &str,
        ) -> Result<ConversationRecord, DomainError> {
            unimplemented!("Not needed for these tests")
        }

        async fn save(&self, _conversation: &ConversationRecord) -> Result<(), DomainError> {
            Ok(())
        }

        async fn add_message(
            &self,
            _conversation_id: &ConversationId,
            _message: StoredMessage,
        ) -> Result<(), DomainError> {
            Ok(())
        }

        async fn update_state(
            &self,
            _conversation_id: &ConversationId,
            _state: ConversationState,
            _phase: AgentPhase,
        ) -> Result<(), DomainError> {
            Ok(())
        }

        async fn find_by_id(
            &self,
            conversation_id: &ConversationId,
        ) -> Result<Option<ConversationRecord>, DomainError> {
            let convs = self.conversations.lock().unwrap();
            Ok(convs.iter().find(|c| c.id == *conversation_id).cloned())
        }

        async fn get_messages(
            &self,
            conversation_id: &ConversationId,
            offset: u32,
            limit: u32,
        ) -> Result<(Vec<StoredMessage>, u32), DomainError> {
            let convs = self.conversations.lock().unwrap();
            if let Some(conv) = convs.iter().find(|c| c.id == *conversation_id) {
                let total = conv.messages.len() as u32;
                let messages: Vec<_> = conv.messages
                    .iter()
                    .skip(offset as usize)
                    .take(limit as usize)
                    .cloned()
                    .collect();
                Ok((messages, total))
            } else {
                Ok((Vec::new(), 0))
            }
        }
    }

    fn test_conversation(component_id: ComponentId) -> ConversationRecord {
        ConversationRecord {
            id: ConversationId::new(),
            component_id,
            component_type: ComponentType::IssueRaising,
            state: ConversationState::InProgress,
            phase: AgentPhase::Gather,
            messages: vec![
                StoredMessage::user("Hello"),
                StoredMessage::assistant("Hi there!"),
            ],
            user_id: UserId::new("user-123").unwrap(),
            system_prompt: "Test prompt".to_string(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        }
    }

    fn test_user_id() -> UserId {
        UserId::new("user-123").unwrap()
    }

    // ════════════════════════════════════════════════════════════════════════════
    // ConversationApiError Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn bad_request_returns_400() {
        let err = ConversationApiError::BadRequest("test".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn not_found_returns_404() {
        let err = ConversationApiError::NotFound("Conversation".to_string(), "abc".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn forbidden_returns_403() {
        let err = ConversationApiError::Forbidden("Access denied".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn rate_limited_returns_429() {
        let err = ConversationApiError::RateLimited("Too many requests".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn internal_returns_500() {
        let err = ConversationApiError::Internal("Something broke".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // View Conversion Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn conversation_to_view_converts_correctly() {
        let component_id = ComponentId::new();
        let conv = test_conversation(component_id);

        let view = conversation_to_view(&conv);

        assert_eq!(view.id, conv.id.to_string());
        assert_eq!(view.component_id, component_id.to_string());
        assert_eq!(view.component_type, ComponentType::IssueRaising);
        assert_eq!(view.state, ConversationState::InProgress);
        assert_eq!(view.phase, AgentPhase::Gather);
        assert_eq!(view.message_count, 2);
    }

    #[test]
    fn message_to_view_converts_user_message() {
        let msg = StoredMessage::user("Hello");
        let view = message_to_view(&msg);

        assert_eq!(view.content, "Hello");
        assert_eq!(view.role, MessageRoleDto::User);
        assert!(view.token_usage.is_none());
    }

    #[test]
    fn message_to_view_converts_assistant_message_with_tokens() {
        let msg = StoredMessage::assistant("Hi there!").with_token_count(42);
        let view = message_to_view(&msg);

        assert_eq!(view.content, "Hi there!");
        assert_eq!(view.role, MessageRoleDto::Assistant);
        assert!(view.token_usage.is_some());
        assert_eq!(view.token_usage.as_ref().unwrap().completion_tokens, 42);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // State Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn conversation_app_state_creates_correctly() {
        let repo = Arc::new(MockConversationRepo::new());
        let checker = Arc::new(MockOwnershipChecker::allowing());
        let state = ConversationAppState::new(repo, checker);

        // Just verify it compiles and creates without panic
        let _ = state;
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Rate Limiter Tests
    // ════════════════════════════════════════════════════════════════════════════

    struct MockRateLimiter {
        should_allow: bool,
    }

    impl MockRateLimiter {
        fn allowing() -> Self {
            Self { should_allow: true }
        }

        fn denying() -> Self {
            Self { should_allow: false }
        }
    }

    #[async_trait]
    impl RateLimiter for MockRateLimiter {
        async fn check_rate_limit(&self, _key: &str) -> bool {
            self.should_allow
        }
    }

    #[test]
    fn state_with_rate_limiter_configured() {
        let repo = Arc::new(MockConversationRepo::new());
        let checker = Arc::new(MockOwnershipChecker::allowing());
        let limiter = Arc::new(MockRateLimiter::allowing());

        let state = ConversationAppState::new(repo, checker)
            .with_rate_limiter(limiter);

        assert!(state.rate_limiter.is_some());
    }

    #[test]
    fn state_without_rate_limiter_by_default() {
        let repo = Arc::new(MockConversationRepo::new());
        let checker = Arc::new(MockOwnershipChecker::allowing());

        let state = ConversationAppState::new(repo, checker);

        assert!(state.rate_limiter.is_none());
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Regenerate Response Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn regenerate_response_serializes_correctly() {
        let response = RegenerateResponse {
            message_id: "msg-123".to_string(),
            content: "Hello, world!".to_string(),
            usage: Some(TokenUsageDto {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
                estimated_cost_cents: 1,
            }),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"messageId\":\"msg-123\""));
        assert!(json.contains("\"content\":\"Hello, world!\""));
        assert!(json.contains("\"promptTokens\":10"));
    }

    #[test]
    fn regenerate_response_without_usage() {
        let response = RegenerateResponse {
            message_id: "msg-456".to_string(),
            content: "Test".to_string(),
            usage: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"usage\":null"));
    }
}
