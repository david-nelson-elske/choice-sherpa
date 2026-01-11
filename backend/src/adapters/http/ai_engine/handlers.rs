//! HTTP handlers for AI Engine endpoints
//!
//! These handlers connect Axum routes to application layer command/query handlers.

use std::sync::Arc;

use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::application::handlers::ai_engine::{
    EndConversationCommand, EndConversationError, EndConversationHandler,
    GetConversationStateError, GetConversationStateHandler, GetConversationStateQuery,
    SendMessageCommand, SendMessageError, SendMessageHandler, StartConversationCommand,
    StartConversationError, StartConversationHandler,
};
use crate::domain::foundation::{ComponentType, CycleId, SessionId};
use crate::ports::{AIProvider, StateStorage};
use std::str::FromStr;

use super::dto::{
    ConversationStateResponse, DeleteConversationResponse, ErrorResponse, SendMessageRequest,
    SendMessageResponse, StartConversationRequest, StartConversationResponse,
};

// ════════════════════════════════════════════════════════════════════════════════
// Application State
// ════════════════════════════════════════════════════════════════════════════════

/// Shared application state containing all dependencies
#[derive(Clone)]
pub struct AIEngineAppState {
    pub storage: Arc<dyn StateStorage>,
    pub ai_provider: Arc<dyn AIProvider>,
}

impl AIEngineAppState {
    pub fn new(storage: Arc<dyn StateStorage>, ai_provider: Arc<dyn AIProvider>) -> Self {
        Self {
            storage,
            ai_provider,
        }
    }

    pub fn start_conversation_handler(&self) -> StartConversationHandler {
        StartConversationHandler::new(self.storage.clone())
    }

    pub fn send_message_handler(&self) -> SendMessageHandler<dyn AIProvider> {
        SendMessageHandler::new(self.storage.clone(), self.ai_provider.clone())
    }

    pub fn end_conversation_handler(&self) -> EndConversationHandler {
        EndConversationHandler::new(self.storage.clone())
    }

    pub fn get_conversation_state_handler(&self) -> GetConversationStateHandler {
        GetConversationStateHandler::new(self.storage.clone())
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Handlers
// ════════════════════════════════════════════════════════════════════════════════

/// Start a new AI conversation
///
/// POST /ai/conversations
pub async fn start_conversation(
    State(app_state): State<AIEngineAppState>,
    Json(req): Json<StartConversationRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // Parse IDs
    let session_id = SessionId::from_str(&req.session_id)
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request("Invalid session_id format")),
            )
        })?;

    let cycle_id = CycleId::from_str(&req.cycle_id)
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request("Invalid cycle_id format")),
            )
        })?;

    // Create command
    let cmd = StartConversationCommand {
        cycle_id,
        session_id,
        initial_component: req.initial_component,
    };

    // Execute command
    let handler = app_state.start_conversation_handler();
    let result = handler.handle(cmd).await.map_err(|e| match e {
        StartConversationError::AlreadyExists(_) => (
            StatusCode::CONFLICT,
            Json(ErrorResponse::conflict(e.to_string())),
        ),
        StartConversationError::Storage(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::internal(msg)),
        ),
        StartConversationError::Domain(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::bad_request(msg.to_string())),
        ),
    })?;

    // Build response
    let response = StartConversationResponse {
        cycle_id: result.state.cycle_id.to_string(),
        current_step: result.state.current_step,
        status: format!("{:?}", result.state.status),
    };

    Ok::<_, (StatusCode, Json<ErrorResponse>)>((StatusCode::CREATED, Json(response)))
}

/// Send a message in a conversation
///
/// POST /ai/conversations/{cycle_id}/messages
pub async fn send_message(
    State(app_state): State<AIEngineAppState>,
    Path(cycle_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // Parse cycle ID
    let cycle_id = CycleId::from_str(&cycle_id)
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request("Invalid cycle_id format")),
            )
        })?;

    // Validate message
    if req.message.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::bad_request("Message cannot be empty")),
        ));
    }

    // Create command
    let cmd = SendMessageCommand {
        cycle_id,
        message: req.message,
    };

    // Execute command
    let handler = app_state.send_message_handler();
    let result = handler.handle(cmd).await.map_err(|e| match e {
        SendMessageError::NotFound(_) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::not_found("Conversation", &cycle_id.to_string())),
        ),
        SendMessageError::Storage(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::internal(msg)),
        ),
        SendMessageError::Orchestrator(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::bad_request(msg)),
        ),
        SendMessageError::Domain(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::bad_request(msg.to_string())),
        ),
        SendMessageError::AIProvider(msg) => (
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse::internal(format!("AI Provider error: {}", msg))),
        ),
    })?;

    // Get turn count from current step state
    let turn_count = result
        .updated_state
        .step_state(result.updated_state.current_step)
        .map(|s| s.turn_count)
        .unwrap_or(0);

    // Build response
    let response = SendMessageResponse {
        response: result.ai_response,
        current_step: result.updated_state.current_step,
        turn_count,
    };

    Ok::<_, (StatusCode, Json<ErrorResponse>)>((StatusCode::OK, Json(response)))
}

