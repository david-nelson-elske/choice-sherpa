//! Consequences Tools - Tools for building the consequence table.
//!
//! Consequences is where users rate how each alternative performs against
//! each objective. Uses Pugh-style ratings (-2 to +2) relative to status quo.

use serde::{Deserialize, Serialize};

use crate::domain::conversation::tools::ToolDefinition;

// ═══════════════════════════════════════════════════════════════════════════
// Enums
// ═══════════════════════════════════════════════════════════════════════════

/// Pugh-style rating scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PughRating {
    /// Much worse than status quo (-2)
    #[serde(rename = "-2")]
    MuchWorse,
    /// Somewhat worse than status quo (-1)
    #[serde(rename = "-1")]
    SomewhatWorse,
    /// Same as status quo (0)
    #[serde(rename = "0")]
    Same,
    /// Somewhat better than status quo (+1)
    #[serde(rename = "+1")]
    SomewhatBetter,
    /// Much better than status quo (+2)
    #[serde(rename = "+2")]
    MuchBetter,
}

impl PughRating {
    /// Returns the numeric value of the rating.
    pub fn value(&self) -> i8 {
        match self {
            Self::MuchWorse => -2,
            Self::SomewhatWorse => -1,
            Self::Same => 0,
            Self::SomewhatBetter => 1,
            Self::MuchBetter => 2,
        }
    }
}

/// Confidence level for a consequence rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    /// High confidence in the rating
    High,
    /// Medium confidence
    Medium,
    /// Low confidence - may need more information
    Low,
    /// Very uncertain - rating is speculative
    Speculative,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Parameters
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters for rating a single consequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateConsequenceParams {
    /// ID of the alternative being rated
    pub alternative_id: String,
    /// ID of the objective being rated against
    pub objective_id: String,
    /// Pugh rating (-2 to +2)
    pub rating: i8,
    /// Reasoning for the rating
    pub reasoning: String,
    /// Confidence level
    pub confidence: ConfidenceLevel,
}

/// A single consequence rating for batch operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsequenceRatingInput {
    /// ID of the alternative
    pub alternative_id: String,
    /// ID of the objective
    pub objective_id: String,
    /// Pugh rating (-2 to +2)
    pub rating: i8,
    /// Brief reasoning
    pub reasoning: String,
}

/// Parameters for batch rating consequences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRateConsequencesParams {
    /// List of consequence ratings to apply
    pub ratings: Vec<ConsequenceRatingInput>,
    /// Default confidence for all ratings
    pub default_confidence: ConfidenceLevel,
}

/// Parameters for adding uncertainty to a consequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddConsequenceUncertaintyParams {
    /// ID of the alternative
    pub alternative_id: String,
    /// ID of the objective
    pub objective_id: String,
    /// Description of the uncertainty
    pub description: String,
    /// How the rating might change if uncertainty resolves
    pub potential_rating_range: Option<String>,
}

/// Parameters for updating rating reasoning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRatingReasoningParams {
    /// ID of the alternative
    pub alternative_id: String,
    /// ID of the objective
    pub objective_id: String,
    /// New reasoning text
    pub reasoning: String,
}

/// Parameters for setting a range estimate on a consequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetConsequenceRangeParams {
    /// ID of the alternative
    pub alternative_id: String,
    /// ID of the objective
    pub objective_id: String,
    /// Pessimistic rating
    pub low: i8,
    /// Most likely rating
    pub expected: i8,
    /// Optimistic rating
    pub high: i8,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Results
// ═══════════════════════════════════════════════════════════════════════════

