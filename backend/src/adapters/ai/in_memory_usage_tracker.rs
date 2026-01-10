//! In-memory usage tracker implementation.
//!
//! This adapter provides an in-memory implementation of the `UsageTracker` port.
//! Useful for:
//! - Development and testing environments
//! - Single-server deployments without persistence requirements
//! - Demonstration and prototyping
//!
//! For production deployments requiring persistence, use a PostgreSQL-backed
//! implementation instead.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::domain::foundation::{SessionId, Timestamp, UserId};
use crate::ports::{
    ProviderUsage, UsageLimitStatus, UsageRecord, UsageSummary, UsageTracker, UsageTrackerError,
};

/// In-memory implementation of the UsageTracker port.
///
/// Thread-safe via internal `Mutex`. Suitable for single-server deployments
/// or testing. Does not persist data across restarts.
///
/// # Example
///
/// ```ignore
/// let tracker = InMemoryUsageTracker::new();
///
/// // Record usage
/// tracker.record_usage(UsageRecord::new(
///     user_id,
///     session_id,
///     "openai",
///     "gpt-4",
///     100,
///     50,
///     15,
///     None,
/// )).await?;
///
/// // Check limits
/// let status = tracker.check_daily_limit(&user_id, 100).await?;
/// if status.is_blocked() {
///     // User has exceeded daily limit
/// }
/// ```
#[derive(Default)]
pub struct InMemoryUsageTracker {
    records: Mutex<Vec<UsageRecord>>,
}

impl InMemoryUsageTracker {
    /// Creates a new empty usage tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns all recorded usage records.
    ///
    /// Useful for testing and debugging.
    pub fn records(&self) -> Vec<UsageRecord> {
        self.records.lock().unwrap().clone()
    }

    /// Clears all recorded usage.
    ///
    /// Useful for testing scenarios that need a clean slate.
    pub fn clear(&self) {
        self.records.lock().unwrap().clear();
    }

    /// Returns the total number of records.
    pub fn len(&self) -> usize {
        self.records.lock().unwrap().len()
    }

    /// Returns true if no records exist.
    pub fn is_empty(&self) -> bool {
        self.records.lock().unwrap().is_empty()
    }
}

#[async_trait]
impl UsageTracker for InMemoryUsageTracker {
    async fn record_usage(&self, record: UsageRecord) -> Result<(), UsageTrackerError> {
        self.records.lock().unwrap().push(record);
        Ok(())
    }

    async fn get_daily_cost(&self, user_id: &UserId) -> Result<u32, UsageTrackerError> {
        let records = self.records.lock().unwrap();
        let today_start = Timestamp::start_of_today();

        let total = records
            .iter()
            .filter(|r| &r.user_id == user_id && r.occurred_at >= today_start)
            .map(|r| r.cost_cents)
            .sum();
        Ok(total)
    }

    async fn get_session_cost(&self, session_id: SessionId) -> Result<u32, UsageTrackerError> {
        let records = self.records.lock().unwrap();
        let total = records
            .iter()
            .filter(|r| r.session_id == session_id)
            .map(|r| r.cost_cents)
            .sum();
        Ok(total)
    }

