//! Decision Quality Tools - Tools for rating decision quality elements.
//!
//! Decision Quality uses 7 elements rated 0-100%. The overall DQ score
//! equals the MINIMUM of all element scores (chain is as strong as weakest link).
//!
//! The 7 DQ Elements:
//! 1. Appropriate Frame - Is the decision well-defined?
//! 2. Clear Values - Are objectives clearly articulated?
//! 3. Creative Alternatives - Are options comprehensive?
//! 4. Meaningful Information - Is data reliable and relevant?
//! 5. Sound Reasoning - Is analysis logically sound?
//! 6. Commitment to Action - Is there willingness to decide?
//! 7. Right People Involved - Are stakeholders included?

use serde::{Deserialize, Serialize};

use crate::domain::conversation::tools::ToolDefinition;

// ═══════════════════════════════════════════════════════════════════════════
// Enums
// ═══════════════════════════════════════════════════════════════════════════

/// The 7 Decision Quality elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DQElement {
    /// Is the decision well-defined?
    AppropriateFrame,
    /// Are objectives clearly articulated?
    ClearValues,
    /// Are options comprehensive?
    CreativeAlternatives,
    /// Is data reliable and relevant?
    MeaningfulInformation,
    /// Is analysis logically sound?
    SoundReasoning,
    /// Is there willingness to decide?
    CommitmentToAction,
    /// Are stakeholders included?
    RightPeopleInvolved,
}

impl DQElement {
    /// Returns all DQ elements.
    pub fn all() -> Vec<DQElement> {
        vec![
            Self::AppropriateFrame,
            Self::ClearValues,
            Self::CreativeAlternatives,
            Self::MeaningfulInformation,
            Self::SoundReasoning,
            Self::CommitmentToAction,
            Self::RightPeopleInvolved,
        ]
    }

    /// Returns a human-readable name for the element.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::AppropriateFrame => "Appropriate Frame",
            Self::ClearValues => "Clear Values",
            Self::CreativeAlternatives => "Creative Alternatives",
            Self::MeaningfulInformation => "Meaningful Information",
            Self::SoundReasoning => "Sound Reasoning",
            Self::CommitmentToAction => "Commitment to Action",
            Self::RightPeopleInvolved => "Right People Involved",
        }
    }

    /// Returns a question for assessing this element.
    pub fn assessment_question(&self) -> &'static str {
        match self {
            Self::AppropriateFrame => "Is the decision well-defined with clear scope and boundaries?",
            Self::ClearValues => "Are objectives clearly articulated with meaningful measures?",
            Self::CreativeAlternatives => "Have we explored a comprehensive range of options?",
            Self::MeaningfulInformation => "Is our information reliable, relevant, and sufficient?",
            Self::SoundReasoning => "Is the analysis logical and free from cognitive biases?",
            Self::CommitmentToAction => "Are we ready and willing to act on the decision?",
            Self::RightPeopleInvolved => "Are the right stakeholders engaged in the process?",
        }
    }
}

/// Category of improvement suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImprovementCategory {
    /// Improve information gathering
    Information,
    /// Improve analysis
    Analysis,
    /// Improve stakeholder engagement
    Engagement,
    /// Improve process
    Process,
    /// Improve framing
    Framing,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Parameters
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters for rating a DQ element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateDQElementParams {
    /// The element to rate
    pub element: DQElement,
    /// Rating from 0 to 100
    pub rating: u8,
    /// Justification for the rating
    pub justification: String,
    /// Evidence supporting the rating
    pub evidence: Vec<String>,
}

/// Parameters for adding a quality improvement suggestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddQualityImprovementParams {
    /// Which element this would improve
    pub element: DQElement,
    /// The improvement suggestion
    pub suggestion: String,
    /// Category of improvement
    pub category: ImprovementCategory,
    /// Estimated effort (low, medium, high)
    pub effort: String,
    /// Potential rating improvement
    pub potential_improvement: u8,
}

/// Parameters for calculating overall DQ.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculateOverallDQParams {
    /// Whether to include detailed breakdown
    pub include_breakdown: bool,
}

/// Parameters for comparing DQ across versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareDQVersionsParams {
    /// Previous version timestamp or ID
    pub previous_version: String,
}

/// Parameters for getting element details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetElementDetailsParams {
    /// The element to get details for
    pub element: DQElement,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Results
// ═══════════════════════════════════════════════════════════════════════════

