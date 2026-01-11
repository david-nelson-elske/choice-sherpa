//! Event infrastructure for domain event publishing and handling.
//!
//! This module provides the core types and traits for event-driven architecture:
//! - `EventId` - Unique identifier for events (deduplication)
//! - `EventMetadata` - Tracing and correlation context
//! - `EventEnvelope` - Transport wrapper for domain events
//! - `DomainEvent` - Trait that all domain events implement
//! - `domain_event!` - Macro to simplify DomainEvent implementations

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fmt;
use uuid::Uuid;

use super::Timestamp;

// ============================================
// DomainEvent Trait
// ============================================

/// Trait that all domain events must implement.
///
/// Provides the contract for event identification, routing, ordering, and versioning.
/// Use the `domain_event!` macro to implement this trait with minimal boilerplate.
///
/// For types that also implement `Serialize`, the `to_envelope()` method
/// is automatically available via the `SerializableDomainEvent` extension trait.
pub trait DomainEvent: Send + Sync {
    /// Returns the event type string (e.g., "session.created.v1").
    /// Used for routing and filtering.
    /// SHOULD include version suffix (e.g., ".v1", ".v2") for explicit versioning.
    fn event_type(&self) -> &'static str;

    /// Returns the schema version number.
    /// MUST match the version suffix in event_type.
    fn schema_version(&self) -> u32;

    /// Returns the ID of the aggregate that emitted this event.
    fn aggregate_id(&self) -> String;

    /// Returns the type of aggregate (e.g., "Session", "Cycle").
    fn aggregate_type(&self) -> &'static str;

    /// Returns when the event occurred.
    fn occurred_at(&self) -> Timestamp;

    /// Returns the unique ID for this event instance.
    fn event_id(&self) -> EventId;
}

/// Extension trait that provides `to_envelope()` for serializable domain events.
///
/// This trait is automatically implemented for any type that implements
/// both `DomainEvent` and `Serialize`. The blanket implementation ensures
/// zero boilerplate for event authors.
///
/// # Example
///
/// ```ignore
/// use serde::Serialize;
///
/// #[derive(Debug, Clone, Serialize)]
/// struct SessionCreated { /* fields */ }
///
/// impl DomainEvent for SessionCreated { /* ... */ }
///
/// // to_envelope() is automatically available:
/// let envelope = event.to_envelope();
/// ```
pub trait SerializableDomainEvent: DomainEvent + Serialize {
    /// Converts this domain event into an `EventEnvelope` for transport.
    ///
    /// This default implementation extracts all required fields from the
    /// `DomainEvent` trait and serializes the event as the payload.
    fn to_envelope(&self) -> EventEnvelope {
        let event_type = self.event_type().to_string();
        let schema_version = EventEnvelope::extract_version(&event_type);

        EventEnvelope {
            event_id: self.event_id(),
            event_type,
            schema_version,
            aggregate_id: self.aggregate_id(),
            aggregate_type: self.aggregate_type().to_string(),
            occurred_at: self.occurred_at(),
            payload: serde_json::to_value(self)
                .expect("Event serialization should never fail for well-formed events"),
            metadata: EventMetadata::default(),
        }
    }
}

// Blanket implementation: any type implementing DomainEvent + Serialize
// automatically gets to_envelope()
impl<T: DomainEvent + Serialize> SerializableDomainEvent for T {}

/// Macro to implement DomainEvent trait with minimal boilerplate.
///
/// # Example
///
/// ```ignore
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// pub struct SessionCreated {
///     pub event_id: EventId,
///     pub session_id: SessionId,
///     pub user_id: UserId,
///     pub title: String,
///     pub created_at: Timestamp,
/// }
///
/// domain_event!(
///     SessionCreated,
///     event_type = "session.created.v1",
///     schema_version = 1,
///     aggregate_id = session_id,
///     aggregate_type = "Session",
///     occurred_at = created_at,
///     event_id = event_id
/// );
/// ```
#[macro_export]
macro_rules! domain_event {
    (
        $event_name:ident,
        event_type = $event_type:expr,
        schema_version = $schema_version:expr,
        aggregate_id = $agg_id_field:ident,
        aggregate_type = $agg_type:expr,
        occurred_at = $occurred_field:ident,
        event_id = $event_id_field:ident
    ) => {
        impl $crate::domain::foundation::DomainEvent for $event_name {
            fn event_type(&self) -> &'static str {
                $event_type
            }

            fn schema_version(&self) -> u32 {
                $schema_version
            }

            fn aggregate_id(&self) -> String {
                self.$agg_id_field.to_string()
            }

            fn aggregate_type(&self) -> &'static str {
                $agg_type
            }

            fn occurred_at(&self) -> $crate::domain::foundation::Timestamp {
                self.$occurred_field
            }

            fn event_id(&self) -> $crate::domain::foundation::EventId {
                self.$event_id_field.clone()
            }
        }
    };
}

