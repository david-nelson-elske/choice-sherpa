//! WebSocket streaming message types for conversation endpoints.
//!
//! Defines the protocol between server and connected clients for AI streaming:
//! - Client → Server: SendMessage, CancelStream, Ping
//! - Server → Client: StreamChunk, StreamComplete, StreamError, Pong, DataExtracted

use serde::{Deserialize, Serialize};

use crate::domain::conversation::AgentPhase;
use crate::domain::foundation::ComponentType;

// ════════════════════════════════════════════════════════════════════════════════
// Client → Server Messages
// ════════════════════════════════════════════════════════════════════════════════

/// All message types that can be received from client.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamClientMessage {
    /// Send a user message to the AI.
    SendMessage(SendMessageRequest),
    /// Cancel an in-progress stream.
    CancelStream(CancelStreamRequest),
    /// Heartbeat ping.
    Ping,
}

/// Request to send a user message.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SendMessageRequest {
    /// Client-generated UUID for tracking.
    pub message_id: String,
    /// User's message text (max 10,000 chars).
    pub content: String,
}

/// Request to cancel an in-progress stream.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CancelStreamRequest {
    /// Message ID of the stream to cancel.
    pub message_id: String,
}

// ════════════════════════════════════════════════════════════════════════════════
// Server → Client Messages
// ════════════════════════════════════════════════════════════════════════════════

/// All message types that can be sent from server to client.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamServerMessage {
    /// Partial AI response content.
    StreamChunk(StreamChunkMessage),
    /// Stream completed successfully.
    StreamComplete(StreamCompleteMessage),
    /// Error during streaming.
    StreamError(StreamErrorMessage),
    /// Heartbeat response.
    Pong(StreamPongMessage),
    /// Structured data extracted from conversation.
    DataExtracted(DataExtractedMessage),
}

/// Partial AI response content delivered incrementally.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StreamChunkMessage {
    /// Matches request message_id.
    pub message_id: String,
    /// Incremental text content.
    pub delta: String,
    /// True if this is the last chunk.
    pub is_final: bool,
}

/// Sent after the final chunk with usage statistics.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StreamCompleteMessage {
    /// Matches request message_id.
    pub message_id: String,
    /// Complete assembled response.
    pub full_content: String,
    /// Token usage for this response.
    pub usage: StreamTokenUsage,
    /// If agent phase changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_transition: Option<PhaseTransition>,
}

/// Token usage statistics for a streaming response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StreamTokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub estimated_cost_cents: u32,
}

/// Agent phase transition notification.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PhaseTransition {
    pub from_phase: AgentPhase,
    pub to_phase: AgentPhase,
}

/// Error during streaming.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StreamErrorMessage {
    /// Matches request message_id.
    pub message_id: String,
    /// Error code for programmatic handling.
    pub error_code: StreamErrorCode,
    /// Human-readable error message.
    pub error: String,
    /// Content received before error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_content: Option<String>,
    /// Whether retry is recommended.
    pub recoverable: bool,
}

/// Error codes for stream errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamErrorCode {
    /// AI provider rate limit.
    RateLimited,
    /// Conversation exceeds token limit.
    ContextTooLong,
    /// AI response blocked by safety filter.
    ContentFiltered,
    /// AI provider unavailable.
    ProviderError,
    /// User cancelled the stream.
    Cancelled,
    /// Stream timed out.
    Timeout,
    /// Unexpected server error.
    InternalError,
}

impl StreamErrorCode {
    /// Returns true if this error is recoverable.
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::RateLimited | Self::ProviderError | Self::Timeout
        )
    }
}

/// Heartbeat response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StreamPongMessage {
    /// ISO 8601 timestamp.
    pub timestamp: String,
}

/// Notifies client that structured data was extracted.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DataExtractedMessage {
    /// Component type for the extracted data.
    pub component_type: ComponentType,
    /// Extracted structured data.
    pub data: serde_json::Value,
    /// ISO 8601 timestamp.
    pub extracted_at: String,
}

