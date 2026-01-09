# Infrastructure: WebSocket Event Bridge

**Type:** Cross-Cutting Infrastructure
**Priority:** P0 (Required for real-time dashboard)
**Depends On:** Event Infrastructure, Foundation module
**Last Updated:** 2026-01-08

> Real-time event delivery to web clients via WebSocket, enabling live dashboard updates without polling.

---

## Problem Statement

Choice Sherpa's event bus handles server-side event distribution, but clients need real-time updates:

1. Dashboard should update when components complete
2. Conversation UI should show streaming AI responses
3. Multi-tab/device scenarios need synchronization
4. Users shouldn't need to refresh to see changes

### Current State

- Event bus publishes to server-side handlers only
- No mechanism for events to reach browser clients
- Dashboard requires manual refresh for updates

### Desired State

- Events automatically pushed to connected clients
- Session-scoped broadcasting (users only see their events)
- Graceful reconnection with catch-up logic
- Efficient binary protocol for high-volume streaming

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           SERVER                                             │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                         Event Bus                                    │   │
│   │                                                                      │   │
│   │   session.created ──────────────────────────────────────────┐       │   │
│   │   cycle.created ────────────────────────────────────────────┤       │   │
│   │   component.completed ──────────────────────────────────────┤       │   │
│   │   conversation.message_sent ────────────────────────────────┤       │   │
│   └─────────────────────────────────────────────────────────────┼───────┘   │
│                                                                  │           │
│                                                                  ▼           │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     WebSocket Event Bridge                          │   │
│   │                                                                      │   │
│   │   ┌─────────────────────────────────────────────────────────────┐   │   │
│   │   │               Room Manager                                   │   │   │
│   │   │                                                              │   │   │
│   │   │   session:sess-1 ──► [ws-conn-a, ws-conn-b]                 │   │   │
│   │   │   session:sess-2 ──► [ws-conn-c]                            │   │   │
│   │   │   user:user-123  ──► [ws-conn-a, ws-conn-c]                 │   │   │
│   │   └─────────────────────────────────────────────────────────────┘   │   │
│   │                                                                      │   │
│   │   Event Filter: Only send events user is authorized to see          │   │
│   │   Room Routing: Send to session-specific rooms                      │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                  │           │
└──────────────────────────────────────────────────────────────────┼───────────┘
                                                                   │
                                   WebSocket (wss://)              │
                                                                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           CLIENT (Browser)                                   │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     WebSocket Client                                 │   │
│   │                                                                      │   │
│   │   - Auto-reconnect with exponential backoff                         │   │
│   │   - Heartbeat/ping-pong for connection health                       │   │
│   │   - Event dispatching to Svelte stores                              │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   ┌────────────────┐  ┌────────────────┐  ┌────────────────────────────┐   │
│   │ Dashboard Store│  │  Session Store │  │  Conversation Store        │   │
│   │                │  │                │  │                            │   │
│   │ on: cycle.*    │  │ on: session.*  │  │ on: conversation.*         │   │
│   │ on: component.*│  │                │  │ on: component.output_updated│   │
│   └────────────────┘  └────────────────┘  └────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Server-Side Components

### WebSocketEventBridge

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Bridges domain events to WebSocket connections
pub struct WebSocketEventBridge {
    /// Room manager for session-scoped broadcasting
    rooms: Arc<RwLock<RoomManager>>,
    /// Authorization checker for filtering events
    access_checker: Arc<dyn AccessChecker>,
}

impl WebSocketEventBridge {
    pub fn new(access_checker: Arc<dyn AccessChecker>) -> Self {
        Self {
            rooms: Arc::new(RwLock::new(RoomManager::new())),
            access_checker,
        }
    }

    /// Register a new WebSocket connection
    pub async fn register_connection(
        &self,
        user_id: UserId,
        session_id: Option<SessionId>,
    ) -> (ConnectionId, broadcast::Receiver<WebSocketMessage>) {
        let mut rooms = self.rooms.write().await;
        rooms.register(user_id, session_id)
    }

    /// Unregister a WebSocket connection
    pub async fn unregister_connection(&self, conn_id: ConnectionId) {
        let mut rooms = self.rooms.write().await;
        rooms.unregister(conn_id);
    }

    /// Broadcast an event to relevant connections
    pub async fn broadcast(&self, event: &EventEnvelope) -> Result<usize, BroadcastError> {
        let rooms = self.rooms.read().await;

        // Determine which room(s) should receive this event
        let targets = self.determine_targets(event).await?;

        let mut sent_count = 0;
        for target in targets {
            if let Some(sender) = rooms.get_sender(&target) {
                let message = self.event_to_message(event);
                if sender.send(message).is_ok() {
                    sent_count += 1;
                }
            }
        }

        Ok(sent_count)
    }

    /// Determine which rooms should receive an event
    async fn determine_targets(&self, event: &EventEnvelope) -> Result<Vec<RoomKey>, BroadcastError> {
        // Events are routed based on their aggregate
        match event.aggregate_type.as_str() {
            "session" => {
                // Session events go to session room
                Ok(vec![RoomKey::Session(SessionId::from_string(&event.aggregate_id))])
            }
            "cycle" | "component" => {
                // Cycle/component events go to their session's room
                // Need to look up session_id from cycle
                let session_id = self.lookup_session_for_event(event).await?;
                Ok(vec![RoomKey::Session(session_id)])
            }
            "conversation" => {
                // Conversation events go to component's session room
                let session_id = self.lookup_session_for_event(event).await?;
                Ok(vec![RoomKey::Session(session_id)])
            }
            "membership" => {
                // Membership events go to user room (affects all their sessions)
                let user_id = self.extract_user_id(event)?;
                Ok(vec![RoomKey::User(user_id)])
            }
            _ => {
                // Unknown aggregate type - log and skip
                tracing::warn!("Unknown aggregate type for broadcast: {}", event.aggregate_type);
                Ok(vec![])
            }
        }
    }

    fn event_to_message(&self, event: &EventEnvelope) -> WebSocketMessage {
        WebSocketMessage::Event {
            event_type: event.event_type.clone(),
            aggregate_type: event.aggregate_type.clone(),
            aggregate_id: event.aggregate_id.clone(),
            payload: event.payload.clone(),
            occurred_at: event.occurred_at,
        }
    }
}

/// Subscribe to event bus and forward to WebSocket bridge
#[async_trait]
impl EventHandler for WebSocketEventBridge {
    async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError> {
        match self.broadcast(&event).await {
            Ok(count) => {
                tracing::debug!(
                    "Broadcast event {} to {} connections",
                    event.event_type,
                    count
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to broadcast event: {}", e);
                // Don't fail the event processing - logging is best effort
                Ok(())
            }
        }
    }

    fn name(&self) -> &'static str {
        "WebSocketEventBridge"
    }
}
```

### Room Manager

```rust
use std::collections::{HashMap, HashSet};
use tokio::sync::broadcast;

/// Manages WebSocket connections organized by rooms
pub struct RoomManager {
    connections: HashMap<ConnectionId, ConnectionInfo>,
    rooms: HashMap<RoomKey, broadcast::Sender<WebSocketMessage>>,
    user_connections: HashMap<UserId, HashSet<ConnectionId>>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum RoomKey {
    /// Room for a specific session (all events for that session)
    Session(SessionId),
    /// Room for a user (cross-session events like membership changes)
    User(UserId),
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub user_id: UserId,
    pub session_id: Option<SessionId>,
    pub connected_at: Timestamp,
    pub rooms: HashSet<RoomKey>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            rooms: HashMap::new(),
            user_connections: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        user_id: UserId,
        session_id: Option<SessionId>,
    ) -> (ConnectionId, broadcast::Receiver<WebSocketMessage>) {
        let conn_id = ConnectionId::new();

        // Create room keys for this connection
        let mut room_keys = HashSet::new();
        room_keys.insert(RoomKey::User(user_id.clone()));
        if let Some(sid) = &session_id {
            room_keys.insert(RoomKey::Session(sid.clone()));
        }

        // Ensure rooms exist and get/create senders
        let (tx, rx) = self.get_or_create_room_channel(&room_keys);

        // Store connection info
        self.connections.insert(conn_id.clone(), ConnectionInfo {
            user_id: user_id.clone(),
            session_id,
            connected_at: Timestamp::now(),
            rooms: room_keys,
        });

        // Track user's connections
        self.user_connections
            .entry(user_id)
            .or_default()
            .insert(conn_id.clone());

        (conn_id, rx)
    }

    pub fn unregister(&mut self, conn_id: ConnectionId) {
        if let Some(info) = self.connections.remove(&conn_id) {
            // Remove from user connections
            if let Some(conns) = self.user_connections.get_mut(&info.user_id) {
                conns.remove(&conn_id);
                if conns.is_empty() {
                    self.user_connections.remove(&info.user_id);
                }
            }

            // Clean up empty rooms
            for room_key in info.rooms {
                if let Some(sender) = self.rooms.get(&room_key) {
                    if sender.receiver_count() == 0 {
                        self.rooms.remove(&room_key);
                    }
                }
            }
        }
    }

    pub fn get_sender(&self, room_key: &RoomKey) -> Option<&broadcast::Sender<WebSocketMessage>> {
        self.rooms.get(room_key)
    }

    fn get_or_create_room_channel(
        &mut self,
        room_keys: &HashSet<RoomKey>,
    ) -> (broadcast::Sender<WebSocketMessage>, broadcast::Receiver<WebSocketMessage>) {
        // For simplicity, we create a single channel that receives from all rooms
        // In production, might want per-room channels with fan-out
        let (tx, rx) = broadcast::channel(256);

        for key in room_keys {
            self.rooms.entry(key.clone()).or_insert_with(|| tx.clone());
        }

        (tx, rx)
    }
}
```

### WebSocket Message Types

```rust
use serde::{Deserialize, Serialize};

/// Messages sent over WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketMessage {
    /// Domain event notification
    Event {
        event_type: String,
        aggregate_type: String,
        aggregate_id: String,
        payload: serde_json::Value,
        occurred_at: Timestamp,
    },

    /// Heartbeat ping
    Ping {
        timestamp: Timestamp,
    },

    /// Heartbeat pong (response to ping)
    Pong {
        timestamp: Timestamp,
    },

    /// Connection established confirmation
    Connected {
        connection_id: String,
        server_time: Timestamp,
    },

    /// Error message
    Error {
        code: String,
        message: String,
    },

    /// Subscription confirmation
    Subscribed {
        room: String,
    },

    /// AI streaming chunk (for conversation)
    StreamChunk {
        conversation_id: String,
        content: String,
        done: bool,
    },
}

