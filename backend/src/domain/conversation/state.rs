//! Conversation state machine.
//!
//! Defines the lifecycle states of a conversation and valid transitions.

use serde::{Deserialize, Serialize};

use crate::domain::foundation::StateMachine;

/// The lifecycle state of a conversation.
///
/// Conversations move through these states from creation to completion:
/// - `Initializing`: Being set up with system prompt and config
/// - `Ready`: Waiting for first user input
/// - `InProgress`: Active dialogue with user
/// - `Confirmed`: Data extracted and awaiting save
/// - `Complete`: Read-only, component finished
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConversationState {
    /// Conversation created, loading configuration.
    #[default]
    Initializing,

    /// System prompt set, opening message added, awaiting first user input.
    Ready,

    /// Active conversation with user.
    InProgress,

    /// Data extracted, awaiting user confirmation.
    Confirmed,

    /// Component completed, conversation is read-only.
    Complete,
}

impl ConversationState {
    /// Returns true if user can send messages in this state.
    pub fn accepts_user_input(&self) -> bool {
        matches!(self, Self::Ready | Self::InProgress | Self::Confirmed)
    }

    /// Returns true if AI can generate a response in this state.
    pub fn can_generate_response(&self) -> bool {
        matches!(self, Self::Ready | Self::InProgress | Self::Confirmed)
    }

    /// Returns true if the conversation is still modifiable.
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::Complete)
    }

    /// Returns true if this is a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Complete)
    }
}

impl StateMachine for ConversationState {
    fn can_transition_to(&self, target: &Self) -> bool {
        use ConversationState::*;
        matches!(
            (self, target),
            // Initial setup flow
            (Initializing, Ready) |
            // First user message starts the conversation
            (Ready, InProgress) |
            // User confirms extracted data
            (InProgress, Confirmed) |
            // User requests changes, go back to gathering
            (Confirmed, InProgress) |
            // Component completed from confirmed state
            (Confirmed, Complete) |
            // Direct completion (e.g., component externally completed)
            (InProgress, Complete)
        )
    }

    fn valid_transitions(&self) -> Vec<Self> {
        use ConversationState::*;
        match self {
            Initializing => vec![Ready],
            Ready => vec![InProgress],
            InProgress => vec![Confirmed, Complete],
            Confirmed => vec![InProgress, Complete],
            Complete => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod state_definition {
        use super::*;

        #[test]
        fn default_state_is_initializing() {
            assert_eq!(ConversationState::default(), ConversationState::Initializing);
        }

        #[test]
        fn serializes_to_snake_case() {
            let state = ConversationState::InProgress;
            let json = serde_json::to_string(&state).unwrap();
            assert_eq!(json, "\"in_progress\"");
        }

        #[test]
        fn deserializes_from_snake_case() {
            let state: ConversationState = serde_json::from_str("\"in_progress\"").unwrap();
            assert_eq!(state, ConversationState::InProgress);
        }
    }

    mod accepts_user_input {
        use super::*;

        #[test]
        fn initializing_does_not_accept_input() {
            assert!(!ConversationState::Initializing.accepts_user_input());
        }

        #[test]
        fn ready_accepts_input() {
            assert!(ConversationState::Ready.accepts_user_input());
        }

        #[test]
        fn in_progress_accepts_input() {
            assert!(ConversationState::InProgress.accepts_user_input());
        }

        #[test]
        fn confirmed_accepts_input() {
            assert!(ConversationState::Confirmed.accepts_user_input());
        }

        #[test]
        fn complete_does_not_accept_input() {
            assert!(!ConversationState::Complete.accepts_user_input());
        }
    }

    mod can_generate_response {
        use super::*;

        #[test]
        fn initializing_cannot_generate_response() {
            assert!(!ConversationState::Initializing.can_generate_response());
        }

        #[test]
        fn ready_can_generate_response() {
            assert!(ConversationState::Ready.can_generate_response());
        }

        #[test]
        fn in_progress_can_generate_response() {
            assert!(ConversationState::InProgress.can_generate_response());
        }

        #[test]
        fn confirmed_can_generate_response() {
            assert!(ConversationState::Confirmed.can_generate_response());
        }

        #[test]
        fn complete_cannot_generate_response() {
            assert!(!ConversationState::Complete.can_generate_response());
        }
    }

    mod is_active {
        use super::*;

        #[test]
        fn initializing_is_active() {
            assert!(ConversationState::Initializing.is_active());
        }

        #[test]
        fn ready_is_active() {
            assert!(ConversationState::Ready.is_active());
        }

        #[test]
        fn in_progress_is_active() {
            assert!(ConversationState::InProgress.is_active());
        }

        #[test]
        fn confirmed_is_active() {
            assert!(ConversationState::Confirmed.is_active());
        }

        #[test]
        fn complete_is_not_active() {
            assert!(!ConversationState::Complete.is_active());
        }
    }

    mod state_machine_trait {
        use super::*;

        #[test]
        fn initializing_transitions_to_ready() {
            let state = ConversationState::Initializing;
            assert!(state.can_transition_to(&ConversationState::Ready));
        }

        #[test]
        fn initializing_cannot_skip_to_in_progress() {
            let state = ConversationState::Initializing;
            assert!(!state.can_transition_to(&ConversationState::InProgress));
        }

        #[test]
        fn ready_transitions_to_in_progress() {
            let state = ConversationState::Ready;
            assert!(state.can_transition_to(&ConversationState::InProgress));
        }

        #[test]
        fn in_progress_transitions_to_confirmed() {
            let state = ConversationState::InProgress;
            assert!(state.can_transition_to(&ConversationState::Confirmed));
        }

        #[test]
        fn in_progress_transitions_to_complete() {
            let state = ConversationState::InProgress;
            assert!(state.can_transition_to(&ConversationState::Complete));
        }

        #[test]
        fn confirmed_can_return_to_in_progress() {
            let state = ConversationState::Confirmed;
            assert!(state.can_transition_to(&ConversationState::InProgress));
        }

        #[test]
        fn confirmed_transitions_to_complete() {
            let state = ConversationState::Confirmed;
            assert!(state.can_transition_to(&ConversationState::Complete));
        }

        #[test]
        fn complete_has_no_valid_transitions() {
            let state = ConversationState::Complete;
            assert!(state.valid_transitions().is_empty());
            assert!(state.is_terminal());
        }

        #[test]
        fn transition_to_succeeds_for_valid_transition() {
            let state = ConversationState::Initializing;
            let result = state.transition_to(ConversationState::Ready);
            assert_eq!(result, Ok(ConversationState::Ready));
        }

        #[test]
        fn transition_to_fails_for_invalid_transition() {
            let state = ConversationState::Initializing;
            let result = state.transition_to(ConversationState::Complete);
            assert!(result.is_err());
        }

        #[test]
        fn valid_transitions_matches_can_transition_to() {
            for state in [
                ConversationState::Initializing,
                ConversationState::Ready,
                ConversationState::InProgress,
                ConversationState::Confirmed,
                ConversationState::Complete,
            ] {
                for valid_target in state.valid_transitions() {
                    assert!(
                        state.can_transition_to(&valid_target),
                        "can_transition_to should return true for {:?} -> {:?}",
                        state,
                        valid_target
                    );
                }
            }
        }
    }
}
