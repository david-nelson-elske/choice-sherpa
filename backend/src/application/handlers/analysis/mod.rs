//! Analysis event handlers.
//!
//! Handlers that respond to domain events and trigger analysis computations.

mod analysis_trigger_handler;

pub use analysis_trigger_handler::{AnalysisTriggerHandler, ComponentCompletedPayload};
