//! Application handlers.
//!
//! Command and query handlers that orchestrate domain operations.

pub mod analysis;
pub mod conversation;
pub mod cycle;
pub mod dashboard;
pub mod membership;
pub mod session;

pub use cycle::{
    // Commands
    ArchiveCycleCommand, ArchiveCycleError, ArchiveCycleHandler, ArchiveCycleResult,
    BranchCycleCommand, BranchCycleError, BranchCycleHandler, BranchCycleResult,
    CompleteComponentCommand, CompleteComponentError, CompleteComponentHandler,
    CompleteComponentResult, CompleteCycleCommand, CompleteCycleError, CompleteCycleHandler,
    CompleteCycleResult, NavigateToComponentCommand, NavigateToComponentError, NavigateToComponentHandler,
    NavigateToComponentResult, StartComponentCommand, StartComponentError, StartComponentHandler,
    StartComponentResult,
    UpdateComponentOutputCommand, UpdateComponentOutputError, UpdateComponentOutputHandler,
    UpdateComponentOutputResult,
    // Events
    ComponentCompletedEvent, ComponentOutputUpdatedEvent, ComponentStartedEvent,
    CreateCycleCommand, CreateCycleError, CreateCycleHandler, CreateCycleResult,
    CycleArchivedEvent, CycleBranchedEvent, CycleCompletedEvent, CycleCreatedEvent,
    NavigatedToComponentEvent,
    // Queries
    GetComponentHandler, GetComponentQuery, GetComponentResult,
    GetCycleHandler, GetCycleQuery, GetCycleResult,
    GetCycleTreeHandler, GetCycleTreeQuery, GetCycleTreeResult,
};
pub use dashboard::{
    // Queries
    CompareCyclesHandler, CompareCyclesQuery, CompareCyclesResult,
    GetComponentDetailHandler, GetComponentDetailQuery, GetComponentDetailResult,
    GetDashboardOverviewHandler, GetDashboardOverviewQuery, GetDashboardOverviewResult,
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
pub use analysis::{AnalysisTriggerHandler, ComponentCompletedPayload};
pub use conversation::{
    // Commands
    SendMessageCommand, SendMessageError, SendMessageHandler, SendMessageResult,
    RegenerateResponseCommand, RegenerateResponseError, RegenerateResponseHandler, RegenerateResponseResult,
    // Queries
    GetConversationHandler, GetConversationQuery,
    // Types
    MessageId, MessageRole, StoredMessage, StreamEvent,
    // Ports
    ComponentOwnershipChecker, ConversationRepository, ConversationRepositoryExt, ConversationRecord, OwnershipInfo,
};
