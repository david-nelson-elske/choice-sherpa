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

pub mod analysis;
pub mod cycle;
pub mod foundation;
pub mod membership;
pub mod proact;
pub mod session;
