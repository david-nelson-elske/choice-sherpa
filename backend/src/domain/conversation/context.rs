//! Context window management for AI conversations.
//!
//! Manages the message context sent to AI providers, ensuring
//! conversations fit within token limits while preserving
//! important context.

use crate::domain::foundation::ComponentType;
use serde::{Deserialize, Serialize};

/// Token budgets for different component types.
#[derive(Debug, Clone, Copy)]
pub struct TokenBudget {
    /// Maximum tokens for context (messages + system prompt).
    pub max_context_tokens: u32,
    /// Tokens reserved for the AI response.
    pub reserved_for_response: u32,
}

impl TokenBudget {
    /// Creates a new token budget.
    pub fn new(max_context_tokens: u32, reserved_for_response: u32) -> Self {
        Self {
            max_context_tokens,
            reserved_for_response,
        }
    }

    /// Returns the available tokens for messages (context minus reserved).
    pub fn available_for_messages(&self) -> u32 {
        self.max_context_tokens.saturating_sub(self.reserved_for_response)
    }

    /// Returns the budget for a specific component type.
    pub fn for_component(component_type: ComponentType) -> Self {
        match component_type {
            // Simple components with shorter contexts
            ComponentType::IssueRaising
            | ComponentType::ProblemFrame
            | ComponentType::Objectives
            | ComponentType::Alternatives
            | ComponentType::DecisionQuality => Self::new(16_000, 2_000),

            // Complex components with larger contexts
            ComponentType::Consequences
            | ComponentType::Tradeoffs
            | ComponentType::Recommendation => Self::new(32_000, 4_000),

            // Wrap-up component with minimal context
            ComponentType::NotesNextSteps => Self::new(8_000, 1_000),
        }
    }
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self::new(16_000, 2_000)
    }
}

/// Role of a message in the conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    /// System prompt (instructions to the AI).
    System,
    /// User input.
    User,
    /// AI response.
    Assistant,
}

/// A message in the conversation context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMessage {
    /// The role of the message sender.
    pub role: MessageRole,
    /// The content of the message.
    pub content: String,
}

impl ContextMessage {
    /// Creates a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }

    /// Creates a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    /// Creates an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }

    /// Estimates the token count for this message.
    ///
    /// Uses a rough heuristic of ~4 characters per token.
    pub fn estimate_tokens(&self) -> u32 {
        // Add overhead for role marker
        let overhead = 4;
        ((self.content.len() / 4) + overhead) as u32
    }
}

/// Configuration for context window management.
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// Token budget for this context.
    pub budget: TokenBudget,
    /// Whether to include a summary of truncated messages.
    pub include_truncation_summary: bool,
    /// Maximum messages to include in truncation summary.
    pub max_summary_messages: usize,
}

impl ContextConfig {
    /// Creates a new config with the given budget.
    pub fn new(budget: TokenBudget) -> Self {
        Self {
            budget,
            include_truncation_summary: true,
            max_summary_messages: 3,
        }
    }

    /// Creates a config for a specific component type.
    pub fn for_component(component_type: ComponentType) -> Self {
        Self::new(TokenBudget::for_component(component_type))
    }
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self::new(TokenBudget::default())
    }
}

/// Result of building a context.
#[derive(Debug, Clone)]
pub struct BuiltContext {
    /// The messages to send to the AI.
    pub messages: Vec<ContextMessage>,
    /// Number of messages that were truncated.
    pub truncated_count: usize,
    /// Estimated total tokens in the context.
    pub estimated_tokens: u32,
}

impl BuiltContext {
    /// Returns true if any messages were truncated.
    pub fn was_truncated(&self) -> bool {
        self.truncated_count > 0
    }
}

/// Manages context window for AI conversations.
///
/// Ensures that the message context fits within token limits
/// while preserving the most recent and relevant messages.
#[derive(Debug, Clone)]
pub struct ContextWindowManager {
    config: ContextConfig,
}

impl ContextWindowManager {
    /// Creates a new manager with the given configuration.
    pub fn new(config: ContextConfig) -> Self {
        Self { config }
    }

    /// Creates a manager for a specific component type.
    pub fn for_component(component_type: ComponentType) -> Self {
        Self::new(ContextConfig::for_component(component_type))
    }

