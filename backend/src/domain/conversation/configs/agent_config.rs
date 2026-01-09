//! Agent configuration for component-specific behavior.
//!
//! Defines how the AI agent should behave in each PrOACT component,
//! including phase-specific prompts and completion criteria.

use crate::domain::foundation::ComponentType;

/// Configuration for an agent within a specific component.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// The component type this configuration is for.
    pub component_type: ComponentType,
    /// Description of the component's purpose.
    pub purpose: &'static str,
    /// Phase-specific prompts and guidance.
    pub phase_prompts: PhasePrompts,
    /// Criteria for completing this component.
    pub completion_criteria: CompletionCriteria,
}

/// Prompts and guidance for each agent phase.
#[derive(Debug, Clone)]
pub struct PhasePrompts {
    /// Guidance for the intro phase.
    pub intro: &'static str,
    /// Guidance for the gather phase.
    pub gather: &'static str,
    /// Guidance for the clarify phase.
    pub clarify: &'static str,
    /// Guidance for the extract phase.
    pub extract: &'static str,
    /// Guidance for the confirm phase.
    pub confirm: &'static str,
}

/// Criteria for component completion.
#[derive(Debug, Clone)]
pub struct CompletionCriteria {
    /// Minimum items required (varies by component).
    pub min_items: usize,
    /// Whether user confirmation is required.
    pub requires_confirmation: bool,
    /// Description of what constitutes completion.
    pub description: &'static str,
}

impl Default for CompletionCriteria {
    fn default() -> Self {
        Self {
            min_items: 1,
            requires_confirmation: true,
            description: "User confirms the extracted data is accurate.",
        }
    }
}

/// Returns the agent configuration for a given component type.
pub fn agent_config_for_component(component_type: ComponentType) -> AgentConfig {
    match component_type {
        ComponentType::IssueRaising => issue_raising_config(),
        ComponentType::ProblemFrame => problem_frame_config(),
        ComponentType::Objectives => objectives_config(),
        ComponentType::Alternatives => alternatives_config(),
        ComponentType::Consequences => consequences_config(),
        ComponentType::Tradeoffs => tradeoffs_config(),
        ComponentType::Recommendation => recommendation_config(),
        ComponentType::DecisionQuality => decision_quality_config(),
        ComponentType::NotesNextSteps => notes_next_steps_config(),
    }
}

fn issue_raising_config() -> AgentConfig {
    AgentConfig {
        component_type: ComponentType::IssueRaising,
        purpose: "Categorize initial thoughts into decisions, objectives, uncertainties",
        phase_prompts: PhasePrompts {
            intro: "Welcome the user. Explain that you'll help capture their initial thoughts about a situation. Ask what's on their mind.",
            gather: "Listen for decisions, goals, concerns, and uncertainties. Categorize internally. Ask: 'Is that something you need to decide, something you want to achieve, or something you're uncertain about?'",
            clarify: "Clarify the distinction between decisions and objectives. Help separate actions from outcomes.",
            extract: "Parse conversation for potential_decisions, objectives, uncertainties, and considerations. Generate unique IDs for each item.",
            confirm: "Present the categorized list. Ask: 'Have I captured everything? Should any items move between categories?'",
        },
        completion_criteria: CompletionCriteria {
            min_items: 1,
            requires_confirmation: true,
            description: "At least 1 potential_decision identified; user confirms categorization.",
        },
    }
}

fn problem_frame_config() -> AgentConfig {
    AgentConfig {
        component_type: ComponentType::ProblemFrame,
        purpose: "Define decision architecture, constraints, stakeholders",
        phase_prompts: PhasePrompts {
            intro: "Review potential decisions from Issue Raising (if available). Ask: 'Which decision should we focus on?'",
            gather: "Identify the primary decision maker. Clarify scope and constraints. Discover stakeholders and their influence. Map decision hierarchy (already made, focal, deferred).",
            clarify: "Ensure the decision statement is specific and actionable. Distinguish between the core decision and surrounding decisions.",
            extract: "Build decision_maker object, focal_decision with statement/scope/constraints, decision_hierarchy, and parties array.",
            confirm: "Present the problem frame summary. Verify the decision statement is actionable and accurately scoped.",
        },
        completion_criteria: CompletionCriteria {
            min_items: 1,
            requires_confirmation: true,
            description: "Decision maker identified; focal decision statement defined (min 10 chars); scope clarified; user confirms frame is accurate.",
        },
    }
}