/// Result of rating a DQ element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateDQElementResult {
    /// Whether the rating was applied
    pub success: bool,
    /// Element name
    pub element_name: String,
    /// Applied rating
    pub rating: u8,
    /// Rating category (poor, fair, good, excellent)
    pub category: String,
    /// Whether this is now the weakest link
    pub is_weakest_link: bool,
    /// Current overall DQ
    pub current_overall_dq: u8,
    /// Number of elements rated
    pub elements_rated: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of adding a quality improvement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddQualityImprovementResult {
    /// ID of the improvement
    pub improvement_id: String,
    /// Total improvements suggested
    pub total_improvements: usize,
    /// Improvements for this element
    pub element_improvements: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Element score with details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementScore {
    /// Element identifier
    pub element: DQElement,
    /// Display name
    pub name: String,
    /// Rating (0-100)
    pub rating: u8,
    /// Rating category
    pub category: String,
    /// Whether this is the weakest link
    pub is_weakest: bool,
    /// Number of improvement suggestions
    pub improvement_count: usize,
}

/// Result of calculating overall DQ.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculateOverallDQResult {
    /// Overall DQ score (minimum of all elements)
    pub overall_dq: u8,
    /// Overall category
    pub overall_category: String,
    /// Weakest element
    pub weakest_element: String,
    /// Strongest element
    pub strongest_element: String,
    /// Average across elements
    pub average_rating: f64,
    /// Detailed breakdown (if requested)
    pub breakdown: Option<Vec<ElementScore>>,
    /// Count of unrated elements
    pub unrated_count: usize,
}

/// Result of comparing DQ versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareDQVersionsResult {
    /// Previous overall DQ
    pub previous_dq: u8,
    /// Current overall DQ
    pub current_dq: u8,
    /// Change in DQ
    pub change: i16,
    /// Elements that improved
    pub improved_elements: Vec<String>,
    /// Elements that declined
    pub declined_elements: Vec<String>,
    /// Elements unchanged
    pub unchanged_elements: Vec<String>,
}

/// Result of getting element details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetElementDetailsResult {
    /// Element name
    pub element_name: String,
    /// Assessment question
    pub assessment_question: String,
    /// Current rating
    pub current_rating: Option<u8>,
    /// Justification
    pub justification: Option<String>,
    /// Evidence
    pub evidence: Vec<String>,
    /// Improvement suggestions
    pub improvements: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Definitions
// ═══════════════════════════════════════════════════════════════════════════

