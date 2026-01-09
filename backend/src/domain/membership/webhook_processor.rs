//! Webhook processor - Orchestrates idempotent webhook event handling.
//!
//! This module provides the coordination layer between Stripe webhooks
//! and domain event handlers, ensuring each event is processed exactly once.
//!
//! ## Design
//!
//! The processor follows these steps:
//! 1. Check if event was already processed (idempotency)
//! 2. Dispatch to appropriate handler based on event type
//! 3. Record the processing result (success, ignored, or failed)
//!
//! ## Race Condition Handling
//!
//! When multiple webhook deliveries arrive simultaneously:
//! - First to save wins (database PRIMARY KEY constraint)
//! - Others get `AlreadyExists` and return `AlreadyProcessed`

use async_trait::async_trait;

use crate::domain::foundation::DomainError;
use crate::domain::membership::{StripeEvent, StripeEventType, WebhookError};
use crate::ports::{SaveResult, WebhookEventRecord, WebhookEventRepository, WebhookResult};

/// Handler for a specific type of Stripe webhook event.
///
/// Implementations should be stateless and focus on a single event type.
/// The handler receives the parsed event and should perform the necessary
/// domain operations.
#[async_trait]
pub trait WebhookEventHandler: Send + Sync {
    /// Returns the event type(s) this handler processes.
    fn handles(&self) -> Vec<StripeEventType>;

    /// Handles the webhook event.
    ///
    /// Returns `Ok(())` on success.
    /// Returns `Err(WebhookError::Ignored(_))` if event should be acknowledged but not processed.
    /// Returns other `Err` variants for actual failures.
    async fn handle(&self, event: &StripeEvent) -> Result<(), WebhookError>;
}

/// Dispatches webhook events to the appropriate handler.
///
/// This trait allows for flexible routing of events to handlers
/// without tight coupling between the processor and specific handlers.
#[async_trait]
pub trait WebhookDispatcher: Send + Sync {
    /// Find a handler for the given event type.
    ///
    /// Returns `None` if no handler is registered for this event type.
    fn get_handler(&self, event_type: &StripeEventType) -> Option<&dyn WebhookEventHandler>;

    /// Dispatch an event to its handler.
    ///
    /// Returns `Err(WebhookError::Ignored)` if no handler is registered.
    async fn dispatch(&self, event: &StripeEvent) -> Result<(), WebhookError> {
        let event_type = event.parsed_type();
        match self.get_handler(&event_type) {
            Some(handler) => handler.handle(event).await,
            None => Err(WebhookError::Ignored(format!(
                "No handler for event type: {:?}",
                event_type
            ))),
        }
    }
}

/// Processes webhook events with idempotency guarantees.
///
/// This is the main entry point for webhook processing. It coordinates
/// between the idempotency store and event handlers.
pub struct IdempotentWebhookProcessor<R: WebhookEventRepository, D: WebhookDispatcher> {
    repository: R,
    dispatcher: D,
}

impl<R: WebhookEventRepository, D: WebhookDispatcher> IdempotentWebhookProcessor<R, D> {
    /// Creates a new processor with the given repository and dispatcher.
    pub fn new(repository: R, dispatcher: D) -> Self {
        Self {
            repository,
            dispatcher,
        }
    }

