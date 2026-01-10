//! Cycle command and query handlers.
//!
//! Handlers for cycle lifecycle operations and queries.

// Command handlers
mod archive_cycle;
mod branch_cycle;
mod complete_component;
mod complete_cycle;
mod create_cycle;
mod navigate_to_component;
mod start_component;
mod update_component_output;

// Query handlers
mod get_component;
mod get_cycle;
mod get_cycle_tree;

pub use archive_cycle::{
    ArchiveCycleCommand, ArchiveCycleError, ArchiveCycleHandler, ArchiveCycleResult,
    CycleArchivedEvent,
};
pub use branch_cycle::{
    BranchCycleCommand, BranchCycleError, BranchCycleHandler, BranchCycleResult, CycleBranchedEvent,
};
pub use complete_component::{
    CompleteComponentCommand, CompleteComponentError, CompleteComponentHandler,
    CompleteComponentResult, ComponentCompletedEvent,
};
pub use complete_cycle::{
    CompleteCycleCommand, CompleteCycleError, CompleteCycleHandler, CompleteCycleResult,
    CycleCompletedEvent,
};
pub use create_cycle::{
    CreateCycleCommand, CreateCycleError, CreateCycleHandler, CreateCycleResult, CycleCreatedEvent,
};
pub use navigate_to_component::{
    NavigateToComponentCommand, NavigateToComponentError, NavigateToComponentHandler,
    NavigateToComponentResult, NavigatedToComponentEvent,
};
pub use start_component::{
    ComponentStartedEvent, StartComponentCommand, StartComponentError, StartComponentHandler,
    StartComponentResult,
};
pub use update_component_output::{
    ComponentOutputUpdatedEvent, UpdateComponentOutputCommand, UpdateComponentOutputError,
    UpdateComponentOutputHandler, UpdateComponentOutputResult,
};

// Query handlers
pub use get_component::{GetComponentHandler, GetComponentQuery, GetComponentResult};
pub use get_cycle::{GetCycleHandler, GetCycleQuery, GetCycleResult};
pub use get_cycle_tree::{GetCycleTreeHandler, GetCycleTreeQuery, GetCycleTreeResult};
