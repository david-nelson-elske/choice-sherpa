# Scaling Readiness Specification

**Version:** 1.0.0
**Created:** 2026-01-08
**Status:** Proposed
**Priority:** P1 (Implement before production)

> Architectural modifications to enable future horizontal scaling without over-engineering the MVP.

---

## Executive Summary

This document specifies changes to Choice Sherpa's architecture that prepare it for horizontal scaling while maintaining the current **modular monolith** deployment model. The goal is to establish **seams**—clean boundaries where future changes can be made with minimal disruption.

### Design Philosophy

| Principle | Application |
|-----------|-------------|
| **Build for now, design for later** | Implement patterns that work in single-instance but scale to multi-instance |
| **No premature distribution** | Keep all modules in-process; add distributed primitives via ports |
| **Explicit consistency boundaries** | Document where eventual consistency is acceptable |
| **Idempotency everywhere** | All operations should be safely retryable |

---

## Table of Contents

1. [Event Infrastructure](#1-event-infrastructure)
2. [Command Infrastructure](#2-command-infrastructure)
3. [Aggregate Refinements](#3-aggregate-refinements)
4. [WebSocket & Streaming](#4-websocket--streaming)
5. [Database Readiness](#5-database-readiness)
6. [External Service Resilience](#6-external-service-resilience)
7. [Observability Foundation](#7-observability-foundation)
8. [Migration Path](#8-migration-path)

---

## 1. Event Infrastructure

### Current State

- `EventPublisher` and `EventSubscriber` ports exist ✅
- `EventEnvelope` has correlation/causation IDs ✅
- `EventHandler` trait emphasizes idempotency ✅
- Only `InMemoryEventBus` adapter exists ⚠️
- No event persistence ❌
- No ordering guarantees ❌

### Scaling Problem

With multiple server instances:
1. Events published on Server A don't reach handlers on Server B
2. Events are lost if server crashes before handlers complete
3. No replay capability for rebuilding read models

### Solution: Transactional Outbox Pattern

```
┌─────────────────────────────────────────────────────────────┐
│                     Application Server                       │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────┐    ┌──────────┐    ┌──────────────────────┐  │
│  │ Command  │───►│ Domain   │───►│ Outbox Writer        │  │
│  │ Handler  │    │ Logic    │    │ (same transaction)   │  │
│  └──────────┘    └──────────┘    └──────────┬───────────┘  │
│                                              │              │
│                                              ▼              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                 PostgreSQL                            │  │
│  │  ┌─────────────┐     ┌─────────────────────────────┐ │  │
│  │  │ Domain      │     │ event_outbox                │ │  │
│  │  │ Tables      │     │ ───────────────────────────│ │  │
│  │  │             │     │ id, event_type, payload,   │ │  │
│  │  │             │     │ aggregate_id, created_at,  │ │  │
│  │  │             │     │ published_at (null=pending)│ │  │
│  │  └─────────────┘     └─────────────────────────────┘ │  │
│  └──────────────────────────────────────────────────────┘  │
│                                              │              │
│                                              ▼              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Outbox Relay Process                     │  │
│  │  1. Poll for unpublished events                      │  │
│  │  2. Publish to Redis                                 │  │
│  │  3. Mark as published                                │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                      Redis Pub/Sub                           │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Channel: domain-events                                │  │
│  │ Messages: EventEnvelope (JSON)                        │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                               │
          ┌────────────────────┼────────────────────┐
          ▼                    ▼                    ▼
    ┌──────────┐         ┌──────────┐         ┌──────────┐
    │ Server 1 │         │ Server 2 │         │ Server 3 │
    │ Handlers │         │ Handlers │         │ Handlers │
    └──────────┘         └──────────┘         └──────────┘
```

### Schema: Event Outbox Table

```sql
-- migrations/YYYYMMDD_create_event_outbox.sql

CREATE TABLE event_outbox (
    -- Identity
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sequence_num    BIGSERIAL NOT NULL,  -- Global ordering

    -- Event data (mirrors EventEnvelope)
    event_id        VARCHAR(255) NOT NULL UNIQUE,  -- For deduplication
    event_type      VARCHAR(255) NOT NULL,
    aggregate_type  VARCHAR(100) NOT NULL,
    aggregate_id    VARCHAR(255) NOT NULL,
    payload         JSONB NOT NULL,
    metadata        JSONB NOT NULL DEFAULT '{}',

    -- Lifecycle
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at    TIMESTAMPTZ,  -- NULL = pending, set when relayed

    -- Partitioning key (for future sharding)
    partition_key   VARCHAR(255) NOT NULL  -- Usually user_id or tenant_id
);

-- Index for relay polling (unpublished events in order)
CREATE INDEX idx_outbox_pending
    ON event_outbox (sequence_num)
    WHERE published_at IS NULL;

-- Index for aggregate event history
CREATE INDEX idx_outbox_aggregate
    ON event_outbox (aggregate_type, aggregate_id, sequence_num);

-- Index for event type subscriptions
CREATE INDEX idx_outbox_event_type
    ON event_outbox (event_type, sequence_num);

-- Partition key for future horizontal scaling
CREATE INDEX idx_outbox_partition
    ON event_outbox (partition_key, sequence_num);
```

### Schema: Event Processing Tracker

```sql
-- Track which events each handler has processed (idempotency)

CREATE TABLE event_processing (
    handler_name    VARCHAR(255) NOT NULL,
    event_id        VARCHAR(255) NOT NULL,
    processed_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (handler_name, event_id)
);

-- Auto-cleanup old records (events older than 7 days)
-- Implement via pg_cron or application-level cleanup
```

### New Port: OutboxWriter

```rust
// backend/src/ports/outbox_writer.rs

use async_trait::async_trait;

/// Port for writing events to the transactional outbox.
///
/// This is used INSTEAD of EventPublisher when you need
/// transactional guarantees (event written with domain changes).
#[async_trait]
pub trait OutboxWriter: Send + Sync {
    /// Write event to outbox within the current transaction.
    ///
    /// The event will be relayed to subscribers asynchronously
    /// after the transaction commits.
    async fn write(&self, event: EventEnvelope, partition_key: &str) -> Result<(), DomainError>;

    /// Write multiple events atomically.
    async fn write_all(&self, events: Vec<EventEnvelope>, partition_key: &str) -> Result<(), DomainError>;
}

/// Combined trait for transactional event operations.
///
/// Implementations must ensure events are written in the same
/// database transaction as domain state changes.
#[async_trait]
pub trait TransactionalOutbox: OutboxWriter {
    /// Begin a new transaction context.
    async fn begin(&self) -> Result<TransactionContext, DomainError>;
}
```

### New Adapter: PostgresOutboxWriter

```rust
// backend/src/adapters/postgres/outbox_writer.rs

pub struct PostgresOutboxWriter {
    pool: PgPool,
}

#[async_trait]
impl OutboxWriter for PostgresOutboxWriter {
    async fn write(&self, event: EventEnvelope, partition_key: &str) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
            INSERT INTO event_outbox
                (event_id, event_type, aggregate_type, aggregate_id,
                 payload, metadata, partition_key)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            event.event_id.as_str(),
            event.event_type,
            event.aggregate_type,
            event.aggregate_id,
            serde_json::to_value(&event.payload)?,
            serde_json::to_value(&event.metadata)?,
            partition_key
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::DatabaseError, e.to_string()))?;

        Ok(())
    }

    async fn write_all(&self, events: Vec<EventEnvelope>, partition_key: &str) -> Result<(), DomainError> {
        // Batch insert for efficiency
        // ...
    }
}
```

### New Adapter: RedisEventPublisher

```rust
// backend/src/adapters/redis/event_publisher.rs

use redis::aio::MultiplexedConnection;

pub struct RedisEventPublisher {
    conn: MultiplexedConnection,
    channel: String,
}

impl RedisEventPublisher {
    pub fn new(conn: MultiplexedConnection) -> Self {
        Self {
            conn,
            channel: "domain-events".to_string(),
        }
    }
}

#[async_trait]
impl EventPublisher for RedisEventPublisher {
    async fn publish(&self, event: EventEnvelope) -> Result<(), DomainError> {
        let payload = serde_json::to_string(&event)
            .map_err(|e| DomainError::new(ErrorCode::InternalError, e.to_string()))?;

        redis::cmd("PUBLISH")
            .arg(&self.channel)
            .arg(payload)
            .query_async(&mut self.conn.clone())
            .await
            .map_err(|e| DomainError::new(ErrorCode::ExternalServiceError, e.to_string()))?;

        Ok(())
    }

    async fn publish_all(&self, events: Vec<EventEnvelope>) -> Result<(), DomainError> {
        // Use pipeline for efficiency
        let mut pipe = redis::pipe();
        for event in &events {
            let payload = serde_json::to_string(event)?;
            pipe.publish(&self.channel, payload);
        }
        pipe.query_async(&mut self.conn.clone()).await?;
        Ok(())
    }
}
```

### Outbox Relay Service

```rust
// backend/src/infrastructure/outbox_relay.rs

/// Background service that relays events from outbox to Redis.
///
/// Runs as a separate task, polling for unpublished events.
pub struct OutboxRelay {
    pool: PgPool,
    publisher: Arc<dyn EventPublisher>,
    batch_size: i32,
    poll_interval: Duration,
}

impl OutboxRelay {
    pub async fn run(&self, shutdown: CancellationToken) {
        loop {
            tokio::select! {
                _ = shutdown.cancelled() => break,
                _ = tokio::time::sleep(self.poll_interval) => {
                    if let Err(e) = self.relay_batch().await {
                        tracing::error!("Outbox relay error: {}", e);
                    }
                }
            }
        }
    }

    async fn relay_batch(&self) -> Result<(), DomainError> {
        // 1. SELECT ... FOR UPDATE SKIP LOCKED (avoid contention)
        // 2. Publish to Redis
        // 3. UPDATE published_at
        // All in a transaction

        let mut tx = self.pool.begin().await?;

        let events = sqlx::query_as!(
            OutboxRow,
            r#"
            SELECT id, event_id, event_type, aggregate_type, aggregate_id,
                   payload, metadata, created_at
            FROM event_outbox
            WHERE published_at IS NULL
            ORDER BY sequence_num
            LIMIT $1
            FOR UPDATE SKIP LOCKED
            "#,
            self.batch_size
        )
        .fetch_all(&mut *tx)
        .await?;

        if events.is_empty() {
            return Ok(());
        }

        let envelopes: Vec<EventEnvelope> = events
            .iter()
            .map(|row| row.to_envelope())
            .collect();

        self.publisher.publish_all(envelopes).await?;

        let ids: Vec<Uuid> = events.iter().map(|e| e.id).collect();
        sqlx::query!(
            "UPDATE event_outbox SET published_at = NOW() WHERE id = ANY($1)",
            &ids
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        tracing::info!("Relayed {} events", events.len());
        Ok(())
    }
}
```

### Event Handler Idempotency

```rust
// backend/src/infrastructure/idempotent_handler.rs

/// Wrapper that ensures handler idempotency via database tracking.
pub struct IdempotentHandler<H: EventHandler> {
    inner: H,
    pool: PgPool,
}

#[async_trait]
impl<H: EventHandler> EventHandler for IdempotentHandler<H> {
    async fn handle(&self, event: EventEnvelope) -> Result<(), DomainError> {
        let event_id = event.event_id.as_str();
        let handler_name = self.inner.name();

        // Try to insert processing record
        let result = sqlx::query!(
            r#"
            INSERT INTO event_processing (handler_name, event_id)
            VALUES ($1, $2)
            ON CONFLICT (handler_name, event_id) DO NOTHING
            "#,
            handler_name,
            event_id
        )
        .execute(&self.pool)
        .await?;

        // If no rows inserted, already processed
        if result.rows_affected() == 0 {
            tracing::debug!(
                "Event {} already processed by {}, skipping",
                event_id,
                handler_name
            );
            return Ok(());
        }

        // Process the event
        self.inner.handle(event).await
    }

    fn name(&self) -> &'static str {
        self.inner.name()
    }
}
```

---

## 2. Command Infrastructure

### Current State

- Command handlers exist but lack idempotency keys
- No standardized request context propagation
- Retry behavior is undefined

### Scaling Problem

1. Network timeouts cause retries → duplicate operations
2. Load balancer failover during processing → lost context
3. No way to correlate distributed operations

### Solution: Idempotency Keys & Request Context

### Enhanced Command Structure

```rust
// backend/src/application/command.rs

/// Request context that flows through all operations.
///
/// Automatically propagated via tower middleware.
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Unique request ID (generated at edge)
    pub request_id: String,

    /// User making the request
    pub user_id: UserId,

    /// Correlation ID for distributed tracing
    pub correlation_id: String,

    /// Optional idempotency key (client-provided)
    pub idempotency_key: Option<String>,

    /// Request timestamp
    pub timestamp: Timestamp,
}

impl RequestContext {
    /// Create context from HTTP headers.
    pub fn from_headers(headers: &HeaderMap, user_id: UserId) -> Self {
        let request_id = headers
            .get("X-Request-ID")
            .and_then(|v| v.to_str().ok())
            .map(String::from)
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let correlation_id = headers
            .get("X-Correlation-ID")
            .and_then(|v| v.to_str().ok())
            .map(String::from)
            .unwrap_or_else(|| request_id.clone());

        let idempotency_key = headers
            .get("Idempotency-Key")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        Self {
            request_id,
            user_id,
            correlation_id,
            idempotency_key,
            timestamp: Timestamp::now(),
        }
    }

    /// Propagate to event metadata.
    pub fn to_event_metadata(&self) -> EventMetadata {
        EventMetadata {
            correlation_id: Some(self.correlation_id.clone()),
            causation_id: None,
            user_id: Some(self.user_id.to_string()),
            trace_id: Some(self.request_id.clone()),
        }
    }
}
```

### Idempotency Storage

```sql
-- migrations/YYYYMMDD_create_idempotency.sql

CREATE TABLE idempotency_keys (
    key             VARCHAR(255) PRIMARY KEY,
    user_id         VARCHAR(255) NOT NULL,

    -- Request info
    request_path    VARCHAR(500) NOT NULL,
    request_hash    VARCHAR(64) NOT NULL,  -- SHA256 of request body

    -- Response info
    response_status SMALLINT NOT NULL,
    response_body   JSONB,

    -- Lifecycle
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '24 hours'
);

-- Auto-cleanup expired keys
CREATE INDEX idx_idempotency_expires ON idempotency_keys (expires_at);
```

### Idempotency Middleware

```rust
// backend/src/adapters/http/middleware/idempotency.rs

/// Middleware that handles idempotent requests.
///
/// If Idempotency-Key header is present:
/// 1. Check if we've seen this key before
/// 2. If yes, return cached response
/// 3. If no, process request and cache response
pub async fn idempotency_middleware(
    State(pool): State<PgPool>,
    req: Request,
    next: Next,
) -> Response {
    let idempotency_key = req
        .headers()
        .get("Idempotency-Key")
        .and_then(|v| v.to_str().ok());

    let Some(key) = idempotency_key else {
        // No idempotency key, proceed normally
        return next.run(req).await;
    };

    // Check for existing response
    if let Some(cached) = get_cached_response(&pool, key).await {
        return cached;
    }

    // Process request
    let response = next.run(req).await;

    // Cache response (only for success/client errors, not server errors)
    if response.status().is_success() || response.status().is_client_error() {
        cache_response(&pool, key, &response).await;
    }

    response
}
```

---

## 3. Aggregate Refinements

### Current State

- Cycle aggregate owns all 9 components (embedded)
- Single version counter for entire aggregate
- Optimistic locking at aggregate level

### Scaling Problem

```
Scenario: Two users editing different components simultaneously

Timeline:
T1: Server A loads Cycle (version 5)
T2: Server B loads Cycle (version 5)
T3: Server A updates IssueRaising, saves (version 6) ✅
T4: Server B updates Objectives, saves (version 6) ❌ CONFLICT!
```

Even though different components were modified, optimistic locking fails.

### Solution: Component-Level Versioning

```sql
-- Modify components table to have independent versions

ALTER TABLE components
    ADD COLUMN version INTEGER NOT NULL DEFAULT 1;

-- Update pattern becomes:
UPDATE components
SET structured_data = $1,
    version = version + 1,
    updated_at = NOW()
WHERE cycle_id = $2
  AND component_type = $3
  AND version = $4
RETURNING version;
```

### Updated Cycle Aggregate

```rust
// backend/src/domain/cycle/cycle.rs

pub struct Cycle {
    id: CycleId,
    session_id: SessionId,
    // ... other fields

    /// Component versions for fine-grained locking
    component_versions: HashMap<ComponentType, u32>,
}

impl Cycle {
    /// Update a single component with optimistic locking.
    ///
    /// Only the modified component's version is checked/incremented.
    pub fn update_component(
        &mut self,
        comp_type: ComponentType,
        expected_version: u32,
        output: serde_json::Value,
    ) -> Result<u32, DomainError> {
        let current = self.component_versions.get(&comp_type).copied().unwrap_or(1);

        if current != expected_version {
            return Err(DomainError::new(
                ErrorCode::ConcurrencyConflict,
                format!(
                    "Component {} was modified (expected version {}, found {})",
                    comp_type, expected_version, current
                )
            ));
        }

        // Update component...
        let new_version = current + 1;
        self.component_versions.insert(comp_type, new_version);

        Ok(new_version)
    }
}
```

### Component View with Version

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentView {
    pub id: ComponentId,
    pub cycle_id: CycleId,
    pub component_type: ComponentType,
    pub status: ComponentStatus,
    pub structured_output: serde_json::Value,
    pub version: u32,  // Add version for client-side tracking
    pub updated_at: Timestamp,
}
```

### Client-Side Optimistic Updates

```typescript
// frontend/src/lib/cycle/use-component.ts

export function useComponent(cycleId: string, componentType: ComponentType) {
    const [component, setComponent] = useState<ComponentView | null>(null);

    const updateComponent = async (output: any) => {
        if (!component) return;

        try {
            const result = await api.updateComponent(cycleId, componentType, {
                output,
                expected_version: component.version,  // Send version
            });

            setComponent(prev => ({
                ...prev!,
                structured_output: output,
                version: result.version,  // Update local version
            }));
        } catch (e) {
            if (e.code === 'CONCURRENCY_CONFLICT') {
                // Refresh and show merge dialog
                const fresh = await api.getComponent(cycleId, componentType);
                showConflictDialog(component, fresh);
            }
            throw e;
        }
    };

    return { component, updateComponent };
}
```

---

## 4. WebSocket & Streaming

### Current State

- WebSocket planned for AI streaming
- No connection registry
- No reconnection protocol

### Scaling Problem

```
User → Load Balancer → Server A (has WebSocket)
User → Load Balancer → Server B (REST request)
  ↓
Server B wants to push update to user's WebSocket
  ↓
Server B doesn't know Server A has the connection!
```

### Solution: Connection Registry + Redis Pub/Sub

```
┌─────────────────────────────────────────────────────────────┐
│                        Redis                                 │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Hash: ws:connections                                   │ │
│  │   user:123 → server-a:ws-conn-456                     │ │
│  │   user:789 → server-b:ws-conn-012                     │ │
│  └────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ PubSub: ws:user:123                                    │ │
│  │   (messages destined for user 123)                    │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Connection Registry Port

```rust
// backend/src/ports/connection_registry.rs

use async_trait::async_trait;

/// Registry for tracking active WebSocket connections.
///
/// Enables sending messages to users regardless of which
/// server instance holds their connection.
#[async_trait]
pub trait ConnectionRegistry: Send + Sync {
    /// Register a connection for a user.
    async fn register(
        &self,
        user_id: &UserId,
        connection_id: &str,
        server_id: &str,
    ) -> Result<(), DomainError>;

    /// Unregister a connection.
    async fn unregister(
        &self,
        user_id: &UserId,
        connection_id: &str,
    ) -> Result<(), DomainError>;

    /// Get connection info for a user.
    async fn get_connection(
        &self,
        user_id: &UserId,
    ) -> Result<Option<ConnectionInfo>, DomainError>;

    /// Send message to a user (via their connection).
    async fn send_to_user(
        &self,
        user_id: &UserId,
        message: WebSocketMessage,
    ) -> Result<(), DomainError>;
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub connection_id: String,
    pub server_id: String,
    pub connected_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub message_type: String,
    pub payload: serde_json::Value,
}
```

### Redis Connection Registry

```rust
// backend/src/adapters/redis/connection_registry.rs

pub struct RedisConnectionRegistry {
    conn: MultiplexedConnection,
    server_id: String,
    ttl: Duration,
}

#[async_trait]
impl ConnectionRegistry for RedisConnectionRegistry {
    async fn register(
        &self,
        user_id: &UserId,
        connection_id: &str,
        server_id: &str,
    ) -> Result<(), DomainError> {
        let key = format!("ws:conn:{}", user_id);
        let value = serde_json::json!({
            "connection_id": connection_id,
            "server_id": server_id,
            "connected_at": Utc::now().to_rfc3339()
        });

        redis::cmd("SET")
            .arg(&key)
            .arg(value.to_string())
            .arg("EX")
            .arg(self.ttl.as_secs())
            .query_async(&mut self.conn.clone())
            .await?;

        Ok(())
    }

    async fn send_to_user(
        &self,
        user_id: &UserId,
        message: WebSocketMessage,
    ) -> Result<(), DomainError> {
        // Publish to user-specific channel
        // The server holding the connection will receive and forward
        let channel = format!("ws:user:{}", user_id);
        let payload = serde_json::to_string(&message)?;

        redis::cmd("PUBLISH")
            .arg(&channel)
            .arg(payload)
            .query_async(&mut self.conn.clone())
            .await?;

        Ok(())
    }
}
```

### WebSocket Handler with Registry

```rust
// backend/src/adapters/http/websocket.rs

pub async fn handle_websocket(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, user))
}

async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    user: AuthenticatedUser,
) {
    let connection_id = Uuid::new_v4().to_string();

    // Register connection
    state.connection_registry
        .register(&user.id, &connection_id, &state.server_id)
        .await
        .unwrap();

    // Subscribe to user's message channel
    let mut pubsub = state.redis.get_async_pubsub().await.unwrap();
    pubsub.subscribe(format!("ws:user:{}", user.id)).await.unwrap();

    let (mut sender, mut receiver) = socket.split();

    // Handle incoming messages and Redis pubsub
    loop {
        tokio::select! {
            // Message from client
            Some(msg) = receiver.next() => {
                handle_client_message(msg, &state, &user).await;
            }

            // Message from Redis (cross-server)
            Some(msg) = pubsub.on_message().next() => {
                let payload: String = msg.get_payload().unwrap();
                sender.send(Message::Text(payload)).await.unwrap();
            }
        }
    }

    // Cleanup on disconnect
    state.connection_registry
        .unregister(&user.id, &connection_id)
        .await
        .unwrap();
}
```

### Reconnection Protocol

```typescript
// frontend/src/lib/websocket/reconnecting-socket.ts

export class ReconnectingWebSocket {
    private socket: WebSocket | null = null;
    private reconnectAttempts = 0;
    private maxReconnectAttempts = 10;
    private baseDelay = 1000;
    private lastEventId: string | null = null;

    async connect(url: string) {
        // Include last event ID for resumption
        const connectUrl = this.lastEventId
            ? `${url}?lastEventId=${this.lastEventId}`
            : url;

        this.socket = new WebSocket(connectUrl);

        this.socket.onmessage = (event) => {
            const data = JSON.parse(event.data);
            this.lastEventId = data.eventId;
            this.onMessage(data);
        };

        this.socket.onclose = () => {
            this.scheduleReconnect();
        };
    }

    private scheduleReconnect() {
        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            this.onMaxRetriesExceeded();
            return;
        }

        // Exponential backoff with jitter
        const delay = this.baseDelay * Math.pow(2, this.reconnectAttempts)
            + Math.random() * 1000;

        this.reconnectAttempts++;

        setTimeout(() => this.connect(this.url), delay);
    }
}
```

---

## 5. Database Readiness

### Current State

- Single PostgreSQL instance
- CQRS separation (Repository vs Reader ports) ✅
- No explicit sharding keys

### Future Scaling Path

```
Phase 1 (Current): Single writer, single reader
Phase 2: Single writer, multiple read replicas
Phase 3: Horizontal sharding by user_id/tenant_id
```

### Preparation: Partition Keys

Add `partition_key` to all tables that may need sharding:

```sql
-- Add to existing tables
ALTER TABLE sessions ADD COLUMN partition_key VARCHAR(255);
UPDATE sessions SET partition_key = user_id;
ALTER TABLE sessions ALTER COLUMN partition_key SET NOT NULL;

ALTER TABLE cycles ADD COLUMN partition_key VARCHAR(255);
UPDATE cycles SET partition_key = (SELECT user_id FROM sessions WHERE id = cycles.session_id);
ALTER TABLE cycles ALTER COLUMN partition_key SET NOT NULL;

-- Add index for future sharding
CREATE INDEX idx_sessions_partition ON sessions (partition_key);
CREATE INDEX idx_cycles_partition ON cycles (partition_key);
```

### Read Replica Configuration

```rust
// backend/src/infrastructure/database.rs

pub struct DatabasePools {
    /// Writer pool - primary database
    pub writer: PgPool,

    /// Reader pool - read replicas (or primary if no replicas)
    pub reader: PgPool,
}

impl DatabasePools {
    pub async fn from_config(config: &DatabaseConfig) -> Result<Self, Error> {
        let writer = PgPoolOptions::new()
            .max_connections(config.writer_max_connections)
            .connect(&config.writer_url)
            .await?;

        let reader_url = config.reader_url.as_ref()
            .unwrap_or(&config.writer_url);

        let reader = PgPoolOptions::new()
            .max_connections(config.reader_max_connections)
            .connect(reader_url)
            .await?;

        Ok(Self { writer, reader })
    }
}
```

### Repository/Reader Split

```rust
// Repositories use writer pool
pub struct PostgresSessionRepository {
    pool: PgPool,  // writer pool
}

// Readers use reader pool
pub struct PostgresSessionReader {
    pool: PgPool,  // reader pool
}

// Wire up in main.rs
let repos = Repositories {
    session: PostgresSessionRepository::new(db.writer.clone()),
    cycle: PostgresCycleRepository::new(db.writer.clone()),
};

let readers = Readers {
    session: PostgresSessionReader::new(db.reader.clone()),
    cycle: PostgresCycleReader::new(db.reader.clone()),
    dashboard: PostgresDashboardReader::new(db.reader.clone()),
};
```

---

## 6. External Service Resilience

### Current State

- `AIProvider` port exists
- No circuit breaker
- No retry policy
- No fallback

### Solution: Resilience Patterns

### Circuit Breaker Port

```rust
// backend/src/ports/circuit_breaker.rs

use async_trait::async_trait;

/// State of a circuit breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,      // Normal operation
    Open,        // Failing, reject requests
    HalfOpen,    // Testing if service recovered
}

/// Circuit breaker for external service calls.
#[async_trait]
pub trait CircuitBreaker: Send + Sync {
    /// Check if circuit allows the call.
    fn allow(&self) -> bool;

    /// Record a successful call.
    fn record_success(&self);

    /// Record a failed call.
    fn record_failure(&self);

    /// Get current state.
    fn state(&self) -> CircuitState;
}
```

### Resilient AI Provider Wrapper

```rust
// backend/src/adapters/ai/resilient_provider.rs

/// Wrapper that adds resilience to any AIProvider.
pub struct ResilientAIProvider<P: AIProvider> {
    inner: P,
    circuit_breaker: Arc<dyn CircuitBreaker>,
    retry_policy: RetryPolicy,
    fallback: Option<Arc<dyn AIProvider>>,
}

impl<P: AIProvider> ResilientAIProvider<P> {
    pub fn new(provider: P) -> Self {
        Self {
            inner: provider,
            circuit_breaker: Arc::new(DefaultCircuitBreaker::new(
                5,                          // failure_threshold
                Duration::from_secs(30),    // reset_timeout
            )),
            retry_policy: RetryPolicy::exponential(3, Duration::from_millis(100)),
            fallback: None,
        }
    }

    pub fn with_fallback(mut self, fallback: impl AIProvider + 'static) -> Self {
        self.fallback = Some(Arc::new(fallback));
        self
    }
}

#[async_trait]
impl<P: AIProvider> AIProvider for ResilientAIProvider<P> {
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, DomainError> {
        // Check circuit breaker
        if !self.circuit_breaker.allow() {
            if let Some(fallback) = &self.fallback {
                return fallback.complete(req).await;
            }
            return Err(DomainError::new(
                ErrorCode::ExternalServiceError,
                "AI provider circuit open"
            ));
        }

        // Retry with exponential backoff
        let mut attempts = 0;
        loop {
            match self.inner.complete(req.clone()).await {
                Ok(response) => {
                    self.circuit_breaker.record_success();
                    return Ok(response);
                }
                Err(e) if self.retry_policy.should_retry(attempts, &e) => {
                    attempts += 1;
                    let delay = self.retry_policy.delay(attempts);
                    tokio::time::sleep(delay).await;
                }
                Err(e) => {
                    self.circuit_breaker.record_failure();

                    // Try fallback if available
                    if let Some(fallback) = &self.fallback {
                        return fallback.complete(req).await;
                    }

                    return Err(e);
                }
            }
        }
    }
}
```

### Retry Policy

```rust
// backend/src/infrastructure/retry.rs

pub struct RetryPolicy {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
    jitter: bool,
}

impl RetryPolicy {
    pub fn exponential(max_attempts: u32, base_delay: Duration) -> Self {
        Self {
            max_attempts,
            base_delay,
            max_delay: Duration::from_secs(30),
            jitter: true,
        }
    }

    pub fn should_retry(&self, attempts: u32, error: &DomainError) -> bool {
        attempts < self.max_attempts && Self::is_retryable(error)
    }

    pub fn delay(&self, attempt: u32) -> Duration {
        let base = self.base_delay.as_millis() as u64;
        let exp_delay = base * 2u64.pow(attempt);
        let capped = exp_delay.min(self.max_delay.as_millis() as u64);

        let jitter = if self.jitter {
            (rand::random::<f64>() * 0.3 * capped as f64) as u64
        } else {
            0
        };

        Duration::from_millis(capped + jitter)
    }

    fn is_retryable(error: &DomainError) -> bool {
        matches!(
            error.code,
            ErrorCode::ExternalServiceError | ErrorCode::RateLimitExceeded
        )
    }
}
```

---

## 7. Observability Foundation

### Distributed Tracing

Ensure all operations propagate trace context:

```rust
// backend/src/infrastructure/tracing.rs

use opentelemetry::trace::{TraceContextExt, Tracer};
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Extract trace context from incoming request.
pub fn extract_trace_context(headers: &HeaderMap) -> Option<opentelemetry::Context> {
    let extractor = opentelemetry_http::HeaderExtractor(headers);
    let ctx = opentelemetry::global::get_text_map_propagator(|prop| {
        prop.extract(&extractor)
    });

    if ctx.span().span_context().is_valid() {
        Some(ctx)
    } else {
        None
    }
}

/// Inject trace context into outgoing request.
pub fn inject_trace_context(headers: &mut HeaderMap) {
    let ctx = tracing::Span::current().context();
    opentelemetry::global::get_text_map_propagator(|prop| {
        let mut injector = opentelemetry_http::HeaderInjector(headers);
        prop.inject_context(&ctx, &mut injector);
    });
}
```

### Metrics

```rust
// backend/src/infrastructure/metrics.rs

use prometheus::{
    register_histogram_vec, register_int_counter_vec,
    HistogramVec, IntCounterVec,
};

lazy_static! {
    pub static ref HTTP_REQUESTS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "http_requests_total",
        "Total HTTP requests",
        &["method", "path", "status"]
    ).unwrap();

    pub static ref HTTP_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request duration",
        &["method", "path"],
        vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    ).unwrap();

    pub static ref EVENT_PUBLISHED_TOTAL: IntCounterVec = register_int_counter_vec!(
        "event_published_total",
        "Events published",
        &["event_type"]
    ).unwrap();

    pub static ref EVENT_HANDLER_DURATION: HistogramVec = register_histogram_vec!(
        "event_handler_duration_seconds",
        "Event handler duration",
        &["handler", "event_type"]
    ).unwrap();

    pub static ref AI_PROVIDER_REQUESTS: IntCounterVec = register_int_counter_vec!(
        "ai_provider_requests_total",
        "AI provider requests",
        &["provider", "status"]
    ).unwrap();

    pub static ref CIRCUIT_BREAKER_STATE: IntCounterVec = register_int_counter_vec!(
        "circuit_breaker_state_changes_total",
        "Circuit breaker state changes",
        &["name", "from_state", "to_state"]
    ).unwrap();
}
```

---

## 8. Migration Path

### Phase 1: Foundation (MVP)

Implement without breaking changes:

| Component | Current | Action |
|-----------|---------|--------|
| Event publishing | In-memory only | Add outbox table, keep in-memory for now |
| Commands | No idempotency | Add RequestContext, optional idempotency key |
| Components | Aggregate versioning | Add component version column |
| WebSocket | Not implemented | Design with registry from start |
| Database | Single pool | Add partition_key columns |

### Phase 2: Pre-Scaling (Before multi-instance)

Enable distributed operation:

| Component | Action |
|-----------|--------|
| Event publishing | Deploy outbox relay + Redis pub/sub |
| Connection registry | Deploy Redis adapter |
| Database | Configure read replicas |
| Observability | Enable distributed tracing |

### Phase 3: Horizontal Scaling

Full horizontal capability:

| Component | Action |
|-----------|--------|
| Load balancer | Deploy with health checks |
| Multiple instances | Run N API servers |
| Database | Consider Citus or sharding |
| Events | Consider dedicated event store |

---

## Summary: Specification Changes

### New Files to Create

| File | Purpose |
|------|---------|
| `backend/src/ports/outbox_writer.rs` | Transactional outbox port |
| `backend/src/ports/connection_registry.rs` | WebSocket connection tracking |
| `backend/src/ports/circuit_breaker.rs` | External service resilience |
| `backend/src/adapters/postgres/outbox_writer.rs` | Postgres outbox implementation |
| `backend/src/adapters/redis/event_publisher.rs` | Redis pub/sub events |
| `backend/src/adapters/redis/connection_registry.rs` | Redis connection tracking |
| `backend/src/infrastructure/outbox_relay.rs` | Background event relay |
| `backend/src/infrastructure/idempotent_handler.rs` | Handler idempotency wrapper |
| `backend/src/adapters/ai/resilient_provider.rs` | Circuit breaker + retry |
| `migrations/YYYYMMDD_create_event_outbox.sql` | Outbox table |
| `migrations/YYYYMMDD_create_idempotency.sql` | Idempotency keys |
| `migrations/YYYYMMDD_add_partition_keys.sql` | Sharding preparation |
| `migrations/YYYYMMDD_add_component_versions.sql` | Fine-grained locking |

### Files to Modify

| File | Change |
|------|--------|
| `docs/architecture/SYSTEM-ARCHITECTURE.md` | Add scaling section reference |
| `docs/architecture/consistency-patterns.md` | Add idempotency patterns |
| `backend/src/domain/cycle/cycle.rs` | Add component-level versioning |
| `backend/src/ports/mod.rs` | Export new ports |
| `backend/src/adapters/mod.rs` | Export new adapters |

### Frontend Changes

| File | Change |
|------|--------|
| `frontend/src/lib/api/client.ts` | Add idempotency key header support |
| `frontend/src/lib/websocket/` | Implement reconnecting socket |
| `frontend/src/lib/cycle/use-component.ts` | Add version tracking |

---

## Decision Log

| Decision | Rationale | Trade-offs |
|----------|-----------|------------|
| Outbox pattern over direct publish | Guarantees event delivery with domain transaction | Slight latency (polling interval) |
| Redis pub/sub over dedicated broker | Already in stack, sufficient for scale | Less features than Kafka/RabbitMQ |
| Component-level versioning | Reduces contention without major refactor | Slightly more complex client code |
| Partition keys now | Minimal cost, enables future sharding | Unused until Phase 3 |
| Circuit breaker wrapper | Non-invasive, composable | Per-request overhead |

---

*Version: 1.0.0*
*Created: 2026-01-08*
*Status: Proposed*
