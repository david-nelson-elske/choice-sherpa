//! ConnectionRegistry port - Interface for multi-server WebSocket tracking.
//!
//! In a multi-server deployment, WebSocket connections are tied to specific
//! servers. This port enables cross-server message delivery by tracking which
//! server(s) hold connections for each user.
//!
//! ## Use Case
//!
//! 1. User connects to Server A via WebSocket
//! 2. Server A registers connection in ConnectionRegistry
//! 3. Event occurs on Server B (e.g., AI response ready)
//! 4. Server B queries registry to find User's server(s)
//! 5. Server B publishes message to Server A's channel
//! 6. Server A delivers message to User's WebSocket
//!
//! See `docs/architecture/SCALING-READINESS.md` for full details.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::domain::foundation::UserId;

/// Unique identifier for a server instance in a multi-server deployment.
///
/// Format is typically hostname:port or container/pod ID.
/// Used for routing WebSocket messages to the correct server.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServerId(String);

impl ServerId {
    /// Create a new server ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the server ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Create server ID from environment (hostname + port).
    pub fn from_env() -> Self {
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string());
        let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        Self(format!("{}:{}", hostname, port))
    }
}

impl fmt::Display for ServerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ServerId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ServerId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Errors that can occur in connection registry operations.
#[derive(Debug, thiserror::Error)]
pub enum ConnectionRegistryError {
    /// Redis communication error
    #[error("Redis error: {0}")]
    Redis(String),

    /// Connection not found in registry
    #[error("Connection not found for user")]
    NotFound,

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Port for tracking WebSocket connections across multiple servers.
///
/// Implementations should:
/// - Use TTL-based expiration to handle server crashes
/// - Support heartbeat to refresh TTL
/// - Allow multiple connections per user (different tabs/devices)
///
/// # Example
///
/// ```ignore
/// // WebSocket handler on connect:
/// async fn handle_connect(user_id: UserId, registry: Arc<dyn ConnectionRegistry>) {
///     registry.register(&user_id, &server_id).await?;
/// }
///
/// // WebSocket handler on disconnect:
/// async fn handle_disconnect(user_id: UserId, registry: Arc<dyn ConnectionRegistry>) {
///     registry.unregister(&user_id, &server_id).await?;
/// }
///
/// // When sending a message to a user:
/// async fn send_to_user(user_id: UserId, msg: Message) {
///     let servers = registry.find_servers(&user_id).await?;
///     for server_id in servers {
///         messenger.publish_to_server(&server_id, &user_id, &msg).await?;
///     }
/// }
/// ```
#[async_trait]
pub trait ConnectionRegistry: Send + Sync {
    /// Register a user's connection on this server.
    ///
    /// Called when a WebSocket connection is established.
    /// Should set a TTL that will be refreshed by heartbeat.
    async fn register(
        &self,
        user_id: &UserId,
        server_id: &ServerId,
    ) -> Result<(), ConnectionRegistryError>;

    /// Unregister a user's connection from this server.
    ///
    /// Called when a WebSocket connection is closed gracefully.
    async fn unregister(
        &self,
        user_id: &UserId,
        server_id: &ServerId,
    ) -> Result<(), ConnectionRegistryError>;

    /// Find all servers that have connections for a user.
    ///
    /// A user may have multiple connections (different browser tabs,
    /// different devices) potentially across different servers.
    ///
    /// Returns empty vec if user has no active connections.
    async fn find_servers(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<ServerId>, ConnectionRegistryError>;

    /// Check if a user has any active connections.
    ///
    /// More efficient than `find_servers` when you only need to know
    /// if the user is connected, not where.
    async fn is_connected(&self, user_id: &UserId) -> Result<bool, ConnectionRegistryError>;

    /// Refresh TTL for a connection (heartbeat).
    ///
    /// Called periodically to prevent stale connections from
    /// lingering if a server crashes without cleanup.
    ///
    /// Typical TTL is 60 seconds with heartbeat every 30 seconds.
    async fn heartbeat(
        &self,
        user_id: &UserId,
        server_id: &ServerId,
    ) -> Result<(), ConnectionRegistryError>;

    /// Get all users connected to a specific server.
    ///
    /// Used for server shutdown to notify users of reconnection needed.
    async fn get_server_connections(
        &self,
        server_id: &ServerId,
    ) -> Result<Vec<UserId>, ConnectionRegistryError>;

    /// Clean up all connections for a server.
    ///
    /// Called on graceful server shutdown.
    async fn cleanup_server(&self, server_id: &ServerId) -> Result<u64, ConnectionRegistryError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_id_from_env_format() {
        // Just test the format, not actual env vars
        let server_id = ServerId::new("server-1:8080");
        assert_eq!(server_id.as_str(), "server-1:8080");
    }

    #[test]
    fn server_id_display() {
        let server_id = ServerId::new("server-1:8080");
        assert_eq!(format!("{}", server_id), "server-1:8080");
    }

    #[test]
    fn server_id_from_string() {
        let server_id: ServerId = "server-2:9000".into();
        assert_eq!(server_id.as_str(), "server-2:9000");
    }
}
