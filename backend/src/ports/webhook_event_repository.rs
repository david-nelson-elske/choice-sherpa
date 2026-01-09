//! WebhookEventRepository port - Interface for tracking processed Stripe webhooks.
//!
//! This port enables idempotent webhook handling by tracking which webhook events
//! have been processed. Unlike the general ProcessedEventStore, this stores
//! the full webhook payload and result for debugging and auditing.
//!
//! ## Why Webhook Idempotency Matters
//!
//! Stripe may deliver the same webhook multiple times due to:
//! - Network timeouts
//! - 5xx response from our endpoint (triggers retry)
//! - Our endpoint returning success but Stripe not receiving it
//!
//! All webhook handlers MUST be idempotent.

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::foundation::DomainError;

/// Record of a processed webhook event.
#[derive(Debug, Clone)]
pub struct WebhookEventRecord {
    /// Stripe event ID (evt_xxx format).
    pub event_id: String,

    /// Type of Stripe event (e.g., "checkout.session.completed").
    pub event_type: String,

    /// When the event was processed.
    pub processed_at: DateTime<Utc>,

    /// Result of processing: "success", "ignored", or "failed".
    pub result: String,

    /// Error message if processing failed.
    pub error_message: Option<String>,

    /// Original event payload for debugging.
    pub payload: serde_json::Value,
}

impl WebhookEventRecord {
    /// Creates a new success record.
    pub fn success(
        event_id: impl Into<String>,
        event_type: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            event_type: event_type.into(),
            processed_at: Utc::now(),
            result: "success".to_string(),
            error_message: None,
            payload,
        }
    }

    /// Creates a new ignored record.
    pub fn ignored(
        event_id: impl Into<String>,
        event_type: impl Into<String>,
        reason: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            event_type: event_type.into(),
            processed_at: Utc::now(),
            result: "ignored".to_string(),
            error_message: Some(reason.into()),
            payload,
        }
    }

    /// Creates a new failure record.
    pub fn failed(
        event_id: impl Into<String>,
        event_type: impl Into<String>,
        error: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            event_type: event_type.into(),
            processed_at: Utc::now(),
            result: "failed".to_string(),
            error_message: Some(error.into()),
            payload,
        }
    }
}

/// Result of attempting to save a webhook event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveResult {
    /// Record was inserted (first time seeing this event).
    Inserted,
    /// Record already exists (duplicate event).
    AlreadyExists,
}

/// Port for storing and retrieving processed webhook events.
///
/// Implementations should use database constraints (PRIMARY KEY on event_id)
/// to prevent race conditions during concurrent webhook processing.
#[async_trait]
pub trait WebhookEventRepository: Send + Sync {
    /// Find a previously processed event by its Stripe event ID.
    ///
    /// Returns `None` if the event hasn't been processed yet.
    async fn find_by_event_id(
        &self,
        event_id: &str,
    ) -> Result<Option<WebhookEventRecord>, DomainError>;

    /// Attempt to save a webhook event record.
    ///
    /// Uses `ON CONFLICT DO NOTHING` semantics to handle race conditions.
    /// Returns `SaveResult::Inserted` if this is the first time seeing the event,
    /// or `SaveResult::AlreadyExists` if another process already inserted it.
    async fn save(&self, record: WebhookEventRecord) -> Result<SaveResult, DomainError>;

    /// Delete records older than the specified timestamp.
    ///
    /// Returns the number of records deleted.
    /// Used for cleanup/retention policy (e.g., keep 30 days).
    async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<u64, DomainError>;
}

