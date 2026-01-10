//! Cycle domain events.
//!
//! Events emitted during cycle lifecycle operations. These events are used
//! for event sourcing, audit trails, and triggering side effects.

use crate::domain::foundation::{ComponentType, CycleId, Timestamp};
use serde::{Deserialize, Serialize};

/// Events that can occur during cycle lifecycle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CycleEvent {
    /// A new cycle was created.
    Created {
        cycle_id: CycleId,
        created_at: Timestamp,
    },

    /// A cycle was branched from a parent.
    Branched {
        cycle_id: CycleId,
        parent_cycle_id: CycleId,
        branch_point: ComponentType,
        created_at: Timestamp,
    },

    /// A cycle was completed.
    Completed { cycle_id: CycleId },

    /// A cycle was archived.
    Archived { cycle_id: CycleId },

    /// A component was started.
    ComponentStarted {
        cycle_id: CycleId,
        component_type: ComponentType,
    },

    /// A component was completed.
    ComponentCompleted {
        cycle_id: CycleId,
        component_type: ComponentType,
    },

    /// A component was marked for revision.
    ComponentMarkedForRevision {
        cycle_id: CycleId,
        component_type: ComponentType,
        reason: String,
    },

    /// Navigation changed to a different component.
    NavigatedTo {
        cycle_id: CycleId,
        component_type: ComponentType,
    },

    /// A component's output was updated.
    ComponentOutputUpdated {
        cycle_id: CycleId,
        component_type: ComponentType,
    },
}

impl CycleEvent {
    /// Returns the cycle ID associated with this event.
    ///
    /// Every cycle event is associated with exactly one cycle.
    pub fn cycle_id(&self) -> CycleId {
        match self {
            CycleEvent::Created { cycle_id, .. } => *cycle_id,
            CycleEvent::Branched { cycle_id, .. } => *cycle_id,
            CycleEvent::Completed { cycle_id } => *cycle_id,
            CycleEvent::Archived { cycle_id } => *cycle_id,
            CycleEvent::ComponentStarted { cycle_id, .. } => *cycle_id,
            CycleEvent::ComponentCompleted { cycle_id, .. } => *cycle_id,
            CycleEvent::ComponentMarkedForRevision { cycle_id, .. } => *cycle_id,
            CycleEvent::NavigatedTo { cycle_id, .. } => *cycle_id,
            CycleEvent::ComponentOutputUpdated { cycle_id, .. } => *cycle_id,
        }
    }

    /// Returns the event type name for logging and debugging.
    pub fn event_type(&self) -> &'static str {
        match self {
            CycleEvent::Created { .. } => "CycleCreated",
            CycleEvent::Branched { .. } => "CycleBranched",
            CycleEvent::Completed { .. } => "CycleCompleted",
            CycleEvent::Archived { .. } => "CycleArchived",
            CycleEvent::ComponentStarted { .. } => "ComponentStarted",
            CycleEvent::ComponentCompleted { .. } => "ComponentCompleted",
            CycleEvent::ComponentMarkedForRevision { .. } => "ComponentMarkedForRevision",
            CycleEvent::NavigatedTo { .. } => "NavigatedTo",
            CycleEvent::ComponentOutputUpdated { .. } => "ComponentOutputUpdated",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cycle_id() -> CycleId {
        CycleId::new()
    }

    // ───────────────────────────────────────────────────────────────
    // cycle_id accessor tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn cycle_id_returns_id_for_created() {
        let id = test_cycle_id();
        let event = CycleEvent::Created {
            cycle_id: id,
            created_at: Timestamp::now(),
        };
        assert_eq!(event.cycle_id(), id);
    }

    #[test]
    fn cycle_id_returns_id_for_branched() {
        let id = test_cycle_id();
        let parent_id = test_cycle_id();
        let event = CycleEvent::Branched {
            cycle_id: id,
            parent_cycle_id: parent_id,
            branch_point: ComponentType::Objectives,
            created_at: Timestamp::now(),
        };
        assert_eq!(event.cycle_id(), id);
    }

    #[test]
    fn cycle_id_returns_id_for_completed() {
        let id = test_cycle_id();
        let event = CycleEvent::Completed { cycle_id: id };
        assert_eq!(event.cycle_id(), id);
    }

    #[test]
    fn cycle_id_returns_id_for_archived() {
        let id = test_cycle_id();
        let event = CycleEvent::Archived { cycle_id: id };
        assert_eq!(event.cycle_id(), id);
    }

    #[test]
    fn cycle_id_returns_id_for_component_started() {
        let id = test_cycle_id();
        let event = CycleEvent::ComponentStarted {
            cycle_id: id,
            component_type: ComponentType::IssueRaising,
        };
        assert_eq!(event.cycle_id(), id);
    }

    #[test]
    fn cycle_id_returns_id_for_component_completed() {
        let id = test_cycle_id();
        let event = CycleEvent::ComponentCompleted {
            cycle_id: id,
            component_type: ComponentType::ProblemFrame,
        };
        assert_eq!(event.cycle_id(), id);
    }

    #[test]
    fn cycle_id_returns_id_for_component_marked_for_revision() {
        let id = test_cycle_id();
        let event = CycleEvent::ComponentMarkedForRevision {
            cycle_id: id,
            component_type: ComponentType::Alternatives,
            reason: "Need more options".to_string(),
        };
        assert_eq!(event.cycle_id(), id);
    }

