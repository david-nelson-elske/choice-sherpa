//! CancelMembershipHandler - Command handler for cancelling memberships.

use std::sync::Arc;

use crate::domain::foundation::{EventId, SerializableDomainEvent, Timestamp, UserId};
use crate::domain::membership::{Membership, MembershipError, MembershipEvent};
use crate::ports::{EventPublisher, MembershipRepository};

/// Command to cancel a membership.
#[derive(Debug, Clone)]
pub struct CancelMembershipCommand {
    pub user_id: UserId,
}

/// Result of successful membership cancellation.
#[derive(Debug, Clone)]
pub struct CancelMembershipResult {
    pub membership: Membership,
    pub event: MembershipEvent,
    /// When access will end (period_end).
    pub effective_at: Timestamp,
}

/// Handler for cancelling memberships.
///
/// Cancellation takes effect at the end of the current billing period.
/// Users retain access until then.
pub struct CancelMembershipHandler {
    repository: Arc<dyn MembershipRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl CancelMembershipHandler {
    pub fn new(
        repository: Arc<dyn MembershipRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            repository,
            event_publisher,
        }
    }

    pub async fn handle(
        &self,
        cmd: CancelMembershipCommand,
    ) -> Result<CancelMembershipResult, MembershipError> {
        // 1. Find the user's membership
        let mut membership = self
            .repository
            .find_by_user_id(&cmd.user_id)
            .await?
            .ok_or_else(|| MembershipError::not_found_for_user(cmd.user_id.clone()))?;

        let effective_at = membership.current_period_end;

        // 2. Cancel the membership (domain logic)
        membership.cancel().map_err(|e| {
            MembershipError::invalid_state(
                format!("{:?}", membership.status),
                e.to_string(),
            )
        })?;

        // 3. Persist the update
        self.repository.update(&membership).await?;

        // 4. Create and publish event
        let event = MembershipEvent::Cancelled {
            event_id: EventId::new(),
            membership_id: membership.id,
            user_id: cmd.user_id,
            effective_at,
            occurred_at: Timestamp::now(),
        };

        let envelope = event.to_envelope();
        self.event_publisher.publish(envelope).await?;

        Ok(CancelMembershipResult {
            membership,
            event,
            effective_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{DomainError, ErrorCode, EventEnvelope, MembershipId};
    use crate::domain::membership::{MembershipStatus, MembershipTier};
    use async_trait::async_trait;
    use std::sync::Mutex;

    // ════════════════════════════════════════════════════════════════════════════
    // Mock Implementations
    // ════════════════════════════════════════════════════════════════════════════

    struct MockMembershipRepository {
        memberships: Mutex<Vec<Membership>>,
        fail_update: bool,
    }

    impl MockMembershipRepository {
        fn new() -> Self {
            Self {
                memberships: Mutex::new(Vec::new()),
                fail_update: false,
            }
        }

        fn with_membership(membership: Membership) -> Self {
            Self {
                memberships: Mutex::new(vec![membership]),
                fail_update: false,
            }
        }

        fn failing_update() -> Self {
            Self {
                memberships: Mutex::new(Vec::new()),
                fail_update: true,
            }
        }

        fn get_memberships(&self) -> Vec<Membership> {
            self.memberships.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl MembershipRepository for MockMembershipRepository {
        async fn save(&self, membership: &Membership) -> Result<(), DomainError> {
            self.memberships.lock().unwrap().push(membership.clone());
            Ok(())
        }

        async fn update(&self, membership: &Membership) -> Result<(), DomainError> {
            if self.fail_update {
                return Err(DomainError::new(
                    ErrorCode::DatabaseError,
                    "Simulated update failure",
                ));
            }
            let mut memberships = self.memberships.lock().unwrap();
            if let Some(m) = memberships.iter_mut().find(|m| m.id == membership.id) {
                *m = membership.clone();
            }
            Ok(())
        }

        async fn find_by_id(
            &self,
            id: &MembershipId,
        ) -> Result<Option<Membership>, DomainError> {
            let memberships = self.memberships.lock().unwrap();
            Ok(memberships.iter().find(|m| &m.id == id).cloned())
        }

        async fn find_by_user_id(
            &self,
            user_id: &UserId,
        ) -> Result<Option<Membership>, DomainError> {
            let memberships = self.memberships.lock().unwrap();
            Ok(memberships.iter().find(|m| &m.user_id == user_id).cloned())
        }

        async fn find_expiring_within_days(
            &self,
            _days: u32,
        ) -> Result<Vec<Membership>, DomainError> {
            Ok(vec![])
        }

        async fn delete(&self, _id: &MembershipId) -> Result<(), DomainError> {
            Ok(())
        }

        async fn find_by_stripe_subscription_id(
            &self,
            _subscription_id: &str,
        ) -> Result<Option<Membership>, DomainError> {
            Ok(None)
        }

        async fn find_by_stripe_customer_id(
            &self,
            _customer_id: &str,
        ) -> Result<Option<Membership>, DomainError> {
            Ok(None)
        }
    }

    struct MockEventPublisher {
        published_events: Mutex<Vec<EventEnvelope>>,
        fail_publish: bool,
    }

    impl MockEventPublisher {
        fn new() -> Self {
            Self {
                published_events: Mutex::new(Vec::new()),
                fail_publish: false,
            }
        }

        fn failing() -> Self {
            Self {
                published_events: Mutex::new(Vec::new()),
                fail_publish: true,
            }
        }

        fn published_events(&self) -> Vec<EventEnvelope> {
            self.published_events.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl EventPublisher for MockEventPublisher {
        async fn publish(&self, event: EventEnvelope) -> Result<(), DomainError> {
            if self.fail_publish {
                return Err(DomainError::new(
                    ErrorCode::InternalError,
                    "Simulated publish failure",
                ));
            }
            self.published_events.lock().unwrap().push(event);
            Ok(())
        }

        async fn publish_all(&self, events: Vec<EventEnvelope>) -> Result<(), DomainError> {
            for event in events {
                self.publish(event).await?;
            }
            Ok(())
        }
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Test Helpers
    // ════════════════════════════════════════════════════════════════════════════

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    fn active_membership(user_id: UserId) -> Membership {
        Membership::create_free(
            MembershipId::new(),
            user_id,
            MembershipTier::Annual,
            "WORKSHOP2026-A7K9M3".to_string(),
            Timestamp::now(),
            Timestamp::now().add_days(365),
        )
    }

    fn pending_membership(user_id: UserId) -> Membership {
        Membership::create_paid(
            MembershipId::new(),
            user_id,
            MembershipTier::Monthly,
            "cus_123".to_string(),
        )
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Success Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn cancels_active_membership() {
        let user_id = test_user_id();
        let membership = active_membership(user_id.clone());
        let repo = Arc::new(MockMembershipRepository::with_membership(membership));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CancelMembershipHandler::new(repo.clone(), publisher);

        let cmd = CancelMembershipCommand { user_id };

        let result = handler.handle(cmd).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.membership.status, MembershipStatus::Cancelled);
        assert!(result.membership.cancelled_at.is_some());
    }

    #[tokio::test]
    async fn publishes_cancelled_event() {
        let user_id = test_user_id();
        let membership = active_membership(user_id.clone());
        let repo = Arc::new(MockMembershipRepository::with_membership(membership));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CancelMembershipHandler::new(repo, publisher.clone());

        let cmd = CancelMembershipCommand { user_id };

        handler.handle(cmd).await.unwrap();

        let events = publisher.published_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "membership.cancelled");
    }

    #[tokio::test]
    async fn updates_membership_in_repository() {
        let user_id = test_user_id();
        let membership = active_membership(user_id.clone());
        let repo = Arc::new(MockMembershipRepository::with_membership(membership));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CancelMembershipHandler::new(repo.clone(), publisher);

        let cmd = CancelMembershipCommand { user_id };

        handler.handle(cmd).await.unwrap();

        let memberships = repo.get_memberships();
        assert_eq!(memberships.len(), 1);
        assert_eq!(memberships[0].status, MembershipStatus::Cancelled);
    }

    #[tokio::test]
    async fn returns_effective_date() {
        let user_id = test_user_id();
        let membership = active_membership(user_id.clone());
        let period_end = membership.current_period_end;
        let repo = Arc::new(MockMembershipRepository::with_membership(membership));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CancelMembershipHandler::new(repo, publisher);

        let cmd = CancelMembershipCommand { user_id };

        let result = handler.handle(cmd).await.unwrap();
        assert_eq!(result.effective_at, period_end);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Failure Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn fails_when_membership_not_found() {
        let repo = Arc::new(MockMembershipRepository::new());
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CancelMembershipHandler::new(repo, publisher.clone());

        let cmd = CancelMembershipCommand {
            user_id: test_user_id(),
        };

        let result = handler.handle(cmd).await;
        assert!(matches!(
            result,
            Err(MembershipError::NotFoundForUser(_))
        ));
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn fails_when_already_cancelled() {
        let user_id = test_user_id();
        let mut membership = active_membership(user_id.clone());
        membership.cancel().unwrap(); // Already cancelled
        let repo = Arc::new(MockMembershipRepository::with_membership(membership));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CancelMembershipHandler::new(repo, publisher.clone());

        let cmd = CancelMembershipCommand { user_id };

        let result = handler.handle(cmd).await;
        assert!(matches!(result, Err(MembershipError::InvalidState { .. })));
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn fails_when_pending() {
        let user_id = test_user_id();
        let membership = pending_membership(user_id.clone());
        let repo = Arc::new(MockMembershipRepository::with_membership(membership));
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CancelMembershipHandler::new(repo, publisher.clone());

        let cmd = CancelMembershipCommand { user_id };

        let result = handler.handle(cmd).await;
        assert!(matches!(result, Err(MembershipError::InvalidState { .. })));
    }

    #[tokio::test]
    async fn fails_when_repository_update_fails() {
        let user_id = test_user_id();
        let membership = active_membership(user_id.clone());
        let mut repo = MockMembershipRepository::failing_update();
        repo.memberships.lock().unwrap().push(membership);
        let repo = Arc::new(repo);
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CancelMembershipHandler::new(repo, publisher.clone());

        let cmd = CancelMembershipCommand { user_id };

        let result = handler.handle(cmd).await;
        assert!(result.is_err());
        assert!(publisher.published_events().is_empty());
    }

    #[tokio::test]
    async fn does_not_publish_on_update_failure() {
        let user_id = test_user_id();
        let membership = active_membership(user_id.clone());
        let mut repo = MockMembershipRepository::failing_update();
        repo.memberships.lock().unwrap().push(membership);
        let repo = Arc::new(repo);
        let publisher = Arc::new(MockEventPublisher::new());

        let handler = CancelMembershipHandler::new(repo, publisher.clone());

        let cmd = CancelMembershipCommand { user_id };

        let _ = handler.handle(cmd).await;
        assert!(publisher.published_events().is_empty());
    }
}
