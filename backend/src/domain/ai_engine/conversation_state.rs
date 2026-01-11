//! Conversation State Entity
//!
//! Tracks the complete state of a conversation within a cycle,
//! independent of the AI provider.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::foundation::{ComponentType, CycleId, SessionId};

use super::values::{CycleStatus, MessageId};

/// Complete state of a conversation within a cycle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConversationState {
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    pub current_step: ComponentType,
    pub status: CycleStatus,
    pub branch_info: Option<BranchInfo>,
    pub step_states: HashMap<ComponentType, StepState>,
    pub message_history: Vec<Message>,
    pub compressed_context: Option<CompressedContext>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ConversationState {
    /// Create a new conversation state
    pub fn new(cycle_id: CycleId, session_id: SessionId, initial_step: ComponentType) -> Self {
        let now = Utc::now();
        let mut step_states = HashMap::new();

        // Initialize the first step as in progress
        step_states.insert(
            initial_step,
            StepState {
                status: StepStatus::InProgress,
                started_at: Some(now),
                completed_at: None,
                turn_count: 0,
                summary: None,
                key_outputs: Vec::new(),
            },
        );

        Self {
            cycle_id,
            session_id,
            current_step: initial_step,
            status: CycleStatus::InProgress,
            branch_info: None,
            step_states,
            message_history: Vec::new(),
            compressed_context: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Transition to a new step
    pub fn transition_to(&mut self, step: ComponentType) {
        // Mark current step as completed if it's in progress
        if let Some(current_state) = self.step_states.get_mut(&self.current_step) {
            if current_state.status == StepStatus::InProgress {
                current_state.status = StepStatus::Completed;
                current_state.completed_at = Some(Utc::now());
            }
        }

        // Initialize new step or resume it
        let step_state = self.step_states.entry(step).or_insert_with(|| StepState {
            status: StepStatus::NotStarted,
            started_at: None,
            completed_at: None,
            turn_count: 0,
            summary: None,
            key_outputs: Vec::new(),
        });

        // Start the step if not already started
        if step_state.status == StepStatus::NotStarted {
            step_state.status = StepStatus::InProgress;
            step_state.started_at = Some(Utc::now());
        } else if step_state.status == StepStatus::Completed {
            // Reopen completed step
            step_state.status = StepStatus::InProgress;
        }

        self.current_step = step;
        self.updated_at = Utc::now();
    }

    /// Add a message to the history
    pub fn add_message(&mut self, role: MessageRole, content: String) -> MessageId {
        let message_id = MessageId::new();
        let message = Message {
            id: message_id,
            role,
            content,
            timestamp: Utc::now(),
            step_context: self.current_step,
            metadata: None,
        };

        self.message_history.push(message);

        // Increment turn count for current step
        if let Some(step_state) = self.step_states.get_mut(&self.current_step) {
            if role == MessageRole::User {
                step_state.turn_count += 1;
            }
        }

        self.updated_at = Utc::now();
        message_id
    }

    /// Complete the current step with a summary
    pub fn complete_current_step(&mut self, summary: String, key_outputs: Vec<String>) {
        if let Some(step_state) = self.step_states.get_mut(&self.current_step) {
            step_state.status = StepStatus::Completed;
            step_state.completed_at = Some(Utc::now());
            step_state.summary = Some(summary);
            step_state.key_outputs = key_outputs;
        }

        self.updated_at = Utc::now();
    }

    /// Get messages for the current step only
    pub fn messages_for_current_step(&self) -> Vec<&Message> {
        self.message_history
            .iter()
            .filter(|m| m.step_context == self.current_step)
            .collect()
    }

    /// Get the step state for a component
    pub fn step_state(&self, component: ComponentType) -> Option<&StepState> {
        self.step_states.get(&component)
    }

    /// Check if a step is completed
    pub fn is_step_completed(&self, component: ComponentType) -> bool {
        self.step_states
            .get(&component)
            .map(|s| s.status == StepStatus::Completed)
            .unwrap_or(false)
    }

    /// Count completed steps
    pub fn completed_step_count(&self) -> usize {
        self.step_states
            .values()
            .filter(|s| s.status == StepStatus::Completed)
            .count()
    }

    /// Mark the cycle as completed
    pub fn mark_completed(&mut self) {
        self.status = CycleStatus::Completed;
        self.updated_at = Utc::now();
    }

    /// Mark the cycle as abandoned
    pub fn mark_abandoned(&mut self) {
        self.status = CycleStatus::Abandoned;
        self.updated_at = Utc::now();
    }

    /// Set branch information
    pub fn set_branch_info(&mut self, parent_cycle: CycleId, branch_point: ComponentType, branch_name: String) {
        self.branch_info = Some(BranchInfo {
            parent_cycle,
            branch_point,
            branch_name,
        });
        self.updated_at = Utc::now();
    }

    /// Set compressed context
    pub fn set_compressed_context(&mut self, summary: String, token_estimate: u32) {
        self.compressed_context = Some(CompressedContext {
            summary,
            token_estimate,
            compressed_at: Utc::now(),
        });
        self.updated_at = Utc::now();
    }
}

/// State of an individual step
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepState {
    pub status: StepStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub turn_count: u32,
    pub summary: Option<String>,
    pub key_outputs: Vec<String>,
}

/// Status of a step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    NotStarted,
    InProgress,
    Completed,
    Skipped,
}

/// Message in the conversation history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub id: MessageId,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub step_context: ComponentType,
    pub metadata: Option<MessageMetadata>,
}