// ════════════════════════════════════════════════════════════════════════════════
// Message Validation
// ════════════════════════════════════════════════════════════════════════════════

/// Maximum allowed message length (10,000 characters).
pub const MAX_MESSAGE_LENGTH: usize = 10_000;

impl SendMessageRequest {
    /// Validates the message content.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.content.is_empty() {
            return Err("Message content cannot be empty");
        }
        if self.content.len() > MAX_MESSAGE_LENGTH {
            return Err("Message content exceeds maximum length");
        }
        if self.message_id.is_empty() {
            return Err("Message ID cannot be empty");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod client_messages {
        use super::*;

        #[test]
        fn deserializes_send_message() {
            let json = r#"{
                "type": "send_message",
                "message_id": "550e8400-e29b-41d4-a716-446655440000",
                "content": "Hello, AI!"
            }"#;

            let msg: StreamClientMessage = serde_json::from_str(json).unwrap();
            match msg {
                StreamClientMessage::SendMessage(req) => {
                    assert_eq!(req.message_id, "550e8400-e29b-41d4-a716-446655440000");
                    assert_eq!(req.content, "Hello, AI!");
                }
                _ => panic!("Expected SendMessage"),
            }
        }

        #[test]
        fn deserializes_cancel_stream() {
            let json = r#"{
                "type": "cancel_stream",
                "message_id": "550e8400-e29b-41d4-a716-446655440000"
            }"#;

            let msg: StreamClientMessage = serde_json::from_str(json).unwrap();
            match msg {
                StreamClientMessage::CancelStream(req) => {
                    assert_eq!(req.message_id, "550e8400-e29b-41d4-a716-446655440000");
                }
                _ => panic!("Expected CancelStream"),
            }
        }

        #[test]
        fn deserializes_ping() {
            let json = r#"{"type": "ping"}"#;
            let msg: StreamClientMessage = serde_json::from_str(json).unwrap();
            assert!(matches!(msg, StreamClientMessage::Ping));
        }
    }

    mod server_messages {
        use super::*;

        #[test]
        fn serializes_stream_chunk() {
            let msg = StreamServerMessage::StreamChunk(StreamChunkMessage {
                message_id: "abc".to_string(),
                delta: "Hello".to_string(),
                is_final: false,
            });

            let json = serde_json::to_string(&msg).unwrap();
            assert!(json.contains(r#""type":"stream_chunk""#));
            assert!(json.contains(r#""delta":"Hello""#));
            assert!(json.contains(r#""is_final":false"#));
        }

        #[test]
        fn serializes_stream_complete() {
            let msg = StreamServerMessage::StreamComplete(StreamCompleteMessage {
                message_id: "abc".to_string(),
                full_content: "Hello, world!".to_string(),
                usage: StreamTokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                    estimated_cost_cents: 1,
                },
                phase_transition: None,
            });

            let json = serde_json::to_string(&msg).unwrap();
            assert!(json.contains(r#""type":"stream_complete""#));
            assert!(json.contains(r#""full_content":"Hello, world!""#));
            assert!(!json.contains("phase_transition"));
        }

        #[test]
        fn serializes_stream_complete_with_phase_transition() {
            let msg = StreamServerMessage::StreamComplete(StreamCompleteMessage {
                message_id: "abc".to_string(),
                full_content: "Response".to_string(),
                usage: StreamTokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                    estimated_cost_cents: 1,
                },
                phase_transition: Some(PhaseTransition {
                    from_phase: AgentPhase::Intro,
                    to_phase: AgentPhase::Gather,
                }),
            });

            let json = serde_json::to_string(&msg).unwrap();
            assert!(json.contains("phase_transition"));
            assert!(json.contains("from_phase"));
            assert!(json.contains("to_phase"));
        }

        #[test]
        fn serializes_stream_error() {
            let msg = StreamServerMessage::StreamError(StreamErrorMessage {
                message_id: "abc".to_string(),
                error_code: StreamErrorCode::RateLimited,
                error: "Too many requests".to_string(),
                partial_content: None,
                recoverable: true,
            });

            let json = serde_json::to_string(&msg).unwrap();
            assert!(json.contains(r#""type":"stream_error""#));
            assert!(json.contains(r#""error_code":"rate_limited""#));
            assert!(json.contains(r#""recoverable":true"#));
        }

        #[test]
        fn serializes_pong() {
            let msg = StreamServerMessage::Pong(StreamPongMessage {
                timestamp: "2026-01-10T00:00:00Z".to_string(),
            });

            let json = serde_json::to_string(&msg).unwrap();
            assert!(json.contains(r#""type":"pong""#));
            assert!(json.contains(r#""timestamp":"2026-01-10T00:00:00Z""#));
        }

        #[test]
        fn serializes_data_extracted() {
            let msg = StreamServerMessage::DataExtracted(DataExtractedMessage {
                component_type: ComponentType::IssueRaising,
                data: serde_json::json!({
                    "decisions": [{"id": "d1", "description": "Test"}]
                }),
                extracted_at: "2026-01-10T00:00:00Z".to_string(),
            });

            let json = serde_json::to_string(&msg).unwrap();
            assert!(json.contains(r#""type":"data_extracted""#));
            assert!(json.contains(r#""component_type""#));
            assert!(json.contains(r#""decisions""#));
        }
    }

    mod error_codes {
        use super::*;

        #[test]
        fn rate_limited_is_recoverable() {
            assert!(StreamErrorCode::RateLimited.is_recoverable());
        }

        #[test]
        fn provider_error_is_recoverable() {
            assert!(StreamErrorCode::ProviderError.is_recoverable());
        }

        #[test]
        fn timeout_is_recoverable() {
            assert!(StreamErrorCode::Timeout.is_recoverable());
        }

        #[test]
        fn context_too_long_is_not_recoverable() {
            assert!(!StreamErrorCode::ContextTooLong.is_recoverable());
        }

        #[test]
        fn content_filtered_is_not_recoverable() {
            assert!(!StreamErrorCode::ContentFiltered.is_recoverable());
        }

        #[test]
        fn cancelled_is_not_recoverable() {
            assert!(!StreamErrorCode::Cancelled.is_recoverable());
        }

        #[test]
        fn internal_error_is_not_recoverable() {
            assert!(!StreamErrorCode::InternalError.is_recoverable());
        }
    }

    mod message_validation {
        use super::*;

        #[test]
        fn validates_valid_message() {
            let req = SendMessageRequest {
                message_id: "abc-123".to_string(),
                content: "Hello".to_string(),
            };
            assert!(req.validate().is_ok());
        }

        #[test]
        fn rejects_empty_content() {
            let req = SendMessageRequest {
                message_id: "abc-123".to_string(),
                content: "".to_string(),
            };
            assert_eq!(req.validate().err(), Some("Message content cannot be empty"));
        }

        #[test]
        fn rejects_empty_message_id() {
            let req = SendMessageRequest {
                message_id: "".to_string(),
                content: "Hello".to_string(),
            };
            assert_eq!(req.validate().err(), Some("Message ID cannot be empty"));
        }

        #[test]
        fn rejects_oversized_content() {
            let req = SendMessageRequest {
                message_id: "abc-123".to_string(),
                content: "x".repeat(MAX_MESSAGE_LENGTH + 1),
            };
            assert_eq!(
                req.validate().err(),
                Some("Message content exceeds maximum length")
            );
        }

        #[test]
        fn accepts_max_length_content() {
            let req = SendMessageRequest {
                message_id: "abc-123".to_string(),
                content: "x".repeat(MAX_MESSAGE_LENGTH),
            };
            assert!(req.validate().is_ok());
        }
    }
}
