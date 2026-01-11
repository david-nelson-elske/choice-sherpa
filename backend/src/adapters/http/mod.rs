//! HTTP adapters - REST API implementations.
//!
//! Each domain module has its own HTTP adapter for endpoint exposure.
//!
//! ## Middleware
//!
//! - `middleware::auth` - Authentication middleware and extractors
//! - `middleware::rate_limit` - Rate limiting middleware

pub mod ai_engine;
pub mod conversation;
pub mod cycle;
pub mod dashboard;
pub mod membership;
pub mod middleware;
pub mod session;
pub mod tools;

// Re-export key types for convenience
pub use ai_engine::AIEngineAppState;
pub use conversation::conversation_routes;
pub use conversation::ConversationAppState;
pub use cycle::CycleAppState;
pub use dashboard::dashboard_routes;
pub use dashboard::DashboardAppState;
pub use membership::MembershipAppState;
pub use membership::membership_router;
pub use middleware::{auth_middleware, AuthRejection, AuthState, OptionalAuth, RequireAuth};
pub use middleware::{
    rate_limit_middleware, RateLimitCheck, RateLimitRejection, RateLimiterState,
};
pub use session::session_routes;
pub use session::SessionHandlers;
pub use tools::ToolsAppState;
pub use tools::tools_router;
