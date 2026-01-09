//! UsageTracker port - Interface for tracking AI usage and costs.
//!
//! This port defines how AI token usage is tracked and queried,
//! enabling cost attribution per user, session, and daily limits.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::foundation::{ComponentType, SessionId, Timestamp, UserId};

/// Record of AI usage for a single request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    /// User who made the request.
    pub user_id: UserId,
    /// Session context.
    pub session_id: SessionId,
    /// AI provider used.
    pub provider: String,
    /// Model used.
    pub model: String,
    /// Tokens in the prompt.
    pub prompt_tokens: u32,
    /// Tokens in the completion.
    pub completion_tokens: u32,
    /// Cost in cents.
    pub cost_cents: u32,
    /// Component type (for analytics).
    pub component_type: Option<ComponentType>,
    /// When the usage occurred.
    pub occurred_at: Timestamp,
}

impl UsageRecord {
    /// Creates a new usage record.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user_id: UserId,
        session_id: SessionId,
        provider: impl Into<String>,
        model: impl Into<String>,
        prompt_tokens: u32,
        completion_tokens: u32,
        cost_cents: u32,
        component_type: Option<ComponentType>,
    ) -> Self {
        Self {
            user_id,
            session_id,
            provider: provider.into(),
            model: model.into(),
            prompt_tokens,
            completion_tokens,
            cost_cents,
            component_type,
            occurred_at: Timestamp::now(),
        }
    }

    /// Total tokens used.
    pub fn total_tokens(&self) -> u32 {
        self.prompt_tokens + self.completion_tokens
    }
}

/// Summary of usage for a user.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageSummary {
    /// Total cost in cents.
    pub total_cost_cents: u32,
    /// Total tokens used.
    pub total_tokens: u32,
    /// Number of requests.
    pub request_count: u32,
    /// Breakdown by provider.
    pub by_provider: Vec<ProviderUsage>,
}

/// Usage breakdown by provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUsage {
    /// Provider name.
    pub provider: String,
    /// Cost in cents for this provider.
    pub cost_cents: u32,
    /// Tokens used with this provider.
    pub tokens: u32,
    /// Number of requests to this provider.
    pub requests: u32,
}

/// Status of usage relative to a limit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsageLimitStatus {
    /// Under the limit, no concerns.
    UnderLimit {
        /// Cents remaining before limit.
        remaining_cents: u32,
    },
    /// Approaching the limit (>= 80% used).
    Warning {
        /// Cents remaining before limit.
        remaining_cents: u32,
        /// Percentage of limit used.
        percent_used: u8,
    },
    /// At or over the limit.
    AtLimit,
}

impl UsageLimitStatus {
    /// Calculates limit status from current usage and limit.
    ///
    /// - Under 80% used: `UnderLimit`
    /// - 80-99% used: `Warning`
    /// - 100%+ used: `AtLimit`
    pub fn from_usage(current_cents: u32, limit_cents: u32) -> Self {
        if limit_cents == 0 {
            return Self::AtLimit;
        }

        if current_cents >= limit_cents {
            Self::AtLimit
        } else {
            let remaining = limit_cents - current_cents;
            let percent_used = ((current_cents as f64 / limit_cents as f64) * 100.0) as u8;

            if percent_used >= 80 {
                Self::Warning {
                    remaining_cents: remaining,
                    percent_used,
                }
            } else {
                Self::UnderLimit {
                    remaining_cents: remaining,
                }
            }
        }
    }

    /// Returns true if usage should be blocked.
    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::AtLimit)
    }

    /// Returns true if user should be warned.
    pub fn should_warn(&self) -> bool {
        matches!(self, Self::Warning { .. })
    }
}

/// Port for tracking AI usage and costs.
///
/// Implementations may store usage in PostgreSQL, Redis, or memory.
#[async_trait]
pub trait UsageTracker: Send + Sync {
    /// Records a usage event.
    async fn record_usage(&self, record: UsageRecord) -> Result<(), UsageTrackerError>;

    /// Gets total cost for a user today (UTC).
    async fn get_daily_cost(&self, user_id: &UserId) -> Result<u32, UsageTrackerError>;

    /// Gets total cost for a specific session.
    async fn get_session_cost(&self, session_id: SessionId) -> Result<u32, UsageTrackerError>;

    /// Gets usage summary for a user within a time range.
    async fn get_usage_summary(
        &self,
        user_id: &UserId,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<UsageSummary, UsageTrackerError>;

    /// Checks if user is within daily limit.
    async fn check_daily_limit(
        &self,
        user_id: &UserId,
        limit_cents: u32,
    ) -> Result<UsageLimitStatus, UsageTrackerError>;

    /// Checks if session is within its limit.
    async fn check_session_limit(
        &self,
        session_id: SessionId,
        limit_cents: u32,
    ) -> Result<UsageLimitStatus, UsageTrackerError>;
}

/// Errors from the usage tracker.
#[derive(Debug, thiserror::Error)]
pub enum UsageTrackerError {
    /// Database error.
    #[error("database error: {0}")]
    Database(String),

    /// User not found.
    #[error("user not found: {0}")]
    UserNotFound(String),

    /// Session not found.
    #[error("session not found: {0}")]
    SessionNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_record_calculates_total_tokens() {
        let record = UsageRecord::new(
            UserId::new("user-1").unwrap(),
            SessionId::new(),
            "openai",
            "gpt-4",
            100,
            50,
            15,
            None,
        );

        assert_eq!(record.total_tokens(), 150);
    }

    #[test]
    fn usage_limit_status_under_limit() {
        let status = UsageLimitStatus::from_usage(50, 100);
        assert!(matches!(status, UsageLimitStatus::UnderLimit { remaining_cents: 50 }));
        assert!(!status.is_blocked());
        assert!(!status.should_warn());
    }

    #[test]
    fn usage_limit_status_warning_at_80_percent() {
        let status = UsageLimitStatus::from_usage(80, 100);
        assert!(matches!(
            status,
            UsageLimitStatus::Warning { remaining_cents: 20, percent_used: 80 }
        ));
        assert!(!status.is_blocked());
        assert!(status.should_warn());
    }

    #[test]
    fn usage_limit_status_warning_at_95_percent() {
        let status = UsageLimitStatus::from_usage(95, 100);
        assert!(matches!(
            status,
            UsageLimitStatus::Warning { remaining_cents: 5, percent_used: 95 }
        ));
        assert!(!status.is_blocked());
        assert!(status.should_warn());
    }

    #[test]
    fn usage_limit_status_at_limit() {
        let status = UsageLimitStatus::from_usage(100, 100);
        assert!(matches!(status, UsageLimitStatus::AtLimit));
        assert!(status.is_blocked());
        assert!(!status.should_warn());
    }

    #[test]
    fn usage_limit_status_over_limit() {
        let status = UsageLimitStatus::from_usage(150, 100);
        assert!(matches!(status, UsageLimitStatus::AtLimit));
        assert!(status.is_blocked());
    }

    #[test]
    fn usage_limit_status_zero_limit_is_at_limit() {
        let status = UsageLimitStatus::from_usage(0, 0);
        assert!(matches!(status, UsageLimitStatus::AtLimit));
        assert!(status.is_blocked());
    }

    #[test]
    fn usage_summary_default_is_empty() {
        let summary = UsageSummary::default();
        assert_eq!(summary.total_cost_cents, 0);
        assert_eq!(summary.total_tokens, 0);
        assert_eq!(summary.request_count, 0);
        assert!(summary.by_provider.is_empty());
    }
}
