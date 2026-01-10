//! WebSocket streaming handler for AI Engine conversations
//!
//! Provides real-time streaming of AI responses over WebSocket connections.

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tokio::sync::mpsc;

use crate::application::handlers::SendMessageCommand;
use crate::domain::foundation::CycleId;

use super::handlers::AIEngineAppState;

// ════════════════════════════════════════════════════════════════════════════════
// WebSocket Message Types
// ════════════════════════════════════════════════════════════════════════════════

/// Client-to-server WebSocket messages
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Send a message and receive streaming response
    SendMessage { content: String },
    /// Ping for keep-alive
    Ping,
}

/// Server-to-client WebSocket messages
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Chunk of streaming response
    StreamChunk { delta: String, is_final: bool },
    /// Complete response with metadata
    StreamComplete {
        full_content: String,
        current_step: String,
        turn_count: u32,
    },
    /// Error during streaming
    StreamError { error: String },
    /// Pong response to ping
    Pong { timestamp: String },
}

// ════════════════════════════════════════════════════════════════════════════════
// WebSocket Handler
// ════════════════════════════════════════════════════════════════════════════════

/// WebSocket upgrade handler
///
/// GET /ai/conversations/{cycle_id}/stream
pub async fn stream_conversation(
    State(app_state): State<AIEngineAppState>,
    Path(cycle_id): Path<String>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    // Parse cycle ID
    let cycle_id = match CycleId::from_str(&cycle_id) {
        Ok(id) => id,
        Err(_) => {
            // Return 400 Bad Request if cycle_id is invalid
            return (
                axum::http::StatusCode::BAD_REQUEST,
                "Invalid cycle_id format",
            )
                .into_response();
        }
    };

    // Upgrade to WebSocket
    ws.on_upgrade(move |socket| handle_socket(socket, cycle_id, app_state))
        .into_response()
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, cycle_id: CycleId, app_state: AIEngineAppState) {
    let (mut sender, mut receiver) = socket.split();

    // Create channel for streaming responses
    let (tx, mut rx) = mpsc::channel::<ServerMessage>(100);

    // Spawn task to send messages to client
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = match serde_json::to_string(&msg) {
                Ok(json) => json,
                Err(e) => {
                    eprintln!("Failed to serialize message: {}", e);
                    continue;
                }
            };

            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        };

        match msg {
            Message::Text(text) => {
                if let Err(e) = handle_text_message(&text, cycle_id, &app_state, tx.clone()).await
                {
                    eprintln!("Error handling message: {}", e);
                    let _ = tx
                        .send(ServerMessage::StreamError {
                            error: e.to_string(),
                        })
                        .await;
                }
            }
            Message::Close(_) => {
                break;
            }
            Message::Ping(data) => {
                // Axum handles pong automatically
                let _ = tx
                    .send(ServerMessage::Pong {
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    })
                    .await;
                drop(data);
            }
            _ => {
                // Ignore binary messages
            }
        }
    }

    // Cleanup
    send_task.abort();
}

/// Handle text message from client
async fn handle_text_message(
    text: &str,
    cycle_id: CycleId,
    app_state: &AIEngineAppState,
    tx: mpsc::Sender<ServerMessage>,
) -> Result<(), String> {
    let client_msg: ClientMessage =
        serde_json::from_str(text).map_err(|e| format!("Invalid JSON: {}", e))?;

    match client_msg {
        ClientMessage::SendMessage { content } => {
            handle_send_message(cycle_id, content, app_state, tx).await?;
        }
        ClientMessage::Ping => {
            tx.send(ServerMessage::Pong {
                timestamp: chrono::Utc::now().to_rfc3339(),
            })
            .await
            .map_err(|e| format!("Failed to send pong: {}", e))?;
        }
    }

    Ok(())
}

/// Handle send message command with streaming
async fn handle_send_message(
    cycle_id: CycleId,
    content: String,
    app_state: &AIEngineAppState,
    tx: mpsc::Sender<ServerMessage>,
) -> Result<(), String> {
    // Validate message
    if content.trim().is_empty() {
        return Err("Message cannot be empty".to_string());
    }

    // Create command
    let cmd = SendMessageCommand {
        cycle_id,
        message: content,
    };

    // Execute command
    let handler = app_state.send_message_handler();
    let result = handler.handle(cmd).await.map_err(|e| e.to_string())?;

    // For now, stream the response in chunks (simulating streaming)
    // TODO: Integrate with actual AI provider streaming
    let response = result.ai_response;
    let chunk_size = 20; // Characters per chunk

    for (i, chunk) in response
        .chars()
        .collect::<Vec<char>>()
        .chunks(chunk_size)
        .enumerate()
    {
        let delta: String = chunk.iter().collect();
        let is_final = i * chunk_size + chunk.len() >= response.len();

        tx.send(ServerMessage::StreamChunk { delta, is_final })
            .await
            .map_err(|e| format!("Failed to send chunk: {}", e))?;

        // Simulate streaming delay
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Send complete message
    tx.send(ServerMessage::StreamComplete {
        full_content: response,
        current_step: format!("{:?}", result.updated_state.current_step),
        turn_count: result
            .updated_state
            .step_state(result.updated_state.current_step)
            .map(|s| s.turn_count)
            .unwrap_or(0),
    })
    .await
    .map_err(|e| format!("Failed to send complete: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_deserialization() {
        let json = r#"{"type":"send_message","content":"Hello"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::SendMessage { content } => assert_eq!(content, "Hello"),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_client_ping_deserialization() {
        let json = r#"{"type":"ping"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::Ping => {}
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_server_message_serialization() {
        let msg = ServerMessage::StreamChunk {
            delta: "Hello".to_string(),
            is_final: false,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("stream_chunk"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_stream_complete_serialization() {
        let msg = ServerMessage::StreamComplete {
            full_content: "Full response".to_string(),
            current_step: "IssueRaising".to_string(),
            turn_count: 5,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("stream_complete"));
        assert!(json.contains("Full response"));
        assert!(json.contains("IssueRaising"));
    }

    #[test]
    fn test_error_message_serialization() {
        let msg = ServerMessage::StreamError {
            error: "Connection lost".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("stream_error"));
        assert!(json.contains("Connection lost"));
    }
}