/// Result of rating a consequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateConsequenceResult {
    /// Whether the rating was applied
    pub success: bool,
    /// Alternative name
    pub alternative_name: String,
    /// Objective name
    pub objective_name: String,
    /// Applied rating
    pub rating: i8,
    /// Count of cells now rated
    pub cells_rated: usize,
    /// Total cells in matrix
    pub total_cells: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of batch rating consequences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRateConsequencesResult {
    /// Number of ratings successfully applied
    pub ratings_applied: usize,
    /// Number of ratings that failed
    pub ratings_failed: usize,
    /// IDs of failed ratings (if any)
    pub failed_ids: Vec<String>,
    /// Cells now rated
    pub cells_rated: usize,
    /// Total cells in matrix
    pub total_cells: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of adding consequence uncertainty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddConsequenceUncertaintyResult {
    /// Whether the uncertainty was added
    pub success: bool,
    /// ID of the created uncertainty
    pub uncertainty_id: String,
    /// Total uncertainties on this cell
    pub cell_uncertainties: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of updating rating reasoning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRatingReasoningResult {
    /// Whether the update succeeded
    pub success: bool,
    /// Alternative name
    pub alternative_name: String,
    /// Objective name
    pub objective_name: String,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of setting a consequence range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetConsequenceRangeResult {
    /// Whether the range was set
    pub success: bool,
    /// Alternative name
    pub alternative_name: String,
    /// Objective name
    pub objective_name: String,
    /// Range span (high - low)
    pub range_span: i8,
    /// Whether the document was updated
    pub document_updated: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Definitions
// ═══════════════════════════════════════════════════════════════════════════

/// Creates the rate_consequence tool definition.
pub fn rate_consequence_tool() -> ToolDefinition {
    ToolDefinition::new(
        "rate_consequence",
        "Rate how an alternative performs on an objective. Use Pugh scale: -2 (much worse) to +2 (much better) relative to status quo.",
        serde_json::json!({
            "type": "object",
            "required": ["alternative_id", "objective_id", "rating", "reasoning", "confidence"],
            "properties": {
                "alternative_id": {
                    "type": "string",
                    "description": "ID of the alternative being rated"
                },
                "objective_id": {
                    "type": "string",
                    "description": "ID of the objective being rated against"
                },
                "rating": {
                    "type": "integer",
                    "minimum": -2,
                    "maximum": 2,
                    "description": "Pugh rating: -2 (much worse), -1 (worse), 0 (same), +1 (better), +2 (much better)"
                },
                "reasoning": {
                    "type": "string",
                    "description": "Explanation for this rating"
                },
                "confidence": {
                    "type": "string",
                    "enum": ["high", "medium", "low", "speculative"],
                    "description": "Confidence level in this rating"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "alternative_name": { "type": "string" },
                "objective_name": { "type": "string" },
                "rating": { "type": "integer" },
                "cells_rated": { "type": "integer" },
                "total_cells": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the batch_rate_consequences tool definition.
pub fn batch_rate_consequences_tool() -> ToolDefinition {
    ToolDefinition::new(
        "batch_rate_consequences",
        "Rate multiple consequence cells at once. Efficient for filling in obvious ratings.",
        serde_json::json!({
            "type": "object",
            "required": ["ratings", "default_confidence"],
            "properties": {
                "ratings": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["alternative_id", "objective_id", "rating", "reasoning"],
                        "properties": {
                            "alternative_id": { "type": "string" },
                            "objective_id": { "type": "string" },
                            "rating": { "type": "integer", "minimum": -2, "maximum": 2 },
                            "reasoning": { "type": "string" }
                        }
                    },
                    "description": "List of consequence ratings to apply"
                },
                "default_confidence": {
                    "type": "string",
                    "enum": ["high", "medium", "low", "speculative"],
                    "description": "Default confidence level for all ratings in this batch"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "ratings_applied": { "type": "integer" },
                "ratings_failed": { "type": "integer" },
                "failed_ids": { "type": "array", "items": { "type": "string" } },
                "cells_rated": { "type": "integer" },
                "total_cells": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the add_consequence_uncertainty tool definition.
pub fn add_consequence_uncertainty_tool() -> ToolDefinition {
    ToolDefinition::new(
        "add_consequence_uncertainty",
        "Flag uncertainty about a specific consequence rating. Use when rating depends on unknown factors.",
        serde_json::json!({
            "type": "object",
            "required": ["alternative_id", "objective_id", "description"],
            "properties": {
                "alternative_id": {
                    "type": "string",
                    "description": "ID of the alternative"
                },
                "objective_id": {
                    "type": "string",
                    "description": "ID of the objective"
                },
                "description": {
                    "type": "string",
                    "description": "What makes this consequence uncertain"
                },
                "potential_rating_range": {
                    "type": "string",
                    "description": "How the rating might change (e.g., 'could be -1 to +1 depending on market conditions')"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "uncertainty_id": { "type": "string" },
                "cell_uncertainties": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the update_rating_reasoning tool definition.
pub fn update_rating_reasoning_tool() -> ToolDefinition {
    ToolDefinition::new(
        "update_rating_reasoning",
        "Update the reasoning for an existing consequence rating without changing the rating itself.",
        serde_json::json!({
            "type": "object",
            "required": ["alternative_id", "objective_id", "reasoning"],
            "properties": {
                "alternative_id": {
                    "type": "string",
                    "description": "ID of the alternative"
                },
                "objective_id": {
                    "type": "string",
                    "description": "ID of the objective"
                },
                "reasoning": {
                    "type": "string",
                    "description": "New reasoning text"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "alternative_name": { "type": "string" },
                "objective_name": { "type": "string" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the set_consequence_range tool definition.
pub fn set_consequence_range_tool() -> ToolDefinition {
    ToolDefinition::new(
        "set_consequence_range",
        "Set a range estimate for a consequence (pessimistic, expected, optimistic). Use for uncertain consequences.",
        serde_json::json!({
            "type": "object",
            "required": ["alternative_id", "objective_id", "low", "expected", "high"],
            "properties": {
                "alternative_id": {
                    "type": "string",
                    "description": "ID of the alternative"
                },
                "objective_id": {
                    "type": "string",
                    "description": "ID of the objective"
                },
                "low": {
                    "type": "integer",
                    "minimum": -2,
                    "maximum": 2,
                    "description": "Pessimistic rating"
                },
                "expected": {
                    "type": "integer",
                    "minimum": -2,
                    "maximum": 2,
                    "description": "Most likely rating"
                },
                "high": {
                    "type": "integer",
                    "minimum": -2,
                    "maximum": 2,
                    "description": "Optimistic rating"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "alternative_name": { "type": "string" },
                "objective_name": { "type": "string" },
                "range_span": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Returns all Consequences tool definitions.
pub fn all_consequences_tools() -> Vec<ToolDefinition> {
    vec![
        rate_consequence_tool(),
        batch_rate_consequences_tool(),
        add_consequence_uncertainty_tool(),
        update_rating_reasoning_tool(),
        set_consequence_range_tool(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pugh_rating_value() {
        assert_eq!(PughRating::MuchWorse.value(), -2);
        assert_eq!(PughRating::SomewhatWorse.value(), -1);
        assert_eq!(PughRating::Same.value(), 0);
        assert_eq!(PughRating::SomewhatBetter.value(), 1);
        assert_eq!(PughRating::MuchBetter.value(), 2);
    }

    #[test]
    fn pugh_rating_serializes_as_numbers() {
        assert_eq!(serde_json::to_string(&PughRating::MuchWorse).unwrap(), "\"-2\"");
        assert_eq!(serde_json::to_string(&PughRating::MuchBetter).unwrap(), "\"+2\"");
    }

    #[test]
    fn confidence_level_serializes_to_snake_case() {
        assert_eq!(serde_json::to_string(&ConfidenceLevel::High).unwrap(), "\"high\"");
        assert_eq!(serde_json::to_string(&ConfidenceLevel::Speculative).unwrap(), "\"speculative\"");
    }

    #[test]
    fn rate_consequence_params_serializes() {
        let params = RateConsequenceParams {
            alternative_id: "alt_a".to_string(),
            objective_id: "obj_1".to_string(),
            rating: 1,
            reasoning: "Better salary".to_string(),
            confidence: ConfidenceLevel::High,
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["rating"], 1);
        assert_eq!(json["confidence"], "high");
    }

    #[test]
    fn all_consequences_tools_returns_five_tools() {
        let tools = all_consequences_tools();
        assert_eq!(tools.len(), 5);
    }

    #[test]
    fn rate_consequence_has_rating_constraints() {
        let tool = rate_consequence_tool();
        let schema = tool.parameters_schema();
        let rating = &schema["properties"]["rating"];
        assert_eq!(rating["minimum"], -2);
        assert_eq!(rating["maximum"], 2);
    }
}
