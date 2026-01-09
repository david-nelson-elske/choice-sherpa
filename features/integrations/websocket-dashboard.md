# Integration: WebSocket Real-Time Dashboard

**Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md
**Type:** Infrastructure + User Experience
**Priority:** P1
**Depends On:** features/integrations/full-proact-journey.md (Phase 1-6)

> Real-time dashboard updates via WebSocket, bridging the event bus to connected frontend clients.

---

## Overview

The WebSocket Dashboard integration connects the internal event bus to frontend clients, enabling real-time updates without polling. When domain events are published (e.g., `ComponentCompleted`, `MessageSent`), connected clients receive immediate notifications to update their UI.

### Architecture Position

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Event Bus                                    │
│   InMemoryEventBus (test) │ RedisEventBus (production)              │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ subscribes
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    WebSocketEventBridge                              │
│   - Subscribes to dashboard-relevant events                         │
│   - Transforms EventEnvelope → DashboardUpdate                      │
│   - Routes to appropriate session rooms                             │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ broadcasts
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      WebSocket Connections                           │
│   Room: session-123    Room: session-456    Room: session-789       │
│   ├── client-a         ├── client-d         ├── client-g            │
│   ├── client-b         └── client-e         └── client-h            │
│   └── client-c                                                       │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ JSON messages
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Frontend Clients                             │
│   - useDashboardLive() hook                                         │
│   - Updates Svelte stores on message                                │
│   - Handles reconnection                                            │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Modules Involved

| Module | Role | Changes Required |
|--------|------|------------------|
| `foundation` | Producer | Add `DashboardUpdate` message types |
| `adapters/websocket` | Both | New module: WebSocket handlers, rooms, bridge |
| `adapters/http` | Producer | WebSocket upgrade endpoint |
| `frontend` | Consumer | `useDashboardLive` hook, store integration |

---

## Data Flow

### Connection Flow

```
Client                    Server
  │                         │
  │──── GET /api/sessions/:id/live ────▶│
  │      Upgrade: websocket              │
  │                                      │
  │◀─── 101 Switching Protocols ────────│
  │                                      │
  │◀─── {"type":"connected"} ───────────│
  │                                      │
  │      (client joins session room)     │
  │                                      │
```

### Event Flow

```
Domain Event Published
         │
         ▼
┌─────────────────────┐
│  WebSocketBridge    │
│  receives event     │
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│  Transform to       │
│  DashboardUpdate    │
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│  Find session room  │
│  from aggregate_id  │
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│  Broadcast to all   │
│  clients in room    │
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│  Frontend receives  │
│  updates store      │
└─────────────────────┘
```

---

## Message Protocol

### Server → Client Messages

```typescript
// Base message structure
interface WebSocketMessage {
  type: string;
  timestamp: string;
  correlationId?: string;
}

// Connection established
interface ConnectedMessage extends WebSocketMessage {
  type: 'connected';
  sessionId: string;
  clientId: string;
}

// Dashboard update (most common)
interface DashboardUpdateMessage extends WebSocketMessage {
  type: 'dashboard.update';
  updateType: DashboardUpdateType;
  data: DashboardUpdateData;
}

type DashboardUpdateType =
  | 'session.metadata'      // Title, description changed
  | 'cycle.created'         // New cycle added
  | 'cycle.progress'        // Cycle progress changed
  | 'component.started'     // Component work began
  | 'component.completed'   // Component finished
  | 'component.output'      // Component output updated
  | 'conversation.message'  // New chat message
  | 'analysis.scores'       // Pugh/DQ scores computed
  | 'cycle.completed';      // Cycle finished

// Example: Component completed
interface ComponentCompletedData {
  cycleId: string;
  componentType: ComponentType;
  completedAt: string;
  progress: {
    completed: number;
    total: number;
    percent: number;
  };
}

// Example: New message
interface ConversationMessageData {
  cycleId: string;
  componentType: ComponentType;
  message: {
    id: string;
    role: 'user' | 'assistant';
    contentPreview: string;  // First 100 chars
    timestamp: string;
  };
}

// Example: Scores computed
interface AnalysisScoresData {
  cycleId: string;
  scoreType: 'pugh' | 'dq';
  scores: Record<string, number>;
  overallScore?: number;
}

// Error message
interface ErrorMessage extends WebSocketMessage {
  type: 'error';
  code: string;
  message: string;
}

// Heartbeat/ping
interface PingMessage extends WebSocketMessage {
  type: 'ping';
}

interface PongMessage extends WebSocketMessage {
  type: 'pong';
}
```