/// Messages received from client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Request to subscribe to a session's events
    Subscribe {
        session_id: String,
    },

    /// Request to unsubscribe from a session
    Unsubscribe {
        session_id: String,
    },

    /// Heartbeat response
    Pong {
        timestamp: Timestamp,
    },
}
```

---

## HTTP/WebSocket Endpoints

### Connection Endpoint

```rust
// backend/src/adapters/http/websocket/routes.rs

/// WebSocket upgrade endpoint
/// GET /api/ws
/// Requires: Authorization header with Bearer token
pub async fn websocket_handler(
    State(bridge): State<Arc<WebSocketEventBridge>>,
    Extension(user_id): Extension<UserId>,
    ws: WebSocketUpgrade,
    Query(params): Query<WebSocketParams>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, bridge, user_id, params.session_id))
}

#[derive(Debug, Deserialize)]
pub struct WebSocketParams {
    /// Optional: Subscribe to specific session on connect
    session_id: Option<String>,
}

async fn handle_socket(
    socket: WebSocket,
    bridge: Arc<WebSocketEventBridge>,
    user_id: UserId,
    initial_session: Option<String>,
) {
    let session_id = initial_session.map(SessionId::from_string);

    // Register connection
    let (conn_id, mut rx) = bridge.register_connection(user_id.clone(), session_id).await;

    // Split socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Send connection confirmation
    let connected_msg = WebSocketMessage::Connected {
        connection_id: conn_id.to_string(),
        server_time: Timestamp::now(),
    };
    sender.send(Message::Text(serde_json::to_string(&connected_msg).unwrap())).await.ok();

    // Spawn task to forward events to client
    let bridge_clone = bridge.clone();
    let conn_id_clone = conn_id.clone();
    let forward_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages from client
    let handle_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        match client_msg {
                            ClientMessage::Subscribe { session_id } => {
                                // TODO: Add to session room
                            }
                            ClientMessage::Unsubscribe { session_id } => {
                                // TODO: Remove from session room
                            }
                            ClientMessage::Pong { .. } => {
                                // Heartbeat acknowledged
                            }
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = forward_task => {}
        _ = handle_task => {}
    }

    // Cleanup
    bridge.unregister_connection(conn_id).await;
}
```

---

## Client-Side Implementation

### TypeScript WebSocket Client

```typescript
// frontend/src/lib/websocket/client.ts

