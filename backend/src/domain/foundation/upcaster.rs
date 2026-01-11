//! Event upcaster infrastructure for schema evolution.
//!
//! Provides the ability to transform events from older schema versions to newer versions,
//! enabling backward compatibility and safe event replay.
//!
//! # Architecture
//!
//! - `Upcaster` trait - Transforms a single version step (v1 → v2)
//! - `UpcasterRegistry` - Chains multiple upcasters to reach current version
//! - `UpcastError` - Error types for failed transformations
//!
//! # Example
//!
//! ```ignore
//! // Define an upcaster for a version step
//! struct SessionCreatedV1ToV2;
//!
//! impl Upcaster for SessionCreatedV1ToV2 {
//!     fn source_type(&self) -> &str { "session.created.v1" }
//!     fn target_type(&self) -> &str { "session.created.v2" }
//!
//!     fn upcast(&self, mut payload: serde_json::Value) -> Result<serde_json::Value, UpcastError> {
//!         // Add new optional field
//!         payload["description"] = serde_json::Value::Null;
//!         Ok(payload)
//!     }
//! }
//! ```

use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

use super::EventEnvelope;

// ============================================
// Error Types
// ============================================

/// Errors that can occur during event upcasting.
#[derive(Debug, Error)]
pub enum UpcastError {
    /// Required field is missing from the source event.
    #[error("missing required field: {0}")]
    MissingField(String),

    /// Field value is invalid or cannot be converted.
    #[error("invalid field value: {0}")]
    InvalidValue(String),

    /// No upcaster path exists from source to target version.
    #[error("incompatible version transition: {from} → {to}")]
    IncompatibleVersions { from: String, to: String },

    /// JSON serialization/deserialization error during transformation.
    #[error("JSON transformation error: {0}")]
    JsonError(#[from] serde_json::Error),
}

// ============================================
// Upcaster Trait
// ============================================

/// Transforms events from one schema version to another.
///
/// Each upcaster handles a single version step (e.g., v1 → v2).
/// Multiple upcasters can be chained together by the registry to
/// transform from any old version to the current version.
///
/// # Implementation Notes
///
/// - Upcasters MUST NOT mutate the source event in storage
/// - Transformations MUST be deterministic (same input → same output)
/// - Upcasters SHOULD preserve data classification (no classification downgrade)
/// - If transformation fails, return UpcastError (don't panic)
pub trait Upcaster: Send + Sync {
    /// Source event type including version (e.g., "session.created.v1").
    fn source_type(&self) -> &str;

    /// Target event type including version (e.g., "session.created.v2").
    fn target_type(&self) -> &str;

    /// Transform the event payload from source to target schema.
    ///
    /// # Arguments
    ///
    /// * `payload` - The source event payload as JSON
    ///
    /// # Returns
    ///
    /// * `Ok(JsonValue)` - The transformed payload in target schema
    /// * `Err(UpcastError)` - If transformation fails
    fn upcast(&self, payload: JsonValue) -> Result<JsonValue, UpcastError>;
}

// ============================================
// Upcaster Registry
// ============================================

/// Registry that manages and chains event upcasters.
///
/// The registry maintains a map of upcasters and can automatically
/// chain them together to transform events from any old version to
/// the current version.
///
/// # Example
///
/// ```ignore
/// let mut registry = UpcasterRegistry::new();
/// registry.register(Box::new(SessionCreatedV1ToV2));
/// registry.register(Box::new(SessionCreatedV2ToV3));
/// registry.set_current_version("session.created", 3);
///
/// // Automatically chains v1→v2→v3
/// let v3_envelope = registry.upcast_to_current(v1_envelope)?;
/// ```
pub struct UpcasterRegistry {
    /// Map from source event_type to upcaster.
    upcasters: HashMap<String, Arc<dyn Upcaster>>,

