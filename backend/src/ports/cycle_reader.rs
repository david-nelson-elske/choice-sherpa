//! Cycle reader port (read side / CQRS queries).
//!
//! Defines the contract for cycle queries and read operations.
//! Optimized for UI display, progress tracking, and tree visualization.
//!
//! # Design
//!
//! - **Read-optimized**: Can use caching, denormalized views
//! - **Separated from write**: CQRS pattern for scalability
//! - **Tree support**: Queries for cycle branches and lineage

use crate::domain::foundation::{
    ComponentStatus, ComponentType, CycleId, CycleStatus, DomainError, SessionId, Timestamp,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Reader port for cycle queries.
///
/// Provides read-optimized views of cycle data.
/// Implementations may use caching for frequently-accessed data.
#[async_trait]
pub trait CycleReader: Send + Sync {
    /// Get detailed cycle view by ID.
    ///
    /// Returns `None` if not found.
    async fn get_by_id(&self, id: &CycleId) -> Result<Option<CycleView>, DomainError>;

    /// List all cycles for a session.
    ///
    /// Returns cycles ordered by created_at descending.
    async fn list_by_session_id(&self, session_id: &SessionId)
        -> Result<Vec<CycleSummary>, DomainError>;

    /// Get the cycle tree for a session.
    ///
    /// Returns the root cycle with all its branches organized hierarchically.
    async fn get_tree(&self, session_id: &SessionId) -> Result<Option<CycleTreeNode>, DomainError>;

    /// Get the progress view for a cycle.
    ///
    /// Returns detailed progress information for all components.
    async fn get_progress(&self, id: &CycleId) -> Result<Option<CycleProgressView>, DomainError>;

    /// Get lineage (path from root to this cycle).
    ///
    /// Returns ordered list from root to the specified cycle.
    async fn get_lineage(&self, id: &CycleId) -> Result<Vec<CycleSummary>, DomainError>;

    /// Get a component's output from a cycle.
    ///
    /// Returns the component's structured output and status information.
    async fn get_component_output(
        &self,
        cycle_id: &CycleId,
        component_type: ComponentType,
    ) -> Result<Option<ComponentOutputView>, DomainError>;
}

/// Detailed view of a cycle for UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleView {
    /// Cycle ID.
    pub id: CycleId,

    /// Session this cycle belongs to.
    pub session_id: SessionId,

    /// Parent cycle ID if this is a branch.
    pub parent_cycle_id: Option<CycleId>,

    /// Component where branching occurred.
    pub branch_point: Option<ComponentType>,

    /// Current cycle status.
    pub status: CycleStatus,

    /// Currently active component.
    pub current_step: ComponentType,

    /// Status of each component.
    pub component_statuses: Vec<ComponentStatusItem>,

    /// Overall progress percentage (0-100).
    pub progress_percent: u8,

    /// Whether cycle is complete.
    pub is_complete: bool,

    /// Number of child branches.
    pub branch_count: u32,

    /// When the cycle was created.
    pub created_at: Timestamp,

    /// When the cycle was last updated.
    pub updated_at: Timestamp,
}

/// Status of a single component within a cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatusItem {
    /// Component type.
    pub component_type: ComponentType,

    /// Current status.
    pub status: ComponentStatus,

    /// Whether this component is the current step.
    pub is_current: bool,
}

/// Summary view of a cycle for lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleSummary {
    /// Cycle ID.
    pub id: CycleId,

    /// Whether this is a branch.
    pub is_branch: bool,

    /// Branch point component (if a branch).
    pub branch_point: Option<ComponentType>,

    /// Current cycle status.
    pub status: CycleStatus,

    /// Currently active component.
    pub current_step: ComponentType,

    /// Overall progress percentage.
    pub progress_percent: u8,

    /// When the cycle was created.
    pub created_at: Timestamp,
}

/// Tree node for cycle hierarchy visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleTreeNode {
    /// Summary of this cycle.
    pub cycle: CycleSummary,

    /// Child branches.
    pub children: Vec<CycleTreeNode>,
}

/// Detailed progress view for a cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleProgressView {
    /// Cycle ID.
    pub cycle_id: CycleId,

    /// Overall progress percentage.
    pub progress_percent: u8,

    /// Number of completed components.
    pub completed_count: u8,

    /// Total required components (excludes optional NotesNextSteps).
    pub required_count: u8,

    /// All required components are complete.
    pub is_complete: bool,

    /// Any component needs revision.
    pub has_revisions: bool,

    /// Detailed status of each component.
    pub steps: Vec<ProgressStep>,

    /// Next recommended action.
    pub next_action: Option<NextAction>,
}

/// Progress information for a single component step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressStep {
    /// Component type.
    pub component_type: ComponentType,

    /// Component display name.
    pub name: String,

    /// Current status.
    pub status: ComponentStatus,

    /// Whether this is the current step.
    pub is_current: bool,

    /// Can navigate to this step.
    pub is_accessible: bool,
}

/// Recommended next action for the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextAction {
    /// Type of action.
    pub action_type: NextActionType,

    /// Target component (if applicable).
    pub component: Option<ComponentType>,

    /// Human-readable description.
    pub description: String,
}

/// Type of next action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NextActionType {
    /// Start the first component.
    StartFirst,

    /// Continue working on current component.
    ContinueCurrent,

    /// Start the next component.
    StartNext,

    /// Revise a component marked for revision.
    ReviseComponent,

    /// Complete the cycle (DQ is done).
    CompleteCycle,

    /// Cycle is already complete.
    AlreadyComplete,
}

/// View of a component's output for queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentOutputView {
    /// The cycle this component belongs to.
    pub cycle_id: CycleId,

    /// Component type.
    pub component_type: ComponentType,

    /// Current status of the component.
    pub status: ComponentStatus,

    /// The structured output data (schema varies by component type).
    pub output: JsonValue,

    /// When the component was last updated.
    pub updated_at: Timestamp,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trait object safety test
    #[test]
    fn cycle_reader_is_object_safe() {
        fn _accepts_dyn(_reader: &dyn CycleReader) {}
    }

    #[test]
    fn next_action_type_is_copyable() {
        let action = NextActionType::StartFirst;
        let copied = action;
        assert_eq!(action, copied);
    }

    #[test]
    fn cycle_summary_serializes_to_json() {
        let summary = CycleSummary {
            id: CycleId::new(),
            is_branch: false,
            branch_point: None,
            status: CycleStatus::Active,
            current_step: ComponentType::IssueRaising,
            progress_percent: 25,
            created_at: Timestamp::now(),
        };

        let json = serde_json::to_string(&summary).expect("serialization failed");
        assert!(json.contains("issue_raising"));
        assert!(json.contains("25"));
    }

    #[test]
    fn cycle_tree_node_can_nest() {
        let child = CycleTreeNode {
            cycle: CycleSummary {
                id: CycleId::new(),
                is_branch: true,
                branch_point: Some(ComponentType::Alternatives),
                status: CycleStatus::Active,
                current_step: ComponentType::Alternatives,
                progress_percent: 50,
                created_at: Timestamp::now(),
            },
            children: vec![],
        };

        let root = CycleTreeNode {
            cycle: CycleSummary {
                id: CycleId::new(),
                is_branch: false,
                branch_point: None,
                status: CycleStatus::Active,
                current_step: ComponentType::Tradeoffs,
                progress_percent: 75,
                created_at: Timestamp::now(),
            },
            children: vec![child],
        };

        assert_eq!(root.children.len(), 1);
        assert!(root.children[0].cycle.is_branch);
    }
}
