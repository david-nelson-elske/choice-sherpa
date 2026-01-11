//! Rate limit configuration types.
//!
//! Defines the configuration for rate limiting across different scopes
//! and membership tiers.

use crate::domain::membership::MembershipTier;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete rate limit configuration.
///
/// Contains limits for global, per-IP, and per-tier rate limiting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Global rate limits (infrastructure protection).
    pub global: GlobalLimits,
    /// Per-IP rate limits (brute-force protection).
    pub per_ip: IpLimits,
    /// Per-tier rate limits (tier-based quotas).
    pub per_tier: HashMap<MembershipTier, TierRateLimits>,
    /// Per-resource rate limits (specific endpoint limits).
    pub resources: HashMap<String, ResourceLimits>,
}

/// Global rate limits for infrastructure protection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalLimits {
    /// Maximum requests per minute globally.
    pub requests_per_minute: u32,
}

/// Per-IP rate limits for brute-force protection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpLimits {
    /// Maximum requests per minute per IP.
    pub requests_per_minute: u32,
    /// Maximum authentication attempts per hour per IP.
    pub auth_attempts_per_hour: u32,
}

/// Rate limits for a specific membership tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierRateLimits {
    /// General API requests per minute.
    pub general_requests_per_minute: u32,
    /// Session CRUD operations per hour.
    pub session_requests_per_hour: u32,
    /// Conversation messages per minute.
    pub conversation_messages_per_minute: u32,
    /// AI completion requests per minute.
    pub ai_completions_per_minute: u32,
    /// AI tokens per day.
    pub ai_tokens_per_day: u32,
    /// Export operations per hour.
    pub exports_per_hour: u32,
    /// Maximum concurrent WebSocket connections.
    pub websocket_connections: u32,
}

/// Rate limits for a specific resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum requests per window.
    pub requests_per_window: u32,
    /// Window duration in seconds.
    pub window_secs: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        let mut per_tier = HashMap::new();
        per_tier.insert(MembershipTier::Free, TierRateLimits::free());
        per_tier.insert(MembershipTier::Monthly, TierRateLimits::monthly());
        per_tier.insert(MembershipTier::Annual, TierRateLimits::annual());

        Self {
            global: GlobalLimits {
                requests_per_minute: 10_000,
            },
            per_ip: IpLimits {
                requests_per_minute: 100,
                auth_attempts_per_hour: 10,
            },
            per_tier,
            resources: HashMap::new(),
        }
    }
}

impl TierRateLimits {
    /// Returns rate limits for the Free tier.
    pub fn free() -> Self {
        Self {
            general_requests_per_minute: 60,
            session_requests_per_hour: 30,
            conversation_messages_per_minute: 10,
            ai_completions_per_minute: 5,
            ai_tokens_per_day: 10_000,
            exports_per_hour: 0,
            websocket_connections: 1,
        }
    }

    /// Returns rate limits for the Monthly tier.
    pub fn monthly() -> Self {
        Self {
            general_requests_per_minute: 300,
            session_requests_per_hour: 100,
            conversation_messages_per_minute: 30,
            ai_completions_per_minute: 15,
            ai_tokens_per_day: 100_000,
            exports_per_hour: 10,
            websocket_connections: 3,
        }
    }

    /// Returns rate limits for the Annual tier.
    pub fn annual() -> Self {
        Self {
            general_requests_per_minute: 600,
            session_requests_per_hour: 300,
            conversation_messages_per_minute: 60,
            ai_completions_per_minute: 30,
            ai_tokens_per_day: 500_000,
            exports_per_hour: 50,
            websocket_connections: 10,
        }
    }

    /// Get the limit and window for a specific resource.
    ///
    /// Returns (limit, window_secs) tuple.
    pub fn limit_for_resource(&self, resource: Option<&str>) -> (u32, u32) {
        match resource {
            Some("ai_completions") => (self.ai_completions_per_minute, 60),
            Some("ai_tokens") => (self.ai_tokens_per_day, 86400),
            Some("conversation") => (self.conversation_messages_per_minute, 60),
            Some("session") => (self.session_requests_per_hour, 3600),
            Some("export") => (self.exports_per_hour, 3600),
            _ => (self.general_requests_per_minute, 60),
        }
    }
}

impl RateLimitConfig {
    /// Get the limits for a specific tier.
    ///
    /// Falls back to Free tier if tier not found.
    pub fn limits_for_tier(&self, tier: MembershipTier) -> &TierRateLimits {
        self.per_tier
            .get(&tier)
            .or_else(|| self.per_tier.get(&MembershipTier::Free))
            .expect("Free tier should always exist")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_all_tiers() {
        let config = RateLimitConfig::default();
        assert!(config.per_tier.contains_key(&MembershipTier::Free));
        assert!(config.per_tier.contains_key(&MembershipTier::Monthly));
        assert!(config.per_tier.contains_key(&MembershipTier::Annual));
    }

    #[test]
    fn default_global_limit_is_10000() {
        let config = RateLimitConfig::default();
        assert_eq!(config.global.requests_per_minute, 10_000);
    }

    #[test]
    fn default_ip_limit_is_100() {
        let config = RateLimitConfig::default();
        assert_eq!(config.per_ip.requests_per_minute, 100);
    }

    #[test]
    fn free_tier_has_lower_limits() {
        let free = TierRateLimits::free();
        let monthly = TierRateLimits::monthly();
        assert!(free.general_requests_per_minute < monthly.general_requests_per_minute);
        assert!(free.ai_completions_per_minute < monthly.ai_completions_per_minute);
    }

    #[test]
    fn annual_tier_has_highest_limits() {
        let monthly = TierRateLimits::monthly();
        let annual = TierRateLimits::annual();
        assert!(annual.general_requests_per_minute > monthly.general_requests_per_minute);
        assert!(annual.ai_tokens_per_day > monthly.ai_tokens_per_day);
    }

    #[test]
    fn free_tier_has_zero_exports() {
        let free = TierRateLimits::free();
        assert_eq!(free.exports_per_hour, 0);
    }

    #[test]
    fn limit_for_resource_returns_ai_limits() {
        let limits = TierRateLimits::free();
        let (limit, window) = limits.limit_for_resource(Some("ai_completions"));
        assert_eq!(limit, 5);
        assert_eq!(window, 60);
    }

    #[test]
    fn limit_for_resource_returns_general_for_unknown() {
        let limits = TierRateLimits::free();
        let (limit, window) = limits.limit_for_resource(Some("unknown"));
        assert_eq!(limit, 60);
        assert_eq!(window, 60);
    }

    #[test]
    fn limit_for_resource_returns_general_for_none() {
        let limits = TierRateLimits::free();
        let (limit, window) = limits.limit_for_resource(None);
        assert_eq!(limit, 60);
        assert_eq!(window, 60);
    }

    #[test]
    fn config_limits_for_tier_returns_correct_tier() {
        let config = RateLimitConfig::default();
        let monthly = config.limits_for_tier(MembershipTier::Monthly);
        assert_eq!(monthly.general_requests_per_minute, 300);
    }

    #[test]
    fn tier_rate_limits_serializes_to_json() {
        let limits = TierRateLimits::free();
        let json = serde_json::to_string(&limits).unwrap();
        assert!(json.contains("\"general_requests_per_minute\":60"));
    }
}
