//! Cycle module - Decision cycle aggregate and lifecycle management.
//!
//! A Cycle represents a complete or partial path through the PrOACT framework.
//! Cycles own their components and support branching for "what-if" exploration.

mod aggregate;
mod events;
mod progress;
mod tree_view;

pub use aggregate::Cycle;
pub use events::CycleEvent;
pub use progress::CycleProgress;
pub use tree_view::{
    BranchMetadata, CycleTreeNode, LetterStatus, PrOACTLetter, PrOACTStatus, PositionHint,
};
