//! Rate limiting middleware for axum.
//!
//! This module provides middleware that enforces rate limits using the `RateLimiter` port.
//!
//! # Architecture
//!
//! The middleware checks multiple rate limit scopes in order:
//! 1. Global rate limit (infrastructure protection)
//! 2. Per-IP rate limit (brute-force protection)
//! 3. Per-user rate limit (if authenticated) with tier-based limits
//!
//! Rate limit status is returned in standard HTTP headers:
//! - `X-RateLimit-Limit`: Maximum requests allowed in the window
//! - `X-RateLimit-Remaining`: Requests remaining in the current window
//! - `X-RateLimit-Reset`: Unix timestamp when the window resets
//! - `Retry-After`: Seconds to wait (only on 429 response)
//!
//! # Example
//!
//! ```ignore
//! use axum::{Router, routing::get, middleware};
//! use std::sync::Arc;
//!
//! let limiter: Arc<dyn RateLimiter> = Arc::new(InMemoryRateLimiter::with_defaults());
//!
//! let app = Router::new()
//!     .route("/api/resource", get(handler))
//!     .layer(middleware::from_fn_with_state(limiter, rate_limit_middleware));
//! ```

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderName, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::domain::foundation::AuthenticatedUser;
use crate::ports::{RateLimitKey, RateLimitResult, RateLimiter};

/// Rate limiter middleware state.
pub type RateLimiterState = Arc<dyn RateLimiter>;

/// Standard rate limit header names.
pub mod headers {
    use super::HeaderName;

    /// Maximum requests allowed in the window.
    pub static X_RATELIMIT_LIMIT: HeaderName = HeaderName::from_static("x-ratelimit-limit");
    /// Requests remaining in the current window.
    pub static X_RATELIMIT_REMAINING: HeaderName = HeaderName::from_static("x-ratelimit-remaining");
    /// Unix timestamp when the window resets.
    pub static X_RATELIMIT_RESET: HeaderName = HeaderName::from_static("x-ratelimit-reset");
}

/// Rate limiting middleware that checks global, IP, and user limits.
///
/// This middleware:
/// 1. Extracts client IP from `ConnectInfo` or forwarded headers
/// 2. Checks global rate limit first
/// 3. Checks per-IP rate limit
/// 4. If authenticated, checks per-user rate limit
/// 5. Returns 429 Too Many Requests if any limit exceeded
/// 6. Adds rate limit headers to all responses
///
/// The middleware returns the most restrictive rate limit in headers.
pub async fn rate_limit_middleware(
    State(limiter): State<RateLimiterState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    request: Request,
    next: Next,
) -> Response {
    // Extract client IP
    let client_ip = extract_client_ip(&request, connect_info.as_ref());

    // Extract authenticated user if present
    let user = request.extensions().get::<AuthenticatedUser>().cloned();

    // Check rate limits in order of scope
    // Global limit is checked first for infrastructure protection
    let global_key = RateLimitKey::global();
    match limiter.check(global_key).await {
        Ok(RateLimitResult::Denied(denied)) => {
            return rate_limit_response(denied.limit, 0, denied.retry_after_secs);
        }
        Err(e) => {
            tracing::warn!("Rate limiter unavailable: {}", e);
            // Continue on error - fail open for availability
        }
        Ok(RateLimitResult::Allowed(_)) => {}
    }

    // Per-IP rate limit
    if let Some(ip) = &client_ip {
        let ip_key = RateLimitKey::ip(ip);
        match limiter.check(ip_key).await {
            Ok(RateLimitResult::Denied(denied)) => {
                return rate_limit_response(denied.limit, 0, denied.retry_after_secs);
            }
            Err(e) => {
                tracing::warn!("Rate limiter unavailable for IP check: {}", e);
            }
            Ok(RateLimitResult::Allowed(_)) => {}
        }
    }

    // Per-user rate limit (if authenticated)
    let user_status = if let Some(ref user) = user {
        let user_key = RateLimitKey::user(&user.id);
        match limiter.check(user_key).await {
            Ok(RateLimitResult::Denied(denied)) => {
                return rate_limit_response(denied.limit, 0, denied.retry_after_secs);
            }
            Ok(RateLimitResult::Allowed(status)) => Some(status),
            Err(e) => {
                tracing::warn!("Rate limiter unavailable for user check: {}", e);
                None
            }
        }
    } else {
        None
    };

    // All checks passed - continue to handler
    let mut response = next.run(request).await;

    // Add rate limit headers from the most specific limit (user > IP > global)
    if let Some(status) = user_status {
        add_rate_limit_headers(&mut response, status.limit, status.remaining, status.reset_at.as_unix_secs());
    } else if let Some(ip) = &client_ip {
        // Get IP status for headers (without consuming a request)
        if let Ok(status) = limiter.status(RateLimitKey::ip(ip)).await {
            add_rate_limit_headers(&mut response, status.limit, status.remaining, status.reset_at.as_unix_secs());
        }
    }

    response
}