/// Creates the rate_dq_element tool definition.
pub fn rate_dq_element_tool() -> ToolDefinition {
    ToolDefinition::new(
        "rate_dq_element",
        "Rate a Decision Quality element (0-100%). Overall DQ equals the minimum score.",
        serde_json::json!({
            "type": "object",
            "required": ["element", "rating", "justification", "evidence"],
            "properties": {
                "element": {
                    "type": "string",
                    "enum": [
                        "appropriate_frame",
                        "clear_values",
                        "creative_alternatives",
                        "meaningful_information",
                        "sound_reasoning",
                        "commitment_to_action",
                        "right_people_involved"
                    ],
                    "description": "The DQ element to rate"
                },
                "rating": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 100,
                    "description": "Rating from 0 (very poor) to 100 (excellent)"
                },
                "justification": {
                    "type": "string",
                    "description": "Justification for this rating"
                },
                "evidence": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Evidence supporting this rating"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "element_name": { "type": "string" },
                "rating": { "type": "integer" },
                "category": { "type": "string" },
                "is_weakest_link": { "type": "boolean" },
                "current_overall_dq": { "type": "integer" },
                "elements_rated": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the add_quality_improvement tool definition.
pub fn add_quality_improvement_tool() -> ToolDefinition {
    ToolDefinition::new(
        "add_quality_improvement",
        "Suggest an improvement that could increase decision quality.",
        serde_json::json!({
            "type": "object",
            "required": ["element", "suggestion", "category", "effort", "potential_improvement"],
            "properties": {
                "element": {
                    "type": "string",
                    "enum": [
                        "appropriate_frame",
                        "clear_values",
                        "creative_alternatives",
                        "meaningful_information",
                        "sound_reasoning",
                        "commitment_to_action",
                        "right_people_involved"
                    ],
                    "description": "The DQ element this would improve"
                },
                "suggestion": {
                    "type": "string",
                    "description": "The improvement suggestion"
                },
                "category": {
                    "type": "string",
                    "enum": ["information", "analysis", "engagement", "process", "framing"],
                    "description": "Category of improvement"
                },
                "effort": {
                    "type": "string",
                    "enum": ["low", "medium", "high"],
                    "description": "Estimated effort to implement"
                },
                "potential_improvement": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 50,
                    "description": "Potential rating improvement (0-50 points)"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "improvement_id": { "type": "string" },
                "total_improvements": { "type": "integer" },
                "element_improvements": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the calculate_overall_dq tool definition.
pub fn calculate_overall_dq_tool() -> ToolDefinition {
    ToolDefinition::new(
        "calculate_overall_dq",
        "Calculate the overall Decision Quality score. Returns the minimum of all element scores.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "include_breakdown": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include detailed breakdown by element"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "overall_dq": { "type": "integer" },
                "overall_category": { "type": "string" },
                "weakest_element": { "type": "string" },
                "strongest_element": { "type": "string" },
                "average_rating": { "type": "number" },
                "breakdown": {
                    "type": "array",
                    "items": { "type": "object" }
                },
                "unrated_count": { "type": "integer" }
            }
        }),
    )
}

/// Creates the compare_dq_versions tool definition.
pub fn compare_dq_versions_tool() -> ToolDefinition {
    ToolDefinition::new(
        "compare_dq_versions",
        "Compare Decision Quality scores across versions. Shows improvement or decline.",
        serde_json::json!({
            "type": "object",
            "required": ["previous_version"],
            "properties": {
                "previous_version": {
                    "type": "string",
                    "description": "Previous version timestamp or ID to compare against"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "previous_dq": { "type": "integer" },
                "current_dq": { "type": "integer" },
                "change": { "type": "integer" },
                "improved_elements": { "type": "array", "items": { "type": "string" } },
                "declined_elements": { "type": "array", "items": { "type": "string" } },
                "unchanged_elements": { "type": "array", "items": { "type": "string" } }
            }
        }),
    )
}

/// Creates the get_element_details tool definition.
pub fn get_element_details_tool() -> ToolDefinition {
    ToolDefinition::new(
        "get_element_details",
        "Get detailed information about a specific DQ element.",
        serde_json::json!({
            "type": "object",
            "required": ["element"],
            "properties": {
                "element": {
                    "type": "string",
                    "enum": [
                        "appropriate_frame",
                        "clear_values",
                        "creative_alternatives",
                        "meaningful_information",
                        "sound_reasoning",
                        "commitment_to_action",
                        "right_people_involved"
                    ],
                    "description": "The DQ element to get details for"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "element_name": { "type": "string" },
                "assessment_question": { "type": "string" },
                "current_rating": { "type": "integer" },
                "justification": { "type": "string" },
                "evidence": { "type": "array", "items": { "type": "string" } },
                "improvements": { "type": "array", "items": { "type": "string" } }
            }
        }),
    )
}

/// Returns all Decision Quality tool definitions.
pub fn all_decision_quality_tools() -> Vec<ToolDefinition> {
    vec![
        rate_dq_element_tool(),
        add_quality_improvement_tool(),
        calculate_overall_dq_tool(),
        compare_dq_versions_tool(),
        get_element_details_tool(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dq_element_all_returns_seven_elements() {
        assert_eq!(DQElement::all().len(), 7);
    }

    #[test]
    fn dq_element_serializes_to_snake_case() {
        assert_eq!(serde_json::to_string(&DQElement::AppropriateFrame).unwrap(), "\"appropriate_frame\"");
        assert_eq!(serde_json::to_string(&DQElement::ClearValues).unwrap(), "\"clear_values\"");
        assert_eq!(serde_json::to_string(&DQElement::RightPeopleInvolved).unwrap(), "\"right_people_involved\"");
    }

    #[test]
    fn dq_element_display_names() {
        assert_eq!(DQElement::AppropriateFrame.display_name(), "Appropriate Frame");
        assert_eq!(DQElement::SoundReasoning.display_name(), "Sound Reasoning");
    }

    #[test]
    fn dq_element_has_assessment_questions() {
        for element in DQElement::all() {
            assert!(!element.assessment_question().is_empty());
        }
    }

    #[test]
    fn improvement_category_serializes() {
        assert_eq!(serde_json::to_string(&ImprovementCategory::Information).unwrap(), "\"information\"");
        assert_eq!(serde_json::to_string(&ImprovementCategory::Engagement).unwrap(), "\"engagement\"");
    }

    #[test]
    fn rate_dq_element_params_serializes() {
        let params = RateDQElementParams {
            element: DQElement::ClearValues,
            rating: 75,
            justification: "Objectives are well-defined".to_string(),
            evidence: vec!["5 fundamental objectives identified".to_string()],
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["element"], "clear_values");
        assert_eq!(json["rating"], 75);
    }

    #[test]
    fn all_decision_quality_tools_returns_five_tools() {
        let tools = all_decision_quality_tools();
        assert_eq!(tools.len(), 5);
    }

    #[test]
    fn rate_dq_element_has_rating_constraints() {
        let tool = rate_dq_element_tool();
        let schema = tool.parameters_schema();
        let rating = &schema["properties"]["rating"];
        assert_eq!(rating["minimum"], 0);
        assert_eq!(rating["maximum"], 100);
    }

    #[test]
    fn add_quality_improvement_has_effort_enum() {
        let tool = add_quality_improvement_tool();
        let schema = tool.parameters_schema();
        let effort = &schema["properties"]["effort"];
        let enum_values = effort["enum"].as_array().unwrap();
        assert_eq!(enum_values.len(), 3);
    }
}
