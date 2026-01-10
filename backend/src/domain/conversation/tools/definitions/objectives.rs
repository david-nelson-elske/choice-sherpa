//! Objectives Tools - Tools for identifying and organizing objectives.
//!
//! Objectives is where users identify what they want to achieve. Objectives
//! can be fundamental (what really matters) or means (ways to achieve
//! fundamental objectives).

use serde::{Deserialize, Serialize};

use crate::domain::conversation::tools::ToolDefinition;

// ═══════════════════════════════════════════════════════════════════════════
// Enums
// ═══════════════════════════════════════════════════════════════════════════

/// Direction for measuring an objective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveDirection {
    /// Higher is better (maximize)
    Higher,
    /// Lower is better (minimize)
    Lower,
    /// Specific target value
    Target,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Parameters
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters for adding an objective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddObjectiveParams {
    /// Name of the objective
    pub name: String,
    /// How this objective is measured
    pub measure: String,
    /// Direction for optimization
    pub direction: ObjectiveDirection,
    /// Whether this is a fundamental objective
    pub is_fundamental: bool,
}

/// Parameters for linking means to fundamental objective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkMeansToFundamentalParams {
    /// ID of the means objective
    pub means_id: String,
    /// ID of the fundamental objective it supports
    pub fundamental_id: String,
}

/// Parameters for updating an objective's measure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateObjectiveMeasureParams {
    /// ID of the objective to update
    pub objective_id: String,
    /// New measure for the objective
    pub new_measure: String,
}

/// Parameters for removing an objective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveObjectiveParams {
    /// ID of the objective to remove
    pub objective_id: String,
    /// Reason for removal (for audit trail)
    pub reason: String,
}

/// Parameters for promoting a means objective to fundamental.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromoteToFundamentalParams {
    /// ID of the means objective to promote
    pub objective_id: String,
    /// Reason for promotion
    pub reason: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Results
// ═══════════════════════════════════════════════════════════════════════════