import { writable, type Writable } from 'svelte/store';

export interface WebSocketEvent {
  type: 'event';
  event_type: string;
  aggregate_type: string;
  aggregate_id: string;
  payload: unknown;
  occurred_at: string;
}

export interface WebSocketClientOptions {
  url: string;
  token: string;
  sessionId?: string;
  onEvent?: (event: WebSocketEvent) => void;
  onConnect?: () => void;
  onDisconnect?: () => void;
  onError?: (error: Error) => void;
}

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private options: WebSocketClientOptions;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 10;
  private baseDelay = 1000;
  private heartbeatInterval: ReturnType<typeof setInterval> | null = null;
  private lastPong: number = Date.now();

  public connected: Writable<boolean> = writable(false);
  public connectionId: Writable<string | null> = writable(null);

  constructor(options: WebSocketClientOptions) {
    this.options = options;
  }

  connect(): void {
    const params = new URLSearchParams();
    if (this.options.sessionId) {
      params.set('session_id', this.options.sessionId);
    }

    const url = `${this.options.url}?${params.toString()}`;

    this.ws = new WebSocket(url);
    this.ws.onopen = this.handleOpen.bind(this);
    this.ws.onclose = this.handleClose.bind(this);
    this.ws.onerror = this.handleError.bind(this);
    this.ws.onmessage = this.handleMessage.bind(this);
  }

  disconnect(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }
    if (this.ws) {
      this.ws.close(1000, 'Client disconnect');
      this.ws = null;
    }
    this.connected.set(false);
  }

  subscribe(sessionId: string): void {
    this.send({ type: 'subscribe', session_id: sessionId });
  }

  unsubscribe(sessionId: string): void {
    this.send({ type: 'unsubscribe', session_id: sessionId });
  }

  private send(message: unknown): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  private handleOpen(): void {
    this.connected.set(true);
    this.reconnectAttempts = 0;
    this.startHeartbeat();
    this.options.onConnect?.();
  }

  private handleClose(event: CloseEvent): void {
    this.connected.set(false);
    this.connectionId.set(null);
    this.stopHeartbeat();
    this.options.onDisconnect?.();

    if (!event.wasClean && this.reconnectAttempts < this.maxReconnectAttempts) {
      this.scheduleReconnect();
    }
  }

  private handleError(event: Event): void {
    this.options.onError?.(new Error('WebSocket error'));
  }

  private handleMessage(event: MessageEvent): void {
    try {
      const message = JSON.parse(event.data);

      switch (message.type) {
        case 'connected':
          this.connectionId.set(message.connection_id);
          break;

        case 'event':
          this.options.onEvent?.(message as WebSocketEvent);
          break;

        case 'ping':
          this.send({ type: 'pong', timestamp: new Date().toISOString() });
          break;

        case 'pong':
          this.lastPong = Date.now();
          break;

        case 'error':
          console.error('WebSocket error:', message.message);
          break;
      }
    } catch (e) {
      console.error('Failed to parse WebSocket message:', e);
    }
  }

  private startHeartbeat(): void {
    this.heartbeatInterval = setInterval(() => {
      this.send({ type: 'ping', timestamp: new Date().toISOString() });

      // Check for stale connection
      if (Date.now() - this.lastPong > 30000) {
        console.warn('WebSocket connection stale, reconnecting...');
        this.ws?.close();
      }
    }, 15000);
  }

  private stopHeartbeat(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }
  }

  private scheduleReconnect(): void {
    const delay = this.baseDelay * Math.pow(2, this.reconnectAttempts);
    const jitter = Math.random() * 1000;
    this.reconnectAttempts++;

    console.log(`Reconnecting in ${delay + jitter}ms (attempt ${this.reconnectAttempts})`);

    setTimeout(() => {
      this.connect();
    }, delay + jitter);
  }
}
```

### Svelte Store Integration

```typescript
// frontend/src/lib/stores/realtime.ts

