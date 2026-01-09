//! Agent phases within a conversation.
//!
//! Phases guide the AI agent's behavior during an active conversation.
//! Unlike ConversationState (which tracks lifecycle), phases determine
//! what kind of dialogue the agent should engage in.

use serde::{Deserialize, Serialize};

/// The current phase of AI agent behavior within an active conversation.
///
/// Phases flow in a general order but can loop or backtrack:
/// - `Intro` → `Gather` → `Clarify` (optional) → `Extract` → `Confirm`
///
/// Each phase has a distinct directive that guides the AI's responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentPhase {
    /// Initial greeting and context setting.
    /// AI explains what this step covers and makes user comfortable.
    Intro,

    /// Actively gathering information through questions.
    /// AI asks probing questions and listens actively.
    Gather,

    /// Clarifying ambiguities or inconsistencies.
    /// AI resolves unclear points before continuing.
    Clarify,

    /// Extracting structured data from conversation.
    /// AI synthesizes conversation into component-specific output.
    Extract,

    /// Confirming extracted data with user.
    /// AI presents results and asks for corrections or approval.
    Confirm,
}

impl AgentPhase {
    /// Returns the AI's primary directive in this phase.
    ///
    /// This directive guides the tone and purpose of the AI's responses.
    pub fn directive(&self) -> &'static str {
        match self {
            Self::Intro => "Set context and make user comfortable. Explain what this step covers.",
            Self::Gather => "Ask probing questions to elicit information. Listen actively.",
            Self::Clarify => "Resolve ambiguities. Ask follow-up questions on unclear points.",
            Self::Extract => "Synthesize conversation into structured output format.",
            Self::Confirm => "Present extracted data to user. Ask for corrections or approval.",
        }
    }

    /// Returns a shorter label for the phase, suitable for UI display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Intro => "Introduction",
            Self::Gather => "Gathering",
            Self::Clarify => "Clarifying",
            Self::Extract => "Extracting",
            Self::Confirm => "Confirming",
        }
    }

    /// Returns true if this phase typically generates AI-initiated responses.
    ///
    /// The Extract phase is primarily a processing phase where AI generates
    /// structured data rather than conversational responses.
    pub fn is_ai_speaking(&self) -> bool {
        !matches!(self, Self::Extract)
    }

    /// Returns true if the phase expects user input.
    pub fn expects_user_input(&self) -> bool {
        matches!(self, Self::Intro | Self::Gather | Self::Clarify | Self::Confirm)
    }

    /// Returns true if this phase involves data extraction or confirmation.
    pub fn is_extraction_related(&self) -> bool {
        matches!(self, Self::Extract | Self::Confirm)
    }

    /// Returns all valid next phases from this phase.
    ///
    /// Note: This is distinct from StateMachine - phases can branch and loop.
    pub fn valid_next_phases(&self) -> Vec<Self> {
        match self {
            Self::Intro => vec![Self::Gather],
            Self::Gather => vec![Self::Gather, Self::Clarify, Self::Extract],
            Self::Clarify => vec![Self::Gather, Self::Extract],
            Self::Extract => vec![Self::Confirm],
            Self::Confirm => vec![Self::Gather],
        }
    }

    /// Returns true if transition to target phase is valid.
    pub fn can_transition_to(&self, target: &Self) -> bool {
        self.valid_next_phases().contains(target)
    }
}

