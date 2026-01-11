//! Event bridge connecting domain events to WebSocket clients.
//!
//! Subscribes to dashboard-relevant domain events and broadcasts them
//! to connected clients in the appropriate session rooms.
//!
//! # Event Flow
//!
//! ```text
//! Domain Event Published
//!          │
//!          ▼
//! ┌────────────────────┐
//! │ WebSocketEventBridge│
//! │  receives event    │
//! └────────────────────┘
//!          │
//!          ▼
//! ┌────────────────────┐
//! │  Transform to      │
//! │  DashboardUpdate   │
//! └────────────────────┘
//!          │
//!          ▼
//! ┌────────────────────┐
//! │  Resolve session   │
//! │  from aggregate_id │
//! └────────────────────┘
//!          │
//!          ▼
//! ┌────────────────────┐
//! │  Broadcast to all  │
//! │  clients in room   │
//! └────────────────────┘
//! ```

use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::foundation::{DomainError, EventEnvelope, SessionId};
use crate::ports::{EventHandler, EventSubscriber};

use super::messages::{DashboardUpdate, DashboardUpdateType};
use super::rooms::RoomManager;

/// Event types that are relevant for dashboard updates.
///
/// These are the domain events that connected clients should know about.
pub const DASHBOARD_EVENT_TYPES: &[&str] = &[
    "session.created",
    "session.renamed",
    "cycle.created",
    "cycle.branched",
    "component.started",
    "component.completed",
    "component.output_updated",
    "message.sent",
    "pugh_scores.computed",
    "dq_scores.computed",
    "cycle.completed",
];

/// Bridge between the event bus and WebSocket connections.
///
/// Implements `EventHandler` to receive domain events and broadcast
/// them to connected clients in the appropriate session rooms.
pub struct WebSocketEventBridge {
    room_manager: Arc<RoomManager>,
}

impl WebSocketEventBridge {
    /// Create a new event bridge with the given room manager.
    pub fn new(room_manager: Arc<RoomManager>) -> Self {
        Self { room_manager }
    }

    /// Create as an Arc (for sharing with event subscriber).
    pub fn new_shared(room_manager: Arc<RoomManager>) -> Arc<Self> {
        Arc::new(Self::new(room_manager))
    }

    /// Register this bridge with an event subscriber.
    ///
    /// Subscribes to all dashboard-relevant event types.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let bridge = WebSocketEventBridge::new_shared(room_manager);
    /// bridge.register(&event_bus);
    /// ```
    pub fn register(self: &Arc<Self>, subscriber: &impl EventSubscriber) {
        subscriber.subscribe_all(DASHBOARD_EVENT_TYPES, self.clone());
    }

    /// Transform a domain event envelope into a dashboard update.
    ///
    /// Returns `None` if the event type is not relevant for dashboard updates.
    fn transform(&self, event: &EventEnvelope) -> Option<DashboardUpdate> {
        let update_type = match event.event_type.as_str() {
            "session.created" | "session.renamed" => DashboardUpdateType::SessionMetadata,
            "cycle.created" | "cycle.branched" => DashboardUpdateType::CycleCreated,
            "component.started" => DashboardUpdateType::ComponentStarted,
            "component.completed" => DashboardUpdateType::ComponentCompleted,
            "component.output_updated" => DashboardUpdateType::ComponentOutput,
            "message.sent" => DashboardUpdateType::ConversationMessage,
            "pugh_scores.computed" | "dq_scores.computed" => DashboardUpdateType::AnalysisScores,
            "cycle.completed" => DashboardUpdateType::CycleCompleted,
            _ => return None,
        };

        Some(DashboardUpdate {
            update_type,
            data: event.payload.clone(),
            timestamp: event.occurred_at,
            correlation_id: event.metadata.correlation_id.clone(),
        })
    }