// Re-export the macro
pub use domain_event;

/// Unique identifier for events (used for deduplication).
///
/// Unlike other IDs in the system, EventId uses a String internally
/// to allow for various ID formats (UUID, ULID, etc.) while maintaining
/// serializability.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventId(String);

impl EventId {
    /// Creates a new random EventId using UUID v4.
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Creates an EventId from an existing string.
    ///
    /// No validation is performed - any non-empty string is accepted.
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Returns the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Metadata for tracing and correlation.
///
/// Provides context that flows through the event system:
/// - `correlation_id` - Links related events across a request
/// - `causation_id` - ID of the event that caused this one
/// - `user_id` - User who triggered this event chain
/// - `trace_id` - Distributed tracing identifier
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventMetadata {
    /// ID linking related events across a single user request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,

    /// ID of the event that directly caused this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<String>,

    /// User who initiated the action that led to this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Distributed tracing span/trace ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

/// Transport envelope for domain events.
///
/// Wraps event-specific data with metadata needed for:
/// - Routing (event_type)
/// - Deduplication (event_id)
/// - Correlation (aggregate_id, metadata)
/// - Ordering (occurred_at)
/// - Versioning (schema_version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Unique ID for this event instance.
    pub event_id: EventId,

    /// Event type for routing (e.g., "session.created.v1").
    pub event_type: String,

    /// Schema version number (extracted from event_type).
    pub schema_version: u32,

    /// ID of the aggregate that emitted this event.
    pub aggregate_id: String,

    /// Type of aggregate (e.g., "Session", "Cycle").
    pub aggregate_type: String,

    /// When the event occurred.
    pub occurred_at: Timestamp,

    /// Event-specific payload as JSON.
    pub payload: JsonValue,

    /// Tracing and correlation metadata.
    pub metadata: EventMetadata,
}

impl EventEnvelope {
    /// Creates a new EventEnvelope with required fields.
    ///
    /// Automatically extracts schema version from event_type suffix (e.g., "session.created.v2" → 2).
    /// If no version suffix is present, defaults to v1.
    pub fn new(
        event_type: impl Into<String>,
        aggregate_id: impl Into<String>,
        aggregate_type: impl Into<String>,
        payload: JsonValue,
    ) -> Self {
        let event_type = event_type.into();
        let schema_version = Self::extract_version(&event_type);

        Self {
            event_id: EventId::new(),
            event_type,
            schema_version,
            aggregate_id: aggregate_id.into(),
            aggregate_type: aggregate_type.into(),
            occurred_at: Timestamp::now(),
            payload,
            metadata: EventMetadata::default(),
        }
    }

    /// Extracts version number from event_type string.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// EventEnvelope::extract_version("session.created.v2") // Returns 2
    /// EventEnvelope::extract_version("session.created.v10") // Returns 10
    /// EventEnvelope::extract_version("legacy.event") // Returns 1 (default)
    /// ```
    pub(crate) fn extract_version(event_type: &str) -> u32 {
        event_type
            .rsplit_once(".v")
            .and_then(|(_, version_str)| version_str.parse::<u32>().ok())
            .unwrap_or(1)
    }

    /// Returns the schema version number.
    ///
    /// This is a convenience method that returns the same value as the `schema_version` field.
    pub fn version(&self) -> u32 {
        self.schema_version
    }

