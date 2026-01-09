# Feature: Dashboard Event Handlers

**Module:** dashboard
**Type:** Event Handling (Consumer)
**Priority:** P1
**Phase:** 6 of Full PrOACT Journey Integration
**Depends On:** All previous phases (1-5)

> Dashboard module subscribes to events from all other modules to maintain real-time read models without direct coupling.

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Required |
| Authorization Model | User can only access their own sessions; WebSocket scoped to session ownership |
| Sensitive Data | Aggregated session/cycle data (Confidential), message previews (Confidential) |
| Rate Limiting | Required - dashboard API requests, WebSocket connections per user |
| Audit Logging | Dashboard access, cache invalidation events |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| Session titles/descriptions | Confidential | User-created content |
| Cycle progress | Internal | Aggregate metrics, safe to log |
| Component status | Internal | Safe to log |
| DQ scores | Confidential | User self-assessment |
| Pugh scores summary | Confidential | Derived from user decisions |
| Message previews | Confidential | **MUST NOT be logged** |
| session_id, cycle_id | Internal | Safe to log |

### Security Events to Log

- Dashboard overview access - Log user_id, session_id, cache hit/miss
- Cache invalidation - Log event_type, affected entity IDs
- WebSocket connection established/closed - Log user_id, session_id
- Authorization failures - Log user_id, attempted session_id, reason

### Authorization Enforcement

1. **API Layer**: Verify user owns session before returning dashboard data
2. **WebSocket**: Verify session ownership on connection and for each forwarded event
3. **Cache Access**: Cache entries are keyed by session_id; user_id check at read time
4. **Event Forwarding**: Only forward events to WebSocket connections with verified ownership

### Cache Security

- Cache entries do not store user_id (authorization checked at access time)
- Event handlers must not trust session_id from events without verification
- Stale cache data must be re-authorized on access, not just on cache population

---

## Problem Statement

The dashboard needs to display data from multiple modules:
- Session metadata from session module
- Cycle progress from cycle module
- Component outputs from cycle module
- Conversation activity from conversation module
- Computed scores from analysis module

Without events, the dashboard would need to:
- Poll all modules constantly
- Have direct dependencies on all module internals
- Miss real-time updates

### Current State

- Dashboard queries each module directly
- No real-time updates
- Tight coupling to all modules

### Desired State

- Dashboard subscribes to events, not modules
- Real-time updates via WebSocket bridge
- Loose coupling - dashboard only knows event schemas
- Cached read models updated incrementally

---

## Event Subscriptions

The dashboard subscribes to events from all modules:

| Event | Source Module | Dashboard Action |
|-------|---------------|------------------|
| `session.created` | session | Initialize session in cache |
| `session.renamed` | session | Update session title |
| `session.archived` | session | Remove from active list |
| `session.cycle_added` | session | Increment cycle count |
| `cycle.created` | cycle | Add cycle to tree |
| `cycle.branched` | cycle | Add branch to tree |
| `component.started` | cycle | Update progress indicator |
| `component.completed` | cycle | Update progress, mark complete |
| `component.output_updated` | cycle | Refresh component panel |
| `message.sent` | conversation | Add to chat display |
| `conversation.started` | conversation | Show chat panel active |
| `analysis.pugh_scores_computed` | analysis | Update scores display |
| `analysis.dq_scores_computed` | analysis | Update DQ gauge |
| `analysis.tradeoffs_analyzed` | analysis | Update tradeoffs panel |
| `cycle.completed` | cycle | Mark cycle complete |

---

## Dashboard Update Handler

### Core Handler

