//! Analysis Module - Pure domain services for decision analysis.
//!
//! This module contains stateless functions that operate on domain objects
//! to perform decision-related calculations and analysis.
//!
//! # Components
//!
//! - `ConsequencesTable` - Core data structure for Pugh matrix analysis
//! - `PughAnalyzer` - Score computation, dominance detection, irrelevant objectives
//! - `DQCalculator` - Decision Quality scoring (7 elements, overall = minimum)
//! - `TradeoffAnalyzer` - Tension analysis for non-dominated alternatives
//!
//! # Design Philosophy
//!
//! All functions are pure (no side effects) and stateless. They take domain
//! objects as input and return computed results. No ports or adapters needed
//! since there's no I/O or external dependencies.

mod consequences_table;
mod dq_calculator;
mod events;
mod pugh_analyzer;
mod tradeoff_analyzer;

// Re-export all public types
pub use consequences_table::{Cell, ConsequencesTable, ConsequencesTableBuilder};
pub use dq_calculator::{
    DQCalculator, DQElement, Priority, DQ_ACCEPTABLE_THRESHOLD, DQ_ELEMENT_NAMES,
};
pub use events::{
    DQElementScore, DQScoresComputed, PughScoresComputed, TensionSummary, TradeoffsAnalyzed,
};
pub use pugh_analyzer::{DominatedAlternative, IrrelevantObjective, PughAnalyzer};
pub use tradeoff_analyzer::{Tension, TradeoffAnalyzer, TradeoffSummary};
