//! Tool Definitions - Specific tool implementations for each PrOACT component.
//!
//! This module contains the parameter and result types for all atomic decision tools,
//! organized by PrOACT component. Each component has its own submodule.
//!
//! ## Module Structure
//!
//! - [`issue_raising`] - Tools for categorizing initial thoughts
//! - [`problem_frame`] - Tools for defining decision architecture
//! - [`objectives`] - Tools for identifying and organizing objectives
//! - [`alternatives`] - Tools for capturing options
//! - [`consequences`] - Tools for building consequence tables
//! - [`tradeoffs`] - Tools for surfacing dominated alternatives
//! - [`recommendation`] - Tools for synthesizing analysis
//! - [`decision_quality`] - Tools for rating decision quality elements
//! - [`cross_cutting`] - Tools available in all components

pub mod issue_raising;
pub mod problem_frame;
pub mod objectives;
pub mod alternatives;
pub mod consequences;
pub mod tradeoffs;
pub mod recommendation;
pub mod decision_quality;
pub mod cross_cutting;

// Re-export common types
pub use issue_raising::*;
pub use problem_frame::*;
pub use objectives::*;
pub use alternatives::*;
pub use consequences::*;
pub use tradeoffs::*;
pub use recommendation::*;
pub use decision_quality::*;
pub use cross_cutting::*;
