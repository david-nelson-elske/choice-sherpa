//! Message templates for component conversations.
//!
//! Provides opening messages and extraction prompts for all 9 PrOACT components.

use crate::domain::foundation::ComponentType;

/// Returns the opening message for a component conversation.
///
/// This is the first message the AI assistant sends to greet the user
/// and set context for the conversation.
pub fn opening_message_for_component(component_type: ComponentType) -> &'static str {
    match component_type {
        ComponentType::IssueRaising => ISSUE_RAISING_OPENING,
        ComponentType::ProblemFrame => PROBLEM_FRAME_OPENING,
        ComponentType::Objectives => OBJECTIVES_OPENING,
        ComponentType::Alternatives => ALTERNATIVES_OPENING,
        ComponentType::Consequences => CONSEQUENCES_OPENING,
        ComponentType::Tradeoffs => TRADEOFFS_OPENING,
        ComponentType::Recommendation => RECOMMENDATION_OPENING,
        ComponentType::DecisionQuality => DECISION_QUALITY_OPENING,
        ComponentType::NotesNextSteps => NOTES_NEXT_STEPS_OPENING,
    }
}

/// Returns the extraction prompt for a component.
///
/// This is the system prompt used when extracting structured data
/// from the conversation.
pub fn extraction_prompt_for_component(component_type: ComponentType) -> &'static str {
    match component_type {
        ComponentType::IssueRaising => ISSUE_RAISING_EXTRACTION,
        ComponentType::ProblemFrame => PROBLEM_FRAME_EXTRACTION,
        ComponentType::Objectives => OBJECTIVES_EXTRACTION,
        ComponentType::Alternatives => ALTERNATIVES_EXTRACTION,
        ComponentType::Consequences => CONSEQUENCES_EXTRACTION,
        ComponentType::Tradeoffs => TRADEOFFS_EXTRACTION,
        ComponentType::Recommendation => RECOMMENDATION_EXTRACTION,
        ComponentType::DecisionQuality => DECISION_QUALITY_EXTRACTION,
        ComponentType::NotesNextSteps => NOTES_NEXT_STEPS_EXTRACTION,
    }
}

// ============================================================================
// Opening Messages
// ============================================================================

const ISSUE_RAISING_OPENING: &str = r#"Hello! I'm here to help you think through an important decision.

Let's start by capturing what's on your mind. This is a brainstorming space where we'll gather all your initial thoughts — decisions you're facing, goals you want to achieve, things you're uncertain about, and any other considerations.

Don't worry about organizing everything perfectly right now. Just share what's been occupying your thoughts about this situation.

**What situation or challenge would you like to work through?**"#;

const PROBLEM_FRAME_OPENING: &str = r#"Now let's focus on defining the core decision.

A well-framed decision is specific, actionable, and has clear boundaries. We'll identify:
- **Who** is the decision maker
- **What** exactly needs to be decided
- **When** is it relevant
- **What's in scope** and what's not

If you've already captured some potential decisions, we can start from there. Otherwise, tell me about the decision you're facing.

**Which decision would you like to focus on?**"#;

const OBJECTIVES_OPENING: &str = r#"With the decision framed, let's explore what outcomes matter most to you.

Objectives come in two types:
- **Fundamental objectives** — things you value for their own sake
- **Means objectives** — things that help you achieve what you truly value

I'll help you distinguish between "what you want" and "how to get it."

**Thinking about this decision, what outcomes matter most to you?**"#;

const ALTERNATIVES_OPENING: &str = r#"Now let's explore your options.

Good decision-making requires considering multiple alternatives, including:
- Options you're already considering
- Creative alternatives you might not have thought of
- **The status quo** — what happens if you do nothing

The status quo is our baseline for comparison, so let's make sure it's clearly defined.

**What options are you considering? And what does "doing nothing" look like in this situation?**"#;

const CONSEQUENCES_OPENING: &str = r#"Let's evaluate how each alternative performs against your objectives.

We'll use a simple rating scale:
- **+2** — Much better than status quo
- **+1** — Somewhat better
- **0** — About the same
- **-1** — Somewhat worse
- **-2** — Much worse than status quo

For each alternative and objective, I'll ask you to rate the comparison and explain your reasoning.

**Let's start with your first objective. How does each alternative compare to the status quo?**"#;

const TRADEOFFS_OPENING: &str = r#"Now let's analyze the tradeoffs in your consequence table.

I'll help you identify:
- **Dominated alternatives** — options that are worse on all objectives
- **Irrelevant objectives** — criteria where all alternatives rate the same
- **Key tensions** — where you gain on some objectives but lose on others

This analysis helps clarify which alternatives deserve serious consideration.

**I've analyzed your ratings. Let me share what I found...**"#;

const RECOMMENDATION_OPENING: &str = r#"Let's synthesize everything we've learned.

I want to be clear: **the decision is yours to make**. My role is to summarize the analysis and highlight what we've discovered, not to tell you what to do.