fn objectives_config() -> AgentConfig {
    AgentConfig {
        component_type: ComponentType::Objectives,
        purpose: "Identify fundamental vs means objectives with measures",
        phase_prompts: PhasePrompts {
            intro: "Reference the focal decision from Problem Frame. Ask: 'What outcomes matter most to you in this decision?'",
            gather: "Capture objectives as stated. Distinguish fundamental (end goals) from means (how to get there). Probe for performance measures. Ask: 'How would you know if you achieved this?'",
            clarify: "Help distinguish between what the user truly values (fundamental) vs. how they might get there (means). Explore 'why' to find underlying objectives.",
            extract: "Separate into fundamental_objectives and means_objectives. Link means to their supporting fundamental objectives. Capture performance measures where stated.",
            confirm: "Present the objective hierarchy. Verify fundamental objectives truly matter for their own sake, not as means to something else.",
        },
        completion_criteria: CompletionCriteria {
            min_items: 1,
            requires_confirmation: true,
            description: "At least 1 fundamental objective; user confirms objectives capture what matters.",
        },
    }
}

fn alternatives_config() -> AgentConfig {
    AgentConfig {
        component_type: ComponentType::Alternatives,
        purpose: "Capture options, strategy tables, status quo baseline",
        phase_prompts: PhasePrompts {
            intro: "Reference objectives from the prior step. Ask: 'What options are you considering? Include doing nothing (status quo).'",
            gather: "Capture each alternative with name and description. Ensure status quo is explicitly stated. Probe for creative alternatives: 'What else could you do?' For complex decisions, consider building a strategy table.",
            clarify: "Distinguish between complete alternatives and components that could be combined. Ensure each option is truly distinct.",
            extract: "Build alternatives array. Designate status_quo_id. Build strategy_table if applicable.",
            confirm: "Present all alternatives. Verify status quo is captured. Ask: 'Are there any other options we should consider?'",
        },
        completion_criteria: CompletionCriteria {
            min_items: 2,
            requires_confirmation: true,
            description: "At least 2 alternatives (including status quo); status quo explicitly identified; user confirms completeness.",
        },
    }
}

fn consequences_config() -> AgentConfig {
    AgentConfig {
        component_type: ComponentType::Consequences,
        purpose: "Build consequence table with Pugh ratings (-2 to +2)",
        phase_prompts: PhasePrompts {
            intro: "Load alternatives and objectives from prior steps. Explain the Pugh rating scale: -2 (much worse) to +2 (much better) compared to status quo. Start with the first objective.",
            gather: "For each objective, evaluate each alternative vs status quo. Ask: 'How does [alternative] compare to status quo on [objective]?' Capture rationale for each rating. Note uncertainty levels.",
            clarify: "Clarify ratings that seem inconsistent. Ask about high-uncertainty assessments. Ensure comparisons are against status quo, not absolute judgments.",
            extract: "Build consequence table with all cells populated. Format: cells map of 'alt_id:obj_id' -> rating, rationale, uncertainty.",
            confirm: "Present the consequence table. Highlight any missing cells. Verify ratings make sense and are consistently applied.",
        },
        completion_criteria: CompletionCriteria {
            min_items: 1,
            requires_confirmation: true,
            description: "All cells in consequence table filled; user confirms ratings are reasonable.",
        },
    }
}

fn tradeoffs_config() -> AgentConfig {
    AgentConfig {
        component_type: ComponentType::Tradeoffs,
        purpose: "Surface dominated alternatives, tensions, irrelevant objectives",
        phase_prompts: PhasePrompts {
            intro: "Load the consequences table. Run dominance analysis automatically. Present initial findings on which alternatives are dominated.",
            gather: "Discuss dominated alternatives (worse on all objectives). Explore tensions between remaining alternatives. Identify irrelevant objectives (same rating across all alternatives). Ask: 'Does this analysis surprise you?'",
            clarify: "Verify dominance conclusions. Discuss whether dominated alternatives might have hidden advantages. Explore if irrelevant objectives truly don't matter.",
            extract: "Build dominated_alternatives array. Build irrelevant_objectives array. Build tensions array with gains/losses for each alternative.",
            confirm: "Present the tradeoff summary. Verify dominance conclusions match user's intuition.",
        },
        completion_criteria: CompletionCriteria {
            min_items: 0,
            requires_confirmation: true,
            description: "Dominance analysis complete; user understands key tradeoffs.",
        },
    }
}

fn recommendation_config() -> AgentConfig {
    AgentConfig {
        component_type: ComponentType::Recommendation,
        purpose: "Synthesize analysis - does NOT decide for user",
        phase_prompts: PhasePrompts {
            intro: "Reference the full analysis path. Explain: 'I'll summarize what we've found, but the decision is yours to make.'",
            gather: "Discuss key considerations from the analysis. Surface remaining uncertainties. Explore if any alternative stands out. Ask: 'What else would help you decide?'",
            clarify: "Clarify any remaining uncertainties. Discuss paths to resolve key unknowns. Do NOT push the user toward a specific choice.",
            extract: "Write synthesis text. Identify standout_option if applicable. List key_considerations. List remaining_uncertainties with potential resolution paths.",
            confirm: "Present the recommendation summary. Emphasize user retains decision authority. Ask if anything is missing from the synthesis.",
        },
        completion_criteria: CompletionCriteria {
            min_items: 1,
            requires_confirmation: true,
            description: "Synthesis written (min 50 chars); user acknowledges summary.",
        },
    }
}

