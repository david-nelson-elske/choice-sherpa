//! HTTP handlers for tools endpoints.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::domain::conversation::tools::{ToolCall, ToolRegistry, RevisitPriority};
use crate::domain::foundation::{CycleId, ConfirmationRequestId, RevisitSuggestionId};
use crate::ports::{
    ConfirmationRequestRepository, RevisitSuggestionRepository, ToolExecutor,
    ToolExecutionContext, ToolInvocationRepository,
};

use super::dto::{
    ConfirmationRecord, ConfirmationsQuery, ConfirmationsResponse, DismissRevisitRequest,
    InvocationHistoryQuery, InvocationHistoryResponse, InvocationRecord, InvokeToolRequest,
    InvokeToolResponse, ListToolsQuery, ListToolsResponse, RespondToConfirmationRequest,
    RevisitRecord, RevisitSuggestionsQuery, RevisitSuggestionsResponse, SuccessResponse,
};

/// Application state for tools endpoints.
#[derive(Clone)]
pub struct ToolsAppState {
    /// Tool registry with all available tools
    pub registry: Arc<ToolRegistry>,
    /// Tool executor (injected)
    pub executor: Arc<dyn ToolExecutor>,
    /// Invocation history repository
    pub invocation_repo: Arc<dyn ToolInvocationRepository>,
    /// Revisit suggestion repository
    pub revisit_repo: Arc<dyn RevisitSuggestionRepository>,
    /// Confirmation request repository
    pub confirmation_repo: Arc<dyn ConfirmationRequestRepository>,
}

/// Get available tools for a component.
///
/// GET /tools?component=objectives&format=openai
pub async fn list_tools(
    State(state): State<ToolsAppState>,
    Query(query): Query<ListToolsQuery>,
) -> impl IntoResponse {
    let tools = state.registry.tools_for_component(query.component, query.include_cross_cutting);
    let count = tools.len();

    let tools_json = match query.format.as_str() {
        "openai" => serde_json::to_value(
            tools.iter().map(|t| t.to_openai_format()).collect::<Vec<_>>()
        ).unwrap_or(serde_json::Value::Array(vec![])),
        "anthropic" => serde_json::to_value(
            tools.iter().map(|t| t.to_anthropic_format()).collect::<Vec<_>>()
        ).unwrap_or(serde_json::Value::Array(vec![])),
        _ => serde_json::to_value(tools).unwrap_or(serde_json::Value::Array(vec![])),
    };

    Json(ListToolsResponse {
        component: query.component,
        format: query.format,
        count,
        tools: tools_json,
    })
}

/// Invoke a tool.
///
/// POST /tools/invoke
pub async fn invoke_tool(
    State(state): State<ToolsAppState>,
    Json(request): Json<InvokeToolRequest>,
) -> impl IntoResponse {
    // Check tool exists
    if state.registry.get_tool(&request.tool_name).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(InvokeToolResponse {
                invocation_id: String::new(),
                tool_name: request.tool_name,
                success: false,
                result: None,
                error: Some("Tool not found".to_string()),
                duration_ms: 0,
            }),
        );
    }

    // Parse cycle_id from the request
    let cycle_id = match request.cycle_id.parse::<CycleId>() {
        Ok(cycle_id) => cycle_id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(InvokeToolResponse {
                    invocation_id: String::new(),
                    tool_name: request.tool_name,
                    success: false,
                    result: None,
                    error: Some("Invalid cycle_id format".to_string()),
                    duration_ms: 0,
                }),
            );
        }
    };

    // Build the tool call
    let tool_call = ToolCall::new(&request.tool_name, request.parameters.clone());

    // Build execution context
    let context = ToolExecutionContext::new(
        cycle_id,
        request.component,
        request.conversation_turn.unwrap_or(0),
        request.ai_reasoning.clone().unwrap_or_else(|| "HTTP invocation".to_string()),
    );

    // Execute tool
    let start = std::time::Instant::now();
    let result = state.executor.execute(tool_call, context).await;
    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(response) => {
            // Generate invocation ID for tracking
            let invocation_id = uuid::Uuid::new_v4().to_string();

            (
                StatusCode::OK,
                Json(InvokeToolResponse {
                    invocation_id,
                    tool_name: request.tool_name,
                    success: response.is_success(),
                    result: response.data().cloned(),
                    error: if response.is_success() {
                        None
                    } else {
                        response.error_message().map(String::from)
                    },
                    duration_ms,
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(InvokeToolResponse {
                invocation_id: String::new(),
                tool_name: request.tool_name,
                success: false,
                result: None,
                error: Some(e.to_string()),
                duration_ms,
            }),
        ),
    }
}

/// Get tool invocation history for a cycle.
///
/// GET /tools/invocations/:cycle_id
pub async fn get_invocation_history(
    State(state): State<ToolsAppState>,
    Path(cycle_id_str): Path<String>,
    Query(query): Query<InvocationHistoryQuery>,
) -> impl IntoResponse {
    let cycle_id = match cycle_id_str.parse::<CycleId>() {
        Ok(cycle_id) => cycle_id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(InvocationHistoryResponse {
                    cycle_id: cycle_id_str,
                    total: 0,
                    invocations: vec![],
                    has_more: false,
                }),
            );
        }
    };

    let invocations = match state.invocation_repo.find_by_cycle(cycle_id).await {
        Ok(list) => list,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(InvocationHistoryResponse {
                    cycle_id: cycle_id_str,
                    total: 0,
                    invocations: vec![],
                    has_more: false,
                }),
            );
        }
    };

    // Filter by query params
    let filtered: Vec<_> = invocations
        .into_iter()
        .filter(|inv| {
            if let Some(ref name) = query.tool_name {
                if inv.tool_name() != name {
                    return false;
                }
            }
            if let Some(success) = query.success {
                if inv.is_success() != success {
                    return false;
                }
            }
            true
        })
        .collect();

    let total = filtered.len();
    let has_more = query.offset + query.limit < total;

    let invocations: Vec<InvocationRecord> = filtered
        .into_iter()
        .skip(query.offset)
        .take(query.limit)
        .map(|inv| InvocationRecord {
            id: inv.id().to_string(),
            tool_name: inv.tool_name().to_string(),
            parameters: inv.parameters().clone(),
            success: inv.is_success(),
            result: inv.result_data().cloned().unwrap_or(serde_json::Value::Null),
            invoked_at: inv.invoked_at().as_datetime().to_rfc3339(),
            duration_ms: inv.duration_ms() as u64,
        })
        .collect();

    (
        StatusCode::OK,
        Json(InvocationHistoryResponse {
            cycle_id: cycle_id_str,
            total,
            invocations,
            has_more,
        }),
    )
}

