//! Conversation aggregate entity.
//!
//! Conversations manage AI-guided dialogues within a PrOACT component.
//! Each conversation belongs to exactly one component and contains
//! an ordered sequence of messages.
//!
//! # Aggregate Boundary
//!
//! Conversation is an aggregate root that owns its messages.
//! - Messages are created and accessed only through the Conversation
//! - Each component has at most one conversation
//! - Conversations reference components by ID (don't own them)

use crate::domain::foundation::{
    ComponentId, ConversationId, DomainError, ErrorCode, StateMachine, Timestamp,
};

use super::message::{Message, MessageId};
use super::state::ConversationState;

use serde::{Deserialize, Serialize};

/// Conversation aggregate - manages dialogue within a component.
///
/// # Invariants
///
/// - `id` is globally unique
/// - `component_id` is required and immutable
/// - Messages are ordered by creation time
/// - State transitions follow `ConversationState` rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Unique identifier for this conversation.
    id: ConversationId,

    /// The component this conversation belongs to.
    component_id: ComponentId,

    /// Current state of the conversation.
    state: ConversationState,

    /// Messages in this conversation (ordered by created_at).
    messages: Vec<Message>,

    /// When the conversation was created.
    created_at: Timestamp,

    /// When the conversation was last updated.
    updated_at: Timestamp,
}