/// Extract client IP from request, checking forwarded headers first.
///
/// Order of precedence:
/// 1. X-Forwarded-For header (first IP in list)
/// 2. X-Real-IP header
/// 3. ConnectInfo socket address
fn extract_client_ip<B>(
    request: &axum::http::Request<B>,
    connect_info: Option<&ConnectInfo<SocketAddr>>,
) -> Option<String> {
    // Check X-Forwarded-For first (for reverse proxy setups)
    if let Some(forwarded) = request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
    {
        // Take the first IP (client IP, before any proxies)
        if let Some(first_ip) = forwarded.split(',').next() {
            return Some(first_ip.trim().to_string());
        }
    }

    // Check X-Real-IP
    if let Some(real_ip) = request
        .headers()
        .get("X-Real-IP")
        .and_then(|h| h.to_str().ok())
    {
        return Some(real_ip.to_string());
    }

    // Fall back to ConnectInfo
    connect_info.map(|ci| ci.0.ip().to_string())
}

/// Create a 429 Too Many Requests response.
fn rate_limit_response(limit: u32, remaining: u32, retry_after_secs: u32) -> Response {
    let mut response = (
        StatusCode::TOO_MANY_REQUESTS,
        Json(serde_json::json!({
            "error": "Rate limit exceeded",
            "code": "RATE_LIMIT_EXCEEDED",
            "retry_after_secs": retry_after_secs
        })),
    )
        .into_response();

    // Add rate limit headers
    let headers = response.headers_mut();
    headers.insert(
        headers::X_RATELIMIT_LIMIT.clone(),
        HeaderValue::from_str(&limit.to_string()).unwrap(),
    );
    headers.insert(
        headers::X_RATELIMIT_REMAINING.clone(),
        HeaderValue::from_str(&remaining.to_string()).unwrap(),
    );
    headers.insert(
        "Retry-After",
        HeaderValue::from_str(&retry_after_secs.to_string()).unwrap(),
    );

    response
}

/// Add rate limit headers to a response.
fn add_rate_limit_headers(response: &mut Response, limit: u32, remaining: u32, reset_at: u64) {
    let headers = response.headers_mut();
    headers.insert(
        headers::X_RATELIMIT_LIMIT.clone(),
        HeaderValue::from_str(&limit.to_string()).unwrap(),
    );
    headers.insert(
        headers::X_RATELIMIT_REMAINING.clone(),
        HeaderValue::from_str(&remaining.to_string()).unwrap(),
    );
    headers.insert(
        headers::X_RATELIMIT_RESET.clone(),
        HeaderValue::from_str(&reset_at.to_string()).unwrap(),
    );
}

/// Rate limit extractor for per-resource limiting in handlers.
///
/// Use this when you need resource-specific rate limiting beyond
/// the general middleware limits. For example, AI completion endpoints
/// may have stricter limits than general API endpoints.
///
/// # Example
///
/// ```ignore
/// async fn ai_completion(
///     RequireAuth(user): RequireAuth,
///     rate_check: RateLimitCheck,
/// ) -> Result<impl IntoResponse, RateLimitRejection> {
///     // Check AI-specific rate limit
///     rate_check.check_resource(&user.user_id, "ai_completions").await?;
///     // ... handle request
/// }
/// ```
#[derive(Clone)]
pub struct RateLimitCheck {
    limiter: Arc<dyn RateLimiter>,
}

impl RateLimitCheck {
    /// Create a new rate limit checker.
    pub fn new(limiter: Arc<dyn RateLimiter>) -> Self {
        Self { limiter }
    }

