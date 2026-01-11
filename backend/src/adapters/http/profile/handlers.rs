//! HTTP handlers for profile endpoints.

use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::adapters::http::middleware::RequireAuth;
use crate::application::handlers::user::{
    CreateProfileCommand, CreateProfileHandler, DeleteProfileCommand, DeleteProfileHandler,
    GetAgentInstructionsHandler, GetAgentInstructionsQuery, GetProfileSummaryHandler,
    GetProfileSummaryQuery, RecordOutcomeCommand, RecordOutcomeHandler,
    UpdateProfileFromDecisionCommand, UpdateProfileFromDecisionHandler,
};
use crate::domain::foundation::{CommandMetadata, CycleId, DomainError, ErrorCode, Timestamp};
use crate::domain::user::ProfileConsent;
use crate::ports::DecisionAnalysisData;

use super::dto::{
    AgentInstructionsResponse, AnalysisResultResponse, CreateProfileRequest,
    DeleteProfileRequest, ErrorResponse, ProfileCommandResponse, ProfileSummaryResponse,
    RecordOutcomeRequest, UpdateConsentRequest, UpdateProfileFromDecisionRequest,
};

// ════════════════════════════════════════════════════════════════════════════
// Handler state
// ════════════════════════════════════════════════════════════════════════════

#[derive(Clone)]
pub struct ProfileHandlers {
    create_handler: Arc<CreateProfileHandler>,
    delete_handler: Arc<DeleteProfileHandler>,
    get_summary_handler: Arc<GetProfileSummaryHandler>,
    get_instructions_handler: Arc<GetAgentInstructionsHandler>,
    record_outcome_handler: Arc<RecordOutcomeHandler>,
    update_from_decision_handler: Arc<UpdateProfileFromDecisionHandler>,
}

