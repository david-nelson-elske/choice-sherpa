//! Problem Frame Tools - Tools for defining decision architecture.
//!
//! Problem Frame is where users define the decision structure: who decides,
//! what's the focal statement, what's in/out of scope, constraints, parties,
//! and deadlines.

use serde::{Deserialize, Serialize};

use crate::domain::conversation::tools::ToolDefinition;

// ═══════════════════════════════════════════════════════════════════════════
// Enums
// ═══════════════════════════════════════════════════════════════════════════

/// Role of a party in the decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartyRole {
    /// Makes the final decision
    DecisionMaker,
    /// Has significant influence
    Stakeholder,
    /// Provides input but doesn't decide
    Advisor,
    /// Affected by but doesn't participate in decision
    Affected,
}

/// Level in decision hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HierarchyLevel {
    /// Strategic, long-term decisions
    Strategic,
    /// Tactical, medium-term decisions
    Tactical,
    /// Operational, day-to-day decisions
    Operational,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Parameters
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters for setting the decision maker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetDecisionMakerParams {
    /// Name of the decision maker
    pub name: String,
    /// Role or title
    pub role: String,
}

/// Parameters for setting the focal statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFocalStatementParams {
    /// The focal decision statement
    pub statement: String,
}

/// Parameters for setting the scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetScopeParams {
    /// What is in scope
    pub in_scope: Vec<String>,
    /// What is out of scope
    pub out_scope: Vec<String>,
}

/// Parameters for adding a constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddConstraintParams {
    /// Type of constraint (budget, time, resource, policy, other)
    pub constraint_type: String,
    /// Description of the constraint
    pub description: String,
}

/// Parameters for adding a party.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPartyParams {
    /// Name of the party
    pub name: String,
    /// Role in the decision
    pub role: PartyRole,
    /// Their concerns or interests
    pub concerns: Vec<String>,
}

/// Parameters for setting the deadline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetDeadlineParams {
    /// Deadline description or date
    pub deadline: String,
    /// Whether this is a hard deadline
    pub hard: bool,
}

/// Parameters for adding to decision hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddHierarchyDecisionParams {
    /// Description of the related decision
    pub description: String,
    /// Level in the hierarchy
    pub level: HierarchyLevel,
    /// Status of this decision
    pub status: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Results
// ═══════════════════════════════════════════════════════════════════════════