    async fn get_usage_summary(
        &self,
        user_id: &UserId,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<UsageSummary, UsageTrackerError> {
        let records = self.records.lock().unwrap();
        let user_records: Vec<_> = records
            .iter()
            .filter(|r| &r.user_id == user_id && r.occurred_at >= from && r.occurred_at <= to)
            .collect();

        let mut by_provider: HashMap<String, (u32, u32, u32)> = HashMap::new();
        for record in &user_records {
            let entry = by_provider
                .entry(record.provider.clone())
                .or_insert((0, 0, 0));
            entry.0 += record.cost_cents;
            entry.1 += record.total_tokens();
            entry.2 += 1;
        }

        Ok(UsageSummary {
            total_cost_cents: user_records.iter().map(|r| r.cost_cents).sum(),
            total_tokens: user_records.iter().map(|r| r.total_tokens()).sum(),
            request_count: user_records.len() as u32,
            by_provider: by_provider
                .into_iter()
                .map(|(provider, (cost, tokens, requests))| ProviderUsage {
                    provider,
                    cost_cents: cost,
                    tokens,
                    requests,
                })
                .collect(),
        })
    }

    async fn check_daily_limit(
        &self,
        user_id: &UserId,
        limit_cents: u32,
    ) -> Result<UsageLimitStatus, UsageTrackerError> {
        let current = self.get_daily_cost(user_id).await?;
        Ok(UsageLimitStatus::from_usage(current, limit_cents))
    }

    async fn check_session_limit(
        &self,
        session_id: SessionId,
        limit_cents: u32,
    ) -> Result<UsageLimitStatus, UsageTrackerError> {
        let current = self.get_session_cost(session_id).await?;
        Ok(UsageLimitStatus::from_usage(current, limit_cents))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn records_and_retrieves_usage() {
        let tracker = InMemoryUsageTracker::new();
        let user_id = UserId::new("user-1").unwrap();
        let session_id = SessionId::new();

        let record = UsageRecord::new(
            user_id.clone(),
            session_id,
            "openai",
            "gpt-4",
            100,
            50,
            15,
            None,
        );

        tracker.record_usage(record).await.unwrap();

        assert_eq!(tracker.len(), 1);
        let records = tracker.records();
        assert_eq!(records[0].provider, "openai");
        assert_eq!(records[0].cost_cents, 15);
    }

    #[tokio::test]
    async fn calculates_daily_cost() {
        let tracker = InMemoryUsageTracker::new();
        let user_id = UserId::new("user-1").unwrap();
        let session_id = SessionId::new();

        // Add two records for same user
        tracker
            .record_usage(UsageRecord::new(
                user_id.clone(),
                session_id,
                "openai",
                "gpt-4",
                100,
                50,
                15,
                None,
            ))
            .await
            .unwrap();

        tracker
            .record_usage(UsageRecord::new(
                user_id.clone(),
                session_id,
                "openai",
                "gpt-4",
                200,
                100,
                30,
                None,
            ))
            .await
            .unwrap();

        let daily_cost = tracker.get_daily_cost(&user_id).await.unwrap();
        assert_eq!(daily_cost, 45); // 15 + 30
    }

    #[tokio::test]
    async fn calculates_session_cost() {
        let tracker = InMemoryUsageTracker::new();
        let user_id = UserId::new("user-1").unwrap();
        let session1 = SessionId::new();
        let session2 = SessionId::new();

        // Add records for different sessions
        tracker
            .record_usage(UsageRecord::new(
                user_id.clone(),
                session1,
                "openai",
                "gpt-4",
                100,
                50,
                15,
                None,
            ))
            .await
            .unwrap();

        tracker
            .record_usage(UsageRecord::new(
                user_id.clone(),
                session2,
                "openai",
                "gpt-4",
                200,
                100,
                30,
                None,
            ))
            .await
            .unwrap();

        let session1_cost = tracker.get_session_cost(session1).await.unwrap();
        let session2_cost = tracker.get_session_cost(session2).await.unwrap();

        assert_eq!(session1_cost, 15);
        assert_eq!(session2_cost, 30);
    }

    #[tokio::test]
    async fn checks_daily_limit_under() {
        let tracker = InMemoryUsageTracker::new();
        let user_id = UserId::new("user-1").unwrap();
        let session_id = SessionId::new();

        tracker
            .record_usage(UsageRecord::new(
                user_id.clone(),
                session_id,
                "openai",
                "gpt-4",
                100,
                50,
                50, // 50 cents used
                None,
            ))
            .await
            .unwrap();

        // Check against 100 cent limit
        let status = tracker.check_daily_limit(&user_id, 100).await.unwrap();
        assert!(!status.is_blocked());
        assert!(!status.should_warn());
    }

    #[tokio::test]
    async fn checks_daily_limit_warning() {
        let tracker = InMemoryUsageTracker::new();
        let user_id = UserId::new("user-1").unwrap();
        let session_id = SessionId::new();

        tracker
            .record_usage(UsageRecord::new(
                user_id.clone(),
                session_id,
                "openai",
                "gpt-4",
                100,
                50,
                85, // 85 cents used
                None,
            ))
            .await
            .unwrap();

        // Check against 100 cent limit (85% used)
        let status = tracker.check_daily_limit(&user_id, 100).await.unwrap();
        assert!(!status.is_blocked());
        assert!(status.should_warn());
    }

    #[tokio::test]
    async fn checks_daily_limit_blocked() {
        let tracker = InMemoryUsageTracker::new();
        let user_id = UserId::new("user-1").unwrap();
        let session_id = SessionId::new();

        tracker
            .record_usage(UsageRecord::new(
                user_id.clone(),
                session_id,
                "openai",
                "gpt-4",
                100,
                50,
                100, // 100 cents used
                None,
            ))
            .await
            .unwrap();

        // Check against 100 cent limit (100% used)
        let status = tracker.check_daily_limit(&user_id, 100).await.unwrap();
        assert!(status.is_blocked());
    }

    #[tokio::test]
    async fn clear_removes_all_records() {
        let tracker = InMemoryUsageTracker::new();
        let user_id = UserId::new("user-1").unwrap();
        let session_id = SessionId::new();

        tracker
            .record_usage(UsageRecord::new(
                user_id.clone(),
                session_id,
                "openai",
                "gpt-4",
                100,
                50,
                15,
                None,
            ))
            .await
            .unwrap();

        assert_eq!(tracker.len(), 1);

        tracker.clear();

        assert!(tracker.is_empty());
    }

    #[tokio::test]
    async fn usage_summary_groups_by_provider() {
        let tracker = InMemoryUsageTracker::new();
        let user_id = UserId::new("user-1").unwrap();
        let session_id = SessionId::new();

        // OpenAI usage
        tracker
            .record_usage(UsageRecord::new(
                user_id.clone(),
                session_id,
                "openai",
                "gpt-4",
                100,
                50,
                15,
                None,
            ))
            .await
            .unwrap();

        // Anthropic usage
        tracker
            .record_usage(UsageRecord::new(
                user_id.clone(),
                session_id,
                "anthropic",
                "claude-3-opus",
                200,
                100,
                30,
                None,
            ))
            .await
            .unwrap();

        let from = Timestamp::now().minus_days(1);
        let to = Timestamp::now().plus_days(1);
        let summary = tracker.get_usage_summary(&user_id, from, to).await.unwrap();

        assert_eq!(summary.total_cost_cents, 45);
        assert_eq!(summary.total_tokens, 450); // 150 + 300
        assert_eq!(summary.request_count, 2);
        assert_eq!(summary.by_provider.len(), 2);
    }
}