impl ProfileHandlers {
    pub fn new(
        create_handler: Arc<CreateProfileHandler>,
        delete_handler: Arc<DeleteProfileHandler>,
        get_summary_handler: Arc<GetProfileSummaryHandler>,
        get_instructions_handler: Arc<GetAgentInstructionsHandler>,
        record_outcome_handler: Arc<RecordOutcomeHandler>,
        update_from_decision_handler: Arc<UpdateProfileFromDecisionHandler>,
    ) -> Self {
        Self {
            create_handler,
            delete_handler,
            get_summary_handler,
            get_instructions_handler,
            record_outcome_handler,
            update_from_decision_handler,
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════
// HTTP handlers
// ════════════════════════════════════════════════════════════════════════════

/// POST /api/profile - Create a new profile
pub async fn create_profile(
    State(handlers): State<ProfileHandlers>,
    RequireAuth(user): RequireAuth,
    Json(req): Json<CreateProfileRequest>,
) -> Response {
    let now = Timestamp::now();
    let consent = ProfileConsent {
        collection_enabled: req.collection_enabled,
        analysis_enabled: req.analysis_enabled,
        agent_access_enabled: req.agent_access_enabled,
        consented_at: now,
        last_reviewed: now,
    };

    let cmd = CreateProfileCommand {
        user_id: user.id.clone(),
        consent,
    };

    let metadata = CommandMetadata::new(user.id).with_correlation_id("http-request");

    match handlers.create_handler.handle(cmd, metadata).await {
        Ok(result) => {
            let response = ProfileCommandResponse {
                profile_id: Some(result.profile_id.to_string()),
                message: "Profile created successfully".to_string(),
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => handle_profile_error(e),
    }
}

/// GET /api/profile - Get profile summary
pub async fn get_profile_summary(
    State(handlers): State<ProfileHandlers>,
    RequireAuth(user): RequireAuth,
) -> Response {
    let query = GetProfileSummaryQuery {
        user_id: user.id.clone(),
    };

    match handlers.get_summary_handler.handle(query).await {
        Ok(Some(summary)) => {
            let response: ProfileSummaryResponse = summary.into();
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::not_found("Profile", &user.id.to_string())),
        )
            .into_response(),
        Err(e) => handle_profile_error(e),
    }
}

/// GET /api/profile/instructions - Get agent instructions
pub async fn get_agent_instructions(
    State(handlers): State<ProfileHandlers>,
    RequireAuth(user): RequireAuth,
) -> Response {
    let query = GetAgentInstructionsQuery {
        user_id: user.id.clone(),
        domain: None, // Could be extracted from query params if needed
    };

    match handlers.get_instructions_handler.handle(query).await {
        Ok(Some(instructions)) => {
            let response: AgentInstructionsResponse = instructions.into();
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => {
            // Return default instructions when profile doesn't exist
            let default_instructions = AgentInstructionsResponse {
                risk_guidance: "No profile data yet. Approach with balanced questioning."
                    .to_string(),
                blind_spot_prompts: vec![],
                communication_adjustments: vec![],
                suggested_questions: vec![],
            };
            (StatusCode::OK, Json(default_instructions)).into_response()
        }
        Err(e) => handle_profile_error(e),
    }
}

/// PUT /api/profile/consent - Update consent settings
pub async fn update_consent(
    State(handlers): State<ProfileHandlers>,
    RequireAuth(user): RequireAuth,
    Json(req): Json<UpdateConsentRequest>,
) -> Response {
    // Note: This endpoint is a placeholder. The actual consent update
    // would require a dedicated command handler that we haven't implemented yet.
    // For now, return a 501 Not Implemented status.
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            code: "NOT_IMPLEMENTED".to_string(),
            message: "Consent update not yet implemented".to_string(),
            details: None,
        }),
    )
        .into_response()
}

/// POST /api/profile/outcome - Record decision outcome
pub async fn record_outcome(
    State(handlers): State<ProfileHandlers>,
    RequireAuth(user): RequireAuth,
    Json(req): Json<RecordOutcomeRequest>,
) -> Response {
    let cycle_id = match req.cycle_id.parse::<CycleId>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::bad_request("Invalid cycle ID")),
            )
                .into_response()
        }
    };

    let cmd = RecordOutcomeCommand {
        user_id: user.id.clone(),
        cycle_id,
        satisfaction: req.satisfaction,
        actual_consequences: req.actual_consequences,
        surprises: req.surprises,
        would_decide_same: req.would_decide_same,
    };

    let metadata = CommandMetadata::new(user.id).with_correlation_id("http-request");

    match handlers.record_outcome_handler.handle(cmd, metadata).await {
        Ok(_) => {
            let response = ProfileCommandResponse {
                profile_id: None,
                message: "Outcome recorded successfully".to_string(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => handle_profile_error(e),
    }
}

/// POST /api/profile/update-from-decision - Update profile from decision
pub async fn update_from_decision(
    State(handlers): State<ProfileHandlers>,
    RequireAuth(user): RequireAuth,
    Json(req): Json<UpdateProfileFromDecisionRequest>,
) -> Response {
    let analysis_data = DecisionAnalysisData {
        title: req.title,
        domain: req.domain,
        dq_score: req.dq_score,
        key_tradeoff: req.key_tradeoff,
        chosen_alternative: req.chosen_alternative,
        objectives: req.objectives,
        alternatives: req.alternatives,
        conversations: vec![], // Empty for now, would be populated from conversation history
        risk_indicators: vec![], // Empty for now, would be detected from conversations
    };

    let cmd = UpdateProfileFromDecisionCommand {
        user_id: user.id.clone(),
        analysis_data,
    };

    let metadata = CommandMetadata::new(user.id).with_correlation_id("http-request");

    match handlers
        .update_from_decision_handler
        .handle(cmd, metadata)
        .await
    {
        Ok(result) => {
            let response: AnalysisResultResponse = result.analysis_result.into();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => handle_profile_error(e),
    }
}

/// DELETE /api/profile - Delete profile
pub async fn delete_profile(
    State(handlers): State<ProfileHandlers>,
    RequireAuth(user): RequireAuth,
    Json(req): Json<DeleteProfileRequest>,
) -> Response {
    let cmd = DeleteProfileCommand {
        user_id: user.id.clone(),
        confirmation: req.confirmation,
    };

    let metadata = CommandMetadata::new(user.id).with_correlation_id("http-request");

    match handlers.delete_handler.handle(cmd, metadata).await {
        Ok(result) => {
            let response = ProfileCommandResponse {
                profile_id: Some(result.deleted_profile_id.to_string()),
                message: "Profile deleted successfully".to_string(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => handle_profile_error(e),
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Error handling
// ════════════════════════════════════════════════════════════════════════════

fn handle_profile_error(error: DomainError) -> Response {
    match error.code() {
        ErrorCode::NotFound => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::not_found("Profile", "unknown")),
        )
            .into_response(),
        ErrorCode::Forbidden => (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::forbidden(error.message())),
        )
            .into_response(),
        ErrorCode::ValidationFailed => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::bad_request(error.message())),
        )
            .into_response(),
        ErrorCode::InternalError => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::internal(error.message())),
        )
            .into_response(),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::internal("An unexpected error occurred")),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_error_not_found_maps_to_404() {
        let error = DomainError::new(ErrorCode::NotFound, "Profile not found");
        let response = handle_profile_error(error);
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn profile_error_forbidden_maps_to_403() {
        let error = DomainError::new(ErrorCode::Forbidden, "Access denied");
        let response = handle_profile_error(error);
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn profile_error_validation_failed_maps_to_400() {
        let error = DomainError::validation("field", "Invalid value");
        let response = handle_profile_error(error);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
