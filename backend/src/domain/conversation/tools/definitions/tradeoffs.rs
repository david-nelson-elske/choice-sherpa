//! Tradeoffs Tools - Tools for analyzing and surfacing tradeoffs.
//!
//! Tradeoffs is where dominated alternatives are identified, irrelevant
//! objectives are surfaced, and key tensions are highlighted. This component
//! helps focus the decision on what truly matters.

use serde::{Deserialize, Serialize};

use crate::domain::conversation::tools::ToolDefinition;

// ═══════════════════════════════════════════════════════════════════════════
// Enums
// ═══════════════════════════════════════════════════════════════════════════

/// Type of dominance relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DominanceType {
    /// Dominated on all objectives (clearly worse)
    Full,
    /// Dominated on weighted objectives (worse considering priorities)
    Weighted,
    /// Nearly dominated (worse on almost all objectives)
    Practical,
}

/// Reason why an objective might be irrelevant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IrrelevanceReason {
    /// All alternatives score the same
    NoVariation,
    /// Objective is redundant with another
    Redundant,
    /// User explicitly marked as unimportant
    UserDismissed,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Parameters - Analysis Tools
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters for computing Pugh totals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputePughTotalsParams {
    /// Whether to include weighting
    pub weighted: bool,
}

/// Parameters for finding dominated alternatives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindDominatedAlternativesParams {
    /// Threshold for practical dominance (0.0-1.0)
    pub practical_threshold: Option<f64>,
}

/// Parameters for checking sensitivity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityCheckParams {
    /// ID of objective to vary
    pub objective_id: String,
    /// Range of weights to test (e.g., 0.5 to 2.0)
    pub weight_range: Option<(f64, f64)>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Parameters - Tradeoff Marking Tools
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters for marking an alternative as dominated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkDominatedParams {
    /// ID of the dominated alternative
    pub alternative_id: String,
    /// ID of the dominating alternative
    pub dominated_by: String,
    /// Type of dominance
    pub dominance_type: DominanceType,
    /// Explanation
    pub explanation: String,
}

/// Parameters for marking an objective as irrelevant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkIrrelevantObjectiveParams {
    /// ID of the objective
    pub objective_id: String,
    /// Reason for irrelevance
    pub reason: IrrelevanceReason,
    /// Additional explanation
    pub explanation: String,
}

/// Parameters for adding a key tension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTensionParams {
    /// First objective in tension
    pub objective_a_id: String,
    /// Second objective in tension
    pub objective_b_id: String,
    /// Description of the tension
    pub description: String,
    /// Which alternatives highlight this tension
    pub highlighted_by: Vec<String>,
}

/// Parameters for clearing dominated status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearDominatedParams {
    /// ID of the alternative to clear
    pub alternative_id: String,
    /// Reason for clearing
    pub reason: String,
}

/// Parameters for highlighting a tradeoff pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightTradeoffParams {
    /// First alternative
    pub alternative_a_id: String,
    /// Second alternative
    pub alternative_b_id: String,
    /// What alternative A gains
    pub a_gains: Vec<String>,
    /// What alternative A loses
    pub a_loses: Vec<String>,
    /// Summary of the tradeoff
    pub summary: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Results - Analysis Tools
// ═══════════════════════════════════════════════════════════════════════════

/// Pugh totals for an alternative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativePughTotal {
    /// Alternative ID
    pub alternative_id: String,
    /// Alternative name
    pub alternative_name: String,
    /// Sum of positive ratings
    pub positives: i32,
    /// Sum of negative ratings
    pub negatives: i32,
    /// Net score (positives - negatives)
    pub net: i32,
    /// Weighted score (if applicable)
    pub weighted_net: Option<f64>,
}

/// Result of computing Pugh totals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputePughTotalsResult {
    /// Totals for each alternative
    pub alternatives: Vec<AlternativePughTotal>,
    /// ID of highest scoring alternative
    pub leader: String,
    /// Whether there's a tie
    pub has_tie: bool,
}

