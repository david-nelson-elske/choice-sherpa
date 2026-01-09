//! Session domain events.
//!
//! Events published when session lifecycle changes occur:
//! - `SessionCreated` - New session created
//! - `SessionRenamed` - Session title changed
//! - `SessionDescriptionUpdated` - Session description changed
//! - `SessionArchived` - Session archived (soft delete)
//! - `CycleAddedToSession` - Cycle linked to session

use serde::{Deserialize, Serialize};

use crate::domain::foundation::{
    domain_event, CycleId, EventId, SessionId, Timestamp, UserId,
};

// ════════════════════════════════════════════════════════════════════════════
// SessionCreated
// ════════════════════════════════════════════════════════════════════════════

/// Published when a new decision session is created.
///
/// Contains all initial session data including title and optional description.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreated {
    /// Unique identifier for this event.
    pub event_id: EventId,

    /// ID of the created session.
    pub session_id: SessionId,

    /// User who created the session.
    pub user_id: UserId,

    /// Session title.
    pub title: String,

    /// Optional description.
    pub description: Option<String>,

    /// When the session was created.
    pub created_at: Timestamp,
}

domain_event!(
    SessionCreated,
    event_type = "session.created",
    aggregate_id = session_id,
    aggregate_type = "Session",
    occurred_at = created_at,
    event_id = event_id
);

// ════════════════════════════════════════════════════════════════════════════
// SessionRenamed
// ════════════════════════════════════════════════════════════════════════════

/// Published when a session's title is changed.
///
/// Captures both old and new title for audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRenamed {
    /// Unique identifier for this event.
    pub event_id: EventId,

    /// ID of the renamed session.
    pub session_id: SessionId,

    /// User who renamed the session.
    pub user_id: UserId,

    /// Previous title.
    pub old_title: String,

    /// New title.
    pub new_title: String,

    /// When the rename occurred.
    pub renamed_at: Timestamp,
}

domain_event!(
    SessionRenamed,
    event_type = "session.renamed",
    aggregate_id = session_id,
    aggregate_type = "Session",
    occurred_at = renamed_at,
    event_id = event_id
);

// ════════════════════════════════════════════════════════════════════════════
// SessionDescriptionUpdated
// ════════════════════════════════════════════════════════════════════════════

/// Published when a session's description is updated.
///
/// Captures both old and new descriptions for audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDescriptionUpdated {
    /// Unique identifier for this event.
    pub event_id: EventId,

    /// ID of the updated session.
    pub session_id: SessionId,

    /// User who updated the description.
    pub user_id: UserId,

    /// Previous description (None if previously empty).
    pub old_description: Option<String>,

    /// New description (None if cleared).
    pub new_description: Option<String>,

    /// When the update occurred.
    pub updated_at: Timestamp,
}

domain_event!(
    SessionDescriptionUpdated,
    event_type = "session.description_updated",
    aggregate_id = session_id,
    aggregate_type = "Session",
    occurred_at = updated_at,
    event_id = event_id
);

// ════════════════════════════════════════════════════════════════════════════
// SessionArchived
// ════════════════════════════════════════════════════════════════════════════

/// Published when a session is archived (soft delete).
///
/// Archived sessions are hidden from active lists but data is preserved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArchived {
    /// Unique identifier for this event.
    pub event_id: EventId,

    /// ID of the archived session.
    pub session_id: SessionId,

    /// User who archived the session.
    pub user_id: UserId,

    /// When the session was archived.
    pub archived_at: Timestamp,
}

domain_event!(
    SessionArchived,
    event_type = "session.archived",
    aggregate_id = session_id,
    aggregate_type = "Session",
    occurred_at = archived_at,
    event_id = event_id
);

// ════════════════════════════════════════════════════════════════════════════
// CycleAddedToSession
// ════════════════════════════════════════════════════════════════════════════

/// Published when a cycle is linked to a session.
///
/// This event is typically published in response to a CycleCreated event
/// from the Cycle module, allowing the Session to maintain its cycle list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleAddedToSession {
    /// Unique identifier for this event.
    pub event_id: EventId,

    /// ID of the session receiving the cycle.
    pub session_id: SessionId,

    /// ID of the cycle being added.
    pub cycle_id: CycleId,

    /// Whether this is the root (first) cycle for the session.
    pub is_root_cycle: bool,

    /// When the cycle was added.
    pub added_at: Timestamp,
}

domain_event!(
    CycleAddedToSession,
    event_type = "session.cycle_added",
    aggregate_id = session_id,
    aggregate_type = "Session",
    occurred_at = added_at,
    event_id = event_id
);

