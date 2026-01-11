//! Rate limiting port for protecting APIs and controlling costs.
//!
//! This port defines the interface for rate limiting operations using
//! a token bucket algorithm. Implementations can use in-memory storage
//! for testing or Redis for production.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::domain::foundation::{Timestamp, UserId};

/// Port for rate limiting operations.
///
/// Implementations should be thread-safe and support concurrent access.
/// The rate limiter uses a fixed-window counter algorithm for simplicity.
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if request is allowed, consuming a token if so.
    ///
    /// Returns `Allowed` with remaining quota or `Denied` with retry info.
    async fn check(&self, key: RateLimitKey) -> Result<RateLimitResult, RateLimitError>;

    /// Get current rate limit status without consuming a token.
    ///
    /// Useful for displaying quota information to users.
    async fn status(&self, key: RateLimitKey) -> Result<RateLimitStatus, RateLimitError>;

    /// Reset rate limit for a key (admin operation).
    ///
    /// Clears the current window, restoring full quota.
    async fn reset(&self, key: RateLimitKey) -> Result<(), RateLimitError>;
}

/// Key identifying what to rate limit.
///
/// Rate limits can be scoped globally, per-IP, per-user, or per-resource.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct RateLimitKey {
    /// The scope of this rate limit.
    pub scope: RateLimitScope,
    /// Identifier within the scope (e.g., IP address, user ID).
    pub identifier: String,
    /// Optional resource for finer-grained limits (e.g., "ai_completions").
    pub resource: Option<String>,
}

/// The scope at which rate limiting is applied.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitScope {
    /// Global rate limit across all requests.
    Global,
    /// Per-IP address rate limit.
    Ip,
    /// Per-authenticated-user rate limit.
    User,
    /// Per-resource rate limit (e.g., specific API endpoint).
    Resource,
}

impl RateLimitKey {
    /// Creates a global rate limit key.
    pub fn global() -> Self {
        Self {
            scope: RateLimitScope::Global,
            identifier: "global".to_string(),
            resource: None,
        }
    }

    /// Creates an IP-based rate limit key.
    pub fn ip(ip: &str) -> Self {
        Self {
            scope: RateLimitScope::Ip,
            identifier: ip.to_string(),
            resource: None,
        }
    }

    /// Creates a user-based rate limit key.
    pub fn user(user_id: &UserId) -> Self {
        Self {
            scope: RateLimitScope::User,
            identifier: user_id.to_string(),
            resource: None,
        }
    }

    /// Creates a user-based rate limit key for a specific resource.
    pub fn user_resource(user_id: &UserId, resource: &str) -> Self {
        Self {
            scope: RateLimitScope::User,
            identifier: user_id.to_string(),
            resource: Some(resource.to_string()),
        }
    }

    /// Returns the Redis key string for this rate limit key.
    pub fn to_redis_key(&self) -> String {
        match &self.resource {
            Some(resource) => format!(
                "ratelimit:{}:{}:{}",
                self.scope.as_str(),
                self.identifier,
                resource
            ),
            None => format!("ratelimit:{}:{}", self.scope.as_str(), self.identifier),
        }
    }
}

impl RateLimitScope {
    /// Returns the string representation of the scope.
    pub fn as_str(&self) -> &'static str {
        match self {
            RateLimitScope::Global => "global",
            RateLimitScope::Ip => "ip",
            RateLimitScope::User => "user",
            RateLimitScope::Resource => "resource",
        }
    }
}

impl fmt::Display for RateLimitScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Result of a rate limit check.
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    /// Request is allowed; includes current status.
    Allowed(RateLimitStatus),
    /// Request is denied; includes denial details.
    Denied(RateLimitDenied),
}

impl RateLimitResult {
    /// Returns true if the request was allowed.
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed(_))
    }

    /// Returns true if the request was denied.
    pub fn is_denied(&self) -> bool {
        matches!(self, RateLimitResult::Denied(_))
    }
}

/// Current rate limit status.
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    /// Maximum requests allowed in the window.
    pub limit: u32,
    /// Remaining requests in the current window.
    pub remaining: u32,
    /// When the current window resets.
    pub reset_at: Timestamp,
    /// Window duration in seconds.
    pub window_secs: u32,
}

/// Details of a rate limit denial.
#[derive(Debug, Clone)]
pub struct RateLimitDenied {
    /// Maximum requests allowed in the window.
    pub limit: u32,
    /// Seconds until the client should retry.
    pub retry_after_secs: u32,
    /// The scope that triggered the denial.
    pub scope: RateLimitScope,
    /// Human-readable message explaining the denial.
    pub message: String,
}

/// Errors that can occur during rate limiting operations.
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    /// Rate limiter backend is unavailable.
    #[error("rate limiter unavailable: {0}")]
    Unavailable(String),

    /// Invalid rate limit key provided.
    #[error("invalid key: {0}")]
    InvalidKey(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_key_has_correct_scope() {
        let key = RateLimitKey::global();
        assert_eq!(key.scope, RateLimitScope::Global);
        assert_eq!(key.identifier, "global");
        assert!(key.resource.is_none());
    }

    #[test]
    fn ip_key_has_correct_scope() {
        let key = RateLimitKey::ip("192.168.1.1");
        assert_eq!(key.scope, RateLimitScope::Ip);
        assert_eq!(key.identifier, "192.168.1.1");
        assert!(key.resource.is_none());
    }

    #[test]
    fn user_key_has_correct_scope() {
        let user_id = UserId::new("user-123").unwrap();
        let key = RateLimitKey::user(&user_id);
        assert_eq!(key.scope, RateLimitScope::User);
        assert_eq!(key.identifier, "user-123");
        assert!(key.resource.is_none());
    }

    #[test]
    fn user_resource_key_includes_resource() {
        let user_id = UserId::new("user-123").unwrap();
        let key = RateLimitKey::user_resource(&user_id, "ai_completions");
        assert_eq!(key.scope, RateLimitScope::User);
        assert_eq!(key.identifier, "user-123");
        assert_eq!(key.resource, Some("ai_completions".to_string()));
    }

    #[test]
    fn redis_key_format_without_resource() {
        let key = RateLimitKey::ip("10.0.0.1");
        assert_eq!(key.to_redis_key(), "ratelimit:ip:10.0.0.1");
    }

    #[test]
    fn redis_key_format_with_resource() {
        let user_id = UserId::new("user-456").unwrap();
        let key = RateLimitKey::user_resource(&user_id, "exports");
        assert_eq!(key.to_redis_key(), "ratelimit:user:user-456:exports");
    }

    #[test]
    fn rate_limit_result_is_allowed_works() {
        let status = RateLimitStatus {
            limit: 100,
            remaining: 50,
            reset_at: Timestamp::now(),
            window_secs: 60,
        };
        let result = RateLimitResult::Allowed(status);
        assert!(result.is_allowed());
        assert!(!result.is_denied());
    }

    #[test]
    fn rate_limit_result_is_denied_works() {
        let denied = RateLimitDenied {
            limit: 100,
            retry_after_secs: 30,
            scope: RateLimitScope::User,
            message: "Rate limit exceeded".to_string(),
        };
        let result = RateLimitResult::Denied(denied);
        assert!(result.is_denied());
        assert!(!result.is_allowed());
    }

    #[test]
    fn scope_as_str_returns_correct_values() {
        assert_eq!(RateLimitScope::Global.as_str(), "global");
        assert_eq!(RateLimitScope::Ip.as_str(), "ip");
        assert_eq!(RateLimitScope::User.as_str(), "user");
        assert_eq!(RateLimitScope::Resource.as_str(), "resource");
    }
}
