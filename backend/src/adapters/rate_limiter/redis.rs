//! Redis-backed rate limiter implementation for production deployments.
//!
//! Uses a simple fixed-window counter algorithm with Redis INCR + EXPIRE.
//! Suitable for multi-server deployments.

use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;

use crate::domain::foundation::Timestamp;
use crate::domain::membership::MembershipTier;
use crate::ports::{
    RateLimitDenied, RateLimitError, RateLimitKey, RateLimitResult, RateLimitScope,
    RateLimitStatus, RateLimiter,
};

use super::config::RateLimitConfig;

/// Redis-backed rate limiter for production multi-server deployments.
///
/// Uses a fixed-window counter algorithm:
/// 1. INCR the key to increment the counter
/// 2. If count is 1, set EXPIRE for the window duration
/// 3. If count > limit, deny the request
///
/// This approach is simple and atomic but has a known edge case at window
/// boundaries where requests can briefly exceed limits. For most use cases,
/// this is acceptable behavior.
#[derive(Clone)]
pub struct RedisRateLimiter {
    conn: MultiplexedConnection,
    config: RateLimitConfig,
    default_tier: MembershipTier,
}

impl RedisRateLimiter {
    /// Create a new Redis rate limiter.
    pub fn new(conn: MultiplexedConnection, config: RateLimitConfig) -> Self {
        Self {
            conn,
            config,
            default_tier: MembershipTier::Free,
        }
    }

    /// Set the default tier for users without explicit tier.
    pub fn with_default_tier(mut self, tier: MembershipTier) -> Self {
        self.default_tier = tier;
        self
    }

    /// Get the limit and window for a key.
    fn limits_for(&self, key: &RateLimitKey) -> (u32, u32) {
        match key.scope {
            RateLimitScope::Global => (self.config.global.requests_per_minute, 60),
            RateLimitScope::Ip => (self.config.per_ip.requests_per_minute, 60),
            RateLimitScope::User => {
                let tier_limits = self.config.limits_for_tier(self.default_tier);
                tier_limits.limit_for_resource(key.resource.as_deref())
            }
            RateLimitScope::Resource => {
                let resource = key.resource.as_deref().unwrap_or("default");
                self.config
                    .resources
                    .get(resource)
                    .map(|r| (r.requests_per_window, r.window_secs))
                    .unwrap_or((100, 60))
            }
        }
    }
}

#[async_trait]
impl RateLimiter for RedisRateLimiter {
    async fn check(&self, key: RateLimitKey) -> Result<RateLimitResult, RateLimitError> {
        let redis_key = key.to_redis_key();
        let (limit, window_secs) = self.limits_for(&key);

        let mut conn = self.conn.clone();

        // Atomic increment
        let count: i64 = conn
            .incr(&redis_key, 1_i64)
            .await
            .map_err(|e: redis::RedisError| RateLimitError::Unavailable(e.to_string()))?;

        // Set expiry on first request in window
        if count == 1 {
            conn.expire::<_, ()>(&redis_key, window_secs as i64)
                .await
                .map_err(|e: redis::RedisError| RateLimitError::Unavailable(e.to_string()))?;
        }

        // Get TTL for reset time
        let ttl: i64 = conn
            .ttl(&redis_key)
            .await
            .map_err(|e: redis::RedisError| RateLimitError::Unavailable(e.to_string()))?;

        let now = Timestamp::now().as_unix_secs();
        let reset_secs = if ttl > 0 { ttl as u64 } else { window_secs as u64 };
        let reset_at = Timestamp::from_unix_secs(now + reset_secs);

        if count as u32 > limit {
            let retry_after = reset_secs as u32;
            return Ok(RateLimitResult::Denied(RateLimitDenied {
                limit,
                retry_after_secs: retry_after.max(1),
                scope: key.scope,
                message: format!(
                    "Rate limit exceeded for {}. Retry after {} seconds.",
                    key.scope, retry_after
                ),
            }));
        }

        let remaining = limit.saturating_sub(count as u32);

        Ok(RateLimitResult::Allowed(RateLimitStatus {
            limit,
            remaining,
            reset_at,
            window_secs,
        }))
    }

    async fn status(&self, key: RateLimitKey) -> Result<RateLimitStatus, RateLimitError> {
        let redis_key = key.to_redis_key();
        let (limit, window_secs) = self.limits_for(&key);

        let mut conn = self.conn.clone();

        // Get current count (or 0 if not set)
        let count: Option<i64> = conn
            .get(&redis_key)
            .await
            .map_err(|e: redis::RedisError| RateLimitError::Unavailable(e.to_string()))?;

        let count = count.unwrap_or(0) as u32;
        let remaining = limit.saturating_sub(count);

        // Get TTL for reset time
        let ttl: i64 = conn
            .ttl(&redis_key)
            .await
            .map_err(|e: redis::RedisError| RateLimitError::Unavailable(e.to_string()))?;

        let now = Timestamp::now().as_unix_secs();
        let reset_secs = if ttl > 0 { ttl as u64 } else { window_secs as u64 };
        let reset_at = Timestamp::from_unix_secs(now + reset_secs);

        Ok(RateLimitStatus {
            limit,
            remaining,
            reset_at,
            window_secs,
        })
    }

    async fn reset(&self, key: RateLimitKey) -> Result<(), RateLimitError> {
        let redis_key = key.to_redis_key();
        let mut conn = self.conn.clone();

        conn.del::<_, ()>(&redis_key)
            .await
            .map_err(|e: redis::RedisError| RateLimitError::Unavailable(e.to_string()))?;

        Ok(())
    }
}

impl std::fmt::Debug for RedisRateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisRateLimiter")
            .field("config", &self.config)
            .field("default_tier", &self.default_tier)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    // Note: Redis integration tests require a running Redis instance
    // and are typically run separately from unit tests.
    //
    // Example test setup:
    //
    // #[tokio::test]
    // #[ignore] // Run with: cargo test -- --ignored
    // async fn test_redis_rate_limiter() {
    //     let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    //     let conn = client.get_multiplexed_tokio_connection().await.unwrap();
    //     let limiter = RedisRateLimiter::new(conn, RateLimitConfig::default());
    //     // ... test code
    // }
}