import { writable, derived } from 'svelte/store';
import { WebSocketClient, type WebSocketEvent } from '../websocket/client';
import { dashboardStore } from './dashboard';
import { sessionStore } from './session';
import { conversationStore } from './conversation';

// Global WebSocket client instance
let wsClient: WebSocketClient | null = null;

export const wsConnected = writable(false);

export function initializeRealtime(token: string, sessionId?: string): void {
  if (wsClient) {
    wsClient.disconnect();
  }

  wsClient = new WebSocketClient({
    url: `${import.meta.env.VITE_WS_URL}/api/ws`,
    token,
    sessionId,
    onEvent: handleEvent,
    onConnect: () => wsConnected.set(true),
    onDisconnect: () => wsConnected.set(false),
  });

  wsClient.connect();
}

export function disconnectRealtime(): void {
  wsClient?.disconnect();
  wsClient = null;
  wsConnected.set(false);
}

function handleEvent(event: WebSocketEvent): void {
  // Route events to appropriate stores
  switch (event.event_type) {
    // Session events
    case 'session.created':
    case 'session.archived':
    case 'session.renamed':
      sessionStore.handleEvent(event);
      break;

    // Cycle events
    case 'cycle.created':
    case 'cycle.branched':
    case 'cycle.archived':
    case 'cycle.completed':
      dashboardStore.handleCycleEvent(event);
      break;

    // Component events
    case 'component.started':
    case 'component.completed':
    case 'component.output_updated':
      dashboardStore.handleComponentEvent(event);
      break;

    // Conversation events
    case 'conversation.started':
    case 'conversation.message_sent':
    case 'conversation.data_extracted':
      conversationStore.handleEvent(event);
      break;

    // Analysis events
    case 'analysis.pugh_computed':
    case 'analysis.dq_computed':
      dashboardStore.handleAnalysisEvent(event);
      break;

    default:
      console.log('Unhandled event:', event.event_type);
  }
}
```

---

## Event Routing Rules

| Event Type | Target Room(s) | Notes |
|------------|----------------|-------|
| `session.*` | `Session:{session_id}` | Direct routing |
| `cycle.*` | `Session:{session_id}` | Look up via cycle→session |
| `component.*` | `Session:{session_id}` | Look up via component→cycle→session |
| `conversation.*` | `Session:{session_id}` | Look up via conversation→component→cycle→session |
| `membership.*` | `User:{user_id}` | Affects all user's sessions |
| `analysis.*` | `Session:{session_id}` | Look up via cycle→session |

---

## Connection Lifecycle

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      CONNECTION STATE MACHINE                                │
│                                                                              │
│   ┌───────────┐     auth success      ┌───────────┐                         │
│   │CONNECTING │─────────────────────►│ CONNECTED │                         │
│   └───────────┘                       └─────┬─────┘                         │
│         │                                   │                                │
│         │ auth failed                       │ server close /                │
│         │                                   │ heartbeat timeout             │
│         ▼                                   ▼                                │
│   ┌───────────┐                       ┌───────────┐                         │
│   │  CLOSED   │◄──────────────────────│RECONNECT- │                         │
│   │           │      max retries      │   ING     │                         │
│   └───────────┘                       └─────┬─────┘                         │
│                                             │                                │
│                                             │ reconnect success             │
│                                             ▼                                │
│                                       ┌───────────┐                         │
│                                       │ CONNECTED │                         │
│                                       └───────────┘                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Reconnection Strategy

| Attempt | Delay | Total Time |
|---------|-------|------------|
| 1 | 1s | 1s |
| 2 | 2s | 3s |
| 3 | 4s | 7s |
| 4 | 8s | 15s |
| 5 | 16s | 31s |
| 6+ | 32s (capped) | ... |

After 10 failed attempts, stop reconnecting and show error UI.

---

## Catch-Up on Reconnect

When a client reconnects after disconnection, they may have missed events. The strategy:

1. **Short Disconnection (<30s)**: Events are buffered in the room channel (256 capacity)
2. **Longer Disconnection**: Client fetches latest state via REST API

```typescript
// frontend/src/lib/stores/realtime.ts

