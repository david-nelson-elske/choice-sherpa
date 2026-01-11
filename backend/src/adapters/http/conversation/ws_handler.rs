//! WebSocket handler for conversation AI streaming.
//!
//! Provides real-time streaming of AI responses to connected clients.
//! Protocol defined in docs/api/streaming-protocol.md.
//!
//! # Connection Flow
//! 1. Client requests WebSocket upgrade with auth token
//! 2. Server validates auth and component ownership (R14, R15)
//! 3. On success, upgrade connection to WebSocket
//! 4. Client sends SendMessage with user content (R16)
//! 5. Server streams TokenChunk events (R17)
//! 6. Server sends StreamComplete when done (R18)
//! 7. On AI error, sends StreamError (R19)
//! 8. On disconnect, cleanup resources (R20)

use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    response::{IntoResponse, Response},
    http::StatusCode,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;

use crate::application::handlers::conversation::{ComponentOwnershipChecker, ConversationRepository};
use crate::domain::foundation::{ComponentId, ErrorCode, Timestamp, UserId};

use super::streaming::{
    SendMessageRequest, StreamChunkMessage, StreamClientMessage, StreamCompleteMessage,
    StreamErrorCode, StreamErrorMessage, StreamPongMessage, StreamServerMessage, StreamTokenUsage,
};

// ════════════════════════════════════════════════════════════════════════════════
// WebSocket State
// ════════════════════════════════════════════════════════════════════════════════

/// State required for conversation WebSocket handling.
#[derive(Clone)]
pub struct ConversationWebSocketState {
    /// Repository for conversation data.
    pub conversation_repo: Arc<dyn ConversationRepository>,
    /// Checker for component ownership validation.
    pub ownership_checker: Arc<dyn ComponentOwnershipChecker>,
    // AI provider would be added here for actual streaming
    // pub ai_provider: Arc<dyn AIProvider>,
}

impl ConversationWebSocketState {
    /// Create new WebSocket state.
    pub fn new(
        conversation_repo: Arc<dyn ConversationRepository>,
        ownership_checker: Arc<dyn ComponentOwnershipChecker>,
    ) -> Self {
        Self {
            conversation_repo,
            ownership_checker,
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Query Parameters
// ════════════════════════════════════════════════════════════════════════════════

/// Query parameters for WebSocket connection.
#[derive(Debug, Deserialize)]
pub struct WsConnectParams {
    /// Auth token for user authentication.
    /// In production, this would be validated against auth provider.
    pub token: Option<String>,
}

// ════════════════════════════════════════════════════════════════════════════════
// WebSocket Upgrade Handler (R14, R15)
// ════════════════════════════════════════════════════════════════════════════════

/// Handle WebSocket upgrade for conversation streaming.
///
/// Route: `GET /api/components/{component_id}/stream`
///
/// # Security (R14, R15)
/// - Validates auth token before upgrade
/// - Verifies user owns the component
/// - Rejects with 401/403 before WebSocket upgrade
pub async fn conversation_ws_handler(
    ws: WebSocketUpgrade,
    Path(component_id): Path<String>,
    Query(params): Query<WsConnectParams>,
    State(state): State<ConversationWebSocketState>,
) -> Response {
    // R15: Validate auth token before upgrade
    let user_id = match validate_auth_token(&params.token).await {
        Ok(user_id) => user_id,
        Err(e) => {
            return (StatusCode::UNAUTHORIZED, e).into_response();
        }
    };

    // Parse component ID
    let component_id: ComponentId = match component_id.parse() {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid component ID format").into_response();
        }
    };

    // R14: Verify ownership before upgrade
    if let Err(e) = state
        .ownership_checker
        .check_ownership(&user_id, &component_id)
        .await
    {
        return match e.code() {
            ErrorCode::Forbidden => {
                (StatusCode::FORBIDDEN, "User does not own this component").into_response()
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Ownership check failed").into_response(),
        };
    }

    // Verify conversation exists
    match state.conversation_repo.find_by_component(&component_id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "Conversation not found").into_response();
        }
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to find conversation").into_response();
        }
    }

    // R14: Upgrade to WebSocket
    ws.on_upgrade(move |socket| handle_conversation_socket(socket, component_id, user_id, state))
}

// ════════════════════════════════════════════════════════════════════════════════
// WebSocket Connection Handler (R16-R20)
// ════════════════════════════════════════════════════════════════════════════════