### Client → Server Messages

```typescript
// Client ping (keepalive)
interface ClientPing {
  type: 'ping';
}

// Request full state (after reconnection)
interface RequestStateMessage {
  type: 'request.state';
}
```

---

## Backend Design

### WebSocket Event Bridge

```rust
// backend/src/adapters/websocket/event_bridge.rs

use std::sync::Arc;

/// Bridge between event bus and WebSocket connections
pub struct WebSocketEventBridge {
    room_manager: Arc<RoomManager>,
}

impl WebSocketEventBridge {
    pub fn new(room_manager: Arc<RoomManager>) -> Self {
        Self { room_manager }
    }

    /// Register as event handler for dashboard-relevant events
    pub fn register(&self, event_bus: &impl EventSubscriber) {
        event_bus.subscribe_all(
            &[
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
            ],
            self.clone(),
        );
    }

    /// Transform domain event to dashboard update
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

    /// Resolve session ID from event
    fn resolve_session_id(&self, event: &EventEnvelope) -> Option<SessionId> {
        // Session events have session_id directly
        if event.aggregate_type == "Session" {
            return Some(SessionId::from_string(&event.aggregate_id));
        }

        // Cycle events need to look up session
        if event.aggregate_type == "Cycle" {
            // Extract from payload or use cached mapping
            if let Some(session_id) = event.payload.get("session_id") {
                return session_id.as_str().map(|s| SessionId::from_string(s));
            }
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
```

### Room Manager

```rust
// backend/src/adapters/websocket/rooms.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Manages WebSocket connection rooms organized by session
pub struct RoomManager {
    /// Map of session_id -> broadcast sender
    rooms: RwLock<HashMap<SessionId, broadcast::Sender<DashboardUpdate>>>,

    /// Map of client_id -> session_id (for cleanup)
    client_sessions: RwLock<HashMap<ClientId, SessionId>>,

    /// Channel capacity for each room
    channel_capacity: usize,
}

impl RoomManager {
    pub fn new(channel_capacity: usize) -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
            client_sessions: RwLock::new(HashMap::new()),
            channel_capacity,
        }
    }

    /// Join a client to a session room
    pub async fn join(
        &self,
        session_id: &SessionId,
        client_id: ClientId,
    ) -> broadcast::Receiver<DashboardUpdate> {
        let mut rooms = self.rooms.write().await;

        // Get or create room
        let sender = rooms
            .entry(session_id.clone())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(self.channel_capacity);
                tx
            });

        // Track client's session
        self.client_sessions
            .write()
            .await
            .insert(client_id, session_id.clone());

        sender.subscribe()
    }

    /// Remove a client from their room
    pub async fn leave(&self, client_id: &ClientId) {
        let mut client_sessions = self.client_sessions.write().await;

        if let Some(session_id) = client_sessions.remove(client_id) {
            // Check if room is empty and clean up
            let rooms = self.rooms.read().await;
            if let Some(sender) = rooms.get(&session_id) {
                if sender.receiver_count() == 0 {
                    drop(rooms);
                    self.rooms.write().await.remove(&session_id);
                }
            }
        }
    }

    /// Broadcast update to all clients in a session room
    pub async fn broadcast_to_session(&self, session_id: &SessionId, update: DashboardUpdate) {
        let rooms = self.rooms.read().await;

        if let Some(sender) = rooms.get(session_id) {
            // Ignore send errors (no receivers is OK)
            let _ = sender.send(update);
        }
    }

    /// Get count of connected clients in a room
    pub async fn client_count(&self, session_id: &SessionId) -> usize {
        let rooms = self.rooms.read().await;
        rooms
            .get(session_id)
            .map(|s| s.receiver_count())
            .unwrap_or(0)
    }

    /// Get all active room IDs (for monitoring)
    pub async fn active_rooms(&self) -> Vec<SessionId> {
        self.rooms.read().await.keys().cloned().collect()
    }
}
```

