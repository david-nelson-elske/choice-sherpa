//! Conversation domain module.
//!
//! Manages AI-guided dialogues within each PrOACT component.
//! Handles conversation lifecycle, agent phases, and data extraction.

mod state;
mod phase;
mod engine;
mod extractor;
mod context;
pub mod configs;

pub use state::ConversationState;
pub use phase::AgentPhase;
pub use engine::{PhaseTransitionEngine, PhaseTransitionConfig, ConversationSnapshot};
pub use extractor::{
    ResponseSanitizer, DataExtractor, ExtractedData,
    SanitizationError, ExtractionError,
    MAX_RESPONSE_LENGTH, MAX_FIELD_LENGTH,
};
pub use context::{
    ContextWindowManager, ContextConfig, TokenBudget, BuiltContext,
    ContextMessage, MessageRole,
};
pub use configs::{
    AgentConfig, PhasePrompts, CompletionCriteria,
    agent_config_for_component, opening_message_for_component,
    extraction_prompt_for_component,
};