    /// Creates an envelope from a domain event with automatic serialization.
    ///
    /// This is the preferred way to create envelopes in command handlers,
    /// as it extracts all required fields from the DomainEvent trait.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let event = SessionCreated { /* ... */ };
    /// let envelope = EventEnvelope::from_event(&event)
    ///     .with_correlation_id(metadata.correlation_id.clone())
    ///     .with_user_id(user_id.to_string());
    /// event_publisher.publish(envelope).await?;
    /// ```
    pub fn from_event<T>(event: &T) -> Self
    where
        T: DomainEvent + Serialize,
    {
        let event_type = event.event_type().to_string();
        let schema_version = Self::extract_version(&event_type);

        Self {
            event_id: event.event_id(),
            event_type,
            schema_version,
            aggregate_id: event.aggregate_id(),
            aggregate_type: event.aggregate_type().to_string(),
            occurred_at: event.occurred_at(),
            payload: serde_json::to_value(event)
                .expect("Event serialization should never fail for well-formed events"),
            metadata: EventMetadata::default(),
        }
    }

    /// Add correlation ID for request tracing.
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.metadata.correlation_id = Some(id.into());
        self
    }

    /// Add causation ID (ID of event that caused this one).
    pub fn with_causation_id(mut self, id: impl Into<String>) -> Self {
        self.metadata.causation_id = Some(id.into());
        self
    }

    /// Add user ID for audit.
    pub fn with_user_id(mut self, id: impl Into<String>) -> Self {
        self.metadata.user_id = Some(id.into());
        self
    }

    /// Add trace ID for distributed tracing.
    pub fn with_trace_id(mut self, id: impl Into<String>) -> Self {
        self.metadata.trace_id = Some(id.into());
        self
    }

    /// Deserialize payload to a specific event type.
    pub fn payload_as<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.payload.clone())
    }
}

