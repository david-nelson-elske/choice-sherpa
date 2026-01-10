//! Data transfer objects for tools HTTP endpoints.

use serde::{Deserialize, Serialize};

use crate::domain::foundation::ComponentType;

// ═══════════════════════════════════════════════════════════════════════════
// Request DTOs
// ═══════════════════════════════════════════════════════════════════════════

/// Request to invoke a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvokeToolRequest {
    /// ID of the cycle (UUID string)
    pub cycle_id: String,
    /// Current component context
    pub component: ComponentType,
    /// Name of the tool to invoke
    pub tool_name: String,
    /// Tool parameters as JSON
    pub parameters: serde_json::Value,
    /// AI's reasoning for this invocation
    pub ai_reasoning: Option<String>,
    /// Current conversation turn
    pub conversation_turn: Option<u32>,
}

/// Request to dismiss a revisit suggestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DismissRevisitRequest {
    /// Reason for dismissal
    pub reason: String,
}

/// Request to respond to a confirmation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespondToConfirmationRequest {
    /// User's choice (option label)
    pub choice: String,
    /// Optional custom input
    pub notes: Option<String>,
}

/// Query parameters for listing tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsQuery {
    /// Component type to get tools for
    pub component: ComponentType,
    /// Output format: "openai" or "anthropic" or "native"
    #[serde(default = "default_format")]
    pub format: String,
    /// Whether to include cross-cutting tools
    #[serde(default = "default_true")]
    pub include_cross_cutting: bool,
}

fn default_format() -> String {
    "native".to_string()
}

fn default_true() -> bool {
    true
}

/// Query parameters for invocation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationHistoryQuery {
    /// Maximum number of results
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Offset for pagination
    #[serde(default)]
    pub offset: usize,
    /// Filter by tool name
    pub tool_name: Option<String>,
    /// Filter by success status
    pub success: Option<bool>,
}

fn default_limit() -> usize {
    50
}

/// Query parameters for revisit suggestions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisitSuggestionsQuery {
    /// Filter by component
    pub component: Option<String>,
    /// Filter by minimum priority (high, medium, low)
    pub min_priority: Option<String>,
}

/// Query parameters for confirmation requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationsQuery {
    /// Only pending confirmations
    #[serde(default = "default_true")]
    pub pending_only: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Response DTOs
// ═══════════════════════════════════════════════════════════════════════════

/// Response with available tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResponse {
    /// Component tools were retrieved for
    pub component: ComponentType,
    /// Format used
    pub format: String,
    /// Number of tools
    pub count: usize,
    /// Tool definitions (format depends on `format` field)
    pub tools: serde_json::Value,
}

/// Response from invoking a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvokeToolResponse {
    /// Invocation ID for tracking
    pub invocation_id: String,
    /// Tool that was invoked
    pub tool_name: String,
    /// Whether invocation succeeded
    pub success: bool,
    /// Result data (if successful)
    pub result: Option<serde_json::Value>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
}

/// A tool invocation record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationRecord {
    /// Invocation ID
    pub id: String,
    /// Tool name
    pub tool_name: String,
    /// Parameters used
    pub parameters: serde_json::Value,
    /// Whether it succeeded
    pub success: bool,
    /// Result or error
    pub result: serde_json::Value,
    /// When invoked
    pub invoked_at: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Response with invocation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationHistoryResponse {
    /// Cycle ID
    pub cycle_id: String,
    /// Total invocations matching filter
    pub total: usize,
    /// Invocations returned
    pub invocations: Vec<InvocationRecord>,
    /// Whether there are more results
    pub has_more: bool,
}

/// A revisit suggestion record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisitRecord {
    /// Suggestion ID
    pub id: String,
    /// Component to revisit
    pub component: String,
    /// Reason for revisit
    pub reason: String,
    /// Priority level
    pub priority: String,
    /// Status
    pub status: String,
    /// When suggested
    pub suggested_at: String,
}

/// Response with revisit suggestions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisitSuggestionsResponse {
    /// Total pending
    pub total_pending: usize,
    /// By priority
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    /// Suggestions
    pub suggestions: Vec<RevisitRecord>,
}

/// A confirmation request record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationRecord {
    /// Confirmation ID
    pub id: String,
    /// Question being asked
    pub question: String,
    /// Available options
    pub options: Vec<String>,
    /// Status
    pub status: String,
    /// User's response (if answered)
    pub response: Option<String>,
    /// When requested
    pub requested_at: String,
    /// When expires
    pub expires_at: Option<String>,
}

/// Response with confirmation requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationsResponse {
    /// Total pending
    pub pending_count: usize,
    /// Confirmations
    pub confirmations: Vec<ConfirmationRecord>,
}

/// Generic success response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessResponse {
    /// Whether operation succeeded
    pub success: bool,
    /// Optional message
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invoke_tool_request_deserializes() {
        let json = r#"{
            "cycle_id": "550e8400-e29b-41d4-a716-446655440000",
            "component": "objectives",
            "tool_name": "add_objective",
            "parameters": {"name": "Maximize income"}
        }"#;
        let req: InvokeToolRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tool_name, "add_objective");
        assert!(req.ai_reasoning.is_none());
    }

    #[test]
    fn list_tools_query_has_defaults() {
        let json = r#"{"component": "objectives"}"#;
        let query: ListToolsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.format, "native");
        assert!(query.include_cross_cutting);
    }

    #[test]
    fn invoke_tool_response_serializes() {
        let resp = InvokeToolResponse {
            invocation_id: "inv_123".to_string(),
            tool_name: "add_objective".to_string(),
            success: true,
            result: Some(serde_json::json!({"objective_id": "obj_1"})),
            error: None,
            duration_ms: 42,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("inv_123"));
        assert!(json.contains("obj_1"));
    }
}
