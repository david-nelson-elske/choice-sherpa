//! Application layer - Commands, Queries, and Handlers.
//!
//! This layer orchestrates domain operations and coordinates between ports.
//! Following CQRS, it separates command handlers (write) from query handlers (read).

pub mod handlers;

pub use handlers::{
    // Session handlers
    ArchiveSessionCommand, ArchiveSessionHandler, ArchiveSessionResult,
    CreateSessionCommand, CreateSessionHandler, CreateSessionResult,
    CycleCreated, SessionCycleTracker,
    RenameSessionCommand, RenameSessionHandler, RenameSessionResult,
};