#[cfg(test)]
impl EventEnvelope {
    /// Creates a test fixture EventEnvelope for use in tests.
    pub fn test_fixture() -> Self {
        Self::new(
            "test.event.v1",
            "test-aggregate-123",
            "TestAggregate",
            serde_json::json!({"test": "data"}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ============================================================
    // EventId Tests
    // ============================================================

    #[test]
    fn event_id_generates_unique_values() {
        let id1 = EventId::new();
        let id2 = EventId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn event_id_from_string_preserves_value() {
        let id = EventId::from_string("test-id-123");
        assert_eq!(id.as_str(), "test-id-123");
    }

    #[test]
    fn event_id_serializes_to_json() {
        let id = EventId::from_string("test-id");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, r#""test-id""#);
    }

    #[test]
    fn event_id_deserializes_from_json() {
        let json = r#""evt-456""#;
        let id: EventId = serde_json::from_str(json).unwrap();
        assert_eq!(id.as_str(), "evt-456");
    }

    #[test]
    fn event_id_displays_correctly() {
        let id = EventId::from_string("display-test");
        assert_eq!(format!("{}", id), "display-test");
    }

    #[test]
    fn event_id_default_creates_new() {
        let id1 = EventId::default();
        let id2 = EventId::default();
        assert_ne!(id1, id2);
    }

    // ============================================================
    // EventMetadata Tests
    // ============================================================

    #[test]
    fn event_metadata_default_has_all_none() {
        let meta = EventMetadata::default();
        assert!(meta.correlation_id.is_none());
        assert!(meta.causation_id.is_none());
        assert!(meta.user_id.is_none());
        assert!(meta.trace_id.is_none());
    }

    #[test]
    fn event_metadata_serializes_without_none_fields() {
        let meta = EventMetadata {
            correlation_id: Some("req-123".to_string()),
            causation_id: None,
            user_id: None,
            trace_id: None,
        };
        let json = serde_json::to_string(&meta).unwrap();
        assert!(json.contains("correlation_id"));
        assert!(!json.contains("causation_id"));
        assert!(!json.contains("user_id"));
        assert!(!json.contains("trace_id"));
    }

    #[test]
    fn event_metadata_round_trip_serialization() {
        let meta = EventMetadata {
            correlation_id: Some("corr-1".to_string()),
            causation_id: Some("cause-1".to_string()),
            user_id: Some("user-1".to_string()),
            trace_id: Some("trace-1".to_string()),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let restored: EventMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(meta, restored);
    }

    // ============================================================
    // EventEnvelope Tests
    // ============================================================

    #[test]
    fn event_envelope_new_creates_with_defaults() {
        let envelope = EventEnvelope::new(
            "session.created",
            "session-123",
            "Session",
            json!({"title": "Test"}),
        );

        assert_eq!(envelope.event_type, "session.created");
        assert_eq!(envelope.aggregate_id, "session-123");
        assert_eq!(envelope.aggregate_type, "Session");
        assert_eq!(envelope.payload["title"], "Test");
        assert!(envelope.metadata.correlation_id.is_none());
    }

    #[test]
    fn event_envelope_builder_chain() {
        let envelope = EventEnvelope::new("test.event", "agg-1", "Test", json!({}))
            .with_correlation_id("req-123")
            .with_causation_id("evt-0")
            .with_user_id("user-456")
            .with_trace_id("trace-789");

        assert_eq!(envelope.metadata.correlation_id, Some("req-123".to_string()));
        assert_eq!(envelope.metadata.causation_id, Some("evt-0".to_string()));
        assert_eq!(envelope.metadata.user_id, Some("user-456".to_string()));
        assert_eq!(envelope.metadata.trace_id, Some("trace-789".to_string()));
    }

    #[test]
    fn event_envelope_serialization_round_trip() {
        let envelope = EventEnvelope::new(
            "session.created",
            "session-123",
            "Session",
            json!({"title": "Test Decision"}),
        )
        .with_correlation_id("req-456");

        let json = serde_json::to_string(&envelope).unwrap();
        let restored: EventEnvelope = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.event_id, envelope.event_id);
        assert_eq!(restored.event_type, envelope.event_type);
        assert_eq!(restored.aggregate_id, envelope.aggregate_id);
        assert_eq!(restored.metadata.correlation_id, envelope.metadata.correlation_id);
    }

    #[test]
    fn event_envelope_payload_as_deserializes() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct TestPayload {
            value: i32,
            name: String,
        }

        let envelope = EventEnvelope::new(
            "test.event",
            "agg-1",
            "Test",
            json!({"value": 42, "name": "test"}),
        );

        let payload: TestPayload = envelope.payload_as().unwrap();
        assert_eq!(payload.value, 42);
        assert_eq!(payload.name, "test");
    }

    #[test]
    fn event_envelope_payload_as_returns_error_on_mismatch() {
        #[derive(Debug, Deserialize)]
        struct WrongPayload {
            missing_field: String,
        }

        let envelope = EventEnvelope::new(
            "test.event",
            "agg-1",
            "Test",
            json!({"different": "data"}),
        );

        let result: Result<WrongPayload, _> = envelope.payload_as();
        assert!(result.is_err());
    }

    // ============================================================
    // DomainEvent::to_envelope() Tests
    // ============================================================

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestSessionCreated {
        event_id: EventId,
        session_id: String,
        title: String,
        occurred_at: Timestamp,
    }

    impl DomainEvent for TestSessionCreated {
        fn event_type(&self) -> &'static str {
            "test.session.created"
        }

        fn schema_version(&self) -> u32 {
            1
        }

        fn aggregate_id(&self) -> String {
            self.session_id.clone()
        }

        fn aggregate_type(&self) -> &'static str {
            "TestSession"
        }

        fn occurred_at(&self) -> Timestamp {
            self.occurred_at
        }

        fn event_id(&self) -> EventId {
            self.event_id.clone()
        }
    }

    #[test]
    fn domain_event_to_envelope_creates_valid_envelope() {
        let event = TestSessionCreated {
            event_id: EventId::from_string("evt-123"),
            session_id: "session-456".to_string(),
            title: "Test Decision".to_string(),
            occurred_at: Timestamp::now(),
        };

        // This should call the default to_envelope() method on DomainEvent trait
        let envelope = event.to_envelope();

        assert_eq!(envelope.event_id.as_str(), "evt-123");
        assert_eq!(envelope.event_type, "test.session.created");
        assert_eq!(envelope.aggregate_id, "session-456");
        assert_eq!(envelope.aggregate_type, "TestSession");
        assert_eq!(envelope.payload["title"], "Test Decision");
    }

    #[test]
    fn domain_event_to_envelope_preserves_occurred_at() {
        let occurred_at = Timestamp::now();
        let event = TestSessionCreated {
            event_id: EventId::new(),
            session_id: "session-1".to_string(),
            title: "Test".to_string(),
            occurred_at,
        };

        let envelope = event.to_envelope();

        assert_eq!(envelope.occurred_at, occurred_at);
    }

    #[test]
    fn domain_event_to_envelope_payload_round_trips() {
        let event = TestSessionCreated {
            event_id: EventId::from_string("evt-789"),
            session_id: "session-abc".to_string(),
            title: "Round Trip Test".to_string(),
            occurred_at: Timestamp::now(),
        };

        let envelope = event.to_envelope();
        let restored: TestSessionCreated = envelope.payload_as().unwrap();

        assert_eq!(restored.event_id.as_str(), "evt-789");
        assert_eq!(restored.session_id, "session-abc");
        assert_eq!(restored.title, "Round Trip Test");
    }

    // ============================================================
    // EventEnvelope Schema Versioning Tests
    // ============================================================

    #[test]
    fn event_envelope_has_schema_version_field() {
        let envelope = EventEnvelope::new(
            "session.created.v1",
            "session-123",
            "Session",
            json!({"title": "Test"}),
        );

        assert_eq!(envelope.schema_version, 1);
    }

    #[test]
    fn event_envelope_extracts_version_from_event_type() {
        let envelope = EventEnvelope::new(
            "session.created.v2",
            "session-123",
            "Session",
            json!({}),
        );

        assert_eq!(envelope.version(), 2);
        assert_eq!(envelope.schema_version, 2);
    }

    #[test]
    fn event_envelope_version_method_returns_schema_version() {
        let envelope = EventEnvelope::new(
            "cycle.completed.v5",
            "cycle-456",
            "Cycle",
            json!({}),
        );

        assert_eq!(envelope.version(), 5);
    }

    #[test]
    fn event_envelope_defaults_to_v1_without_version_suffix() {
        let envelope = EventEnvelope::new(
            "legacy.event",
            "agg-123",
            "Legacy",
            json!({}),
        );

        assert_eq!(envelope.schema_version, 1);
        assert_eq!(envelope.version(), 1);
    }

    // ============================================================
    // DomainEvent schema_version() Tests
    // ============================================================

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEventV2 {
        event_id: EventId,
        aggregate_id: String,
        occurred_at: Timestamp,
        data: String,
    }

    impl DomainEvent for TestEventV2 {
        fn event_type(&self) -> &'static str {
            "test.event.v2"
        }

        fn schema_version(&self) -> u32 {
            2
        }

        fn aggregate_id(&self) -> String {
            self.aggregate_id.clone()
        }

        fn aggregate_type(&self) -> &'static str {
            "TestAggregate"
        }

