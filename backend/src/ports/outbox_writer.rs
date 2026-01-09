//! OutboxWriter port - Interface for transactional event persistence.
//!
//! This port implements the Transactional Outbox Pattern, which ensures
//! domain events are persisted in the same transaction as domain changes,
//! guaranteeing no events are lost even if the application crashes.
//!
//! ## Pattern Overview
//!
//! 1. Command handler saves aggregate AND events in same DB transaction
//! 2. OutboxRelay (background service) reads pending events
//! 3. OutboxRelay publishes to Redis and marks events as processed
//! 4. Handlers receive events through EventSubscriber
//!
//! See `docs/architecture/SCALING-READINESS.md` for full details.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::foundation::{DomainError, EventEnvelope};

/// Status of an outbox entry in the delivery pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxStatus {
    /// Event written but not yet published
    Pending,
    /// Event successfully published to message broker
    Published,
    /// Event failed to publish (will be retried)
    Failed,
}

/// An entry in the event outbox table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEntry {
    /// Unique identifier for this outbox entry
    pub id: Uuid,

    /// The domain event envelope
    pub event: EventEnvelope,

    /// Current delivery status
    pub status: OutboxStatus,

    /// When the event was written to the outbox
    pub created_at: DateTime<Utc>,

    /// When the event was last processed (published or failed)
    pub processed_at: Option<DateTime<Utc>>,

    /// Number of publish attempts
    pub attempts: u32,

    /// Last error message if failed
    pub last_error: Option<String>,

    /// Partition key for future sharding (typically aggregate owner user_id)
    pub partition_key: String,
}

impl OutboxEntry {
    /// Create a new pending outbox entry for an event.
    pub fn new(event: EventEnvelope, partition_key: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            event,
            status: OutboxStatus::Pending,
            created_at: Utc::now(),
            processed_at: None,
            attempts: 0,
            last_error: None,
            partition_key: partition_key.into(),
        }
    }

    /// Mark the entry as successfully published.
    pub fn mark_published(&mut self) {
        self.status = OutboxStatus::Published;
        self.processed_at = Some(Utc::now());
        self.attempts += 1;
    }

    /// Mark the entry as failed with an error.
    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.status = OutboxStatus::Failed;
        self.processed_at = Some(Utc::now());
        self.attempts += 1;
        self.last_error = Some(error.into());
    }
}

/// Port for writing events to the transactional outbox.
///
/// Implementations should:
/// - Be called within the same database transaction as domain changes
/// - Support batch writes for efficiency
/// - Set appropriate partition keys for future sharding
///
/// # Example
///
/// ```ignore
/// // In a command handler:
/// async fn handle(&self, cmd: CreateSession) -> Result<SessionId, CommandError> {
///     let mut txn = self.pool.begin().await?;
///
///     // Save aggregate
///     let mut session = Session::new(cmd.user_id);
///     self.session_repo.save_in_txn(&session, &mut txn).await?;
///
///     // Write events to outbox (same transaction!)
///     let events = session.pull_domain_events();
///     self.outbox.write_in_txn(&events, cmd.user_id.to_string(), &mut txn).await?;
///
///     txn.commit().await?;
///     Ok(session.id())
/// }
/// ```
#[async_trait]
pub trait OutboxWriter: Send + Sync {
    /// Write a single event to the outbox.
    ///
    /// Should be called within an existing database transaction.
    async fn write(
        &self,
        event: &EventEnvelope,
        partition_key: &str,
    ) -> Result<OutboxEntry, DomainError>;

    /// Write multiple events to the outbox in a batch.
    ///
    /// All events will be written atomically.
    async fn write_batch(
        &self,
        events: &[EventEnvelope],
        partition_key: &str,
    ) -> Result<Vec<OutboxEntry>, DomainError>;

    /// Get pending events for processing (used by OutboxRelay).
    ///
    /// Returns events ordered by creation time.
    /// Limit controls batch size for processing.
    async fn get_pending(&self, limit: u32) -> Result<Vec<OutboxEntry>, DomainError>;

    /// Mark an event as successfully published.
    async fn mark_published(&self, id: Uuid) -> Result<(), DomainError>;

    /// Mark an event as failed.
    async fn mark_failed(&self, id: Uuid, error: &str) -> Result<(), DomainError>;

    /// Clean up old published events (retention policy).
    ///
    /// Deletes events that were published more than `older_than_hours` ago.
    async fn cleanup_old(&self, older_than_hours: u32) -> Result<u64, DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outbox_entry_marks_published() {
        let event = EventEnvelope::test_fixture();
        let mut entry = OutboxEntry::new(event, "user-123");

        assert_eq!(entry.status, OutboxStatus::Pending);
        assert_eq!(entry.attempts, 0);

        entry.mark_published();

        assert_eq!(entry.status, OutboxStatus::Published);
        assert_eq!(entry.attempts, 1);
        assert!(entry.processed_at.is_some());
    }

    #[test]
    fn outbox_entry_marks_failed() {
        let event = EventEnvelope::test_fixture();
        let mut entry = OutboxEntry::new(event, "user-123");

        entry.mark_failed("Connection timeout");

        assert_eq!(entry.status, OutboxStatus::Failed);
        assert_eq!(entry.attempts, 1);
        assert_eq!(entry.last_error, Some("Connection timeout".to_string()));
    }
}