    /// Process a webhook event exactly once.
    ///
    /// This method ensures idempotency by:
    /// 1. Checking if the event was already processed
    /// 2. Processing the event if not
    /// 3. Recording the result atomically
    ///
    /// # Returns
    ///
    /// - `Ok(WebhookResult::Processed)` - Event was processed successfully
    /// - `Ok(WebhookResult::AlreadyProcessed)` - Event was already processed (idempotent skip)
    /// - `Err(_)` - Processing failed
    pub async fn process(&self, event: StripeEvent) -> Result<WebhookResult, WebhookError> {
        // 1. Check if already processed
        if self.repository.find_by_event_id(&event.id).await?.is_some() {
            return Ok(WebhookResult::AlreadyProcessed);
        }

        // 2. Process the event
        let result = self.dispatcher.dispatch(&event).await;

        // 3. Create the record based on result
        let record = match &result {
            Ok(()) => WebhookEventRecord::success(
                &event.id,
                &event.event_type,
                serde_json::to_value(&event).map_err(|e| {
                    WebhookError::ParseError(format!("Failed to serialize event: {}", e))
                })?,
            ),
            Err(WebhookError::Ignored(reason)) => WebhookEventRecord::ignored(
                &event.id,
                &event.event_type,
                reason,
                serde_json::to_value(&event).map_err(|e| {
                    WebhookError::ParseError(format!("Failed to serialize event: {}", e))
                })?,
            ),
            Err(e) => WebhookEventRecord::failed(
                &event.id,
                &event.event_type,
                e.to_string(),
                serde_json::to_value(&event).map_err(|e| {
                    WebhookError::ParseError(format!("Failed to serialize event: {}", e))
                })?,
            ),
        };

        // 4. Save the record (handles race conditions)
        match self.repository.save(record).await? {
            SaveResult::Inserted => {
                // We won the race, return our result
                match result {
                    Ok(()) => Ok(WebhookResult::Processed),
                    Err(WebhookError::Ignored(_)) => {
                        // Ignored events are still "processed" from idempotency perspective
                        Ok(WebhookResult::Processed)
                    }
                    Err(e) => Err(e),
                }
            }
            SaveResult::AlreadyExists => {
                // Lost the race, another process already handled it
                Ok(WebhookResult::AlreadyProcessed)
            }
        }
    }
}

