//! Tool Invocation Repository Port - Persistence for tool invocation audit records.
//!
//! This port abstracts storage of tool invocation records for audit,
//! analysis, and debugging purposes.
//!
//! # Example
//!
//! ```ignore
//! use async_trait::async_trait;
//! use choice_sherpa::ports::ToolInvocationRepository;
//!
//! struct PostgresToolInvocationRepository { /* ... */ }
//!
//! #[async_trait]
//! impl ToolInvocationRepository for PostgresToolInvocationRepository {
//!     async fn save(&self, invocation: ToolInvocation) -> Result<(), ToolInvocationRepoError> {
//!         // Insert into tool_invocations table
//!     }
//!     // ... other methods
//! }
//! ```

use async_trait::async_trait;
use thiserror::Error;

use crate::domain::foundation::{ComponentType, CycleId, ToolInvocationId};
use crate::domain::conversation::tools::{ToolInvocation, ToolResult};

/// Port for tool invocation persistence.
///
/// Stores tool invocations for audit trail, analytics, and debugging.
/// Every tool call made by the AI agent is recorded.
#[async_trait]
pub trait ToolInvocationRepository: Send + Sync {
    /// Save a tool invocation record.
    ///
    /// Called after each tool execution completes.
    async fn save(&self, invocation: ToolInvocation) -> Result<(), ToolInvocationRepoError>;

    /// Find a tool invocation by ID.
    async fn find_by_id(
        &self,
        id: ToolInvocationId,
    ) -> Result<Option<ToolInvocation>, ToolInvocationRepoError>;

    /// Find all tool invocations for a cycle.
    ///
    /// Returns invocations ordered by invoked_at (oldest first).
    async fn find_by_cycle(
        &self,
        cycle_id: CycleId,
    ) -> Result<Vec<ToolInvocation>, ToolInvocationRepoError>;

    /// Find tool invocations by cycle and component.
    async fn find_by_cycle_and_component(
        &self,
        cycle_id: CycleId,
        component: ComponentType,
    ) -> Result<Vec<ToolInvocation>, ToolInvocationRepoError>;

    /// Find recent tool invocations for a cycle (last N).
    async fn find_recent(
        &self,
        cycle_id: CycleId,
        limit: usize,
    ) -> Result<Vec<ToolInvocation>, ToolInvocationRepoError>;

    /// Count tool invocations by result type for a cycle.
    ///
    /// Returns a map of ToolResult -> count for analytics.
    async fn count_by_result(
        &self,
        cycle_id: CycleId,
    ) -> Result<ToolInvocationStats, ToolInvocationRepoError>;
}

/// Statistics about tool invocations.
#[derive(Debug, Clone, Default)]
pub struct ToolInvocationStats {
    /// Total number of invocations
    pub total: usize,
    /// Number of successful invocations
    pub success: usize,
    /// Number of validation errors
    pub validation_errors: usize,
    /// Number of not-found errors
    pub not_found: usize,
    /// Number of conflict errors
    pub conflicts: usize,
    /// Number of internal errors
    pub internal_errors: usize,
    /// Average duration in milliseconds
    pub avg_duration_ms: u32,
}

impl ToolInvocationStats {
    /// Returns the success rate as a percentage (0.0 - 100.0).
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.success as f64 / self.total as f64) * 100.0
        }
    }

    /// Increments the counter for a result type.
    pub fn record(&mut self, result: ToolResult) {
        self.total += 1;
        match result {
            ToolResult::Success => self.success += 1,
            ToolResult::ValidationError => self.validation_errors += 1,
            ToolResult::NotFound => self.not_found += 1,
            ToolResult::Conflict => self.conflicts += 1,
            ToolResult::InternalError => self.internal_errors += 1,
        }
    }
}

/// Errors from the tool invocation repository.
#[derive(Debug, Clone, Error)]
pub enum ToolInvocationRepoError {
    /// Database or storage error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl ToolInvocationRepoError {
    /// Creates a storage error.
    pub fn storage(message: impl Into<String>) -> Self {
        Self::StorageError(message.into())
    }

    /// Creates a serialization error.
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::SerializationError(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_success_rate_when_all_success() {
        let stats = ToolInvocationStats {
            total: 10,
            success: 10,
            ..Default::default()
        };

        assert!((stats.success_rate() - 100.0).abs() < 0.01);
    }

    #[test]
    fn stats_success_rate_when_mixed() {
        let stats = ToolInvocationStats {
            total: 10,
            success: 8,
            validation_errors: 2,
            ..Default::default()
        };

        assert!((stats.success_rate() - 80.0).abs() < 0.01);
    }

    #[test]
    fn stats_success_rate_when_empty() {
        let stats = ToolInvocationStats::default();
        assert!((stats.success_rate() - 0.0).abs() < 0.01);
    }

    #[test]
    fn stats_record_increments_counters() {
        let mut stats = ToolInvocationStats::default();

        stats.record(ToolResult::Success);
        stats.record(ToolResult::Success);
        stats.record(ToolResult::ValidationError);
        stats.record(ToolResult::NotFound);

        assert_eq!(stats.total, 4);
        assert_eq!(stats.success, 2);
        assert_eq!(stats.validation_errors, 1);
        assert_eq!(stats.not_found, 1);
    }

    #[tokio::test]
    async fn tool_invocation_repository_trait_is_send_sync() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn ToolInvocationRepository>();
    }
}