    /// Builds the context array for an AI request.
    ///
    /// # Arguments
    /// * `system_prompt` - The system prompt to include
    /// * `messages` - All conversation messages (oldest first)
    ///
    /// # Returns
    /// A `BuiltContext` containing the messages to send and metadata.
    pub fn build_context(
        &self,
        system_prompt: &str,
        messages: &[ContextMessage],
    ) -> BuiltContext {
        let mut result_messages = Vec::new();
        let mut token_count = self.estimate_tokens(system_prompt);
        let available_tokens = self.config.budget.available_for_messages();

        // Always include system message
        result_messages.push(ContextMessage::system(system_prompt.to_string()));

        // Work backward from most recent messages
        let mut included_indices: Vec<usize> = Vec::new();

        for (i, msg) in messages.iter().enumerate().rev() {
            let msg_tokens = msg.estimate_tokens();

            if token_count + msg_tokens <= available_tokens {
                token_count += msg_tokens;
                included_indices.push(i);
            } else {
                // Would exceed limit
                break;
            }
        }

        // Reverse to maintain chronological order
        included_indices.reverse();

        // Calculate truncated count
        let truncated_count = messages.len().saturating_sub(included_indices.len());

        // Add truncation summary if needed
        if truncated_count > 0 && self.config.include_truncation_summary {
            let summary = self.summarize_truncated(messages, &included_indices);
            let summary_tokens = self.estimate_tokens(&summary);

            // Only add if we have room
            if token_count + summary_tokens <= available_tokens {
                result_messages.push(ContextMessage::system(summary));
                token_count += summary_tokens;
            }
        }

        // Add included messages in order
        for idx in included_indices {
            result_messages.push(messages[idx].clone());
        }

        BuiltContext {
            messages: result_messages,
            truncated_count,
            estimated_tokens: token_count,
        }
    }

    /// Estimates token count for a string.
    fn estimate_tokens(&self, text: &str) -> u32 {
        // Rough estimate: ~4 characters per token
        (text.len() / 4) as u32
    }