```rust
// backend/src/application/handlers/dashboard_update.rs

use std::collections::HashSet;
use std::sync::RwLock;

/// Handles all dashboard-relevant events to maintain read models
pub struct DashboardUpdateHandler {
    dashboard_cache: Arc<DashboardCache>,
    websocket_bridge: Arc<WebSocketEventBridge>,
    /// Track processed event IDs for idempotency
    processed_events: RwLock<HashSet<EventId>>,
}

impl DashboardUpdateHandler {
    pub fn new(
        dashboard_cache: Arc<DashboardCache>,
        websocket_bridge: Arc<WebSocketEventBridge>,
    ) -> Self {
        Self {
            dashboard_cache,
            websocket_bridge,
            processed_events: RwLock::new(HashSet::new()),
        }
    }

    /// Check if event was already processed (idempotency)
    fn is_processed(&self, event_id: &EventId) -> bool {
        self.processed_events.read().unwrap().contains(event_id)
    }

    /// Mark event as processed
    fn mark_processed(&self, event_id: EventId) {
        self.processed_events.write().unwrap().insert(event_id);
    }

    /// Clean old processed events (memory management)
    pub fn cleanup_old_events(&self, older_than: Timestamp) {
        // In production, use timestamp-based cleanup
        // For simplicity, just clear if too large
        let mut processed = self.processed_events.write().unwrap();
        if processed.len() > 10000 {
            processed.clear();
        }
    }
}

#[async_trait]
impl EventHandler for DashboardUpdateHandler {
    async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError> {
        // Idempotency check
        if self.is_processed(&event.event_id) {
            return Ok(());
        }

        // Route to appropriate handler
        let result = match event.event_type.as_str() {
            // Session events
            "session.created" => self.handle_session_created(&event).await,
            "session.renamed" => self.handle_session_renamed(&event).await,
            "session.archived" => self.handle_session_archived(&event).await,
            "session.cycle_added" => self.handle_cycle_added(&event).await,

            // Cycle events
            "cycle.created" => self.handle_cycle_created(&event).await,
            "cycle.branched" => self.handle_cycle_branched(&event).await,
            "component.started" => self.handle_component_started(&event).await,
            "component.completed" => self.handle_component_completed(&event).await,
            "component.output_updated" => self.handle_component_output(&event).await,
            "cycle.completed" => self.handle_cycle_completed(&event).await,

            // Conversation events
            "message.sent" => self.handle_message_sent(&event).await,
            "conversation.started" => self.handle_conversation_started(&event).await,

            // Analysis events
            "analysis.pugh_scores_computed" => self.handle_pugh_scores(&event).await,
            "analysis.dq_scores_computed" => self.handle_dq_scores(&event).await,
            "analysis.tradeoffs_analyzed" => self.handle_tradeoffs(&event).await,

            // Unknown event type - ignore
            _ => Ok(()),
        };

        // Mark as processed regardless of success/failure
        // (failure should be logged but not retry with same event)
        self.mark_processed(event.event_id.clone());

        // Forward to WebSocket bridge
        if result.is_ok() {
            self.websocket_bridge.forward_event(&event).await;
        }

        result
    }

    fn name(&self) -> &'static str {
        "DashboardUpdateHandler"
    }
}
```

### Event Handler Implementations