### WebSocket Handler

```rust
// backend/src/adapters/websocket/handler.rs

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(session_id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    // Validate session exists and user has access
    let session_id = SessionId::from_string(&session_id);

    // TODO: Add authentication check here
    // let user = authenticate(&request)?;
    // authorize_session_access(&user, &session_id)?;

    ws.on_upgrade(move |socket| handle_socket(socket, session_id, state))
}

async fn handle_socket(socket: WebSocket, session_id: SessionId, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Generate client ID
    let client_id = ClientId::new();

    // Join session room
    let mut room_rx = state.room_manager.join(&session_id, client_id.clone()).await;

    // Send connected message
    let connected = ConnectedMessage {
        message_type: "connected".to_string(),
        session_id: session_id.to_string(),
        client_id: client_id.to_string(),
        timestamp: Timestamp::now().to_string(),
    };

    if sender
        .send(Message::Text(serde_json::to_string(&connected).unwrap()))
        .await
        .is_err()
    {
        return; // Client disconnected
    }

    // Spawn task to forward room broadcasts to client
    let mut send_task = tokio::spawn(async move {
        while let Ok(update) = room_rx.recv().await {
            let msg = DashboardUpdateMessage {
                message_type: "dashboard.update".to_string(),
                update_type: update.update_type,
                data: update.data,
                timestamp: update.timestamp.to_string(),
                correlation_id: update.correlation_id,
            };

            if sender
                .send(Message::Text(serde_json::to_string(&msg).unwrap()))
                .await
                .is_err()
            {
                break; // Client disconnected
            }
        }
    });

    // Handle incoming messages from client
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        match client_msg.message_type.as_str() {
                            "ping" => {
                                // Respond with pong (handled by send_task)
                            }
                            "request.state" => {
                                // Client wants full state after reconnection
                                // TODO: Fetch and send full dashboard state
                            }
                            _ => {}
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    // Cleanup: leave room
    state.room_manager.leave(&client_id).await;
}
```

### Routes

```rust
// backend/src/adapters/http/routes.rs

use axum::{routing::get, Router};

pub fn websocket_routes() -> Router<AppState> {
    Router::new()
        .route("/api/sessions/:session_id/live", get(ws_handler))
}
```

---

## Frontend Design

### useDashboardLive Hook

```typescript
// frontend/src/lib/hooks/use-dashboard-live.ts

import { onMount, onDestroy } from 'svelte';
import { writable, type Writable } from 'svelte/store';
import type { DashboardUpdate, WebSocketMessage } from '$lib/types';

interface UseDashboardLiveOptions {
  sessionId: string;
  onUpdate?: (update: DashboardUpdate) => void;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
}

interface DashboardLiveState {
  connected: boolean;
  clientId: string | null;
  lastUpdate: DashboardUpdate | null;
  error: Error | null;
}

export function useDashboardLive(options: UseDashboardLiveOptions) {
  const {
    sessionId,
    onUpdate,
    reconnectInterval = 3000,
    maxReconnectAttempts = 10,
  } = options;

  const state: Writable<DashboardLiveState> = writable({
    connected: false,
    clientId: null,
    lastUpdate: null,
    error: null,
  });

  let ws: WebSocket | null = null;
  let reconnectAttempts = 0;
  let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
  let pingInterval: ReturnType<typeof setInterval> | null = null;

  function connect() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${protocol}//${window.location.host}/api/sessions/${sessionId}/live`;

    ws = new WebSocket(url);

    ws.onopen = () => {
      reconnectAttempts = 0;
      startPingInterval();
    };

    ws.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data);
        handleMessage(message);
      } catch (e) {
        console.error('Failed to parse WebSocket message:', e);
      }
    };

    ws.onclose = (event) => {
      state.update((s) => ({ ...s, connected: false }));
      stopPingInterval();

      if (!event.wasClean && reconnectAttempts < maxReconnectAttempts) {
        scheduleReconnect();
      }
    };

    ws.onerror = (error) => {
      state.update((s) => ({ ...s, error: new Error('WebSocket error') }));
    };
  }

  function handleMessage(message: WebSocketMessage) {
    switch (message.type) {
      case 'connected':
        state.update((s) => ({
          ...s,
          connected: true,
          clientId: message.clientId,
          error: null,
        }));
        break;

      case 'dashboard.update':
        const update = message as DashboardUpdate;
        state.update((s) => ({ ...s, lastUpdate: update }));
        onUpdate?.(update);
        dispatchUpdateEvent(update);
        break;

      case 'pong':
        // Heartbeat acknowledged
        break;

      case 'error':
        state.update((s) => ({
          ...s,
          error: new Error(message.message),
        }));
        break;
    }
  }

  function dispatchUpdateEvent(update: DashboardUpdate) {
    // Dispatch custom event for store integration
    window.dispatchEvent(
      new CustomEvent('dashboard:update', { detail: update })
    );
  }

  function scheduleReconnect() {
    reconnectAttempts++;
    const delay = reconnectInterval * Math.pow(1.5, reconnectAttempts - 1);

    reconnectTimeout = setTimeout(() => {
      connect();
    }, Math.min(delay, 30000)); // Cap at 30s
  }

  function startPingInterval() {
    pingInterval = setInterval(() => {
      if (ws?.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: 'ping' }));
      }
    }, 30000); // Ping every 30s
  }

  function stopPingInterval() {
    if (pingInterval) {
      clearInterval(pingInterval);
      pingInterval = null;
    }
  }

  function disconnect() {
    if (reconnectTimeout) {
      clearTimeout(reconnectTimeout);
    }
    stopPingInterval();

    if (ws) {
      ws.close(1000, 'Client disconnect');
      ws = null;
    }
  }

  function requestFullState() {
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ type: 'request.state' }));
    }
  }

  onMount(() => {
    connect();
  });

  onDestroy(() => {
    disconnect();
  });

  return {
    state,
    disconnect,
    reconnect: connect,
    requestFullState,
  };
}
```

### Dashboard Store Integration

```typescript
// frontend/src/lib/stores/dashboard.ts

