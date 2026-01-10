//! Auth provider port for user profile retrieval.
//!
//! This is a secondary authentication port for cases where you need to
//! fetch user information outside of a request context (e.g., batch jobs,
//! admin operations, etc.).
//!
//! # When to Use
//!
//! - **SessionValidator**: Primary port - use for validating incoming requests
//! - **AuthProvider**: Secondary port - use for looking up user profiles by ID
//!
//! Most request handlers should use `SessionValidator` + the extracted
//! `AuthenticatedUser`. Use `AuthProvider` when you need to look up a
//! user that isn't the current requester.
//!
//! # Example
//!
//! ```ignore
//! // In an admin handler that needs to look up another user
//! async fn get_user_details(
//!     auth_provider: Arc<dyn AuthProvider>,
//!     user_id: &UserId,
//! ) -> Result<UserDetails, AuthError> {
//!     let user = auth_provider.get_user(user_id).await?;
//!     Ok(UserDetails::from(user))
//! }
//! ```

use async_trait::async_trait;

use crate::domain::foundation::{AuthenticatedUser, AuthError, UserId};

/// Retrieves user profile information from the auth provider.
///
/// This port is for looking up users by ID, separate from token validation.
/// Use this when you need to fetch profile info for a user other than the
/// current requester.
///
/// # Contract
///
/// Implementations must:
/// - Return the user if they exist in the auth system
/// - Return `AuthError::UserNotFound` if the user doesn't exist
/// - Return `AuthError::ServiceUnavailable` for transient errors
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Get a user by their ID.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The unique user identifier
    ///
    /// # Returns
    ///
    /// * `Ok(AuthenticatedUser)` - User found and profile retrieved
    /// * `Err(AuthError::UserNotFound)` - No user exists with this ID
    /// * `Err(AuthError::ServiceUnavailable)` - Auth provider unreachable
    async fn get_user(&self, user_id: &UserId) -> Result<AuthenticatedUser, AuthError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::RwLock;

    /// Simple mock implementation for testing the trait
    struct TestAuthProvider {
        users: RwLock<HashMap<String, AuthenticatedUser>>,
    }

    impl TestAuthProvider {
        fn new() -> Self {
            Self {
                users: RwLock::new(HashMap::new()),
            }
        }

        fn add_user(&self, user: AuthenticatedUser) {
            self.users
                .write()
                .unwrap()
                .insert(user.id.as_str().to_string(), user);
        }
    }

    #[async_trait]
    impl AuthProvider for TestAuthProvider {
        async fn get_user(&self, user_id: &UserId) -> Result<AuthenticatedUser, AuthError> {
            self.users
                .read()
                .unwrap()
                .get(user_id.as_str())
                .cloned()
                .ok_or(AuthError::UserNotFound)
        }
    }

    fn test_user() -> AuthenticatedUser {
        AuthenticatedUser::new(
            UserId::new("user-123").unwrap(),
            "test@example.com",
            Some("Test User".to_string()),
            true,
        )
    }

    #[tokio::test]
    async fn auth_provider_returns_user_when_exists() {
        let provider = TestAuthProvider::new();
        provider.add_user(test_user());

        let user_id = UserId::new("user-123").unwrap();
        let result = provider.get_user(&user_id).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn auth_provider_returns_not_found_for_missing_user() {
        let provider = TestAuthProvider::new();

        let user_id = UserId::new("nonexistent").unwrap();
        let result = provider.get_user(&user_id).await;

        assert!(matches!(result, Err(AuthError::UserNotFound)));
    }

    #[test]
    fn auth_provider_trait_is_object_safe_and_send_sync() {
        fn _assert_trait_object(_: &dyn AuthProvider) {}
        fn _assert_arc_send_sync<T: Send + Sync + ?Sized>() {}
        _assert_arc_send_sync::<std::sync::Arc<dyn AuthProvider>>();
    }
}