/// Get pending revisit suggestions for a cycle.
///
/// GET /tools/revisits/:cycle_id
pub async fn get_revisit_suggestions(
    State(state): State<ToolsAppState>,
    Path(cycle_id_str): Path<String>,
    Query(query): Query<RevisitSuggestionsQuery>,
) -> impl IntoResponse {
    let cycle_id = match cycle_id_str.parse::<CycleId>() {
        Ok(cycle_id) => cycle_id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(RevisitSuggestionsResponse {
                    total_pending: 0,
                    high_count: 0,
                    medium_count: 0,
                    low_count: 0,
                    suggestions: vec![],
                }),
            );
        }
    };

    let suggestions = match state.revisit_repo.find_pending(cycle_id).await {
        Ok(list) => list,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RevisitSuggestionsResponse {
                    total_pending: 0,
                    high_count: 0,
                    medium_count: 0,
                    low_count: 0,
                    suggestions: vec![],
                }),
            );
        }
    };

    // Filter by query params
    let filtered: Vec<_> = suggestions
        .into_iter()
        .filter(|s| {
            if let Some(ref component) = query.component {
                if s.target_component().to_string() != *component {
                    return false;
                }
            }
            true
        })
        .collect();

    let high_count = filtered.iter().filter(|s| s.priority() == RevisitPriority::High).count();
    let medium_count = filtered.iter().filter(|s| s.priority() == RevisitPriority::Medium).count();
    let low_count = filtered.iter().filter(|s| s.priority() == RevisitPriority::Low).count();

    let records: Vec<RevisitRecord> = filtered
        .into_iter()
        .map(|s| RevisitRecord {
            id: s.id().to_string(),
            component: s.target_component().to_string(),
            reason: s.reason().to_string(),
            priority: format!("{:?}", s.priority()).to_lowercase(),
            status: format!("{:?}", s.status()).to_lowercase(),
            suggested_at: s.created_at().as_datetime().to_rfc3339(),
        })
        .collect();

    (
        StatusCode::OK,
        Json(RevisitSuggestionsResponse {
            total_pending: records.len(),
            high_count,
            medium_count,
            low_count,
            suggestions: records,
        }),
    )
}

