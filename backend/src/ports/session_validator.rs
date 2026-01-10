//! Session validation port for JWT token validation.
//!
//! This port defines the contract for validating access tokens and extracting
//! user identity. It is provider-agnostic - implementations exist for Zitadel,
//! mock testing, and could be added for Auth0, Keycloak, etc.
//!
//! # Security Requirements (per APPLICATION-SECURITY-STANDARD.md A07)
//!
//! All implementations MUST validate:
//! - **Issuer (iss)**: Token must come from expected auth provider
//! - **Audience (aud)**: Token must be intended for this application
//! - **Expiry (exp)**: Token must not be expired
//!
//! # Example Implementation
//!
//! ```ignore
//! pub struct ZitadelValidator { ... }
//!
//! #[async_trait]
//! impl SessionValidator for ZitadelValidator {
//!     async fn validate(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
//!         // 1. Introspect token with Zitadel
//!         // 2. Validate iss, aud, exp claims
//!         // 3. Map claims to AuthenticatedUser
//!     }
//! }
//! ```

use async_trait::async_trait;

use crate::domain::foundation::{AuthenticatedUser, AuthError};

/// Validates access tokens and extracts user identity.
///
/// This is the primary port for authentication. HTTP middleware uses this
/// to validate Bearer tokens and extract the authenticated user.
///
/// # Contract
///
/// Implementations must:
/// - Validate the token signature
/// - Validate issuer, audience, and expiry claims
/// - Return `AuthError::InvalidToken` for malformed/bad signature tokens
/// - Return `AuthError::TokenExpired` for expired tokens
/// - Return `AuthError::ServiceUnavailable` for transient errors
#[async_trait]
pub trait SessionValidator: Send + Sync {
    /// Validate a JWT access token and return the authenticated user.
    ///
    /// # Arguments
    ///
    /// * `token` - The raw JWT token (without "Bearer " prefix)
    ///
    /// # Returns
    ///
    /// * `Ok(AuthenticatedUser)` - Token is valid, user extracted from claims
    /// * `Err(AuthError::InvalidToken)` - Token is malformed or signature invalid
    /// * `Err(AuthError::TokenExpired)` - Token signature valid but expired
    /// * `Err(AuthError::ServiceUnavailable)` - Auth provider unreachable
    async fn validate(&self, token: &str) -> Result<AuthenticatedUser, AuthError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::UserId;
    use std::collections::HashMap;
    use std::sync::RwLock;

    /// Simple mock implementation for testing the trait
    struct TestSessionValidator {
        tokens: RwLock<HashMap<String, AuthenticatedUser>>,
    }

    impl TestSessionValidator {
        fn new() -> Self {
            Self {
                tokens: RwLock::new(HashMap::new()),
            }
        }

        fn add_valid_token(&self, token: &str, user: AuthenticatedUser) {
            self.tokens.write().unwrap().insert(token.to_string(), user);
        }
    }

    #[async_trait]
    impl SessionValidator for TestSessionValidator {
        async fn validate(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
            self.tokens
                .read()
                .unwrap()
                .get(token)
                .cloned()
                .ok_or(AuthError::InvalidToken)
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
    async fn session_validator_returns_user_for_valid_token() {
        let validator = TestSessionValidator::new();
        validator.add_valid_token("valid-token-123", test_user());

        let result = validator.validate("valid-token-123").await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id.as_str(), "user-123");
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn session_validator_returns_error_for_invalid_token() {
        let validator = TestSessionValidator::new();

        let result = validator.validate("invalid-token").await;

        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[tokio::test]
    async fn session_validator_trait_is_send_sync() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn SessionValidator>();
    }
}
