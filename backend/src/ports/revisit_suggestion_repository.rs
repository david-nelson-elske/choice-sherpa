//! Revisit Suggestion Repository Port - Persistence for component revisit suggestions.
//!
//! This port abstracts storage of revisit suggestions created by the AI agent
//! when it identifies that an earlier component may need refinement.
//!
//! # Example
//!
//! ```ignore
//! use async_trait::async_trait;
//! use choice_sherpa::ports::RevisitSuggestionRepository;
//!
//! struct PostgresRevisitSuggestionRepository { /* ... */ }
//!
//! #[async_trait]
//! impl RevisitSuggestionRepository for PostgresRevisitSuggestionRepository {
//!     async fn save(&self, suggestion: RevisitSuggestion) -> Result<(), RevisitSuggestionRepoError> {
//!         // Insert into revisit_suggestions table
//!     }
//!     // ... other methods
//! }
//! ```

use async_trait::async_trait;
use thiserror::Error;

use crate::domain::foundation::{ComponentType, CycleId, RevisitSuggestionId};
use crate::domain::conversation::tools::{RevisitPriority, RevisitSuggestion};

/// Port for revisit suggestion persistence.
///
/// Stores suggestions from the AI agent about revisiting earlier components.
/// These suggestions queue up rather than immediately navigating, respecting
/// linear PrOACT flow.
#[async_trait]
pub trait RevisitSuggestionRepository: Send + Sync {
    /// Save a new revisit suggestion.
    async fn save(&self, suggestion: RevisitSuggestion) -> Result<(), RevisitSuggestionRepoError>;

    /// Update an existing suggestion (e.g., when accepted/dismissed).
    async fn update(&self, suggestion: &RevisitSuggestion) -> Result<(), RevisitSuggestionRepoError>;

    /// Find a suggestion by ID.
    async fn find_by_id(
        &self,
        id: RevisitSuggestionId,
    ) -> Result<Option<RevisitSuggestion>, RevisitSuggestionRepoError>;

    /// Find all pending suggestions for a cycle.
    ///
    /// Returns suggestions ordered by priority (highest first), then by created_at.
    async fn find_pending(
        &self,
        cycle_id: CycleId,
    ) -> Result<Vec<RevisitSuggestion>, RevisitSuggestionRepoError>;

    /// Find pending suggestions for a specific target component.
    async fn find_pending_for_component(
        &self,
        cycle_id: CycleId,
        component: ComponentType,
    ) -> Result<Vec<RevisitSuggestion>, RevisitSuggestionRepoError>;

    /// Find all suggestions for a cycle (any status).
    async fn find_by_cycle(
        &self,
        cycle_id: CycleId,
    ) -> Result<Vec<RevisitSuggestion>, RevisitSuggestionRepoError>;

    /// Count pending suggestions by priority.
    async fn count_pending_by_priority(
        &self,
        cycle_id: CycleId,
    ) -> Result<RevisitSuggestionCounts, RevisitSuggestionRepoError>;

    /// Expire all pending suggestions for a cycle.
    ///
    /// Called when a decision is finalized without addressing remaining suggestions.
    async fn expire_all_pending(
        &self,
        cycle_id: CycleId,
    ) -> Result<usize, RevisitSuggestionRepoError>;
}

/// Counts of pending revisit suggestions by priority.
#[derive(Debug, Clone, Default)]
pub struct RevisitSuggestionCounts {
    /// Number of critical priority suggestions
    pub critical: usize,
    /// Number of high priority suggestions
    pub high: usize,
    /// Number of medium priority suggestions
    pub medium: usize,
    /// Number of low priority suggestions
    pub low: usize,
}

impl RevisitSuggestionCounts {
    /// Returns total pending suggestions.
    pub fn total(&self) -> usize {
        self.critical + self.high + self.medium + self.low
    }

    /// Returns true if there are any urgent (high or critical) suggestions.
    pub fn has_urgent(&self) -> bool {
        self.critical > 0 || self.high > 0
    }

    /// Increments the counter for a given priority.
    pub fn increment(&mut self, priority: RevisitPriority) {
        match priority {
            RevisitPriority::Critical => self.critical += 1,
            RevisitPriority::High => self.high += 1,
            RevisitPriority::Medium => self.medium += 1,
            RevisitPriority::Low => self.low += 1,
        }
    }
}

/// Errors from the revisit suggestion repository.
#[derive(Debug, Clone, Error)]
pub enum RevisitSuggestionRepoError {
    /// Database or storage error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Suggestion not found
    #[error("Suggestion not found: {0}")]
    NotFound(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Concurrent modification detected
    #[error("Concurrent modification: {0}")]
    ConcurrentModification(String),
}

impl RevisitSuggestionRepoError {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_total_sums_all_priorities() {
        let counts = RevisitSuggestionCounts {
            critical: 1,
            high: 2,
            medium: 3,
            low: 4,
        };

        assert_eq!(counts.total(), 10);
    }

    #[test]
    fn counts_has_urgent_when_critical() {
        let counts = RevisitSuggestionCounts {
            critical: 1,
            ..Default::default()
        };

        assert!(counts.has_urgent());
    }

    #[test]
    fn counts_has_urgent_when_high() {
        let counts = RevisitSuggestionCounts {
            high: 1,
            ..Default::default()
        };

        assert!(counts.has_urgent());
    }

    #[test]
    fn counts_not_urgent_when_only_medium_low() {
        let counts = RevisitSuggestionCounts {
            medium: 5,
            low: 10,
            ..Default::default()
        };

        assert!(!counts.has_urgent());
    }

    #[test]
    fn counts_increment_works() {
        let mut counts = RevisitSuggestionCounts::default();

        counts.increment(RevisitPriority::Critical);
        counts.increment(RevisitPriority::High);
        counts.increment(RevisitPriority::High);
        counts.increment(RevisitPriority::Medium);

        assert_eq!(counts.critical, 1);
        assert_eq!(counts.high, 2);
        assert_eq!(counts.medium, 1);
        assert_eq!(counts.low, 0);
    }

    #[tokio::test]
    async fn revisit_suggestion_repository_trait_is_send_sync() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn RevisitSuggestionRepository>();
    }
}
