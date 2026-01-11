//! Conversation entity - Core conversation management.

use crate::domain::conversation::{AgentPhase, ConversationState};
use crate::domain::foundation::{ComponentId, ComponentType, ConversationId, DomainError, ErrorCode, StateMachine, Timestamp};
use crate::domain::proact::{Message, MessageId, Role};

/// Conversation entity - tracks messages and state for a component.
///
/// A conversation is bound to a single component and manages:
/// - Message history (user, assistant, system)
/// - Conversation state (Initializing, Ready, InProgress, etc.)
/// - Current agent phase (Intro, Gather, Extract, etc.)
/// - Extracted structured data
#[derive(Debug, Clone)]
pub struct Conversation {
    id: ConversationId,
    component_id: ComponentId,
    component_type: ComponentType,
    messages: Vec<Message>,
    state: ConversationState,
    current_phase: AgentPhase,
    pending_extraction: Option<serde_json::Value>,
    created_at: Timestamp,
    updated_at: Timestamp,
}

impl Conversation {
    /// Creates a new conversation for a component.
    ///
    /// Initial state is `Initializing` with `Intro` phase.
    pub fn new(component_id: ComponentId, component_type: ComponentType) -> Self {
        let now = Timestamp::now();
        Self {
            id: ConversationId::new(),
            component_id,
            component_type,
            messages: Vec::new(),
            state: ConversationState::Initializing,
            current_phase: AgentPhase::Intro,
            pending_extraction: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitutes a conversation from persistence.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ConversationId,
        component_id: ComponentId,
        component_type: ComponentType,
        messages: Vec<Message>,
        state: ConversationState,
        current_phase: AgentPhase,
        pending_extraction: Option<serde_json::Value>,
        created_at: Timestamp,
        updated_at: Timestamp,
    ) -> Self {
        Self {
            id,
            component_id,
            component_type,
            messages,
            state,
            current_phase,
            pending_extraction,
            created_at,
            updated_at,
        }
    }

    // === Accessors ===

    pub fn id(&self) -> ConversationId {
        self.id
    }

    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }

