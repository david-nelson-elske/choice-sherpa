//! Mock authentication adapters for testing.
//!
//! These adapters implement the `SessionValidator` and `AuthProvider` ports
//! for use in tests, avoiding the need for a real auth provider like Zitadel.
//!
//! # Example
//!
//! ```ignore
//! use choice_sherpa::adapters::auth::{MockSessionValidator, MockAuthProvider};
//! use choice_sherpa::domain::foundation::{AuthenticatedUser, UserId};
//!
//! // Create a validator that accepts specific tokens
//! let validator = MockSessionValidator::new()
//!     .with_user("valid-token", AuthenticatedUser::new(
//!         UserId::new("user-123").unwrap(),
//!         "test@example.com",
//!         Some("Test User".to_string()),
//!         true,
//!     ));
//!
//! // Use in tests
//! let result = validator.validate("valid-token").await;
//! assert!(result.is_ok());
//! ```

use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;

use crate::domain::foundation::{AuthenticatedUser, AuthError, UserId};
use crate::ports::{AuthProvider, SessionValidator};

/// Mock session validator for testing.
///
/// Stores a map of tokens to users. Tokens not in the map return `InvalidToken`.
#[derive(Debug, Default)]
pub struct MockSessionValidator {
    /// Map of valid tokens to their associated users
    tokens: RwLock<HashMap<String, AuthenticatedUser>>,
    /// Optional error to return for all validations (for error testing)
    force_error: RwLock<Option<AuthError>>,
}

impl MockSessionValidator {
    /// Creates a new empty mock validator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a valid token that maps to a user.
    ///
    /// When `validate()` is called with this token, it returns the associated user.
    pub fn with_user(self, token: impl Into<String>, user: AuthenticatedUser) -> Self {
        self.tokens.write().unwrap().insert(token.into(), user);
        self
    }

    /// Adds a valid token with a simple test user.
    ///
    /// Convenience method that creates a user with the given ID.
    pub fn with_test_user(self, token: impl Into<String>, user_id: impl Into<String>) -> Self {
        let user_id = user_id.into();
        let user = AuthenticatedUser::new(
            UserId::new(&user_id).unwrap(),
            format!("{}@test.example.com", user_id),
            Some(format!("Test User {}", user_id)),
            true,
        );
        self.with_user(token, user)
    }

    /// Forces all validations to return the specified error.
    ///
    /// Useful for testing error handling paths.
    pub fn with_error(self, error: AuthError) -> Self {
        *self.force_error.write().unwrap() = Some(error);
        self
    }

    /// Clears the forced error and returns to normal operation.
    pub fn clear_error(&self) {
        *self.force_error.write().unwrap() = None;
    }

    /// Registers a new valid token at runtime.
    pub fn add_token(&self, token: impl Into<String>, user: AuthenticatedUser) {
        self.tokens.write().unwrap().insert(token.into(), user);
    }

    /// Removes a token, making it invalid.
    pub fn remove_token(&self, token: &str) {
        self.tokens.write().unwrap().remove(token);
    }

    /// Returns the number of registered valid tokens.
    pub fn token_count(&self) -> usize {
        self.tokens.read().unwrap().len()
    }
}

#[async_trait]
impl SessionValidator for MockSessionValidator {
    async fn validate(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
        // Check for forced error
        if let Some(error) = self.force_error.read().unwrap().clone() {
            return Err(error);
        }

        // Look up the token
        self.tokens
            .read()
            .unwrap()
            .get(token)
            .cloned()
            .ok_or(AuthError::InvalidToken)
    }
}

/// Mock auth provider for testing.
///
/// Stores a map of user IDs to users. Unknown IDs return `UserNotFound`.
#[derive(Debug, Default)]
pub struct MockAuthProvider {
    /// Map of user IDs to their profiles
    users: RwLock<HashMap<String, AuthenticatedUser>>,
    /// Optional error to return for all lookups (for error testing)
    force_error: RwLock<Option<AuthError>>,
}

impl MockAuthProvider {
    /// Creates a new empty mock provider.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a user to the provider.
    pub fn with_user(self, user: AuthenticatedUser) -> Self {
        self.users
            .write()
            .unwrap()
            .insert(user.id.as_str().to_string(), user);
        self
    }

    /// Adds a simple test user.
    pub fn with_test_user(self, user_id: impl Into<String>) -> Self {
        let user_id = user_id.into();
        let user = AuthenticatedUser::new(
            UserId::new(&user_id).unwrap(),
            format!("{}@test.example.com", user_id),
            Some(format!("Test User {}", user_id)),
            true,
        );
        self.with_user(user)
    }

    /// Forces all lookups to return the specified error.
    pub fn with_error(self, error: AuthError) -> Self {
        *self.force_error.write().unwrap() = Some(error);
        self
    }

    /// Clears the forced error.
    pub fn clear_error(&self) {
        *self.force_error.write().unwrap() = None;
    }

    /// Adds a user at runtime.
    pub fn add_user(&self, user: AuthenticatedUser) {
        self.users
            .write()
            .unwrap()
            .insert(user.id.as_str().to_string(), user);
    }

