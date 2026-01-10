//! AI Engine Domain Module
//!
//! Provides conversational AI capabilities for guiding users through PrOACT decision components.
//! This module is designed as a port-based abstraction enabling multiple AI backends
//! (Claude Code, OpenAI API, Anthropic API) to be swapped without affecting domain logic.
//!
//! # Architecture
//!
//! - **Orchestrator**: Manages PrOACT flow within a cycle
//! - **StepAgent**: Defines behavior specifications for each PrOACT component
//! - **ConversationState**: Tracks context across the session
//! - **Domain Services**: Intent classification, context compression, output extraction
//!
//! # Example
//!
//! ```ignore
//! use ai_engine::{Orchestrator, UserIntent};
//!
//! let orchestrator = Orchestrator::new(cycle_id, ComponentType::IssueRaising);
//! let target_step = orchestrator.route(UserIntent::Continue)?;
//! ```

pub mod conversation_state;
pub mod errors;
pub mod orchestrator;
pub mod services;
pub mod step_agent;
pub mod values;

pub use conversation_state::*;
pub use errors::*;
pub use orchestrator::*;
pub use services::*;
pub use step_agent::*;
pub use values::*;