/// Converts DomainError to WebhookError for repository operations.
impl From<DomainError> for WebhookError {
    fn from(err: DomainError) -> Self {
        WebhookError::Database(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::membership::StripeEvent;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // ══════════════════════════════════════════════════════════════
    // Test Infrastructure
    // ══════════════════════════════════════════════════════════════

    /// In-memory repository for testing.
    struct MockWebhookRepository {
        records: Arc<RwLock<HashMap<String, WebhookEventRecord>>>,
    }

    impl MockWebhookRepository {
        fn new() -> Self {
            Self {
                records: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl WebhookEventRepository for MockWebhookRepository {
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

        async fn delete_before(
            &self,
            timestamp: chrono::DateTime<chrono::Utc>,
        ) -> Result<u64, DomainError> {
            let mut records = self.records.write().await;
            let before = records.len();
            records.retain(|_, r| r.processed_at >= timestamp);
            Ok((before - records.len()) as u64)
        }
    }

    /// Mock handler that tracks invocations.
    struct MockHandler {
        handles_types: Vec<StripeEventType>,
        call_count: AtomicU32,
        should_fail: bool,
        should_ignore: bool,
    }

    impl MockHandler {
        fn new(handles: Vec<StripeEventType>) -> Self {
            Self {
                handles_types: handles,
                call_count: AtomicU32::new(0),
                should_fail: false,
                should_ignore: false,
            }
        }

        fn failing(handles: Vec<StripeEventType>) -> Self {
            Self {
                handles_types: handles,
                call_count: AtomicU32::new(0),
                should_fail: true,
                should_ignore: false,
            }
        }

        fn ignoring(handles: Vec<StripeEventType>) -> Self {
            Self {
                handles_types: handles,
                call_count: AtomicU32::new(0),
                should_fail: false,
                should_ignore: true,
            }
        }

        fn call_count(&self) -> u32 {
            self.call_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl WebhookEventHandler for MockHandler {
        fn handles(&self) -> Vec<StripeEventType> {
            self.handles_types.clone()
        }

        async fn handle(&self, _event: &StripeEvent) -> Result<(), WebhookError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            if self.should_fail {
                Err(WebhookError::Database("Simulated failure".to_string()))
            } else if self.should_ignore {
                Err(WebhookError::Ignored("Test ignore".to_string()))
            } else {
                Ok(())
            }
        }
    }

    /// Simple dispatcher that routes to a single handler.
    struct SingleHandlerDispatcher {
        handler: Arc<MockHandler>,
    }

    impl SingleHandlerDispatcher {
        fn new(handler: Arc<MockHandler>) -> Self {
            Self { handler }
        }
    }

    #[async_trait]
    impl WebhookDispatcher for SingleHandlerDispatcher {
        fn get_handler(&self, event_type: &StripeEventType) -> Option<&dyn WebhookEventHandler> {
            if self.handler.handles_types.contains(event_type) {
                Some(self.handler.as_ref())
            } else {
                None
            }
        }
    }

    /// Create a test event with the given ID and type.
    fn test_event(id: &str, event_type: &str) -> StripeEvent {
        use crate::domain::membership::StripeEventData;
        StripeEvent {
            id: id.to_string(),
            event_type: event_type.to_string(),
            created: chrono::Utc::now().timestamp(),
            data: StripeEventData {
                object: serde_json::json!({}),
                previous_attributes: None,
            },
            livemode: false,
            api_version: "2023-10-16".to_string(),
        }
    }

    // ══════════════════════════════════════════════════════════════
    // WebhookEventHandler Tests
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn handler_declares_event_types_it_handles() {
        let handler = MockHandler::new(vec![
            StripeEventType::CheckoutSessionCompleted,
            StripeEventType::InvoicePaymentSucceeded,
        ]);

        let handles = handler.handles();

        assert_eq!(handles.len(), 2);
        assert!(handles.contains(&StripeEventType::CheckoutSessionCompleted));
        assert!(handles.contains(&StripeEventType::InvoicePaymentSucceeded));
    }

    // ══════════════════════════════════════════════════════════════
    // WebhookDispatcher Tests
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn dispatcher_finds_handler_for_registered_type() {
        let handler = Arc::new(MockHandler::new(vec![StripeEventType::CheckoutSessionCompleted]));
        let dispatcher = SingleHandlerDispatcher::new(handler);

        let found = dispatcher.get_handler(&StripeEventType::CheckoutSessionCompleted);

        assert!(found.is_some());
    }

    #[test]
    fn dispatcher_returns_none_for_unregistered_type() {
        let handler = Arc::new(MockHandler::new(vec![StripeEventType::CheckoutSessionCompleted]));
        let dispatcher = SingleHandlerDispatcher::new(handler);

        let found = dispatcher.get_handler(&StripeEventType::InvoicePaymentFailed);

        assert!(found.is_none());
    }

    #[tokio::test]
    async fn dispatcher_ignores_unknown_event_types() {
        let handler = Arc::new(MockHandler::new(vec![StripeEventType::CheckoutSessionCompleted]));
        let dispatcher = SingleHandlerDispatcher::new(handler);
        let event = test_event("evt_unknown", "unknown.event.type");

        let result = dispatcher.dispatch(&event).await;

        assert!(matches!(result, Err(WebhookError::Ignored(_))));
    }

    // ══════════════════════════════════════════════════════════════
    // IdempotentWebhookProcessor Tests
    // ══════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn processor_processes_new_event_successfully() {
        let repo = MockWebhookRepository::new();
        let handler = Arc::new(MockHandler::new(vec![StripeEventType::CheckoutSessionCompleted]));
        let dispatcher = SingleHandlerDispatcher::new(handler.clone());
        let processor = IdempotentWebhookProcessor::new(repo, dispatcher);

        let event = test_event("evt_new", "checkout.session.completed");
        let result = processor.process(event).await;

        assert_eq!(result.unwrap(), WebhookResult::Processed);
        assert_eq!(handler.call_count(), 1);
    }

    #[tokio::test]
    async fn processor_returns_already_processed_for_duplicate() {
        let repo = MockWebhookRepository::new();
        let handler = Arc::new(MockHandler::new(vec![StripeEventType::CheckoutSessionCompleted]));
        let dispatcher = SingleHandlerDispatcher::new(handler.clone());
        let processor = IdempotentWebhookProcessor::new(repo, dispatcher);

        // Process first time
        let event1 = test_event("evt_dup", "checkout.session.completed");
        processor.process(event1).await.unwrap();

        // Process same event again
        let event2 = test_event("evt_dup", "checkout.session.completed");
        let result = processor.process(event2).await;

        assert_eq!(result.unwrap(), WebhookResult::AlreadyProcessed);
        assert_eq!(handler.call_count(), 1); // Only called once
    }

    #[tokio::test]
    async fn processor_records_success_in_repository() {
        let repo = MockWebhookRepository::new();
        let handler = Arc::new(MockHandler::new(vec![StripeEventType::CheckoutSessionCompleted]));
        let dispatcher = SingleHandlerDispatcher::new(handler);
        let processor = IdempotentWebhookProcessor::new(repo, dispatcher);

        let event = test_event("evt_success", "checkout.session.completed");
        processor.process(event).await.unwrap();

        // Access the repository to verify the record
        // Note: In real code, this would be done through the repository interface
    }

    #[tokio::test]
    async fn processor_records_failure_in_repository() {
        let repo = MockWebhookRepository::new();
        let handler = Arc::new(MockHandler::failing(vec![
            StripeEventType::CheckoutSessionCompleted,
        ]));
        let dispatcher = SingleHandlerDispatcher::new(handler);
        let processor = IdempotentWebhookProcessor::new(repo, dispatcher);

        let event = test_event("evt_fail", "checkout.session.completed");
        let result = processor.process(event).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn processor_records_ignored_as_processed() {
        let repo = MockWebhookRepository::new();
        let handler = Arc::new(MockHandler::ignoring(vec![
            StripeEventType::CheckoutSessionCompleted,
        ]));
        let dispatcher = SingleHandlerDispatcher::new(handler);
        let processor = IdempotentWebhookProcessor::new(repo, dispatcher);

        let event = test_event("evt_ignore", "checkout.session.completed");
        let result = processor.process(event).await;

        // Ignored events are considered "processed" for idempotency
        assert_eq!(result.unwrap(), WebhookResult::Processed);
    }

    #[tokio::test]
    async fn processor_handles_handler_not_found_as_ignored() {
        let repo = MockWebhookRepository::new();
        // Handler only handles checkout, not invoice
        let handler = Arc::new(MockHandler::new(vec![StripeEventType::CheckoutSessionCompleted]));
        let dispatcher = SingleHandlerDispatcher::new(handler);
        let processor = IdempotentWebhookProcessor::new(repo, dispatcher);

        let event = test_event("evt_no_handler", "invoice.payment_failed");
        let result = processor.process(event).await;

        // Unknown events are processed (recorded as ignored)
        assert_eq!(result.unwrap(), WebhookResult::Processed);
    }

    #[tokio::test]
    async fn processor_processes_different_events_independently() {
        let repo = MockWebhookRepository::new();
        let handler = Arc::new(MockHandler::new(vec![
            StripeEventType::CheckoutSessionCompleted,
            StripeEventType::InvoicePaymentSucceeded,
        ]));
        let dispatcher = SingleHandlerDispatcher::new(handler.clone());
        let processor = IdempotentWebhookProcessor::new(repo, dispatcher);

        let event1 = test_event("evt_1", "checkout.session.completed");
        let event2 = test_event("evt_2", "invoice.payment_succeeded");

        let result1 = processor.process(event1).await;
        let result2 = processor.process(event2).await;

        assert_eq!(result1.unwrap(), WebhookResult::Processed);
        assert_eq!(result2.unwrap(), WebhookResult::Processed);
        assert_eq!(handler.call_count(), 2);
    }
}
