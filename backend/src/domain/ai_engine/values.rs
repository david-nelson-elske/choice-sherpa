//! Value Objects for AI Engine Domain
//!
//! These types represent domain concepts that are defined by their attributes
//! rather than an identity. They are immutable and can be freely copied.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::foundation::ComponentType;

/// User's intent derived from their message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserIntent {
    /// Continue working on current step
    Continue,
    /// Navigate to a specific step
    Navigate(ComponentType),
    /// Create an alternate cycle branch
    Branch,
    /// Request summary of current state
    Summarize,
    /// Signal completion of current step
    Complete,
}

/// Summary of a completed step
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepSummary {
    pub component: ComponentType,
    pub summary: String,
    pub key_outputs: Vec<String>,
    pub conflicts: Vec<String>,
    pub completed_at: DateTime<Utc>,
}

impl StepSummary {
    /// Create a new step summary
    pub fn new(
        component: ComponentType,
        summary: String,
        key_outputs: Vec<String>,
        conflicts: Vec<String>,
    ) -> Self {
        Self {
            component,
            summary,
            key_outputs,
            conflicts,
            completed_at: Utc::now(),
        }
    }

    /// Check if the summary is empty
    pub fn is_empty(&self) -> bool {
        self.summary.is_empty() && self.key_outputs.is_empty()
    }
}

/// Context passed to a step agent
#[derive(Debug, Clone)]
pub struct StepContext {
    pub component: ComponentType,
    pub prior_summaries: Vec<StepSummary>,
    pub relevant_outputs: HashMap<ComponentType, String>,
}

impl StepContext {
    /// Create a new step context
    pub fn new(component: ComponentType) -> Self {
        Self {
            component,
            prior_summaries: Vec::new(),
            relevant_outputs: HashMap::new(),
        }
    }

    /// Add a prior step summary
    pub fn with_summary(mut self, summary: StepSummary) -> Self {
        self.prior_summaries.push(summary);
        self
    }

    /// Add relevant output from another step
    pub fn with_output(mut self, component: ComponentType, output: String) -> Self {
        self.relevant_outputs.insert(component, output);
        self
    }

    /// Get summaries for a specific component
    pub fn summaries_for(&self, component: ComponentType) -> Vec<&StepSummary> {
        self.prior_summaries
            .iter()
            .filter(|s| s.component == component)
            .collect()
    }
}

/// Trait for structured output from any step
pub trait StructuredOutput: Send + Sync {
    fn component(&self) -> ComponentType;
    fn validate(&self) -> Result<(), ValidationError>;
    fn to_yaml(&self) -> Result<String, SerializationError>;
    fn as_any(&self) -> &dyn Any;
}

/// Cycle status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CycleStatus {
    Draft,
    InProgress,
    Completed,
    Abandoned,
}

/// Message ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(Uuid);

impl MessageId {
    /// Create a new message ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for MessageId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// Validation error
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum ValidationError {
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid field value: {field} - {reason}")]
    InvalidValue { field: String, reason: String },
    #[error("Validation failed: {0}")]
    Failed(String),
}

/// Serialization error
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum SerializationError {
    #[error("Serialization failed: {0}")]
    Failed(String),
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_intent_variants() {
        let intents = vec![
            UserIntent::Continue,
            UserIntent::Navigate(ComponentType::IssueRaising),
            UserIntent::Branch,
            UserIntent::Summarize,
            UserIntent::Complete,
        ];

        assert_eq!(intents.len(), 5);
        assert_eq!(intents[0], UserIntent::Continue);
    }

    #[test]
    fn test_step_summary_new() {
        let summary = StepSummary::new(
            ComponentType::IssueRaising,
            "User identified 3 decisions".to_string(),
            vec!["Decision 1".to_string()],
            vec![],
        );

        assert_eq!(summary.component, ComponentType::IssueRaising);
        assert_eq!(summary.summary, "User identified 3 decisions");
        assert_eq!(summary.key_outputs.len(), 1);
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_step_summary_is_empty() {
        let empty = StepSummary::new(
            ComponentType::IssueRaising,
            String::new(),
            vec![],
            vec![],
        );

        assert!(empty.is_empty());
    }

    #[test]
    fn test_step_context_new() {
        let context = StepContext::new(ComponentType::ProblemFrame);
        assert_eq!(context.component, ComponentType::ProblemFrame);
        assert!(context.prior_summaries.is_empty());
        assert!(context.relevant_outputs.is_empty());
    }

    #[test]
    fn test_step_context_with_summary() {
        let summary = StepSummary::new(
            ComponentType::IssueRaising,
            "Summary".to_string(),
            vec![],
            vec![],
        );

        let context = StepContext::new(ComponentType::ProblemFrame).with_summary(summary.clone());

        assert_eq!(context.prior_summaries.len(), 1);
        assert_eq!(context.prior_summaries[0], summary);
    }

    #[test]
    fn test_step_context_with_output() {
        let context = StepContext::new(ComponentType::ProblemFrame)
            .with_output(ComponentType::IssueRaising, "Output data".to_string());

        assert_eq!(context.relevant_outputs.len(), 1);
        assert_eq!(
            context.relevant_outputs[&ComponentType::IssueRaising],
            "Output data"
        );
    }

    #[test]
    fn test_step_context_summaries_for() {
        let summary1 = StepSummary::new(
            ComponentType::IssueRaising,
            "First".to_string(),
            vec![],
            vec![],
        );
        let summary2 = StepSummary::new(
            ComponentType::ProblemFrame,
            "Second".to_string(),
            vec![],
            vec![],
        );

        let context = StepContext::new(ComponentType::Objectives)
            .with_summary(summary1)
            .with_summary(summary2);

        let issue_summaries = context.summaries_for(ComponentType::IssueRaising);
        assert_eq!(issue_summaries.len(), 1);
        assert_eq!(issue_summaries[0].summary, "First");
    }

    #[test]
    fn test_cycle_status_variants() {
        let statuses = vec![
            CycleStatus::Draft,
            CycleStatus::InProgress,
            CycleStatus::Completed,
            CycleStatus::Abandoned,
        ];

        assert_eq!(statuses.len(), 4);
    }

    #[test]
    fn test_message_id_new() {
        let id1 = MessageId::new();
        let id2 = MessageId::new();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_message_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id = MessageId::from_uuid(uuid);

        assert_eq!(id.as_uuid(), &uuid);
    }

    #[test]
    fn test_message_id_display() {
        let id = MessageId::new();
        let display = format!("{}", id);

        assert!(!display.is_empty());
        assert_eq!(display.len(), 36); // UUID string length
    }

    #[test]
    fn test_message_id_from_str() {
        let uuid = Uuid::new_v4();
        let uuid_str = uuid.to_string();
        let id: MessageId = uuid_str.parse().unwrap();

        assert_eq!(id.as_uuid(), &uuid);
    }

    #[test]
    fn test_validation_error_variants() {
        let err1 = ValidationError::MissingField("name".to_string());
        let err2 = ValidationError::InvalidValue {
            field: "age".to_string(),
            reason: "must be positive".to_string(),
        };
        let err3 = ValidationError::Failed("general error".to_string());

        assert!(matches!(err1, ValidationError::MissingField(_)));
        assert!(matches!(err2, ValidationError::InvalidValue { .. }));
        assert!(matches!(err3, ValidationError::Failed(_)));
    }
}
