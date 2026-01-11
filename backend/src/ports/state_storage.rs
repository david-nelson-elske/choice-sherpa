//! State Storage Port - Interface for persisting conversation state.
//!
//! This port defines how conversation state is saved and loaded,
//! supporting both file-based and database-backed storage.

use async_trait::async_trait;

use crate::domain::ai_engine::{values::StructuredOutput, ConversationState};
use crate::domain::foundation::{ComponentType, CycleId};

/// Errors that can occur during state storage operations
#[derive(Debug, thiserror::Error)]
pub enum StateStorageError {
    #[error("State not found for cycle: {0}")]
    NotFound(CycleId),

    #[error("Failed to serialize state: {0}")]
    SerializationFailed(String),

    #[error("Failed to deserialize state: {0}")]
    DeserializationFailed(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Output not found for cycle: {cycle_id}, component: {component:?}")]
    OutputNotFound {
        cycle_id: CycleId,
        component: ComponentType,
    },
}

/// Port for persisting and loading conversation state
#[async_trait]
pub trait StateStorage: Send + Sync {
    /// Save conversation state
    ///
    /// # Arguments
    /// * `cycle_id` - The cycle ID
    /// * `state` - The conversation state to save
    ///
    /// # Errors
    /// Returns `StateStorageError` if save fails
    async fn save_state(
        &self,
        cycle_id: CycleId,
        state: &ConversationState,
    ) -> Result<(), StateStorageError>;

    /// Load conversation state
    ///
    /// # Arguments
    /// * `cycle_id` - The cycle ID
    ///
    /// # Returns
    /// The loaded conversation state
    ///
    /// # Errors
    /// Returns `StateStorageError::NotFound` if no state exists
    async fn load_state(&self, cycle_id: CycleId) -> Result<ConversationState, StateStorageError>;

    /// Save structured output for a component
    ///
    /// # Arguments
    /// * `cycle_id` - The cycle ID
    /// * `component` - The component type
    /// * `output` - The structured output to save
    ///
    /// # Errors
    /// Returns `StateStorageError` if save fails
    async fn save_step_output(
        &self,
        cycle_id: CycleId,
        component: ComponentType,
        output: &dyn StructuredOutput,
    ) -> Result<(), StateStorageError>;

    /// Load structured output for a component
    ///
    /// # Arguments
    /// * `cycle_id` - The cycle ID
    /// * `component` - The component type
    ///
    /// # Returns
    /// The structured output as YAML string
    ///
    /// # Errors
    /// Returns `StateStorageError::OutputNotFound` if no output exists
    async fn load_step_output(
        &self,
        cycle_id: CycleId,
        component: ComponentType,
    ) -> Result<String, StateStorageError>;

    /// Check if state exists for a cycle
    ///
    /// # Arguments
    /// * `cycle_id` - The cycle ID
    ///
    /// # Returns
    /// `true` if state exists, `false` otherwise
    async fn exists(&self, cycle_id: CycleId) -> Result<bool, StateStorageError>;

    /// Delete all state for a cycle
    ///
    /// # Arguments
    /// * `cycle_id` - The cycle ID
    ///
    /// # Errors
    /// Returns `StateStorageError` if deletion fails
    async fn delete(&self, cycle_id: CycleId) -> Result<(), StateStorageError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn test_cycle_id() -> CycleId {
        CycleId::new()
    }

    #[test]
    fn test_state_storage_error_not_found() {
        let err = StateStorageError::NotFound(test_cycle_id());
        assert!(err.to_string().contains("State not found"));
    }

    #[test]
    fn test_state_storage_error_output_not_found() {
        let err = StateStorageError::OutputNotFound {
            cycle_id: test_cycle_id(),
            component: ComponentType::Objectives,
        };
        assert!(err.to_string().contains("Output not found"));
        assert!(err.to_string().contains("Objectives"));
    }

    #[test]
    fn test_state_storage_error_serialization() {
        let err = StateStorageError::SerializationFailed("Invalid YAML".to_string());
        assert!(err.to_string().contains("serialize"));
    }
}
