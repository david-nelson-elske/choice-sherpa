//! Ownership trait for user-owned resources.
//!
//! This module provides the `OwnedByUser` trait that standardizes
//! ownership checking across all user-owned aggregates.
//!
//! # DRY Pattern
//!
//! Instead of each aggregate implementing its own `is_owner()` method
//! with ad-hoc error handling, they implement this trait which provides:
//! - Consistent method names across all aggregates
//! - A `check_ownership()` method that returns proper domain errors
//! - Clear semantics for audit logging
//!
//! # Example
//!
//! ```ignore
//! impl OwnedByUser for Session {
//!     fn owner_id(&self) -> &UserId {
//!         &self.user_id
//!     }
//! }
//!
//! // In a handler:
//! session.check_ownership(&user_id)?;  // Returns Err(Forbidden) if not owner
//! ```

use super::{DomainError, ErrorCode, UserId};

/// Trait for aggregates that have a single owner.
///
/// Implementors should return the `UserId` of the owning user.
/// The trait provides default implementations for ownership checking.
///
/// # Security Note
///
/// This trait is designed for single-owner resources. For shared resources
/// (e.g., team-owned), use a different authorization mechanism.
pub trait OwnedByUser {
    /// Returns the ID of the user who owns this resource.
    fn owner_id(&self) -> &UserId;

    /// Checks if the given user is the owner.
    ///
    /// Returns `true` if `user_id` matches `owner_id()`.
    fn is_owner(&self, user_id: &UserId) -> bool {
        self.owner_id() == user_id
    }

    /// Validates ownership, returning an error if the user is not the owner.
    ///
    /// This is the preferred method to use in command handlers as it
    /// returns a properly formed `DomainError` with `Forbidden` code.
    ///
    /// # Example
    ///
    /// ```ignore
    /// pub async fn handle(&self, cmd: UpdateSessionCommand, metadata: CommandMetadata)
    ///     -> Result<(), DomainError>
    /// {
    ///     let session = self.repo.find_by_id(cmd.session_id).await?
    ///         .ok_or_else(|| DomainError::new(ErrorCode::SessionNotFound, "Session not found"))?;
    ///
    ///     // Check ownership - returns Err(Forbidden) if not owner
    ///     session.check_ownership(&metadata.user_id)?;
    ///
    ///     // ... proceed with update
    /// }
    /// ```
    fn check_ownership(&self, user_id: &UserId) -> Result<(), DomainError> {
        if self.is_owner(user_id) {
            Ok(())
        } else {
            Err(DomainError::new(
                ErrorCode::Forbidden,
                "User does not own this resource",
            )
            .with_detail("owner_id", self.owner_id().to_string())
            .with_detail("requested_by", user_id.to_string()))
        }
    }
}

/// Trait for aggregates that belong to a parent resource.
///
/// Some resources don't have direct user ownership but belong to
/// an owned parent (e.g., Cycle belongs to Session, Component belongs to Cycle).
/// This trait enables ownership checks through the parent chain.
///
/// # Example
///
/// ```ignore
/// impl BelongsToOwned for Cycle {
///     type Parent = Session;
///
///     fn parent_id(&self) -> SessionId {
///         self.session_id
///     }
/// }
/// ```
pub trait BelongsToOwned {
    /// The type of the parent aggregate.
    type Parent;

    /// Type alias for the parent's ID type.
    /// This is typically SessionId, CycleId, etc.
    type ParentId: Clone + Send + Sync;

    /// Returns the ID of the parent resource.
    fn parent_id(&self) -> Self::ParentId;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test struct that implements OwnedByUser
    struct TestResource {
        owner: UserId,
        #[allow(dead_code)]
        name: String,
    }

    impl OwnedByUser for TestResource {
        fn owner_id(&self) -> &UserId {
            &self.owner
        }
    }

    fn test_user(id: &str) -> UserId {
        UserId::new(id).unwrap()
    }

    #[test]
    fn is_owner_returns_true_for_owner() {
        let owner = test_user("owner-123");
        let resource = TestResource {
            owner: owner.clone(),
            name: "Test".to_string(),
        };

        assert!(resource.is_owner(&owner));
    }

    #[test]
    fn is_owner_returns_false_for_non_owner() {
        let owner = test_user("owner-123");
        let other = test_user("other-456");
        let resource = TestResource {
            owner,
            name: "Test".to_string(),
        };

        assert!(!resource.is_owner(&other));
    }

    #[test]
    fn check_ownership_succeeds_for_owner() {
        let owner = test_user("owner-123");
        let resource = TestResource {
            owner: owner.clone(),
            name: "Test".to_string(),
        };

        let result = resource.check_ownership(&owner);
        assert!(result.is_ok());
    }

    #[test]
    fn check_ownership_fails_for_non_owner() {
        let owner = test_user("owner-123");
        let other = test_user("other-456");
        let resource = TestResource {
            owner,
            name: "Test".to_string(),
        };

        let result = resource.check_ownership(&other);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::Forbidden);
        assert!(err.message.contains("does not own"));
    }

    #[test]
    fn check_ownership_error_includes_details() {
        let owner = test_user("owner-123");
        let other = test_user("other-456");
        let resource = TestResource {
            owner,
            name: "Test".to_string(),
        };

        let err = resource.check_ownership(&other).unwrap_err();

        assert_eq!(err.details.get("owner_id"), Some(&"owner-123".to_string()));
        assert_eq!(
            err.details.get("requested_by"),
            Some(&"other-456".to_string())
        );
    }

    #[test]
    fn owner_id_returns_correct_id() {
        let owner = test_user("owner-789");
        let resource = TestResource {
            owner: owner.clone(),
            name: "Test".to_string(),
        };

        assert_eq!(resource.owner_id(), &owner);
    }
}
