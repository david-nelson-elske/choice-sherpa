//! Tool definition - schema and metadata for a tool.
//!
//! Defines the interface for a tool that the AI agent can invoke.

use serde::{Deserialize, Serialize};

/// Definition of a tool that can be invoked by the AI agent.
///
/// Contains the schema and documentation needed for:
/// - AI providers (OpenAI/Anthropic tool calling)
/// - Parameter validation before execution
/// - API documentation generation
///
/// # Examples
///
/// ```ignore
/// use choice_sherpa::domain::conversation::tools::ToolDefinition;
///
/// let definition = ToolDefinition::new(
///     "add_objective",
///     "Add a new objective to the decision document",
///     serde_json::json!({
///         "type": "object",
///         "required": ["name", "measure", "direction"],
///         "properties": {
///             "name": { "type": "string", "description": "Name of the objective" },
///             "measure": { "type": "string", "description": "How this objective is measured" },
///             "direction": { "type": "string", "enum": ["higher", "lower", "target"] },
///             "is_fundamental": { "type": "boolean", "default": false }
///         }
///     }),
///     serde_json::json!({
///         "type": "object",
///         "properties": {
///             "objective_id": { "type": "string" },
///             "objective_type": { "type": "string" },
///             "total_fundamental": { "type": "integer" },
///             "total_means": { "type": "integer" }
///         }
///     }),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Unique name of the tool (e.g., "add_objective")
    name: String,

    /// Human-readable description for AI and docs
    description: String,

    /// JSON Schema for the parameters
    parameters_schema: serde_json::Value,

    /// JSON Schema for the return value
    returns_schema: serde_json::Value,
}

impl ToolDefinition {
    /// Creates a new tool definition.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters_schema: serde_json::Value,
        returns_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters_schema,
            returns_schema,
        }
    }

    /// Creates a tool definition with no return value.
    pub fn void(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters_schema,
            returns_schema: serde_json::json!({"type": "null"}),
        }
    }

    /// Returns the tool name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the parameters schema.
    pub fn parameters_schema(&self) -> &serde_json::Value {
        &self.parameters_schema
    }

    /// Returns the returns schema.
    pub fn returns_schema(&self) -> &serde_json::Value {
        &self.returns_schema
    }

    /// Converts to OpenAI tool format.
    ///
    /// OpenAI expects a specific structure for function calling.
    pub fn to_openai_format(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": self.parameters_schema
            }
        })
    }

    /// Converts to Anthropic tool format.
    ///
    /// Anthropic expects a slightly different structure.
    pub fn to_anthropic_format(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "description": self.description,
            "input_schema": self.parameters_schema
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_params_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "required": ["name"],
            "properties": {
                "name": { "type": "string" }
            }
        })
    }

    fn sample_returns_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": { "type": "string" }
            }
        })
    }

    #[test]
    fn new_creates_definition() {
        let def = ToolDefinition::new(
            "add_objective",
            "Add objective",
            sample_params_schema(),
            sample_returns_schema(),
        );

        assert_eq!(def.name(), "add_objective");
        assert_eq!(def.description(), "Add objective");
    }

    #[test]
    fn void_creates_null_returns() {
        let def = ToolDefinition::void(
            "set_focal_statement",
            "Set the focal statement",
            sample_params_schema(),
        );

        assert_eq!(def.returns_schema()["type"], "null");
    }

    #[test]
    fn to_openai_format_has_correct_structure() {
        let def = ToolDefinition::new(
            "add_objective",
            "Add objective",
            sample_params_schema(),
            sample_returns_schema(),
        );

        let openai = def.to_openai_format();

        assert_eq!(openai["type"], "function");
        assert_eq!(openai["function"]["name"], "add_objective");
        assert_eq!(openai["function"]["description"], "Add objective");
        assert!(openai["function"]["parameters"].is_object());
    }

    #[test]
    fn to_anthropic_format_has_correct_structure() {
        let def = ToolDefinition::new(
            "add_objective",
            "Add objective",
            sample_params_schema(),
            sample_returns_schema(),
        );

        let anthropic = def.to_anthropic_format();

        assert_eq!(anthropic["name"], "add_objective");
        assert_eq!(anthropic["description"], "Add objective");
        assert!(anthropic["input_schema"].is_object());
    }

    #[test]
    fn serializes_to_json() {
        let def = ToolDefinition::new(
            "test_tool",
            "Test description",
            serde_json::json!({}),
            serde_json::json!({}),
        );

        let json = serde_json::to_string(&def).unwrap();
        assert!(json.contains("test_tool"));
        assert!(json.contains("Test description"));
    }

    #[test]
    fn deserializes_from_json() {
        let json = r#"{
            "name": "my_tool",
            "description": "My tool",
            "parameters_schema": {},
            "returns_schema": {}
        }"#;

        let def: ToolDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.name(), "my_tool");
    }
}
