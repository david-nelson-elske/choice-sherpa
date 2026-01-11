//! In-memory rate limiter implementation for testing and development.
//!
//! Uses a fixed-window counter algorithm with an in-memory HashMap.
//! Not suitable for production multi-server deployments.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::foundation::Timestamp;
use crate::domain::membership::MembershipTier;
use crate::ports::{
    RateLimitDenied, RateLimitError, RateLimitKey, RateLimitResult, RateLimitScope,
    RateLimitStatus, RateLimiter,
};

use super::config::{RateLimitConfig, TierRateLimits};

/// In-memory rate limiter for testing and single-server deployments.
///
/// Uses a fixed-window counter algorithm. Each window tracks the count
/// of requests and resets when the window expires.
#[derive(Debug)]
pub struct InMemoryRateLimiter {
    /// Rate limit configuration.
    config: RateLimitConfig,
    /// Per-key window state.
    windows: Arc<RwLock<HashMap<String, WindowState>>>,
    /// Default tier for users without explicit tier.
    default_tier: MembershipTier,
}

/// State for a single rate limit window.
#[derive(Debug, Clone)]
struct WindowState {
    /// Number of requests in the current window.
    count: u32,
    /// When the current window started.
    window_start: u64,
    /// Window duration in seconds.
    window_secs: u32,
}

impl InMemoryRateLimiter {
    /// Create a new in-memory rate limiter with default configuration.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            windows: Arc::new(RwLock::new(HashMap::new())),
            default_tier: MembershipTier::Free,
        }
    }

    /// Create a rate limiter with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(RateLimitConfig::default())
    }

    /// Set the default tier for users.
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

    /// Get current timestamp as unix seconds.
    fn now_secs() -> u64 {
        Timestamp::now().as_unix_secs()
    }
}

#[async_trait]
impl RateLimiter for InMemoryRateLimiter {
    async fn check(&self, key: RateLimitKey) -> Result<RateLimitResult, RateLimitError> {
        let redis_key = key.to_redis_key();
        let (limit, window_secs) = self.limits_for(&key);
        let now = Self::now_secs();

        let mut windows = self.windows.write().await;

        // Get or create window state
        let state = windows.entry(redis_key.clone()).or_insert_with(|| WindowState {
            count: 0,
            window_start: now,
            window_secs,
        });

        // Check if window has expired
        let window_end = state.window_start + state.window_secs as u64;
        if now >= window_end {
            // Reset window
            state.count = 0;
            state.window_start = now;
        }

        // Check limit
        if state.count >= limit {
            let retry_after = (state.window_start + state.window_secs as u64)
                .saturating_sub(now) as u32;

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

        // Increment counter
        state.count += 1;
        let remaining = limit.saturating_sub(state.count);
        let reset_at = Timestamp::from_unix_secs(state.window_start + state.window_secs as u64);

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
        let now = Self::now_secs();

        let windows = self.windows.read().await;

        let (count, window_start) = windows
            .get(&redis_key)
            .map(|state| {
                // Check if window is still valid
                let window_end = state.window_start + state.window_secs as u64;
                if now >= window_end {
                    (0, now) // Window expired
                } else {
                    (state.count, state.window_start)
                }
            })
            .unwrap_or((0, now));

        let remaining = limit.saturating_sub(count);
        let reset_at = Timestamp::from_unix_secs(window_start + window_secs as u64);

        Ok(RateLimitStatus {
            limit,
            remaining,
            reset_at,
            window_secs,
        })
    }

    async fn reset(&self, key: RateLimitKey) -> Result<(), RateLimitError> {
        let redis_key = key.to_redis_key();
        let mut windows = self.windows.write().await;
        windows.remove(&redis_key);
        Ok(())
    }
}

/// In-memory rate limiter with tier awareness.
///
/// Extends InMemoryRateLimiter to support per-user tier lookups.
#[derive(Debug)]
pub struct TierAwareRateLimiter {
    inner: InMemoryRateLimiter,
    /// User tier overrides for testing.
    user_tiers: Arc<RwLock<HashMap<String, MembershipTier>>>,
}

impl TierAwareRateLimiter {
    /// Create a new tier-aware rate limiter.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            inner: InMemoryRateLimiter::new(config),
            user_tiers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set a user's tier for testing.
    pub async fn set_user_tier(&self, user_id: &str, tier: MembershipTier) {
        let mut tiers = self.user_tiers.write().await;
        tiers.insert(user_id.to_string(), tier);
    }

