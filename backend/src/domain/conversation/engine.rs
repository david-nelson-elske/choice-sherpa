//! Phase transition engine.
//!
//! Determines when to transition between agent phases based on
//! conversation context and component-specific rules.

use crate::domain::foundation::ComponentType;
use super::phase::AgentPhase;

/// Configuration for component-specific phase transition behavior.
#[derive(Debug, Clone)]
pub struct PhaseTransitionConfig {
    /// Minimum user messages before extraction can begin.
    pub min_messages_for_extraction: usize,
    /// Keywords that signal user is done providing input.
    pub completion_signals: Vec<String>,
    /// Keywords that trigger clarification.
    pub clarify_triggers: Vec<String>,
}

impl Default for PhaseTransitionConfig {
    fn default() -> Self {
        Self {
            min_messages_for_extraction: 3,
            completion_signals: vec![
                "done".to_string(),
                "that's all".to_string(),
                "that's it".to_string(),
                "nothing else".to_string(),
                "no more".to_string(),
                "ready".to_string(),
            ],
            clarify_triggers: vec![
                "?".to_string(),
                "not sure".to_string(),
                "maybe".to_string(),
                "unclear".to_string(),
                "confused".to_string(),
            ],
        }
    }
}

impl PhaseTransitionConfig {
    /// Creates component-specific configuration.
    pub fn for_component(component_type: ComponentType) -> Self {
        match component_type {
            ComponentType::IssueRaising => Self {
                min_messages_for_extraction: 3,
                completion_signals: vec![
                    "done".to_string(),
                    "that's all".to_string(),
                    "that's everything".to_string(),
                ],
                ..Default::default()
            },
            ComponentType::ProblemFrame => Self {
                min_messages_for_extraction: 2,
                completion_signals: vec![
                    "that's the decision".to_string(),
                    "yes, that's right".to_string(),
                ],
                ..Default::default()
            },
            ComponentType::Objectives => Self {
                min_messages_for_extraction: 2,
                ..Default::default()
            },
            ComponentType::Alternatives => Self {
                min_messages_for_extraction: 2,
                ..Default::default()
            },
            ComponentType::Consequences => Self {
                min_messages_for_extraction: 4,
                ..Default::default()
            },
            ComponentType::Tradeoffs => Self {
                min_messages_for_extraction: 3,
                ..Default::default()
            },
            ComponentType::Recommendation => Self {
                min_messages_for_extraction: 2,
                ..Default::default()
            },
            ComponentType::DecisionQuality => Self {
                min_messages_for_extraction: 3,
                ..Default::default()
            },
            ComponentType::NotesNextSteps => Self {
                min_messages_for_extraction: 1,
                completion_signals: vec![
                    "done".to_string(),
                    "that's all".to_string(),
                    "ready to finish".to_string(),
                ],
                ..Default::default()
            },
        }
    }
}

/// Snapshot of conversation state for phase transition decisions.
#[derive(Debug, Clone)]
pub struct ConversationSnapshot {
    /// Number of user messages in the conversation.
    pub user_message_count: usize,
    /// Content of the most recent user message.
    pub latest_user_message: Option<String>,
    /// Type of component this conversation is for.
    pub component_type: ComponentType,
}

impl ConversationSnapshot {
    /// Creates a new conversation snapshot.
    pub fn new(
        user_message_count: usize,
        latest_user_message: Option<String>,
        component_type: ComponentType,
    ) -> Self {
        Self {
            user_message_count,
            latest_user_message,
            component_type,
        }
    }

    /// Returns true if the latest message contains a completion signal.
    pub fn contains_completion_signal(&self, signals: &[String]) -> bool {
        if let Some(msg) = &self.latest_user_message {
            let lower = msg.to_lowercase();
            signals.iter().any(|signal| lower.contains(&signal.to_lowercase()))
        } else {
            false
        }
    }

    /// Returns true if the latest message contains clarification triggers.
    pub fn contains_clarify_trigger(&self, triggers: &[String]) -> bool {
        if let Some(msg) = &self.latest_user_message {
            let lower = msg.to_lowercase();
            triggers.iter().any(|trigger| lower.contains(&trigger.to_lowercase()))
        } else {
            false
        }
    }

    /// Returns true if user appears to request changes.
    pub fn requests_changes(&self) -> bool {
        if let Some(msg) = &self.latest_user_message {
            let lower = msg.to_lowercase();
            let change_signals = [
                "change",
                "modify",
                "update",
                "wrong",
                "incorrect",
                "fix",
                "revise",
                "edit",
                "no, ",
                "not quite",
                "that's not",
            ];
            change_signals.iter().any(|signal| lower.contains(signal))
        } else {
            false
        }
    }