/// Result of setting the decision maker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetDecisionMakerResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of setting the focal statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFocalStatementResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// The statement that was set
    pub statement: String,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of setting the scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetScopeResult {
    /// Number of in-scope items
    pub in_scope_count: usize,
    /// Number of out-scope items
    pub out_scope_count: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of adding a constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddConstraintResult {
    /// ID of the created constraint
    pub constraint_id: String,
    /// Total number of constraints
    pub total_constraints: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of adding a party.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPartyResult {
    /// ID of the added party
    pub party_id: String,
    /// Total number of parties
    pub total_parties: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of setting the deadline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetDeadlineResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// Whether it's a hard deadline
    pub is_hard: bool,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of adding to decision hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddHierarchyDecisionResult {
    /// ID of the added decision
    pub decision_id: String,
    /// Total decisions in hierarchy
    pub total_in_hierarchy: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Definitions
// ═══════════════════════════════════════════════════════════════════════════

/// Creates the set_decision_maker tool definition.
pub fn set_decision_maker_tool() -> ToolDefinition {
    ToolDefinition::new(
        "set_decision_maker",
        "Define who makes the final decision. Use when user identifies who will decide.",
        serde_json::json!({
            "type": "object",
            "required": ["name", "role"],
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the decision maker"
                },
                "role": {
                    "type": "string",
                    "description": "Role or title of the decision maker"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the set_focal_statement tool definition.
pub fn set_focal_statement_tool() -> ToolDefinition {
    ToolDefinition::new(
        "set_focal_statement",
        "Set the focal decision statement. This is the clear, concise statement of what needs to be decided.",
        serde_json::json!({
            "type": "object",
            "required": ["statement"],
            "properties": {
                "statement": {
                    "type": "string",
                    "description": "The focal decision statement",
                    "minLength": 10
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "statement": { "type": "string" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the set_scope tool definition.
pub fn set_scope_tool() -> ToolDefinition {
    ToolDefinition::new(
        "set_scope",
        "Define what is in and out of scope for this decision.",
        serde_json::json!({
            "type": "object",
            "required": ["in_scope", "out_scope"],
            "properties": {
                "in_scope": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of things that are in scope"
                },
                "out_scope": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of things that are out of scope"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "in_scope_count": { "type": "integer" },
                "out_scope_count": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the add_constraint tool definition.
pub fn add_constraint_tool() -> ToolDefinition {
    ToolDefinition::new(
        "add_constraint",
        "Add a constraint on the decision. Use for budget limits, time constraints, resource limitations, or policy requirements.",
        serde_json::json!({
            "type": "object",
            "required": ["constraint_type", "description"],
            "properties": {
                "constraint_type": {
                    "type": "string",
                    "enum": ["budget", "time", "resource", "policy", "other"],
                    "description": "Type of constraint"
                },
                "description": {
                    "type": "string",
                    "description": "Description of the constraint"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "constraint_id": { "type": "string" },
                "total_constraints": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the add_party tool definition.
pub fn add_party_tool() -> ToolDefinition {
    ToolDefinition::new(
        "add_party",
        "Add a stakeholder or party involved in the decision.",
        serde_json::json!({
            "type": "object",
            "required": ["name", "role", "concerns"],
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the party"
                },
                "role": {
                    "type": "string",
                    "enum": ["decision_maker", "stakeholder", "advisor", "affected"],
                    "description": "Role in the decision"
                },
                "concerns": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Their concerns or interests"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "party_id": { "type": "string" },
                "total_parties": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the set_deadline tool definition.
pub fn set_deadline_tool() -> ToolDefinition {
    ToolDefinition::new(
        "set_deadline",
        "Set the decision deadline. Indicate whether it's a hard or soft deadline.",
        serde_json::json!({
            "type": "object",
            "required": ["deadline", "hard"],
            "properties": {
                "deadline": {
                    "type": "string",
                    "description": "Deadline description or date"
                },
                "hard": {
                    "type": "boolean",
                    "description": "Whether this is a hard deadline that cannot be extended"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "is_hard": { "type": "boolean" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the add_hierarchy_decision tool definition.
pub fn add_hierarchy_decision_tool() -> ToolDefinition {
    ToolDefinition::new(
        "add_hierarchy_decision",
        "Add a related decision to the hierarchy. Use to show how this decision relates to larger or smaller decisions.",
        serde_json::json!({
            "type": "object",
            "required": ["description", "level", "status"],
            "properties": {
                "description": {
                    "type": "string",
                    "description": "Description of the related decision"
                },
                "level": {
                    "type": "string",
                    "enum": ["strategic", "tactical", "operational"],
                    "description": "Level in the decision hierarchy"
                },
                "status": {
                    "type": "string",
                    "description": "Current status of this decision (e.g., 'pending', 'decided', 'implemented')"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "decision_id": { "type": "string" },
                "total_in_hierarchy": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Returns all Problem Frame tool definitions.
pub fn all_problem_frame_tools() -> Vec<ToolDefinition> {
    vec![
        set_decision_maker_tool(),
        set_focal_statement_tool(),
        set_scope_tool(),
        add_constraint_tool(),
        add_party_tool(),
        set_deadline_tool(),
        add_hierarchy_decision_tool(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn party_role_serializes_to_snake_case() {
        let role = PartyRole::DecisionMaker;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"decision_maker\"");
    }

    #[test]
    fn hierarchy_level_serializes_to_snake_case() {
        let level = HierarchyLevel::Strategic;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"strategic\"");
    }

    #[test]
    fn add_party_params_serializes() {
        let params = AddPartyParams {
            name: "CEO".to_string(),
            role: PartyRole::Stakeholder,
            concerns: vec!["ROI".to_string(), "Timeline".to_string()],
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["role"], "stakeholder");
        assert_eq!(json["concerns"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn all_problem_frame_tools_returns_seven_tools() {
        let tools = all_problem_frame_tools();
        assert_eq!(tools.len(), 7);
    }

    #[test]
    fn tool_definitions_have_correct_names() {
        let tools = all_problem_frame_tools();
        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();

        assert!(names.contains(&"set_decision_maker"));
        assert!(names.contains(&"set_focal_statement"));
        assert!(names.contains(&"set_scope"));
        assert!(names.contains(&"add_constraint"));
        assert!(names.contains(&"add_party"));
        assert!(names.contains(&"set_deadline"));
        assert!(names.contains(&"add_hierarchy_decision"));
    }

    #[test]
    fn add_constraint_has_enum_type() {
        let tool = add_constraint_tool();
        let schema = tool.parameters_schema();
        let constraint_type = &schema["properties"]["constraint_type"];
        assert!(constraint_type["enum"].is_array());
    }
}
