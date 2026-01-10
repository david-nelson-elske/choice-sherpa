//! Cross-Cutting Tools - Tools available in all PrOACT components.
//!
//! These tools handle concerns that span components: uncertainty management,
//! revisit suggestions, user confirmations, document access, and notes.

use serde::{Deserialize, Serialize};

use crate::domain::conversation::tools::ToolDefinition;

// ═══════════════════════════════════════════════════════════════════════════
// Enums
// ═══════════════════════════════════════════════════════════════════════════

/// Priority level for revisit suggestions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisitPriority {
    /// Should revisit urgently
    High,
    /// Normal priority
    Medium,
    /// Low priority, optional
    Low,
}

/// Status of an uncertainty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UncertaintyStatus {
    /// Not yet addressed
    Open,
    /// Being investigated
    InProgress,
    /// Resolved with information
    Resolved,
    /// Accepted as unresolvable
    Accepted,
}

/// Type of document section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentSection {
    /// Problem frame section
    ProblemFrame,
    /// Objectives section
    Objectives,
    /// Alternatives section
    Alternatives,
    /// Consequences section
    Consequences,
    /// Tradeoffs section
    Tradeoffs,
    /// Recommendation section
    Recommendation,
    /// Decision quality section
    DecisionQuality,
    /// Notes section
    Notes,
    /// Full document
    Full,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Parameters - Uncertainty Management
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters for flagging an uncertainty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagUncertaintyParams {
    /// Description of the uncertainty
    pub description: String,
    /// Component where this was identified
    pub component: String,
    /// Related item ID (objective, alternative, etc.)
    pub related_to: Option<String>,
    /// Whether this is resolvable with more information
    pub resolvable: bool,
}

/// Parameters for resolving an uncertainty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveUncertaintyParams {
    /// ID of the uncertainty
    pub uncertainty_id: String,
    /// Resolution text
    pub resolution: String,
    /// How confidence was improved
    pub confidence_improvement: Option<String>,
}

/// Parameters for listing uncertainties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUncertaintiesParams {
    /// Filter by status
    pub status_filter: Option<UncertaintyStatus>,
    /// Filter by component
    pub component_filter: Option<String>,
    /// Only resolvable uncertainties
    pub resolvable_only: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Parameters - Revisit Suggestions
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters for suggesting a revisit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestRevisitParams {
    /// Component to revisit
    pub component: String,
    /// Reason for revisit
    pub reason: String,
    /// Priority level
    pub priority: RevisitPriority,
    /// Specific item to revisit
    pub item_id: Option<String>,
}

/// Parameters for getting pending revisits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPendingRevisitsParams {
    /// Filter by component
    pub component_filter: Option<String>,
    /// Minimum priority
    pub min_priority: Option<RevisitPriority>,
}

/// Parameters for dismissing a revisit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DismissRevisitParams {
    /// ID of the revisit suggestion
    pub revisit_id: String,
    /// Reason for dismissal
    pub reason: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Parameters - User Confirmation
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters for requesting confirmation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestConfirmationParams {
    /// What to confirm
    pub question: String,
    /// Context for the confirmation
    pub context: String,
    /// Default response if timeout
    pub default_response: Option<bool>,
    /// Available options (if not yes/no)
    pub options: Option<Vec<String>>,
}

/// Parameters for recording user choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordUserChoiceParams {
    /// ID of the confirmation request
    pub confirmation_id: String,
    /// User's choice
    pub choice: String,
    /// Any additional notes
    pub notes: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Parameters - Document Access
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters for getting a document section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDocumentSectionParams {
    /// Section to retrieve
    pub section: DocumentSection,
    /// Include completed items only
    pub completed_only: bool,
}

/// Parameters for getting document summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDocumentSummaryParams {
    /// Maximum length in characters
    pub max_length: Option<usize>,
    /// Sections to include
    pub include_sections: Option<Vec<DocumentSection>>,
}

/// Parameters for adding a note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddNoteParams {
    /// Note content
    pub content: String,
    /// Related component
    pub component: Option<String>,
    /// Related item ID
    pub related_to: Option<String>,
    /// Tags for the note
    pub tags: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Results - Uncertainty Management
// ═══════════════════════════════════════════════════════════════════════════

/// Result of flagging an uncertainty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagUncertaintyResult {
    /// ID of the created uncertainty
    pub uncertainty_id: String,
    /// Total open uncertainties
    pub open_count: usize,
    /// Total resolvable uncertainties
    pub resolvable_count: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// Result of resolving an uncertainty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveUncertaintyResult {
    /// Whether resolution succeeded
    pub success: bool,
    /// Remaining open uncertainties
    pub remaining_open: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// An uncertainty item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertaintyItem {
    /// Uncertainty ID
    pub id: String,
    /// Description
    pub description: String,
    /// Component
    pub component: String,
    /// Status
    pub status: UncertaintyStatus,
    /// Whether resolvable
    pub resolvable: bool,
}

