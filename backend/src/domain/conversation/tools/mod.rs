//! Atomic Decision Tools - Tool-augmented AI agent architecture.
//!
//! This module provides the domain types for tool invocation and tracking.
//! Instead of generating unstructured text, the AI agent invokes precise
//! tools that directly manipulate the decision document and component state.
//!
//! ## Key Types
//!
//! - [`ToolInvocation`] - Entity tracking every tool call for audit
//! - [`ToolResult`] - Outcome of a tool execution
//! - [`ToolCall`] - Request to invoke a tool
//! - [`ToolResponse`] - Result returned from a tool
//! - [`ToolDefinition`] - Schema and metadata for a tool
//! - [`ToolRegistry`] - Central registry for component-based tool lookup
//! - [`RevisitSuggestion`] - Queued suggestion to revisit a component
//! - [`ConfirmationRequest`] - User confirmation request from agent
//!
//! ## Design Principles
//!
//! 1. **Granularity**: Each tool does ONE thing well
//! 2. **Composability**: Complex behaviors emerge from simple tool combinations
//! 3. **Auditability**: Every invocation is logged with reasoning
//! 4. **Consistency**: Tools enforce domain invariants automatically

mod tool_result;
mod tool_invocation;
mod tool_call;
mod tool_definition;
mod tool_registry;
mod revisit_suggestion;
mod confirmation_request;

pub use tool_result::ToolResult;
pub use tool_invocation::ToolInvocation;
pub use tool_call::{ToolCall, ToolResponse};
pub use tool_definition::ToolDefinition;
pub use tool_registry::ToolRegistry;
pub use revisit_suggestion::{RevisitSuggestion, RevisitPriority, SuggestionStatus};
pub use confirmation_request::{ConfirmationRequest, ConfirmationStatus, ConfirmationOption};
