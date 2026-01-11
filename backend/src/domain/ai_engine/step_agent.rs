//! Step Agent Specifications
//!
//! Defines behavior specifications for each PrOACT component.
//! Each agent has a role, objectives, techniques, output schema, and transition rules.

use serde::{Deserialize, Serialize};

use crate::domain::foundation::ComponentType;

/// Specification for a PrOACT step agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepAgentSpec {
    pub component: ComponentType,
    pub role: String,
    pub objectives: Vec<String>,
    pub techniques: Vec<String>,
    pub output_schema: OutputSchema,
    pub transitions: TransitionRules,
}

/// Rules for when a step can transition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransitionRules {
    pub min_turns: u32,
    pub required_outputs: Vec<String>,
    pub completion_signals: Vec<String>,
}

/// Schema for structured output
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutputSchema {
    pub schema_version: String,
    pub fields: Vec<SchemaField>,
}

/// A field in an output schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaField {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub description: String,
}

/// Field types in output schemas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

/// Predefined step agent specifications for all 8 PrOACT components
pub mod agents {
    use super::*;

    /// Issue Raising agent specification
    pub fn issue_raising() -> StepAgentSpec {
        StepAgentSpec {
            component: ComponentType::IssueRaising,
            role: "Help user categorize their initial thoughts into decisions, objectives, and uncertainties".to_string(),
            objectives: vec![
                "Identify all potential decisions".to_string(),
                "Distinguish between decisions and constraints".to_string(),
                "Surface hidden objectives".to_string(),
                "Catalog key uncertainties".to_string(),
            ],
            techniques: vec![
                "Ask open-ended questions about the situation".to_string(),
                "Probe for multiple decision points".to_string(),
                "Distinguish 'what we control' from 'what we don't'".to_string(),
            ],
            output_schema: OutputSchema {
                schema_version: "1.0".to_string(),
                fields: vec![
                    SchemaField {
                        name: "decisions".to_string(),
                        field_type: FieldType::Array,
                        required: true,
                        description: "List of potential decisions identified".to_string(),
                    },
                    SchemaField {
                        name: "objectives".to_string(),
                        field_type: FieldType::Array,
                        required: true,
                        description: "Initial objectives mentioned".to_string(),
                    },
                    SchemaField {
                        name: "uncertainties".to_string(),
                        field_type: FieldType::Array,
                        required: true,
                        description: "Key uncertainties that could affect the decision".to_string(),
                    },
                ],
            },
            transitions: TransitionRules {
                min_turns: 2,
                required_outputs: vec!["decisions".to_string()],
                completion_signals: vec!["ready to frame".to_string(), "move to problem".to_string()],
            },
        }
    }

    /// Problem Frame agent specification
    pub fn problem_frame() -> StepAgentSpec {
        StepAgentSpec {
            component: ComponentType::ProblemFrame,
            role: "Help user define decision architecture, constraints, and stakeholders".to_string(),
            objectives: vec![
                "Define the decision boundary".to_string(),
                "Identify constraints (time, budget, policy)".to_string(),
                "Map stakeholders and their interests".to_string(),
                "Clarify what makes a 'good' decision in this context".to_string(),
            ],
            techniques: vec![
                "Ask about deadlines and milestones".to_string(),
                "Probe for budget and resource constraints".to_string(),
                "Identify who cares about this decision and why".to_string(),
            ],
            output_schema: OutputSchema {
                schema_version: "1.0".to_string(),
                fields: vec![
                    SchemaField {
                        name: "decision_statement".to_string(),
                        field_type: FieldType::String,
                        required: true,
                        description: "Clear statement of the decision to be made".to_string(),
                    },
                    SchemaField {
                        name: "constraints".to_string(),
                        field_type: FieldType::Array,
                        required: true,
                        description: "Time, budget, policy, or other constraints".to_string(),
                    },
                    SchemaField {
                        name: "stakeholders".to_string(),
                        field_type: FieldType::Array,
                        required: true,
                        description: "People or groups affected by this decision".to_string(),
                    },
                ],
            },
            transitions: TransitionRules {
                min_turns: 2,
                required_outputs: vec!["decision_statement".to_string()],
                completion_signals: vec!["ready for objectives".to_string(), "frame is clear".to_string()],
            },
        }
    }

