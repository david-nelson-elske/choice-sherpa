//! AI Engine Port - Primary port for AI conversation management.
//!
//! This port abstracts the entire AI conversation system, allowing different
//! AI backends to be swapped without affecting application logic.

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

use crate::domain::ai_engine::ConversationState;
use crate::domain::foundation::{CycleId, SessionId};

use super::AIError;

/// Session handle for an active AI conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionHandle(u64);

impl SessionHandle {
    /// Create a new session handle
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the inner ID
    pub fn id(&self) -> u64 {
        self.0
    }
}

/// Response chunk from AI conversation
#[derive(Debug, Clone)]
pub struct ResponseChunk {
    pub content: String,
    pub is_final: bool,
}

/// Primary port for AI conversation management
#[async_trait]
pub trait AIEngine: Send + Sync {
    /// Start a new conversation session for a cycle
    ///
    /// # Arguments
    /// * `cycle_id` - The cycle to start a conversation for
    /// * `session_id` - The parent session ID
    /// * `initial_state` - Initial conversation state
    ///
    /// # Returns
    /// A session handle for the conversation
    ///
    /// # Errors
    /// Returns `AIError` if the session cannot be started
    async fn start_session(
        &self,
        cycle_id: CycleId,
        session_id: SessionId,
        initial_state: ConversationState,
    ) -> Result<SessionHandle, AIError>;

    /// Send a message and get streaming response
    ///
    /// # Arguments
    /// * `handle` - The session handle
    /// * `message` - The user's message
    ///
    /// # Returns
    /// A stream of response chunks
    ///
    /// # Errors
    /// Returns `AIError` if message cannot be sent or processed
    async fn send_message(
        &self,
        handle: SessionHandle,
        message: String,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ResponseChunk, AIError>> + Send>>, AIError>;

    /// Get the current conversation state
    ///
    /// # Arguments
    /// * `handle` - The session handle
    ///
    /// # Returns
    /// The current conversation state
    ///
    /// # Errors
    /// Returns `AIError` if state cannot be retrieved
    async fn get_state(&self, handle: SessionHandle) -> Result<ConversationState, AIError>;

    /// End a conversation session
    ///
    /// # Arguments
    /// * `handle` - The session handle to end
    ///
    /// # Errors
    /// Returns `AIError` if session cannot be ended cleanly
    async fn end_session(&self, handle: SessionHandle) -> Result<(), AIError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_handle_new() {
        let handle = SessionHandle::new(42);
        assert_eq!(handle.id(), 42);
    }

    #[test]
    fn test_session_handle_equality() {
        let handle1 = SessionHandle::new(100);
        let handle2 = SessionHandle::new(100);
        let handle3 = SessionHandle::new(200);

        assert_eq!(handle1, handle2);
        assert_ne!(handle1, handle3);
    }

    #[test]
    fn test_response_chunk_is_final() {
        let chunk = ResponseChunk {
            content: "Hello".to_string(),
            is_final: false,
        };

        assert!(!chunk.is_final);
    }
}
