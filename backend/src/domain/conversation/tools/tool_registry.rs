//! Tool Registry - Central registry for all available atomic decision tools.
//!
//! The registry manages tool definitions and provides component-based tool lookup.
//! It supports both component-specific tools (available only in certain components)
//! and cross-cutting tools (available in all components).
//!
//! # Example
//!
//! ```
//! use choice_sherpa::domain::conversation::tools::{ToolRegistry, ToolDefinition};
//! use choice_sherpa::domain::foundation::ComponentType;
//!
//! let mut registry = ToolRegistry::new();
//!
//! // Register a component-specific tool using the simple builder
//! let add_objective = ToolDefinition::simple(
//!     "add_objective",
//!     "Add an objective to the decision analysis",
//! );
//! registry.register_for_component("add_objective", add_objective, ComponentType::Objectives);
//!
//! // Get tools for a component
//! let tools = registry.tools_for_component(ComponentType::Objectives, true);
//! assert!(!tools.is_empty());
//! ```

use std::collections::HashMap;

use crate::domain::foundation::ComponentType;
use super::ToolDefinition;

/// Central registry for all atomic decision tools.
///
/// Manages tool definitions and provides lookup by component. Tools are
/// categorized as either component-specific or cross-cutting.
#[derive(Debug, Clone)]
pub struct ToolRegistry {
    /// All registered tools by name
    tools: HashMap<String, ToolDefinition>,

    /// Mapping from component to tool names available in that component
    component_tools: HashMap<ComponentType, Vec<String>>,

