//! Alternatives Tools - Tools for capturing decision options.
//!
//! Alternatives is where users define the options they're considering.
//! Includes support for strategy tables to systematically generate alternatives.

use serde::{Deserialize, Serialize};

use crate::domain::conversation::tools::ToolDefinition;

// ═══════════════════════════════════════════════════════════════════════════
// Tool Parameters
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters for adding an alternative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAlternativeParams {
    /// Name of the alternative
    pub name: String,
    /// Description of the alternative
    pub description: String,
    /// Whether this is the status quo option
    pub is_status_quo: bool,
}

/// Parameters for updating an alternative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAlternativeParams {
    /// ID of the alternative to update
    pub alternative_id: String,
    /// New description
    pub description: String,
}

/// Parameters for removing an alternative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveAlternativeParams {
    /// ID of the alternative to remove
    pub alternative_id: String,
    /// Reason for removal (for audit trail)
    pub reason: String,
}

/// Parameters for adding a strategy dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddStrategyDimensionParams {
    /// Name of the dimension (e.g., "Location", "Timing")
    pub dimension: String,
    /// Available options for this dimension
    pub options: Vec<String>,
}

/// Parameters for setting an alternative's strategy choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetAlternativeStrategyParams {
    /// ID of the alternative
    pub alternative_id: String,
    /// ID of the strategy dimension
    pub dimension_id: String,
    /// Chosen option for this dimension
    pub option: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Results
// ═══════════════════════════════════════════════════════════════════════════

/// Result of adding an alternative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAlternativeResult {
    /// ID of the created alternative
    pub alternative_id: String,
    /// Letter assigned (A, B, C, etc.)
    pub letter: String,
    /// Total number of alternatives
    pub total_alternatives: usize,
    /// Whether a status quo exists
    pub has_status_quo: bool,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of updating an alternative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAlternativeResult {
    /// Whether the update succeeded
    pub success: bool,
    /// Name of the updated alternative
    pub alternative_name: String,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of removing an alternative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveAlternativeResult {
    /// Whether the removal succeeded
    pub success: bool,
    /// Name of the removed alternative
    pub removed_name: String,
    /// Remaining number of alternatives
    pub remaining_alternatives: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of adding a strategy dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddStrategyDimensionResult {
    /// ID of the created dimension
    pub dimension_id: String,
    /// Total number of dimensions
    pub total_dimensions: usize,
    /// Number of possible combinations
    pub possible_combinations: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of setting an alternative's strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetAlternativeStrategyResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// Alternative name
    pub alternative_name: String,
    /// Dimension name
    pub dimension_name: String,
    /// Chosen option
    pub chosen_option: String,
    /// Whether the document was updated
    pub document_updated: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Definitions
// ═══════════════════════════════════════════════════════════════════════════

/// Creates the add_alternative tool definition.
pub fn add_alternative_tool() -> ToolDefinition {
    ToolDefinition::new(
        "add_alternative",
        "Add an alternative (option) to consider. Use when user mentions a possible choice or path.",
        serde_json::json!({
            "type": "object",
            "required": ["name", "description", "is_status_quo"],
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Brief name for the alternative (e.g., 'Accept VP offer', 'Stay current')"
                },
                "description": {
                    "type": "string",
                    "description": "Detailed description of this alternative"
                },
                "is_status_quo": {
                    "type": "boolean",
                    "description": "True if this represents the current state or 'do nothing' option"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "alternative_id": { "type": "string" },
                "letter": { "type": "string" },
                "total_alternatives": { "type": "integer" },
                "has_status_quo": { "type": "boolean" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the update_alternative tool definition.
pub fn update_alternative_tool() -> ToolDefinition {
    ToolDefinition::new(
        "update_alternative",
        "Update the description of an existing alternative.",
        serde_json::json!({
            "type": "object",
            "required": ["alternative_id", "description"],
            "properties": {
                "alternative_id": {
                    "type": "string",
                    "description": "ID of the alternative to update"
                },
                "description": {
                    "type": "string",
                    "description": "New description"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "alternative_name": { "type": "string" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the remove_alternative tool definition.
pub fn remove_alternative_tool() -> ToolDefinition {
    ToolDefinition::new(
        "remove_alternative",
        "Remove an alternative from consideration. Requires reason for audit trail.",
        serde_json::json!({
            "type": "object",
            "required": ["alternative_id", "reason"],
            "properties": {
                "alternative_id": {
                    "type": "string",
                    "description": "ID of the alternative to remove"
                },
                "reason": {
                    "type": "string",
                    "description": "Reason for removal"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "removed_name": { "type": "string" },
                "remaining_alternatives": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the add_strategy_dimension tool definition.
pub fn add_strategy_dimension_tool() -> ToolDefinition {
    ToolDefinition::new(
        "add_strategy_dimension",
        "Add a dimension to the strategy table. Use to systematically generate alternatives by combining options across dimensions.",
        serde_json::json!({
            "type": "object",
            "required": ["dimension", "options"],
            "properties": {
                "dimension": {
                    "type": "string",
                    "description": "Name of the dimension (e.g., 'Location', 'Timing', 'Scope')"
                },
                "options": {
                    "type": "array",
                    "items": { "type": "string" },
                    "minItems": 2,
                    "description": "Available options for this dimension"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "dimension_id": { "type": "string" },
                "total_dimensions": { "type": "integer" },
                "possible_combinations": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the set_alternative_strategy tool definition.
pub fn set_alternative_strategy_tool() -> ToolDefinition {
    ToolDefinition::new(
        "set_alternative_strategy",
        "Set the strategy choice for an alternative on a specific dimension.",
        serde_json::json!({
            "type": "object",
            "required": ["alternative_id", "dimension_id", "option"],
            "properties": {
                "alternative_id": {
                    "type": "string",
                    "description": "ID of the alternative"
                },
                "dimension_id": {
                    "type": "string",
                    "description": "ID of the strategy dimension"
                },
                "option": {
                    "type": "string",
                    "description": "Chosen option for this dimension"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "alternative_name": { "type": "string" },
                "dimension_name": { "type": "string" },
                "chosen_option": { "type": "string" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Returns all Alternatives tool definitions.
pub fn all_alternatives_tools() -> Vec<ToolDefinition> {
    vec![
        add_alternative_tool(),
        update_alternative_tool(),
        remove_alternative_tool(),
        add_strategy_dimension_tool(),
        set_alternative_strategy_tool(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_alternative_params_serializes() {
        let params = AddAlternativeParams {
            name: "Accept VP offer".to_string(),
            description: "Accept the VP position at TechCorp".to_string(),
            is_status_quo: false,
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["is_status_quo"], false);
    }

    #[test]
    fn add_alternative_result_deserializes() {
        let json = serde_json::json!({
            "alternative_id": "alt_a",
            "letter": "A",
            "total_alternatives": 3,
            "has_status_quo": true,
            "document_updated": true
        });
        let result: AddAlternativeResult = serde_json::from_value(json).unwrap();
        assert_eq!(result.letter, "A");
        assert!(result.has_status_quo);
    }

    #[test]
    fn all_alternatives_tools_returns_five_tools() {
        let tools = all_alternatives_tools();
        assert_eq!(tools.len(), 5);
    }

    #[test]
    fn add_strategy_dimension_requires_min_items() {
        let tool = add_strategy_dimension_tool();
        let schema = tool.parameters_schema();
        let options = &schema["properties"]["options"];
        assert_eq!(options["minItems"], 2);
    }
}