fn decision_quality_config() -> AgentConfig {
    AgentConfig {
        component_type: ComponentType::DecisionQuality,
        purpose: "Rate 7 DQ elements, compute overall score",
        phase_prompts: PhasePrompts {
            intro: "Explain the Decision Quality framework. Present the 7 elements: (1) Helpful Frame, (2) Creative Alternatives, (3) Relevant Information, (4) Clear Values, (5) Sound Reasoning, (6) Commitment to Action, (7) Right People Involved.",
            gather: "For each element, ask user to rate 0-100%. Discuss rationale for each rating. Identify improvement paths for low scores. Ask: 'What would it take to raise this score?'",
            clarify: "Challenge overly optimistic or pessimistic ratings. Ensure ratings reflect actual evidence from the process, not wishful thinking.",
            extract: "Build elements array with scores and rationale. Compute overall_score as MIN of all element scores.",
            confirm: "Present the DQ scorecard. If overall < 100%, discuss what would improve it. Ask if scores reflect user's confidence in the decision.",
        },
        completion_criteria: CompletionCriteria {
            min_items: 7,
            requires_confirmation: true,
            description: "All 7 elements scored; user confirms scores reflect their confidence.",
        },
    }
}

fn notes_next_steps_config() -> AgentConfig {
    AgentConfig {
        component_type: ComponentType::NotesNextSteps,
        purpose: "Wrap-up notes, open questions, action items",
        phase_prompts: PhasePrompts {
            intro: "Ask: 'What questions or thoughts remain?' Probe for planned actions.",
            gather: "Capture notes and observations. List open questions. Define action items with owners and due dates. If DQ = 100%, capture decision affirmation.",
            clarify: "Clarify ownership and timing for action items. Ensure open questions are clearly stated.",
            extract: "Build notes array. Build open_questions array. Build planned_actions array with owner and due_date fields.",
            confirm: "Present the summary. Verify next steps are clear. Ask if user is ready to wrap up this decision process.",
        },
        completion_criteria: CompletionCriteria {
            min_items: 0,
            requires_confirmation: true,
            description: "User confirms they're ready to wrap up.",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_components_have_configs() {
        let components = [
            ComponentType::IssueRaising,
            ComponentType::ProblemFrame,
            ComponentType::Objectives,
            ComponentType::Alternatives,
            ComponentType::Consequences,
            ComponentType::Tradeoffs,
            ComponentType::Recommendation,
            ComponentType::DecisionQuality,
            ComponentType::NotesNextSteps,
        ];

        for component in components {
            let config = agent_config_for_component(component);
            assert_eq!(config.component_type, component);
            assert!(!config.purpose.is_empty());
        }
    }

    #[test]
    fn issue_raising_config_correct() {
        let config = agent_config_for_component(ComponentType::IssueRaising);
        assert!(config.purpose.contains("Categorize"));
        assert_eq!(config.completion_criteria.min_items, 1);
        assert!(config.completion_criteria.requires_confirmation);
    }

    #[test]
    fn alternatives_requires_at_least_two() {
        let config = agent_config_for_component(ComponentType::Alternatives);
        assert_eq!(config.completion_criteria.min_items, 2);
    }

    #[test]
    fn decision_quality_requires_seven_elements() {
        let config = agent_config_for_component(ComponentType::DecisionQuality);
        assert_eq!(config.completion_criteria.min_items, 7);
    }

    #[test]
    fn tradeoffs_can_have_zero_items() {
        let config = agent_config_for_component(ComponentType::Tradeoffs);
        assert_eq!(config.completion_criteria.min_items, 0);
    }

    #[test]
    fn all_phase_prompts_non_empty() {
        let components = [
            ComponentType::IssueRaising,
            ComponentType::ProblemFrame,
            ComponentType::Objectives,
            ComponentType::Alternatives,
            ComponentType::Consequences,
            ComponentType::Tradeoffs,
            ComponentType::Recommendation,
            ComponentType::DecisionQuality,
            ComponentType::NotesNextSteps,
        ];

        for component in components {
            let config = agent_config_for_component(component);
            let prompts = &config.phase_prompts;
            assert!(!prompts.intro.is_empty(), "{:?} intro empty", component);
            assert!(!prompts.gather.is_empty(), "{:?} gather empty", component);
            assert!(!prompts.clarify.is_empty(), "{:?} clarify empty", component);
            assert!(!prompts.extract.is_empty(), "{:?} extract empty", component);
            assert!(!prompts.confirm.is_empty(), "{:?} confirm empty", component);
        }
    }

    #[test]
    fn consequences_mentions_pugh_rating() {
        let config = agent_config_for_component(ComponentType::Consequences);
        assert!(config.purpose.contains("Pugh"));
    }

    #[test]
    fn recommendation_emphasizes_user_authority() {
        let config = agent_config_for_component(ComponentType::Recommendation);
        assert!(config.phase_prompts.intro.contains("decision is yours"));
    }
}
