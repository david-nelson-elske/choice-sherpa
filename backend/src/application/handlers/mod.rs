//! Application handlers.
//!
//! Command and query handlers that orchestrate domain operations.

pub mod cycle;
pub mod membership;
pub mod session;

pub use cycle::{
    // Commands
    BranchCycleCommand, BranchCycleError, BranchCycleHandler, BranchCycleResult, CycleBranchedEvent,
    CreateCycleCommand, CreateCycleError, CreateCycleHandler, CreateCycleResult, CycleCreatedEvent,
    GenerateDocumentCommand, GenerateDocumentError, GenerateDocumentHandler, GenerateDocumentResult,
};
pub use membership::{
    // Commands
    CancelMembershipCommand, CancelMembershipHandler, CancelMembershipResult,
    CreateFreeMembershipCommand, CreateFreeMembershipHandler, CreateFreeMembershipResult,
    CreatePaidMembershipCommand, CreatePaidMembershipHandler, CreatePaidMembershipResult,
    HandlePaymentWebhookCommand, HandlePaymentWebhookHandler, HandlePaymentWebhookResult,
    // Queries
    CheckAccessHandler, CheckAccessQuery, CheckAccessResult,
    GetMembershipHandler, GetMembershipQuery, GetMembershipResult,
    GetMembershipStatsHandler, GetMembershipStatsQuery, GetMembershipStatsResult,
};
pub use session::{
    ArchiveSessionCommand, ArchiveSessionHandler, ArchiveSessionResult,
    CreateSessionCommand, CreateSessionHandler, CreateSessionResult,
    CycleCreated, SessionCycleTracker,
    RenameSessionCommand, RenameSessionHandler, RenameSessionResult,
};
