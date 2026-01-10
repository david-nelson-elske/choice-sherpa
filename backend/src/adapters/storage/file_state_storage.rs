//! File-based State Storage Adapter
//!
//! Stores conversation state and step outputs as YAML files on disk.
//! Organized by cycle_id for easy navigation and debugging.

use async_trait::async_trait;
use serde_yaml;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::domain::ai_engine::{values::StructuredOutput, ConversationState};
use crate::domain::foundation::{ComponentType, CycleId};
use crate::ports::{StateStorage, StateStorageError};

/// File-based storage for conversation state
#[derive(Debug, Clone)]
pub struct FileStateStorage {
    base_path: PathBuf,
}

impl FileStateStorage {
    /// Create a new file storage with a base directory
    ///
    /// # Arguments
    /// * `base_path` - The root directory for storing conversation data
    ///
    /// # Example
    /// ```ignore
    /// let storage = FileStateStorage::new("./data/conversations");
    /// ```
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Get the directory path for a specific cycle
    fn cycle_dir(&self, cycle_id: CycleId) -> PathBuf {
        self.base_path.join(cycle_id.to_string())
    }

    /// Get the state file path for a cycle
    fn state_file_path(&self, cycle_id: CycleId) -> PathBuf {
        self.cycle_dir(cycle_id).join("state.yaml")
    }

    /// Get the outputs directory for a cycle
    fn outputs_dir(&self, cycle_id: CycleId) -> PathBuf {
        self.cycle_dir(cycle_id).join("outputs")
    }

    /// Get the output file path for a specific component
    fn output_file_path(&self, cycle_id: CycleId, component: ComponentType) -> PathBuf {
        self.outputs_dir(cycle_id)
            .join(format!("{:?}.yaml", component))
    }

    /// Ensure directory exists
    async fn ensure_dir(&self, path: &Path) -> Result<(), StateStorageError> {
        fs::create_dir_all(path)
            .await
            .map_err(|e| StateStorageError::IoError(e.to_string()))
    }
}

#[async_trait]
impl StateStorage for FileStateStorage {
    async fn save_state(
        &self,
        cycle_id: CycleId,
        state: &ConversationState,
    ) -> Result<(), StateStorageError> {
        let dir = self.cycle_dir(cycle_id);
        self.ensure_dir(&dir).await?;

        let file_path = self.state_file_path(cycle_id);

        // Serialize to YAML
        let yaml = serde_yaml::to_string(state)
            .map_err(|e| StateStorageError::SerializationFailed(e.to_string()))?;

        // Write to file
        fs::write(&file_path, yaml)
            .await
            .map_err(|e| StateStorageError::IoError(e.to_string()))?;

        Ok(())
    }

    async fn load_state(&self, cycle_id: CycleId) -> Result<ConversationState, StateStorageError> {
        let file_path = self.state_file_path(cycle_id);

        // Check if file exists
        if !file_path.exists() {
            return Err(StateStorageError::NotFound(cycle_id));
        }

        // Read file
        let yaml = fs::read_to_string(&file_path)
            .await
            .map_err(|e| StateStorageError::IoError(e.to_string()))?;

        // Deserialize from YAML
        let state = serde_yaml::from_str(&yaml)
            .map_err(|e| StateStorageError::DeserializationFailed(e.to_string()))?;

        Ok(state)
    }

    async fn save_step_output(
        &self,
        cycle_id: CycleId,
        component: ComponentType,
        output: &dyn StructuredOutput,
    ) -> Result<(), StateStorageError> {
        let dir = self.outputs_dir(cycle_id);
        self.ensure_dir(&dir).await?;

        let file_path = self.output_file_path(cycle_id, component);

        // Convert to YAML
        let yaml = output
            .to_yaml()
            .map_err(|e| StateStorageError::SerializationFailed(e.to_string()))?;

        // Write to file
        fs::write(&file_path, yaml)
            .await
            .map_err(|e| StateStorageError::IoError(e.to_string()))?;

        Ok(())
    }

    async fn load_step_output(
        &self,
        cycle_id: CycleId,
        component: ComponentType,
    ) -> Result<String, StateStorageError> {
        let file_path = self.output_file_path(cycle_id, component);

        // Check if file exists
        if !file_path.exists() {
            return Err(StateStorageError::OutputNotFound {
                cycle_id,
                component,
            });
        }

        // Read file
        let yaml = fs::read_to_string(&file_path)
            .await
            .map_err(|e| StateStorageError::IoError(e.to_string()))?;

        Ok(yaml)
    }

    async fn exists(&self, cycle_id: CycleId) -> Result<bool, StateStorageError> {
        let file_path = self.state_file_path(cycle_id);
        Ok(file_path.exists())
    }

    async fn delete(&self, cycle_id: CycleId) -> Result<(), StateStorageError> {
        let dir = self.cycle_dir(cycle_id);

        if dir.exists() {
            fs::remove_dir_all(&dir)
                .await
                .map_err(|e| StateStorageError::IoError(e.to_string()))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ai_engine::conversation_state::{ConversationState, MessageRole};
    use crate::domain::foundation::SessionId;
    use tempfile::TempDir;

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
    async fn test_file_storage_save_and_load_state() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStateStorage::new(temp_dir.path());

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
    async fn test_file_storage_load_nonexistent_state() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStateStorage::new(temp_dir.path());

        let cycle_id = test_cycle_id();

        let result = storage.load_state(cycle_id).await;

        assert!(matches!(result, Err(StateStorageError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_file_storage_save_and_load_output() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStateStorage::new(temp_dir.path());

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
    async fn test_file_storage_load_nonexistent_output() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStateStorage::new(temp_dir.path());

        let cycle_id = test_cycle_id();

        let result = storage
            .load_step_output(cycle_id, ComponentType::Objectives)
            .await;

        assert!(matches!(result, Err(StateStorageError::OutputNotFound { .. })));
    }

    #[tokio::test]
    async fn test_file_storage_exists() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStateStorage::new(temp_dir.path());

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
    async fn test_file_storage_delete() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStateStorage::new(temp_dir.path());

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

        // Delete
        storage.delete(cycle_id).await.unwrap();

        // Should not exist anymore
        assert!(!storage.exists(cycle_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_file_storage_multiple_cycles() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStateStorage::new(temp_dir.path());

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
    async fn test_file_storage_update_state() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStateStorage::new(temp_dir.path());

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
    async fn test_file_storage_cycle_dir_structure() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStateStorage::new(temp_dir.path());

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

        // Verify directory structure
        let cycle_dir = storage.cycle_dir(cycle_id);
        assert!(cycle_dir.exists());
        assert!(storage.state_file_path(cycle_id).exists());
        assert!(storage.outputs_dir(cycle_id).exists());
        assert!(storage
            .output_file_path(cycle_id, ComponentType::Objectives)
            .exists());
    }
}
