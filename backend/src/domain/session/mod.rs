//! Session domain module.
//!
//! Handles decision session lifecycle including creation, modification,
//! and archival. Sessions are the top-level containers for decision contexts.
//!
//! # Aggregate
//!
//! - `Session` - The session aggregate entity
//!
//! # Events
//!
//! - `SessionCreated` - Published when a new session is created
//! - `SessionRenamed` - Published when a session's title changes
//! - `SessionDescriptionUpdated` - Published when description changes
//! - `SessionArchived` - Published when a session is archived
//! - `CycleAddedToSession` - Published when a cycle is linked to the session

mod aggregate;
mod events;

pub use aggregate::{Session, MAX_TITLE_LENGTH};
pub use events::{
    CycleAddedToSession, SessionArchived, SessionCreated, SessionDescriptionUpdated,
    SessionRenamed,
};
