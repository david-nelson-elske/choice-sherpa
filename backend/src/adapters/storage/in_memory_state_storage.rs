//! In-Memory State Storage Adapter
//!
//! Stores conversation state and step outputs in memory.
//! Useful for testing and development.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::ai_engine::{values::StructuredOutput, ConversationState};
use crate::domain::foundation::{ComponentType, CycleId};
use crate::ports::{StateStorage, StateStorageError};

/// In-memory storage for conversation state
#[derive(Debug, Clone)]
pub struct InMemoryStateStorage {
    states: Arc<RwLock<HashMap<CycleId, ConversationState>>>,
    outputs: Arc<RwLock<HashMap<(CycleId, ComponentType), String>>>,
}

impl InMemoryStateStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
            outputs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Clear all stored data (useful for tests)
    pub async fn clear(&self) {
        self.states.write().await.clear();
        self.outputs.write().await.clear();
    }

    /// Get the number of stored states
    pub async fn state_count(&self) -> usize {
        self.states.read().await.len()
    }

    /// Get the number of stored outputs
    pub async fn output_count(&self) -> usize {
        self.outputs.read().await.len()
    }
}

impl Default for InMemoryStateStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StateStorage for InMemoryStateStorage {
    async fn save_state(
        &self,
        cycle_id: CycleId,
        state: &ConversationState,
    ) -> Result<(), StateStorageError> {
        let mut states = self.states.write().await;
        states.insert(cycle_id, state.clone());
        Ok(())
    }

    async fn load_state(&self, cycle_id: CycleId) -> Result<ConversationState, StateStorageError> {
        let states = self.states.read().await;
        states
            .get(&cycle_id)
            .cloned()
            .ok_or(StateStorageError::NotFound(cycle_id))
    }

    async fn save_step_output(
        &self,
        cycle_id: CycleId,
        component: ComponentType,
        output: &dyn StructuredOutput,
    ) -> Result<(), StateStorageError> {
        let yaml = output
            .to_yaml()
            .map_err(|e| StateStorageError::SerializationFailed(e.to_string()))?;

        let mut outputs = self.outputs.write().await;
        outputs.insert((cycle_id, component), yaml);
        Ok(())
    }

    async fn load_step_output(
        &self,
        cycle_id: CycleId,
        component: ComponentType,
    ) -> Result<String, StateStorageError> {
        let outputs = self.outputs.read().await;
        outputs
            .get(&(cycle_id, component))
            .cloned()
            .ok_or(StateStorageError::OutputNotFound {
                cycle_id,
                component,
            })
    }

    async fn exists(&self, cycle_id: CycleId) -> Result<bool, StateStorageError> {
        let states = self.states.read().await;
        Ok(states.contains_key(&cycle_id))
    }