/// Result of listing uncertainties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUncertaintiesResult {
    /// List of uncertainties
    pub uncertainties: Vec<UncertaintyItem>,
    /// Total count
    pub total_count: usize,
    /// Open count
    pub open_count: usize,
    /// Resolved count
    pub resolved_count: usize,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Results - Revisit Suggestions
// ═══════════════════════════════════════════════════════════════════════════

/// Result of suggesting a revisit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestRevisitResult {
    /// ID of the suggestion
    pub revisit_id: String,
    /// Total pending revisits
    pub pending_count: usize,
    /// High priority count
    pub high_priority_count: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

/// A revisit suggestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisitItem {
    /// Revisit ID
    pub id: String,
    /// Component to revisit
    pub component: String,
    /// Reason
    pub reason: String,
    /// Priority
    pub priority: RevisitPriority,
}

/// Result of getting pending revisits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPendingRevisitsResult {
    /// List of pending revisits
    pub revisits: Vec<RevisitItem>,
    /// Total pending
    pub total_pending: usize,
    /// By priority counts
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
}

/// Result of dismissing a revisit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DismissRevisitResult {
    /// Whether dismissal succeeded
    pub success: bool,
    /// Remaining pending
    pub remaining_pending: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Results - User Confirmation
// ═══════════════════════════════════════════════════════════════════════════

/// Result of requesting confirmation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestConfirmationResult {
    /// ID of the confirmation request
    pub confirmation_id: String,
    /// Status (pending, answered)
    pub status: String,
    /// Number of pending confirmations
    pub pending_count: usize,
}

/// Result of recording user choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordUserChoiceResult {
    /// Whether recording succeeded
    pub success: bool,
    /// The choice recorded
    pub choice: String,
    /// Remaining pending confirmations
    pub remaining_pending: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Results - Document Access
// ═══════════════════════════════════════════════════════════════════════════

/// Result of getting a document section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDocumentSectionResult {
    /// Section name
    pub section_name: String,
    /// Section content (JSON)
    pub content: serde_json::Value,
    /// Number of items
    pub item_count: usize,
    /// Last updated timestamp
    pub last_updated: Option<String>,
}

/// Result of getting document summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDocumentSummaryResult {
    /// Summary text
    pub summary: String,
    /// Summary length
    pub length: usize,
    /// Sections included
    pub sections_included: Vec<String>,
    /// Completeness percentage
    pub completeness: f64,
}

/// Result of adding a note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddNoteResult {
    /// ID of the note
    pub note_id: String,
    /// Total notes
    pub total_notes: usize,
    /// Whether the document was updated
    pub document_updated: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Definitions - Uncertainty Management
// ═══════════════════════════════════════════════════════════════════════════

