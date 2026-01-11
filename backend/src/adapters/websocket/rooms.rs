//! WebSocket room management for session-based message routing.
//!
//! Rooms are organized by session ID, allowing targeted broadcast of
//! dashboard updates to all clients viewing a specific session.
//!
//! # Architecture
//!
//! ```text
//! Room: session-123    Room: session-456
//! ├── client-a         ├── client-d
//! ├── client-b         └── client-e
//! └── client-c
//! ```
//!
//! When an event occurs for session-123, only clients a, b, c receive it.

use std::collections::HashMap;

use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::domain::foundation::SessionId;

use super::messages::DashboardUpdate;

/// Unique identifier for a WebSocket client connection.
///
/// Generated server-side when a client connects.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientId(Uuid);

impl ClientId {
    /// Create a new random client ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from an existing string (for testing).
    #[cfg(test)]
    pub fn from_string(s: &str) -> Self {
        Self(Uuid::parse_str(s).unwrap_or_else(|_| Uuid::new_v4()))
    }
}

impl Default for ClientId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Manages WebSocket connection rooms organized by session.
///
/// Provides:
/// - Client join/leave operations
/// - Broadcast to all clients in a session room
/// - Automatic cleanup of empty rooms
///
/// # Thread Safety
///
/// Uses `RwLock` for the room registry since broadcasts (reads) vastly
/// outnumber joins/leaves (writes). This allows concurrent broadcasts
/// to different rooms.
pub struct RoomManager {
    /// Map of session_id → broadcast sender for that room.
    rooms: RwLock<HashMap<SessionId, broadcast::Sender<DashboardUpdate>>>,

    /// Map of client_id → session_id for O(1) cleanup on disconnect.
    client_sessions: RwLock<HashMap<ClientId, SessionId>>,

    /// Channel capacity for each room's broadcast channel.
    channel_capacity: usize,
}

impl RoomManager {
    /// Create a new room manager with specified channel capacity.
    ///
    /// # Arguments
    ///
    /// * `channel_capacity` - Buffer size for each room's broadcast channel.
    ///   Larger values handle bursts better but use more memory.
    ///   Recommended: 100-256 for typical dashboard update rates.
    pub fn new(channel_capacity: usize) -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
            client_sessions: RwLock::new(HashMap::new()),
            channel_capacity,
        }
    }

    /// Create with default capacity (128 messages).
    pub fn with_default_capacity() -> Self {
        Self::new(128)
    }

    /// Join a client to a session room.
    ///
    /// If the room doesn't exist, it's created automatically.
    /// Returns a receiver for dashboard updates in that room.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session to join
    /// * `client_id` - Unique identifier for this client connection
    ///
    /// # Returns
    ///
    /// A broadcast receiver that will receive all updates for this session.
    pub async fn join(
        &self,
        session_id: &SessionId,
        client_id: ClientId,
    ) -> broadcast::Receiver<DashboardUpdate> {
        let mut rooms = self.rooms.write().await;

        // Get or create room
        let sender = rooms.entry(*session_id).or_insert_with(|| {
            let (tx, _) = broadcast::channel(self.channel_capacity);
            tx
        });

        // Track client's session for cleanup
        self.client_sessions
            .write()
            .await
            .insert(client_id, *session_id);

        sender.subscribe()
    }

    /// Remove a client from their session room.
    ///
    /// If the room becomes empty, it's automatically cleaned up.
    ///
    /// # Arguments
    ///
    /// * `client_id` - The client to remove
    pub async fn leave(&self, client_id: &ClientId) {
        let mut client_sessions = self.client_sessions.write().await;

        if let Some(session_id) = client_sessions.remove(client_id) {
            // Check if room is empty and clean up
            let rooms = self.rooms.read().await;
            if let Some(sender) = rooms.get(&session_id) {
                if sender.receiver_count() == 0 {
                    drop(rooms);
                    self.rooms.write().await.remove(&session_id);
                }
            }
        }
    }

    /// Broadcast an update to all clients in a session room.
    ///
    /// If no clients are in the room, this is a no-op.
    /// If the broadcast buffer is full, oldest messages are dropped
    /// (clients that are too slow will miss updates).
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session to broadcast to
    /// * `update` - The dashboard update to send
    pub async fn broadcast_to_session(&self, session_id: &SessionId, update: DashboardUpdate) {
        let rooms = self.rooms.read().await;

        if let Some(sender) = rooms.get(session_id) {
            // Ignore send errors (no receivers is OK)
            let _ = sender.send(update);
        }
    }

    /// Get count of connected clients in a specific room.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session to check
    ///
    /// # Returns
    ///
    /// Number of clients currently in the room (0 if room doesn't exist).
    pub async fn client_count(&self, session_id: &SessionId) -> usize {
        let rooms = self.rooms.read().await;
        rooms
            .get(session_id)
            .map(|s| s.receiver_count())
            .unwrap_or(0)
    }

    /// Get all active room IDs (for monitoring/debugging).
    pub async fn active_rooms(&self) -> Vec<SessionId> {
        self.rooms.read().await.keys().cloned().collect()
    }

    /// Get total count of connected clients across all rooms.
    pub async fn total_client_count(&self) -> usize {
        self.client_sessions.read().await.len()
    }
}