/// Get conversation state
///
/// GET /ai/conversations/{cycle_id}
pub async fn get_conversation_state(
    State(app_state): State<AIEngineAppState>,
    Path(cycle_id): Path<String>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // Parse cycle ID
    let cycle_id = CycleId::from_str(&cycle_id)
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request("Invalid cycle_id format")),
            )
        })?;

    // Create query
    let query = GetConversationStateQuery { cycle_id };

    // Execute query
    let handler = app_state.get_conversation_state_handler();
    let result = handler.handle(query).await.map_err(|e| {
        match e {
            GetConversationStateError::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::not_found("Conversation", &cycle_id.to_string())),
            ),
            GetConversationStateError::Storage(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::internal(msg)),
            ),
            GetConversationStateError::Domain(msg) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request(msg.to_string())),
            ),
        }
    })?;

    // Collect completed steps
    let completed_steps: Vec<ComponentType> = result
        .state
        .step_states
        .iter()
        .filter_map(|(component, step_state)| {
            use crate::domain::ai_engine::conversation_state::StepStatus;
            if step_state.status == StepStatus::Completed {
                Some(*component)
            } else {
                None
            }
        })
        .collect();

    // Build response
    let response = ConversationStateResponse {
        cycle_id: result.state.cycle_id.to_string(),
        session_id: result.state.session_id.to_string(),
        current_step: result.state.current_step,
        status: format!("{:?}", result.state.status),
        message_count: result.state.message_history.len(),
        completed_steps,
    };

    Ok::<_, (StatusCode, Json<ErrorResponse>)>((StatusCode::OK, Json(response)))
}

/// End a conversation
///
/// DELETE /ai/conversations/{cycle_id}
pub async fn end_conversation(
    State(app_state): State<AIEngineAppState>,
    Path(cycle_id): Path<String>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // Parse cycle ID
    let cycle_id = CycleId::from_str(&cycle_id)
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request("Invalid cycle_id format")),
            )
        })?;

    // Create command
    let cmd = EndConversationCommand { cycle_id };

    // Execute command
    let handler = app_state.end_conversation_handler();
    handler.handle(cmd).await.map_err(|e| match e {
        EndConversationError::NotFound(_) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::not_found("Conversation", &cycle_id.to_string())),
        ),
        EndConversationError::Storage(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::internal(msg)),
        ),
        EndConversationError::Domain(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::bad_request(msg.to_string())),
        ),
    })?;

    // Build response
    let response = DeleteConversationResponse {
        message: format!("Conversation {} ended successfully", cycle_id),
    };

    Ok::<_, (StatusCode, Json<ErrorResponse>)>((StatusCode::OK, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::{InMemoryStateStorage, MockAIProvider};
    use crate::domain::ai_engine::ConversationState;
    use crate::domain::foundation::ComponentType;

    fn test_app_state() -> AIEngineAppState {
        AIEngineAppState {
            storage: Arc::new(InMemoryStateStorage::new()),
            ai_provider: Arc::new(MockAIProvider::new().with_response("Test AI response")),
        }
    }

    #[tokio::test]
    async fn test_start_conversation_handler() {
        let app_state = test_app_state();
        let session_id = SessionId::new();
        let cycle_id = CycleId::new();

        let req = StartConversationRequest {
            session_id: session_id.to_string(),
            cycle_id: cycle_id.to_string(),
            initial_component: ComponentType::IssueRaising,
        };

        let result = start_conversation(State(app_state), Json(req)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_message_handler() {
        let app_state = test_app_state();
        let cycle_id = CycleId::new();
        let state = ConversationState::new(
            cycle_id,
            SessionId::new(),
            ComponentType::IssueRaising,
        );
        app_state
            .storage
            .save_state(cycle_id, &state)
            .await
            .unwrap();

        let req = SendMessageRequest {
            message: "Hello AI".to_string(),
        };

        let result = send_message(State(app_state), Path(cycle_id.to_string()), Json(req)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_conversation_state_handler() {
        let app_state = test_app_state();
        let cycle_id = CycleId::new();
        let state = ConversationState::new(
            cycle_id,
            SessionId::new(),
            ComponentType::IssueRaising,
        );
        app_state
            .storage
            .save_state(cycle_id, &state)
            .await
            .unwrap();

        let result = get_conversation_state(State(app_state), Path(cycle_id.to_string())).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_end_conversation_handler() {
        let app_state = test_app_state();
        let cycle_id = CycleId::new();
        let state = ConversationState::new(
            cycle_id,
            SessionId::new(),
            ComponentType::IssueRaising,
        );
        app_state
            .storage
            .save_state(cycle_id, &state)
            .await
            .unwrap();

        let result = end_conversation(State(app_state), Path(cycle_id.to_string())).await;
        assert!(result.is_ok());
    }
}
