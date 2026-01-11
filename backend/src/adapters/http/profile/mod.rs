//! HTTP adapter for profile endpoints.

mod dto;
mod handlers;
mod routes;

pub use dto::{
    AgentInstructionsResponse, AnalysisResultResponse, CreateProfileRequest,
    DeleteProfileRequest, ErrorResponse, ProfileCommandResponse, ProfileSummaryResponse,
    RecordOutcomeRequest, UpdateConsentRequest, UpdateProfileFromDecisionRequest,
};
pub use handlers::ProfileHandlers;
pub use routes::profile_routes;
