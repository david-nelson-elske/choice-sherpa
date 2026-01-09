//! PrOACT Types module - Component types and traits for the PrOACT framework.
//!
//! This module defines:
//! - The Component trait that all PrOACT components implement
//! - The ComponentBase struct with lifecycle methods
//! - The 9 concrete component types (IssueRaising, ProblemFrame, etc.)
//! - The ComponentVariant enum for pattern matching
//! - Message types for conversation history

mod errors;
mod message;
mod component;
mod component_variant;
mod issue_raising;
mod problem_frame;
mod objectives;
mod alternatives;
mod consequences;
mod tradeoffs;
mod recommendation;
mod decision_quality;
mod notes_next_steps;

pub use errors::ComponentError;
pub use message::{Message, MessageId, MessageMetadata, Role};
pub use component::{Component, ComponentBase};
pub use component_variant::ComponentVariant;
pub use issue_raising::{IssueRaising, IssueRaisingOutput};
pub use problem_frame::{
    Constraint, DecisionHierarchy, LinkedDecision, Party, ProblemFrame, ProblemFrameOutput,
};
pub use objectives::{
    FundamentalObjective, MeansObjective, Objectives, ObjectivesOutput, PerformanceMeasure,
};
pub use alternatives::{
    Alternative, Alternatives, AlternativesOutput, DecisionColumn, Strategy, StrategyTable,
};
pub use consequences::{Cell, Consequences, ConsequencesOutput, ConsequencesTable, Uncertainty};
pub use tradeoffs::{
    DominatedAlternative, IrrelevantObjective, Tension, Tradeoffs, TradeoffsOutput,
};
pub use recommendation::{Recommendation, RecommendationOutput};
pub use decision_quality::{
    DecisionQuality, DecisionQualityOutput, DQElement, DQ_ELEMENT_NAMES,
};
pub use notes_next_steps::{NotesNextSteps, NotesNextStepsOutput, PlannedAction};