```rust
impl DashboardUpdateHandler {
    // === Session Events ===

    async fn handle_session_created(&self, event: &EventEnvelope) -> Result<(), DomainError> {
        let payload: SessionCreated = event.payload_as()?;

        self.dashboard_cache.create_session_entry(SessionCacheEntry {
            session_id: payload.session_id,
            title: payload.title,
            description: payload.description,
            status: SessionStatus::Active,
            cycle_count: 0,
            updated_at: payload.created_at,
        }).await;

        Ok(())
    }

    async fn handle_session_renamed(&self, event: &EventEnvelope) -> Result<(), DomainError> {
        let payload: SessionRenamed = event.payload_as()?;

        self.dashboard_cache.update_session(
            payload.session_id,
            |entry| {
                entry.title = payload.new_title.clone();
                entry.updated_at = payload.renamed_at;
            },
        ).await;

        Ok(())
    }

    async fn handle_session_archived(&self, event: &EventEnvelope) -> Result<(), DomainError> {
        let payload: SessionArchived = event.payload_as()?;

        self.dashboard_cache.update_session(
            payload.session_id,
            |entry| {
                entry.status = SessionStatus::Archived;
                entry.updated_at = payload.archived_at;
            },
        ).await;

        Ok(())
    }

    async fn handle_cycle_added(&self, event: &EventEnvelope) -> Result<(), DomainError> {
        let payload: CycleAddedToSession = event.payload_as()?;

        self.dashboard_cache.update_session(
            payload.session_id,
            |entry| {
                entry.cycle_count += 1;
                entry.updated_at = payload.added_at;
            },
        ).await;

        Ok(())
    }

    // === Cycle Events ===

    async fn handle_cycle_created(&self, event: &EventEnvelope) -> Result<(), DomainError> {
        let payload: CycleCreated = event.payload_as()?;

        self.dashboard_cache.create_cycle_entry(CycleCacheEntry {
            cycle_id: payload.cycle_id,
            session_id: payload.session_id,
            parent_cycle_id: payload.parent_cycle_id,
            status: CycleStatus::Active,
            progress: CycleProgressSnapshot::default(),
            created_at: payload.created_at,
        }).await;

        Ok(())
    }

    async fn handle_cycle_branched(&self, event: &EventEnvelope) -> Result<(), DomainError> {
        let payload: CycleBranched = event.payload_as()?;

        // Create branch entry
        self.dashboard_cache.create_cycle_entry(CycleCacheEntry {
            cycle_id: payload.cycle_id,
            session_id: payload.session_id,
            parent_cycle_id: Some(payload.parent_cycle_id),
            status: CycleStatus::Active,
            progress: CycleProgressSnapshot {
                completed_count: payload.inherited_components.len() as i32,
                total_count: 9,
                percent_complete: (payload.inherited_components.len() * 100 / 9) as i32,
                current_step: payload.branch_point,
            },
            created_at: payload.branched_at,
        }).await;

        // Update parent to show it has branches
        self.dashboard_cache.add_child_cycle(
            payload.parent_cycle_id,
            payload.cycle_id,
        ).await;

        Ok(())
    }

    async fn handle_component_started(&self, event: &EventEnvelope) -> Result<(), DomainError> {
        let payload: ComponentStarted = event.payload_as()?;

        self.dashboard_cache.update_cycle(
            payload.cycle_id,
            |entry| {
                entry.progress.current_step = payload.component_type;
            },
        ).await;

        self.dashboard_cache.update_component_status(
            payload.cycle_id,
            payload.component_type,
            ComponentStatus::InProgress,
        ).await;

        Ok(())
    }

    async fn handle_component_completed(&self, event: &EventEnvelope) -> Result<(), DomainError> {
        let payload: ComponentCompleted = event.payload_as()?;

        self.dashboard_cache.update_cycle(
            payload.cycle_id,
            |entry| {
                entry.progress = payload.progress.clone();
            },
        ).await;

        self.dashboard_cache.update_component_status(
            payload.cycle_id,
            payload.component_type,
            ComponentStatus::Complete,
        ).await;

        Ok(())
    }

    async fn handle_component_output(&self, event: &EventEnvelope) -> Result<(), DomainError> {
        let payload: ComponentOutputUpdated = event.payload_as()?;

        // Mark component data as stale to force refresh on next read
        self.dashboard_cache.invalidate_component_data(
            payload.cycle_id,
            payload.component_type,
        ).await;

        Ok(())
    }

    async fn handle_cycle_completed(&self, event: &EventEnvelope) -> Result<(), DomainError> {
        let payload: CycleCompleted = event.payload_as()?;

        self.dashboard_cache.update_cycle(
            payload.cycle_id,
            |entry| {
                entry.status = CycleStatus::Completed;
                entry.progress.completed_count = entry.progress.total_count;
                entry.progress.percent_complete = 100;
            },
        ).await;

        // Store DQ score if available
        if let Some(dq_score) = payload.dq_overall_score {
            self.dashboard_cache.set_cycle_dq_score(
                payload.cycle_id,
                dq_score,
            ).await;
        }

        Ok(())
    }

    // === Conversation Events ===

    async fn handle_message_sent(&self, event: &EventEnvelope) -> Result<(), DomainError> {
        let payload: MessageSent = event.payload_as()?;

        self.dashboard_cache.add_recent_message(RecentMessage {
            cycle_id: payload.cycle_id,
            component_type: payload.component_type,
            role: payload.role,
            preview: payload.content_preview,
            timestamp: payload.sent_at,
        }).await;

        Ok(())
    }

    async fn handle_conversation_started(&self, event: &EventEnvelope) -> Result<(), DomainError> {
        let payload: ConversationStarted = event.payload_as()?;

        self.dashboard_cache.mark_conversation_active(
            payload.cycle_id,
            payload.component_type,
        ).await;

        Ok(())
    }

    // === Analysis Events ===

    async fn handle_pugh_scores(&self, event: &EventEnvelope) -> Result<(), DomainError> {
        let payload: PughScoresComputed = event.payload_as()?;

        self.dashboard_cache.set_pugh_scores(
            payload.cycle_id,
            PughScoresSummary {
                scores: payload.alternative_scores,
                best_alternative: payload.best_alternative_id,
                dominated_count: payload.dominated_alternatives.len() as i32,
            },
        ).await;

        Ok(())
    }

    async fn handle_dq_scores(&self, event: &EventEnvelope) -> Result<(), DomainError> {
        let payload: DQScoresComputed = event.payload_as()?;

        self.dashboard_cache.set_dq_scores(
            payload.cycle_id,
            DQScoresSummary {
                overall: payload.overall_score,
                weakest: payload.weakest_element,
                element_count: payload.element_scores.len() as i32,
            },
        ).await;

        Ok(())
    }

    async fn handle_tradeoffs(&self, event: &EventEnvelope) -> Result<(), DomainError> {
        let payload: TradeoffsAnalyzed = event.payload_as()?;

        self.dashboard_cache.set_tradeoffs_summary(
            payload.cycle_id,
            TradeoffsSummary {
                dominated_count: payload.dominated_count,
                tension_count: payload.tension_summaries.len() as i32,
            },
        ).await;

        Ok(())
    }
}
```