async function onReconnect(): Promise<void> {
  const lastEventTime = localStorage.getItem('last_event_time');

  if (lastEventTime) {
    const disconnectedFor = Date.now() - new Date(lastEventTime).getTime();

    if (disconnectedFor > 30_000) {
      // Refetch full state
      await dashboardStore.refetch();
      await sessionStore.refetch();
    }
    // Else: trust that buffered events will catch us up
  }
}
```

---

## Security

### Authentication

WebSocket connections must be authenticated:

```rust
// Verify token before accepting WebSocket upgrade
pub async fn websocket_handler(
    State(auth): State<Arc<AuthService>>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, StatusCode> {
    let token = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = auth.verify_token(token).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user_id = UserId::from_string(&claims.sub);

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, user_id)))
}
```

### Authorization Filtering

Events are filtered before broadcast - users only receive events they're authorized to see:

```rust
async fn determine_targets(&self, event: &EventEnvelope) -> Result<Vec<RoomKey>, BroadcastError> {
    // Only send to rooms where user has access
    // Events for session X only go to users who own session X
    // This is enforced by room membership (user joins session room after authorization check)
}
```

---

## Configuration

```bash
# WebSocket Configuration
WS_MAX_CONNECTIONS_PER_USER=5
WS_HEARTBEAT_INTERVAL_SECS=15
WS_HEARTBEAT_TIMEOUT_SECS=30
WS_ROOM_CHANNEL_CAPACITY=256
WS_MAX_MESSAGE_SIZE_BYTES=65536
```

---

## Metrics

| Metric | Description |
|--------|-------------|
| `ws_connections_total` | Total WebSocket connections (counter) |
| `ws_connections_active` | Current active connections (gauge) |
| `ws_messages_sent_total` | Messages sent to clients (counter) |
| `ws_messages_received_total` | Messages received from clients (counter) |
| `ws_events_broadcast_total` | Events broadcast (counter, by event_type) |
| `ws_reconnect_total` | Client reconnection attempts (counter) |

---

## File Structure

```
backend/src/adapters/
├── websocket/
│   ├── mod.rs
│   ├── event_bridge.rs      # WebSocketEventBridge
│   ├── room_manager.rs      # RoomManager
│   ├── messages.rs          # WebSocketMessage, ClientMessage
│   ├── handler.rs           # HTTP upgrade handler
│   └── connection.rs        # Connection lifecycle