    /// Resolve the session ID from an event envelope.
    ///
    /// For session events, uses the aggregate_id directly.
    /// For cycle/component events, extracts session_id from the payload.
    fn resolve_session_id(&self, event: &EventEnvelope) -> Option<SessionId> {
        // Session events have session_id as the aggregate_id
        if event.aggregate_type == "Session" {
            return event.aggregate_id.parse().ok();
        }

        // Cycle and component events should include session_id in payload
        if event.aggregate_type == "Cycle" || event.aggregate_type == "Component" {
            if let Some(session_id) = event.payload.get("session_id") {
                return session_id.as_str().and_then(|s| s.parse().ok());
            }
        }

        // Conversation events may also include session_id
        if let Some(session_id) = event.payload.get("session_id") {
            return session_id.as_str().and_then(|s| s.parse().ok());
        }

        None
    }
}

#[async_trait]
impl EventHandler for WebSocketEventBridge {
    async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError> {
        // Transform to dashboard update
        let Some(update) = self.transform(&event) else {
            return Ok(()); // Event not relevant for dashboard
        };

        // Resolve session for room routing
        let Some(session_id) = self.resolve_session_id(&event) else {
            tracing::debug!(
                event_type = %event.event_type,
                aggregate_type = %event.aggregate_type,
                aggregate_id = %event.aggregate_id,
                "Cannot resolve session ID for event, skipping WebSocket broadcast"
            );
            return Ok(()); // Can't route without session
        };

        // Broadcast to session room
        self.room_manager
            .broadcast_to_session(&session_id, update)
            .await;

        Ok(())
    }

    fn name(&self) -> &'static str {
        "WebSocketEventBridge"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{EventId, EventMetadata, Timestamp};
    use serde_json::json;

    fn session_event(event_type: &str, session_id: &str) -> EventEnvelope {
        EventEnvelope {
            event_id: EventId::new(),
            event_type: event_type.to_string(),
            aggregate_id: session_id.to_string(),
            aggregate_type: "Session".to_string(),
            occurred_at: Timestamp::now(),
            payload: json!({
                "session_id": session_id,
                "title": "Test Session"
            }),
            metadata: EventMetadata::default(),
        }
    }

    fn cycle_event(event_type: &str, cycle_id: &str, session_id: &str) -> EventEnvelope {
        EventEnvelope {
            event_id: EventId::new(),
            event_type: event_type.to_string(),
            aggregate_id: cycle_id.to_string(),
            aggregate_type: "Cycle".to_string(),
            occurred_at: Timestamp::now(),
            payload: json!({
                "cycle_id": cycle_id,
                "session_id": session_id
            }),
            metadata: EventMetadata::default(),
        }
    }

    fn test_session_id() -> SessionId {
        "550e8400-e29b-41d4-a716-446655440000".parse().unwrap()
    }

    #[test]
    fn transform_session_created_to_metadata_update() {
        let room_manager = Arc::new(RoomManager::default());
        let bridge = WebSocketEventBridge::new(room_manager);

        let event = session_event("session.created", &test_session_id().to_string());
        let update = bridge.transform(&event);

        assert!(update.is_some());
        let update = update.unwrap();
        assert_eq!(update.update_type, DashboardUpdateType::SessionMetadata);
    }

    #[test]
    fn transform_cycle_created_to_cycle_update() {
        let room_manager = Arc::new(RoomManager::default());
        let bridge = WebSocketEventBridge::new(room_manager);

        let event = cycle_event(
            "cycle.created",
            "cycle-123",
            &test_session_id().to_string(),
        );
        let update = bridge.transform(&event);

        assert!(update.is_some());
        let update = update.unwrap();
        assert_eq!(update.update_type, DashboardUpdateType::CycleCreated);
    }

    #[test]
    fn transform_component_completed_to_component_update() {
        let room_manager = Arc::new(RoomManager::default());
        let bridge = WebSocketEventBridge::new(room_manager);

        let event = EventEnvelope {
            event_id: EventId::new(),
            event_type: "component.completed".to_string(),
            aggregate_id: "component-123".to_string(),
            aggregate_type: "Component".to_string(),
            occurred_at: Timestamp::now(),
            payload: json!({
                "component_id": "component-123",
                "session_id": test_session_id().to_string(),
                "component_type": "objectives"
            }),
            metadata: EventMetadata::default(),
        };

        let update = bridge.transform(&event);

        assert!(update.is_some());
        let update = update.unwrap();
        assert_eq!(update.update_type, DashboardUpdateType::ComponentCompleted);
    }

