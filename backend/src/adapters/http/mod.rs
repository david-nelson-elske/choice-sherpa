//! HTTP adapters - REST API implementations.
//!
//! Each domain module has its own HTTP adapter for endpoint exposure.

pub mod membership;
pub mod tools;

// Re-export key types for convenience
pub use membership::MembershipAppState;
pub use membership::membership_router;
pub use tools::ToolsAppState;
pub use tools::tools_router;
