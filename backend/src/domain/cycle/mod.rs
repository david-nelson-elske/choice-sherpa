//! Cycle module - Decision cycle aggregate and lifecycle management.
//!
//! A Cycle represents a complete or partial path through the PrOACT framework.
//! Cycles own their components and support branching for "what-if" exploration.

mod aggregate;
mod events;

pub use aggregate::Cycle;
pub use events::CycleEvent;