frontend/src/lib/
├── websocket/
│   ├── client.ts            # WebSocketClient class
│   ├── types.ts             # Message types
│   └── reconnect.ts         # Reconnection logic
└── stores/
    └── realtime.ts          # Store integration
```

---

## Related Documents

- **Event Infrastructure**: `features/foundation/event-infrastructure.md`
- **Event Flow Architecture**: `features/infrastructure/event-flow-architecture.md`
- **Dashboard Module**: `docs/modules/dashboard.md`
- **Conversation Module**: `docs/modules/conversation.md`

---

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Required - Bearer token in Authorization header |
| Authorization Model | Users receive only events for resources they own |
| Sensitive Data | Event payloads (Confidential), Connection state (Internal) |
| Rate Limiting | Required - see limits below |
| Audit Logging | Log connection events, subscription changes; never log message contents |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| Event payloads | Confidential | Only send to authorized users |
| Connection ID | Internal | Opaque, used for session tracking |
| Room membership | Internal | Do not expose which rooms exist |
| User session mappings | Internal | Do not expose to other users |

### Rate Limiting Requirements

| Limit | Value | Action on Exceed |
|-------|-------|------------------|
| Connections per user | 5 | Reject new connection with 429 |
| Connection attempts per minute | 10 per IP | Reject with 429, backoff required |
| Messages from client per minute | 60 | Drop messages, send warning |
| Subscribe requests per minute | 20 | Reject subscription, send error |
| Maximum message size | 64 KB | Close connection with error |

```rust
pub struct WebSocketRateLimits {
    pub max_connections_per_user: usize,         // 5
    pub connection_attempts_per_minute: usize,   // 10
    pub messages_per_minute: usize,              // 60
    pub subscribe_requests_per_minute: usize,    // 20
    pub max_message_size_bytes: usize,           // 65536
}

