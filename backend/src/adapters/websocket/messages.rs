//! WebSocket message types for real-time dashboard updates.
//!
//! Defines the protocol between server and connected clients:
//! - Server → Client: Connection status, dashboard updates, errors, pings
//! - Client → Server: Pings, state requests

use serde::{Deserialize, Serialize};

use crate::domain::foundation::{ComponentType, Timestamp};

// ============================================
// Server → Client Messages
// ============================================

/// All message types that can be sent from server to client.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Connection established successfully.
    Connected(ConnectedMessage),

    /// Dashboard update notification.
    #[serde(rename = "dashboard.update")]
    DashboardUpdate(DashboardUpdateMessage),

    /// Error occurred.
    Error(ErrorMessage),

    /// Heartbeat response.
    Pong(PongMessage),
}

/// Sent when client successfully connects and joins a session room.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedMessage {
    pub session_id: String,
    pub client_id: String,
    pub timestamp: String,
}

/// Dashboard update notification with typed payload.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardUpdateMessage {
    pub update_type: DashboardUpdateType,
    pub data: serde_json::Value,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
}

/// Types of dashboard updates that can be sent to clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardUpdateType {
    /// Session title/description changed.
    SessionMetadata,
    /// New cycle created.
    CycleCreated,
    /// Cycle progress changed.
    CycleProgress,
    /// Component work began.
    ComponentStarted,
    /// Component finished.
    ComponentCompleted,
    /// Component output updated.
    ComponentOutput,
    /// New chat message.
    ConversationMessage,
    /// Pugh/DQ scores computed.
    AnalysisScores,
    /// Cycle finished.
    CycleCompleted,
}

/// Error message sent to client.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorMessage {
    pub code: String,
    pub message: String,
    pub timestamp: String,
}

/// Heartbeat response.
#[derive(Debug, Clone, Serialize)]
pub struct PongMessage {
    pub timestamp: String,
}

// ============================================
// Client → Server Messages
// ============================================

/// All message types that can be received from client.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Heartbeat request.
    Ping,

    /// Request full dashboard state (after reconnection).
    #[serde(rename = "request.state")]
    RequestState,
}

// ============================================
// Internal Types
// ============================================

/// Internal representation of a dashboard update for broadcasting.
///
/// This is what the event bridge creates and sends to rooms.
#[derive(Debug, Clone)]
pub struct DashboardUpdate {
    pub update_type: DashboardUpdateType,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
    pub correlation_id: Option<String>,
}

impl DashboardUpdate {
    /// Convert to a server message for sending to clients.
    pub fn to_server_message(self) -> ServerMessage {
        ServerMessage::DashboardUpdate(DashboardUpdateMessage {
            update_type: self.update_type,
            data: self.data,
            timestamp: self.timestamp.as_datetime().to_rfc3339(),
            correlation_id: self.correlation_id,
        })
    }
}

// ============================================
// Payload Types for Specific Update Types
// ============================================

/// Payload for component completion updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentCompletedData {
    pub cycle_id: String,
    pub component_type: ComponentType,
    pub completed_at: String,
    pub progress: ProgressInfo,
}

/// Progress information for a cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub completed: u8,
    pub total: u8,
    pub percent: u8,
}

/// Payload for new conversation message updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationMessageData {
    pub cycle_id: String,
    pub component_type: ComponentType,
    pub message: MessagePreview,
}

/// Preview of a message (truncated for safety).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessagePreview {
    pub id: String,
    pub role: MessageRole,
    pub content_preview: String,
    pub timestamp: String,
}

/// Message role (user or assistant).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

/// Payload for analysis score updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisScoresData {
    pub cycle_id: String,
    pub score_type: ScoreType,
    pub scores: std::collections::HashMap<String, f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overall_score: Option<f64>,
}

/// Type of analysis score.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScoreType {
    Pugh,
    Dq,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_message_serializes_with_type_tag() {
        let msg = ServerMessage::Connected(ConnectedMessage {
            session_id: "session-123".to_string(),
            client_id: "client-456".to_string(),
            timestamp: "2025-01-10T00:00:00Z".to_string(),
        });

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"connected""#));
        assert!(json.contains(r#""sessionId":"session-123""#));
    }

    #[test]
    fn dashboard_update_message_serializes_correctly() {
        let msg = ServerMessage::DashboardUpdate(DashboardUpdateMessage {
            update_type: DashboardUpdateType::ComponentCompleted,
            data: serde_json::json!({"cycleId": "cycle-123"}),
            timestamp: "2025-01-10T00:00:00Z".to_string(),
            correlation_id: Some("req-789".to_string()),
        });

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"dashboard.update""#));
        assert!(json.contains(r#""updateType":"component_completed""#));
    }

    #[test]
    fn client_message_deserializes_ping() {
        let json = r#"{"type": "ping"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Ping));
    }

    #[test]
    fn client_message_deserializes_request_state() {
        let json = r#"{"type": "request.state"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::RequestState));
    }

    #[test]
    fn dashboard_update_converts_to_server_message() {
        let update = DashboardUpdate {
            update_type: DashboardUpdateType::CycleCreated,
            data: serde_json::json!({"cycleId": "cycle-123"}),
            timestamp: Timestamp::now(),
            correlation_id: None,
        };

        let msg = update.to_server_message();
        assert!(matches!(msg, ServerMessage::DashboardUpdate(_)));
    }

    #[test]
    fn error_message_serializes_correctly() {
        let msg = ServerMessage::Error(ErrorMessage {
            code: "AUTH_FAILED".to_string(),
            message: "Authentication required".to_string(),
            timestamp: "2025-01-10T00:00:00Z".to_string(),
        });

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"error""#));
        assert!(json.contains(r#""code":"AUTH_FAILED""#));
    }
}