    #[test]
    fn cycle_id_returns_id_for_navigated_to() {
        let id = test_cycle_id();
        let event = CycleEvent::NavigatedTo {
            cycle_id: id,
            component_type: ComponentType::Tradeoffs,
        };
        assert_eq!(event.cycle_id(), id);
    }

    #[test]
    fn cycle_id_returns_id_for_component_output_updated() {
        let id = test_cycle_id();
        let event = CycleEvent::ComponentOutputUpdated {
            cycle_id: id,
            component_type: ComponentType::IssueRaising,
        };
        assert_eq!(event.cycle_id(), id);
    }

    // ───────────────────────────────────────────────────────────────
    // event_type accessor tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn event_type_returns_correct_name() {
        let id = test_cycle_id();

        assert_eq!(
            CycleEvent::Created {
                cycle_id: id,
                created_at: Timestamp::now()
            }
            .event_type(),
            "CycleCreated"
        );

        assert_eq!(
            CycleEvent::Completed { cycle_id: id }.event_type(),
            "CycleCompleted"
        );

        assert_eq!(
            CycleEvent::Archived { cycle_id: id }.event_type(),
            "CycleArchived"
        );

        assert_eq!(
            CycleEvent::ComponentStarted {
                cycle_id: id,
                component_type: ComponentType::IssueRaising
            }
            .event_type(),
            "ComponentStarted"
        );

        assert_eq!(
            CycleEvent::ComponentOutputUpdated {
                cycle_id: id,
                component_type: ComponentType::Objectives
            }
            .event_type(),
            "ComponentOutputUpdated"
        );
    }

    // ───────────────────────────────────────────────────────────────
    // JSON serialization tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn serializes_created_to_json() {
        let id = test_cycle_id();
        let event = CycleEvent::Created {
            cycle_id: id,
            created_at: Timestamp::now(),
        };

        let json = serde_json::to_string(&event).expect("serialization failed");
        assert!(json.contains("Created"));
        assert!(json.contains(&id.to_string()));
    }

    #[test]
    fn serializes_component_started_to_json() {
        let id = test_cycle_id();
        let event = CycleEvent::ComponentStarted {
            cycle_id: id,
            component_type: ComponentType::Objectives,
        };

        let json = serde_json::to_string(&event).expect("serialization failed");
        assert!(json.contains("ComponentStarted"));
        assert!(json.contains("objectives")); // snake_case from ComponentType
    }

    #[test]
    fn serializes_branched_to_json() {
        let id = test_cycle_id();
        let parent_id = test_cycle_id();
        let event = CycleEvent::Branched {
            cycle_id: id,
            parent_cycle_id: parent_id,
            branch_point: ComponentType::Alternatives,
            created_at: Timestamp::now(),
        };

        let json = serde_json::to_string(&event).expect("serialization failed");
        assert!(json.contains("Branched"));
        assert!(json.contains("parent_cycle_id"));
        assert!(json.contains("branch_point"));
    }

    // ───────────────────────────────────────────────────────────────
    // JSON deserialization tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn deserializes_created_from_json() {
        let id = test_cycle_id();
        let original = CycleEvent::Created {
            cycle_id: id,
            created_at: Timestamp::now(),
        };

        let json = serde_json::to_string(&original).expect("serialization failed");
        let deserialized: CycleEvent = serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(deserialized.cycle_id(), id);
    }

    #[test]
    fn deserializes_component_started_from_json() {
        let id = test_cycle_id();
        let original = CycleEvent::ComponentStarted {
            cycle_id: id,
            component_type: ComponentType::Consequences,
        };

        let json = serde_json::to_string(&original).expect("serialization failed");
        let deserialized: CycleEvent = serde_json::from_str(&json).expect("deserialization failed");

        if let CycleEvent::ComponentStarted {
            cycle_id,
            component_type,
        } = deserialized
        {
            assert_eq!(cycle_id, id);
            assert_eq!(component_type, ComponentType::Consequences);
        } else {
            panic!("Expected ComponentStarted event");
        }
    }

    #[test]
    fn roundtrip_preserves_all_fields() {
        let id = test_cycle_id();
        let parent_id = test_cycle_id();

        let original = CycleEvent::Branched {
            cycle_id: id,
            parent_cycle_id: parent_id,
            branch_point: ComponentType::Recommendation,
            created_at: Timestamp::now(),
        };

        let json = serde_json::to_string(&original).expect("serialization failed");
        let deserialized: CycleEvent = serde_json::from_str(&json).expect("deserialization failed");

        if let CycleEvent::Branched {
            cycle_id,
            parent_cycle_id,
            branch_point,
            ..
        } = deserialized
        {
            assert_eq!(cycle_id, id);
            assert_eq!(parent_cycle_id, parent_id);
            assert_eq!(branch_point, ComponentType::Recommendation);
        } else {
            panic!("Expected Branched event");
        }
    }

    #[test]
    fn deserializes_component_marked_for_revision_preserves_reason() {
        let id = test_cycle_id();
        let reason = "Analysis was incomplete".to_string();
        let original = CycleEvent::ComponentMarkedForRevision {
            cycle_id: id,
            component_type: ComponentType::Tradeoffs,
            reason: reason.clone(),
        };

        let json = serde_json::to_string(&original).expect("serialization failed");
        let deserialized: CycleEvent = serde_json::from_str(&json).expect("deserialization failed");

        if let CycleEvent::ComponentMarkedForRevision {
            reason: r, ..
        } = deserialized
        {
            assert_eq!(r, reason);
        } else {
            panic!("Expected ComponentMarkedForRevision event");
        }
    }
}