    /// Get a user's tier.
    pub async fn get_user_tier(&self, user_id: &str) -> MembershipTier {
        let tiers = self.user_tiers.read().await;
        tiers.get(user_id).copied().unwrap_or(MembershipTier::Free)
    }

    /// Get tier-specific limits for a user.
    pub async fn limits_for_user(&self, user_id: &str) -> &TierRateLimits {
        let tier = self.get_user_tier(user_id).await;
        self.inner.config.limits_for_tier(tier)
    }
}

#[async_trait]
impl RateLimiter for TierAwareRateLimiter {
    async fn check(&self, key: RateLimitKey) -> Result<RateLimitResult, RateLimitError> {
        self.inner.check(key).await
    }

    async fn status(&self, key: RateLimitKey) -> Result<RateLimitStatus, RateLimitError> {
        self.inner.status(key).await
    }

    async fn reset(&self, key: RateLimitKey) -> Result<(), RateLimitError> {
        self.inner.reset(key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::UserId;

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    // ─── Basic Functionality Tests ───────────────────────────────────

    #[tokio::test]
    async fn allows_requests_within_limit() {
        let limiter = InMemoryRateLimiter::with_defaults();
        let key = RateLimitKey::ip("192.168.1.1");

        // IP limit is 100 per minute
        for i in 0..10 {
            let result = limiter.check(key.clone()).await.unwrap();
            assert!(
                result.is_allowed(),
                "Request {} should be allowed",
                i + 1
            );
        }
    }

    #[tokio::test]
    async fn denies_requests_at_limit() {
        let mut config = RateLimitConfig::default();
        config.per_ip.requests_per_minute = 5; // Low limit for testing
        let limiter = InMemoryRateLimiter::new(config);
        let key = RateLimitKey::ip("192.168.1.1");

        // Use up the limit
        for _ in 0..5 {
            let result = limiter.check(key.clone()).await.unwrap();
            assert!(result.is_allowed());
        }

        // Next request should be denied
        let result = limiter.check(key.clone()).await.unwrap();
        assert!(result.is_denied());

        if let RateLimitResult::Denied(denied) = result {
            assert_eq!(denied.limit, 5);
            assert!(denied.retry_after_secs > 0);
            assert_eq!(denied.scope, RateLimitScope::Ip);
        }
    }

    #[tokio::test]
    async fn status_returns_remaining_count() {
        let mut config = RateLimitConfig::default();
        config.per_ip.requests_per_minute = 10;
        let limiter = InMemoryRateLimiter::new(config);
        let key = RateLimitKey::ip("10.0.0.1");

        // Initial status should show full limit
        let status = limiter.status(key.clone()).await.unwrap();
        assert_eq!(status.limit, 10);
        assert_eq!(status.remaining, 10);

        // Use 3 requests
        for _ in 0..3 {
            limiter.check(key.clone()).await.unwrap();
        }

        // Status should show 7 remaining
        let status = limiter.status(key.clone()).await.unwrap();
        assert_eq!(status.remaining, 7);
    }

    #[tokio::test]
    async fn reset_clears_counter() {
        let mut config = RateLimitConfig::default();
        config.per_ip.requests_per_minute = 5;
        let limiter = InMemoryRateLimiter::new(config);
        let key = RateLimitKey::ip("10.0.0.2");

        // Use up the limit
        for _ in 0..5 {
            limiter.check(key.clone()).await.unwrap();
        }

        // Verify denied
        let result = limiter.check(key.clone()).await.unwrap();
        assert!(result.is_denied());

        // Reset
        limiter.reset(key.clone()).await.unwrap();

        // Should be allowed again
        let result = limiter.check(key.clone()).await.unwrap();
        assert!(result.is_allowed());
    }

    // ─── Global Limit Tests ───────────────────────────────────────────

    #[tokio::test]
    async fn global_limit_applies_to_all_requests() {
        let mut config = RateLimitConfig::default();
        config.global.requests_per_minute = 3;
        let limiter = InMemoryRateLimiter::new(config);
        let key = RateLimitKey::global();

        for _ in 0..3 {
            let result = limiter.check(key.clone()).await.unwrap();
            assert!(result.is_allowed());
        }

        let result = limiter.check(key.clone()).await.unwrap();
        assert!(result.is_denied());
    }

    // ─── User Limit Tests ─────────────────────────────────────────────

    #[tokio::test]
    async fn user_limit_uses_tier_config() {
        let limiter = InMemoryRateLimiter::with_defaults();
        let user_id = test_user_id();
        let key = RateLimitKey::user(&user_id);

        // Default tier is Free, which has 60 req/min
        let status = limiter.status(key.clone()).await.unwrap();
        assert_eq!(status.limit, 60);
    }

    #[tokio::test]
    async fn user_resource_limit_applies_correctly() {
        let limiter = InMemoryRateLimiter::with_defaults();
        let user_id = test_user_id();
        let key = RateLimitKey::user_resource(&user_id, "ai_completions");

        // Free tier has 5 AI completions/min
        let status = limiter.status(key.clone()).await.unwrap();
        assert_eq!(status.limit, 5);
    }

    // ─── Different Keys Are Independent ───────────────────────────────

    #[tokio::test]
    async fn different_ips_have_independent_limits() {
        let mut config = RateLimitConfig::default();
        config.per_ip.requests_per_minute = 3;
        let limiter = InMemoryRateLimiter::new(config);

        let key1 = RateLimitKey::ip("1.1.1.1");
        let key2 = RateLimitKey::ip("2.2.2.2");

        // Exhaust limit for key1
        for _ in 0..3 {
            limiter.check(key1.clone()).await.unwrap();
        }
        let result = limiter.check(key1.clone()).await.unwrap();
        assert!(result.is_denied());

        // key2 should still have its full limit
        let result = limiter.check(key2.clone()).await.unwrap();
        assert!(result.is_allowed());
    }

    // ─── TierAwareRateLimiter Tests ───────────────────────────────────

    #[tokio::test]
    async fn tier_aware_limiter_defaults_to_free() {
        let limiter = TierAwareRateLimiter::new(RateLimitConfig::default());
        let tier = limiter.get_user_tier("unknown-user").await;
        assert_eq!(tier, MembershipTier::Free);
    }

    #[tokio::test]
    async fn tier_aware_limiter_uses_set_tier() {
        let limiter = TierAwareRateLimiter::new(RateLimitConfig::default());
        limiter
            .set_user_tier("premium-user", MembershipTier::Annual)
            .await;

        let tier = limiter.get_user_tier("premium-user").await;
        assert_eq!(tier, MembershipTier::Annual);
    }

    // ─── Remaining Counter Accuracy Tests ────────────────────────────

    #[tokio::test]
    async fn remaining_decrements_correctly() {
        let mut config = RateLimitConfig::default();
        config.per_ip.requests_per_minute = 10;
        let limiter = InMemoryRateLimiter::new(config);
        let key = RateLimitKey::ip("test-ip");

        for expected_remaining in (0..10).rev() {
            let result = limiter.check(key.clone()).await.unwrap();
            if let RateLimitResult::Allowed(status) = result {
                assert_eq!(
                    status.remaining, expected_remaining as u32,
                    "After request, remaining should be {}",
                    expected_remaining
                );
            }
        }
    }
}