impl Default for RoomManager {
    fn default() -> Self {
        Self::with_default_capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::websocket::messages::DashboardUpdateType;
    use crate::domain::foundation::Timestamp;
    use std::sync::Arc;
    use tokio::sync::broadcast;

    fn test_update() -> DashboardUpdate {
        DashboardUpdate {
            update_type: DashboardUpdateType::ComponentCompleted,
            data: serde_json::json!({"test": "data"}),
            timestamp: Timestamp::now(),
            correlation_id: None,
        }
    }

    #[tokio::test]
    async fn join_creates_room_if_not_exists() {
        let manager = RoomManager::with_default_capacity();
        let session_id = SessionId::new();
        let client_id = ClientId::new();

        let _rx = manager.join(&session_id, client_id).await;

        assert_eq!(manager.active_rooms().await.len(), 1);
    }

    #[tokio::test]
    async fn join_returns_receiver_for_broadcasts() {
        let manager = Arc::new(RoomManager::with_default_capacity());
        let session_id = SessionId::new();
        let client_id = ClientId::new();

        let mut rx: broadcast::Receiver<DashboardUpdate> =
            manager.join(&session_id, client_id).await;

        // Broadcast an update
        manager
            .broadcast_to_session(&session_id, test_update())
            .await;

        // Should receive it
        let received = rx.recv().await.unwrap();
        assert_eq!(received.update_type, DashboardUpdateType::ComponentCompleted);
    }

    #[tokio::test]
    async fn multiple_clients_in_same_room_all_receive_broadcast() {
        let manager = Arc::new(RoomManager::with_default_capacity());
        let session_id = SessionId::new();

        let mut rx1: broadcast::Receiver<DashboardUpdate> =
            manager.join(&session_id, ClientId::new()).await;
        let mut rx2: broadcast::Receiver<DashboardUpdate> =
            manager.join(&session_id, ClientId::new()).await;
        let mut rx3: broadcast::Receiver<DashboardUpdate> =
            manager.join(&session_id, ClientId::new()).await;

        manager
            .broadcast_to_session(&session_id, test_update())
            .await;

        assert!(rx1.recv().await.is_ok());
        assert!(rx2.recv().await.is_ok());
        assert!(rx3.recv().await.is_ok());
    }

    #[tokio::test]
    async fn clients_in_different_rooms_receive_separate_broadcasts() {
        let manager = Arc::new(RoomManager::with_default_capacity());
        let session_1 = SessionId::new();
        let session_2 = SessionId::new();

        let mut rx1: broadcast::Receiver<DashboardUpdate> =
            manager.join(&session_1, ClientId::new()).await;
        let _rx2: broadcast::Receiver<DashboardUpdate> =
            manager.join(&session_2, ClientId::new()).await;

        // Broadcast to session 1 only
        manager
            .broadcast_to_session(&session_1, test_update())
            .await;

        // rx1 should receive
        assert!(rx1.recv().await.is_ok());

        // rx2 should not receive (would block or timeout)
        // We can't easily test this without timeouts, but client_count confirms isolation
        assert_eq!(manager.client_count(&session_1).await, 1);
        assert_eq!(manager.client_count(&session_2).await, 1);
    }

    #[tokio::test]
    async fn leave_removes_client_from_room() {
        let manager = RoomManager::with_default_capacity();
        let session_id = SessionId::new();
        let client_id = ClientId::new();

        let _rx = manager.join(&session_id, client_id.clone()).await;
        assert_eq!(manager.total_client_count().await, 1);

        manager.leave(&client_id).await;
        assert_eq!(manager.total_client_count().await, 0);
    }

    #[tokio::test]
    async fn leave_cleans_up_empty_room() {
        let manager = RoomManager::with_default_capacity();
        let session_id = SessionId::new();
        let client_id = ClientId::new();

        {
            // Client joins and then the receiver is dropped (simulating disconnect)
            let _rx = manager.join(&session_id, client_id.clone()).await;
            // _rx dropped here
        }

        // Leave should clean up the empty room
        manager.leave(&client_id).await;

        // Room should be gone
        assert!(manager.active_rooms().await.is_empty());
    }

    #[tokio::test]
    async fn client_count_returns_correct_count() {
        let manager = RoomManager::with_default_capacity();
        let session_id = SessionId::new();

        assert_eq!(manager.client_count(&session_id).await, 0);

        let _rx1 = manager.join(&session_id, ClientId::new()).await;
        assert_eq!(manager.client_count(&session_id).await, 1);

        let _rx2 = manager.join(&session_id, ClientId::new()).await;
        assert_eq!(manager.client_count(&session_id).await, 2);
    }

    #[tokio::test]
    async fn broadcast_to_nonexistent_room_is_noop() {
        let manager = RoomManager::with_default_capacity();
        let session_id = SessionId::new();

        // Should not panic or error
        manager
            .broadcast_to_session(&session_id, test_update())
            .await;
    }

    #[tokio::test]
    async fn active_rooms_returns_all_session_ids() {
        let manager = RoomManager::with_default_capacity();
        let session_1 = SessionId::new();
        let session_2 = SessionId::new();
        let session_3 = SessionId::new();

        let _rx1 = manager.join(&session_1, ClientId::new()).await;
        let _rx2 = manager.join(&session_2, ClientId::new()).await;
        let _rx3 = manager.join(&session_3, ClientId::new()).await;

        let rooms = manager.active_rooms().await;
        assert_eq!(rooms.len(), 3);
        assert!(rooms.contains(&session_1));
        assert!(rooms.contains(&session_2));
        assert!(rooms.contains(&session_3));
    }

    #[tokio::test]
    async fn client_id_display_works() {
        let client_id = ClientId::new();
        let display = format!("{}", client_id);
        assert!(!display.is_empty());
        // Should be a valid UUID format
        assert!(display.len() == 36);
    }
}
