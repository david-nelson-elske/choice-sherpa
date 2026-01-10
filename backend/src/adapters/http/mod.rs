//! HTTP adapters - REST API implementations.
//!
//! Each domain module has its own HTTP adapter for endpoint exposure.

pub mod membership;
pub mod tools;

// Re-export membership types with explicit names to avoid conflicts
pub use membership::MembershipAppState;
pub use membership::membership_router;

// Re-export tools types with explicit names
pub use tools::ToolsAppState;
pub use tools::tools_router;