/// Result of adding an objective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddObjectiveResult {
    /// ID of the created objective
    pub objective_id: String,
    /// Type of objective (fundamental or means)
    pub objective_type: String,
    /// Total fundamental objectives
    pub total_fundamental: usize,
    /// Total means objectives
    pub total_means: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of linking means to fundamental.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkMeansToFundamentalResult {
    /// Whether the link was created
    pub success: bool,
    /// Name of the means objective
    pub means_name: String,
    /// Name of the fundamental objective
    pub fundamental_name: String,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of updating an objective's measure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateObjectiveMeasureResult {
    /// Whether the update succeeded
    pub success: bool,
    /// The objective that was updated
    pub objective_name: String,
    /// The new measure
    pub new_measure: String,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of removing an objective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveObjectiveResult {
    /// Whether the removal succeeded
    pub success: bool,
    /// Name of the removed objective
    pub removed_name: String,
    /// Remaining fundamental count
    pub remaining_fundamental: usize,
    /// Remaining means count
    pub remaining_means: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of promoting to fundamental.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromoteToFundamentalResult {
    /// Whether the promotion succeeded
    pub success: bool,
    /// Name of the promoted objective
    pub objective_name: String,
    /// New total fundamental count
    pub total_fundamental: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Definitions
// ═══════════════════════════════════════════════════════════════════════════

/// Creates the add_objective tool definition.
pub fn add_objective_tool() -> ToolDefinition {
    ToolDefinition::new(
        "add_objective",
        "Add an objective to the decision analysis. Use when user mentions something they want to achieve or avoid.",
        serde_json::json!({
            "type": "object",
            "required": ["name", "measure", "direction", "is_fundamental"],
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Brief name for the objective (e.g., 'Maximize compensation', 'Minimize commute')"
                },
                "measure": {
                    "type": "string",
                    "description": "How to measure this objective (e.g., 'Total comp in $/year', 'Minutes per day')"
                },
                "direction": {
                    "type": "string",
                    "enum": ["higher", "lower", "target"],
                    "description": "Whether higher values are better, lower are better, or a specific target"
                },
                "is_fundamental": {
                    "type": "boolean",
                    "description": "True if fundamental objective (what really matters), false if means objective (way to achieve something else)"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "objective_id": { "type": "string" },
                "objective_type": { "type": "string" },
                "total_fundamental": { "type": "integer" },
                "total_means": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the link_means_to_fundamental tool definition.
pub fn link_means_to_fundamental_tool() -> ToolDefinition {
    ToolDefinition::new(
        "link_means_to_fundamental",
        "Link a means objective to the fundamental objective it supports. Use to show objective hierarchy.",
        serde_json::json!({
            "type": "object",
            "required": ["means_id", "fundamental_id"],
            "properties": {
                "means_id": {
                    "type": "string",
                    "description": "ID of the means objective"
                },
                "fundamental_id": {
                    "type": "string",
                    "description": "ID of the fundamental objective it supports"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "means_name": { "type": "string" },
                "fundamental_name": { "type": "string" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the update_objective_measure tool definition.
pub fn update_objective_measure_tool() -> ToolDefinition {
    ToolDefinition::new(
        "update_objective_measure",
        "Refine how an objective is measured. Use when user provides better measurement criteria.",
        serde_json::json!({
            "type": "object",
            "required": ["objective_id", "new_measure"],
            "properties": {
                "objective_id": {
                    "type": "string",
                    "description": "ID of the objective to update"
                },
                "new_measure": {
                    "type": "string",
                    "description": "New measurement criteria"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "objective_name": { "type": "string" },
                "new_measure": { "type": "string" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the remove_objective tool definition.
pub fn remove_objective_tool() -> ToolDefinition {
    ToolDefinition::new(
        "remove_objective",
        "Remove an objective from the analysis. Requires a reason for audit trail.",
        serde_json::json!({
            "type": "object",
            "required": ["objective_id", "reason"],
            "properties": {
                "objective_id": {
                    "type": "string",
                    "description": "ID of the objective to remove"
                },
                "reason": {
                    "type": "string",
                    "description": "Reason for removal (for audit trail)"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "removed_name": { "type": "string" },
                "remaining_fundamental": { "type": "integer" },
                "remaining_means": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the promote_to_fundamental tool definition.
pub fn promote_to_fundamental_tool() -> ToolDefinition {
    ToolDefinition::new(
        "promote_to_fundamental",
        "Promote a means objective to fundamental. Use when user realizes a means objective is actually a core value.",
        serde_json::json!({
            "type": "object",
            "required": ["objective_id", "reason"],
            "properties": {
                "objective_id": {
                    "type": "string",
                    "description": "ID of the means objective to promote"
                },
                "reason": {
                    "type": "string",
                    "description": "Reason for promotion"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "objective_name": { "type": "string" },
                "total_fundamental": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Returns all Objectives tool definitions.
pub fn all_objectives_tools() -> Vec<ToolDefinition> {
    vec![
        add_objective_tool(),
        link_means_to_fundamental_tool(),
        update_objective_measure_tool(),
        remove_objective_tool(),
        promote_to_fundamental_tool(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn objective_direction_serializes_to_snake_case() {
        assert_eq!(serde_json::to_string(&ObjectiveDirection::Higher).unwrap(), "\"higher\"");
        assert_eq!(serde_json::to_string(&ObjectiveDirection::Lower).unwrap(), "\"lower\"");
        assert_eq!(serde_json::to_string(&ObjectiveDirection::Target).unwrap(), "\"target\"");
    }

    #[test]
    fn add_objective_params_serializes() {
        let params = AddObjectiveParams {
            name: "Maximize compensation".to_string(),
            measure: "Total comp in $/year".to_string(),
            direction: ObjectiveDirection::Higher,
            is_fundamental: true,
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["direction"], "higher");
        assert_eq!(json["is_fundamental"], true);
    }

    #[test]
    fn all_objectives_tools_returns_five_tools() {
        let tools = all_objectives_tools();
        assert_eq!(tools.len(), 5);
    }

    #[test]
    fn add_objective_tool_has_direction_enum() {
        let tool = add_objective_tool();
        let schema = tool.parameters_schema();
        let direction = &schema["properties"]["direction"];
        let enum_values = direction["enum"].as_array().unwrap();
        assert_eq!(enum_values.len(), 3);
    }
}