    /// Returns true if user appears to approve the current state.
    pub fn indicates_approval(&self) -> bool {
        if let Some(msg) = &self.latest_user_message {
            let lower = msg.to_lowercase();
            let approval_signals = [
                "looks good",
                "that's right",
                "correct",
                "yes",
                "approve",
                "confirm",
                "perfect",
                "good",
                "okay",
            ];
            approval_signals.iter().any(|signal| lower.contains(signal))
        } else {
            false
        }
    }
}

/// Engine for determining phase transitions based on conversation state.
#[derive(Debug, Clone)]
pub struct PhaseTransitionEngine {
    config: PhaseTransitionConfig,
}

impl PhaseTransitionEngine {
    /// Creates a new engine with the given configuration.
    pub fn new(config: PhaseTransitionConfig) -> Self {
        Self { config }
    }

    /// Creates an engine with default configuration for a component type.
    pub fn for_component(component_type: ComponentType) -> Self {
        Self::new(PhaseTransitionConfig::for_component(component_type))
    }

    /// Determines the next phase based on current state and conversation.
    pub fn next_phase(
        &self,
        current: AgentPhase,
        snapshot: &ConversationSnapshot,
    ) -> AgentPhase {
        match current {
            AgentPhase::Intro => self.transition_from_intro(snapshot),
            AgentPhase::Gather => self.transition_from_gather(snapshot),
            AgentPhase::Clarify => self.transition_from_clarify(snapshot),
            AgentPhase::Extract => self.transition_from_extract(),
            AgentPhase::Confirm => self.transition_from_confirm(snapshot),
        }
    }

    fn transition_from_intro(&self, snapshot: &ConversationSnapshot) -> AgentPhase {
        // Move to gather after first user message
        if snapshot.user_message_count >= 1 {
            AgentPhase::Gather
        } else {
            AgentPhase::Intro
        }
    }

    fn transition_from_gather(&self, snapshot: &ConversationSnapshot) -> AgentPhase {
        // Check for extraction readiness
        if self.is_ready_for_extraction(snapshot) {
            return AgentPhase::Extract;
        }

        // Check for clarification need
        if snapshot.contains_clarify_trigger(&self.config.clarify_triggers) {
            return AgentPhase::Clarify;
        }

        // Stay in gather
        AgentPhase::Gather
    }

    fn transition_from_clarify(&self, snapshot: &ConversationSnapshot) -> AgentPhase {
        // After clarification, check if ready for extraction
        if self.is_ready_for_extraction(snapshot) {
            AgentPhase::Extract
        } else {
            // Return to gathering
            AgentPhase::Gather
        }
    }

    fn transition_from_extract(&self) -> AgentPhase {
        // Always move to confirm after extraction
        AgentPhase::Confirm
    }

    fn transition_from_confirm(&self, snapshot: &ConversationSnapshot) -> AgentPhase {
        // If user requests changes, go back to gather
        if snapshot.requests_changes() {
            AgentPhase::Gather
        } else {
            // Stay in confirm (completion is handled via ConversationState)
            AgentPhase::Confirm
        }
    }

    /// Returns true if the conversation has enough information for extraction.
    pub fn is_ready_for_extraction(&self, snapshot: &ConversationSnapshot) -> bool {
        // Have enough messages
        let has_enough_messages =
            snapshot.user_message_count >= self.config.min_messages_for_extraction;

        // Or user has signaled completion
        let has_completion_signal =
            snapshot.contains_completion_signal(&self.config.completion_signals);

        has_enough_messages || has_completion_signal
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &PhaseTransitionConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod config {
        use super::*;

        #[test]
        fn default_config_has_sensible_values() {
            let config = PhaseTransitionConfig::default();
            assert_eq!(config.min_messages_for_extraction, 3);
            assert!(!config.completion_signals.is_empty());
            assert!(!config.clarify_triggers.is_empty());
        }

        #[test]
        fn issue_raising_config_requires_3_messages() {
            let config = PhaseTransitionConfig::for_component(ComponentType::IssueRaising);
            assert_eq!(config.min_messages_for_extraction, 3);
        }

        #[test]
        fn consequences_config_requires_4_messages() {
            let config = PhaseTransitionConfig::for_component(ComponentType::Consequences);
            assert_eq!(config.min_messages_for_extraction, 4);
        }

        #[test]
        fn notes_next_steps_config_requires_1_message() {
            let config = PhaseTransitionConfig::for_component(ComponentType::NotesNextSteps);
            assert_eq!(config.min_messages_for_extraction, 1);
        }
    }

    mod snapshot {
        use super::*;

        #[test]
        fn contains_completion_signal_detects_done() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("I'm done with this".to_string()),
                ComponentType::IssueRaising,
            );
            let signals = vec!["done".to_string()];
            assert!(snapshot.contains_completion_signal(&signals));
        }

