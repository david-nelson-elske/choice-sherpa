//! Command infrastructure for CQRS handlers.
//!
//! This module provides the standard types for command handlers:
//! - `CommandMetadata` - Context that flows through command processing
//!
//! # DRY Pattern
//!
//! Instead of each handler accepting `correlation_id: Option<String>,
//! user_id: String, trace_id: Option<String>`, they accept a single
//! `CommandMetadata` struct. This:
//! - Reduces function parameter count
//! - Ensures consistent naming across all handlers
//! - Makes it easy to add new metadata fields without changing signatures

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::UserId;

/// Metadata context for command handlers.
///
/// Carries tracing, correlation, and authentication context through
/// the command processing pipeline. This should be passed to all
/// command handlers and propagated to emitted events.
///
/// # Example
///
/// ```ignore
/// pub struct CreateSessionHandler {
///     repo: Arc<dyn SessionRepository>,
///     publisher: Arc<dyn EventPublisher>,
/// }
///
/// impl CreateSessionHandler {
///     pub async fn handle(
///         &self,
///         cmd: CreateSessionCommand,
///         metadata: CommandMetadata,
///     ) -> Result<SessionId, DomainError> {
///         // ... handler logic
///
///         // Propagate metadata to events
///         let envelope = EventEnvelope::from_event(&event)
///             .with_correlation_id(metadata.correlation_id())
///             .with_user_id(metadata.user_id.to_string());
///
///         self.publisher.publish(envelope).await?;
///         Ok(session.id)
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandMetadata {
    /// The user executing this command (required for authorization).
    pub user_id: UserId,

    /// Links related operations across a single user request.
    /// Generated at API boundary if not provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    correlation_id: Option<String>,

    /// Distributed tracing span/trace ID.
    /// Propagated from incoming requests (e.g., from OpenTelemetry).
    #[serde(skip_serializing_if = "Option::is_none")]
    trace_id: Option<String>,

    /// Source of this command (e.g., "api", "websocket", "scheduler").
    /// Useful for audit logs and debugging.
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
}

impl CommandMetadata {
    /// Creates new command metadata with required user ID.
    ///
    /// Generates a correlation ID automatically if not provided later.
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            correlation_id: None,
            trace_id: None,
            source: None,
        }
    }

    /// Creates metadata with all optional fields populated.
    ///
    /// Use this when reconstructing from serialized form or tests.
    pub fn with_all(
        user_id: UserId,
        correlation_id: Option<String>,
        trace_id: Option<String>,
        source: Option<String>,
    ) -> Self {
        Self {
            user_id,
            correlation_id,
            trace_id,
            source,
        }
    }

    /// Builder: Add correlation ID for request tracing.
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Builder: Add trace ID for distributed tracing.
    pub fn with_trace_id(mut self, id: impl Into<String>) -> Self {
        self.trace_id = Some(id.into());
        self
    }

    /// Builder: Add source identifier.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Returns the correlation ID, generating one if not set.
    ///
    /// This ensures every command has a correlation ID for tracing,
    /// even if the API layer didn't provide one.
    pub fn correlation_id(&self) -> String {
        self.correlation_id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string())
    }

    /// Returns the correlation ID only if explicitly set.
    pub fn correlation_id_opt(&self) -> Option<&str> {
        self.correlation_id.as_deref()
    }

    /// Returns the trace ID if set.
    pub fn trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }

    /// Returns the source if set.
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }
}

#[cfg(test)]
impl CommandMetadata {
    /// Creates a test fixture with a test user ID.
    ///
    /// Only available in test builds.
    pub fn test_fixture() -> Self {
        Self::new(UserId::new("test-user-123").unwrap())
            .with_correlation_id("test-correlation-id")
            .with_source("test")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_with_user_id() {
        let user_id = UserId::new("user-123").unwrap();
        let metadata = CommandMetadata::new(user_id.clone());

        assert_eq!(metadata.user_id, user_id);
        assert!(metadata.correlation_id.is_none());
        assert!(metadata.trace_id.is_none());
        assert!(metadata.source.is_none());
    }

    #[test]
    fn builder_chain_sets_all_fields() {
        let user_id = UserId::new("user-456").unwrap();
        let metadata = CommandMetadata::new(user_id)
            .with_correlation_id("corr-123")
            .with_trace_id("trace-456")
            .with_source("api");

        assert_eq!(metadata.correlation_id, Some("corr-123".to_string()));
        assert_eq!(metadata.trace_id, Some("trace-456".to_string()));
        assert_eq!(metadata.source, Some("api".to_string()));
    }

    #[test]
    fn correlation_id_generates_if_missing() {
        let metadata = CommandMetadata::new(UserId::new("user").unwrap());

        let id = metadata.correlation_id();

        // Should generate a new ID when not set
        assert!(!id.is_empty());
        // Note: Since correlation_id() generates new UUID each time when not set,
        // each call will be different. This is intentional for the use case where
        // the ID is only needed once per command.
    }

    #[test]
    fn correlation_id_returns_set_value() {
        let metadata = CommandMetadata::new(UserId::new("user").unwrap())
            .with_correlation_id("my-correlation-id");

        assert_eq!(metadata.correlation_id(), "my-correlation-id");
        assert_eq!(metadata.correlation_id_opt(), Some("my-correlation-id"));
    }

    #[test]
    fn correlation_id_opt_returns_none_when_not_set() {
        let metadata = CommandMetadata::new(UserId::new("user").unwrap());
        assert!(metadata.correlation_id_opt().is_none());
    }

    #[test]
    fn accessors_return_correct_values() {
        let metadata = CommandMetadata::new(UserId::new("user").unwrap())
            .with_trace_id("trace-id")
            .with_source("websocket");

        assert_eq!(metadata.trace_id(), Some("trace-id"));
        assert_eq!(metadata.source(), Some("websocket"));
    }

    #[test]
    fn with_all_populates_all_fields() {
        let user_id = UserId::new("user-789").unwrap();
        let metadata = CommandMetadata::with_all(
            user_id.clone(),
            Some("corr-all".to_string()),
            Some("trace-all".to_string()),
            Some("scheduler".to_string()),
        );

        assert_eq!(metadata.user_id, user_id);
        assert_eq!(metadata.correlation_id, Some("corr-all".to_string()));
        assert_eq!(metadata.trace_id, Some("trace-all".to_string()));
        assert_eq!(metadata.source, Some("scheduler".to_string()));
    }

    #[test]
    fn serialization_round_trip() {
        let metadata = CommandMetadata::new(UserId::new("user-ser").unwrap())
            .with_correlation_id("ser-corr")
            .with_trace_id("ser-trace");

        let json = serde_json::to_string(&metadata).unwrap();
        let restored: CommandMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(metadata, restored);
    }

    #[test]
    fn serialization_skips_none_fields() {
        let metadata = CommandMetadata::new(UserId::new("user-skip").unwrap());

        let json = serde_json::to_string(&metadata).unwrap();

        assert!(json.contains("user_id"));
        assert!(!json.contains("correlation_id"));
        assert!(!json.contains("trace_id"));
        assert!(!json.contains("source"));
    }

    #[test]
    fn test_fixture_creates_valid_metadata() {
        let metadata = CommandMetadata::test_fixture();

        assert_eq!(metadata.user_id.as_str(), "test-user-123");
        assert_eq!(metadata.correlation_id(), "test-correlation-id");
        assert_eq!(metadata.source(), Some("test"));
    }
}
