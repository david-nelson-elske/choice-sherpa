//! Tool Executor Port - Interface for executing atomic decision tools.
//!
//! This port abstracts the execution of tools invoked by the AI agent.
//! Tools are the mechanism by which the agent modifies the decision document
//! and component state.
//!
//! # Design
//!
//! - Tools are invoked with structured parameters
//! - Each tool call is validated before execution
//! - Results include both data and metadata (document updated, suggestions)
//! - Supports component-specific tool filtering
//!
//! # Example
//!
//! ```ignore
//! use async_trait::async_trait;
//! use choice_sherpa::ports::ToolExecutor;
//!
//! struct DecisionToolExecutor { /* ... */ }
//!
//! #[async_trait]
//! impl ToolExecutor for DecisionToolExecutor {
//!     async fn execute(
//!         &self,
//!         call: ToolCall,
//!         context: ToolExecutionContext,
//!     ) -> Result<ToolResponse, ToolExecutionError> {
//!         // 1. Validate parameters
//!         // 2. Execute tool logic
//!         // 3. Update document/state
//!         // 4. Return result
//!     }
//!     // ... other methods
//! }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::foundation::{ComponentType, CycleId, DomainError, ValidationError};
use crate::domain::conversation::tools::{ToolCall, ToolDefinition, ToolResponse};

/// Port for executing atomic decision tools.
///
/// Implementations handle the actual tool logic, document updates,
/// and state management. The executor is responsible for:
///
/// - Validating tool parameters against schemas
/// - Executing tool business logic
/// - Updating the decision document
/// - Generating appropriate responses
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool and return the result.
    ///
    /// # Arguments
    ///
    /// * `call` - The tool call with name and parameters
    /// * `context` - Execution context (cycle, component, etc.)
    ///
    /// # Returns
    ///
    /// * `Ok(ToolResponse)` - Tool executed (check `is_success()` for outcome)
    /// * `Err(ToolExecutionError)` - Execution failed (infra error, validation, etc.)
    async fn execute(
        &self,
        call: ToolCall,
        context: ToolExecutionContext,
    ) -> Result<ToolResponse, ToolExecutionError>;

    /// Get available tools for a specific component.
    ///
    /// Returns tool definitions that include:
    /// - Name and description
    /// - Parameter JSON Schema
    /// - Return value JSON Schema
    ///
    /// # Arguments
    ///
    /// * `component` - The component type (affects available tools)
    /// * `include_cross_cutting` - Whether to include tools available in all components
    fn available_tools(
        &self,
        component: ComponentType,
        include_cross_cutting: bool,
    ) -> Vec<ToolDefinition>;

    /// Validate tool parameters before execution.
    ///
    /// This allows early validation without executing the tool.
    /// Useful for checking AI-generated parameters.
    ///
    /// # Arguments
    ///
    /// * `call` - The tool call to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Parameters are valid
    /// * `Err(ValidationError)` - Parameters invalid (with details)
    fn validate(&self, call: &ToolCall) -> Result<(), ValidationError>;

    /// Check if a tool exists.
    fn has_tool(&self, name: &str) -> bool;

    /// Get a tool definition by name.
    fn get_tool(&self, name: &str) -> Option<ToolDefinition>;
}

/// Context for tool execution.
///
/// Provides the minimal information needed for tool execution,
/// following the "minimal context" design principle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionContext {
    /// The cycle being worked on
    pub cycle_id: CycleId,

    /// Current component
    pub current_component: ComponentType,

    /// Current conversation turn (for audit logging)
    pub conversation_turn: u32,

    /// What triggered this tool call (for audit logging)
    pub trigger: String,

    /// Summary counts (not full data)
    pub objectives_count: usize,
    pub alternatives_count: usize,

    /// Just IDs, not full objects (token efficiency)
    pub objective_ids: Vec<String>,
    pub alternative_ids: Vec<String>,
}

impl ToolExecutionContext {
    /// Creates a new execution context.
    pub fn new(
        cycle_id: CycleId,
        current_component: ComponentType,
        conversation_turn: u32,
        trigger: impl Into<String>,
    ) -> Self {
        Self {
            cycle_id,
            current_component,
            conversation_turn,
            trigger: trigger.into(),
            objectives_count: 0,
            alternatives_count: 0,
            objective_ids: Vec::new(),
            alternative_ids: Vec::new(),
        }
    }

    /// Sets objective information.
    pub fn with_objectives(mut self, count: usize, ids: Vec<String>) -> Self {
        self.objectives_count = count;
        self.objective_ids = ids;
        self
    }

    /// Sets alternative information.
    pub fn with_alternatives(mut self, count: usize, ids: Vec<String>) -> Self {
        self.alternatives_count = count;
        self.alternative_ids = ids;
        self
    }
}

/// Errors that can occur during tool execution.
#[derive(Debug, Clone, Error)]
pub enum ToolExecutionError {
    /// Tool not found
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// Parameter validation failed
    #[error("Validation error: {0}")]
    ValidationFailed(#[from] ValidationError),

    /// Domain error during execution
    #[error("Domain error: {0}")]
    DomainError(#[from] DomainError),

    /// Infrastructure/system error
    #[error("System error: {0}")]
    SystemError(String),
}

impl ToolExecutionError {
    /// Creates a system error.
    pub fn system(message: impl Into<String>) -> Self {
        Self::SystemError(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_context_new_creates_minimal_context() {
        let ctx = ToolExecutionContext::new(
            CycleId::new(),
            ComponentType::Objectives,
            5,
            "User mentioned cost concerns",
        );

        assert_eq!(ctx.current_component, ComponentType::Objectives);
        assert_eq!(ctx.conversation_turn, 5);
        assert_eq!(ctx.trigger, "User mentioned cost concerns");
        assert_eq!(ctx.objectives_count, 0);
    }

    #[test]
    fn execution_context_with_objectives_sets_counts() {
        let ctx = ToolExecutionContext::new(
            CycleId::new(),
            ComponentType::Consequences,
            1,
            "Rating cells",
        )
        .with_objectives(3, vec!["obj-1".into(), "obj-2".into(), "obj-3".into()]);

        assert_eq!(ctx.objectives_count, 3);
        assert_eq!(ctx.objective_ids.len(), 3);
    }

    #[test]
    fn execution_context_with_alternatives_sets_counts() {
        let ctx = ToolExecutionContext::new(
            CycleId::new(),
            ComponentType::Tradeoffs,
            2,
            "Analyzing dominated",
        )
        .with_alternatives(2, vec!["alt-A".into(), "alt-B".into()]);

        assert_eq!(ctx.alternatives_count, 2);
        assert_eq!(ctx.alternative_ids.len(), 2);
    }

    #[test]
    fn tool_execution_error_from_validation() {
        let validation_err = ValidationError::empty_field("name");
        let exec_err: ToolExecutionError = validation_err.into();

        assert!(matches!(exec_err, ToolExecutionError::ValidationFailed(_)));
    }

    #[test]
    fn tool_execution_error_system() {
        let err = ToolExecutionError::system("Database connection failed");
        assert!(matches!(err, ToolExecutionError::SystemError(_)));
        assert!(err.to_string().contains("Database connection"));
    }

    #[tokio::test]
    async fn tool_executor_trait_is_send_sync() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn ToolExecutor>();
    }
}
