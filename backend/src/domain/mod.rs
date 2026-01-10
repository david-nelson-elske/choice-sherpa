//! Domain layer containing business logic and domain types.
//!
//! # Module Organization
//!
//! - `foundation` - Shared domain primitives (value objects, IDs, enums, errors)
//! - `membership` - Subscription lifecycle and access control
//! - `proact` - PrOACT component types and traits
//! - `session` - Decision session lifecycle and events
//! - `cycle` - Decision cycle aggregate and lifecycle management
//! - `analysis` - Pure domain services for decision analysis (Pugh, DQ, tradeoffs)
//! - `conversation` - AI-guided dialogues within PrOACT components
//! - `ai_engine` - AI conversation orchestration and PrOACT flow management

pub mod ai_engine;
pub mod analysis;
pub mod conversation;
pub mod cycle;
pub mod foundation;
pub mod membership;
pub mod proact;
pub mod session;
