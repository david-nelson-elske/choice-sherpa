//! Authentication middleware and extractors for axum.
//!
//! This module provides:
//! - `auth_middleware` - Layer that validates Bearer tokens and injects user into extensions
//! - `RequireAuth` - Extractor that requires authentication
//! - `OptionalAuth` - Extractor for optional authentication
//!
//! # Architecture
//!
//! The middleware uses the `SessionValidator` port, keeping it provider-agnostic.
//! Whether using Zitadel, Auth0, or a mock for testing, the middleware doesn't change.
//!
//! ```text
//! Request → auth_middleware → injects AuthenticatedUser into extensions
//!                                      ↓
//!                              Handler → RequireAuth extractor reads from extensions
//! ```
//!
//! # Example
//!
//! ```ignore
//! use axum::{Router, routing::get, middleware};
//! use std::sync::Arc;
//!
//! let validator: Arc<dyn SessionValidator> = Arc::new(MockSessionValidator::new());
//!
//! let app = Router::new()
//!     .route("/api/protected", get(protected_handler))
//!     .layer(middleware::from_fn_with_state(validator.clone(), auth_middleware));
//!
//! async fn protected_handler(RequireAuth(user): RequireAuth) -> String {
//!     format!("Hello, {}!", user.email)
//! }
//! ```

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::domain::foundation::{AuthenticatedUser, AuthError};
use crate::ports::SessionValidator;

/// Auth middleware state - wraps the session validator.
pub type AuthState = Arc<dyn SessionValidator>;

/// Authentication middleware that validates Bearer tokens.
///
/// This middleware:
/// 1. Extracts the Bearer token from the Authorization header
/// 2. Validates the token using the `SessionValidator` port
/// 3. On success, injects `AuthenticatedUser` into request extensions
/// 4. On missing token, continues without injecting (for optional auth routes)
/// 5. On invalid token, returns 401 Unauthorized
///
/// # Token Extraction
///
/// Expects the token in the `Authorization` header with `Bearer` prefix:
/// ```text
/// Authorization: Bearer <token>
/// ```
pub async fn auth_middleware(
    State(validator): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Response {
    // Extract Bearer token from Authorization header
    let token = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    match token {
        Some(token) => {
            // Validate the token
            match validator.validate(token).await {
                Ok(user) => {
                    // Inject authenticated user into request extensions
                    request.extensions_mut().insert(user);
                    next.run(request).await
                }
                Err(e) => {
                    // Token validation failed
                    let (status, message) = match &e {
                        AuthError::TokenExpired => {
                            (StatusCode::UNAUTHORIZED, "Token expired")
                        }
                        AuthError::InvalidToken => {
                            (StatusCode::UNAUTHORIZED, "Invalid token")
                        }
                        AuthError::ServiceUnavailable(msg) => {
                            tracing::error!("Auth service unavailable: {}", msg);
                            (StatusCode::SERVICE_UNAVAILABLE, "Authentication service unavailable")
                        }
                        _ => (StatusCode::UNAUTHORIZED, "Authentication failed"),
                    };

                    (
                        status,
                        Json(serde_json::json!({
                            "error": message,
                            "code": "AUTH_ERROR"
                        })),
                    )
                        .into_response()
                }
            }
        }
        None => {
            // No token provided - continue without auth
            // Handlers can use RequireAuth to enforce authentication
            next.run(request).await
        }
    }
}

/// Extractor that requires authentication.
///
/// Use this extractor in handlers that require an authenticated user.
/// If no user is in the request extensions (i.e., auth middleware didn't
/// successfully validate a token), returns 401 Unauthorized.
///
/// # Example
///
/// ```ignore
/// async fn my_handler(RequireAuth(user): RequireAuth) -> impl IntoResponse {
///     format!("Hello, {}!", user.email)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RequireAuth(pub AuthenticatedUser);

impl<S> axum::extract::FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
{
    type Rejection = AuthRejection;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut axum::http::request::Parts,
        _state: &'life1 S,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            parts
                .extensions
                .get::<AuthenticatedUser>()
                .cloned()
                .map(RequireAuth)
                .ok_or(AuthRejection::Unauthenticated)
        })
    }
}

/// Extractor for optional authentication.
///
/// Use when authentication is optional - returns `None` if no valid
/// token was provided, `Some(user)` if authenticated.
///
/// # Example
///
/// ```ignore
/// async fn my_handler(OptionalAuth(user): OptionalAuth) -> impl IntoResponse {
///     match user {
///         Some(u) => format!("Hello, {}!", u.email),
///         None => "Hello, guest!".to_string(),
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct OptionalAuth(pub Option<AuthenticatedUser>);

impl<S> axum::extract::FromRequestParts<S> for OptionalAuth
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut axum::http::request::Parts,
        _state: &'life1 S,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let user = parts.extensions.get::<AuthenticatedUser>().cloned();
            Ok(OptionalAuth(user))
        })
    }
}

