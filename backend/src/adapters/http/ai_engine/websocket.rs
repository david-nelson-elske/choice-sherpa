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

use crate::domain::ai_engine::{conversation_state::MessageRole, step_agent, ConversationState};
use crate::domain::foundation::{ComponentType, ConversationId, CycleId, UserId};
use crate::ports::{
    CompletionRequest, Message as AIMessage, MessageRole as AIMessageRole, RequestMetadata,
};

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
        #[serde(skip_serializing_if = "Option::is_none")]
        prompt_tokens: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        completion_tokens: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        estimated_cost_cents: Option<u32>,
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

    // 1. Load existing conversation state
    let mut state = app_state
        .storage
        .load_state(cycle_id)
        .await
        .map_err(|e| format!("Failed to load state: {}", e))?;

    // 2. Add user message to history
    state.add_message(MessageRole::User, content.clone());

    // 3. Build system prompt from step agent spec
    let system_prompt = build_system_prompt(state.current_step);

    // 4. Convert conversation history to AI messages
    let messages = convert_messages_to_ai_format(&state);

    // 5. Build request metadata
    let metadata = RequestMetadata::new(
        UserId::new("system").unwrap(), // TODO: Get actual user_id from context
        state.session_id,
        ConversationId::new(), // TODO: Map CycleId to ConversationId
        format!("ws-cycle-{}", state.cycle_id),
    );

    // 6. Build completion request
    let mut request = CompletionRequest::new(metadata)
        .with_system_prompt(system_prompt)
        .with_max_tokens(2000)
        .with_temperature(0.7)
        .with_component_type(state.current_step);

    // Add messages
    for msg in messages {
        request = request.with_message(msg.role, msg.content);
    }

    // 7. Start streaming from AI provider
    let mut stream = app_state
        .ai_provider
        .stream_complete(request)
        .await
        .map_err(|e| format!("AI provider error: {}", e))?;

    // 8. Stream chunks to client
    let mut full_response = String::new();
    let mut token_usage = None;

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                // Accumulate response
                full_response.push_str(&chunk.delta);

                // Check if final before moving chunk
                let is_final = chunk.is_final();

                // Capture token usage from final chunk
                if let Some(usage) = chunk.usage {
                    token_usage = Some(usage);
                }

                // Send chunk to client
                tx.send(ServerMessage::StreamChunk {
                    delta: chunk.delta,
                    is_final,
                })
                .await
                .map_err(|e| format!("Failed to send chunk: {}", e))?;
            }
            Err(e) => {
                // Send error to client
                tx.send(ServerMessage::StreamError {
                    error: format!("Streaming error: {}", e),
                })
                .await
                .map_err(|e| format!("Failed to send error: {}", e))?;

                return Err(format!("AI streaming error: {}", e));
            }
        }
    }

    // 9. Add AI response to conversation history
    state.add_message(MessageRole::Assistant, full_response.clone());

    // 10. Persist updated state
    app_state
        .storage
        .save_state(cycle_id, &state)
        .await
        .map_err(|e| format!("Failed to save state: {}", e))?;

    // 11. Send completion message with token usage
    let (prompt_tokens, completion_tokens, cost_cents) = token_usage
        .map(|u| {
            (
                Some(u.prompt_tokens),
                Some(u.completion_tokens),
                Some(u.estimated_cost_cents),
            )
        })
        .unwrap_or((None, None, None));

    tx.send(ServerMessage::StreamComplete {
        full_content: full_response,
        current_step: format!("{:?}", state.current_step),
        turn_count: state
            .step_state(state.current_step)
            .map(|s| s.turn_count)
            .unwrap_or(0),
        prompt_tokens,
        completion_tokens,
        estimated_cost_cents: cost_cents,
    })
    .await
    .map_err(|e| format!("Failed to send complete: {}", e))?;

    Ok(())
}

/// Build system prompt from step agent specification
fn build_system_prompt(component: ComponentType) -> String {
    let spec = step_agent::agents::get(component)
        .expect("All component types should have agent specs");

    format!(
        "You are a thoughtful decision professional helping users work through the {} phase of their decision-making process.\n\n\
        Role: {}\n\n\
        Objectives:\n{}\n\n\
        Techniques:\n{}\n\n\
        Guide the user through this phase with probing questions and thoughtful reflection. \
        Do not make decisions for them - help them think clearly about their situation.",
        spec.component.to_string().to_lowercase().replace('_', " "),
        spec.role,
        spec.objectives
            .iter()
            .map(|o| format!("- {}", o))
            .collect::<Vec<_>>()
            .join("\n"),
        spec.techniques
            .iter()
            .map(|t| format!("- {}", t))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

/// Convert conversation history to AI provider message format
fn convert_messages_to_ai_format(state: &ConversationState) -> Vec<AIMessage> {
    state
        .message_history
        .iter()
        .map(|msg| {
            let role = match msg.role {
                MessageRole::System => AIMessageRole::System,
                MessageRole::User => AIMessageRole::User,
                MessageRole::Assistant => AIMessageRole::Assistant,
            };
            AIMessage::new(role, msg.content.clone())
        })
        .collect()
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
            prompt_tokens: Some(100),
            completion_tokens: Some(50),
            estimated_cost_cents: Some(15),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("stream_complete"));
        assert!(json.contains("Full response"));
        assert!(json.contains("IssueRaising"));
        assert!(json.contains("prompt_tokens"));
        assert!(json.contains("100"));
    }

    #[test]
    fn test_stream_complete_without_usage() {
        let msg = ServerMessage::StreamComplete {
            full_content: "Full response".to_string(),
            current_step: "IssueRaising".to_string(),
            turn_count: 5,
            prompt_tokens: None,
            completion_tokens: None,
            estimated_cost_cents: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("stream_complete"));
        assert!(json.contains("Full response"));
        // Token fields should be omitted when None
        assert!(!json.contains("prompt_tokens"));
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