    /// Objectives agent specification
    pub fn objectives() -> StepAgentSpec {
        StepAgentSpec {
            component: ComponentType::Objectives,
            role: "Help user identify fundamental vs means objectives with measures".to_string(),
            objectives: vec![
                "Distinguish fundamental from means objectives".to_string(),
                "Define measures for each objective".to_string(),
                "Identify objective hierarchy".to_string(),
                "Surface hidden objectives".to_string(),
            ],
            techniques: vec![
                "Ask 'why is that important?' to find fundamental objectives".to_string(),
                "Ask 'how would you measure that?' for each objective".to_string(),
                "Look for conflicts between objectives".to_string(),
            ],
            output_schema: OutputSchema {
                schema_version: "1.0".to_string(),
                fields: vec![
                    SchemaField {
                        name: "fundamental_objectives".to_string(),
                        field_type: FieldType::Array,
                        required: true,
                        description: "Core objectives that matter for their own sake".to_string(),
                    },
                    SchemaField {
                        name: "means_objectives".to_string(),
                        field_type: FieldType::Array,
                        required: true,
                        description: "Objectives that serve other objectives".to_string(),
                    },
                    SchemaField {
                        name: "measures".to_string(),
                        field_type: FieldType::Object,
                        required: true,
                        description: "How each objective will be measured".to_string(),
                    },
                ],
            },
            transitions: TransitionRules {
                min_turns: 3,
                required_outputs: vec!["fundamental_objectives".to_string()],
                completion_signals: vec!["objectives are clear".to_string(), "ready for alternatives".to_string()],
            },
        }
    }

    /// Alternatives agent specification
    pub fn alternatives() -> StepAgentSpec {
        StepAgentSpec {
            component: ComponentType::Alternatives,
            role: "Help user capture options, strategy tables, and status quo baseline".to_string(),
            objectives: vec![
                "Generate creative alternatives".to_string(),
                "Include status quo as baseline".to_string(),
                "Build strategy table if complex".to_string(),
                "Avoid premature evaluation".to_string(),
            ],
            techniques: vec![
                "Brainstorm without judging".to_string(),
                "Ask 'what if we did nothing?'".to_string(),
                "Combine elements to form new alternatives".to_string(),
            ],
            output_schema: OutputSchema {
                schema_version: "1.0".to_string(),
                fields: vec![
                    SchemaField {
                        name: "alternatives".to_string(),
                        field_type: FieldType::Array,
                        required: true,
                        description: "List of alternatives under consideration".to_string(),
                    },
                    SchemaField {
                        name: "status_quo".to_string(),
                        field_type: FieldType::String,
                        required: true,
                        description: "Description of the 'do nothing' option".to_string(),
                    },
                    SchemaField {
                        name: "strategy_table".to_string(),
                        field_type: FieldType::Object,
                        required: false,
                        description: "Matrix of decisions and choices (if complex)".to_string(),
                    },
                ],
            },
            transitions: TransitionRules {
                min_turns: 2,
                required_outputs: vec!["alternatives".to_string(), "status_quo".to_string()],
                completion_signals: vec!["ready to evaluate".to_string(), "alternatives complete".to_string()],
            },
        }
    }

