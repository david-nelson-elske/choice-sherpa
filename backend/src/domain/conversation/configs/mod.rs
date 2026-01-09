//! Component-specific agent configurations.
//!
//! Defines tailored agent behavior for each PrOACT component,
//! including phase-specific prompts and completion criteria.

mod agent_config;
mod templates;

pub use agent_config::{
    AgentConfig, PhasePrompts, CompletionCriteria,
    agent_config_for_component,
};
pub use templates::{
    opening_message_for_component,
    extraction_prompt_for_component,
};
