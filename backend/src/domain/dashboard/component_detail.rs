use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::domain::foundation::{ComponentId, ComponentStatus, ComponentType, CycleId};

/// Detailed view of a single component
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDetailView {
    pub component_id: ComponentId,
    pub cycle_id: CycleId,
    pub component_type: ComponentType,
    pub status: ComponentStatus,

    /// Full structured output (type-specific JSON)
    pub structured_output: serde_json::Value,

    /// Conversation metadata
    pub conversation_message_count: usize,
    pub last_message_at: Option<DateTime<Utc>>,

    /// Actions
    pub can_branch: bool,
    pub can_revise: bool,

    /// Navigation context
    pub previous_component: Option<ComponentType>,
    pub next_component: Option<ComponentType>,
}

impl ComponentDetailView {
    /// Returns display name for the component
    pub fn display_name(&self) -> &'static str {
        self.component_type.display_name()
    }

    /// Returns true if component has been started
    pub fn is_started(&self) -> bool {
        self.status.is_started()
    }

    /// Returns true if component is complete
    pub fn is_complete(&self) -> bool {
        self.status.is_complete()
    }
}

#[cfg(test)]
#[path = "component_detail_test.rs"]
mod component_detail_test;