impl Default for AgentPhase {
    fn default() -> Self {
        Self::Intro
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod phase_basics {
        use super::*;

        #[test]
        fn default_phase_is_intro() {
            assert_eq!(AgentPhase::default(), AgentPhase::Intro);
        }

        #[test]
        fn serializes_to_snake_case() {
            let phase = AgentPhase::Gather;
            let json = serde_json::to_string(&phase).unwrap();
            assert_eq!(json, "\"gather\"");
        }

        #[test]
        fn deserializes_from_snake_case() {
            let phase: AgentPhase = serde_json::from_str("\"clarify\"").unwrap();
            assert_eq!(phase, AgentPhase::Clarify);
        }

        #[test]
        fn all_phases_have_directives() {
            for phase in [
                AgentPhase::Intro,
                AgentPhase::Gather,
                AgentPhase::Clarify,
                AgentPhase::Extract,
                AgentPhase::Confirm,
            ] {
                assert!(!phase.directive().is_empty());
            }
        }

        #[test]
        fn all_phases_have_labels() {
            for phase in [
                AgentPhase::Intro,
                AgentPhase::Gather,
                AgentPhase::Clarify,
                AgentPhase::Extract,
                AgentPhase::Confirm,
            ] {
                assert!(!phase.label().is_empty());
            }
        }
    }

    mod is_ai_speaking {
        use super::*;

        #[test]
        fn intro_is_ai_speaking() {
            assert!(AgentPhase::Intro.is_ai_speaking());
        }

        #[test]
        fn gather_is_ai_speaking() {
            assert!(AgentPhase::Gather.is_ai_speaking());
        }

        #[test]
        fn clarify_is_ai_speaking() {
            assert!(AgentPhase::Clarify.is_ai_speaking());
        }

        #[test]
        fn extract_is_not_ai_speaking() {
            // Extract is a processing phase, not conversational
            assert!(!AgentPhase::Extract.is_ai_speaking());
        }

        #[test]
        fn confirm_is_ai_speaking() {
            assert!(AgentPhase::Confirm.is_ai_speaking());
        }
    }

    mod expects_user_input {
        use super::*;

        #[test]
        fn intro_expects_input() {
            assert!(AgentPhase::Intro.expects_user_input());
        }

        #[test]
        fn gather_expects_input() {
            assert!(AgentPhase::Gather.expects_user_input());
        }

        #[test]
        fn clarify_expects_input() {
            assert!(AgentPhase::Clarify.expects_user_input());
        }

        #[test]
        fn extract_does_not_expect_input() {
            assert!(!AgentPhase::Extract.expects_user_input());
        }

        #[test]
        fn confirm_expects_input() {
            assert!(AgentPhase::Confirm.expects_user_input());
        }
    }

    mod is_extraction_related {
        use super::*;

        #[test]
        fn intro_is_not_extraction_related() {
            assert!(!AgentPhase::Intro.is_extraction_related());
        }

        #[test]
        fn gather_is_not_extraction_related() {
            assert!(!AgentPhase::Gather.is_extraction_related());
        }

        #[test]
        fn clarify_is_not_extraction_related() {
            assert!(!AgentPhase::Clarify.is_extraction_related());
        }

        #[test]
        fn extract_is_extraction_related() {
            assert!(AgentPhase::Extract.is_extraction_related());
        }

        #[test]
        fn confirm_is_extraction_related() {
            assert!(AgentPhase::Confirm.is_extraction_related());
        }
    }

    mod phase_transitions {
        use super::*;

        #[test]
        fn intro_transitions_to_gather() {
            let phase = AgentPhase::Intro;
            assert!(phase.can_transition_to(&AgentPhase::Gather));
            assert_eq!(phase.valid_next_phases(), vec![AgentPhase::Gather]);
        }

        #[test]
        fn gather_can_loop_or_proceed() {
            let phase = AgentPhase::Gather;
            assert!(phase.can_transition_to(&AgentPhase::Gather));
            assert!(phase.can_transition_to(&AgentPhase::Clarify));
            assert!(phase.can_transition_to(&AgentPhase::Extract));
            // Cannot go back to intro
            assert!(!phase.can_transition_to(&AgentPhase::Intro));
        }

        #[test]
        fn clarify_returns_to_gather_or_proceeds() {
            let phase = AgentPhase::Clarify;
            assert!(phase.can_transition_to(&AgentPhase::Gather));
            assert!(phase.can_transition_to(&AgentPhase::Extract));
            // Cannot loop to clarify
            assert!(!phase.can_transition_to(&AgentPhase::Clarify));
        }

        #[test]
        fn extract_always_proceeds_to_confirm() {
            let phase = AgentPhase::Extract;
            assert!(phase.can_transition_to(&AgentPhase::Confirm));
            assert_eq!(phase.valid_next_phases(), vec![AgentPhase::Confirm]);
        }

        #[test]
        fn confirm_can_return_to_gather_for_changes() {
            let phase = AgentPhase::Confirm;
            assert!(phase.can_transition_to(&AgentPhase::Gather));
            // Completion is handled via ConversationState, not phase
            assert!(!phase.can_transition_to(&AgentPhase::Intro));
        }
    }

    mod directive_content {
        use super::*;

        #[test]
        fn intro_directive_mentions_context() {
            let directive = AgentPhase::Intro.directive();
            assert!(directive.contains("context") || directive.contains("comfortable"));
        }

        #[test]
        fn gather_directive_mentions_questions() {
            let directive = AgentPhase::Gather.directive();
            assert!(directive.contains("question") || directive.contains("probing"));
        }

        #[test]
        fn clarify_directive_mentions_ambiguity() {
            let directive = AgentPhase::Clarify.directive();
            assert!(directive.contains("ambigu") || directive.contains("unclear"));
        }

        #[test]
        fn extract_directive_mentions_structured() {
            let directive = AgentPhase::Extract.directive();
            assert!(directive.contains("structured") || directive.contains("output"));
        }

        #[test]
        fn confirm_directive_mentions_corrections() {
            let directive = AgentPhase::Confirm.directive();
            assert!(directive.contains("correct") || directive.contains("approval"));
        }
    }
}
