//! Issue Raising Tools - Tools for the first PrOACT component.
//!
//! Issue Raising is where users dump their initial thoughts and the AI agent
//! helps categorize them into decisions, objectives, uncertainties, and
//! general considerations.

use serde::{Deserialize, Serialize};

use crate::domain::conversation::tools::ToolDefinition;

// ═══════════════════════════════════════════════════════════════════════════
// Tool Parameters
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters for adding a potential decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPotentialDecisionParams {
    /// Description of the decision to consider
    pub description: String,
}

/// Parameters for adding an objective idea.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddObjectiveIdeaParams {
    /// Description of the objective mentioned
    pub description: String,
}

/// Parameters for adding an uncertainty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddUncertaintyParams {
    /// Description of the uncertainty
    pub description: String,
    /// Whether this uncertainty can be resolved with more information
    pub resolvable: bool,
}

/// Parameters for adding a general consideration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddConsiderationParams {
    /// Description of the consideration
    pub description: String,
}

/// Parameters for setting the focal decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFocalDecisionParams {
    /// ID of the decision to focus on
    pub decision_id: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Results
// ═══════════════════════════════════════════════════════════════════════════

/// Result of adding a potential decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPotentialDecisionResult {
    /// ID of the created decision
    pub decision_id: String,
    /// Current count of potential decisions
    pub current_count: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of adding an objective idea.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddObjectiveIdeaResult {
    /// ID of the created objective idea
    pub objective_idea_id: String,
    /// Current count of objective ideas
    pub current_count: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of adding an uncertainty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddUncertaintyResult {
    /// ID of the created uncertainty
    pub uncertainty_id: String,
    /// Current count of uncertainties
    pub current_count: usize,
    /// Count of resolvable uncertainties
    pub resolvable_count: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of adding a consideration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddConsiderationResult {
    /// ID of the created consideration
    pub consideration_id: String,
    /// Current count of considerations
    pub current_count: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of setting the focal decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFocalDecisionResult {
    /// Whether the focal decision was set
    pub success: bool,
    /// The decision that is now focal
    pub focal_decision: String,
    /// Whether the document was updated
    pub document_updated: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Definitions
// ═══════════════════════════════════════════════════════════════════════════

/// Creates the add_potential_decision tool definition.
pub fn add_potential_decision_tool() -> ToolDefinition {
    ToolDefinition::new(
        "add_potential_decision",
        "Add a decision to consider. Use when user mentions something they need to decide.",
        serde_json::json!({
            "type": "object",
            "required": ["description"],
            "properties": {
                "description": {
                    "type": "string",
                    "description": "Description of the decision to consider",
                    "minLength": 1
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "decision_id": { "type": "string" },
                "current_count": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the add_objective_idea tool definition.
pub fn add_objective_idea_tool() -> ToolDefinition {
    ToolDefinition::new(
        "add_objective_idea",
        "Capture an objective mentioned by the user. Use when they express something they want to achieve or avoid.",
        serde_json::json!({
            "type": "object",
            "required": ["description"],
            "properties": {
                "description": {
                    "type": "string",
                    "description": "Description of the objective mentioned",
                    "minLength": 1
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "objective_idea_id": { "type": "string" },
                "current_count": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the add_uncertainty tool definition.
pub fn add_uncertainty_tool() -> ToolDefinition {
    ToolDefinition::new(
        "add_uncertainty",
        "Flag an uncertainty in the decision. Use when user expresses doubt or mentions unknown factors.",
        serde_json::json!({
            "type": "object",
            "required": ["description", "resolvable"],
            "properties": {
                "description": {
                    "type": "string",
                    "description": "Description of the uncertainty",
                    "minLength": 1
                },
                "resolvable": {
                    "type": "boolean",
                    "description": "Whether this uncertainty can be resolved with more information"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "uncertainty_id": { "type": "string" },
                "current_count": { "type": "integer" },
                "resolvable_count": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the add_consideration tool definition.
pub fn add_consideration_tool() -> ToolDefinition {
    ToolDefinition::new(
        "add_consideration",
        "Add a general consideration that doesn't fit other categories. Use for context, constraints, or other relevant information.",
        serde_json::json!({
            "type": "object",
            "required": ["description"],
            "properties": {
                "description": {
                    "type": "string",
                    "description": "Description of the consideration",
                    "minLength": 1
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "consideration_id": { "type": "string" },
                "current_count": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the set_focal_decision tool definition.
pub fn set_focal_decision_tool() -> ToolDefinition {
    ToolDefinition::new(
        "set_focal_decision",
        "Mark which decision to focus on. Use when user indicates which decision is primary.",
        serde_json::json!({
            "type": "object",
            "required": ["decision_id"],
            "properties": {
                "decision_id": {
                    "type": "string",
                    "description": "ID of the decision to focus on"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "focal_decision": { "type": "string" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Returns all Issue Raising tool definitions.
pub fn all_issue_raising_tools() -> Vec<ToolDefinition> {
    vec![
        add_potential_decision_tool(),
        add_objective_idea_tool(),
        add_uncertainty_tool(),
        add_consideration_tool(),
        set_focal_decision_tool(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_potential_decision_params_serializes() {
        let params = AddPotentialDecisionParams {
            description: "Should I accept the job offer?".to_string(),
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["description"], "Should I accept the job offer?");
    }

    #[test]
    fn add_potential_decision_result_deserializes() {
        let json = serde_json::json!({
            "decision_id": "dec_123",
            "current_count": 3,
            "document_updated": true
        });
        let result: AddPotentialDecisionResult = serde_json::from_value(json).unwrap();
        assert_eq!(result.decision_id, "dec_123");
        assert_eq!(result.current_count, 3);
        assert!(result.document_updated);
    }

    #[test]
    fn add_uncertainty_params_includes_resolvable() {
        let params = AddUncertaintyParams {
            description: "What is the team culture like?".to_string(),
            resolvable: true,
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["resolvable"], true);
    }

    #[test]
    fn all_issue_raising_tools_returns_five_tools() {
        let tools = all_issue_raising_tools();
        assert_eq!(tools.len(), 5);
    }

    #[test]
    fn tool_definitions_have_correct_names() {
        let tools = all_issue_raising_tools();
        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();

        assert!(names.contains(&"add_potential_decision"));
        assert!(names.contains(&"add_objective_idea"));
        assert!(names.contains(&"add_uncertainty"));
        assert!(names.contains(&"add_consideration"));
        assert!(names.contains(&"set_focal_decision"));
    }

    #[test]
    fn tool_definitions_have_required_fields() {
        let tool = add_potential_decision_tool();
        let schema = tool.parameters_schema();

        let required = schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v == "description"));
    }
}