/// Result of webhook processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebhookResult {
    /// Event was processed successfully.
    Processed,
    /// Event was already processed (idempotent skip).
    AlreadyProcessed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// In-memory implementation for testing.
    struct InMemoryWebhookEventRepository {
        records: Arc<RwLock<HashMap<String, WebhookEventRecord>>>,
    }

    impl InMemoryWebhookEventRepository {
        fn new() -> Self {
            Self {
                records: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl WebhookEventRepository for InMemoryWebhookEventRepository {
        async fn find_by_event_id(
            &self,
            event_id: &str,
        ) -> Result<Option<WebhookEventRecord>, DomainError> {
            let records = self.records.read().await;
            Ok(records.get(event_id).cloned())
        }

        async fn save(&self, record: WebhookEventRecord) -> Result<SaveResult, DomainError> {
            let mut records = self.records.write().await;
            if records.contains_key(&record.event_id) {
                Ok(SaveResult::AlreadyExists)
            } else {
                records.insert(record.event_id.clone(), record);
                Ok(SaveResult::Inserted)
            }
        }

        async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<u64, DomainError> {
            let mut records = self.records.write().await;
            let before_count = records.len();
            records.retain(|_, r| r.processed_at >= timestamp);
            let after_count = records.len();
            Ok((before_count - after_count) as u64)
        }
    }

    // ══════════════════════════════════════════════════════════════
    // WebhookEventRecord Tests
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn success_record_has_correct_fields() {
        let record = WebhookEventRecord::success(
            "evt_123",
            "checkout.session.completed",
            serde_json::json!({"id": "test"}),
        );

        assert_eq!(record.event_id, "evt_123");
        assert_eq!(record.event_type, "checkout.session.completed");
        assert_eq!(record.result, "success");
        assert!(record.error_message.is_none());
    }

    #[test]
    fn ignored_record_includes_reason() {
        let record = WebhookEventRecord::ignored(
            "evt_456",
            "invoice.paid",
            "Membership already active",
            serde_json::json!({}),
        );

        assert_eq!(record.result, "ignored");
        assert_eq!(
            record.error_message,
            Some("Membership already active".to_string())
        );
    }

    #[test]
    fn failed_record_includes_error() {
        let record = WebhookEventRecord::failed(
            "evt_789",
            "invoice.payment_failed",
            "Database connection failed",
            serde_json::json!({}),
        );

        assert_eq!(record.result, "failed");
        assert_eq!(
            record.error_message,
            Some("Database connection failed".to_string())
        );
    }

    // ══════════════════════════════════════════════════════════════
    // Repository Tests
    // ══════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn find_returns_none_for_new_event() {
        let repo = InMemoryWebhookEventRepository::new();

        let result = repo.find_by_event_id("evt_new").await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn find_returns_record_after_save() {
        let repo = InMemoryWebhookEventRepository::new();
        let record = WebhookEventRecord::success(
            "evt_saved",
            "checkout.session.completed",
            serde_json::json!({"test": true}),
        );

        repo.save(record.clone()).await.unwrap();
        let found = repo.find_by_event_id("evt_saved").await.unwrap();

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.event_id, "evt_saved");
        assert_eq!(found.result, "success");
    }

    #[tokio::test]
    async fn save_returns_inserted_for_new_event() {
        let repo = InMemoryWebhookEventRepository::new();
        let record = WebhookEventRecord::success("evt_new", "type", serde_json::json!({}));

        let result = repo.save(record).await.unwrap();

        assert_eq!(result, SaveResult::Inserted);
    }

    #[tokio::test]
    async fn save_returns_already_exists_for_duplicate() {
        let repo = InMemoryWebhookEventRepository::new();
        let record1 = WebhookEventRecord::success("evt_dup", "type", serde_json::json!({}));
        let record2 = WebhookEventRecord::success("evt_dup", "type", serde_json::json!({}));

        repo.save(record1).await.unwrap();
        let result = repo.save(record2).await.unwrap();

        assert_eq!(result, SaveResult::AlreadyExists);
    }

    #[tokio::test]
    async fn delete_before_removes_old_records() {
        let repo = InMemoryWebhookEventRepository::new();

        // Create records with different timestamps
        let old_record = WebhookEventRecord {
            event_id: "evt_old".to_string(),
            event_type: "type".to_string(),
            processed_at: Utc::now() - chrono::Duration::days(60),
            result: "success".to_string(),
            error_message: None,
            payload: serde_json::json!({}),
        };
        let new_record = WebhookEventRecord::success("evt_new", "type", serde_json::json!({}));

        repo.save(old_record).await.unwrap();
        repo.save(new_record).await.unwrap();

        // Delete records older than 30 days
        let cutoff = Utc::now() - chrono::Duration::days(30);
        let deleted = repo.delete_before(cutoff).await.unwrap();

        assert_eq!(deleted, 1);
        assert!(repo.find_by_event_id("evt_old").await.unwrap().is_none());
        assert!(repo.find_by_event_id("evt_new").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn different_events_stored_separately() {
        let repo = InMemoryWebhookEventRepository::new();
        let record1 = WebhookEventRecord::success("evt_1", "type_a", serde_json::json!({}));
        let record2 = WebhookEventRecord::failed("evt_2", "type_b", "error", serde_json::json!({}));

        repo.save(record1).await.unwrap();
        repo.save(record2).await.unwrap();

        let found1 = repo.find_by_event_id("evt_1").await.unwrap().unwrap();
        let found2 = repo.find_by_event_id("evt_2").await.unwrap().unwrap();

        assert_eq!(found1.result, "success");
        assert_eq!(found2.result, "failed");
    }
}
