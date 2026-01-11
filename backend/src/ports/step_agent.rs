//! Step Agent Port - Interface for PrOACT step-specific AI behavior.
//!
//! This port defines how AI agents behave for each PrOACT component,
//! including system prompts, tool definitions, and output parsing.

use async_trait::async_trait;

use crate::domain::ai_engine::{step_agent::StepAgentSpec, values::StructuredOutput, ExtractionError};
use crate::domain::foundation::ComponentType;

/// Tool definition for function calling
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Port for step-specific AI agent behavior
#[async_trait]
pub trait StepAgent: Send + Sync {
    /// Get the system prompt for a component
    ///
    /// # Arguments
    /// * `component` - The PrOACT component type
    ///
    /// # Returns
    /// The system prompt text
    fn get_system_prompt(&self, component: ComponentType) -> String;

    /// Get tool definitions for a component
    ///
    /// # Arguments
    /// * `component` - The PrOACT component type
    ///
    /// # Returns
    /// List of available tools for the component
    fn get_tools(&self, component: ComponentType) -> Vec<ToolDefinition>;

    /// Parse output from AI response into structured format
    ///
    /// # Arguments
    /// * `component` - The PrOACT component type
    /// * `response` - The AI's response text
    ///
    /// # Returns
    /// Structured output matching the component's schema
    ///
    /// # Errors
    /// Returns `ExtractionError` if parsing fails
    async fn parse_output(
        &self,
        component: ComponentType,
        response: &str,
    ) -> Result<Box<dyn StructuredOutput>, ExtractionError>;

    /// Get the agent specification for a component
    ///
    /// # Arguments
    /// * `component` - The PrOACT component type
    ///
    /// # Returns
    /// The complete agent specification
    fn get_spec(&self, component: ComponentType) -> Option<StepAgentSpec>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definition_fields() {
        let tool = ToolDefinition {
            name: "analyze_tradeoffs".to_string(),
            description: "Analyze tradeoffs between alternatives".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "alternatives": {"type": "array"}
                }
            }),
        };

        assert_eq!(tool.name, "analyze_tradeoffs");
        assert!(!tool.description.is_empty());
    }
}