    /// Tools available in all components
    cross_cutting_tools: Vec<String>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    /// Creates a new empty tool registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            component_tools: HashMap::new(),
            cross_cutting_tools: Vec::new(),
        }
    }

    /// Registers a tool for a specific component.
    ///
    /// The tool will only be available when working on that component.
    pub fn register_for_component(
        &mut self,
        name: impl Into<String>,
        definition: ToolDefinition,
        component: ComponentType,
    ) {
        let name = name.into();
        self.tools.insert(name.clone(), definition);
        self.component_tools
            .entry(component)
            .or_default()
            .push(name);
    }

    /// Registers a tool for multiple components.
    pub fn register_for_components(
        &mut self,
        name: impl Into<String>,
        definition: ToolDefinition,
        components: &[ComponentType],
    ) {
        let name = name.into();
        self.tools.insert(name.clone(), definition);
        for component in components {
            self.component_tools
                .entry(*component)
                .or_default()
                .push(name.clone());
        }
    }

    /// Registers a cross-cutting tool available in all components.
    pub fn register_cross_cutting(
        &mut self,
        name: impl Into<String>,
        definition: ToolDefinition,
    ) {
        let name = name.into();
        self.tools.insert(name.clone(), definition);
        self.cross_cutting_tools.push(name);
    }

    /// Gets all tools available for a component.
    ///
    /// # Arguments
    ///
    /// * `component` - The component to get tools for
    /// * `include_cross_cutting` - Whether to include cross-cutting tools
    ///
    /// Returns tool definitions for the component, with component-specific
    /// tools first, followed by cross-cutting tools.
    pub fn tools_for_component(
        &self,
        component: ComponentType,
        include_cross_cutting: bool,
    ) -> Vec<&ToolDefinition> {
        let mut tools: Vec<&ToolDefinition> = Vec::new();

        // Add component-specific tools
        if let Some(tool_names) = self.component_tools.get(&component) {
            for name in tool_names {
                if let Some(tool) = self.tools.get(name) {
                    tools.push(tool);
                }
            }
        }

        // Add cross-cutting tools if requested
        if include_cross_cutting {
            for name in &self.cross_cutting_tools {
                if let Some(tool) = self.tools.get(name) {
                    tools.push(tool);
                }
            }
        }

        tools
    }

    /// Gets a tool definition by name.
    pub fn get_tool(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    /// Checks if a tool is registered.
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Checks if a tool is available for a specific component.
    pub fn is_available_for_component(&self, name: &str, component: ComponentType) -> bool {
        // Check if it's a cross-cutting tool
        if self.cross_cutting_tools.contains(&name.to_string()) {
            return true;
        }

        // Check if it's registered for this component
        if let Some(tool_names) = self.component_tools.get(&component) {
            return tool_names.contains(&name.to_string());
        }

        false
    }

    /// Returns all registered tool names.
    pub fn all_tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// Returns the number of registered tools.
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// Returns the number of cross-cutting tools.
    pub fn cross_cutting_count(&self) -> usize {
        self.cross_cutting_tools.len()
    }

    /// Converts tools for a component to OpenAI function format.
    ///
    /// Returns a JSON array of tool definitions in OpenAI's function calling format.
    pub fn to_openai_tools(&self, component: ComponentType) -> Vec<serde_json::Value> {
        self.tools_for_component(component, true)
            .iter()
            .map(|tool| tool.to_openai_format())
            .collect()
    }

    /// Converts tools for a component to Anthropic tool format.
    ///
    /// Returns a JSON array of tool definitions in Anthropic's tool use format.
    pub fn to_anthropic_tools(&self, component: ComponentType) -> Vec<serde_json::Value> {
        self.tools_for_component(component, true)
            .iter()
            .map(|tool| tool.to_anthropic_format())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tool(name: &str) -> ToolDefinition {
        ToolDefinition::simple(name, format!("Description for {}", name))
    }

    #[test]
    fn new_registry_is_empty() {
        let registry = ToolRegistry::new();

        assert_eq!(registry.tool_count(), 0);
        assert_eq!(registry.cross_cutting_count(), 0);
    }

    #[test]
    fn register_for_component_adds_tool() {
        let mut registry = ToolRegistry::new();
        registry.register_for_component(
            "add_objective",
            sample_tool("add_objective"),
            ComponentType::Objectives,
        );

        assert!(registry.has_tool("add_objective"));
        assert_eq!(registry.tool_count(), 1);
    }

    #[test]
    fn tools_for_component_returns_correct_tools() {
        let mut registry = ToolRegistry::new();
        registry.register_for_component(
            "add_objective",
            sample_tool("add_objective"),
            ComponentType::Objectives,
        );
        registry.register_for_component(
            "add_alternative",
            sample_tool("add_alternative"),
            ComponentType::Alternatives,
        );

        let objectives_tools = registry.tools_for_component(ComponentType::Objectives, false);
        assert_eq!(objectives_tools.len(), 1);
        assert_eq!(objectives_tools[0].name(), "add_objective");

        let alternatives_tools = registry.tools_for_component(ComponentType::Alternatives, false);
        assert_eq!(alternatives_tools.len(), 1);
        assert_eq!(alternatives_tools[0].name(), "add_alternative");
    }

    #[test]
    fn cross_cutting_tools_appear_in_all_components() {
        let mut registry = ToolRegistry::new();
        registry.register_cross_cutting("flag_uncertainty", sample_tool("flag_uncertainty"));

        let objectives_tools = registry.tools_for_component(ComponentType::Objectives, true);
        assert_eq!(objectives_tools.len(), 1);

        let alternatives_tools = registry.tools_for_component(ComponentType::Alternatives, true);
        assert_eq!(alternatives_tools.len(), 1);

        // Cross-cutting excluded
        let no_cross = registry.tools_for_component(ComponentType::Objectives, false);
        assert_eq!(no_cross.len(), 0);
    }

    #[test]
    fn is_available_for_component_checks_correctly() {
        let mut registry = ToolRegistry::new();
        registry.register_for_component(
            "add_objective",
            sample_tool("add_objective"),
            ComponentType::Objectives,
        );
        registry.register_cross_cutting("flag_uncertainty", sample_tool("flag_uncertainty"));

        // Component-specific tool
        assert!(registry.is_available_for_component("add_objective", ComponentType::Objectives));
        assert!(!registry.is_available_for_component("add_objective", ComponentType::Alternatives));

        // Cross-cutting tool available everywhere
        assert!(registry.is_available_for_component("flag_uncertainty", ComponentType::Objectives));
        assert!(registry.is_available_for_component("flag_uncertainty", ComponentType::Alternatives));
        assert!(registry.is_available_for_component("flag_uncertainty", ComponentType::Consequences));
    }

    #[test]
    fn register_for_multiple_components() {
        let mut registry = ToolRegistry::new();
        registry.register_for_components(
            "rate_consequence",
            sample_tool("rate_consequence"),
            &[ComponentType::Consequences, ComponentType::Tradeoffs],
        );

        assert!(registry.is_available_for_component("rate_consequence", ComponentType::Consequences));
        assert!(registry.is_available_for_component("rate_consequence", ComponentType::Tradeoffs));
        assert!(!registry.is_available_for_component("rate_consequence", ComponentType::Objectives));
    }

    #[test]
    fn get_tool_returns_definition() {
        let mut registry = ToolRegistry::new();
        let tool = ToolDefinition::simple("add_objective", "Add an objective")
            .with_parameter("name", "string", "Objective name", true);
        registry.register_for_component("add_objective", tool, ComponentType::Objectives);

        let retrieved = registry.get_tool("add_objective").unwrap();
        assert_eq!(retrieved.name(), "add_objective");
        assert!(!retrieved.parameters_schema().is_null());
    }

    #[test]
    fn to_openai_tools_returns_formatted_tools() {
        let mut registry = ToolRegistry::new();
        registry.register_for_component(
            "add_objective",
            sample_tool("add_objective"),
            ComponentType::Objectives,
        );

        let openai_tools = registry.to_openai_tools(ComponentType::Objectives);
        assert_eq!(openai_tools.len(), 1);
        assert_eq!(openai_tools[0]["type"], "function");
        assert_eq!(openai_tools[0]["function"]["name"], "add_objective");
    }

    #[test]
    fn to_anthropic_tools_returns_formatted_tools() {
        let mut registry = ToolRegistry::new();
        registry.register_for_component(
            "add_objective",
            sample_tool("add_objective"),
            ComponentType::Objectives,
        );

        let anthropic_tools = registry.to_anthropic_tools(ComponentType::Objectives);
        assert_eq!(anthropic_tools.len(), 1);
        assert_eq!(anthropic_tools[0]["name"], "add_objective");
    }

    #[test]
    fn all_tool_names_returns_registered_tools() {
        let mut registry = ToolRegistry::new();
        registry.register_for_component("tool_a", sample_tool("tool_a"), ComponentType::Objectives);
        registry.register_cross_cutting("tool_b", sample_tool("tool_b"));

        let names = registry.all_tool_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"tool_a"));
        assert!(names.contains(&"tool_b"));
    }
}