/// A dominated alternative finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DominatedAlternative {
    /// ID of the dominated alternative
    pub alternative_id: String,
    /// Name of the dominated alternative
    pub alternative_name: String,
    /// ID of the dominating alternative
    pub dominated_by_id: String,
    /// Name of the dominating alternative
    pub dominated_by_name: String,
    /// Type of dominance
    pub dominance_type: DominanceType,
    /// Objectives where dominated alt is worse
    pub worse_on: Vec<String>,
    /// Objectives where dominated alt is equal
    pub equal_on: Vec<String>,
    /// Objectives where dominated alt is better (for practical dominance)
    pub better_on: Vec<String>,
}

/// Result of finding dominated alternatives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindDominatedAlternativesResult {
    /// List of dominated alternatives
    pub dominated: Vec<DominatedAlternative>,
    /// Count by dominance type
    pub full_count: usize,
    pub weighted_count: usize,
    pub practical_count: usize,
}

/// An irrelevant objective finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrrelevantObjective {
    /// ID of the objective
    pub objective_id: String,
    /// Name of the objective
    pub objective_name: String,
    /// Reason for irrelevance
    pub reason: IrrelevanceReason,
    /// Rating value (if no variation)
    pub common_rating: Option<i8>,
    /// Redundant with (if redundant)
    pub redundant_with: Option<String>,
}

/// Result of finding irrelevant objectives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindIrrelevantObjectivesResult {
    /// List of irrelevant objectives
    pub irrelevant: Vec<IrrelevantObjective>,
    /// Total objectives analyzed
    pub total_objectives: usize,
    /// Remaining relevant objectives
    pub relevant_count: usize,
}

/// Result of sensitivity check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityCheckResult {
    /// Objective analyzed
    pub objective_name: String,
    /// Whether result changes with objective weight
    pub is_sensitive: bool,
    /// Weight at which winner changes (if any)
    pub switch_point: Option<f64>,
    /// Winner below switch point
    pub low_weight_winner: Option<String>,
    /// Winner above switch point
    pub high_weight_winner: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Results - Tradeoff Marking Tools
// ═══════════════════════════════════════════════════════════════════════════

