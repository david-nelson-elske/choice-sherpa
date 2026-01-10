//! Authentication types for the domain layer.
//!
//! These types represent an authenticated user extracted from a JWT token.
//! They have **no external dependencies** - any auth provider (Zitadel, Auth0,
//! Keycloak) can populate them via the `SessionValidator` port.
//!
//! # Design Decisions
//!
//! - `AuthenticatedUser` contains only the claims we actually use
//! - `AuthError` is domain-centric, not provider-specific
//! - Types are `Clone` for easy use in request handlers
//!
//! # Example
//!
//! ```ignore
//! // In HTTP middleware, after JWT validation:
//! let user = AuthenticatedUser {
//!     id: UserId::new("user-123")?,
//!     email: "user@example.com".to_string(),
//!     display_name: Some("Alice".to_string()),
//!     email_verified: true,
//! };
//!
//! // Inject into request extensions for handlers to use
//! request.extensions_mut().insert(user);
//! ```

use super::UserId;
use thiserror::Error;

/// Authenticated user extracted from a validated JWT.
///
/// This is a **domain type** with no provider dependencies.
/// Any OIDC provider can populate this struct via the `SessionValidator` port.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    /// The unique user identifier from the auth provider.
    pub id: UserId,

    /// User's email address from the token claims.
    pub email: String,

    /// Display name if available (may come from `name` or `preferred_username` claim).
    pub display_name: Option<String>,

    /// Whether the user's email has been verified by the auth provider.
    pub email_verified: bool,
}

impl AuthenticatedUser {
    /// Creates a new authenticated user.
    ///
    /// This is typically called by the `SessionValidator` adapter after
    /// successfully validating a JWT token.
    pub fn new(
        id: UserId,
        email: impl Into<String>,
        display_name: Option<String>,
        email_verified: bool,
    ) -> Self {
        Self {
            id,
            email: email.into(),
            display_name,
            email_verified,
        }
    }

    /// Returns the user's display name, or email as fallback.
    pub fn display_name_or_email(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.email)
    }
}

/// Authentication errors that can occur during token validation.
///
/// These errors are **domain-centric** - they describe what went wrong
/// from the application's perspective, not the auth provider's.
#[derive(Debug, Clone, Error)]
pub enum AuthError {
    /// The token is missing, malformed, or has an invalid signature.
    #[error("Invalid or expired token")]
    InvalidToken,

    /// The token has expired (separate from InvalidToken for specific handling).
    #[error("Token expired")]
    TokenExpired,

    /// Token is valid but the user no longer exists in the system.
    #[error("User not found")]
    UserNotFound,

    /// User exists but lacks required permissions for this action.
    #[error("Insufficient permissions")]
    InsufficientPermissions,

    /// The authentication service is unavailable (network, config, etc.).
    #[error("Auth service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl AuthError {
    /// Creates a service unavailable error with a message.
    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::ServiceUnavailable(message.into())
    }

    /// Returns true if this error indicates the user should re-authenticate.
    pub fn requires_reauthentication(&self) -> bool {
        matches!(
            self,
            AuthError::InvalidToken | AuthError::TokenExpired | AuthError::UserNotFound
        )
    }

    /// Returns true if this is a transient error that may succeed on retry.
    pub fn is_transient(&self) -> bool {
        matches!(self, AuthError::ServiceUnavailable(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_user_id() -> UserId {
        UserId::new("user-123").unwrap()
    }

    #[test]
    fn authenticated_user_new_creates_user() {
        let user = AuthenticatedUser::new(
            test_user_id(),
            "test@example.com",
            Some("Test User".to_string()),
            true,
        );

        assert_eq!(user.id.as_str(), "user-123");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.display_name, Some("Test User".to_string()));
        assert!(user.email_verified);
    }

    #[test]
    fn authenticated_user_display_name_or_email_returns_name_when_present() {
        let user = AuthenticatedUser::new(
            test_user_id(),
            "test@example.com",
            Some("Alice".to_string()),
            true,
        );

        assert_eq!(user.display_name_or_email(), "Alice");
    }

    #[test]
    fn authenticated_user_display_name_or_email_returns_email_when_no_name() {
        let user = AuthenticatedUser::new(test_user_id(), "bob@example.com", None, true);

        assert_eq!(user.display_name_or_email(), "bob@example.com");
    }

    #[test]
    fn auth_error_invalid_token_displays_correctly() {
        let err = AuthError::InvalidToken;
        assert_eq!(format!("{}", err), "Invalid or expired token");
    }

    #[test]
    fn auth_error_token_expired_displays_correctly() {
        let err = AuthError::TokenExpired;
        assert_eq!(format!("{}", err), "Token expired");
    }

    #[test]
    fn auth_error_service_unavailable_displays_message() {
        let err = AuthError::service_unavailable("Connection refused");
        assert_eq!(format!("{}", err), "Auth service unavailable: Connection refused");
    }

    #[test]
    fn auth_error_requires_reauthentication_for_token_errors() {
        assert!(AuthError::InvalidToken.requires_reauthentication());
        assert!(AuthError::TokenExpired.requires_reauthentication());
        assert!(AuthError::UserNotFound.requires_reauthentication());
        assert!(!AuthError::InsufficientPermissions.requires_reauthentication());
        assert!(!AuthError::service_unavailable("").requires_reauthentication());
    }

    #[test]
    fn auth_error_is_transient_for_service_errors() {
        assert!(AuthError::service_unavailable("timeout").is_transient());
        assert!(!AuthError::InvalidToken.is_transient());
        assert!(!AuthError::TokenExpired.is_transient());
    }
}
