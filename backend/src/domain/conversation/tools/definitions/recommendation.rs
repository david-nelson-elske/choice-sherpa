//! Recommendation Tools - Tools for synthesizing decision analysis.
//!
//! Recommendation is where the analysis is summarized and presented to the
//! decision maker. The AI agent DOES NOT decide for the user - it synthesizes
//! the analysis and highlights what makes each alternative stand out.

use serde::{Deserialize, Serialize};

use crate::domain::conversation::tools::ToolDefinition;

// ═══════════════════════════════════════════════════════════════════════════
// Enums
// ═══════════════════════════════════════════════════════════════════════════

/// Type of consideration to add.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsiderationType {
    /// In favor of an alternative
    Pro,
    /// Against an alternative
    Con,
    /// Neutral observation
    Neutral,
    /// Key risk to consider
    Risk,
    /// Important opportunity
    Opportunity,
}

/// Urgency level for remaining uncertainties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UncertaintyUrgency {
    /// Must resolve before deciding
    Blocking,
    /// Should resolve if possible
    Important,
    /// Nice to know but not critical
    Optional,
    /// Can be resolved after decision
    PostDecision,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Parameters
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters for setting the synthesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetSynthesisParams {
    /// Overall synthesis text
    pub synthesis: String,
    /// Top alternatives to consider (in order)
    pub top_alternatives: Vec<String>,
    /// Key differentiating factors
    pub key_factors: Vec<String>,
}

/// Parameters for setting what makes an alternative stand out.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetStandoutParams {
    /// ID of the alternative
    pub alternative_id: String,
    /// What makes this alternative unique
    pub standout_qualities: Vec<String>,
    /// Best for what type of person/situation
    pub best_for: String,
    /// Key risks if choosing this alternative
    pub key_risks: Vec<String>,
}

/// Parameters for adding a key consideration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddKeyConsiderationParams {
    /// The consideration
    pub consideration: String,
    /// Type of consideration
    pub consideration_type: ConsiderationType,
    /// Which alternatives this applies to
    pub applies_to: Vec<String>,
    /// Source or reasoning
    pub source: Option<String>,
}

/// Parameters for adding a remaining uncertainty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRemainingUncertaintyParams {
    /// Description of the uncertainty
    pub description: String,
    /// How urgent is resolution
    pub urgency: UncertaintyUrgency,
    /// How to potentially resolve it
    pub resolution_path: Option<String>,
    /// Which alternatives are most affected
    pub affects: Vec<String>,
}

/// Parameters for setting a decision prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetDecisionPromptParams {
    /// The key question to ask
    pub question: String,
    /// Context for the question
    pub context: String,
    /// The tradeoff this question addresses
    pub addresses_tradeoff: Option<String>,
}

/// Parameters for summarizing the decision frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizeFrameParams {
    /// Include objectives summary
    pub include_objectives: bool,
    /// Include alternatives summary
    pub include_alternatives: bool,
    /// Include constraints
    pub include_constraints: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Results
// ═══════════════════════════════════════════════════════════════════════════

