//! HTTP adapters - REST API implementations.
//!
//! Each domain module has its own HTTP adapter for endpoint exposure.

pub mod cycle;
pub mod membership;

// Re-export key types for convenience
pub use cycle::CycleAppState;
pub use membership::MembershipAppState;