impl Conversation {
    /// Creates a new conversation for a component.
    ///
    /// The conversation starts in `Initializing` state.
    pub fn new(id: ConversationId, component_id: ComponentId) -> Self {
        let now = Timestamp::now();
        Self {
            id,
            component_id,
            state: ConversationState::Initializing,
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitutes a conversation from persistence (no validation).
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ConversationId,
        component_id: ComponentId,
        state: ConversationState,
        messages: Vec<Message>,
        created_at: Timestamp,
        updated_at: Timestamp,
    ) -> Self {
        Self {
            id,
            component_id,
            state,
            messages,
            created_at,
            updated_at,
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Accessors
    // ─────────────────────────────────────────────────────────────────────────

    /// Returns the conversation ID.
    pub fn id(&self) -> &ConversationId {
        &self.id
    }

    /// Returns the component ID this conversation belongs to.
    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    /// Returns the current state.
    pub fn state(&self) -> ConversationState {
        self.state
    }

    /// Returns all messages in order.
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Returns the number of messages.
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Returns when the conversation was created.
    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    /// Returns when the conversation was last updated.
    pub fn updated_at(&self) -> &Timestamp {
        &self.updated_at
    }

    /// Returns the last message, if any.
    pub fn last_message(&self) -> Option<&Message> {
        self.messages.last()
    }

    /// Returns a message by ID.
    pub fn find_message(&self, id: &MessageId) -> Option<&Message> {
        self.messages.iter().find(|m| m.id() == id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // State queries
    // ─────────────────────────────────────────────────────────────────────────

    /// Returns true if the conversation is complete.
    pub fn is_complete(&self) -> bool {
        self.state.is_terminal()
    }

    /// Returns true if the conversation can accept new messages.
    pub fn can_add_message(&self) -> bool {
        self.state.is_active()
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Mutations
    // ─────────────────────────────────────────────────────────────────────────

    /// Adds a message to the conversation.
    ///
    /// Messages are appended in order. The conversation must be in an
    /// active state to accept messages.
    ///
    /// # Errors
    ///
    /// - `InvalidStateTransition` if conversation is complete
    pub fn add_message(&mut self, message: Message) -> Result<(), DomainError> {
        if !self.can_add_message() {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                "Cannot add message to a completed conversation",
            ));
        }

        self.messages.push(message);
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Transitions to the Ready state.
    ///
    /// Called after system prompt and opening message are set.
    ///
    /// # Errors
    ///
    /// - `InvalidStateTransition` if not in Initializing state
    pub fn mark_ready(&mut self) -> Result<(), DomainError> {
        self.transition_to(ConversationState::Ready)
    }

    /// Transitions to InProgress state.
    ///
    /// Called when first user message is received.
    ///
    /// # Errors
    ///
    /// - `InvalidStateTransition` if not in Ready state
    pub fn start(&mut self) -> Result<(), DomainError> {
        self.transition_to(ConversationState::InProgress)
    }

    /// Transitions to Confirmed state.
    ///
    /// Called when data is extracted and awaiting confirmation.
    ///
    /// # Errors
    ///
    /// - `InvalidStateTransition` if not in InProgress state
    pub fn confirm(&mut self) -> Result<(), DomainError> {
        self.transition_to(ConversationState::Confirmed)
    }

    /// Returns to InProgress from Confirmed.
    ///
    /// Called when user requests changes after confirmation.
    ///
    /// # Errors
    ///
    /// - `InvalidStateTransition` if not in Confirmed state
    pub fn revise(&mut self) -> Result<(), DomainError> {
        if self.state != ConversationState::Confirmed {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                "Can only revise from Confirmed state",
            ));
        }
        self.transition_to(ConversationState::InProgress)
    }

    /// Completes the conversation.
    ///
    /// # Errors
    ///
    /// - `InvalidStateTransition` if not in InProgress or Confirmed state
    pub fn complete(&mut self) -> Result<(), DomainError> {
        self.transition_to(ConversationState::Complete)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Private helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn transition_to(&mut self, target: ConversationState) -> Result<(), DomainError> {
        if !self.state.can_transition_to(&target) {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!(
                    "Cannot transition from {:?} to {:?}",
                    self.state, target
                ),
            ));
        }
        self.state = target;
        self.updated_at = Timestamp::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::foundation::StateMachine;

    use super::*;

    fn test_conversation() -> Conversation {
        Conversation::new(ConversationId::new(), ComponentId::new())
    }

    mod construction {
        use super::*;

        #[test]
        fn new_conversation_has_initializing_state() {
            let conv = test_conversation();
            assert_eq!(conv.state(), ConversationState::Initializing);
        }

        #[test]
        fn new_conversation_has_no_messages() {
            let conv = test_conversation();
            assert!(conv.messages().is_empty());
            assert_eq!(conv.message_count(), 0);
        }

        #[test]
        fn new_conversation_stores_component_id() {
            let component_id = ComponentId::new();
            let conv = Conversation::new(ConversationId::new(), component_id);
            assert_eq!(conv.component_id(), &component_id);
        }

        #[test]
        fn new_conversation_sets_timestamps() {
            let conv = test_conversation();
            assert_eq!(conv.created_at(), conv.updated_at());
        }
    }

    mod add_message {
        use super::*;

        #[test]
        fn adds_message_in_initializing_state() {
            let mut conv = test_conversation();
            let msg = Message::system("System prompt").unwrap();
            assert!(conv.add_message(msg).is_ok());
            assert_eq!(conv.message_count(), 1);
        }

        #[test]
        fn preserves_message_order() {
            let mut conv = test_conversation();
            conv.mark_ready().unwrap();

            let msg1 = Message::user("First").unwrap();
            let msg2 = Message::assistant("Second").unwrap();

            conv.add_message(msg1).unwrap();
            conv.add_message(msg2).unwrap();

            assert_eq!(conv.messages()[0].content(), "First");
            assert_eq!(conv.messages()[1].content(), "Second");
        }

        #[test]
        fn fails_when_complete() {
            let mut conv = test_conversation();
            conv.mark_ready().unwrap();
            conv.start().unwrap();
            conv.complete().unwrap();

            let msg = Message::user("Too late").unwrap();
            let result = conv.add_message(msg);
            assert!(result.is_err());
        }

        #[test]
        fn updates_timestamp() {
            let mut conv = test_conversation();
            let before = *conv.updated_at();

            // Small delay to ensure timestamp changes
            std::thread::sleep(std::time::Duration::from_millis(1));

            let msg = Message::system("Hello").unwrap();
            conv.add_message(msg).unwrap();

            assert!(conv.updated_at().as_datetime() >= before.as_datetime());
        }
    }

    mod state_transitions {
        use super::*;

        #[test]
        fn mark_ready_from_initializing() {
            let mut conv = test_conversation();
            assert!(conv.mark_ready().is_ok());
            assert_eq!(conv.state(), ConversationState::Ready);
        }

        #[test]
        fn start_from_ready() {
            let mut conv = test_conversation();
            conv.mark_ready().unwrap();
            assert!(conv.start().is_ok());
            assert_eq!(conv.state(), ConversationState::InProgress);
        }

        #[test]
        fn confirm_from_in_progress() {
            let mut conv = test_conversation();
            conv.mark_ready().unwrap();
            conv.start().unwrap();
            assert!(conv.confirm().is_ok());
            assert_eq!(conv.state(), ConversationState::Confirmed);
        }

        #[test]
        fn revise_from_confirmed() {
            let mut conv = test_conversation();
            conv.mark_ready().unwrap();
            conv.start().unwrap();
            conv.confirm().unwrap();
            assert!(conv.revise().is_ok());
            assert_eq!(conv.state(), ConversationState::InProgress);
        }

        #[test]
        fn complete_from_in_progress() {
            let mut conv = test_conversation();
            conv.mark_ready().unwrap();
            conv.start().unwrap();
            assert!(conv.complete().is_ok());
            assert_eq!(conv.state(), ConversationState::Complete);
        }

        #[test]
        fn complete_from_confirmed() {
            let mut conv = test_conversation();
            conv.mark_ready().unwrap();
            conv.start().unwrap();
            conv.confirm().unwrap();
            assert!(conv.complete().is_ok());
            assert_eq!(conv.state(), ConversationState::Complete);
        }

        #[test]
        fn cannot_skip_states() {
            let mut conv = test_conversation();
            // Cannot go directly from Initializing to InProgress
            assert!(conv.start().is_err());
        }

        #[test]
        fn cannot_revise_from_in_progress() {
            let mut conv = test_conversation();
            conv.mark_ready().unwrap();
            conv.start().unwrap();
            assert!(conv.revise().is_err());
        }
    }

    mod state_queries {
        use super::*;

        #[test]
        fn is_complete_false_initially() {
            let conv = test_conversation();
            assert!(!conv.is_complete());
        }

        #[test]
        fn is_complete_true_after_complete() {
            let mut conv = test_conversation();
            conv.mark_ready().unwrap();
            conv.start().unwrap();
            conv.complete().unwrap();
            assert!(conv.is_complete());
        }

        #[test]
        fn can_add_message_when_active() {
            let conv = test_conversation();
            assert!(conv.can_add_message());
        }

        #[test]
        fn cannot_add_message_when_complete() {
            let mut conv = test_conversation();
            conv.mark_ready().unwrap();
            conv.start().unwrap();
            conv.complete().unwrap();
            assert!(!conv.can_add_message());
        }
    }

    mod message_queries {
        use super::*;

        #[test]
        fn last_message_returns_none_when_empty() {
            let conv = test_conversation();
            assert!(conv.last_message().is_none());
        }

        #[test]
        fn last_message_returns_most_recent() {
            let mut conv = test_conversation();
            conv.add_message(Message::system("First").unwrap()).unwrap();
            conv.add_message(Message::system("Last").unwrap()).unwrap();
            assert_eq!(conv.last_message().unwrap().content(), "Last");
        }

        #[test]
        fn find_message_by_id() {
            let mut conv = test_conversation();
            let msg = Message::system("Hello").unwrap();
            let msg_id = *msg.id();
            conv.add_message(msg).unwrap();

            let found = conv.find_message(&msg_id);
            assert!(found.is_some());
            assert_eq!(found.unwrap().content(), "Hello");
        }

        #[test]
        fn find_message_returns_none_for_unknown() {
            let conv = test_conversation();
            let unknown_id = crate::domain::conversation::message::MessageId::new();
            assert!(conv.find_message(&unknown_id).is_none());
        }
    }

    mod reconstitute {
        use super::*;

        #[test]
        fn reconstitute_preserves_all_fields() {
            let id = ConversationId::new();
            let component_id = ComponentId::new();
            let state = ConversationState::InProgress;
            let messages = vec![Message::system("Test").unwrap()];
            let created_at = Timestamp::now();
            let updated_at = Timestamp::now();

            let conv = Conversation::reconstitute(
                id,
                component_id,
                state,
                messages.clone(),
                created_at,
                updated_at,
            );

            assert_eq!(conv.id(), &id);
            assert_eq!(conv.component_id(), &component_id);
            assert_eq!(conv.state(), state);
            assert_eq!(conv.messages().len(), 1);
            assert_eq!(conv.created_at(), &created_at);
            assert_eq!(conv.updated_at(), &updated_at);
        }
    }
}