    /// Consequences agent specification
    pub fn consequences() -> StepAgentSpec {
        StepAgentSpec {
            component: ComponentType::Consequences,
            role: "Help user build consequence table with Pugh ratings (-2 to +2)".to_string(),
            objectives: vec![
                "Rate each alternative against each objective".to_string(),
                "Use Pugh scale (-2 to +2)".to_string(),
                "Document reasoning for ratings".to_string(),
                "Identify information gaps".to_string(),
            ],
            techniques: vec![
                "Go through alternatives one by one".to_string(),
                "For each, rate against all objectives".to_string(),
                "Ask for evidence supporting each rating".to_string(),
            ],
            output_schema: OutputSchema {
                schema_version: "1.0".to_string(),
                fields: vec![
                    SchemaField {
                        name: "consequence_table".to_string(),
                        field_type: FieldType::Object,
                        required: true,
                        description: "Matrix of alternatives x objectives with Pugh ratings".to_string(),
                    },
                    SchemaField {
                        name: "rating_rationales".to_string(),
                        field_type: FieldType::Object,
                        required: true,
                        description: "Explanation for each rating".to_string(),
                    },
                    SchemaField {
                        name: "information_gaps".to_string(),
                        field_type: FieldType::Array,
                        required: false,
                        description: "Uncertainties that need more research".to_string(),
                    },
                ],
            },
            transitions: TransitionRules {
                min_turns: 3,
                required_outputs: vec!["consequence_table".to_string()],
                completion_signals: vec!["ratings complete".to_string(), "ready for tradeoffs".to_string()],
            },
        }
    }

    /// Tradeoffs agent specification
    pub fn tradeoffs() -> StepAgentSpec {
        StepAgentSpec {
            component: ComponentType::Tradeoffs,
            role: "Help user surface dominated alternatives, tensions, and irrelevant objectives".to_string(),
            objectives: vec![
                "Identify dominated alternatives (can be eliminated)".to_string(),
                "Highlight key tradeoffs (tensions between objectives)".to_string(),
                "Find irrelevant objectives (all alternatives score same)".to_string(),
                "Simplify the decision space".to_string(),
            ],
            techniques: vec![
                "Compare alternatives pairwise".to_string(),
                "Look for alternatives that are worse on all objectives".to_string(),
                "Identify objectives where all alternatives are equal".to_string(),
            ],
            output_schema: OutputSchema {
                schema_version: "1.0".to_string(),
                fields: vec![
                    SchemaField {
                        name: "dominated_alternatives".to_string(),
                        field_type: FieldType::Array,
                        required: true,
                        description: "Alternatives that can be eliminated".to_string(),
                    },
                    SchemaField {
                        name: "key_tradeoffs".to_string(),
                        field_type: FieldType::Array,
                        required: true,
                        description: "Important tensions between objectives".to_string(),
                    },
                    SchemaField {
                        name: "irrelevant_objectives".to_string(),
                        field_type: FieldType::Array,
                        required: false,
                        description: "Objectives that don't differentiate alternatives".to_string(),
                    },
                ],
            },
            transitions: TransitionRules {
                min_turns: 2,
                required_outputs: vec!["key_tradeoffs".to_string()],
                completion_signals: vec!["tradeoffs clear".to_string(), "ready for recommendation".to_string()],
            },
        }
    }

    /// Recommendation agent specification
    pub fn recommendation() -> StepAgentSpec {
        StepAgentSpec {
            component: ComponentType::Recommendation,
            role: "Help user synthesize analysis (not deciding for them)".to_string(),
            objectives: vec![
                "Summarize key insights from analysis".to_string(),
                "Highlight which alternatives survived tradeoff analysis".to_string(),
                "Identify remaining uncertainties".to_string(),
                "Support user's own decision-making (not prescribe)".to_string(),
            ],
            techniques: vec![
                "Recap the journey through PrOACT".to_string(),
                "Present findings, not recommendations".to_string(),
                "Ask 'what stands out to you from this analysis?'".to_string(),
            ],
            output_schema: OutputSchema {
                schema_version: "1.0".to_string(),
                fields: vec![
                    SchemaField {
                        name: "synthesis".to_string(),
                        field_type: FieldType::String,
                        required: true,
                        description: "Summary of the analysis and key insights".to_string(),
                    },
                    SchemaField {
                        name: "surviving_alternatives".to_string(),
                        field_type: FieldType::Array,
                        required: true,
                        description: "Alternatives that remain viable".to_string(),
                    },
                    SchemaField {
                        name: "remaining_questions".to_string(),
                        field_type: FieldType::Array,
                        required: false,
                        description: "Unresolved questions or uncertainties".to_string(),
                    },
                ],
            },
            transitions: TransitionRules {
                min_turns: 2,
                required_outputs: vec!["synthesis".to_string()],
                completion_signals: vec!["synthesis complete".to_string(), "ready for decision quality".to_string()],
            },
        }
    }