/// Creates the flag_uncertainty tool definition.
pub fn flag_uncertainty_tool() -> ToolDefinition {
    ToolDefinition::new(
        "flag_uncertainty",
        "Flag an uncertainty that affects the decision. Use when user expresses doubt or mentions unknowns.",
        serde_json::json!({
            "type": "object",
            "required": ["description", "component", "resolvable"],
            "properties": {
                "description": {
                    "type": "string",
                    "description": "Description of the uncertainty"
                },
                "component": {
                    "type": "string",
                    "description": "Component where this was identified"
                },
                "related_to": {
                    "type": "string",
                    "description": "ID of related item (objective, alternative, etc.)"
                },
                "resolvable": {
                    "type": "boolean",
                    "description": "Whether this can be resolved with more information"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "uncertainty_id": { "type": "string" },
                "open_count": { "type": "integer" },
                "resolvable_count": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the resolve_uncertainty tool definition.
pub fn resolve_uncertainty_tool() -> ToolDefinition {
    ToolDefinition::new(
        "resolve_uncertainty",
        "Mark an uncertainty as resolved with new information.",
        serde_json::json!({
            "type": "object",
            "required": ["uncertainty_id", "resolution"],
            "properties": {
                "uncertainty_id": {
                    "type": "string",
                    "description": "ID of the uncertainty to resolve"
                },
                "resolution": {
                    "type": "string",
                    "description": "How the uncertainty was resolved"
                },
                "confidence_improvement": {
                    "type": "string",
                    "description": "How confidence was improved"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "remaining_open": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the list_uncertainties tool definition.
pub fn list_uncertainties_tool() -> ToolDefinition {
    ToolDefinition::new(
        "list_uncertainties",
        "List all uncertainties, optionally filtered by status or component.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "status_filter": {
                    "type": "string",
                    "enum": ["open", "in_progress", "resolved", "accepted"],
                    "description": "Filter by status"
                },
                "component_filter": {
                    "type": "string",
                    "description": "Filter by component"
                },
                "resolvable_only": {
                    "type": "boolean",
                    "default": false,
                    "description": "Only show resolvable uncertainties"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "uncertainties": { "type": "array" },
                "total_count": { "type": "integer" },
                "open_count": { "type": "integer" },
                "resolved_count": { "type": "integer" }
            }
        }),
    )
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Definitions - Revisit Suggestions
// ═══════════════════════════════════════════════════════════════════════════

/// Creates the suggest_revisit tool definition.
pub fn suggest_revisit_tool() -> ToolDefinition {
    ToolDefinition::new(
        "suggest_revisit",
        "Suggest revisiting a previous component. Use when new information affects earlier work.",
        serde_json::json!({
            "type": "object",
            "required": ["component", "reason", "priority"],
            "properties": {
                "component": {
                    "type": "string",
                    "description": "Component to revisit"
                },
                "reason": {
                    "type": "string",
                    "description": "Why revisit is suggested"
                },
                "priority": {
                    "type": "string",
                    "enum": ["high", "medium", "low"],
                    "description": "Priority level"
                },
                "item_id": {
                    "type": "string",
                    "description": "Specific item to revisit"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "revisit_id": { "type": "string" },
                "pending_count": { "type": "integer" },
                "high_priority_count": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Creates the get_pending_revisits tool definition.
pub fn get_pending_revisits_tool() -> ToolDefinition {
    ToolDefinition::new(
        "get_pending_revisits",
        "Get list of pending revisit suggestions.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "component_filter": {
                    "type": "string",
                    "description": "Filter by component"
                },
                "min_priority": {
                    "type": "string",
                    "enum": ["high", "medium", "low"],
                    "description": "Minimum priority to include"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "revisits": { "type": "array" },
                "total_pending": { "type": "integer" },
                "high_count": { "type": "integer" },
                "medium_count": { "type": "integer" },
                "low_count": { "type": "integer" }
            }
        }),
    )
}

/// Creates the dismiss_revisit tool definition.
pub fn dismiss_revisit_tool() -> ToolDefinition {
    ToolDefinition::new(
        "dismiss_revisit",
        "Dismiss a revisit suggestion as no longer needed.",
        serde_json::json!({
            "type": "object",
            "required": ["revisit_id", "reason"],
            "properties": {
                "revisit_id": {
                    "type": "string",
                    "description": "ID of the revisit to dismiss"
                },
                "reason": {
                    "type": "string",
                    "description": "Reason for dismissal"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "remaining_pending": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Definitions - User Confirmation
// ═══════════════════════════════════════════════════════════════════════════

/// Creates the request_confirmation tool definition.
pub fn request_confirmation_tool() -> ToolDefinition {
    ToolDefinition::new(
        "request_confirmation",
        "Request explicit confirmation from user before proceeding.",
        serde_json::json!({
            "type": "object",
            "required": ["question", "context"],
            "properties": {
                "question": {
                    "type": "string",
                    "description": "What to confirm"
                },
                "context": {
                    "type": "string",
                    "description": "Context for the confirmation"
                },
                "default_response": {
                    "type": "boolean",
                    "description": "Default if timeout"
                },
                "options": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Available options (if not yes/no)"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "confirmation_id": { "type": "string" },
                "status": { "type": "string" },
                "pending_count": { "type": "integer" }
            }
        }),
    )
}

/// Creates the record_user_choice tool definition.
pub fn record_user_choice_tool() -> ToolDefinition {
    ToolDefinition::new(
        "record_user_choice",
        "Record user's response to a confirmation request.",
        serde_json::json!({
            "type": "object",
            "required": ["confirmation_id", "choice"],
            "properties": {
                "confirmation_id": {
                    "type": "string",
                    "description": "ID of the confirmation request"
                },
                "choice": {
                    "type": "string",
                    "description": "User's choice"
                },
                "notes": {
                    "type": "string",
                    "description": "Additional notes"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "choice": { "type": "string" },
                "remaining_pending": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool Definitions - Document Access
// ═══════════════════════════════════════════════════════════════════════════

/// Creates the get_document_section tool definition.
pub fn get_document_section_tool() -> ToolDefinition {
    ToolDefinition::new(
        "get_document_section",
        "Retrieve a specific section of the decision document.",
        serde_json::json!({
            "type": "object",
            "required": ["section"],
            "properties": {
                "section": {
                    "type": "string",
                    "enum": [
                        "problem_frame",
                        "objectives",
                        "alternatives",
                        "consequences",
                        "tradeoffs",
                        "recommendation",
                        "decision_quality",
                        "notes",
                        "full"
                    ],
                    "description": "Section to retrieve"
                },
                "completed_only": {
                    "type": "boolean",
                    "default": false,
                    "description": "Only include completed items"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "section_name": { "type": "string" },
                "content": { "type": "object" },
                "item_count": { "type": "integer" },
                "last_updated": { "type": "string" }
            }
        }),
    )
}

/// Creates the get_document_summary tool definition.
pub fn get_document_summary_tool() -> ToolDefinition {
    ToolDefinition::new(
        "get_document_summary",
        "Get a text summary of the decision document.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "max_length": {
                    "type": "integer",
                    "description": "Maximum length in characters"
                },
                "include_sections": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Sections to include in summary"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "summary": { "type": "string" },
                "length": { "type": "integer" },
                "sections_included": { "type": "array", "items": { "type": "string" } },
                "completeness": { "type": "number" }
            }
        }),
    )
}

/// Creates the add_note tool definition.
pub fn add_note_tool() -> ToolDefinition {
    ToolDefinition::new(
        "add_note",
        "Add a note to the decision document. Use for observations, reminders, or context.",
        serde_json::json!({
            "type": "object",
            "required": ["content", "tags"],
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Note content"
                },
                "component": {
                    "type": "string",
                    "description": "Related component"
                },
                "related_to": {
                    "type": "string",
                    "description": "Related item ID"
                },
                "tags": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Tags for the note"
                }
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "note_id": { "type": "string" },
                "total_notes": { "type": "integer" },
                "document_updated": { "type": "boolean" }
            }
        }),
    )
}

/// Returns all Cross-Cutting tool definitions.
pub fn all_cross_cutting_tools() -> Vec<ToolDefinition> {
    vec![
        // Uncertainty management
        flag_uncertainty_tool(),
        resolve_uncertainty_tool(),
        list_uncertainties_tool(),
        // Revisit suggestions
        suggest_revisit_tool(),
        get_pending_revisits_tool(),
        dismiss_revisit_tool(),
        // User confirmation
        request_confirmation_tool(),
        record_user_choice_tool(),
        // Document access
        get_document_section_tool(),
        get_document_summary_tool(),
        add_note_tool(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revisit_priority_serializes_to_snake_case() {
        assert_eq!(serde_json::to_string(&RevisitPriority::High).unwrap(), "\"high\"");
        assert_eq!(serde_json::to_string(&RevisitPriority::Low).unwrap(), "\"low\"");
    }

    #[test]
    fn uncertainty_status_serializes() {
        assert_eq!(serde_json::to_string(&UncertaintyStatus::Open).unwrap(), "\"open\"");
        assert_eq!(serde_json::to_string(&UncertaintyStatus::InProgress).unwrap(), "\"in_progress\"");
        assert_eq!(serde_json::to_string(&UncertaintyStatus::Resolved).unwrap(), "\"resolved\"");
    }

    #[test]
    fn document_section_serializes() {
        assert_eq!(serde_json::to_string(&DocumentSection::ProblemFrame).unwrap(), "\"problem_frame\"");
        assert_eq!(serde_json::to_string(&DocumentSection::DecisionQuality).unwrap(), "\"decision_quality\"");
    }

    #[test]
    fn flag_uncertainty_params_serializes() {
        let params = FlagUncertaintyParams {
            description: "Market conditions unknown".to_string(),
            component: "alternatives".to_string(),
            related_to: Some("alt_b".to_string()),
            resolvable: true,
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["resolvable"], true);
        assert!(json["related_to"].is_string());
    }

    #[test]
    fn all_cross_cutting_tools_returns_eleven_tools() {
        let tools = all_cross_cutting_tools();
        assert_eq!(tools.len(), 11);
    }

    #[test]
    fn tool_names_are_distinct() {
        let tools = all_cross_cutting_tools();
        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        let unique_names: std::collections::HashSet<&str> = names.iter().copied().collect();
        assert_eq!(names.len(), unique_names.len());
    }

    #[test]
    fn get_document_section_has_section_enum() {
        let tool = get_document_section_tool();
        let schema = tool.parameters_schema();
        let section = &schema["properties"]["section"];
        let enum_values = section["enum"].as_array().unwrap();
        assert_eq!(enum_values.len(), 9);
    }

    #[test]
    fn list_uncertainties_has_status_filter_enum() {
        let tool = list_uncertainties_tool();
        let schema = tool.parameters_schema();
        let status_filter = &schema["properties"]["status_filter"];
        let enum_values = status_filter["enum"].as_array().unwrap();
        assert_eq!(enum_values.len(), 4);
    }
}