    /// Current version for each event base type.
    /// Base type is event_type without version suffix (e.g., "session.created").
    current_versions: HashMap<String, u32>,
}

impl UpcasterRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            upcasters: HashMap::new(),
            current_versions: HashMap::new(),
        }
    }

    /// Registers an upcaster for a specific version transition.
    ///
    /// # Arguments
    ///
    /// * `upcaster` - The upcaster to register
    ///
    /// # Example
    ///
    /// ```ignore
    /// registry.register(Box::new(SessionCreatedV1ToV2));
    /// ```
    pub fn register(&mut self, upcaster: Arc<dyn Upcaster>) {
        self.upcasters
            .insert(upcaster.source_type().to_string(), upcaster);
    }

    /// Sets the current version for an event type.
    ///
    /// # Arguments
    ///
    /// * `base_type` - Event type without version suffix (e.g., "session.created")
    /// * `version` - Current version number
    ///
    /// # Example
    ///
    /// ```ignore
    /// registry.set_current_version("session.created", 3);
    /// ```
    pub fn set_current_version(&mut self, base_type: impl Into<String>, version: u32) {
        self.current_versions.insert(base_type.into(), version);
    }

    /// Upcasts an event envelope to the current version.
    ///
    /// Automatically chains multiple upcasters if needed to reach the current version.
    ///
    /// # Arguments
    ///
    /// * `envelope` - The event envelope to upcast
    ///
    /// # Returns
    ///
    /// * `Ok(EventEnvelope)` - Envelope with payload transformed to current version
    /// * `Err(UpcastError)` - If no upcaster path exists or transformation fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let current = registry.upcast_to_current(old_envelope)?;
    /// ```
    pub fn upcast_to_current(
        &self,
        envelope: EventEnvelope,
    ) -> Result<EventEnvelope, UpcastError> {
        let base_type = Self::extract_base_type(&envelope.event_type);
        let target_version = self
            .current_versions
            .get(&base_type)
            .copied()
            .unwrap_or(envelope.schema_version);

        // If already at current version, return as-is
        if envelope.schema_version >= target_version {
            return Ok(envelope);
        }

        let mut current = envelope;

        // Chain upcasters until we reach target version
        while current.schema_version < target_version {
            let upcaster = self.upcasters.get(&current.event_type).ok_or_else(|| {
                UpcastError::IncompatibleVersions {
                    from: current.event_type.clone(),
                    to: format!("{}.v{}", base_type, target_version),
                }
            })?;

            let new_payload = upcaster.upcast(current.payload)?;
            let new_version = current.schema_version + 1;

            current = EventEnvelope {
                event_id: current.event_id,
                event_type: upcaster.target_type().to_string(),
                schema_version: new_version,
                aggregate_id: current.aggregate_id,
                aggregate_type: current.aggregate_type,
                occurred_at: current.occurred_at,
                payload: new_payload,
                metadata: current.metadata,
            };
        }

        Ok(current)
    }

    /// Extracts base type from versioned event_type.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// extract_base_type("session.created.v2") // Returns "session.created"
    /// extract_base_type("cycle.completed")    // Returns "cycle.completed"
    /// ```
    fn extract_base_type(event_type: &str) -> String {
        event_type
            .rsplit_once(".v")
            .map(|(base, _)| base.to_string())
            .unwrap_or_else(|| event_type.to_string())
    }
}