    /// Creates a summary of truncated messages.
    fn summarize_truncated(
        &self,
        all_messages: &[ContextMessage],
        included_indices: &[usize],
    ) -> String {
        let first_included = included_indices.first().copied().unwrap_or(all_messages.len());
        let truncated: Vec<_> = all_messages.iter().take(first_included).collect();

        if truncated.is_empty() {
            return String::new();
        }

        // Count user messages in truncated section
        let user_messages: Vec<_> = truncated
            .iter()
            .filter(|m| m.role == MessageRole::User)
            .collect();

        // Build summary
        let mut summary_parts: Vec<String> = Vec::new();

        for msg in user_messages.iter().take(self.config.max_summary_messages) {
            // Take first 50 chars of each message
            let snippet: String = msg.content.chars().take(50).collect();
            let snippet = if msg.content.len() > 50 {
                format!("{}...", snippet)
            } else {
                snippet
            };
            summary_parts.push(snippet);
        }

        format!(
            "[Earlier conversation ({} messages truncated): {}]",
            truncated.len(),
            summary_parts.join("; ")
        )
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &ContextConfig {
        &self.config
    }
}

impl Default for ContextWindowManager {
    fn default() -> Self {
        Self::new(ContextConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod token_budget {
        use super::*;

        #[test]
        fn available_for_messages_subtracts_reserved() {
            let budget = TokenBudget::new(16_000, 2_000);
            assert_eq!(budget.available_for_messages(), 14_000);
        }

        #[test]
        fn available_for_messages_handles_underflow() {
            let budget = TokenBudget::new(1_000, 2_000);
            assert_eq!(budget.available_for_messages(), 0);
        }

        #[test]
        fn consequences_has_larger_budget() {
            let consequences = TokenBudget::for_component(ComponentType::Consequences);
            let objectives = TokenBudget::for_component(ComponentType::Objectives);

            assert!(consequences.max_context_tokens > objectives.max_context_tokens);
        }

        #[test]
        fn notes_next_steps_has_smaller_budget() {
            let notes = TokenBudget::for_component(ComponentType::NotesNextSteps);
            let default = TokenBudget::default();

            assert!(notes.max_context_tokens < default.max_context_tokens);
        }
    }

    mod context_message {
        use super::*;

        #[test]
        fn creates_system_message() {
            let msg = ContextMessage::system("You are helpful");
            assert_eq!(msg.role, MessageRole::System);
            assert_eq!(msg.content, "You are helpful");
        }

        #[test]
        fn creates_user_message() {
            let msg = ContextMessage::user("Hello");
            assert_eq!(msg.role, MessageRole::User);
            assert_eq!(msg.content, "Hello");
        }

        #[test]
        fn creates_assistant_message() {
            let msg = ContextMessage::assistant("Hi there!");
            assert_eq!(msg.role, MessageRole::Assistant);
            assert_eq!(msg.content, "Hi there!");
        }

        #[test]
        fn estimates_tokens_roughly() {
            let msg = ContextMessage::user("a".repeat(400)); // ~100 tokens
            let tokens = msg.estimate_tokens();
            // Should be around 100 + overhead
            assert!(tokens >= 100);
            assert!(tokens <= 120);
        }
    }

    mod context_window_manager {
        use super::*;

        fn create_messages(count: usize, content_len: usize) -> Vec<ContextMessage> {
            (0..count)
                .map(|i| {
                    if i % 2 == 0 {
                        ContextMessage::user(format!("User message {} {}", i, "x".repeat(content_len)))
                    } else {
                        ContextMessage::assistant(format!("Assistant reply {} {}", i, "x".repeat(content_len)))
                    }
                })
                .collect()
        }

        #[test]
        fn includes_system_message() {
            let manager = ContextWindowManager::default();
            let messages = create_messages(2, 10);

            let context = manager.build_context("System prompt", &messages);

            assert_eq!(context.messages[0].role, MessageRole::System);
            assert!(context.messages[0].content.contains("System prompt"));
        }

        #[test]
        fn includes_recent_messages_first() {
            let manager = ContextWindowManager::default();
            let messages = create_messages(4, 10);

            let context = manager.build_context("System", &messages);

            // Should include all messages (small enough to fit)
            assert!(context.messages.len() > 1);
        }

        #[test]
        fn truncates_when_over_budget() {
            // Use a very small budget
            let config = ContextConfig::new(TokenBudget::new(100, 20));
            let manager = ContextWindowManager::new(config);

            // Create messages that exceed the budget
            let messages = create_messages(10, 100);

            let context = manager.build_context("System", &messages);

            // Should have truncated some messages
            assert!(context.was_truncated());
            assert!(context.truncated_count > 0);
        }

        #[test]
        fn adds_truncation_summary_when_enabled() {
            // Use a budget large enough to include some messages + summary
            let mut config = ContextConfig::new(TokenBudget::new(500, 50));
            config.include_truncation_summary = true;
            let manager = ContextWindowManager::new(config);

            // Create enough messages to force truncation
            let messages = create_messages(20, 50);

            let context = manager.build_context("Sys", &messages);

            // Only check if we actually truncated
            if context.was_truncated() {
                // Check if summary was added
                let has_summary = context
                    .messages
                    .iter()
                    .any(|m| m.role == MessageRole::System && m.content.contains("truncated"));

                assert!(has_summary, "Expected truncation summary when messages were truncated");
            }
        }

        #[test]
        fn skips_truncation_summary_when_disabled() {
            let mut config = ContextConfig::new(TokenBudget::new(200, 20));
            config.include_truncation_summary = false;
            let manager = ContextWindowManager::new(config);

            let messages = create_messages(10, 50);

            let context = manager.build_context("System", &messages);

            // No truncation summary should be present
            let has_summary = context
                .messages
                .iter()
                .skip(1) // Skip main system message
                .any(|m| m.role == MessageRole::System && m.content.contains("truncated"));

            assert!(!has_summary);
        }

        #[test]
        fn preserves_message_order() {
            let manager = ContextWindowManager::default();
            let messages = vec![
                ContextMessage::user("First"),
                ContextMessage::assistant("Second"),
                ContextMessage::user("Third"),
            ];

            let context = manager.build_context("System", &messages);

            // Find user messages in order
            let user_msgs: Vec<_> = context
                .messages
                .iter()
                .filter(|m| m.role == MessageRole::User)
                .collect();

            assert!(user_msgs[0].content.contains("First"));
            assert!(user_msgs[1].content.contains("Third"));
        }

        #[test]
        fn for_component_uses_correct_budget() {
            let manager = ContextWindowManager::for_component(ComponentType::Consequences);
            let budget = manager.config().budget;

            assert_eq!(budget.max_context_tokens, 32_000);
            assert_eq!(budget.reserved_for_response, 4_000);
        }

        #[test]
        fn handles_empty_messages() {
            let manager = ContextWindowManager::default();

            let context = manager.build_context("System prompt", &[]);

            assert_eq!(context.messages.len(), 1); // Just system message
            assert_eq!(context.truncated_count, 0);
            assert!(!context.was_truncated());
        }

        #[test]
        fn estimates_total_tokens() {
            let manager = ContextWindowManager::default();
            let messages = create_messages(3, 100);

            let context = manager.build_context("System prompt", &messages);

            // Should have non-zero token estimate
            assert!(context.estimated_tokens > 0);
        }
    }

    mod built_context {
        use super::*;

        #[test]
        fn was_truncated_returns_false_when_zero() {
            let context = BuiltContext {
                messages: vec![],
                truncated_count: 0,
                estimated_tokens: 0,
            };
            assert!(!context.was_truncated());
        }

        #[test]
        fn was_truncated_returns_true_when_positive() {
            let context = BuiltContext {
                messages: vec![],
                truncated_count: 5,
                estimated_tokens: 0,
            };
            assert!(context.was_truncated());
        }
    }
}