    #[test]
    fn transform_unknown_event_returns_none() {
        let room_manager = Arc::new(RoomManager::default());
        let bridge = WebSocketEventBridge::new(room_manager);

        let event = EventEnvelope {
            event_id: EventId::new(),
            event_type: "unknown.event".to_string(),
            aggregate_id: "some-id".to_string(),
            aggregate_type: "Unknown".to_string(),
            occurred_at: Timestamp::now(),
            payload: json!({}),
            metadata: EventMetadata::default(),
        };

        let update = bridge.transform(&event);
        assert!(update.is_none());
    }

    #[test]
    fn resolve_session_id_from_session_event() {
        let room_manager = Arc::new(RoomManager::default());
        let bridge = WebSocketEventBridge::new(room_manager);

        let session_id = test_session_id();
        let event = session_event("session.created", &session_id.to_string());

        let resolved = bridge.resolve_session_id(&event);
        assert_eq!(resolved, Some(session_id));
    }

    #[test]
    fn resolve_session_id_from_cycle_event() {
        let room_manager = Arc::new(RoomManager::default());
        let bridge = WebSocketEventBridge::new(room_manager);

        let session_id = test_session_id();
        let event = cycle_event("cycle.created", "cycle-123", &session_id.to_string());

        let resolved = bridge.resolve_session_id(&event);
        assert_eq!(resolved, Some(session_id));
    }

    #[test]
    fn resolve_session_id_returns_none_for_missing_session() {
        let room_manager = Arc::new(RoomManager::default());
        let bridge = WebSocketEventBridge::new(room_manager);

        let event = EventEnvelope {
            event_id: EventId::new(),
            event_type: "cycle.created".to_string(),
            aggregate_id: "cycle-123".to_string(),
            aggregate_type: "Cycle".to_string(),
            occurred_at: Timestamp::now(),
            payload: json!({"cycle_id": "cycle-123"}), // No session_id
            metadata: EventMetadata::default(),
        };

        let resolved = bridge.resolve_session_id(&event);
        assert!(resolved.is_none());
    }

    #[tokio::test]
    async fn handle_broadcasts_to_correct_room() {
        let room_manager = Arc::new(RoomManager::default());
        let bridge = WebSocketEventBridge::new(room_manager.clone());

        let session_id = test_session_id();

        // Join a client to the session room
        let mut rx = room_manager.join(&session_id, super::super::ClientId::new()).await;

        // Handle an event
        let event = session_event("session.renamed", &session_id.to_string());
        bridge.handle(event).await.unwrap();

        // Should receive the update
        let received = rx.recv().await.unwrap();
        assert_eq!(received.update_type, DashboardUpdateType::SessionMetadata);
    }

    #[tokio::test]
    async fn handle_skips_irrelevant_events() {
        let room_manager = Arc::new(RoomManager::default());
        let bridge = WebSocketEventBridge::new(room_manager);

        let event = EventEnvelope {
            event_id: EventId::new(),
            event_type: "internal.cleanup".to_string(),
            aggregate_id: "some-id".to_string(),
            aggregate_type: "Internal".to_string(),
            occurred_at: Timestamp::now(),
            payload: json!({}),
            metadata: EventMetadata::default(),
        };

        // Should complete without error
        let result = bridge.handle(event).await;
        assert!(result.is_ok());
    }

    #[test]
    fn dashboard_event_types_includes_all_relevant_events() {
        let expected = [
            "session.created",
            "session.renamed",
            "cycle.created",
            "cycle.branched",
            "component.started",
            "component.completed",
            "component.output_updated",
            "message.sent",
            "pugh_scores.computed",
            "dq_scores.computed",
            "cycle.completed",
        ];

        for event_type in expected {
            assert!(
                DASHBOARD_EVENT_TYPES.contains(&event_type),
                "Missing event type: {}",
                event_type
            );
        }
    }

    #[test]
    fn new_shared_creates_arc() {
        let room_manager = Arc::new(RoomManager::default());
        let bridge = WebSocketEventBridge::new_shared(room_manager);

        // Should be usable as Arc
        let _clone = Arc::clone(&bridge);
    }
}