---

## Dashboard Cache

### Cache Structure

```rust
// backend/src/adapters/cache/dashboard_cache.rs

use std::collections::HashMap;
use std::sync::RwLock;

/// In-memory cache for dashboard read models
pub struct DashboardCache {
    sessions: RwLock<HashMap<SessionId, SessionCacheEntry>>,
    cycles: RwLock<HashMap<CycleId, CycleCacheEntry>>,
    component_statuses: RwLock<HashMap<(CycleId, ComponentType), ComponentStatus>>,
    pugh_scores: RwLock<HashMap<CycleId, PughScoresSummary>>,
    dq_scores: RwLock<HashMap<CycleId, DQScoresSummary>>,
    recent_messages: RwLock<HashMap<CycleId, Vec<RecentMessage>>>,
}

#[derive(Debug, Clone)]
pub struct SessionCacheEntry {
    pub session_id: SessionId,
    pub title: String,
    pub description: Option<String>,
    pub status: SessionStatus,
    pub cycle_count: i32,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone)]
pub struct CycleCacheEntry {
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    pub parent_cycle_id: Option<CycleId>,
    pub status: CycleStatus,
    pub progress: CycleProgressSnapshot,
    pub child_cycles: Vec<CycleId>,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone)]
pub struct PughScoresSummary {
    pub scores: HashMap<String, i32>,
    pub best_alternative: Option<String>,
    pub dominated_count: i32,
}

#[derive(Debug, Clone)]
pub struct DQScoresSummary {
    pub overall: Percentage,
    pub weakest: String,
    pub element_count: i32,
}

#[derive(Debug, Clone)]
pub struct RecentMessage {
    pub cycle_id: CycleId,
    pub component_type: ComponentType,
    pub role: Role,
    pub preview: String,
    pub timestamp: Timestamp,
}

impl DashboardCache {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            cycles: RwLock::new(HashMap::new()),
            component_statuses: RwLock::new(HashMap::new()),
            pugh_scores: RwLock::new(HashMap::new()),
            dq_scores: RwLock::new(HashMap::new()),
            recent_messages: RwLock::new(HashMap::new()),
        }
    }

    // === Session Operations ===

    pub async fn create_session_entry(&self, entry: SessionCacheEntry) {
        self.sessions.write().unwrap().insert(entry.session_id, entry);
    }

    pub async fn update_session<F>(&self, id: SessionId, updater: F)
    where
        F: FnOnce(&mut SessionCacheEntry),
    {
        if let Some(entry) = self.sessions.write().unwrap().get_mut(&id) {
            updater(entry);
        }
    }

    pub async fn get_session(&self, id: SessionId) -> Option<SessionCacheEntry> {
        self.sessions.read().unwrap().get(&id).cloned()
    }

    // === Cycle Operations ===

    pub async fn create_cycle_entry(&self, entry: CycleCacheEntry) {
        self.cycles.write().unwrap().insert(entry.cycle_id, entry);
    }

    pub async fn update_cycle<F>(&self, id: CycleId, updater: F)
    where
        F: FnOnce(&mut CycleCacheEntry),
    {
        if let Some(entry) = self.cycles.write().unwrap().get_mut(&id) {
            updater(entry);
        }
    }

    pub async fn add_child_cycle(&self, parent_id: CycleId, child_id: CycleId) {
        if let Some(entry) = self.cycles.write().unwrap().get_mut(&parent_id) {
            entry.child_cycles.push(child_id);
        }
    }

    // === Component Operations ===

    pub async fn update_component_status(
        &self,
        cycle_id: CycleId,
        comp_type: ComponentType,
        status: ComponentStatus,
    ) {
        self.component_statuses
            .write()
            .unwrap()
            .insert((cycle_id, comp_type), status);
    }

    pub async fn invalidate_component_data(&self, cycle_id: CycleId, comp_type: ComponentType) {
        // In a real implementation, this would mark the data as stale
        // For now, just log that it needs refresh
    }

    // === Analysis Operations ===

    pub async fn set_pugh_scores(&self, cycle_id: CycleId, scores: PughScoresSummary) {
        self.pugh_scores.write().unwrap().insert(cycle_id, scores);
    }

    pub async fn set_dq_scores(&self, cycle_id: CycleId, scores: DQScoresSummary) {
        self.dq_scores.write().unwrap().insert(cycle_id, scores);
    }

    pub async fn set_cycle_dq_score(&self, cycle_id: CycleId, score: Percentage) {
        if let Some(entry) = self.dq_scores.write().unwrap().get_mut(&cycle_id) {
            entry.overall = score;
        }
    }

    // === Message Operations ===

    pub async fn add_recent_message(&self, message: RecentMessage) {
        let cycle_id = message.cycle_id;
        let mut messages = self.recent_messages.write().unwrap();
        let cycle_messages = messages.entry(cycle_id).or_insert_with(Vec::new);

        cycle_messages.push(message);

        // Keep only last 50 messages per cycle
        if cycle_messages.len() > 50 {
            cycle_messages.remove(0);
        }
    }

    pub async fn mark_conversation_active(&self, cycle_id: CycleId, comp_type: ComponentType) {
        // Track which component has active conversation
    }

    // === Dashboard View Assembly ===

    pub async fn get_dashboard_overview(&self, session_id: SessionId) -> Option<DashboardOverview> {
        let session = self.sessions.read().unwrap().get(&session_id)?.clone();

        // Find active cycle (most recent)
        let active_cycle = self.cycles
            .read()
            .unwrap()
            .values()
            .filter(|c| c.session_id == session_id && c.status == CycleStatus::Active)
            .max_by_key(|c| c.created_at)
            .cloned();

        let active_cycle_id = active_cycle.as_ref().map(|c| c.cycle_id);

        // Get DQ score if available
        let dq_score = active_cycle_id
            .and_then(|id| self.dq_scores.read().unwrap().get(&id).cloned())
            .map(|s| s.overall);

        Some(DashboardOverview {
            session_id: session.session_id,
            session_title: session.title,
            cycle_count: session.cycle_count,
            active_cycle_id,
            dq_score,
            last_updated: session.updated_at,
        })
    }
}
```