    /// Removes a user.
    pub fn remove_user(&self, user_id: &UserId) {
        self.users.write().unwrap().remove(user_id.as_str());
    }

    /// Returns the number of registered users.
    pub fn user_count(&self) -> usize {
        self.users.read().unwrap().len()
    }
}

#[async_trait]
impl AuthProvider for MockAuthProvider {
    async fn get_user(&self, user_id: &UserId) -> Result<AuthenticatedUser, AuthError> {
        // Check for forced error
        if let Some(error) = self.force_error.read().unwrap().clone() {
            return Err(error);
        }

        // Look up the user
        self.users
            .read()
            .unwrap()
            .get(user_id.as_str())
            .cloned()
            .ok_or(AuthError::UserNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_user() -> AuthenticatedUser {
        AuthenticatedUser::new(
            UserId::new("user-123").unwrap(),
            "test@example.com",
            Some("Test User".to_string()),
            true,
        )
    }

    // ════════════════════════════════════════════════════════════════════════════
    // MockSessionValidator Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn mock_validator_returns_user_for_registered_token() {
        let validator = MockSessionValidator::new().with_user("valid-token", test_user());

        let result = validator.validate("valid-token").await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id.as_str(), "user-123");
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn mock_validator_returns_invalid_token_for_unknown() {
        let validator = MockSessionValidator::new();

        let result = validator.validate("unknown-token").await;

        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[tokio::test]
    async fn mock_validator_with_test_user_creates_user() {
        let validator = MockSessionValidator::new().with_test_user("my-token", "user-456");

        let result = validator.validate("my-token").await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id.as_str(), "user-456");
        assert!(user.email.contains("user-456"));
    }

    #[tokio::test]
    async fn mock_validator_with_error_forces_error() {
        let validator = MockSessionValidator::new()
            .with_user("valid-token", test_user())
            .with_error(AuthError::ServiceUnavailable("Test error".to_string()));

        let result = validator.validate("valid-token").await;

        assert!(matches!(result, Err(AuthError::ServiceUnavailable(_))));
    }

    #[tokio::test]
    async fn mock_validator_clear_error_restores_normal_operation() {
        let validator = MockSessionValidator::new()
            .with_user("valid-token", test_user())
            .with_error(AuthError::ServiceUnavailable("Test".to_string()));

        // First, error is forced
        assert!(validator.validate("valid-token").await.is_err());

        // Clear error
        validator.clear_error();

        // Now validation works
        assert!(validator.validate("valid-token").await.is_ok());
    }

    #[tokio::test]
    async fn mock_validator_add_token_works_at_runtime() {
        let validator = MockSessionValidator::new();

        // Initially no tokens
        assert!(validator.validate("new-token").await.is_err());

        // Add token
        validator.add_token("new-token", test_user());

        // Now it works
        assert!(validator.validate("new-token").await.is_ok());
    }

    #[tokio::test]
    async fn mock_validator_remove_token_invalidates() {
        let validator = MockSessionValidator::new().with_user("token", test_user());

        // Works initially
        assert!(validator.validate("token").await.is_ok());

        // Remove token
        validator.remove_token("token");

        // Now fails
        assert!(validator.validate("token").await.is_err());
    }

    #[test]
    fn mock_validator_token_count_tracks_tokens() {
        let validator = MockSessionValidator::new()
            .with_test_user("t1", "u1")
            .with_test_user("t2", "u2");

        assert_eq!(validator.token_count(), 2);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // MockAuthProvider Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn mock_provider_returns_user_when_exists() {
        let provider = MockAuthProvider::new().with_user(test_user());

        let user_id = UserId::new("user-123").unwrap();
        let result = provider.get_user(&user_id).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn mock_provider_returns_not_found_for_unknown() {
        let provider = MockAuthProvider::new();

        let user_id = UserId::new("unknown").unwrap();
        let result = provider.get_user(&user_id).await;

        assert!(matches!(result, Err(AuthError::UserNotFound)));
    }

    #[tokio::test]
    async fn mock_provider_with_test_user_creates_user() {
        let provider = MockAuthProvider::new().with_test_user("user-789");

        let user_id = UserId::new("user-789").unwrap();
        let result = provider.get_user(&user_id).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert!(user.email.contains("user-789"));
    }

    #[tokio::test]
    async fn mock_provider_with_error_forces_error() {
        let provider = MockAuthProvider::new()
            .with_user(test_user())
            .with_error(AuthError::ServiceUnavailable("Down".to_string()));

        let user_id = UserId::new("user-123").unwrap();
        let result = provider.get_user(&user_id).await;

        assert!(matches!(result, Err(AuthError::ServiceUnavailable(_))));
    }

    #[test]
    fn mock_provider_user_count_tracks_users() {
        let provider = MockAuthProvider::new()
            .with_test_user("u1")
            .with_test_user("u2")
            .with_test_user("u3");

        assert_eq!(provider.user_count(), 3);
    }
}