impl Default for WebSocketRateLimits {
    fn default() -> Self {
        Self {
            max_connections_per_user: 5,
            connection_attempts_per_minute: 10,
            messages_per_minute: 60,
            subscribe_requests_per_minute: 20,
            max_message_size_bytes: 65536,
        }
    }
}
```

### Origin Validation

WebSocket connections MUST validate the Origin header to prevent cross-site WebSocket hijacking:

```rust
pub async fn websocket_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, StatusCode> {
    // Validate Origin header
    let origin = headers.get("Origin")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::FORBIDDEN)?;

    if !state.config.allowed_origins.contains(&origin.to_string()) {
        tracing::warn!(origin = %origin, "Rejected WebSocket from unauthorized origin");
        return Err(StatusCode::FORBIDDEN);
    }

    // Continue with authentication...
}
```

### Authorization Filtering

Events sent to clients MUST be filtered by authorization. Users only receive events for resources they own:

```rust
impl WebSocketEventBridge {
    async fn broadcast(&self, event: &EventEnvelope) -> Result<usize, BroadcastError> {
        let rooms = self.rooms.read().await;

        // Determine which rooms should receive this event
        let targets = self.determine_targets(event).await?;

        for target in targets {
            // CRITICAL: Only send to rooms where users have access
            // Room membership is established during subscription with access check
            if let Some(sender) = rooms.get_sender(&target) {
                let message = self.event_to_message(event);
                sender.send(message).ok();
            }
        }

        Ok(sent_count)
    }
}

// Subscription requires authorization check
pub async fn handle_subscribe(
    &self,
    conn_id: &ConnectionId,
    session_id: SessionId,
    user_id: &UserId,
) -> Result<(), WebSocketError> {
    // CRITICAL: Verify user owns this session before adding to room
    self.access_checker
        .check_session_access(user_id, &session_id)
        .await
        .map_err(|_| WebSocketError::Unauthorized)?;

    // Only after authorization, add to room
    self.rooms.write().await.add_to_room(conn_id, RoomKey::Session(session_id));

    Ok(())
}
```

### Security Guidelines

1. **Authentication**: Every WebSocket connection MUST be authenticated before upgrade:
   - Extract Bearer token from Authorization header
   - Validate token with auth service
   - Reject with 401 if invalid

2. **Connection Cleanup**: Properly clean up connections on disconnect to prevent resource exhaustion:
   - Remove from all rooms
   - Cancel any pending operations
   - Log disconnection for audit

3. **Message Validation**: Validate all incoming messages:
   - Parse JSON safely with size limits
   - Reject unknown message types
   - Rate limit message frequency

4. **Heartbeat Security**: Heartbeat mechanism prevents connection hijacking:
   - Server sends ping every 15s
   - Client must respond within 30s
   - Stale connections are terminated

---

*Version: 1.0.0*
*Created: 2026-01-08*