---

## Acceptance Criteria

### AC1: Handles All Event Types

**Given** any dashboard-relevant event is published
**When** `DashboardUpdateHandler` processes it
**Then** the appropriate cache update occurs

### AC2: Idempotent Processing

**Given** the same event is processed twice
**When** handler checks processed events set
**Then** second processing is skipped (no duplicate updates)

### AC3: WebSocket Forwarding

**Given** a valid event is processed
**When** cache is updated successfully
**Then** event is forwarded to WebSocket bridge for client broadcast

### AC4: Graceful Unknown Events

**Given** an unknown event type is received
**When** handler processes it
**Then** event is ignored without error

### AC5: Incremental Updates

**Given** a `session.renamed` event is received
**When** cache is updated
**Then** only the title field is modified (not full session replaced)

### AC6: Cache Consistency

**Given** a `component.completed` event is received
**When** cache is updated
**Then** both cycle progress and component status are updated atomically

---

## File Structure

```
backend/src/application/handlers/
├── mod.rs                      # Add dashboard handler
├── dashboard_update.rs         # NEW: DashboardUpdateHandler
└── dashboard_update_test.rs    # NEW: Tests

backend/src/adapters/cache/
├── mod.rs                      # NEW: Module exports
├── dashboard_cache.rs          # NEW: DashboardCache
└── dashboard_cache_test.rs     # NEW: Tests
```

