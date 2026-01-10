//! Tool call and response types.
//!
//! These are the request/response value objects for tool execution.

use serde::{Deserialize, Serialize};

/// A request to invoke a tool.
///
/// Represents the agent's intent to call a specific tool with parameters.
/// Parameters are passed as JSON to support the varying schemas of
/// different tools.
///
/// # Examples
///
/// ```ignore
/// use choice_sherpa::domain::conversation::tools::ToolCall;
///
/// let call = ToolCall::new(
///     "add_objective",
///     serde_json::json!({
///         "name": "Minimize cost",
///         "measure": "Total USD spent",
///         "direction": "lower",
///         "is_fundamental": true
///     }),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Name of the tool to invoke
    name: String,

    /// Parameters for the tool (JSON object)
    parameters: serde_json::Value,
}

impl ToolCall {
    /// Creates a new tool call.
    pub fn new(name: impl Into<String>, parameters: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            parameters,
        }
    }

    /// Returns the tool name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the parameters.
    pub fn parameters(&self) -> &serde_json::Value {
        &self.parameters
    }

    /// Consumes self and returns the parameters.
    pub fn into_parameters(self) -> serde_json::Value {
        self.parameters
    }
}

/// Response from executing a tool.
///
/// Contains the result of tool execution, including success/failure,
/// return data, and any suggestions the tool wants to surface.
///
/// # Examples
///
/// ```ignore
/// use choice_sherpa::domain::conversation::tools::ToolResponse;
///
/// // Success response
/// let response = ToolResponse::success(
///     serde_json::json!({ "objective_id": "obj-123" }),
///     true, // document was updated
/// );
///
/// // Error response
/// let response = ToolResponse::error("Objective not found".to_string());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResponse {
    /// Whether the tool executed successfully
    success: bool,

    /// Data returned by the tool (if successful)
    data: Option<serde_json::Value>,

    /// Error message (if failed)
    error: Option<String>,

    /// Whether the decision document was updated
    document_updated: bool,

    /// Suggestions surfaced by the tool (e.g., "Consider adding more alternatives")
    suggestions: Vec<String>,
}

impl ToolResponse {
    /// Creates a successful response with data.
    pub fn success(data: serde_json::Value, document_updated: bool) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            document_updated,
            suggestions: Vec::new(),
        }
    }

    /// Creates a successful response without data.
    pub fn success_empty(document_updated: bool) -> Self {
        Self {
            success: true,
            data: None,
            error: None,
            document_updated,
            suggestions: Vec::new(),
        }
    }

    /// Creates an error response.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
            document_updated: false,
            suggestions: Vec::new(),
        }
    }

    /// Adds a suggestion to the response.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    /// Adds multiple suggestions to the response.
    pub fn with_suggestions(mut self, suggestions: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.suggestions.extend(suggestions.into_iter().map(Into::into));
        self
    }

    /// Returns whether the tool succeeded.
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns the response data (if any).
    pub fn data(&self) -> Option<&serde_json::Value> {
        self.data.as_ref()
    }

    /// Returns the error message (if any).
    pub fn error_message(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Returns whether the document was updated.
    pub fn document_updated(&self) -> bool {
        self.document_updated
    }

    /// Returns the suggestions.
    pub fn suggestions(&self) -> &[String] {
        &self.suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_call_new_creates_with_params() {
        let call = ToolCall::new("add_objective", serde_json::json!({"name": "Test"}));

        assert_eq!(call.name(), "add_objective");
        assert_eq!(call.parameters()["name"], "Test");
    }

    #[test]
    fn tool_call_serializes_to_json() {
        let call = ToolCall::new("set_focal_statement", serde_json::json!({"statement": "Should we expand?"}));

        let json = serde_json::to_string(&call).unwrap();
        assert!(json.contains("set_focal_statement"));
        assert!(json.contains("Should we expand?"));
    }

    #[test]
    fn tool_call_into_parameters_consumes() {
        let call = ToolCall::new("test", serde_json::json!({"key": "value"}));
        let params = call.into_parameters();

        assert_eq!(params["key"], "value");
    }

    #[test]
    fn tool_response_success_creates_success() {
        let response = ToolResponse::success(serde_json::json!({"id": "123"}), true);

        assert!(response.is_success());
        assert!(response.document_updated());
        assert!(response.data().is_some());
        assert!(response.error_message().is_none());
    }

    #[test]
    fn tool_response_error_creates_error() {
        let response = ToolResponse::error("Not found");

        assert!(!response.is_success());
        assert!(!response.document_updated());
        assert!(response.data().is_none());
        assert_eq!(response.error_message(), Some("Not found"));
    }

    #[test]
    fn tool_response_with_suggestion_adds_suggestion() {
        let response = ToolResponse::success_empty(false)
            .with_suggestion("Consider adding more alternatives");

        assert_eq!(response.suggestions().len(), 1);
        assert_eq!(response.suggestions()[0], "Consider adding more alternatives");
    }

    #[test]
    fn tool_response_with_suggestions_adds_multiple() {
        let response = ToolResponse::success_empty(false)
            .with_suggestions(["First", "Second"]);

        assert_eq!(response.suggestions().len(), 2);
    }

    #[test]
    fn tool_response_serializes_to_json() {
        let response = ToolResponse::success(serde_json::json!({"ok": true}), true)
            .with_suggestion("Tip");

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("document_updated"));
        assert!(json.contains("Tip"));
    }
}