/// Handle an established WebSocket connection for conversation streaming.
async fn handle_conversation_socket(
    socket: WebSocket,
    component_id: ComponentId,
    user_id: UserId,
    state: ConversationWebSocketState,
) {
    let (mut sender, mut receiver) = socket.split();

    tracing::info!(
        component_id = %component_id,
        user_id = %user_id,
        "WebSocket connection established"
    );

    // Process incoming messages
    while let Some(result) = receiver.next().await {
        match result {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<StreamClientMessage>(&text) {
                    Ok(client_msg) => {
                        match client_msg {
                            // R16: Handle user message
                            StreamClientMessage::SendMessage(req) => {
                                if let Err(e) = req.validate() {
                                    let error_msg = StreamServerMessage::StreamError(StreamErrorMessage {
                                        message_id: req.message_id.clone(),
                                        error_code: StreamErrorCode::InternalError,
                                        error: e.to_string(),
                                        partial_content: None,
                                        recoverable: false,
                                    });
                                    if send_server_message(&mut sender, &error_msg).await.is_err() {
                                        break;
                                    }
                                    continue;
                                }

                                // R17, R18: Stream AI response
                                handle_send_message(&mut sender, &req, &component_id, &state).await;
                            }

                            // Handle cancel request
                            StreamClientMessage::CancelStream(req) => {
                                tracing::debug!(
                                    message_id = %req.message_id,
                                    "Cancel stream requested"
                                );
                                // Send cancelled error
                                let cancelled = StreamServerMessage::StreamError(StreamErrorMessage {
                                    message_id: req.message_id,
                                    error_code: StreamErrorCode::Cancelled,
                                    error: "Stream cancelled by user".to_string(),
                                    partial_content: None,
                                    recoverable: false,
                                });
                                if send_server_message(&mut sender, &cancelled).await.is_err() {
                                    break;
                                }
                            }

                            // Handle ping
                            StreamClientMessage::Ping => {
                                let pong = StreamServerMessage::Pong(StreamPongMessage {
                                    timestamp: Timestamp::now().as_datetime().to_rfc3339(),
                                });
                                if send_server_message(&mut sender, &pong).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse client message: {}", e);
                    }
                }
            }
            Ok(Message::Close(_)) => {
                tracing::debug!(component_id = %component_id, "Client closed connection");
                break;
            }
            Ok(Message::Ping(data)) => {
                if sender.send(Message::Pong(data)).await.is_err() {
                    break;
                }
            }
            Ok(_) => {} // Ignore other message types
            Err(e) => {
                tracing::debug!(component_id = %component_id, "WebSocket error: {}", e);
                break;
            }
        }
    }

    // R20: Cleanup on disconnect
    cleanup_connection(&component_id, &user_id).await;

    tracing::info!(
        component_id = %component_id,
        user_id = %user_id,
        "WebSocket connection closed"
    );
}

// ════════════════════════════════════════════════════════════════════════════════
// Message Handling (R17, R18, R19)
// ════════════════════════════════════════════════════════════════════════════════

/// Handle a SendMessage request by streaming AI response.
async fn handle_send_message<S>(
    sender: &mut S,
    req: &SendMessageRequest,
    component_id: &ComponentId,
    _state: &ConversationWebSocketState,
) where
    S: SinkExt<Message> + Unpin,
    S::Error: std::fmt::Debug,
{
    tracing::debug!(
        message_id = %req.message_id,
        component_id = %component_id,
        "Processing user message"
    );

    // In a full implementation, this would:
    // 1. Save the user message to conversation
    // 2. Build AI prompt with conversation context
    // 3. Stream AI response chunks via sender
    // 4. Save assistant message when complete
    // 5. Check for phase transitions

    // For now, send a placeholder response demonstrating the protocol
    // R17: Stream token chunks
    let chunks = ["Hello! ", "I am ", "the AI ", "assistant. ", "How can I help?"];
    let mut full_content = String::new();

    for (i, chunk) in chunks.iter().enumerate() {
        full_content.push_str(chunk);
        let is_final = i == chunks.len() - 1;

        let chunk_msg = StreamServerMessage::StreamChunk(StreamChunkMessage {
            message_id: req.message_id.clone(),
            delta: chunk.to_string(),
            is_final,
        });

        if let Err(e) = send_server_message(sender, &chunk_msg).await {
            tracing::debug!("Failed to send chunk: {:?}", e);
            // R19: Send error on failure
            let error_msg = StreamServerMessage::StreamError(StreamErrorMessage {
                message_id: req.message_id.clone(),
                error_code: StreamErrorCode::InternalError,
                error: "Failed to stream response".to_string(),
                partial_content: Some(full_content),
                recoverable: true,
            });
            let _ = send_server_message(sender, &error_msg).await;
            return;
        }

        // Small delay to simulate streaming
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // R18: Send completion event
    let complete_msg = StreamServerMessage::StreamComplete(StreamCompleteMessage {
        message_id: req.message_id.clone(),
        full_content,
        usage: StreamTokenUsage {
            prompt_tokens: 10,
            completion_tokens: 15,
            total_tokens: 25,
            estimated_cost_cents: 1,
        },
        phase_transition: None,
    });

    if let Err(e) = send_server_message(sender, &complete_msg).await {
        tracing::debug!("Failed to send complete message: {:?}", e);
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Helper Functions
// ════════════════════════════════════════════════════════════════════════════════

/// Validate auth token and extract user ID.
///
/// In production, this would validate against the auth provider.
async fn validate_auth_token(token: &Option<String>) -> Result<UserId, &'static str> {
    match token {
        Some(t) if !t.is_empty() => {
            // In production: validate JWT, extract user_id claim
            // For now, use a placeholder validation
            if t.starts_with("test_") || t.len() > 10 {
                // Mock: extract user ID from token or use placeholder
                Ok(UserId::new("user-from-token").unwrap())
            } else {
                Err("Invalid token format")
            }
        }
        _ => Err("Missing authentication token"),
    }
}

/// Send a server message over the WebSocket.
async fn send_server_message<S>(sender: &mut S, msg: &StreamServerMessage) -> Result<(), S::Error>
where
    S: SinkExt<Message> + Unpin,
{
    let json = serde_json::to_string(msg).expect("Failed to serialize message");
    sender.send(Message::Text(json)).await
}

/// Cleanup resources when connection closes (R20).
async fn cleanup_connection(component_id: &ComponentId, user_id: &UserId) {
    // In production, this would:
    // - Cancel any in-flight AI requests
    // - Release any held resources
    // - Update connection tracking
    tracing::debug!(
        component_id = %component_id,
        user_id = %user_id,
        "Cleaned up connection resources"
    );
}

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    mod auth_validation {
        use super::*;

        #[tokio::test]
        async fn rejects_missing_token() {
            let result = validate_auth_token(&None).await;
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "Missing authentication token");
        }

        #[tokio::test]
        async fn rejects_empty_token() {
            let result = validate_auth_token(&Some("".to_string())).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn accepts_valid_token() {
            let result = validate_auth_token(&Some("test_valid_token".to_string())).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn accepts_long_token() {
            let result = validate_auth_token(&Some("a_very_long_auth_token_here".to_string())).await;
            assert!(result.is_ok());
        }
    }

    mod ws_state {
        use super::*;
        use crate::application::handlers::conversation::{ConversationRecord, OwnershipInfo, StoredMessage};
        use crate::domain::conversation::{AgentPhase, ConversationState};
        use crate::domain::foundation::{ComponentType, ConversationId, CycleId, DomainError, SessionId, Timestamp};
        use async_trait::async_trait;
        use std::sync::Mutex;

        struct MockOwnershipChecker;

        #[async_trait]
        impl ComponentOwnershipChecker for MockOwnershipChecker {
            async fn check_ownership(
                &self,
                _user_id: &UserId,
                _component_id: &ComponentId,
            ) -> Result<OwnershipInfo, DomainError> {
                Ok(OwnershipInfo {
                    session_id: SessionId::new(),
                    cycle_id: CycleId::new(),
                    component_type: ComponentType::IssueRaising,
                })
            }
        }

        struct MockConversationRepo;

        #[async_trait]
        impl ConversationRepository for MockConversationRepo {
            async fn find_by_component(
                &self,
                _component_id: &ComponentId,
            ) -> Result<Option<ConversationRecord>, DomainError> {
                Ok(Some(ConversationRecord {
                    id: ConversationId::new(),
                    component_id: ComponentId::new(),
                    component_type: ComponentType::IssueRaising,
                    state: ConversationState::InProgress,
                    phase: AgentPhase::Gather,
                    messages: vec![],
                    user_id: UserId::new("test").unwrap(),
                    system_prompt: "Test".to_string(),
                    created_at: Timestamp::now(),
                    updated_at: Timestamp::now(),
                }))
            }

            async fn create(
                &self,
                _component_id: &ComponentId,
                _component_type: ComponentType,
                _user_id: &UserId,
                _system_prompt: &str,
            ) -> Result<ConversationRecord, DomainError> {
                unimplemented!()
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
                _conversation_id: &ConversationId,
            ) -> Result<Option<ConversationRecord>, DomainError> {
                Ok(None)
            }

            async fn get_messages(
                &self,
                _conversation_id: &ConversationId,
                _offset: u32,
                _limit: u32,
            ) -> Result<(Vec<StoredMessage>, u32), DomainError> {
                Ok((vec![], 0))
            }
        }

        #[test]
        fn ws_state_creates_correctly() {
            let repo = Arc::new(MockConversationRepo);
            let checker = Arc::new(MockOwnershipChecker);

            let state = ConversationWebSocketState::new(repo, checker);

            // Just verify it creates without panic
            let _ = state;
        }
    }
}