---

## Test Specifications

```rust
#[tokio::test]
async fn handles_session_created() {
    let cache = Arc::new(DashboardCache::new());
    let ws_bridge = Arc::new(MockWebSocketBridge::new());
    let handler = DashboardUpdateHandler::new(cache.clone(), ws_bridge);

    let event = create_event("session.created", json!({
        "session_id": "sess-1",
        "user_id": "user-1",
        "title": "My Decision",
        "description": "Important choice",
        "created_at": "2026-01-07T10:00:00Z"
    }));

    handler.handle(event).await.unwrap();

    let session = cache.get_session(SessionId::from_string("sess-1")).await;
    assert!(session.is_some());
    assert_eq!(session.unwrap().title, "My Decision");
}

#[tokio::test]
async fn handles_component_completed_updates_progress() {
    let cache = Arc::new(DashboardCache::new());
    let ws_bridge = Arc::new(MockWebSocketBridge::new());
    let handler = DashboardUpdateHandler::new(cache.clone(), ws_bridge);

    // Setup cycle in cache
    cache.create_cycle_entry(CycleCacheEntry {
        cycle_id: CycleId::from_string("cycle-1"),
        session_id: SessionId::from_string("sess-1"),
        parent_cycle_id: None,
        status: CycleStatus::Active,
        progress: CycleProgressSnapshot::default(),
        child_cycles: vec![],
        created_at: Timestamp::now(),
    }).await;

    let event = create_event("component.completed", json!({
        "cycle_id": "cycle-1",
        "session_id": "sess-1",
        "component_id": "comp-1",
        "component_type": "objectives",
        "completed_at": "2026-01-07T10:00:00Z",
        "progress": {
            "completed_count": 3,
            "total_count": 9,
            "percent_complete": 33,
            "current_step": "alternatives"
        }
    }));

    handler.handle(event).await.unwrap();

    let cycles = cache.cycles.read().unwrap();
    let cycle = cycles.get(&CycleId::from_string("cycle-1")).unwrap();
    assert_eq!(cycle.progress.completed_count, 3);
    assert_eq!(cycle.progress.percent_complete, 33);
}

#[tokio::test]
async fn idempotent_processing() {
    let cache = Arc::new(DashboardCache::new());
    let ws_bridge = Arc::new(MockWebSocketBridge::new());
    let handler = DashboardUpdateHandler::new(cache.clone(), ws_bridge.clone());

    let event = create_event_with_id("evt-123", "session.created", json!({
        "session_id": "sess-1",
        "user_id": "user-1",
        "title": "First",
        "created_at": "2026-01-07T10:00:00Z"
    }));

    // First processing
    handler.handle(event.clone()).await.unwrap();

    // Modify title in event (simulating duplicate with different data - shouldn't happen but tests idempotency)
    let duplicate = create_event_with_id("evt-123", "session.created", json!({
        "session_id": "sess-1",
        "user_id": "user-1",
        "title": "Second",
        "created_at": "2026-01-07T10:00:00Z"
    }));

    // Second processing should skip
    handler.handle(duplicate).await.unwrap();

    // Title should still be "First"
    let session = cache.get_session(SessionId::from_string("sess-1")).await.unwrap();
    assert_eq!(session.title, "First");

    // WebSocket bridge should only receive once
    assert_eq!(ws_bridge.forward_count(), 1);
}

#[tokio::test]
async fn forwards_to_websocket_bridge() {
    let cache = Arc::new(DashboardCache::new());
    let ws_bridge = Arc::new(MockWebSocketBridge::new());
    let handler = DashboardUpdateHandler::new(cache.clone(), ws_bridge.clone());

    let event = create_event("message.sent", json!({
        "conversation_id": "conv-1",
        "message_id": "msg-1",
        "component_id": "comp-1",
        "cycle_id": "cycle-1",
        "session_id": "sess-1",
        "component_type": "issue_raising",
        "role": "user",
        "content_preview": "I need to decide about...",
        "sent_at": "2026-01-07T10:00:00Z"
    }));

    handler.handle(event).await.unwrap();

    // Verify forwarded to WebSocket
    assert!(ws_bridge.was_forwarded("message.sent"));
}

#[tokio::test]
async fn ignores_unknown_event_types() {
    let cache = Arc::new(DashboardCache::new());
    let ws_bridge = Arc::new(MockWebSocketBridge::new());
    let handler = DashboardUpdateHandler::new(cache.clone(), ws_bridge);

    let event = create_event("unknown.event.type", json!({}));

    // Should not error
    let result = handler.handle(event).await;
    assert!(result.is_ok());
}
```