/// Result of setting the synthesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetSynthesisResult {
    /// Whether synthesis was set
    pub success: bool,
    /// Number of top alternatives identified
    pub top_count: usize,
    /// Number of key factors
    pub factors_count: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of setting standout qualities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetStandoutResult {
    /// Whether standout was set
    pub success: bool,
    /// Name of the alternative
    pub alternative_name: String,
    /// Number of standout qualities
    pub qualities_count: usize,
    /// Number of risks identified
    pub risks_count: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of adding a key consideration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddKeyConsiderationResult {
    /// ID of the consideration
    pub consideration_id: String,
    /// Total considerations
    pub total_considerations: usize,
    /// Counts by type
    pub pros_count: usize,
    pub cons_count: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of adding a remaining uncertainty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRemainingUncertaintyResult {
    /// ID of the uncertainty
    pub uncertainty_id: String,
    /// Total remaining uncertainties
    pub total_uncertainties: usize,
    /// Count of blocking uncertainties
    pub blocking_count: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of setting a decision prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetDecisionPromptResult {
    /// Whether prompt was set
    pub success: bool,
    /// Total decision prompts
    pub total_prompts: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of summarizing the frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizeFrameResult {
    /// The summary text
    pub summary: String,
    /// Number of objectives included
    pub objectives_count: usize,
    /// Number of alternatives included
    pub alternatives_count: usize,
    /// Number of constraints included
    pub constraints_count: usize,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Definitions
// ═══════════════════════════════════════════════════════════════════════════

/// Creates the set_synthesis tool definition.
pub fn set_synthesis_tool() -> ToolDefinition {
    ToolDefinition::new(
        "set_synthesis",
        "Set the overall synthesis of the decision analysis. Summarizes key findings without making the decision.",
        serde_json::json!({
            "type": "object",
            "required": ["synthesis", "top_alternatives", "key_factors"],
            "properties": {
                "synthesis": {
                    "type": "string",
                    "description": "Overall synthesis text summarizing the analysis"
                },
                "top_alternatives": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "IDs of top alternatives in order of analysis strength"
                },
                "key_factors": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Key differentiating factors identified"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "top_count": { "type": "integer" },
                "factors_count": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the set_standout tool definition.
pub fn set_standout_tool() -> ToolDefinition {
    ToolDefinition::new(
        "set_standout",
        "Define what makes an alternative stand out. Helps user understand each option's unique value.",
        serde_json::json!({
            "type": "object",
            "required": ["alternative_id", "standout_qualities", "best_for", "key_risks"],
            "properties": {
                "alternative_id": {
                    "type": "string",
                    "description": "ID of the alternative"
                },
                "standout_qualities": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "What makes this alternative unique or compelling"
                },
                "best_for": {
                    "type": "string",
                    "description": "What type of person or situation this is best for"
                },
                "key_risks": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Key risks if choosing this alternative"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "alternative_name": { "type": "string" },
                "qualities_count": { "type": "integer" },
                "risks_count": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the add_key_consideration tool definition.
pub fn add_key_consideration_tool() -> ToolDefinition {
    ToolDefinition::new(
        "add_key_consideration",
        "Add a key consideration for the decision. Pros, cons, risks, or opportunities.",
        serde_json::json!({
            "type": "object",
            "required": ["consideration", "consideration_type", "applies_to"],
            "properties": {
                "consideration": {
                    "type": "string",
                    "description": "The consideration"
                },
                "consideration_type": {
                    "type": "string",
                    "enum": ["pro", "con", "neutral", "risk", "opportunity"],
                    "description": "Type of consideration"
                },
                "applies_to": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Alternative IDs this applies to (empty for general)"
                },
                "source": {
                    "type": "string",
                    "description": "Source or reasoning for this consideration"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "consideration_id": { "type": "string" },
                "total_considerations": { "type": "integer" },
                "pros_count": { "type": "integer" },
                "cons_count": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the add_remaining_uncertainty tool definition.
pub fn add_remaining_uncertainty_tool() -> ToolDefinition {
    ToolDefinition::new(
        "add_remaining_uncertainty",
        "Flag an uncertainty that remains unresolved. Indicates what's still unknown before deciding.",
        serde_json::json!({
            "type": "object",
            "required": ["description", "urgency", "affects"],
            "properties": {
                "description": {
                    "type": "string",
                    "description": "What is uncertain"
                },
                "urgency": {
                    "type": "string",
                    "enum": ["blocking", "important", "optional", "post_decision"],
                    "description": "How urgently this needs resolution"
                },
                "resolution_path": {
                    "type": "string",
                    "description": "How to potentially resolve this uncertainty"
                },
                "affects": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Alternative IDs most affected by this uncertainty"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "uncertainty_id": { "type": "string" },
                "total_uncertainties": { "type": "integer" },
                "blocking_count": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the set_decision_prompt tool definition.
pub fn set_decision_prompt_tool() -> ToolDefinition {
    ToolDefinition::new(
        "set_decision_prompt",
        "Set a key question for the decision maker to consider. Helps focus reflection.",
        serde_json::json!({
            "type": "object",
            "required": ["question", "context"],
            "properties": {
                "question": {
                    "type": "string",
                    "description": "The key question to ask"
                },
                "context": {
                    "type": "string",
                    "description": "Context for why this question matters"
                },
                "addresses_tradeoff": {
                    "type": "string",
                    "description": "ID of tradeoff this question addresses"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "total_prompts": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the summarize_frame tool definition.
pub fn summarize_frame_tool() -> ToolDefinition {
    ToolDefinition::new(
        "summarize_frame",
        "Generate a summary of the decision frame. Useful for review or handoff.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "include_objectives": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include objectives summary"
                },
                "include_alternatives": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include alternatives summary"
                },
                "include_constraints": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include constraints"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "summary": { "type": "string" },
                "objectives_count": { "type": "integer" },
                "alternatives_count": { "type": "integer" },
                "constraints_count": { "type": "integer" }
            }
        }),
    )
}

/// Returns all Recommendation tool definitions.
pub fn all_recommendation_tools() -> Vec<ToolDefinition> {
    vec![
        set_synthesis_tool(),
        set_standout_tool(),
        add_key_consideration_tool(),
        add_remaining_uncertainty_tool(),
        set_decision_prompt_tool(),
        summarize_frame_tool(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consideration_type_serializes_to_snake_case() {
        assert_eq!(serde_json::to_string(&ConsiderationType::Pro).unwrap(), "\"pro\"");
        assert_eq!(serde_json::to_string(&ConsiderationType::Con).unwrap(), "\"con\"");
        assert_eq!(serde_json::to_string(&ConsiderationType::Risk).unwrap(), "\"risk\"");
    }

    #[test]
    fn uncertainty_urgency_serializes() {
        assert_eq!(serde_json::to_string(&UncertaintyUrgency::Blocking).unwrap(), "\"blocking\"");
        assert_eq!(serde_json::to_string(&UncertaintyUrgency::PostDecision).unwrap(), "\"post_decision\"");
    }

    #[test]
    fn set_standout_params_serializes() {
        let params = SetStandoutParams {
            alternative_id: "alt_a".to_string(),
            standout_qualities: vec!["High salary".to_string()],
            best_for: "Risk-tolerant individuals".to_string(),
            key_risks: vec!["Startup may fail".to_string()],
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["standout_qualities"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn all_recommendation_tools_returns_six_tools() {
        let tools = all_recommendation_tools();
        assert_eq!(tools.len(), 6);
    }

    #[test]
    fn tool_definitions_have_correct_names() {
        let tools = all_recommendation_tools();
        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();

        assert!(names.contains(&"set_synthesis"));
        assert!(names.contains(&"set_standout"));
        assert!(names.contains(&"add_key_consideration"));
        assert!(names.contains(&"add_remaining_uncertainty"));
        assert!(names.contains(&"set_decision_prompt"));
        assert!(names.contains(&"summarize_frame"));
    }

    #[test]
    fn add_key_consideration_has_consideration_type_enum() {
        let tool = add_key_consideration_tool();
        let schema = tool.parameters_schema();
        let consideration_type = &schema["properties"]["consideration_type"];
        let enum_values = consideration_type["enum"].as_array().unwrap();
        assert_eq!(enum_values.len(), 5);
    }
}