        #[test]
        fn contains_completion_signal_is_case_insensitive() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("I'm DONE with this".to_string()),
                ComponentType::IssueRaising,
            );
            let signals = vec!["done".to_string()];
            assert!(snapshot.contains_completion_signal(&signals));
        }

        #[test]
        fn contains_completion_signal_returns_false_when_not_present() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("Here's some more info".to_string()),
                ComponentType::IssueRaising,
            );
            let signals = vec!["done".to_string()];
            assert!(!snapshot.contains_completion_signal(&signals));
        }

        #[test]
        fn requests_changes_detects_change_keywords() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("Please change the second item".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.requests_changes());
        }

        #[test]
        fn requests_changes_detects_wrong() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("That's wrong".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.requests_changes());
        }

        #[test]
        fn requests_changes_returns_false_for_approval() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("Looks good!".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(!snapshot.requests_changes());
        }

        #[test]
        fn indicates_approval_detects_positive_response() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("That looks good to me".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.indicates_approval());
        }

        #[test]
        fn indicates_approval_detects_yes() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("Yes, that's correct".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.indicates_approval());
        }
    }

    mod engine_transitions {
        use super::*;

        fn engine() -> PhaseTransitionEngine {
            PhaseTransitionEngine::for_component(ComponentType::IssueRaising)
        }

        fn snapshot(user_msgs: usize, latest: Option<&str>) -> ConversationSnapshot {
            ConversationSnapshot::new(
                user_msgs,
                latest.map(|s| s.to_string()),
                ComponentType::IssueRaising,
            )
        }

        #[test]
        fn intro_stays_without_user_input() {
            let next = engine().next_phase(AgentPhase::Intro, &snapshot(0, None));
            assert_eq!(next, AgentPhase::Intro);
        }

        #[test]
        fn intro_transitions_to_gather_after_first_message() {
            let next = engine().next_phase(AgentPhase::Intro, &snapshot(1, Some("Hello")));
            assert_eq!(next, AgentPhase::Gather);
        }

        #[test]
        fn gather_stays_with_few_messages() {
            let next = engine().next_phase(AgentPhase::Gather, &snapshot(1, Some("Some info")));
            assert_eq!(next, AgentPhase::Gather);
        }

        #[test]
        fn gather_transitions_to_extract_with_enough_messages() {
            let next = engine().next_phase(AgentPhase::Gather, &snapshot(3, Some("More info")));
            assert_eq!(next, AgentPhase::Extract);
        }

        #[test]
        fn gather_transitions_to_extract_on_completion_signal() {
            let next = engine().next_phase(AgentPhase::Gather, &snapshot(1, Some("I'm done")));
            assert_eq!(next, AgentPhase::Extract);
        }

        #[test]
        fn gather_transitions_to_clarify_on_trigger() {
            let next = engine().next_phase(AgentPhase::Gather, &snapshot(1, Some("I'm not sure about that")));
            assert_eq!(next, AgentPhase::Clarify);
        }

        #[test]
        fn clarify_returns_to_gather_if_not_ready() {
            let next = engine().next_phase(AgentPhase::Clarify, &snapshot(1, Some("That makes sense now")));
            assert_eq!(next, AgentPhase::Gather);
        }

        #[test]
        fn clarify_transitions_to_extract_if_ready() {
            let next = engine().next_phase(AgentPhase::Clarify, &snapshot(3, Some("Got it")));
            assert_eq!(next, AgentPhase::Extract);
        }

        #[test]
        fn extract_always_transitions_to_confirm() {
            let next = engine().next_phase(AgentPhase::Extract, &snapshot(3, None));
            assert_eq!(next, AgentPhase::Confirm);
        }

        #[test]
        fn confirm_stays_on_approval() {
            let next = engine().next_phase(AgentPhase::Confirm, &snapshot(4, Some("Looks good!")));
            assert_eq!(next, AgentPhase::Confirm);
        }

        #[test]
        fn confirm_returns_to_gather_on_change_request() {
            let next = engine().next_phase(AgentPhase::Confirm, &snapshot(4, Some("Please change item 2")));
            assert_eq!(next, AgentPhase::Gather);
        }
    }

    mod is_ready_for_extraction {
        use super::*;

        fn engine() -> PhaseTransitionEngine {
            PhaseTransitionEngine::for_component(ComponentType::IssueRaising)
        }

        fn snapshot(user_msgs: usize, latest: Option<&str>) -> ConversationSnapshot {
            ConversationSnapshot::new(
                user_msgs,
                latest.map(|s| s.to_string()),
                ComponentType::IssueRaising,
            )
        }

        #[test]
        fn not_ready_with_too_few_messages() {
            assert!(!engine().is_ready_for_extraction(&snapshot(1, Some("Info"))));
        }

        #[test]
        fn ready_with_enough_messages() {
            assert!(engine().is_ready_for_extraction(&snapshot(3, Some("More info"))));
        }

        #[test]
        fn ready_with_completion_signal_even_if_few_messages() {
            assert!(engine().is_ready_for_extraction(&snapshot(1, Some("I'm done"))));
        }
    }

    mod component_specific {
        use super::*;

        #[test]
        fn consequences_requires_more_messages() {
            let engine = PhaseTransitionEngine::for_component(ComponentType::Consequences);
            let snapshot = ConversationSnapshot::new(
                3,
                Some("Rating info".to_string()),
                ComponentType::Consequences,
            );

            // 3 messages not enough for consequences (needs 4)
            assert!(!engine.is_ready_for_extraction(&snapshot));

            let snapshot = ConversationSnapshot::new(
                4,
                Some("More rating info".to_string()),
                ComponentType::Consequences,
            );

            // 4 messages is enough
            assert!(engine.is_ready_for_extraction(&snapshot));
        }

        #[test]
        fn notes_next_steps_ready_with_one_message() {
            let engine = PhaseTransitionEngine::for_component(ComponentType::NotesNextSteps);
            let snapshot = ConversationSnapshot::new(
                1,
                Some("Just one note".to_string()),
                ComponentType::NotesNextSteps,
            );

            assert!(engine.is_ready_for_extraction(&snapshot));
        }
    }

    mod completion_signals {
        use super::*;

        #[test]
        fn detects_thats_all() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("That's all I have".to_string()),
                ComponentType::IssueRaising,
            );
            let signals = PhaseTransitionConfig::default().completion_signals;
            assert!(snapshot.contains_completion_signal(&signals));
        }

        #[test]
        fn detects_thats_it() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("That's it for now".to_string()),
                ComponentType::IssueRaising,
            );
            let signals = PhaseTransitionConfig::default().completion_signals;
            assert!(snapshot.contains_completion_signal(&signals));
        }

        #[test]
        fn detects_nothing_else() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("I have nothing else to add".to_string()),
                ComponentType::IssueRaising,
            );
            let signals = PhaseTransitionConfig::default().completion_signals;
            assert!(snapshot.contains_completion_signal(&signals));
        }

        #[test]
        fn detects_no_more() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("No more thoughts from me".to_string()),
                ComponentType::IssueRaising,
            );
            let signals = PhaseTransitionConfig::default().completion_signals;
            assert!(snapshot.contains_completion_signal(&signals));
        }

        #[test]
        fn detects_ready() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("I'm ready to proceed".to_string()),
                ComponentType::IssueRaising,
            );
            let signals = PhaseTransitionConfig::default().completion_signals;
            assert!(snapshot.contains_completion_signal(&signals));
        }

        #[test]
        fn returns_false_for_none_message() {
            let snapshot = ConversationSnapshot::new(
                1,
                None,
                ComponentType::IssueRaising,
            );
            let signals = PhaseTransitionConfig::default().completion_signals;
            assert!(!snapshot.contains_completion_signal(&signals));
        }
    }

    mod clarify_triggers {
        use super::*;

        #[test]
        fn detects_question_mark() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("What does that mean?".to_string()),
                ComponentType::IssueRaising,
            );
            let triggers = PhaseTransitionConfig::default().clarify_triggers;
            assert!(snapshot.contains_clarify_trigger(&triggers));
        }

        #[test]
        fn detects_not_sure() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("I'm not sure about that".to_string()),
                ComponentType::IssueRaising,
            );
            let triggers = PhaseTransitionConfig::default().clarify_triggers;
            assert!(snapshot.contains_clarify_trigger(&triggers));
        }

        #[test]
        fn detects_maybe() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("Maybe that's right, maybe not".to_string()),
                ComponentType::IssueRaising,
            );
            let triggers = PhaseTransitionConfig::default().clarify_triggers;
            assert!(snapshot.contains_clarify_trigger(&triggers));
        }

        #[test]
        fn detects_unclear() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("This is unclear to me".to_string()),
                ComponentType::IssueRaising,
            );
            let triggers = PhaseTransitionConfig::default().clarify_triggers;
            assert!(snapshot.contains_clarify_trigger(&triggers));
        }

        #[test]
        fn detects_confused() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("I'm confused about this".to_string()),
                ComponentType::IssueRaising,
            );
            let triggers = PhaseTransitionConfig::default().clarify_triggers;
            assert!(snapshot.contains_clarify_trigger(&triggers));
        }

        #[test]
        fn returns_false_for_none_message() {
            let snapshot = ConversationSnapshot::new(
                1,
                None,
                ComponentType::IssueRaising,
            );
            let triggers = PhaseTransitionConfig::default().clarify_triggers;
            assert!(!snapshot.contains_clarify_trigger(&triggers));
        }
    }

    mod change_request_signals {
        use super::*;

        #[test]
        fn detects_modify() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("Please modify the second item".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.requests_changes());
        }

        #[test]
        fn detects_update() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("Update the description".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.requests_changes());
        }

        #[test]
        fn detects_incorrect() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("That's incorrect".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.requests_changes());
        }

        #[test]
        fn detects_fix() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("Fix the wording".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.requests_changes());
        }

        #[test]
        fn detects_revise() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("Revise the objectives".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.requests_changes());
        }

        #[test]
        fn detects_edit() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("Edit the third one".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.requests_changes());
        }

        #[test]
        fn detects_no_comma() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("No, that's not what I meant".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.requests_changes());
        }

        #[test]
        fn detects_not_quite() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("Not quite right".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.requests_changes());
        }

        #[test]
        fn detects_thats_not() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("That's not what I said".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.requests_changes());
        }

        #[test]
        fn returns_false_for_none_message() {
            let snapshot = ConversationSnapshot::new(
                1,
                None,
                ComponentType::IssueRaising,
            );
            assert!(!snapshot.requests_changes());
        }
    }

    mod approval_signals {
        use super::*;

        #[test]
        fn detects_thats_right() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("That's right, exactly".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.indicates_approval());
        }

        #[test]
        fn detects_correct() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("That is correct".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.indicates_approval());
        }

        #[test]
        fn detects_approve() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("I approve this".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.indicates_approval());
        }

        #[test]
        fn detects_confirm() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("I can confirm that".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.indicates_approval());
        }

        #[test]
        fn detects_perfect() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("Perfect, thanks!".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.indicates_approval());
        }

        #[test]
        fn detects_okay() {
            let snapshot = ConversationSnapshot::new(
                1,
                Some("Okay, that works".to_string()),
                ComponentType::IssueRaising,
            );
            assert!(snapshot.indicates_approval());
        }

        #[test]
        fn returns_false_for_none_message() {
            let snapshot = ConversationSnapshot::new(
                1,
                None,
                ComponentType::IssueRaising,
            );
            assert!(!snapshot.indicates_approval());
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn completion_signal_wins_over_clarify_trigger() {
            // If message contains both completion signal and clarify trigger,
            // the engine should favor extraction when ready
            let engine = PhaseTransitionEngine::for_component(ComponentType::IssueRaising);
            let snapshot = ConversationSnapshot::new(
                1,
                Some("I'm done? Yes, that's all.".to_string()),
                ComponentType::IssueRaising,
            );

            // Should extract because "done" and "that's all" are completion signals
            let next = engine.next_phase(AgentPhase::Gather, &snapshot);
            assert_eq!(next, AgentPhase::Extract);
        }

        #[test]
        fn clarify_trigger_only_activates_when_not_ready() {
            let engine = PhaseTransitionEngine::for_component(ComponentType::IssueRaising);
            let snapshot = ConversationSnapshot::new(
                3, // Enough messages
                Some("I'm not sure about that?".to_string()),
                ComponentType::IssueRaising,
            );

            // Should still extract because we have enough messages
            let next = engine.next_phase(AgentPhase::Gather, &snapshot);
            assert_eq!(next, AgentPhase::Extract);
        }

        #[test]
        fn empty_message_stays_in_current_phase() {
            let engine = PhaseTransitionEngine::for_component(ComponentType::IssueRaising);
            let snapshot = ConversationSnapshot::new(
                1,
                Some("".to_string()),
                ComponentType::IssueRaising,
            );

            let next = engine.next_phase(AgentPhase::Gather, &snapshot);
            assert_eq!(next, AgentPhase::Gather);
        }

        #[test]
        fn whitespace_only_message_stays_in_current_phase() {
            let engine = PhaseTransitionEngine::for_component(ComponentType::IssueRaising);
            let snapshot = ConversationSnapshot::new(
                1,
                Some("   \n\t  ".to_string()),
                ComponentType::IssueRaising,
            );

            let next = engine.next_phase(AgentPhase::Gather, &snapshot);
            assert_eq!(next, AgentPhase::Gather);
        }

        #[test]
        fn all_components_have_valid_configs() {
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
                let engine = PhaseTransitionEngine::for_component(component);
                let config = engine.config();

                // All configs should have non-empty completion signals
                assert!(
                    !config.completion_signals.is_empty(),
                    "{:?} should have completion signals",
                    component
                );

                // All configs should have non-empty clarify triggers
                assert!(
                    !config.clarify_triggers.is_empty(),
                    "{:?} should have clarify triggers",
                    component
                );

                // Min messages should be positive
                assert!(
                    config.min_messages_for_extraction >= 1,
                    "{:?} should require at least 1 message",
                    component
                );
            }
        }

        #[test]
        fn phase_transitions_form_valid_cycle() {
            // Test the complete lifecycle: Intro -> Gather -> Extract -> Confirm
            let engine = PhaseTransitionEngine::for_component(ComponentType::IssueRaising);

            // Start at Intro, send first message
            let snapshot = ConversationSnapshot::new(1, Some("Hello".to_string()), ComponentType::IssueRaising);
            let phase = engine.next_phase(AgentPhase::Intro, &snapshot);
            assert_eq!(phase, AgentPhase::Gather);

            // Continue gathering, say "done"
            let snapshot = ConversationSnapshot::new(2, Some("I'm done".to_string()), ComponentType::IssueRaising);
            let phase = engine.next_phase(AgentPhase::Gather, &snapshot);
            assert_eq!(phase, AgentPhase::Extract);

            // Extract always goes to Confirm
            let snapshot = ConversationSnapshot::new(2, None, ComponentType::IssueRaising);
            let phase = engine.next_phase(AgentPhase::Extract, &snapshot);
            assert_eq!(phase, AgentPhase::Confirm);

            // Confirm with approval stays in Confirm
            let snapshot = ConversationSnapshot::new(3, Some("Looks good".to_string()), ComponentType::IssueRaising);
            let phase = engine.next_phase(AgentPhase::Confirm, &snapshot);
            assert_eq!(phase, AgentPhase::Confirm);
        }

        #[test]
        fn confirm_can_loop_back_to_gather() {
            let engine = PhaseTransitionEngine::for_component(ComponentType::IssueRaising);

            // At confirm, request change
            let snapshot = ConversationSnapshot::new(3, Some("Please change item 1".to_string()), ComponentType::IssueRaising);
            let phase = engine.next_phase(AgentPhase::Confirm, &snapshot);
            assert_eq!(phase, AgentPhase::Gather);

            // Gather again with more info
            let snapshot = ConversationSnapshot::new(4, Some("Here's updated info".to_string()), ComponentType::IssueRaising);
            let phase = engine.next_phase(AgentPhase::Gather, &snapshot);
            assert_eq!(phase, AgentPhase::Extract);

            // Extract goes back to confirm
            let snapshot = ConversationSnapshot::new(4, None, ComponentType::IssueRaising);
            let phase = engine.next_phase(AgentPhase::Extract, &snapshot);
            assert_eq!(phase, AgentPhase::Confirm);
        }

        #[test]
        fn clarify_loop_works() {
            let engine = PhaseTransitionEngine::for_component(ComponentType::IssueRaising);

            // Gather -> Clarify on uncertainty
            let snapshot = ConversationSnapshot::new(1, Some("I'm not sure what to include".to_string()), ComponentType::IssueRaising);
            let phase = engine.next_phase(AgentPhase::Gather, &snapshot);
            assert_eq!(phase, AgentPhase::Clarify);

            // Clarify -> Gather after clarification
            let snapshot = ConversationSnapshot::new(2, Some("Okay that makes sense".to_string()), ComponentType::IssueRaising);
            let phase = engine.next_phase(AgentPhase::Clarify, &snapshot);
            assert_eq!(phase, AgentPhase::Gather);
        }
    }
}