---

## Event Registration

```rust
// backend/src/main.rs or setup module

fn register_dashboard_handlers(event_bus: &impl EventSubscriber, deps: &Dependencies) {
    let dashboard_handler = DashboardUpdateHandler::new(
        deps.dashboard_cache.clone(),
        deps.websocket_bridge.clone(),
    );

    // Subscribe to all dashboard-relevant events
    event_bus.subscribe_all(
        &[
            // Session events
            "session.created",
            "session.renamed",
            "session.archived",
            "session.cycle_added",
            // Cycle events
            "cycle.created",
            "cycle.branched",
            "component.started",
            "component.completed",
            "component.output_updated",
            "cycle.completed",
            // Conversation events
            "message.sent",
            "conversation.started",
            // Analysis events
            "analysis.pugh_scores_computed",
            "analysis.dq_scores_computed",
            "analysis.tradeoffs_analyzed",
        ],
        dashboard_handler,
    );
}
```

---

## Dependencies

### Module Dependencies

- All event types from all modules
- `ports::event_subscriber` - EventHandler trait
- No direct module dependencies (only event schemas)

---

## Related Documents

- **Integration Spec:** features/integrations/full-proact-journey.md
- **WebSocket:** features/integrations/websocket-dashboard.md
- **Checklist:** REQUIREMENTS/CHECKLIST-events.md (Phase 6)
- **Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md

---

*Version: 1.0.0*
*Created: 2026-01-07*
*Phase: 6 of 8*
