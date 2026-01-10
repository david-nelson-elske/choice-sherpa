//! Cycle repository port (write side).
//!
//! Defines the contract for persisting and retrieving Cycle aggregates.
//! Implementations handle the actual database operations.
//!
//! # Design
//!
//! - **Write-focused**: Optimized for aggregate persistence
//! - **Event publishing**: Implementations should publish domain events
//! - **Session-scoped**: Cycles belong to sessions

use crate::domain::cycle::Cycle;
use crate::domain::foundation::{CycleId, DomainError, SessionId};
use async_trait::async_trait;

/// Repository port for Cycle aggregate persistence.
///
/// Handles write operations for cycle lifecycle management.
/// Implementations must ensure:
/// - Domain event publication on state changes
/// - Proper indexing for session-based queries
#[async_trait]
pub trait CycleRepository: Send + Sync {
    /// Save a new cycle.
    ///
    /// # Errors
    ///
    /// - `DatabaseError` on persistence failure
    async fn save(&self, cycle: &Cycle) -> Result<(), DomainError>;

    /// Update an existing cycle.
    ///
    /// # Errors
    ///
    /// - `CycleNotFound` if cycle doesn't exist
    /// - `DatabaseError` on persistence failure
    async fn update(&self, cycle: &Cycle) -> Result<(), DomainError>;

    /// Find a cycle by its ID.
    ///
    /// Returns `None` if not found.
    async fn find_by_id(&self, id: &CycleId) -> Result<Option<Cycle>, DomainError>;

    /// Check if a cycle exists.
    async fn exists(&self, id: &CycleId) -> Result<bool, DomainError>;

    /// Find all cycles belonging to a session.
    ///
    /// Returns cycles ordered by created_at descending.
    async fn find_by_session_id(&self, session_id: &SessionId) -> Result<Vec<Cycle>, DomainError>;

    /// Find the primary (non-branched) cycle for a session.
    ///
    /// Returns the root cycle that has no parent.
    async fn find_primary_by_session_id(
        &self,
        session_id: &SessionId,
    ) -> Result<Option<Cycle>, DomainError>;

    /// Find all branches of a given parent cycle.
    ///
    /// Returns cycles where parent_cycle_id matches the given ID.
    async fn find_branches(&self, parent_id: &CycleId) -> Result<Vec<Cycle>, DomainError>;

    /// Count cycles for a session.
    ///
    /// Includes both primary and branch cycles.
    async fn count_by_session_id(&self, session_id: &SessionId) -> Result<u32, DomainError>;

    /// Delete a cycle (primarily for testing).
    ///
    /// In production, cycles should be archived rather than deleted.
    ///
    /// # Errors
    ///
    /// - `CycleNotFound` if cycle doesn't exist
    /// - `DatabaseError` on persistence failure
    async fn delete(&self, id: &CycleId) -> Result<(), DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trait object safety test
    #[test]
    fn cycle_repository_is_object_safe() {
        fn _accepts_dyn(_repo: &dyn CycleRepository) {}
    }
}
