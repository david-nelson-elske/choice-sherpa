//! Membership repository port (write side).
//!
//! Defines the contract for persisting and retrieving Membership aggregates.
//! Implementations handle the actual database operations.
//!
//! # Design
//!
//! - **Write-focused**: Optimized for aggregate persistence
//! - **Event publishing**: Implementations should publish domain events
//! - **Unique constraint**: Only one membership per user
//!
//! # Example
//!
//! ```ignore
//! async fn create_free_membership(
//!     repo: &dyn MembershipRepository,
//!     user_id: &UserId,
//!     promo_code: &str,
//! ) -> Result<Membership, DomainError> {
//!     // Check if user already has membership
//!     if repo.find_by_user_id(user_id).await?.is_some() {
//!         return Err(DomainError::validation("user_id", "User already has membership"));
//!     }
//!
//!     let membership = Membership::create_free(
//!         MembershipId::new(),
//!         user_id.clone(),
//!         MembershipTier::Annual,
//!         promo_code.to_string(),
//!         Timestamp::now(),
//!         Timestamp::now().add_days(365),
//!     );
//!
//!     repo.save(&membership).await?;
//!     Ok(membership)
//! }
//! ```

use crate::domain::foundation::{DomainError, MembershipId, UserId};
use crate::domain::membership::Membership;
use async_trait::async_trait;

/// Repository port for Membership aggregate persistence.
///
/// Handles write operations for membership lifecycle management.
/// Implementations must ensure:
/// - Unique user_id constraint
/// - Domain event publication on state changes
/// - Optimistic locking for concurrent updates
#[async_trait]
pub trait MembershipRepository: Send + Sync {
    /// Save a new membership.
    ///
    /// # Errors
    ///
    /// - `ValidationFailed` if user already has a membership
    /// - `DatabaseError` on persistence failure
    async fn save(&self, membership: &Membership) -> Result<(), DomainError>;

    /// Update an existing membership.
    ///
    /// # Errors
    ///
    /// - `MembershipNotFound` if membership doesn't exist
    /// - `DatabaseError` on persistence failure
    async fn update(&self, membership: &Membership) -> Result<(), DomainError>;

    /// Find a membership by its ID.
    ///
    /// Returns `None` if not found.
    async fn find_by_id(&self, id: &MembershipId) -> Result<Option<Membership>, DomainError>;

    /// Find a membership by user ID.
    ///
    /// Returns `None` if user has no membership.
    /// This is the primary lookup method since each user has at most one membership.
    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Option<Membership>, DomainError>;

    /// Find memberships expiring within the specified number of days.
    ///
    /// Used for renewal reminders and grace period tracking.
    async fn find_expiring_within_days(&self, days: u32) -> Result<Vec<Membership>, DomainError>;

    /// Delete a membership (primarily for testing).
    ///
    /// In production, memberships should transition to Expired rather than being deleted.
    ///
    /// # Errors
    ///
    /// - `MembershipNotFound` if membership doesn't exist
    /// - `DatabaseError` on persistence failure
    async fn delete(&self, id: &MembershipId) -> Result<(), DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trait object safety test
    #[test]
    fn membership_repository_is_object_safe() {
        fn _accepts_dyn(_repo: &dyn MembershipRepository) {}
    }
}