import { writable, derived } from 'svelte/store';
import type { DashboardOverview, DashboardUpdate } from '$lib/types';

// Core dashboard state
export const dashboardOverview = writable<DashboardOverview | null>(null);

// Update handlers by type
const updateHandlers: Record<string, (data: any) => void> = {
  'cycle.created': (data) => {
    dashboardOverview.update((overview) => {
      if (!overview) return overview;
      return {
        ...overview,
        cycle_count: overview.cycle_count + 1,
      };
    });
  },

  'component.completed': (data) => {
    dashboardOverview.update((overview) => {
      if (!overview) return overview;
      // Update progress for the active cycle
      // This is a simplified example
      return overview;
    });
  },

  'component.output': (data) => {
    dashboardOverview.update((overview) => {
      if (!overview) return overview;
      // Update specific component output
      return overview;
    });
  },

  'conversation.message': (data) => {
    // Update conversation store instead
    conversationMessages.update((msgs) => [...msgs, data.message]);
  },

  'analysis.scores': (data) => {
    dashboardOverview.update((overview) => {
      if (!overview) return overview;
      if (data.scoreType === 'dq') {
        return { ...overview, dq_score: data.overallScore };
      }
      // Update Pugh scores in alternatives
      return overview;
    });
  },
};

// Listen for WebSocket updates
if (typeof window !== 'undefined') {
  window.addEventListener('dashboard:update', ((event: CustomEvent) => {
    const update: DashboardUpdate = event.detail;
    const handler = updateHandlers[update.updateType];
    if (handler) {
      handler(update.data);
    }
  }) as EventListener);
}

// Derived stores for specific views
export const currentProgress = derived(dashboardOverview, ($overview) => {
  if (!$overview) return { completed: 0, total: 9, percent: 0 };
  // Calculate from component statuses
  return { completed: 3, total: 9, percent: 33 };
});

export const hasRecommendation = derived(dashboardOverview, ($overview) => {
  return $overview?.recommendation != null;
});
```

### Component Usage

```svelte
<!-- frontend/src/routes/dashboard/[sessionId]/+page.svelte -->
<script lang="ts">
  import { page } from '$app/stores';
  import { useDashboardLive } from '$lib/hooks/use-dashboard-live';
  import { dashboardOverview } from '$lib/stores/dashboard';

  const sessionId = $page.params.sessionId;

  const { state } = useDashboardLive({
    sessionId,
    onUpdate: (update) => {
      console.log('Dashboard update:', update.updateType);
    },
  });
