//! Foundation module - Shared domain primitives.
//!
//! Contains value objects, identifiers, enums, and error types
//! that form the vocabulary of the Choice Sherpa domain.

mod auth;
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
mod upcaster;
mod command;

pub use auth::{AuthenticatedUser, AuthError};
pub use ids::{
    SessionId, CycleId, ComponentId, ConversationId, UserId, MembershipId,
    ToolInvocationId, RevisitSuggestionId, ConfirmationRequestId,
};
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
pub use upcaster::{Upcaster, UpcasterRegistry, UpcastError, EventDeserializer, DeserializeError, EventReplayer, ReplayStats};
pub use command::CommandMetadata;
