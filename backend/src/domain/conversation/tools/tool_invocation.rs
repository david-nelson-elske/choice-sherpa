//! Tool invocation entity - audit record for every tool call.
//!
//! Every tool invocation is logged for audit, analysis, and debugging.
//! This entity captures what was called, why, and what happened.

use serde::{Deserialize, Serialize};

use crate::domain::foundation::{
    ComponentType, CycleId, Timestamp, ToolInvocationId,
};

use super::ToolResult;

/// A recorded tool invocation for audit and analysis.
///
/// Every tool call made by the AI agent is captured as a `ToolInvocation`.
/// This provides:
/// - **Audit trail**: What tools did the agent use and why?
/// - **Performance analysis**: How long do tools take?
/// - **Debugging**: What parameters were passed?
/// - **Decision quality**: Did tools succeed or fail?
///
/// # Invariants
///
/// - `invoked_at` must be before or equal to `completed_at`
/// - `duration_ms` must equal the difference between timestamps
/// - `result_data` is present only when `result` is `Success`
///
/// # Example
///
/// ```ignore
/// use choice_sherpa::domain::conversation::tools::{ToolInvocation, ToolResult};
///
/// let invocation = ToolInvocation::new(
///     cycle_id,
///     ComponentType::Objectives,
///     "add_objective".to_string(),
///     serde_json::json!({ "name": "Minimize cost", "direction": "lower" }),
///     5, // conversation turn
///     "User mentioned wanting to reduce expenses".to_string(),
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    /// Unique identifier for this invocation
    id: ToolInvocationId,

    /// The cycle this tool was invoked within
    cycle_id: CycleId,

    /// The component that was being worked on
    component: ComponentType,

    /// Name of the tool that was invoked
    tool_name: String,

    /// Parameters passed to the tool (JSON)
    parameters: serde_json::Value,

    /// Outcome of the tool execution
    result: ToolResult,

    /// Data returned by the tool (if successful)
    result_data: Option<serde_json::Value>,

    /// Which conversation turn triggered this invocation
    conversation_turn: u32,

    /// What in the conversation triggered this tool call
    /// (helps understand agent reasoning)
    triggered_by: String,

    /// When the tool was invoked
    invoked_at: Timestamp,

    /// When the tool completed
    completed_at: Timestamp,

    /// Execution duration in milliseconds
    duration_ms: u32,
}