    pub fn component_type(&self) -> ComponentType {
        self.component_type
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn state(&self) -> ConversationState {
        self.state
    }

    pub fn current_phase(&self) -> AgentPhase {
        self.current_phase
    }

    pub fn pending_extraction(&self) -> Option<&serde_json::Value> {
        self.pending_extraction.as_ref()
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn user_message_count(&self) -> usize {
        self.messages.iter().filter(|m| m.role == Role::User).count()
    }

    pub fn last_message(&self) -> Option<&Message> {
        self.messages.last()
    }

    pub fn last_assistant_message(&self) -> Option<&Message> {
        self.messages.iter().rev().find(|m| m.role == Role::Assistant)
    }

    // === State Transitions ===

    /// Transitions conversation to Ready state.
    ///
    /// Can only transition from Initializing.
    pub fn mark_ready(&mut self) -> Result<(), DomainError> {
        if !self.state.can_transition_to(&ConversationState::Ready) {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!("Cannot transition from {:?} to Ready", self.state)
            ));
        }
        self.state = ConversationState::Ready;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Transitions conversation to InProgress state.
    ///
    /// Can transition from Ready or Confirmed.
    pub fn mark_in_progress(&mut self) -> Result<(), DomainError> {
        if !self.state.can_transition_to(&ConversationState::InProgress) {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!("Cannot transition from {:?} to InProgress", self.state)
            ));
        }
        self.state = ConversationState::InProgress;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Transitions conversation to Confirmed state.
    ///
    /// Can only transition from InProgress. Should have pending extraction.
    pub fn mark_confirmed(&mut self) -> Result<(), DomainError> {
        if !self.state.can_transition_to(&ConversationState::Confirmed) {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!("Cannot transition from {:?} to Confirmed", self.state)
            ));
        }
        if self.pending_extraction.is_none() {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                "Cannot confirm without pending extraction"
            ));
        }
        self.state = ConversationState::Confirmed;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Transitions conversation to Complete state.
    ///
    /// Can transition from Confirmed. Conversation becomes read-only.
    pub fn mark_complete(&mut self) -> Result<(), DomainError> {
        if !self.state.can_transition_to(&ConversationState::Complete) {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!("Cannot transition from {:?} to Complete", self.state)
            ));
        }
        self.state = ConversationState::Complete;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    // === Phase Management ===

    /// Updates the current agent phase.
    pub fn set_phase(&mut self, phase: AgentPhase) {
        self.current_phase = phase;
        self.updated_at = Timestamp::now();
    }

    /// Returns valid next phases from current phase.
    pub fn valid_next_phases(&self) -> Vec<AgentPhase> {
        self.current_phase.valid_next_phases()
    }

    /// Checks if transition to target phase is valid.
    pub fn can_transition_to_phase(&self, target: AgentPhase) -> bool {
        self.current_phase.can_transition_to(&target)
    }

    // === Message Management ===

    /// Adds a user message to the conversation.
    ///
    /// Automatically transitions to InProgress if currently Ready.
    pub fn add_user_message(&mut self, content: impl Into<String>) -> Result<&Message, DomainError> {
        if !self.state.accepts_user_input() {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!("Cannot add user message in {:?} state", self.state)
            ));
        }

        // Auto-transition to InProgress
        if self.state == ConversationState::Ready {
            self.mark_in_progress()?;
        }

        let message = Message::user(content);
        self.messages.push(message);
        self.updated_at = Timestamp::now();
        Ok(self.messages.last().unwrap())
    }

    /// Adds an assistant message to the conversation.
    pub fn add_assistant_message(&mut self, content: impl Into<String>) -> Result<&Message, DomainError> {
        if !self.state.can_generate_response() {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!("Cannot add assistant message in {:?} state", self.state)
            ));
        }

        let message = Message::assistant(content);
        self.messages.push(message);
        self.updated_at = Timestamp::now();
        Ok(self.messages.last().unwrap())
    }

    /// Adds an assistant message with a specific ID (for streaming).
    pub fn add_assistant_message_with_id(
        &mut self,
        id: MessageId,
        content: impl Into<String>,
    ) -> Result<&Message, DomainError> {
        if !self.state.can_generate_response() {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!("Cannot add assistant message in {:?} state", self.state)
            ));
        }

        let mut message = Message::assistant(content);
        message.id = id;
        self.messages.push(message);
        self.updated_at = Timestamp::now();
        Ok(self.messages.last().unwrap())
    }

    /// Adds a system message to the conversation.
    pub fn add_system_message(&mut self, content: impl Into<String>) -> Result<&Message, DomainError> {
        let message = Message::system(content);
        self.messages.push(message);
        self.updated_at = Timestamp::now();
        Ok(self.messages.last().unwrap())
    }

    /// Removes the last assistant message (for regeneration).
    pub fn remove_last_assistant_message(&mut self) -> Option<Message> {
        if let Some(pos) = self.messages.iter().rposition(|m| m.role == Role::Assistant) {
            self.updated_at = Timestamp::now();
            Some(self.messages.remove(pos))
        } else {
            None
        }
    }

    // === Extraction Management ===

    /// Sets pending extraction data.
    pub fn set_pending_extraction(&mut self, data: serde_json::Value) {
        self.pending_extraction = Some(data);
        self.updated_at = Timestamp::now();
    }

    /// Clears pending extraction.
    pub fn clear_pending_extraction(&mut self) {
        self.pending_extraction = None;
        self.updated_at = Timestamp::now();
    }

    /// Revises extracted data without losing conversation history.
    pub fn revise_extraction(&mut self, revised_data: serde_json::Value) -> Result<(), DomainError> {
        if !self.state.is_active() {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                "Cannot revise completed conversation"
            ));
        }

        // Store revision as a system message
        self.add_system_message(format!(
            "User revised extraction: {}",
            serde_json::to_string_pretty(&revised_data).unwrap_or_default()
        ))?;

        // Update pending extraction
        self.pending_extraction = Some(revised_data);

        // Return to confirm phase
        self.current_phase = AgentPhase::Confirm;
        self.updated_at = Timestamp::now();

        Ok(())
    }

    // === Context Building ===

    /// Returns messages formatted for AI context (last N messages).
    pub fn get_context_messages(&self, max_messages: usize) -> Vec<&Message> {
        let start = self.messages.len().saturating_sub(max_messages);
        self.messages[start..].iter().collect()
    }

    /// Estimates token count for the conversation.
    pub fn estimate_tokens(&self) -> u32 {
        // Rough estimate: ~4 chars per token
        let total_chars: usize = self.messages.iter().map(|m| m.content.len()).sum();
        (total_chars / 4) as u32
    }

    /// Formats conversation for extraction prompt.
    pub fn format_for_extraction(&self) -> String {
        self.messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    Role::User => "User",
                    Role::Assistant => "Assistant",
                    Role::System => "System",
                };
                format!("{}: {}", role, m.content)
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Checks if conversation contains a completion signal.
    pub fn contains_completion_signal(&self) -> bool {
        let signals = ["done", "finished", "that's all", "that's it", "complete"];
        self.messages.iter().rev().take(2).any(|m| {
            let content_lower = m.content.to_lowercase();
            signals.iter().any(|signal| content_lower.contains(signal))
        })
    }

    /// Checks if conversation mentions all of the given terms.
    pub fn mentions_all(&self, terms: &[&str]) -> bool {
        let all_content = self.messages
            .iter()
            .map(|m| m.content.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");

        terms.iter().all(|term| all_content.contains(&term.to_lowercase()))
    }

    // === Reopening ===

    /// Reopens conversation from complete state (with authorization).
    pub fn reopen(&mut self) -> Result<(), DomainError> {
        if self.state != ConversationState::Complete {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                "Can only reopen completed conversations"
            ));
        }

        self.state = ConversationState::InProgress;
        self.current_phase = AgentPhase::Gather;
        self.add_system_message("Conversation reopened for additional discussion.")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::ComponentType;

    fn create_test_conversation() -> Conversation {
        Conversation::new(
            ComponentId::new(),
            ComponentType::IssueRaising,
        )
    }

    #[test]
    fn new_conversation_starts_in_initializing_state() {
        let conv = create_test_conversation();
        assert_eq!(conv.state(), ConversationState::Initializing);
        assert_eq!(conv.current_phase(), AgentPhase::Intro);
        assert_eq!(conv.message_count(), 0);
    }

    #[test]
    fn can_transition_to_ready() {
        let mut conv = create_test_conversation();
        conv.mark_ready().unwrap();
        assert_eq!(conv.state(), ConversationState::Ready);
    }

    #[test]
    fn adding_user_message_auto_transitions_to_in_progress() {
        let mut conv = create_test_conversation();
        conv.mark_ready().unwrap();
        conv.add_user_message("Hello").unwrap();
        assert_eq!(conv.state(), ConversationState::InProgress);
        assert_eq!(conv.message_count(), 1);
    }

    #[test]
    fn can_add_assistant_message() {
        let mut conv = create_test_conversation();
        conv.mark_ready().unwrap();
        conv.add_user_message("Hello").unwrap();
        conv.add_assistant_message("Hi there!").unwrap();
        assert_eq!(conv.message_count(), 2);
    }

    #[test]
    fn last_assistant_message_finds_correct_message() {
        let mut conv = create_test_conversation();
        conv.mark_ready().unwrap();
        conv.add_user_message("Hello").unwrap();
        conv.add_assistant_message("Hi!").unwrap();
        conv.add_user_message("How are you?").unwrap();
        conv.add_assistant_message("I'm good!").unwrap();

        let last = conv.last_assistant_message().unwrap();
        assert_eq!(last.content, "I'm good!");
    }

    #[test]
    fn remove_last_assistant_message_removes_correct_message() {
        let mut conv = create_test_conversation();
        conv.mark_ready().unwrap();
        conv.add_user_message("Hello").unwrap();
        conv.add_assistant_message("Hi!").unwrap();
        conv.add_user_message("How are you?").unwrap();

        let removed = conv.remove_last_assistant_message();
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().content, "Hi!");
        assert_eq!(conv.message_count(), 2);
    }

    #[test]
    fn user_message_count_only_counts_user_messages() {
        let mut conv = create_test_conversation();
        conv.mark_ready().unwrap();
        conv.add_user_message("Hello").unwrap();
        conv.add_assistant_message("Hi!").unwrap();
        conv.add_user_message("How are you?").unwrap();
        conv.add_assistant_message("Good!").unwrap();

        assert_eq!(conv.user_message_count(), 2);
    }

    #[test]
    fn cannot_transition_to_confirmed_without_extraction() {
        let mut conv = create_test_conversation();
        conv.mark_ready().unwrap();
        conv.add_user_message("Hello").unwrap();

        let result = conv.mark_confirmed();
        assert!(result.is_err());
    }

    #[test]
    fn can_transition_to_confirmed_with_extraction() {
        let mut conv = create_test_conversation();
        conv.mark_ready().unwrap();
        conv.add_user_message("Hello").unwrap();
        conv.set_pending_extraction(serde_json::json!({"test": "data"}));

        let result = conv.mark_confirmed();
        assert!(result.is_ok());
        assert_eq!(conv.state(), ConversationState::Confirmed);
    }

    #[test]
    fn can_transition_to_complete_from_confirmed() {
        let mut conv = create_test_conversation();
        conv.mark_ready().unwrap();
        conv.add_user_message("Hello").unwrap();
        conv.set_pending_extraction(serde_json::json!({"test": "data"}));
        conv.mark_confirmed().unwrap();

        let result = conv.mark_complete();
        assert!(result.is_ok());
        assert_eq!(conv.state(), ConversationState::Complete);
    }

    #[test]
    fn cannot_add_messages_when_complete() {
        let mut conv = create_test_conversation();
        conv.mark_ready().unwrap();
        conv.add_user_message("Hello").unwrap();
        conv.set_pending_extraction(serde_json::json!({"test": "data"}));
        conv.mark_confirmed().unwrap();
        conv.mark_complete().unwrap();

        let result = conv.add_user_message("Another message");
        assert!(result.is_err());
    }

    #[test]
    fn can_reopen_completed_conversation() {
        let mut conv = create_test_conversation();
        conv.mark_ready().unwrap();
        conv.add_user_message("Hello").unwrap();
        conv.set_pending_extraction(serde_json::json!({"test": "data"}));
        conv.mark_confirmed().unwrap();
        conv.mark_complete().unwrap();

        let result = conv.reopen();
        assert!(result.is_ok());
        assert_eq!(conv.state(), ConversationState::InProgress);
        assert_eq!(conv.current_phase(), AgentPhase::Gather);
    }

    #[test]
    fn contains_completion_signal_detects_signals() {
        let mut conv = create_test_conversation();
        conv.mark_ready().unwrap();
        conv.add_user_message("That's all for now").unwrap();

        assert!(conv.contains_completion_signal());
    }

    #[test]
    fn mentions_all_checks_for_terms() {
        let mut conv = create_test_conversation();
        conv.mark_ready().unwrap();
        conv.add_user_message("I need to decide on a car").unwrap();
        conv.add_assistant_message("What matters to you?").unwrap();
        conv.add_user_message("The price and safety").unwrap();

        assert!(conv.mentions_all(&["decide", "car"]));
        assert!(conv.mentions_all(&["price", "safety"]));
        assert!(!conv.mentions_all(&["boat", "airplane"]));
    }

    #[test]
    fn valid_next_phases_returns_possible_phases() {
        let mut conv = create_test_conversation();
        assert_eq!(conv.current_phase(), AgentPhase::Intro);

        let next_phases = conv.valid_next_phases();
        assert_eq!(next_phases, vec![AgentPhase::Gather]);

        conv.set_phase(AgentPhase::Gather);
        let next_phases = conv.valid_next_phases();
        assert!(next_phases.contains(&AgentPhase::Gather));
        assert!(next_phases.contains(&AgentPhase::Clarify));
        assert!(next_phases.contains(&AgentPhase::Extract));
    }

    #[test]
    fn estimate_tokens_returns_reasonable_value() {
        let mut conv = create_test_conversation();
        conv.mark_ready().unwrap();
        conv.add_user_message("Hello there! How are you doing today?").unwrap();
        conv.add_assistant_message("I'm doing well, thank you for asking!").unwrap();

        let tokens = conv.estimate_tokens();
        assert!(tokens > 0);
        assert!(tokens < 100); // Should be around 15-20 tokens
    }
}