impl Default for UpcasterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{EventId, EventMetadata, Timestamp};
    use serde_json::json;

    // Test upcaster: v1 → v2 (adds optional field)
    struct TestEventV1ToV2;

    impl Upcaster for TestEventV1ToV2 {
        fn source_type(&self) -> &str {
            "test.event.v1"
        }

        fn target_type(&self) -> &str {
            "test.event.v2"
        }

        fn upcast(&self, mut payload: JsonValue) -> Result<JsonValue, UpcastError> {
            // Add optional description field
            payload["description"] = JsonValue::Null;
            Ok(payload)
        }
    }

    // Test upcaster: v2 → v3 (transforms field)
    struct TestEventV2ToV3;

    impl Upcaster for TestEventV2ToV3 {
        fn source_type(&self) -> &str {
            "test.event.v2"
        }

        fn target_type(&self) -> &str {
            "test.event.v3"
        }

        fn upcast(&self, mut payload: JsonValue) -> Result<JsonValue, UpcastError> {
            // Transform user_id to owner object
            let user_id = payload
                .get("user_id")
                .ok_or_else(|| UpcastError::MissingField("user_id".to_string()))?
                .clone();

            payload["owner"] = json!({
                "user_id": user_id,
                "display_name": "Unknown"
            });

            Ok(payload)
        }
    }

    // ============================================================
    // Upcaster Trait Tests
    // ============================================================

    #[test]
    fn upcaster_transforms_v1_to_v2() {
        let upcaster = TestEventV1ToV2;

        let v1_payload = json!({
            "event_id": "evt-1",
            "data": "test"
        });

        let v2_payload = upcaster.upcast(v1_payload).unwrap();

        assert_eq!(v2_payload["data"], "test");
        assert!(v2_payload["description"].is_null());
    }

    #[test]
    fn upcaster_transforms_v2_to_v3() {
        let upcaster = TestEventV2ToV3;

        let v2_payload = json!({
            "user_id": "user-123",
            "data": "test"
        });

        let v3_payload = upcaster.upcast(v2_payload).unwrap();

        assert_eq!(v3_payload["owner"]["user_id"], "user-123");
        assert_eq!(v3_payload["owner"]["display_name"], "Unknown");
    }

    #[test]
    fn upcaster_returns_error_for_missing_field() {
        let upcaster = TestEventV2ToV3;

        let invalid_payload = json!({
            "data": "test"
            // Missing user_id
        });

        let result = upcaster.upcast(invalid_payload);

        assert!(result.is_err());
        assert!(matches!(result, Err(UpcastError::MissingField(_))));
    }

    // ============================================================
    // UpcasterRegistry Tests
    // ============================================================

    #[test]
    fn registry_upcasts_single_version_step() {
        let mut registry = UpcasterRegistry::new();
        registry.register(Arc::new(TestEventV1ToV2));
        registry.set_current_version("test.event", 2);

        let v1_envelope = EventEnvelope {
            event_id: EventId::from_string("evt-1"),
            event_type: "test.event.v1".to_string(),
            schema_version: 1,
            aggregate_id: "agg-1".to_string(),
            aggregate_type: "Test".to_string(),
            occurred_at: Timestamp::now(),
            payload: json!({"data": "test"}),
            metadata: EventMetadata::default(),
        };

        let v2_envelope = registry.upcast_to_current(v1_envelope).unwrap();

        assert_eq!(v2_envelope.schema_version, 2);
        assert_eq!(v2_envelope.event_type, "test.event.v2");
        assert!(v2_envelope.payload["description"].is_null());
    }

    #[test]
    fn registry_chains_multiple_upcasters() {
        let mut registry = UpcasterRegistry::new();
        registry.register(Arc::new(TestEventV1ToV2));
        registry.register(Arc::new(TestEventV2ToV3));
        registry.set_current_version("test.event", 3);

        let v1_envelope = EventEnvelope {
            event_id: EventId::from_string("evt-1"),
            event_type: "test.event.v1".to_string(),
            schema_version: 1,
            aggregate_id: "agg-1".to_string(),
            aggregate_type: "Test".to_string(),
            occurred_at: Timestamp::now(),
            payload: json!({"user_id": "user-123", "data": "test"}),
            metadata: EventMetadata::default(),
        };

        let v3_envelope = registry.upcast_to_current(v1_envelope).unwrap();

        assert_eq!(v3_envelope.schema_version, 3);
        assert_eq!(v3_envelope.event_type, "test.event.v3");
        assert_eq!(v3_envelope.payload["owner"]["user_id"], "user-123");
    }

    #[test]
    fn registry_returns_unchanged_if_already_current_version() {
        let registry = UpcasterRegistry::new();

        let current_envelope = EventEnvelope {
            event_id: EventId::from_string("evt-1"),
            event_type: "test.event.v2".to_string(),
            schema_version: 2,
            aggregate_id: "agg-1".to_string(),
            aggregate_type: "Test".to_string(),
            occurred_at: Timestamp::now(),
            payload: json!({"data": "test"}),
            metadata: EventMetadata::default(),
        };

        let result = registry.upcast_to_current(current_envelope.clone()).unwrap();

        assert_eq!(result.schema_version, 2);
        assert_eq!(result.event_type, "test.event.v2");
    }

    #[test]
    fn registry_returns_error_for_missing_upcaster() {
        let mut registry = UpcasterRegistry::new();
        // No upcasters registered, but we say current version is 3
        registry.set_current_version("test.event", 3);

        let v1_envelope = EventEnvelope {
            event_id: EventId::from_string("evt-1"),
            event_type: "test.event.v1".to_string(),
            schema_version: 1,
            aggregate_id: "agg-1".to_string(),
            aggregate_type: "Test".to_string(),
            occurred_at: Timestamp::now(),
            payload: json!({"data": "test"}),
            metadata: EventMetadata::default(),
        };

        let result = registry.upcast_to_current(v1_envelope);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(UpcastError::IncompatibleVersions { .. })
        ));
    }

    #[test]
    fn extract_base_type_removes_version_suffix() {
        assert_eq!(
            UpcasterRegistry::extract_base_type("session.created.v2"),
            "session.created"
        );
        assert_eq!(
            UpcasterRegistry::extract_base_type("cycle.completed.v10"),
            "cycle.completed"
        );
        assert_eq!(
            UpcasterRegistry::extract_base_type("legacy.event"),
            "legacy.event"
        );
    }

    #[test]
    fn registry_preserves_envelope_metadata() {
        let mut registry = UpcasterRegistry::new();
        registry.register(Arc::new(TestEventV1ToV2));
        registry.set_current_version("test.event", 2);

        let v1_envelope = EventEnvelope {
            event_id: EventId::from_string("evt-original"),
            event_type: "test.event.v1".to_string(),
            schema_version: 1,
            aggregate_id: "agg-123".to_string(),
            aggregate_type: "Test".to_string(),
            occurred_at: Timestamp::now(),
            payload: json!({"data": "test"}),
            metadata: EventMetadata {
                correlation_id: Some("corr-1".to_string()),
                causation_id: None,
                user_id: Some("user-1".to_string()),
                trace_id: None,
            },
        };

        let occurred_at = v1_envelope.occurred_at;
        let v2_envelope = registry.upcast_to_current(v1_envelope).unwrap();

        // Metadata should be preserved
        assert_eq!(v2_envelope.event_id.as_str(), "evt-original");
        assert_eq!(v2_envelope.aggregate_id, "agg-123");
        assert_eq!(v2_envelope.aggregate_type, "Test");
        assert_eq!(v2_envelope.occurred_at, occurred_at);
        assert_eq!(
            v2_envelope.metadata.correlation_id,
            Some("corr-1".to_string())
        );
        assert_eq!(v2_envelope.metadata.user_id, Some("user-1".to_string()));
    }
}
