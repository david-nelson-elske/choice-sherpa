//! HTTP adapters - REST API implementations.
//!
//! Each domain module has its own HTTP adapter for endpoint exposure.

pub mod cycle;
pub mod membership;

pub use cycle::{cycle_router, CycleAppState};
pub use membership::*;
