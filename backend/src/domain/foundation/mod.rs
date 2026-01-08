//! Foundation module - Shared domain primitives.
//!
//! Contains value objects, identifiers, enums, and error types
//! that form the vocabulary of the Choice Sherpa domain.

mod ids;
mod timestamp;
mod percentage;
mod rating;
mod component_type;
mod component_status;
mod cycle_status;
mod session_status;
mod errors;

pub use ids::{SessionId, CycleId, ComponentId, UserId};
pub use timestamp::Timestamp;
pub use percentage::Percentage;
pub use rating::Rating;
pub use component_type::ComponentType;
pub use component_status::ComponentStatus;
pub use cycle_status::CycleStatus;
pub use session_status::SessionStatus;
pub use errors::{DomainError, ErrorCode, ValidationError};