/// Result of marking an alternative as dominated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkDominatedResult {
    /// Whether marking succeeded
    pub success: bool,
    /// Name of the dominated alternative
    pub alternative_name: String,
    /// Total dominated alternatives
    pub total_dominated: usize,
    /// Remaining active alternatives
    pub active_alternatives: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of marking an objective as irrelevant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkIrrelevantObjectiveResult {
    /// Whether marking succeeded
    pub success: bool,
    /// Name of the objective
    pub objective_name: String,
    /// Remaining relevant objectives
    pub remaining_relevant: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of adding a tension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTensionResult {
    /// ID of the created tension
    pub tension_id: String,
    /// Total tensions identified
    pub total_tensions: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of clearing dominated status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearDominatedResult {
    /// Whether clearing succeeded
    pub success: bool,
    /// Name of the alternative
    pub alternative_name: String,
    /// Remaining dominated count
    pub remaining_dominated: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of highlighting a tradeoff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightTradeoffResult {
    /// ID of the created tradeoff highlight
    pub tradeoff_id: String,
    /// Total tradeoffs highlighted
    pub total_tradeoffs: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Definitions - Analysis Tools
// ═══════════════════════════════════════════════════════════════════════════

/// Creates the compute_pugh_totals tool definition.
pub fn compute_pugh_totals_tool() -> ToolDefinition {
    ToolDefinition::new(
        "compute_pugh_totals",
        "Calculate Pugh matrix totals for each alternative. Returns positives, negatives, and net scores.",
        serde_json::json!({
            "type": "object",
            "required": ["weighted"],
            "properties": {
                "weighted": {
                    "type": "boolean",
                    "description": "Whether to apply objective weights to the calculation"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "alternatives": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "alternative_id": { "type": "string" },
                            "alternative_name": { "type": "string" },
                            "positives": { "type": "integer" },
                            "negatives": { "type": "integer" },
                            "net": { "type": "integer" },
                            "weighted_net": { "type": "number" }
                        }
                    }
                },
                "leader": { "type": "string" },
                "has_tie": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the find_dominated_alternatives tool definition.
pub fn find_dominated_alternatives_tool() -> ToolDefinition {
    ToolDefinition::new(
        "find_dominated_alternatives",
        "Find alternatives that are dominated by others. Dominated = worse on all/most objectives.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "practical_threshold": {
                    "type": "number",
                    "minimum": 0.0,
                    "maximum": 1.0,
                    "description": "Threshold for practical dominance (e.g., 0.8 = worse on 80% of objectives)"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "dominated": {
                    "type": "array",
                    "items": { "type": "object" }
                },
                "full_count": { "type": "integer" },
                "weighted_count": { "type": "integer" },
                "practical_count": { "type": "integer" }
            }
        }),
    )
}

/// Creates the find_irrelevant_objectives tool definition.
pub fn find_irrelevant_objectives_tool() -> ToolDefinition {
    ToolDefinition::new(
        "find_irrelevant_objectives",
        "Find objectives that don't differentiate between alternatives. These can be deprioritized.",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "irrelevant": {
                    "type": "array",
                    "items": { "type": "object" }
                },
                "total_objectives": { "type": "integer" },
                "relevant_count": { "type": "integer" }
            }
        }),
    )
}

/// Creates the sensitivity_check tool definition.
pub fn sensitivity_check_tool() -> ToolDefinition {
    ToolDefinition::new(
        "sensitivity_check",
        "Check how sensitive the result is to an objective's importance. Useful for uncertain priorities.",
        serde_json::json!({
            "type": "object",
            "required": ["objective_id"],
            "properties": {
                "objective_id": {
                    "type": "string",
                    "description": "ID of objective to analyze"
                },
                "weight_range": {
                    "type": "array",
                    "items": { "type": "number" },
                    "minItems": 2,
                    "maxItems": 2,
                    "description": "Range of weights to test [low, high] (e.g., [0.5, 2.0])"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "objective_name": { "type": "string" },
                "is_sensitive": { "type": "boolean" },
                "switch_point": { "type": "number" },
                "low_weight_winner": { "type": "string" },
                "high_weight_winner": { "type": "string" }
            }
        }),
    )
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Definitions - Tradeoff Marking Tools
// ═══════════════════════════════════════════════════════════════════════════

/// Creates the mark_dominated tool definition.
pub fn mark_dominated_tool() -> ToolDefinition {
    ToolDefinition::new(
        "mark_dominated",
        "Mark an alternative as dominated by another. Dominated alternatives are deprioritized.",
        serde_json::json!({
            "type": "object",
            "required": ["alternative_id", "dominated_by", "dominance_type", "explanation"],
            "properties": {
                "alternative_id": {
                    "type": "string",
                    "description": "ID of the dominated alternative"
                },
                "dominated_by": {
                    "type": "string",
                    "description": "ID of the dominating alternative"
                },
                "dominance_type": {
                    "type": "string",
                    "enum": ["full", "weighted", "practical"],
                    "description": "Type of dominance"
                },
                "explanation": {
                    "type": "string",
                    "description": "Why this alternative is dominated"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "alternative_name": { "type": "string" },
                "total_dominated": { "type": "integer" },
                "active_alternatives": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the mark_irrelevant_objective tool definition.
pub fn mark_irrelevant_objective_tool() -> ToolDefinition {
    ToolDefinition::new(
        "mark_irrelevant_objective",
        "Mark an objective as not relevant for this decision. Won't affect scoring.",
        serde_json::json!({
            "type": "object",
            "required": ["objective_id", "reason", "explanation"],
            "properties": {
                "objective_id": {
                    "type": "string",
                    "description": "ID of the objective"
                },
                "reason": {
                    "type": "string",
                    "enum": ["no_variation", "redundant", "user_dismissed"],
                    "description": "Reason for irrelevance"
                },
                "explanation": {
                    "type": "string",
                    "description": "Additional explanation"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "objective_name": { "type": "string" },
                "remaining_relevant": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the add_tension tool definition.
pub fn add_tension_tool() -> ToolDefinition {
    ToolDefinition::new(
        "add_tension",
        "Record a tension between two objectives. Tensions highlight key tradeoffs.",
        serde_json::json!({
            "type": "object",
            "required": ["objective_a_id", "objective_b_id", "description"],
            "properties": {
                "objective_a_id": {
                    "type": "string",
                    "description": "First objective in tension"
                },
                "objective_b_id": {
                    "type": "string",
                    "description": "Second objective in tension"
                },
                "description": {
                    "type": "string",
                    "description": "Description of the tension"
                },
                "highlighted_by": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Alternative IDs that highlight this tension"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "tension_id": { "type": "string" },
                "total_tensions": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the clear_dominated tool definition.
pub fn clear_dominated_tool() -> ToolDefinition {
    ToolDefinition::new(
        "clear_dominated",
        "Remove dominated status from an alternative. Use when user wants to reconsider.",
        serde_json::json!({
            "type": "object",
            "required": ["alternative_id", "reason"],
            "properties": {
                "alternative_id": {
                    "type": "string",
                    "description": "ID of the alternative"
                },
                "reason": {
                    "type": "string",
                    "description": "Reason for clearing dominated status"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "alternative_name": { "type": "string" },
                "remaining_dominated": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the highlight_tradeoff tool definition.
pub fn highlight_tradeoff_tool() -> ToolDefinition {
    ToolDefinition::new(
        "highlight_tradeoff",
        "Highlight a key tradeoff between two alternatives. Makes the decision clearer.",
        serde_json::json!({
            "type": "object",
            "required": ["alternative_a_id", "alternative_b_id", "a_gains", "a_loses", "summary"],
            "properties": {
                "alternative_a_id": {
                    "type": "string",
                    "description": "First alternative"
                },
                "alternative_b_id": {
                    "type": "string",
                    "description": "Second alternative"
                },
                "a_gains": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Objective IDs where alternative A is better"
                },
                "a_loses": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Objective IDs where alternative A is worse"
                },
                "summary": {
                    "type": "string",
                    "description": "Summary of the tradeoff"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "tradeoff_id": { "type": "string" },
                "total_tradeoffs": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Returns all Tradeoffs tool definitions.
pub fn all_tradeoffs_tools() -> Vec<ToolDefinition> {
    vec![
        // Analysis tools
        compute_pugh_totals_tool(),
        find_dominated_alternatives_tool(),
        find_irrelevant_objectives_tool(),
        sensitivity_check_tool(),
        // Marking tools
        mark_dominated_tool(),
        mark_irrelevant_objective_tool(),
        add_tension_tool(),
        clear_dominated_tool(),
        highlight_tradeoff_tool(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dominance_type_serializes_to_snake_case() {
        assert_eq!(serde_json::to_string(&DominanceType::Full).unwrap(), "\"full\"");
        assert_eq!(serde_json::to_string(&DominanceType::Weighted).unwrap(), "\"weighted\"");
        assert_eq!(serde_json::to_string(&DominanceType::Practical).unwrap(), "\"practical\"");
    }

    #[test]
    fn irrelevance_reason_serializes() {
        assert_eq!(serde_json::to_string(&IrrelevanceReason::NoVariation).unwrap(), "\"no_variation\"");
        assert_eq!(serde_json::to_string(&IrrelevanceReason::Redundant).unwrap(), "\"redundant\"");
    }

    #[test]
    fn mark_dominated_params_serializes() {
        let params = MarkDominatedParams {
            alternative_id: "alt_c".to_string(),
            dominated_by: "alt_a".to_string(),
            dominance_type: DominanceType::Full,
            explanation: "Worse on all objectives".to_string(),
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["dominance_type"], "full");
    }

    #[test]
    fn all_tradeoffs_tools_returns_nine_tools() {
        let tools = all_tradeoffs_tools();
        assert_eq!(tools.len(), 9);
    }

    #[test]
    fn compute_pugh_totals_has_required_weighted_param() {
        let tool = compute_pugh_totals_tool();
        let schema = tool.parameters_schema();
        let required = schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v == "weighted"));
    }

    #[test]
    fn tool_names_are_distinct() {
        let tools = all_tradeoffs_tools();
        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        let unique_names: std::collections::HashSet<&str> = names.iter().copied().collect();
        assert_eq!(names.len(), unique_names.len());
    }
}
