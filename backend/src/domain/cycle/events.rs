//! Cycle domain events.

use crate::domain::foundation::{ComponentType, CycleId, Timestamp};
use serde::{Deserialize, Serialize};

/// Events that can occur during cycle lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}