    /// Decision Quality agent specification
    pub fn decision_quality() -> StepAgentSpec {
        StepAgentSpec {
            component: ComponentType::DecisionQuality,
            role: "Help user rate 7 elements of decision quality (0-100%)".to_string(),
            objectives: vec![
                "Rate each DQ element (Frame, Alternatives, Information, Values, Reasoning, Commitment, Follow-through)".to_string(),
                "Overall DQ = minimum of all element scores".to_string(),
                "100% = 'good decision at time made, regardless of outcome'".to_string(),
                "Identify areas for improvement".to_string(),
            ],
            techniques: vec![
                "Go through each element one by one".to_string(),
                "Ask for evidence of quality in each area".to_string(),
                "Remind that this rates the process, not the outcome".to_string(),
            ],
            output_schema: OutputSchema {
                schema_version: "1.0".to_string(),
                fields: vec![
                    SchemaField {
                        name: "element_scores".to_string(),
                        field_type: FieldType::Object,
                        required: true,
                        description: "Scores (0-100) for each of 7 DQ elements".to_string(),
                    },
                    SchemaField {
                        name: "overall_score".to_string(),
                        field_type: FieldType::Number,
                        required: true,
                        description: "Minimum of all element scores".to_string(),
                    },
                    SchemaField {
                        name: "improvement_areas".to_string(),
                        field_type: FieldType::Array,
                        required: false,
                        description: "Elements scoring below threshold".to_string(),
                    },
                ],
            },
            transitions: TransitionRules {
                min_turns: 2,
                required_outputs: vec!["element_scores".to_string(), "overall_score".to_string()],
                completion_signals: vec!["dq rating complete".to_string(), "cycle complete".to_string()],
            },
        }
    }

    /// Get all agent specifications
    pub fn all() -> Vec<StepAgentSpec> {
        vec![
            issue_raising(),
            problem_frame(),
            objectives(),
            alternatives(),
            consequences(),
            tradeoffs(),
            recommendation(),
            decision_quality(),
        ]
    }