impl ToolInvocation {
    /// Creates a new tool invocation record (before completion).
    ///
    /// Use `complete` or `complete_with_error` to record the result.
    pub fn new(
        cycle_id: CycleId,
        component: ComponentType,
        tool_name: String,
        parameters: serde_json::Value,
        conversation_turn: u32,
        triggered_by: String,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            id: ToolInvocationId::new(),
            cycle_id,
            component,
            tool_name,
            parameters,
            result: ToolResult::Success, // Will be updated on complete
            result_data: None,
            conversation_turn,
            triggered_by,
            invoked_at: now,
            completed_at: now, // Will be updated on complete
            duration_ms: 0,    // Will be updated on complete
        }
    }

    /// Records successful completion of the tool.
    pub fn complete(&mut self, result_data: Option<serde_json::Value>) {
        let now = Timestamp::now();
        self.completed_at = now;
        self.duration_ms = self.calculate_duration_ms(now);
        self.result = ToolResult::Success;
        self.result_data = result_data;
    }

    /// Records failed completion of the tool.
    pub fn complete_with_error(&mut self, result: ToolResult, error_data: Option<serde_json::Value>) {
        debug_assert!(!result.is_success(), "Use complete() for successful results");
        let now = Timestamp::now();
        self.completed_at = now;
        self.duration_ms = self.calculate_duration_ms(now);
        self.result = result;
        self.result_data = error_data;
    }

    fn calculate_duration_ms(&self, completed: Timestamp) -> u32 {
        let duration = completed.duration_since(&self.invoked_at);
        duration.num_milliseconds().max(0) as u32
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Getters
    // ═══════════════════════════════════════════════════════════════════════

    /// Returns the unique identifier.
    pub fn id(&self) -> ToolInvocationId {
        self.id
    }

    /// Returns the cycle this invocation belongs to.
    pub fn cycle_id(&self) -> CycleId {
        self.cycle_id
    }

    /// Returns the component being worked on.
    pub fn component(&self) -> ComponentType {
        self.component
    }

    /// Returns the name of the tool.
    pub fn tool_name(&self) -> &str {
        &self.tool_name
    }

    /// Returns the parameters passed to the tool.
    pub fn parameters(&self) -> &serde_json::Value {
        &self.parameters
    }

    /// Returns the execution result.
    pub fn result(&self) -> ToolResult {
        self.result
    }

    /// Returns the result data (if any).
    pub fn result_data(&self) -> Option<&serde_json::Value> {
        self.result_data.as_ref()
    }

    /// Returns the conversation turn that triggered this.
    pub fn conversation_turn(&self) -> u32 {
        self.conversation_turn
    }

    /// Returns what triggered this invocation.
    pub fn triggered_by(&self) -> &str {
        &self.triggered_by
    }

    /// Returns when the tool was invoked.
    pub fn invoked_at(&self) -> Timestamp {
        self.invoked_at
    }

    /// Returns when the tool completed.
    pub fn completed_at(&self) -> Timestamp {
        self.completed_at
    }

    /// Returns the execution duration in milliseconds.
    pub fn duration_ms(&self) -> u32 {
        self.duration_ms
    }

    /// Returns true if the tool executed successfully.
    pub fn is_success(&self) -> bool {
        self.result.is_success()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Reconstitution (for loading from storage)
    // ═══════════════════════════════════════════════════════════════════════

    /// Reconstitutes a ToolInvocation from stored data.
    ///
    /// This bypasses validation and should only be used by repositories.
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ToolInvocationId,
        cycle_id: CycleId,
        component: ComponentType,
        tool_name: String,
        parameters: serde_json::Value,
        result: ToolResult,
        result_data: Option<serde_json::Value>,
        conversation_turn: u32,
        triggered_by: String,
        invoked_at: Timestamp,
        completed_at: Timestamp,
        duration_ms: u32,
    ) -> Self {
        Self {
            id,
            cycle_id,
            component,
            tool_name,
            parameters,
            result,
            result_data,
            conversation_turn,
            triggered_by,
            invoked_at,
            completed_at,
            duration_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cycle_id() -> CycleId {
        CycleId::new()
    }

    #[test]
    fn new_creates_invocation_with_defaults() {
        let invocation = ToolInvocation::new(
            test_cycle_id(),
            ComponentType::Objectives,
            "add_objective".to_string(),
            serde_json::json!({"name": "Test"}),
            1,
            "User said...".to_string(),
        );

        assert_eq!(invocation.tool_name(), "add_objective");
        assert_eq!(invocation.component(), ComponentType::Objectives);
        assert_eq!(invocation.conversation_turn(), 1);
        assert_eq!(invocation.triggered_by(), "User said...");
    }

    #[test]
    fn complete_records_success() {
        let mut invocation = ToolInvocation::new(
            test_cycle_id(),
            ComponentType::Alternatives,
            "add_alternative".to_string(),
            serde_json::json!({}),
            2,
            "trigger".to_string(),
        );

        invocation.complete(Some(serde_json::json!({"id": "alt-123"})));

        assert!(invocation.is_success());
        assert_eq!(invocation.result(), ToolResult::Success);
        assert!(invocation.result_data().is_some());
    }

    #[test]
    fn complete_with_error_records_failure() {
        let mut invocation = ToolInvocation::new(
            test_cycle_id(),
            ComponentType::Consequences,
            "rate_consequence".to_string(),
            serde_json::json!({}),
            3,
            "trigger".to_string(),
        );

        invocation.complete_with_error(ToolResult::NotFound, Some(serde_json::json!({"error": "Objective not found"})));

        assert!(!invocation.is_success());
        assert_eq!(invocation.result(), ToolResult::NotFound);
    }

    #[test]
    fn id_is_unique() {
        let inv1 = ToolInvocation::new(
            test_cycle_id(),
            ComponentType::IssueRaising,
            "tool".to_string(),
            serde_json::json!({}),
            1,
            "t".to_string(),
        );
        let inv2 = ToolInvocation::new(
            test_cycle_id(),
            ComponentType::IssueRaising,
            "tool".to_string(),
            serde_json::json!({}),
            1,
            "t".to_string(),
        );

        assert_ne!(inv1.id(), inv2.id());
    }

    #[test]
    fn serializes_to_json() {
        let invocation = ToolInvocation::new(
            test_cycle_id(),
            ComponentType::ProblemFrame,
            "set_focal_statement".to_string(),
            serde_json::json!({"statement": "Should we expand?"}),
            1,
            "User mentioned expansion".to_string(),
        );

        let json = serde_json::to_string(&invocation).unwrap();
        assert!(json.contains("set_focal_statement"));
        assert!(json.contains("problem_frame"));
    }

    #[test]
    fn reconstitute_preserves_all_fields() {
        let id = ToolInvocationId::new();
        let cycle_id = test_cycle_id();
        let now = Timestamp::now();

        let invocation = ToolInvocation::reconstitute(
            id,
            cycle_id,
            ComponentType::Tradeoffs,
            "mark_dominated".to_string(),
            serde_json::json!({"alt_id": "A"}),
            ToolResult::Success,
            Some(serde_json::json!({"marked": true})),
            5,
            "Analysis showed A is dominated".to_string(),
            now,
            now,
            42,
        );

        assert_eq!(invocation.id(), id);
        assert_eq!(invocation.cycle_id(), cycle_id);
        assert_eq!(invocation.duration_ms(), 42);
    }
}