I'll share:
- Key considerations from our analysis
- Any alternative that seems to stand out
- Remaining uncertainties you might want to resolve

**Here's what the analysis reveals...**"#;

const DECISION_QUALITY_OPENING: &str = r#"Let's assess the quality of your decision process using the Decision Quality framework.

We'll rate seven elements on a scale of 0-100%:

1. **Helpful Frame** — Is the decision well-defined?
2. **Creative Alternatives** — Have you considered enough options?
3. **Relevant Information** — Do you have the facts you need?
4. **Clear Values** — Do you know what matters most?
5. **Sound Reasoning** — Is the logic connecting values to choice solid?
6. **Commitment to Action** — Are you ready to execute?
7. **Right People Involved** — Are the right stakeholders engaged?

Your overall Decision Quality score is the **minimum** of all seven elements — because a chain is only as strong as its weakest link.

**Let's start with the first element. How would you rate your Helpful Frame (0-100%)?**"#;

const NOTES_NEXT_STEPS_OPENING: &str = r#"We're in the final stretch. Let's capture any remaining thoughts and plan your next steps.

I'll help you document:
- **Notes** — any observations or insights from this process
- **Open questions** — things you still want to figure out
- **Action items** — specific next steps with owners and timelines

**What questions or thoughts remain? What actions do you plan to take?**"#;

// ============================================================================
// Extraction Prompts
// ============================================================================

const ISSUE_RAISING_EXTRACTION: &str = r#"Extract structured data from the conversation about issue raising.

Output JSON with the following structure:
{
  "potential_decisions": [
    {"id": "uuid", "description": "string"}
  ],
  "objectives": [
    {"id": "uuid", "description": "string"}
  ],
  "uncertainties": [
    {"id": "uuid", "description": "string"}
  ],
  "considerations": [
    {"id": "uuid", "description": "string"}
  ]
}

Rules:
- Generate UUIDs for new items
- potential_decisions are choices that need to be made
- objectives are goals or outcomes the user wants
- uncertainties are things the user is unsure about
- considerations are other relevant factors
- Be conservative — only extract what was clearly stated"#;

const PROBLEM_FRAME_EXTRACTION: &str = r#"Extract structured data about the problem frame.

Output JSON with the following structure:
{
  "decision_maker": {
    "name": "string",
    "role": "string (optional)"
  },
  "focal_decision": {
    "statement": "string (min 10 chars)",
    "scope": "string",
    "constraints": ["string"]
  },
  "decision_hierarchy": {
    "already_made": ["string"],
    "deferred": ["string"]
  },
  "parties": [
    {
      "name": "string",
      "role": "string",
      "influence": "high|medium|low"
    }
  ]
}

Rules:
- The focal decision statement must be specific and actionable
- Constraints are limitations on the decision
- Parties include stakeholders affected by or influencing the decision"#;

const OBJECTIVES_EXTRACTION: &str = r#"Extract structured data about objectives.

Output JSON with the following structure:
{
  "fundamental_objectives": [
    {
      "id": "uuid",
      "description": "string",
      "performance_measure": "string (optional)"
    }
  ],
  "means_objectives": [
    {
      "id": "uuid",
      "description": "string",
      "supports_fundamental_id": "uuid (optional)"
    }
  ]
}

Rules:
- Fundamental objectives are valued for their own sake
- Means objectives are ways to achieve fundamental objectives
- Link means objectives to the fundamental objectives they support
- Include performance measures where the user specified how they'd measure success"#;

const ALTERNATIVES_EXTRACTION: &str = r#"Extract structured data about alternatives.

Output JSON with the following structure:
{
  "alternatives": [
    {
      "id": "uuid",
      "name": "string",
      "description": "string"
    }
  ],
  "status_quo_id": "uuid",
  "strategy_table": {
    "dimensions": ["string"],
    "combinations": [
      {
        "id": "uuid",
        "name": "string",
        "dimension_values": {"dimension_name": "value"}
      }
    ]
  }
}

Rules:
- status_quo_id must reference one of the alternatives
- Include strategy_table only if the conversation discussed strategy dimensions
- Each alternative must have a distinct name and description"#;

const CONSEQUENCES_EXTRACTION: &str = r#"Extract the consequence table data.

Output JSON with the following structure:
{
  "cells": {
    "alternative_id:objective_id": {
      "rating": -2|-1|0|1|2,
      "rationale": "string",
      "uncertainty": "low|medium|high"
    }
  }
}

Rules:
- Ratings are relative to status quo (-2 to +2)
- Every alternative-objective combination should have a cell
- Include the rationale explaining the rating
- Uncertainty reflects confidence in the rating"#;

const TRADEOFFS_EXTRACTION: &str = r#"Extract tradeoff analysis results.

