//! Domain Services for AI Engine
//!
//! Service traits that define operations needed by the domain but implemented by adapters.

use async_trait::async_trait;

use crate::domain::foundation::ComponentType;

use super::{
    conversation_state::{CompressedContext, Message},
    errors::{CompressionError, ExtractionError},
    values::{StructuredOutput, UserIntent},
};

/// Classifies user intent from message content
pub trait IntentClassifier: Send + Sync {
    /// Classify user's intent from their message
    ///
    /// # Arguments
    /// * `message` - The user's message text
    /// * `current_step` - The current PrOACT step
    ///
    /// # Returns
    /// The classified intent
    fn classify(&self, message: &str, current_step: ComponentType) -> UserIntent;
}

/// Compresses conversation history for token efficiency
#[async_trait]
pub trait ContextCompressor: Send + Sync {
    /// Compress a sequence of messages into a summary
    ///
    /// # Arguments
    /// * `messages` - The messages to compress
    ///
    /// # Returns
    /// A compressed context with summary and token estimate
    ///
    /// # Errors
    /// Returns `CompressionError` if compression fails
    async fn compress(&self, messages: &[Message]) -> Result<CompressedContext, CompressionError>;
}

/// Extracts structured output from AI responses
#[async_trait]
pub trait OutputExtractor: Send + Sync {
    /// Extract structured data from an AI response
    ///
    /// # Arguments
    /// * `response` - The AI's response text
    /// * `component` - The PrOACT component type
    ///
    /// # Returns
    /// Structured output matching the component's schema
    ///
    /// # Errors
    /// Returns `ExtractionError` if extraction fails
    async fn extract(
        &self,
        response: &str,
        component: ComponentType,
    ) -> Result<Box<dyn StructuredOutput>, ExtractionError>;
}

/// Simple rule-based intent classifier (default implementation)
pub struct RuleBasedIntentClassifier;

impl IntentClassifier for RuleBasedIntentClassifier {
    fn classify(&self, message: &str, _current_step: ComponentType) -> UserIntent {
        let lowercase = message.to_lowercase();

        // Check for completion signals
        if lowercase.contains("done")
            || lowercase.contains("complete")
            || lowercase.contains("finished")
            || lowercase.contains("next step")
            || lowercase.contains("move on")
        {
            return UserIntent::Complete;
        }

        // Check for navigation signals
        if lowercase.contains("go to")
            || lowercase.contains("jump to")
            || lowercase.contains("back to")
        {
            // Try to extract component from message
            if lowercase.contains("issue") {
                return UserIntent::Navigate(ComponentType::IssueRaising);
            } else if lowercase.contains("problem") || lowercase.contains("frame") {
                return UserIntent::Navigate(ComponentType::ProblemFrame);
            } else if lowercase.contains("objective") {
                return UserIntent::Navigate(ComponentType::Objectives);
            } else if lowercase.contains("alternative") {
                return UserIntent::Navigate(ComponentType::Alternatives);
            } else if lowercase.contains("consequence") {
                return UserIntent::Navigate(ComponentType::Consequences);
            } else if lowercase.contains("tradeoff") {
                return UserIntent::Navigate(ComponentType::Tradeoffs);
            } else if lowercase.contains("recommendation") {
                return UserIntent::Navigate(ComponentType::Recommendation);
            } else if lowercase.contains("decision quality") || lowercase.contains("dq") {
                return UserIntent::Navigate(ComponentType::DecisionQuality);
            }
        }

        // Check for branch signals
        if lowercase.contains("branch")
            || lowercase.contains("what if")
            || lowercase.contains("alternative scenario")
        {
            return UserIntent::Branch;
        }

        // Check for summarize signals
        if lowercase.contains("summary")
            || lowercase.contains("recap")
            || lowercase.contains("where are we")
        {
            return UserIntent::Summarize;
        }

        // Default to continue
        UserIntent::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_based_classifier_continue() {
        let classifier = RuleBasedIntentClassifier;

        let intent = classifier.classify("I think we have three main objectives", ComponentType::Objectives);

        assert_eq!(intent, UserIntent::Continue);
    }

    #[test]
    fn test_rule_based_classifier_complete() {
        let classifier = RuleBasedIntentClassifier;

        assert_eq!(
            classifier.classify("I'm done with this step", ComponentType::IssueRaising),
            UserIntent::Complete
        );
        assert_eq!(
            classifier.classify("That's complete", ComponentType::ProblemFrame),
            UserIntent::Complete
        );
        assert_eq!(
            classifier.classify("Let's move on", ComponentType::Objectives),
            UserIntent::Complete
        );
    }

    #[test]
    fn test_rule_based_classifier_navigate() {
        let classifier = RuleBasedIntentClassifier;

        assert_eq!(
            classifier.classify("Go to objectives", ComponentType::IssueRaising),
            UserIntent::Navigate(ComponentType::Objectives)
        );
        assert_eq!(
            classifier.classify("Back to alternatives", ComponentType::Consequences),
            UserIntent::Navigate(ComponentType::Alternatives)
        );
        assert_eq!(
            classifier.classify("Jump to decision quality", ComponentType::Recommendation),
            UserIntent::Navigate(ComponentType::DecisionQuality)
        );
    }

    #[test]
    fn test_rule_based_classifier_branch() {
        let classifier = RuleBasedIntentClassifier;

        assert_eq!(
            classifier.classify("Let's create a branch", ComponentType::Alternatives),
            UserIntent::Branch
        );
        assert_eq!(
            classifier.classify("What if we tried a different approach?", ComponentType::Objectives),
            UserIntent::Branch
        );
    }

    #[test]
    fn test_rule_based_classifier_summarize() {
        let classifier = RuleBasedIntentClassifier;

        assert_eq!(
            classifier.classify("Can you give me a summary?", ComponentType::Recommendation),
            UserIntent::Summarize
        );
        assert_eq!(
            classifier.classify("Where are we in the process?", ComponentType::Consequences),
            UserIntent::Summarize
        );
    }

    #[test]
    fn test_rule_based_classifier_case_insensitive() {
        let classifier = RuleBasedIntentClassifier;

        assert_eq!(
            classifier.classify("DONE", ComponentType::IssueRaising),
            UserIntent::Complete
        );
        assert_eq!(
            classifier.classify("Go To OBJECTIVES", ComponentType::ProblemFrame),
            UserIntent::Navigate(ComponentType::Objectives)
        );
    }
}
