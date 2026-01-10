//! Storage Adapters
//!
//! Implementations of the StateStorage port for persisting conversation state.
//!
//! ## Available Adapters
//!
//! - **FileStateStorage** - Stores state as YAML files on disk
//! - **InMemoryStateStorage** - Stores state in memory (testing/development)
//!
//! ## Usage
//!
//! ```ignore
//! use adapters::storage::{FileStateStorage, InMemoryStateStorage};
//!
//! // Production: file-based storage
//! let storage = FileStateStorage::new("./data/conversations");
//!
//! // Testing: in-memory storage
//! let storage = InMemoryStateStorage::new();
//! ```

mod file_state_storage;
mod in_memory_state_storage;

pub use file_state_storage::FileStateStorage;
pub use in_memory_state_storage::InMemoryStateStorage;
