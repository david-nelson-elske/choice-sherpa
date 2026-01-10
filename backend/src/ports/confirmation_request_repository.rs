//! Confirmation Request Repository Port - Persistence for user confirmation requests.
//!
//! This port abstracts storage of confirmation requests created by the AI agent
//! when it needs explicit user input before proceeding.
//!
//! # Example
//!
//! ```ignore
//! use async_trait::async_trait;
//! use choice_sherpa::ports::ConfirmationRequestRepository;
//!
//! struct PostgresConfirmationRequestRepository { /* ... */ }
//!
//! #[async_trait]
//! impl ConfirmationRequestRepository for PostgresConfirmationRequestRepository {
//!     async fn save(&self, request: ConfirmationRequest) -> Result<(), ConfirmationRequestRepoError> {
//!         // Insert into confirmation_requests table
//!     }
//!     // ... other methods
//! }
//! ```

use async_trait::async_trait;
use thiserror::Error;

use crate::domain::foundation::{ConfirmationRequestId, CycleId};
use crate::domain::conversation::tools::{ConfirmationRequest, ConfirmationStatus};

/// Port for confirmation request persistence.
///
/// Stores confirmation requests from the AI agent that pause conversation
/// until the user responds.
#[async_trait]
pub trait ConfirmationRequestRepository: Send + Sync {
    /// Save a new confirmation request.
    async fn save(&self, request: ConfirmationRequest) -> Result<(), ConfirmationRequestRepoError>;

    /// Update an existing request (e.g., when user responds).
    async fn update(&self, request: &ConfirmationRequest) -> Result<(), ConfirmationRequestRepoError>;

    /// Find a request by ID.
    async fn find_by_id(
        &self,
        id: ConfirmationRequestId,
    ) -> Result<Option<ConfirmationRequest>, ConfirmationRequestRepoError>;

    /// Find the current pending request for a cycle (at most one).
    ///
    /// Returns the most recent pending request, or None if no pending request exists.
    async fn find_pending(
        &self,
        cycle_id: CycleId,
    ) -> Result<Option<ConfirmationRequest>, ConfirmationRequestRepoError>;

    /// Find all requests for a cycle (any status).
    async fn find_by_cycle(
        &self,
        cycle_id: CycleId,
    ) -> Result<Vec<ConfirmationRequest>, ConfirmationRequestRepoError>;

    /// Find requests that have expired but are still pending.
    ///
    /// Used by a background job to expire stale requests.
    async fn find_expired_pending(&self) -> Result<Vec<ConfirmationRequest>, ConfirmationRequestRepoError>;

    /// Expire a specific request.
    async fn expire(&self, id: ConfirmationRequestId) -> Result<(), ConfirmationRequestRepoError>;

    /// Count requests by status for a cycle.
    async fn count_by_status(
        &self,
        cycle_id: CycleId,
    ) -> Result<ConfirmationRequestCounts, ConfirmationRequestRepoError>;
}

/// Counts of confirmation requests by status.
#[derive(Debug, Clone, Default)]
pub struct ConfirmationRequestCounts {
    /// Number of pending requests
    pub pending: usize,
    /// Number of confirmed requests
    pub confirmed: usize,
    /// Number of rejected requests
    pub rejected: usize,
    /// Number of expired requests
    pub expired: usize,
}

impl ConfirmationRequestCounts {
    /// Returns total requests.
    pub fn total(&self) -> usize {
        self.pending + self.confirmed + self.rejected + self.expired
    }

    /// Returns true if there is a pending request.
    pub fn has_pending(&self) -> bool {
        self.pending > 0
    }

    /// Returns the confirmation rate (confirmed / (confirmed + rejected)).
    ///
    /// Returns 0.0 if no responses have been given.
    pub fn confirmation_rate(&self) -> f64 {
        let responded = self.confirmed + self.rejected;
        if responded == 0 {
            0.0
        } else {
            self.confirmed as f64 / responded as f64
        }
    }

    /// Increments the counter for a given status.
    pub fn increment(&mut self, status: ConfirmationStatus) {
        match status {
            ConfirmationStatus::Pending => self.pending += 1,
            ConfirmationStatus::Confirmed => self.confirmed += 1,
            ConfirmationStatus::Rejected => self.rejected += 1,
            ConfirmationStatus::Expired => self.expired += 1,
        }
    }
}

/// Errors from the confirmation request repository.
#[derive(Debug, Clone, Error)]
pub enum ConfirmationRequestRepoError {
    /// Database or storage error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Request not found
    #[error("Confirmation request not found: {0}")]
    NotFound(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Request already resolved
    #[error("Request already resolved: {0}")]
    AlreadyResolved(String),

    /// Concurrent modification detected
    #[error("Concurrent modification: {0}")]
    ConcurrentModification(String),
}

impl ConfirmationRequestRepoError {
    /// Creates a storage error.
    pub fn storage(message: impl Into<String>) -> Self {
        Self::StorageError(message.into())
    }

    /// Creates a not found error.
    pub fn not_found(id: impl std::fmt::Display) -> Self {
        Self::NotFound(id.to_string())
    }

    /// Creates a serialization error.
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::SerializationError(message.into())
    }

    /// Creates an already resolved error.
    pub fn already_resolved(id: impl std::fmt::Display) -> Self {
        Self::AlreadyResolved(id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_total_sums_all_statuses() {
        let counts = ConfirmationRequestCounts {
            pending: 1,
            confirmed: 5,
            rejected: 2,
            expired: 1,
        };

        assert_eq!(counts.total(), 9);
    }

    #[test]
    fn counts_has_pending() {
        let with_pending = ConfirmationRequestCounts {
            pending: 1,
            ..Default::default()
        };
        let without_pending = ConfirmationRequestCounts {
            confirmed: 5,
            ..Default::default()
        };

        assert!(with_pending.has_pending());
        assert!(!without_pending.has_pending());
    }

    #[test]
    fn counts_confirmation_rate_all_confirmed() {
        let counts = ConfirmationRequestCounts {
            confirmed: 10,
            ..Default::default()
        };

        assert!((counts.confirmation_rate() - 1.0).abs() < 0.01);
    }

    #[test]
    fn counts_confirmation_rate_half_rejected() {
        let counts = ConfirmationRequestCounts {
            confirmed: 5,
            rejected: 5,
            ..Default::default()
        };

        assert!((counts.confirmation_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn counts_confirmation_rate_no_responses() {
        let counts = ConfirmationRequestCounts {
            pending: 3,
            expired: 2,
            ..Default::default()
        };

        assert!((counts.confirmation_rate() - 0.0).abs() < 0.01);
    }

    #[test]
    fn counts_increment_works() {
        let mut counts = ConfirmationRequestCounts::default();

        counts.increment(ConfirmationStatus::Pending);
        counts.increment(ConfirmationStatus::Confirmed);
        counts.increment(ConfirmationStatus::Confirmed);
        counts.increment(ConfirmationStatus::Rejected);

        assert_eq!(counts.pending, 1);
        assert_eq!(counts.confirmed, 2);
        assert_eq!(counts.rejected, 1);
        assert_eq!(counts.expired, 0);
    }

    #[tokio::test]
    async fn confirmation_request_repository_trait_is_send_sync() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn ConfirmationRequestRepository>();
    }
}