</script>

<div class="dashboard">
  {#if $state.connected}
    <span class="status connected">Live</span>
  {:else}
    <span class="status disconnected">Reconnecting...</span>
  {/if}

  {#if $dashboardOverview}
    <h1>{$dashboardOverview.session_title}</h1>
    <!-- Dashboard content -->
  {:else}
    <p>Loading...</p>
  {/if}
</div>
```

---

## Failure Modes

| Failure | Impact | Detection | Recovery |
|---------|--------|-----------|----------|
| WebSocket disconnect | Client stops receiving updates | `onclose` event | Auto-reconnect with backoff |
| Room not found | New connections fail | Error on join | Create room on demand |
| Event parsing fails | Update lost | JSON parse error | Log, continue |
| Client ping timeout | Stale connection | No pong response | Server closes connection |
| High message volume | Slow clients lag | Channel buffer full | Oldest messages dropped |

### Reconnection Strategy

```
Attempt 1: Wait 3s
Attempt 2: Wait 4.5s
Attempt 3: Wait 6.75s
...
Attempt N: Wait min(3s * 1.5^(N-1), 30s)
Max attempts: 10
```

After max attempts, show permanent error and require manual refresh.

---

## Coordination Points

### Event Types for WebSocket

| Domain Event | WebSocket Update Type | Data Payload |
|--------------|----------------------|--------------|
| `session.created` | `session.metadata` | title, description |
| `session.renamed` | `session.metadata` | title |
| `cycle.created` | `cycle.created` | cycleId, sessionId |
| `cycle.branched` | `cycle.created` | cycleId, parentId, branchPoint |
| `component.started` | `component.started` | cycleId, componentType |
| `component.completed` | `component.completed` | cycleId, componentType, progress |
| `component.output_updated` | `component.output` | cycleId, componentType, summary |
| `message.sent` | `conversation.message` | cycleId, componentType, message |
| `pugh_scores.computed` | `analysis.scores` | cycleId, scores |
| `dq_scores.computed` | `analysis.scores` | cycleId, scores, overall |
| `cycle.completed` | `cycle.completed` | cycleId, dqScore |

---

## Implementation Phases

### Phase 1: Basic Infrastructure

**Goal:** WebSocket connection with rooms

**Deliverables:**
- [ ] `RoomManager` struct with join/leave
- [ ] WebSocket upgrade handler
- [ ] Connected/disconnected messages
- [ ] Basic `useDashboardLive` hook

**Exit Criteria:** Client can connect and receive connected message

---

### Phase 2: Event Bridge

**Goal:** Domain events flow to clients

**Deliverables:**
- [ ] `WebSocketEventBridge` handler
- [ ] Event → DashboardUpdate transformation
- [ ] Session ID resolution from events
- [ ] Broadcast to rooms

**Exit Criteria:** Publishing event reaches connected clients

---

### Phase 3: Frontend Integration

**Goal:** Updates reflected in UI

**Deliverables:**
- [ ] Dashboard store update handlers
- [ ] Update type discrimination
- [ ] Optimistic UI patterns
- [ ] Error handling

**Exit Criteria:** UI updates in real-time on events

---

### Phase 4: Reliability

**Goal:** Handle disconnections gracefully

**Deliverables:**
- [ ] Reconnection with backoff
- [ ] Request full state on reconnect
- [ ] Ping/pong heartbeat
- [ ] Connection status UI

**Exit Criteria:** Client recovers from network issues

---

## Testing Strategy

### Unit Tests

| Component | Test Focus |
|-----------|------------|
| RoomManager | Join/leave, broadcast, cleanup |
| EventBridge | Event transformation, routing |
| useDashboardLive | Connection lifecycle, reconnect |
| Store handlers | Update application |

### Integration Tests

| Test | Scenario |
|------|----------|
| ConnectionFlow | Connect → receive connected → disconnect |
| EventDelivery | Publish event → client receives update |
| RoomIsolation | Events only reach correct session |
| Reconnection | Disconnect → reconnect → state sync |

### E2E Tests

| Journey | Steps |
|---------|-------|
| LiveDashboard | Open dashboard → complete component → see update |
| MultiClient | Two clients → one updates → both see change |
| Reconnect | Disconnect network → reconnect → see missed updates |

---

## API Contracts

### Endpoints

| Method | Path | Purpose |
|--------|------|---------|
| `GET` (upgrade) | `/api/sessions/:id/live` | WebSocket connection |

### Message Schemas

See [Message Protocol](#message-protocol) section above.

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Required (token in query param or cookie) |
| Authorization Model | User must own session or have share access |
| Sensitive Data | Real-time decision updates (filtered by authorization) |
| Rate Limiting | Required: 5 connections per user, 100/minute connection attempts per IP |
| Audit Logging | Connection established/closed, authorization failures |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| `clientId` | Internal | Ephemeral; safe to log |
| `sessionId` | Internal | Safe to log |
| Dashboard update payloads | Confidential | Filtered by user authorization |
| `content_preview` | Confidential | Truncated to 100 chars for safety |

### Security Controls

- **Origin Validation**: Reject WebSocket upgrades from unauthorized origins
  ```rust
  let allowed_origins = ["https://app.choicesherpa.com", "https://choicesherpa.com"];
  let origin = request.headers().get("Origin");
  if !allowed_origins.contains(&origin) {
      return Err(AuthError::InvalidOrigin);
  }
  ```
- **Authentication Required**: WebSocket upgrade MUST verify user token before accepting
- **Authorization Check**: User must have access to the specific session
- **Connection Rate Limiting**:
  - Maximum 5 concurrent connections per user
  - Maximum 100 connection attempts per minute per IP
- **Message Size Limit**: Incoming messages capped at 4KB
- **Events Filtered by Authorization**: Only broadcast events user is authorized to see
- **No Sensitive Data in Previews**: `content_preview` limited to 100 characters

### WebSocket-Specific Security

```rust
// Required authentication middleware for WebSocket
async fn authenticate_ws(
    request: &Request,
    session_id: &SessionId,
) -> Result<UserId, AuthError> {
    // 1. Validate origin
    let origin = request.headers().get("Origin")
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthError::MissingOrigin)?;

    if !is_allowed_origin(origin) {
        return Err(AuthError::InvalidOrigin);
    }

    // 2. Extract and validate token
    let token = request
        .headers()
        .get("Authorization")
        .or_else(|| request.uri().query().and_then(|q| parse_token(q)))
        .ok_or(AuthError::MissingToken)?;

    let user_id = validate_token(token)?;

    // 3. Check session access
    if !user_has_session_access(&user_id, session_id).await? {
        return Err(AuthError::Forbidden);
    }

    Ok(user_id)
}
```

### Connection Limits

| Limit | Value | Enforcement |
|-------|-------|-------------|
| Connections per user | 5 | Reject new connections if exceeded |
| Connection attempts per IP | 100/minute | Rate limiter at load balancer |
| Incoming message size | 4KB | Close connection on oversized message |
| Ping interval | 30 seconds | Server-initiated |
| Ping timeout | 60 seconds | Close connection if no pong |

---

## File Structure

```
backend/src/adapters/websocket/
├── mod.rs                    # Module exports
├── event_bridge.rs           # WebSocketEventBridge
├── event_bridge_test.rs
├── rooms.rs                  # RoomManager
├── rooms_test.rs
├── handler.rs                # WebSocket handler
├── handler_test.rs
├── messages.rs               # Message types
└── auth.rs                   # WebSocket authentication

backend/src/adapters/http/
├── routes.rs                 # Add WebSocket route

frontend/src/lib/
├── hooks/
│   ├── use-dashboard-live.ts
│   └── use-dashboard-live.test.ts
├── stores/
│   └── dashboard.ts          # Add update handlers
└── types/
    └── websocket.ts          # Message types
```

---

## Related Documents

- **Integration Spec:** features/integrations/full-proact-journey.md
- **Checklist:** REQUIREMENTS/CHECKLIST-events.md (Phase 8)
- **Feature:** features/foundation/event-infrastructure.md
- **Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md

---

*Version: 1.0.0*
*Created: 2026-01-07*
*Depends On: Event Infrastructure (Phase 1-6)*
