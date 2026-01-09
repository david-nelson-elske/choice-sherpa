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
mod state_machine;
mod errors;
mod events;
mod command;

pub use ids::{SessionId, CycleId, ComponentId, ConversationId, UserId, MembershipId};
pub use timestamp::Timestamp;
pub use percentage::Percentage;
pub use rating::Rating;
pub use component_type::ComponentType;
pub use component_status::ComponentStatus;
pub use cycle_status::CycleStatus;
pub use session_status::SessionStatus;
pub use state_machine::StateMachine;
pub use errors::{DomainError, ErrorCode, ValidationError};
pub use events::{DomainEvent, SerializableDomainEvent, EventId, EventMetadata, EventEnvelope, domain_event};
pub use command::CommandMetadata;