    async fn delete(&self, cycle_id: CycleId) -> Result<(), StateStorageError> {
        // Remove state
        self.states.write().await.remove(&cycle_id);

        // Remove all outputs for this cycle
        let mut outputs = self.outputs.write().await;
        outputs.retain(|(cid, _), _| *cid != cycle_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ai_engine::conversation_state::{ConversationState, MessageRole};
    use crate::domain::foundation::SessionId;

    // Mock structured output for testing
    struct MockStructuredOutput {
        component: ComponentType,
        data: String,
    }

    impl StructuredOutput for MockStructuredOutput {
        fn component(&self) -> ComponentType {
            self.component
        }

        fn validate(&self) -> Result<(), crate::domain::ai_engine::values::ValidationError> {
            Ok(())
        }

        fn to_yaml(&self) -> Result<String, crate::domain::ai_engine::values::SerializationError> {
            Ok(self.data.clone())
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    fn test_cycle_id() -> CycleId {
        CycleId::new()
    }

    fn test_session_id() -> SessionId {
        SessionId::new()
    }

    fn test_state(cycle_id: CycleId) -> ConversationState {
        ConversationState::new(cycle_id, test_session_id(), ComponentType::IssueRaising)
    }

    #[tokio::test]
    async fn test_memory_storage_save_and_load_state() {
        let storage = InMemoryStateStorage::new();

        let cycle_id = test_cycle_id();
        let state = test_state(cycle_id);

        // Save state
        storage.save_state(cycle_id, &state).await.unwrap();

        // Load state
        let loaded_state = storage.load_state(cycle_id).await.unwrap();

        assert_eq!(loaded_state.cycle_id, state.cycle_id);
        assert_eq!(loaded_state.current_step, state.current_step);
        assert_eq!(loaded_state.status, state.status);
    }

    #[tokio::test]
    async fn test_memory_storage_load_nonexistent_state() {
        let storage = InMemoryStateStorage::new();

        let cycle_id = test_cycle_id();

        let result = storage.load_state(cycle_id).await;

        assert!(matches!(result, Err(StateStorageError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_memory_storage_save_and_load_output() {
        let storage = InMemoryStateStorage::new();

        let cycle_id = test_cycle_id();
        let output = MockStructuredOutput {
            component: ComponentType::Objectives,
            data: "fundamental_objectives:\n  - Maximize value\n".to_string(),
        };

        // Save output
        storage
            .save_step_output(cycle_id, ComponentType::Objectives, &output)
            .await
            .unwrap();

        // Load output
        let loaded = storage
            .load_step_output(cycle_id, ComponentType::Objectives)
            .await
            .unwrap();

        assert_eq!(loaded, output.data);
    }

    #[tokio::test]
    async fn test_memory_storage_load_nonexistent_output() {
        let storage = InMemoryStateStorage::new();

        let cycle_id = test_cycle_id();

        let result = storage
            .load_step_output(cycle_id, ComponentType::Objectives)
            .await;

        assert!(matches!(
            result,
            Err(StateStorageError::OutputNotFound { .. })
        ));
    }

    #[tokio::test]
    async fn test_memory_storage_exists() {
        let storage = InMemoryStateStorage::new();

        let cycle_id = test_cycle_id();
        let state = test_state(cycle_id);

        // Should not exist initially
        assert!(!storage.exists(cycle_id).await.unwrap());

        // Save state
        storage.save_state(cycle_id, &state).await.unwrap();

        // Should exist now
        assert!(storage.exists(cycle_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_storage_delete() {
        let storage = InMemoryStateStorage::new();

        let cycle_id = test_cycle_id();
        let state = test_state(cycle_id);

        // Save state and output
        storage.save_state(cycle_id, &state).await.unwrap();
        let output = MockStructuredOutput {
            component: ComponentType::Objectives,
            data: "test".to_string(),
        };
        storage
            .save_step_output(cycle_id, ComponentType::Objectives, &output)
            .await
            .unwrap();

        // Verify exists
        assert!(storage.exists(cycle_id).await.unwrap());
        assert_eq!(storage.state_count().await, 1);
        assert_eq!(storage.output_count().await, 1);

        // Delete
        storage.delete(cycle_id).await.unwrap();

        // Should not exist anymore
        assert!(!storage.exists(cycle_id).await.unwrap());
        assert_eq!(storage.state_count().await, 0);
        assert_eq!(storage.output_count().await, 0);
    }

    #[tokio::test]
    async fn test_memory_storage_multiple_cycles() {
        let storage = InMemoryStateStorage::new();

        let cycle1 = test_cycle_id();
        let cycle2 = test_cycle_id();

        let state1 = test_state(cycle1);
        let state2 = test_state(cycle2);

        // Save both states
        storage.save_state(cycle1, &state1).await.unwrap();
        storage.save_state(cycle2, &state2).await.unwrap();

        // Load both states
        let loaded1 = storage.load_state(cycle1).await.unwrap();
        let loaded2 = storage.load_state(cycle2).await.unwrap();

        assert_eq!(loaded1.cycle_id, cycle1);
        assert_eq!(loaded2.cycle_id, cycle2);
        assert_ne!(cycle1, cycle2);
    }

    #[tokio::test]
    async fn test_memory_storage_update_state() {
        let storage = InMemoryStateStorage::new();

        let cycle_id = test_cycle_id();
        let mut state = test_state(cycle_id);

        // Save initial state
        storage.save_state(cycle_id, &state).await.unwrap();

        // Update state
        state.add_message(MessageRole::User, "Hello".to_string());
        state.transition_to(ComponentType::ProblemFrame);

        // Save updated state
        storage.save_state(cycle_id, &state).await.unwrap();

        // Load and verify
        let loaded = storage.load_state(cycle_id).await.unwrap();

        assert_eq!(loaded.current_step, ComponentType::ProblemFrame);
        assert_eq!(loaded.message_history.len(), 1);
    }

    #[tokio::test]
    async fn test_memory_storage_clear() {
        let storage = InMemoryStateStorage::new();

        let cycle1 = test_cycle_id();
        let cycle2 = test_cycle_id();

        let state1 = test_state(cycle1);
        let state2 = test_state(cycle2);

        // Save multiple states
        storage.save_state(cycle1, &state1).await.unwrap();
        storage.save_state(cycle2, &state2).await.unwrap();

        assert_eq!(storage.state_count().await, 2);

        // Clear
        storage.clear().await;

        assert_eq!(storage.state_count().await, 0);
        assert_eq!(storage.output_count().await, 0);
    }

    #[tokio::test]
    async fn test_memory_storage_delete_removes_all_outputs() {
        let storage = InMemoryStateStorage::new();

        let cycle_id = test_cycle_id();
        let state = test_state(cycle_id);

        // Save state with multiple outputs
        storage.save_state(cycle_id, &state).await.unwrap();

        let output1 = MockStructuredOutput {
            component: ComponentType::Objectives,
            data: "test1".to_string(),
        };
        let output2 = MockStructuredOutput {
            component: ComponentType::Alternatives,
            data: "test2".to_string(),
        };

        storage
            .save_step_output(cycle_id, ComponentType::Objectives, &output1)
            .await
            .unwrap();
        storage
            .save_step_output(cycle_id, ComponentType::Alternatives, &output2)
            .await
            .unwrap();

        assert_eq!(storage.output_count().await, 2);

        // Delete cycle
        storage.delete(cycle_id).await.unwrap();

        // All outputs should be removed
        assert_eq!(storage.output_count().await, 0);
    }

    #[tokio::test]
    async fn test_memory_storage_thread_safe() {
        let storage = InMemoryStateStorage::new();

        let cycle_id = test_cycle_id();
        let state = test_state(cycle_id);

        // Clone storage for concurrent access
        let storage1 = storage.clone();
        let storage2 = storage.clone();

        // Save from different tasks
        let handle1 = tokio::spawn(async move {
            storage1.save_state(cycle_id, &state).await.unwrap();
        });

        let handle2 = tokio::spawn(async move {
            // Give first task a chance to write
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            let loaded = storage2.load_state(cycle_id).await;
            assert!(loaded.is_ok());
        });

        handle1.await.unwrap();
        handle2.await.unwrap();
    }
}