/// Rejection type for authentication failures.
#[derive(Debug, Clone)]
pub enum AuthRejection {
    /// No valid authentication token was provided.
    Unauthenticated,
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthRejection::Unauthenticated => {
                (StatusCode::UNAUTHORIZED, "Authentication required")
            }
        };

        (
            status,
            Json(serde_json::json!({
                "error": message,
                "code": "UNAUTHENTICATED"
            })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::auth::MockSessionValidator;
    use crate::domain::foundation::UserId;

    fn test_user() -> AuthenticatedUser {
        AuthenticatedUser::new(
            UserId::new("user-123").unwrap(),
            "test@example.com",
            Some("Test User".to_string()),
            true,
        )
    }

    // ════════════════════════════════════════════════════════════════════════════
    // SessionValidator Tests (indirect via MockSessionValidator)
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn validator_returns_user_for_valid_token() {
        let validator: Arc<dyn SessionValidator> = Arc::new(
            MockSessionValidator::new().with_user("valid-token", test_user()),
        );

        let result = validator.validate("valid-token").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().email, "test@example.com");
    }

    #[tokio::test]
    async fn validator_returns_error_for_invalid_token() {
        let validator: Arc<dyn SessionValidator> = Arc::new(MockSessionValidator::new());

        let result = validator.validate("invalid-token").await;
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    // ════════════════════════════════════════════════════════════════════════════
    // RequireAuth Extractor Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn require_auth_extracts_user_from_extensions() {
        use axum::extract::FromRequestParts;
        use axum::http::Request;

        // Create a request with AuthenticatedUser in extensions
        let mut request: Request<()> = Request::builder()
            .uri("/test")
            .body(())
            .unwrap();
        request.extensions_mut().insert(test_user());

        // Split into parts
        let (mut parts, _body) = request.into_parts();

        // Extract using RequireAuth
        let result: Result<RequireAuth, AuthRejection> =
            RequireAuth::from_request_parts(&mut parts, &()).await;

        assert!(result.is_ok());
        let RequireAuth(user) = result.unwrap();
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn require_auth_fails_without_user() {
        use axum::extract::FromRequestParts;
        use axum::http::Request;

        // Create a request WITHOUT AuthenticatedUser
        let request: Request<()> = Request::builder()
            .uri("/test")
            .body(())
            .unwrap();

        let (mut parts, _body) = request.into_parts();

        let result: Result<RequireAuth, AuthRejection> =
            RequireAuth::from_request_parts(&mut parts, &()).await;

        assert!(matches!(result, Err(AuthRejection::Unauthenticated)));
    }

    // ════════════════════════════════════════════════════════════════════════════
    // OptionalAuth Extractor Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn optional_auth_returns_some_when_present() {
        use axum::extract::FromRequestParts;
        use axum::http::Request;

        let mut request: Request<()> = Request::builder()
            .uri("/test")
            .body(())
            .unwrap();
        request.extensions_mut().insert(test_user());

        let (mut parts, _body) = request.into_parts();

        let result: Result<OptionalAuth, std::convert::Infallible> =
            OptionalAuth::from_request_parts(&mut parts, &()).await;

        assert!(result.is_ok());
        let OptionalAuth(user) = result.unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().email, "test@example.com");
    }

    #[tokio::test]
    async fn optional_auth_returns_none_when_absent() {
        use axum::extract::FromRequestParts;
        use axum::http::Request;

        let request: Request<()> = Request::builder()
            .uri("/test")
            .body(())
            .unwrap();

        let (mut parts, _body) = request.into_parts();

        let result: Result<OptionalAuth, std::convert::Infallible> =
            OptionalAuth::from_request_parts(&mut parts, &()).await;

        assert!(result.is_ok());
        let OptionalAuth(user) = result.unwrap();
        assert!(user.is_none());
    }

    // ════════════════════════════════════════════════════════════════════════════
    // AuthRejection Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn auth_rejection_returns_401() {
        let rejection = AuthRejection::Unauthenticated;
        let response = rejection.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Token Extraction Helper Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn bearer_token_extraction() {
        // Test the pattern used in auth_middleware
        let header_value = "Bearer my-secret-token";
        let token = header_value.strip_prefix("Bearer ");
        assert_eq!(token, Some("my-secret-token"));

        // Without Bearer prefix
        let header_value = "my-secret-token";
        let token = header_value.strip_prefix("Bearer ");
        assert_eq!(token, None);

        // With different prefix
        let header_value = "Basic dXNlcjpwYXNz";
        let token = header_value.strip_prefix("Bearer ");
        assert_eq!(token, None);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Type Safety Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn auth_state_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AuthState>();
    }

    #[test]
    fn require_auth_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RequireAuth>();
    }

    #[test]
    fn optional_auth_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OptionalAuth>();
    }
}
