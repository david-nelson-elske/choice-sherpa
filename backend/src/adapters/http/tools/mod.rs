//! Tools HTTP adapter - REST API for atomic decision tools.
//!
//! Provides endpoints for:
//! - Getting available tools by component
//! - Invoking tools
//! - Viewing invocation history
//! - Managing revisit suggestions
//! - Managing confirmation requests

pub mod dto;
pub mod handlers;
pub mod routes;

// Export DTOs for external use
pub use dto::*;

// Export handlers state and router
pub use handlers::ToolsAppState;
pub use routes::tools_router;