    /// Check rate limit for a specific resource.
    pub async fn check_resource(
        &self,
        user_id: &crate::domain::foundation::UserId,
        resource: &str,
    ) -> Result<crate::ports::RateLimitStatus, RateLimitRejection> {
        let key = RateLimitKey::user_resource(user_id, resource);
        match self.limiter.check(key).await {
            Ok(RateLimitResult::Allowed(status)) => Ok(status),
            Ok(RateLimitResult::Denied(denied)) => Err(RateLimitRejection {
                limit: denied.limit,
                retry_after_secs: denied.retry_after_secs,
                message: denied.message,
            }),
            Err(e) => {
                tracing::warn!("Rate limiter unavailable: {}", e);
                // Fail open - return a dummy status
                Ok(crate::ports::RateLimitStatus {
                    limit: 0,
                    remaining: 0,
                    reset_at: crate::domain::foundation::Timestamp::now(),
                    window_secs: 60,
                })
            }
        }
    }
}

/// Rejection for rate limit exceeded.
#[derive(Debug, Clone)]
pub struct RateLimitRejection {
    /// The rate limit that was exceeded.
    pub limit: u32,
    /// Seconds until the limit resets.
    pub retry_after_secs: u32,
    /// Human-readable message.
    pub message: String,
}

impl IntoResponse for RateLimitRejection {
    fn into_response(self) -> Response {
        rate_limit_response(self.limit, 0, self.retry_after_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::rate_limiter::{InMemoryRateLimiter, RateLimitConfig};
    use crate::domain::foundation::UserId;
    use axum::http::Request;

    fn test_limiter() -> Arc<dyn RateLimiter> {
        Arc::new(InMemoryRateLimiter::with_defaults())
    }

    // ════════════════════════════════════════════════════════════════════════════
    // IP Extraction Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn extract_ip_from_x_forwarded_for() {
        let request = Request::builder()
            .uri("/test")
            .header("X-Forwarded-For", "1.2.3.4, 5.6.7.8")
            .body(())
            .unwrap();

        let ip = extract_client_ip(&request, None);
        assert_eq!(ip, Some("1.2.3.4".to_string()));
    }

    #[test]
    fn extract_ip_from_x_real_ip() {
        let request = Request::builder()
            .uri("/test")
            .header("X-Real-IP", "9.8.7.6")
            .body(())
            .unwrap();

        let ip = extract_client_ip(&request, None);
        assert_eq!(ip, Some("9.8.7.6".to_string()));
    }

    #[test]
    fn extract_ip_prefers_x_forwarded_for() {
        let request = Request::builder()
            .uri("/test")
            .header("X-Forwarded-For", "1.2.3.4")
            .header("X-Real-IP", "5.6.7.8")
            .body(())
            .unwrap();

        let ip = extract_client_ip(&request, None);
        assert_eq!(ip, Some("1.2.3.4".to_string()));
    }

    #[test]
    fn extract_ip_returns_none_without_headers() {
        let request = Request::builder().uri("/test").body(()).unwrap();

        let ip = extract_client_ip(&request, None);
        assert_eq!(ip, None);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Rate Limit Check Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn rate_limit_check_allows_within_limit() {
        let limiter = test_limiter();
        let checker = RateLimitCheck::new(limiter);
        let user_id = UserId::new("test-user").unwrap();

        let result = checker.check_resource(&user_id, "ai_completions").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn rate_limit_check_denies_at_limit() {
        let mut config = RateLimitConfig::default();
        // Set a very low AI completion limit
        config
            .per_tier
            .get_mut(&crate::domain::membership::MembershipTier::Free)
            .unwrap()
            .ai_completions_per_minute = 2;

        let limiter: Arc<dyn RateLimiter> = Arc::new(InMemoryRateLimiter::new(config));
        let checker = RateLimitCheck::new(limiter);
        let user_id = UserId::new("test-user").unwrap();

        // Use up the limit
        checker.check_resource(&user_id, "ai_completions").await.unwrap();
        checker.check_resource(&user_id, "ai_completions").await.unwrap();

        // Third request should be denied
        let result = checker.check_resource(&user_id, "ai_completions").await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.limit, 2);
        assert!(err.retry_after_secs > 0);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Response Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn rate_limit_response_has_429_status() {
        let response = rate_limit_response(100, 0, 60);
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn rate_limit_response_has_retry_after_header() {
        let response = rate_limit_response(100, 0, 30);
        let retry_after = response.headers().get("Retry-After").unwrap();
        assert_eq!(retry_after, "30");
    }

    #[test]
    fn rate_limit_response_has_limit_headers() {
        let response = rate_limit_response(100, 0, 60);
        assert!(response.headers().contains_key("x-ratelimit-limit"));
        assert!(response.headers().contains_key("x-ratelimit-remaining"));
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Type Safety Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn rate_limiter_state_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RateLimiterState>();
    }

    #[test]
    fn rate_limit_check_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RateLimitCheck>();
    }
}