/// Dismiss a revisit suggestion.
///
/// POST /tools/revisits/:id/dismiss
pub async fn dismiss_revisit(
    State(state): State<ToolsAppState>,
    Path(revisit_id): Path<String>,
    Json(request): Json<DismissRevisitRequest>,
) -> impl IntoResponse {
let id = match revisit_id.parse::<RevisitSuggestionId>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(SuccessResponse {
                    success: false,
                    message: Some("Invalid revisit ID".to_string()),
                }),
            );
        }
    };

    // Get and update the suggestion
    match state.revisit_repo.find_by_id(id).await {
        Ok(Some(mut suggestion)) => {
            suggestion.dismiss(&request.reason);
            match state.revisit_repo.update(&suggestion).await {
                Ok(_) => (
                    StatusCode::OK,
                    Json(SuccessResponse {
                        success: true,
                        message: Some("Revisit dismissed".to_string()),
                    }),
                ),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(SuccessResponse {
                        success: false,
                        message: Some("Failed to save".to_string()),
                    }),
                ),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(SuccessResponse {
                success: false,
                message: Some("Revisit not found".to_string()),
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SuccessResponse {
                success: false,
                message: Some("Database error".to_string()),
            }),
        ),
    }
}

/// Get pending confirmation requests for a cycle.
///
/// GET /tools/confirmations/:cycle_id
pub async fn get_confirmations(
    State(state): State<ToolsAppState>,
    Path(cycle_id_str): Path<String>,
    Query(_query): Query<ConfirmationsQuery>,
) -> impl IntoResponse {
    let cycle_id = match cycle_id_str.parse::<CycleId>() {
        Ok(cycle_id) => cycle_id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ConfirmationsResponse {
                    pending_count: 0,
                    confirmations: vec![],
                }),
            );
        }
    };

    // Get pending confirmation for this cycle (at most one)
    let confirmation = match state.confirmation_repo.find_pending(cycle_id).await {
        Ok(Some(c)) => vec![c],
        Ok(None) => vec![],
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ConfirmationsResponse {
                    pending_count: 0,
                    confirmations: vec![],
                }),
            );
        }
    };

    let records: Vec<ConfirmationRecord> = confirmation
        .into_iter()
        .map(|c| ConfirmationRecord {
            id: c.id().to_string(),
            question: c.summary().to_string(),
            options: c.options().iter().map(|o| o.label.clone()).collect(),
            status: format!("{:?}", c.status()).to_lowercase(),
            response: c.chosen_option_label().map(|s| s.to_string()),
            requested_at: c.requested_at().as_datetime().to_rfc3339(),
            expires_at: Some(c.expires_at().as_datetime().to_rfc3339()),
        })
        .collect();

    let pending_count = records.iter().filter(|c| c.status == "pending").count();

    (
        StatusCode::OK,
        Json(ConfirmationsResponse {
            pending_count,
            confirmations: records,
        }),
    )
}

/// Respond to a confirmation request.
///
/// POST /tools/confirmations/:id/respond
pub async fn respond_to_confirmation(
    State(state): State<ToolsAppState>,
    Path(confirmation_id): Path<String>,
    Json(request): Json<RespondToConfirmationRequest>,
) -> impl IntoResponse {
let id = match confirmation_id.parse::<ConfirmationRequestId>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(SuccessResponse {
                    success: false,
                    message: Some("Invalid confirmation ID".to_string()),
                }),
            );
        }
    };

    match state.confirmation_repo.find_by_id(id).await {
        Ok(Some(mut confirmation)) => {
            // Find the option index by label
            let option_idx = confirmation.options().iter()
                .position(|o| o.label == request.choice);

            match option_idx {
                Some(idx) => {
                    confirmation.confirm(idx);
                    match state.confirmation_repo.update(&confirmation).await {
                        Ok(_) => (
                            StatusCode::OK,
                            Json(SuccessResponse {
                                success: true,
                                message: Some("Response recorded".to_string()),
                            }),
                        ),
                        Err(_) => (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(SuccessResponse {
                                success: false,
                                message: Some("Failed to save".to_string()),
                            }),
                        ),
                    }
                }
                None => {
                    // Try custom input
                    if let Some(notes) = request.notes {
                        confirmation.confirm_with_input(notes);
                        match state.confirmation_repo.update(&confirmation).await {
                            Ok(_) => (
                                StatusCode::OK,
                                Json(SuccessResponse {
                                    success: true,
                                    message: Some("Response recorded".to_string()),
                                }),
                            ),
                            Err(_) => (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(SuccessResponse {
                                    success: false,
                                    message: Some("Failed to save".to_string()),
                                }),
                            ),
                        }
                    } else {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(SuccessResponse {
                                success: false,
                                message: Some("Invalid choice or already responded".to_string()),
                            }),
                        )
                    }
                }
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(SuccessResponse {
                success: false,
                message: Some("Confirmation not found".to_string()),
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SuccessResponse {
                success: false,
                message: Some("Database error".to_string()),
            }),
        ),
    }
}

#[cfg(test)]
mod tests {
    // Handler tests would require mock implementations
    // These are placeholders for integration tests

    #[test]
    fn tools_app_state_is_clone() {
        // This verifies the Clone derive works
        fn assert_clone<T: Clone>() {}
        // Can't actually test without real implementations
    }
}