// ════════════════════════════════════════════════════════════════════════════
// Unit Tests
// ════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{DomainEvent, SerializableDomainEvent};

    // ────────────────────────────────────────────────────────────────────────
    // SessionCreated Tests
    // ────────────────────────────────────────────────────────────────────────

    #[test]
    fn session_created_implements_domain_event() {
        let event = SessionCreated {
            event_id: EventId::new(),
            session_id: SessionId::new(),
            user_id: UserId::new("user-1").unwrap(),
            title: "Test Decision".to_string(),
            description: None,
            created_at: Timestamp::now(),
        };

        assert_eq!(event.event_type(), "session.created");
        assert_eq!(event.aggregate_type(), "Session");
        assert!(!event.aggregate_id().is_empty());
    }

    #[test]
    fn session_created_serializes_to_json() {
        let session_id = SessionId::new();
        let event = SessionCreated {
            event_id: EventId::from_string("evt-1"),
            session_id,
            user_id: UserId::new("user-1").unwrap(),
            title: "My Decision".to_string(),
            description: Some("Description".to_string()),
            created_at: Timestamp::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("My Decision"));
        assert!(json.contains(&session_id.to_string()));
    }

    #[test]
    fn session_created_to_envelope_works() {
        let event = SessionCreated {
            event_id: EventId::from_string("evt-123"),
            session_id: SessionId::new(),
            user_id: UserId::new("user-1").unwrap(),
            title: "Test".to_string(),
            description: None,
            created_at: Timestamp::now(),
        };

        let envelope = event.to_envelope();
        assert_eq!(envelope.event_type, "session.created");
        assert_eq!(envelope.aggregate_type, "Session");
        assert_eq!(envelope.event_id.as_str(), "evt-123");
    }

    // ────────────────────────────────────────────────────────────────────────
    // SessionRenamed Tests
    // ────────────────────────────────────────────────────────────────────────

    #[test]
    fn session_renamed_captures_both_titles() {
        let event = SessionRenamed {
            event_id: EventId::new(),
            session_id: SessionId::new(),
            user_id: UserId::new("user-1").unwrap(),
            old_title: "Old Title".to_string(),
            new_title: "New Title".to_string(),
            renamed_at: Timestamp::now(),
        };

        assert_eq!(event.old_title, "Old Title");
        assert_eq!(event.new_title, "New Title");
        assert_eq!(event.event_type(), "session.renamed");
    }

    #[test]
    fn session_renamed_serializes_correctly() {
        let event = SessionRenamed {
            event_id: EventId::new(),
            session_id: SessionId::new(),
            user_id: UserId::new("user-1").unwrap(),
            old_title: "Before".to_string(),
            new_title: "After".to_string(),
            renamed_at: Timestamp::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Before"));
        assert!(json.contains("After"));
    }

    // ────────────────────────────────────────────────────────────────────────
    // SessionDescriptionUpdated Tests
    // ────────────────────────────────────────────────────────────────────────

    #[test]
    fn session_description_updated_captures_both_descriptions() {
        let event = SessionDescriptionUpdated {
            event_id: EventId::new(),
            session_id: SessionId::new(),
            user_id: UserId::new("user-1").unwrap(),
            old_description: Some("Old desc".to_string()),
            new_description: Some("New desc".to_string()),
            updated_at: Timestamp::now(),
        };

        assert_eq!(event.old_description, Some("Old desc".to_string()));
        assert_eq!(event.new_description, Some("New desc".to_string()));
        assert_eq!(event.event_type(), "session.description_updated");
    }

    #[test]
    fn session_description_updated_handles_none_values() {
        let event = SessionDescriptionUpdated {
            event_id: EventId::new(),
            session_id: SessionId::new(),
            user_id: UserId::new("user-1").unwrap(),
            old_description: None,
            new_description: Some("First description".to_string()),
            updated_at: Timestamp::now(),
        };

        assert!(event.old_description.is_none());
        assert!(event.new_description.is_some());
    }

    // ────────────────────────────────────────────────────────────────────────
    // SessionArchived Tests
    // ────────────────────────────────────────────────────────────────────────

    #[test]
    fn session_archived_implements_domain_event() {
        let event = SessionArchived {
            event_id: EventId::new(),
            session_id: SessionId::new(),
            user_id: UserId::new("user-1").unwrap(),
            archived_at: Timestamp::now(),
        };

        assert_eq!(event.event_type(), "session.archived");
        assert_eq!(event.aggregate_type(), "Session");
    }

    #[test]
    fn session_archived_serializes_correctly() {
        let event = SessionArchived {
            event_id: EventId::from_string("evt-archive"),
            session_id: SessionId::new(),
            user_id: UserId::new("user-1").unwrap(),
            archived_at: Timestamp::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let restored: SessionArchived = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.event_id.as_str(), "evt-archive");
    }

    // ────────────────────────────────────────────────────────────────────────
    // CycleAddedToSession Tests
    // ────────────────────────────────────────────────────────────────────────

    #[test]
    fn cycle_added_to_session_implements_domain_event() {
        let event = CycleAddedToSession {
            event_id: EventId::new(),
            session_id: SessionId::new(),
            cycle_id: CycleId::new(),
            is_root_cycle: true,
            added_at: Timestamp::now(),
        };

        assert_eq!(event.event_type(), "session.cycle_added");
        assert_eq!(event.aggregate_type(), "Session");
    }

    #[test]
    fn cycle_added_to_session_tracks_root_status() {
        let root_event = CycleAddedToSession {
            event_id: EventId::new(),
            session_id: SessionId::new(),
            cycle_id: CycleId::new(),
            is_root_cycle: true,
            added_at: Timestamp::now(),
        };

        let branch_event = CycleAddedToSession {
            event_id: EventId::new(),
            session_id: SessionId::new(),
            cycle_id: CycleId::new(),
            is_root_cycle: false,
            added_at: Timestamp::now(),
        };

        assert!(root_event.is_root_cycle);
        assert!(!branch_event.is_root_cycle);
    }

    #[test]
    fn cycle_added_serialization_round_trip() {
        let session_id = SessionId::new();
        let cycle_id = CycleId::new();
        let event = CycleAddedToSession {
            event_id: EventId::from_string("evt-cycle"),
            session_id,
            cycle_id,
            is_root_cycle: true,
            added_at: Timestamp::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let restored: CycleAddedToSession = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.session_id, session_id);
        assert_eq!(restored.cycle_id, cycle_id);
        assert!(restored.is_root_cycle);
    }

    // ────────────────────────────────────────────────────────────────────────
    // Envelope Tests (via SerializableDomainEvent)
    // ────────────────────────────────────────────────────────────────────────

    #[test]
    fn all_events_produce_valid_envelopes() {
        let session_id = SessionId::new();
        let user_id = UserId::new("user-1").unwrap();

        let created = SessionCreated {
            event_id: EventId::new(),
            session_id,
            user_id: user_id.clone(),
            title: "Test".to_string(),
            description: None,
            created_at: Timestamp::now(),
        };

        let renamed = SessionRenamed {
            event_id: EventId::new(),
            session_id,
            user_id: user_id.clone(),
            old_title: "Old".to_string(),
            new_title: "New".to_string(),
            renamed_at: Timestamp::now(),
        };

        let description_updated = SessionDescriptionUpdated {
            event_id: EventId::new(),
            session_id,
            user_id: user_id.clone(),
            old_description: None,
            new_description: Some("Desc".to_string()),
            updated_at: Timestamp::now(),
        };

        let archived = SessionArchived {
            event_id: EventId::new(),
            session_id,
            user_id: user_id.clone(),
            archived_at: Timestamp::now(),
        };

        let cycle_added = CycleAddedToSession {
            event_id: EventId::new(),
            session_id,
            cycle_id: CycleId::new(),
            is_root_cycle: true,
            added_at: Timestamp::now(),
        };

        // All should produce envelopes with Session aggregate type
        assert_eq!(created.to_envelope().aggregate_type, "Session");
        assert_eq!(renamed.to_envelope().aggregate_type, "Session");
        assert_eq!(description_updated.to_envelope().aggregate_type, "Session");
        assert_eq!(archived.to_envelope().aggregate_type, "Session");
        assert_eq!(cycle_added.to_envelope().aggregate_type, "Session");

        // All should have the same aggregate ID (session_id)
        let expected_agg_id = session_id.to_string();
        assert_eq!(created.to_envelope().aggregate_id, expected_agg_id);
        assert_eq!(renamed.to_envelope().aggregate_id, expected_agg_id);
        assert_eq!(description_updated.to_envelope().aggregate_id, expected_agg_id);
        assert_eq!(archived.to_envelope().aggregate_id, expected_agg_id);
        assert_eq!(cycle_added.to_envelope().aggregate_id, expected_agg_id);
    }
}
