//! User/Profile application handlers.
//!
//! Command and query handlers for decision profile management.

mod create_profile;
mod delete_profile;
mod get_agent_instructions;
mod get_profile_summary;
mod record_outcome;
mod update_profile_from_decision;

pub use create_profile::{CreateProfileCommand, CreateProfileHandler, CreateProfileResult};
pub use delete_profile::{DeleteProfileCommand, DeleteProfileHandler, DeleteProfileResult};
pub use get_agent_instructions::{GetAgentInstructionsHandler, GetAgentInstructionsQuery};
pub use get_profile_summary::{GetProfileSummaryHandler, GetProfileSummaryQuery};
pub use record_outcome::{RecordOutcomeCommand, RecordOutcomeHandler, RecordOutcomeResult};
pub use update_profile_from_decision::{
    UpdateProfileFromDecisionCommand, UpdateProfileFromDecisionHandler,
    UpdateProfileFromDecisionResult,
};