/// Role of a message sender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Optional message metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageMetadata {
    pub token_count: Option<u32>,
    pub model: Option<String>,
    pub cost_cents: Option<u32>,
}

/// Branch information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BranchInfo {
    pub parent_cycle: CycleId,
    pub branch_point: ComponentType,
    pub branch_name: String,
}

/// Compressed context for token efficiency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompressedContext {
    pub summary: String,
    pub token_estimate: u32,
    pub compressed_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn test_cycle_id() -> CycleId {
        CycleId::new()
    }

    fn test_session_id() -> SessionId {
        SessionId::new()
    }

    #[test]
    fn test_conversation_state_new() {
        let state = ConversationState::new(
            test_cycle_id(),
            test_session_id(),
            ComponentType::IssueRaising,
        );

        assert_eq!(state.current_step, ComponentType::IssueRaising);
        assert_eq!(state.status, CycleStatus::InProgress);
        assert!(state.message_history.is_empty());
        assert_eq!(state.step_states.len(), 1);

        let step_state = state.step_state(ComponentType::IssueRaising).unwrap();
        assert_eq!(step_state.status, StepStatus::InProgress);
        assert!(step_state.started_at.is_some());
    }

    #[test]
    fn test_conversation_state_add_message() {
        let mut state = ConversationState::new(
            test_cycle_id(),
            test_session_id(),
            ComponentType::IssueRaising,
        );

        let msg_id = state.add_message(MessageRole::User, "Hello".to_string());

        assert_eq!(state.message_history.len(), 1);
        assert_eq!(state.message_history[0].id, msg_id);
        assert_eq!(state.message_history[0].role, MessageRole::User);
        assert_eq!(state.message_history[0].content, "Hello");

        // Check turn count incremented
        let step_state = state.step_state(ComponentType::IssueRaising).unwrap();
        assert_eq!(step_state.turn_count, 1);
    }

    #[test]
    fn test_conversation_state_transition_to() {
        let mut state = ConversationState::new(
            test_cycle_id(),
            test_session_id(),
            ComponentType::IssueRaising,
        );

        state.transition_to(ComponentType::ProblemFrame);

        assert_eq!(state.current_step, ComponentType::ProblemFrame);

        // Previous step should be completed
        let prev_state = state.step_state(ComponentType::IssueRaising).unwrap();
        assert_eq!(prev_state.status, StepStatus::Completed);
        assert!(prev_state.completed_at.is_some());

        // New step should be in progress
        let new_state = state.step_state(ComponentType::ProblemFrame).unwrap();
        assert_eq!(new_state.status, StepStatus::InProgress);
        assert!(new_state.started_at.is_some());
    }

    #[test]
    fn test_conversation_state_complete_current_step() {
        let mut state = ConversationState::new(
            test_cycle_id(),
            test_session_id(),
            ComponentType::IssueRaising,
        );

        state.complete_current_step(
            "Identified 3 key decisions".to_string(),
            vec!["Decision 1".to_string()],
        );

        let step_state = state.step_state(ComponentType::IssueRaising).unwrap();
        assert_eq!(step_state.status, StepStatus::Completed);
        assert!(step_state.completed_at.is_some());
        assert_eq!(
            step_state.summary,
            Some("Identified 3 key decisions".to_string())
        );
        assert_eq!(step_state.key_outputs.len(), 1);
    }

    #[test]
    fn test_conversation_state_messages_for_current_step() {
        let mut state = ConversationState::new(
            test_cycle_id(),
            test_session_id(),
            ComponentType::IssueRaising,
        );

        state.add_message(MessageRole::User, "Message 1".to_string());
        state.add_message(MessageRole::Assistant, "Response 1".to_string());

        state.transition_to(ComponentType::ProblemFrame);
        state.add_message(MessageRole::User, "Message 2".to_string());

        let current_messages = state.messages_for_current_step();
        assert_eq!(current_messages.len(), 1);
        assert_eq!(current_messages[0].content, "Message 2");
    }

    #[test]
    fn test_conversation_state_is_step_completed() {
        let mut state = ConversationState::new(
            test_cycle_id(),
            test_session_id(),
            ComponentType::IssueRaising,
        );

        assert!(!state.is_step_completed(ComponentType::IssueRaising));

        state.complete_current_step("Done".to_string(), vec![]);

        assert!(state.is_step_completed(ComponentType::IssueRaising));
        assert!(!state.is_step_completed(ComponentType::ProblemFrame));
    }

    #[test]
    fn test_conversation_state_completed_step_count() {
        let mut state = ConversationState::new(
            test_cycle_id(),
            test_session_id(),
            ComponentType::IssueRaising,
        );

        assert_eq!(state.completed_step_count(), 0);

        state.complete_current_step("Done".to_string(), vec![]);
        assert_eq!(state.completed_step_count(), 1);

        state.transition_to(ComponentType::ProblemFrame);
        state.complete_current_step("Done 2".to_string(), vec![]);
        assert_eq!(state.completed_step_count(), 2);
    }

    #[test]
    fn test_conversation_state_mark_completed() {
        let mut state = ConversationState::new(
            test_cycle_id(),
            test_session_id(),
            ComponentType::IssueRaising,
        );

        state.mark_completed();

        assert_eq!(state.status, CycleStatus::Completed);
    }

    #[test]
    fn test_conversation_state_mark_abandoned() {
        let mut state = ConversationState::new(
            test_cycle_id(),
            test_session_id(),
            ComponentType::IssueRaising,
        );

        state.mark_abandoned();

        assert_eq!(state.status, CycleStatus::Abandoned);
    }

    #[test]
    fn test_conversation_state_set_branch_info() {
        let mut state = ConversationState::new(
            test_cycle_id(),
            test_session_id(),
            ComponentType::IssueRaising,
        );

        let parent_cycle = CycleId::new();
        state.set_branch_info(
            parent_cycle,
            ComponentType::Alternatives,
            "What-if branch".to_string(),
        );

        assert!(state.branch_info.is_some());
        let branch_info = state.branch_info.as_ref().unwrap();
        assert_eq!(branch_info.parent_cycle, parent_cycle);
        assert_eq!(branch_info.branch_point, ComponentType::Alternatives);
        assert_eq!(branch_info.branch_name, "What-if branch");
    }

    #[test]
    fn test_conversation_state_set_compressed_context() {
        let mut state = ConversationState::new(
            test_cycle_id(),
            test_session_id(),
            ComponentType::IssueRaising,
        );

        state.set_compressed_context("Compressed summary".to_string(), 150);

        assert!(state.compressed_context.is_some());
        let context = state.compressed_context.as_ref().unwrap();
        assert_eq!(context.summary, "Compressed summary");
        assert_eq!(context.token_estimate, 150);
    }

    #[test]
    fn test_step_status_transitions() {
        let statuses = vec![
            StepStatus::NotStarted,
            StepStatus::InProgress,
            StepStatus::Completed,
            StepStatus::Skipped,
        ];

        assert_eq!(statuses.len(), 4);
    }

    #[test]
    fn test_message_role_variants() {
        let roles = vec![MessageRole::User, MessageRole::Assistant, MessageRole::System];

        assert_eq!(roles.len(), 3);
    }
}