Output JSON with the following structure:
{
  "dominated_alternatives": [
    {
      "id": "uuid",
      "dominated_by": "uuid",
      "reason": "string"
    }
  ],
  "irrelevant_objectives": [
    {
      "id": "uuid",
      "reason": "string"
    }
  ],
  "tensions": [
    {
      "alternative_id": "uuid",
      "gains_on": ["objective_id"],
      "loses_on": ["objective_id"]
    }
  ]
}

Rules:
- An alternative is dominated if another is at least as good on all objectives and better on at least one
- An objective is irrelevant if all alternatives have the same rating
- Tensions show the tradeoff profile of each non-dominated alternative"#;

const RECOMMENDATION_EXTRACTION: &str = r#"Extract the recommendation synthesis.

Output JSON with the following structure:
{
  "synthesis": "string (min 50 chars)",
  "standout_option": {
    "alternative_id": "uuid (optional)",
    "reason": "string"
  },
  "key_considerations": ["string"],
  "remaining_uncertainties": [
    {
      "description": "string",
      "resolution_path": "string (optional)"
    }
  ]
}

Rules:
- The synthesis summarizes the analysis without making the decision
- standout_option is only included if one alternative clearly emerges
- Key considerations are the most important factors for the decision
- Include paths to resolve uncertainties where discussed"#;

const DECISION_QUALITY_EXTRACTION: &str = r#"Extract Decision Quality scores.

Output JSON with the following structure:
{
  "elements": [
    {
      "name": "Helpful Frame|Creative Alternatives|Relevant Information|Clear Values|Sound Reasoning|Commitment to Action|Right People Involved",
      "score": 0-100,
      "rationale": "string"
    }
  ],
  "overall_score": 0-100
}

Rules:
- All 7 elements must be scored
- overall_score is the MINIMUM of all element scores
- Include the user's rationale for each score"#;

const NOTES_NEXT_STEPS_EXTRACTION: &str = r#"Extract notes and next steps.

Output JSON with the following structure:
{
  "notes": ["string"],
  "open_questions": ["string"],
  "planned_actions": [
    {
      "description": "string",
      "owner": "string (optional)",
      "due_date": "string (optional)"
    }
  ],
  "decision_affirmation": "string (optional)"
}

Rules:
- Notes are observations or insights from the process
- Open questions are unresolved items
- Planned actions should have descriptions; owner and due_date are optional
- decision_affirmation captures the user's decision statement if they made one"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_components_have_opening_messages() {
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
            let message = opening_message_for_component(component);
            assert!(!message.is_empty(), "{:?} has empty opening message", component);
            assert!(message.len() >= 100, "{:?} opening message too short", component);
        }
    }

    #[test]
    fn all_components_have_extraction_prompts() {
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
            let prompt = extraction_prompt_for_component(component);
            assert!(!prompt.is_empty(), "{:?} has empty extraction prompt", component);
            assert!(prompt.contains("JSON"), "{:?} extraction prompt should mention JSON", component);
        }
    }

    #[test]
    fn issue_raising_opening_asks_question() {
        let message = opening_message_for_component(ComponentType::IssueRaising);
        assert!(message.contains("?"));
    }

    #[test]
    fn alternatives_opening_mentions_status_quo() {
        let message = opening_message_for_component(ComponentType::Alternatives);
        assert!(message.to_lowercase().contains("status quo"));
    }

    #[test]
    fn consequences_opening_explains_rating_scale() {
        let message = opening_message_for_component(ComponentType::Consequences);
        assert!(message.contains("+2"));
        assert!(message.contains("-2"));
    }

    #[test]
    fn recommendation_emphasizes_user_decision() {
        let message = opening_message_for_component(ComponentType::Recommendation);
        assert!(message.contains("decision is yours"));
    }

    #[test]
    fn decision_quality_lists_seven_elements() {
        let message = opening_message_for_component(ComponentType::DecisionQuality);
        assert!(message.contains("Helpful Frame"));
        assert!(message.contains("Creative Alternatives"));
        assert!(message.contains("Relevant Information"));
        assert!(message.contains("Clear Values"));
        assert!(message.contains("Sound Reasoning"));
        assert!(message.contains("Commitment to Action"));
        assert!(message.contains("Right People Involved"));
    }

    #[test]
    fn extraction_prompts_have_output_structure() {
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
            let prompt = extraction_prompt_for_component(component);
            // Should have structure indication
            assert!(
                prompt.contains("{") && prompt.contains("}"),
                "{:?} extraction prompt should show JSON structure",
                component
            );
        }
    }

    #[test]
    fn consequences_extraction_mentions_rating_range() {
        let prompt = extraction_prompt_for_component(ComponentType::Consequences);
        assert!(prompt.contains("-2"));
        assert!(prompt.contains("2"));
    }

    #[test]
    fn decision_quality_extraction_mentions_minimum() {
        let prompt = extraction_prompt_for_component(ComponentType::DecisionQuality);
        assert!(prompt.to_lowercase().contains("minimum"));
    }
}