        fn occurred_at(&self) -> Timestamp {
            self.occurred_at
        }

        fn event_id(&self) -> EventId {
            self.event_id.clone()
        }
    }

    #[test]
    fn domain_event_schema_version_returns_correct_version() {
        let event = TestEventV2 {
            event_id: EventId::new(),
            aggregate_id: "agg-123".to_string(),
            occurred_at: Timestamp::now(),
            data: "test data".to_string(),
        };

        assert_eq!(event.schema_version(), 2);
    }

    #[test]
    fn domain_event_to_envelope_includes_schema_version() {
        let event = TestEventV2 {
            event_id: EventId::from_string("evt-v2-test"),
            aggregate_id: "agg-456".to_string(),
            occurred_at: Timestamp::now(),
            data: "test".to_string(),
        };

        let envelope = event.to_envelope();

        // Schema version should come from event_type parsing (not from trait method yet)
        assert_eq!(envelope.schema_version, 2);
        assert_eq!(envelope.version(), 2);
        assert_eq!(envelope.event_type, "test.event.v2");
    }

    #[test]
    fn domain_event_schema_version_matches_event_type() {
        let event = TestEventV2 {
            event_id: EventId::new(),
            aggregate_id: "agg-789".to_string(),
            occurred_at: Timestamp::now(),
            data: "test".to_string(),
        };

        // Version from trait should match version in event_type
        let version_from_trait = event.schema_version();
        let version_from_type = EventEnvelope::extract_version(event.event_type());

        assert_eq!(version_from_trait, version_from_type);
        assert_eq!(version_from_trait, 2);
    }
}