    /// Get agent specification by component type
    pub fn get(component: ComponentType) -> Option<StepAgentSpec> {
        match component {
            ComponentType::IssueRaising => Some(issue_raising()),
            ComponentType::ProblemFrame => Some(problem_frame()),
            ComponentType::Objectives => Some(objectives()),
            ComponentType::Alternatives => Some(alternatives()),
            ComponentType::Consequences => Some(consequences()),
            ComponentType::Tradeoffs => Some(tradeoffs()),
            ComponentType::Recommendation => Some(recommendation()),
            ComponentType::DecisionQuality => Some(decision_quality()),
            ComponentType::NotesNextSteps => None, // No agent for notes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_agent_spec_all_components_defined() {
        let specs = agents::all();
        assert_eq!(specs.len(), 8); // All 8 PrOACT components (excluding NotesNextSteps)
    }

    #[test]
    fn test_step_agent_spec_has_required_fields() {
        let spec = agents::issue_raising();

        assert_eq!(spec.component, ComponentType::IssueRaising);
        assert!(!spec.role.is_empty());
        assert!(!spec.objectives.is_empty());
        assert!(!spec.techniques.is_empty());
        assert!(!spec.output_schema.fields.is_empty());
        assert!(spec.transitions.min_turns > 0);
    }

    #[test]
    fn test_issue_raising_agent() {
        let spec = agents::issue_raising();

        assert_eq!(spec.component, ComponentType::IssueRaising);
        assert!(spec.role.contains("categorize"));
        assert_eq!(spec.output_schema.fields.len(), 3); // decisions, objectives, uncertainties
    }

    #[test]
    fn test_problem_frame_agent() {
        let spec = agents::problem_frame();

        assert_eq!(spec.component, ComponentType::ProblemFrame);
        assert!(spec.role.contains("decision architecture"));
        assert!(spec
            .output_schema
            .fields
            .iter()
            .any(|f| f.name == "decision_statement"));
    }

    #[test]
    fn test_objectives_agent() {
        let spec = agents::objectives();

        assert_eq!(spec.component, ComponentType::Objectives);
        assert!(spec.role.contains("fundamental vs means"));
        assert!(spec
            .output_schema
            .fields
            .iter()
            .any(|f| f.name == "fundamental_objectives"));
    }

    #[test]
    fn test_alternatives_agent() {
        let spec = agents::alternatives();

        assert_eq!(spec.component, ComponentType::Alternatives);
        assert!(spec.role.contains("status quo"));
        assert!(spec
            .output_schema
            .fields
            .iter()
            .any(|f| f.name == "status_quo"));
    }

    #[test]
    fn test_consequences_agent() {
        let spec = agents::consequences();

        assert_eq!(spec.component, ComponentType::Consequences);
        assert!(spec.role.contains("Pugh"));
        assert!(spec
            .output_schema
            .fields
            .iter()
            .any(|f| f.name == "consequence_table"));
    }

    #[test]
    fn test_tradeoffs_agent() {
        let spec = agents::tradeoffs();

        assert_eq!(spec.component, ComponentType::Tradeoffs);
        assert!(spec.role.contains("dominated"));
        assert!(spec
            .output_schema
            .fields
            .iter()
            .any(|f| f.name == "key_tradeoffs"));
    }

    #[test]
    fn test_recommendation_agent() {
        let spec = agents::recommendation();

        assert_eq!(spec.component, ComponentType::Recommendation);
        assert!(spec.role.contains("synthesize"));
        assert!(spec.role.contains("not deciding for them"));
        assert!(spec
            .output_schema
            .fields
            .iter()
            .any(|f| f.name == "synthesis"));
    }

    #[test]
    fn test_decision_quality_agent() {
        let spec = agents::decision_quality();

        assert_eq!(spec.component, ComponentType::DecisionQuality);
        assert!(spec.role.contains("7 elements"));
        assert!(spec
            .output_schema
            .fields
            .iter()
            .any(|f| f.name == "overall_score"));
    }

    #[test]
    fn test_get_agent_by_component() {
        let spec = agents::get(ComponentType::Alternatives).unwrap();
        assert_eq!(spec.component, ComponentType::Alternatives);

        let no_spec = agents::get(ComponentType::NotesNextSteps);
        assert!(no_spec.is_none());
    }

    #[test]
    fn test_transition_rules_validate_correctly() {
        let spec = agents::objectives();

        assert_eq!(spec.transitions.min_turns, 3);
        assert!(spec
            .transitions
            .required_outputs
            .contains(&"fundamental_objectives".to_string()));
        assert!(!spec.transitions.completion_signals.is_empty());
    }

    #[test]
    fn test_output_schema_field_types() {
        let field_types = vec![
            FieldType::String,
            FieldType::Number,
            FieldType::Boolean,
            FieldType::Array,
            FieldType::Object,
        ];

        assert_eq!(field_types.len(), 5);
    }

    #[test]
    fn test_schema_field_required_flag() {
        let spec = agents::alternatives();

        let alternatives_field = spec
            .output_schema
            .fields
            .iter()
            .find(|f| f.name == "alternatives")
            .unwrap();
        assert!(alternatives_field.required);

        let strategy_table_field = spec
            .output_schema
            .fields
            .iter()
            .find(|f| f.name == "strategy_table")
            .unwrap();
        assert!(!strategy_table_field.required);
    }
}
