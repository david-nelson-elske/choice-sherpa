//! WebSocket upgrade handler for real-time dashboard connections.
//!
//! Handles the HTTP → WebSocket upgrade and manages the connection lifecycle:
//! 1. Validate session exists and user has access
//! 2. Upgrade to WebSocket
//! 3. Join session room
//! 4. Send/receive messages until disconnect
//! 5. Clean up room membership

use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;

use crate::domain::foundation::{SessionId, Timestamp};

use super::{
    messages::{ClientMessage, ConnectedMessage, ServerMessage},
    rooms::{ClientId, RoomManager},
    DashboardUpdate,
};

/// State required for WebSocket handling.
///
/// Extracted from the application state.
#[derive(Clone)]
pub struct WebSocketState {
    /// Room manager for session-based routing.
    pub room_manager: Arc<RoomManager>,
    // TODO: Add session repository for validation
    // TODO: Add auth provider for user validation
}

impl WebSocketState {
    /// Create a new WebSocket state.
    pub fn new(room_manager: Arc<RoomManager>) -> Self {
        Self { room_manager }
    }
}

/// Handle WebSocket upgrade requests for session dashboard.
///
/// Route: `GET /api/sessions/:session_id/live`
///
/// # Security
///
/// Currently performs minimal validation. Production should:
/// - Validate user authentication (via cookie or query param token)
/// - Verify user has access to the session
/// - Check rate limits
/// - Validate origin header
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(session_id): Path<String>,
    State(state): State<WebSocketState>,
) -> Response {
    // Parse session ID
    let session_id: SessionId = match session_id.parse() {
        Ok(id) => id,
        Err(_) => {
            return Response::builder()
                .status(400)
                .body("Invalid session ID".into())
                .unwrap();
        }
    };

    // TODO: Validate user has access to session
    // let user_id = authenticate_request(&request)?;
    // authorize_session_access(&user_id, &session_id)?;

    // Upgrade to WebSocket
    ws.on_upgrade(move |socket| handle_socket(socket, session_id, state))
}

/// Handle an established WebSocket connection.
///
/// This function runs for the lifetime of the connection, handling:
/// - Joining the session room
/// - Forwarding room broadcasts to the client
/// - Processing client messages (ping, request state)
/// - Cleanup on disconnect
async fn handle_socket(socket: WebSocket, session_id: SessionId, state: WebSocketState) {
    let (mut sender, mut receiver) = socket.split();

    // Generate client ID
    let client_id = ClientId::new();

    // Join session room
    let mut room_rx: broadcast::Receiver<DashboardUpdate> = state
        .room_manager
        .join(&session_id, client_id.clone())
        .await;

    // Send connected message
    let connected = ServerMessage::Connected(ConnectedMessage {
        session_id: session_id.to_string(),
        client_id: client_id.to_string(),
        timestamp: Timestamp::now().as_datetime().to_rfc3339(),
    });

    if let Err(e) = send_message(&mut sender, &connected).await {
        tracing::debug!("Failed to send connected message: {}", e);
        return; // Client disconnected immediately
    }

    // Spawn task to forward room broadcasts to client
    let mut send_task = {
        let client_id_clone = client_id.clone();
        tokio::spawn(async move {
            while let Ok(update) = room_rx.recv().await {
                let msg = update.to_server_message();
                if let Err(e) = send_message(&mut sender, &msg).await {
                    tracing::debug!(
                        client_id = %client_id_clone,
                        "Send error, closing connection: {}",
                        e
                    );
                    break;
                }
            }
        })
    };

    // Handle incoming messages from client
    let room_manager = state.room_manager.clone();
    let client_id_for_recv = client_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        match client_msg {
                            ClientMessage::Ping => {
                                // Pong is handled in the send task via room broadcast
                                // But we should respond directly here
                                tracing::trace!(
                                    client_id = %client_id_for_recv,
                                    "Received ping"
                                );
                            }
                            ClientMessage::RequestState => {
                                // TODO: Fetch and send full dashboard state
                                tracing::debug!(
                                    client_id = %client_id_for_recv,
                                    "State request received (not implemented)"
                                );
                            }
                        }
                    }
                }
                Ok(Message::Binary(_)) => {
                    // Binary messages not supported
                    tracing::warn!(
                        client_id = %client_id_for_recv,
                        "Received unsupported binary message"
                    );
                }
                Ok(Message::Ping(_)) => {
                    // WebSocket protocol ping - handled automatically by axum
                }
                Ok(Message::Pong(_)) => {
                    // WebSocket protocol pong - handled automatically by axum
                }
                Ok(Message::Close(_)) => {
                    tracing::debug!(
                        client_id = %client_id_for_recv,
                        "Client sent close frame"
                    );
                    break;
                }
                Err(e) => {
                    tracing::debug!(
                        client_id = %client_id_for_recv,
                        "Receive error: {}",
                        e
                    );
                    break;
                }
            }
        }

        // Return room_manager for cleanup
        room_manager
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        }
        result = &mut recv_task => {
            send_task.abort();
            // Cleanup: leave room
            if let Ok(room_manager) = result {
                room_manager.leave(&client_id).await;
            }
            return;
        }
    }

    // Cleanup: leave room (send_task finished first)
    state.room_manager.leave(&client_id).await;
}

/// Send a JSON message over the WebSocket.
async fn send_message(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    msg: &ServerMessage,
) -> Result<(), axum::Error> {
    let json = serde_json::to_string(msg).expect("ServerMessage serialization should not fail");
    sender.send(Message::Text(json)).await
}

/// Create axum router for WebSocket endpoint.
///
/// # Example
///
/// ```ignore
/// let app = Router::new()
///     .nest("/api", websocket_router())
///     .with_state(app_state);
/// ```
pub fn websocket_router() -> axum::Router<WebSocketState> {
    use axum::routing::get;

    axum::Router::new().route("/sessions/{session_id}/live", get(ws_handler))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn websocket_state_creates_successfully() {
        let room_manager = Arc::new(RoomManager::default());
        let state = WebSocketState::new(room_manager.clone());

        // Verify room manager is shared
        assert!(Arc::ptr_eq(&state.room_manager, &room_manager));
    }

    #[test]
    fn websocket_router_creates_route() {
        let _router = websocket_router();
        // Basic smoke test - router should create without panic
    }
}
