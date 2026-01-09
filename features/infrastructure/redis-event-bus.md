# Feature: Redis Event Bus

**Module:** adapters/events
**Type:** Infrastructure Adapter
**Priority:** P2
**Phase:** 7 of Full PrOACT Journey Integration
**Depends On:** features/foundation/event-infrastructure.md

> Production-grade event bus using Redis Streams for reliable, persistent event delivery with consumer groups and dead-letter queue support.

---

## Problem Statement

The `InMemoryEventBus` works perfectly for testing but has limitations for production:
- Events lost on server restart
- Single process only (no horizontal scaling)
- No replay capability
- No consumer groups (competing consumers)
- No dead-letter queue for failed events

### Current State

- In-memory event bus for development/testing
- No persistence
- Single-node only

### Desired State

- Redis Streams-based event bus for production
- Persistent events with configurable retention
- Consumer groups for load distribution
- Dead-letter queue for failed events
- Replay capability for recovery

---

## Redis Streams Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Redis Server                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Stream: choice-sherpa:events                                       │
│   ┌────────────────────────────────────────────────────────────┐    │
│   │ 1704614400000-0: {type: "session.created", data: "..."}    │    │
│   │ 1704614400001-0: {type: "cycle.created", data: "..."}      │    │
│   │ 1704614400002-0: {type: "component.started", data: "..."}  │    │
│   │ ...                                                         │    │
│   └────────────────────────────────────────────────────────────┘    │
│                                │                                     │
│                                ▼                                     │
│   Consumer Groups:                                                   │
│   ┌────────────────────────────────────────────────────────────┐    │
│   │ Group: dashboard-handlers                                   │    │
│   │   - consumer-1 (server-a)                                   │    │
│   │   - consumer-2 (server-b)                                   │    │
│   │   - PEL: [pending entries...]                               │    │
│   └────────────────────────────────────────────────────────────┘    │
│   ┌────────────────────────────────────────────────────────────┐    │
│   │ Group: analysis-handlers                                    │    │
│   │   - consumer-1 (server-a)                                   │    │
│   └────────────────────────────────────────────────────────────┘    │
│                                                                      │
│   Stream: choice-sherpa:events:dlq (Dead Letter Queue)              │
│   ┌────────────────────────────────────────────────────────────┐    │
│   │ Failed events with error metadata                           │    │
│   └────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Technical Design

### RedisEventBus Structure

```rust
// backend/src/adapters/events/redis.rs

use redis::{AsyncCommands, Client, aio::ConnectionManager};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Redis Streams-based event bus for production
pub struct RedisEventBus {
    /// Redis connection pool
    client: Client,
    /// Connection manager for async operations
    conn: RwLock<ConnectionManager>,
    /// Configuration
    config: RedisEventBusConfig,
    /// Consumer ID for this instance
    consumer_id: String,
    /// Registered handlers by event type
    handlers: Arc<RwLock<HashMap<String, Vec<Arc<dyn EventHandler>>>>>,
}

#[derive(Debug, Clone)]
pub struct RedisEventBusConfig {
    /// Stream name for events
    pub stream_name: String,
    /// Consumer group name
    pub consumer_group: String,
    /// Dead letter queue stream name
    pub dlq_stream_name: String,
    /// Max retry attempts before DLQ
    pub max_retries: i32,
    /// Stream max length (MAXLEN ~)
    pub max_stream_length: i64,
    /// Claim idle messages after (ms)
    pub claim_idle_after_ms: i64,
    /// Block timeout for XREADGROUP (ms)
    pub block_timeout_ms: i64,
    /// Require TLS for Redis connection (default: true in production)
    pub require_tls: bool,
}

impl Default for RedisEventBusConfig {
    fn default() -> Self {
        let environment = std::env::var("ENVIRONMENT").unwrap_or_default();
        Self {
            stream_name: "choice-sherpa:events".to_string(),
            consumer_group: "default-handlers".to_string(),
            dlq_stream_name: "choice-sherpa:events:dlq".to_string(),
            max_retries: 3,
            max_stream_length: 100_000,
            claim_idle_after_ms: 60_000, // 1 minute
            block_timeout_ms: 5_000,     // 5 seconds
            // SECURITY: Default to requiring TLS in production
            require_tls: environment == "production",
        }
    }
}

impl RedisEventBus {
    pub async fn new(redis_url: &str, config: RedisEventBusConfig) -> Result<Self, DomainError> {
        // SECURITY: Validate TLS requirement in production
        if config.require_tls && !redis_url.starts_with("rediss://") {
            return Err(DomainError::new(
                ErrorCode::ConfigError,
                "TLS required: use rediss:// scheme"
            ));
        }

        let client = Client::open(redis_url)
            .map_err(|e| DomainError::new(ErrorCode::InternalError, &e.to_string()))?;

        let conn = ConnectionManager::new(client.clone())
            .await
            .map_err(|e| DomainError::new(ErrorCode::InternalError, &e.to_string()))?;

        let consumer_id = format!(
            "consumer-{}-{}",
            hostname::get().unwrap_or_default().to_string_lossy(),
            std::process::id()
        );

        let bus = Self {
            client,
            conn: RwLock::new(conn),
            config,
            consumer_id,
            handlers: Arc::new(RwLock::new(HashMap::new())),
        };

        // Ensure consumer group exists
        bus.ensure_consumer_group().await?;

        Ok(bus)
    }

    /// Create consumer group if it doesn't exist
    async fn ensure_consumer_group(&self) -> Result<(), DomainError> {
        let mut conn = self.conn.write().await;

        // Try to create group, ignore error if already exists
        let result: Result<(), redis::RedisError> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(&self.config.stream_name)
            .arg(&self.config.consumer_group)
            .arg("$") // Start from latest
            .arg("MKSTREAM") // Create stream if doesn't exist
            .query_async(&mut *conn)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) if e.to_string().contains("BUSYGROUP") => Ok(()), // Already exists
            Err(e) => Err(DomainError::new(ErrorCode::InternalError, &e.to_string())),
        }
    }

    /// Start consuming events (run in background task)
    pub async fn start_consuming(&self) -> Result<(), DomainError> {
        loop {
            // Read from consumer group
            let events = self.read_events().await?;

            for (stream_id, event) in events {
                let result = self.process_event(&stream_id, event).await;

                if result.is_ok() {
                    // Acknowledge successful processing
                    self.ack_event(&stream_id).await?;
                }
                // Failed events are handled by retry logic
            }

            // Claim idle messages from dead consumers
            self.claim_idle_messages().await?;
        }
    }

    async fn read_events(&self) -> Result<Vec<(String, EventEnvelope)>, DomainError> {
        let mut conn = self.conn.write().await;

        // XREADGROUP GROUP group consumer [COUNT count] [BLOCK ms] STREAMS key ID
        let result: Vec<HashMap<String, Vec<(String, HashMap<String, String>)>>> =
            redis::cmd("XREADGROUP")
                .arg("GROUP")
                .arg(&self.config.consumer_group)
                .arg(&self.consumer_id)
                .arg("COUNT")
                .arg(10)
                .arg("BLOCK")
                .arg(self.config.block_timeout_ms)
                .arg("STREAMS")
                .arg(&self.config.stream_name)
                .arg(">") // Only new messages
                .query_async(&mut *conn)
                .await
                .map_err(|e| DomainError::new(ErrorCode::InternalError, &e.to_string()))?;

        let mut events = Vec::new();

        for stream_data in result {
            if let Some(messages) = stream_data.get(&self.config.stream_name) {
                for (stream_id, fields) in messages {
                    if let Some(data) = fields.get("data") {
                        match serde_json::from_str::<EventEnvelope>(data) {
                            Ok(event) => events.push((stream_id.clone(), event)),
                            Err(e) => {
                                // Log parse error, move to DLQ
                                self.move_to_dlq(stream_id, None, &e.to_string()).await?;
                            }
                        }
                    }
                }
            }
        }

        Ok(events)
    }

    async fn process_event(
        &self,
        stream_id: &str,
        event: EventEnvelope,
    ) -> Result<(), DomainError> {
        let handlers = self.handlers.read().await;

        if let Some(type_handlers) = handlers.get(&event.event_type) {
            for handler in type_handlers {
                if let Err(e) = handler.handle(event.clone()).await {
                    // Check retry count
                    let retry_count = self.get_retry_count(stream_id).await?;

                    if retry_count >= self.config.max_retries {
                        // Move to DLQ
                        self.move_to_dlq(stream_id, Some(&event), &e.to_string()).await?;
                    } else {
                        // Increment retry count, will be picked up again
                        self.increment_retry_count(stream_id).await?;
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }

    async fn ack_event(&self, stream_id: &str) -> Result<(), DomainError> {
        let mut conn = self.conn.write().await;

        redis::cmd("XACK")
            .arg(&self.config.stream_name)
            .arg(&self.config.consumer_group)
            .arg(stream_id)
            .query_async::<_, i64>(&mut *conn)
            .await
            .map_err(|e| DomainError::new(ErrorCode::InternalError, &e.to_string()))?;

        Ok(())
    }

    async fn claim_idle_messages(&self) -> Result<(), DomainError> {
        let mut conn = self.conn.write().await;

        // XAUTOCLAIM to claim idle messages
        let _: () = redis::cmd("XAUTOCLAIM")
            .arg(&self.config.stream_name)
            .arg(&self.config.consumer_group)
            .arg(&self.consumer_id)
            .arg(self.config.claim_idle_after_ms)
            .arg("0-0") // Start from beginning of PEL
            .arg("COUNT")
            .arg(10)
            .query_async(&mut *conn)
            .await
            .map_err(|e| DomainError::new(ErrorCode::InternalError, &e.to_string()))?;

        Ok(())
    }

    async fn move_to_dlq(
        &self,
        stream_id: &str,
        event: Option<&EventEnvelope>,
        error: &str,
    ) -> Result<(), DomainError> {
        let mut conn = self.conn.write().await;

        let dlq_entry = DLQEntry {
            original_stream_id: stream_id.to_string(),
            event: event.cloned(),
            error: error.to_string(),
            failed_at: Timestamp::now(),
            consumer_id: self.consumer_id.clone(),
        };

        let data = serde_json::to_string(&dlq_entry)
            .map_err(|e| DomainError::new(ErrorCode::InternalError, &e.to_string()))?;

        // Add to DLQ stream
        redis::cmd("XADD")
            .arg(&self.config.dlq_stream_name)
            .arg("MAXLEN")
            .arg("~")
            .arg(10_000) // Keep last 10k DLQ entries
            .arg("*")
            .arg("data")
            .arg(&data)
            .query_async::<_, String>(&mut *conn)
            .await
            .map_err(|e| DomainError::new(ErrorCode::InternalError, &e.to_string()))?;

        // Acknowledge original message (remove from PEL)
        self.ack_event(stream_id).await?;

        Ok(())
    }

    async fn get_retry_count(&self, _stream_id: &str) -> Result<i32, DomainError> {
        // In production, store retry count in Redis hash
        // For simplicity, return 0 (each delivery is considered first attempt)
        // The PEL (Pending Entries List) tracks delivery attempts
        Ok(0)
    }

    async fn increment_retry_count(&self, _stream_id: &str) -> Result<(), DomainError> {
        // In production, increment in Redis hash
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DLQEntry {
    pub original_stream_id: String,
    pub event: Option<EventEnvelope>,
    pub error: String,
    pub failed_at: Timestamp,
    pub consumer_id: String,
}
```

### EventPublisher Implementation

```rust
#[async_trait]
impl EventPublisher for RedisEventBus {
    async fn publish(&self, event: EventEnvelope) -> Result<(), DomainError> {
        let mut conn = self.conn.write().await;

        let data = serde_json::to_string(&event)
            .map_err(|e| DomainError::new(ErrorCode::InternalError, &e.to_string()))?;

        // XADD stream MAXLEN ~ count * field value
        redis::cmd("XADD")
            .arg(&self.config.stream_name)
            .arg("MAXLEN")
            .arg("~")
            .arg(self.config.max_stream_length)
            .arg("*") // Auto-generate ID
            .arg("type")
            .arg(&event.event_type)
            .arg("data")
            .arg(&data)
            .query_async::<_, String>(&mut *conn)
            .await
            .map_err(|e| DomainError::new(ErrorCode::InternalError, &e.to_string()))?;

        Ok(())
    }

    async fn publish_all(&self, events: Vec<EventEnvelope>) -> Result<(), DomainError> {
        let mut conn = self.conn.write().await;

        // Use pipeline for atomicity
        let mut pipe = redis::pipe();

        for event in &events {
            let data = serde_json::to_string(event)
                .map_err(|e| DomainError::new(ErrorCode::InternalError, &e.to_string()))?;

            pipe.cmd("XADD")
                .arg(&self.config.stream_name)
                .arg("MAXLEN")
                .arg("~")
                .arg(self.config.max_stream_length)
                .arg("*")
                .arg("type")
                .arg(&event.event_type)
                .arg("data")
                .arg(&data);
        }

        pipe.query_async::<_, ()>(&mut *conn)
            .await
            .map_err(|e| DomainError::new(ErrorCode::InternalError, &e.to_string()))?;

        Ok(())
    }
}
```

### EventSubscriber Implementation

```rust
impl EventSubscriber for RedisEventBus {
    fn subscribe<H: EventHandler + 'static>(&self, event_type: &str, handler: H) {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let mut handlers = self.handlers.write().await;
            handlers
                .entry(event_type.to_string())
                .or_default()
                .push(Arc::new(handler));
        });
    }

    fn subscribe_all<H: EventHandler + 'static>(&self, event_types: &[&str], handler: H) {
        let handler = Arc::new(handler);
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let mut handlers = self.handlers.write().await;
            for event_type in event_types {
                handlers
                    .entry(event_type.to_string())
                    .or_default()
                    .push(Arc::clone(&handler));
            }
        });
    }
}
```

---

## Dead Letter Queue Admin API

```rust
// backend/src/adapters/http/admin/dlq_handlers.rs

/// List DLQ entries
pub async fn list_dlq(
    State(state): State<AppState>,
    Query(params): Query<DLQListParams>,
) -> Result<Json<Vec<DLQEntry>>, ApiError> {
    let entries = state.redis_event_bus
        .list_dlq_entries(params.limit.unwrap_or(100))
        .await?;

    Ok(Json(entries))
}

/// Replay a DLQ entry
pub async fn replay_dlq(
    State(state): State<AppState>,
    Path(stream_id): Path<String>,
) -> Result<Json<ReplayResult>, ApiError> {
    let result = state.redis_event_bus
        .replay_dlq_entry(&stream_id)
        .await?;

    Ok(Json(result))
}

/// Clear DLQ entries older than given timestamp
pub async fn clear_dlq(
    State(state): State<AppState>,
    Json(body): Json<ClearDLQRequest>,
) -> Result<Json<ClearResult>, ApiError> {
    let cleared = state.redis_event_bus
        .clear_dlq_before(body.before)
        .await?;

    Ok(Json(ClearResult { cleared_count: cleared }))
}

// Routes
pub fn dlq_routes() -> Router<AppState> {
    Router::new()
        .route("/admin/dlq", get(list_dlq))
        .route("/admin/dlq/:stream_id/replay", post(replay_dlq))
        .route("/admin/dlq", delete(clear_dlq))
}
```

### DLQ Admin API Security

All DLQ endpoints require admin authentication and authorization:

```rust
/// List DLQ entries - ADMIN ONLY
pub async fn list_dlq(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<DLQListParams>,
) -> Result<Json<Vec<DLQEntry>>, ApiError> {
    // SECURITY: Require admin role
    if !claims.roles.contains(&"admin".to_string()) {
        tracing::warn!(
            user_id = %claims.sub,
            "Unauthorized DLQ access attempt"
        );
        return Err(ApiError::Forbidden("Admin access required".to_string()));
    }

    let entries = state.redis_event_bus
        .list_dlq_entries(params.limit.unwrap_or(100))
        .await?;

    Ok(Json(entries))
}

// Route configuration with auth middleware
pub fn dlq_routes() -> Router<AppState> {
    Router::new()
        .route("/admin/dlq", get(list_dlq))
        .route("/admin/dlq/:stream_id/replay", post(replay_dlq))
        .route("/admin/dlq", delete(clear_dlq))
        .layer(middleware::from_fn(require_authenticated))
        .layer(middleware::from_fn(require_admin_role))
}
```

### DLQ Security Considerations

| Concern | Mitigation |
|---------|------------|
| Unauthorized access | Admin role required for all DLQ operations |
| Event data exposure | DLQ entries may contain sensitive payloads - admin audit logging required |
| Replay attacks | Replayed events maintain original timestamps, logged for audit |
| Mass deletion | Clear operation requires confirmation parameter |

---

## Configuration

```rust
// backend/src/config/mod.rs

#[derive(Debug, Clone, Deserialize)]
pub struct EventBusConfig {
    /// Use Redis or InMemory
    pub adapter: EventBusAdapter,
    /// Redis URL (if adapter = Redis)
    pub redis_url: Option<String>,
    /// Redis stream configuration
    pub redis: Option<RedisEventBusConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventBusAdapter {
    InMemory,
    Redis,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            adapter: EventBusAdapter::InMemory,
            redis_url: None,
            redis: None,
        }
    }
}

// Factory function
pub fn create_event_bus(config: &EventBusConfig) -> Result<Arc<dyn EventBus>, DomainError> {
    match config.adapter {
        EventBusAdapter::InMemory => {
            Ok(Arc::new(InMemoryEventBus::new()))
        }
        EventBusAdapter::Redis => {
            let redis_url = config.redis_url.as_ref()
                .ok_or_else(|| DomainError::new(ErrorCode::ValidationFailed, "Redis URL required"))?;
            let redis_config = config.redis.clone().unwrap_or_default();

            let rt = tokio::runtime::Handle::current();
            let bus = rt.block_on(RedisEventBus::new(redis_url, redis_config))?;
            Ok(Arc::new(bus))
        }
    }
}
```

### Environment Variables

```yaml
# .env.production
EVENT_BUS_ADAPTER=redis
# SECURITY: Use rediss:// scheme for TLS-encrypted connections
EVENT_BUS_REDIS_URL=rediss://:${REDIS_PASSWORD}@redis.example.com:6379
EVENT_BUS_REDIS_REQUIRE_TLS=true
EVENT_BUS_REDIS_STREAM=choice-sherpa:events
EVENT_BUS_REDIS_CONSUMER_GROUP=handlers
EVENT_BUS_REDIS_MAX_RETRIES=3
EVENT_BUS_REDIS_MAX_STREAM_LENGTH=100000
```

---

## Acceptance Criteria

### AC1: Publish Persists to Stream

**Given** an event is published via Redis bus
**When** the publish completes
**Then** the event is stored in Redis Stream with MAXLEN trimming

### AC2: Consumer Groups Distribute Load

**Given** multiple server instances with same consumer group
**When** events are published
**Then** each event is delivered to only one consumer in the group

### AC3: Failed Events Retry

**Given** a handler fails processing an event
**When** retry count < max retries
**Then** event stays in PEL for redelivery

### AC4: DLQ on Max Retries

**Given** a handler fails max_retries times
**When** the event fails again
**Then** event is moved to DLQ stream and acknowledged

### AC5: Idle Message Claiming

**Given** a consumer crashes with pending messages
**When** another consumer runs XAUTOCLAIM
**Then** idle messages are claimed and processed

### AC6: Replay from DLQ

**Given** an event is in the DLQ
**When** admin triggers replay
**Then** event is republished to main stream

### AC7: Graceful Degradation

**Given** Redis is unavailable
**When** publish is attempted
**Then** Error is returned (caller can implement fallback)

---

## File Structure

```
backend/src/adapters/events/
├── mod.rs                    # Module exports, factory function
├── in_memory.rs              # Existing InMemoryEventBus
├── in_memory_test.rs
├── redis.rs                  # NEW: RedisEventBus
├── redis_test.rs             # NEW: Integration tests
├── dlq.rs                    # NEW: DLQ operations
└── dlq_test.rs               # NEW

backend/src/adapters/http/admin/
├── mod.rs                    # Admin routes
├── dlq_handlers.rs           # NEW: DLQ API
└── dlq_handlers_test.rs      # NEW

backend/src/config/
├── mod.rs                    # MODIFY: Add EventBusConfig
└── event_bus.rs              # NEW: Event bus configuration
```

---

## Test Specifications

### Integration Tests (Require Redis)

```rust
#[tokio::test]
#[ignore] // Requires Redis
async fn redis_bus_publish_and_consume() {
    let config = RedisEventBusConfig {
        stream_name: "test:events".to_string(),
        consumer_group: "test-group".to_string(),
        ..Default::default()
    };

    let bus = RedisEventBus::new("redis://localhost:6379", config)
        .await
        .unwrap();

    let received = Arc::new(AtomicBool::new(false));
    let received_clone = received.clone();

    struct TestHandler(Arc<AtomicBool>);

    #[async_trait]
    impl EventHandler for TestHandler {
        async fn handle(&self, _: EventEnvelope) -> Result<(), DomainError> {
            self.0.store(true, Ordering::SeqCst);
            Ok(())
        }
        fn name(&self) -> &'static str { "TestHandler" }
    }

    bus.subscribe("test.event", TestHandler(received_clone));

    // Publish event
    let event = test_envelope("test.event", "agg-1");
    bus.publish(event).await.unwrap();

    // Start consuming in background
    let bus_clone = Arc::new(bus);
    tokio::spawn(async move {
        let _ = bus_clone.start_consuming().await;
    });

    // Wait for processing
    tokio::time::sleep(Duration::from_secs(1)).await;

    assert!(received.load(Ordering::SeqCst));
}

#[tokio::test]
#[ignore]
async fn redis_bus_moves_to_dlq_after_max_retries() {
    let config = RedisEventBusConfig {
        stream_name: "test:events:dlq-test".to_string(),
        dlq_stream_name: "test:events:dlq".to_string(),
        max_retries: 1,
        ..Default::default()
    };

    let bus = RedisEventBus::new("redis://localhost:6379", config.clone())
        .await
        .unwrap();

    // Handler that always fails
    struct FailingHandler;

    #[async_trait]
    impl EventHandler for FailingHandler {
        async fn handle(&self, _: EventEnvelope) -> Result<(), DomainError> {
            Err(DomainError::new(ErrorCode::InternalError, "Always fails"))
        }
        fn name(&self) -> &'static str { "FailingHandler" }
    }

    bus.subscribe("test.event", FailingHandler);

    // Publish event
    bus.publish(test_envelope("test.event", "agg-1")).await.unwrap();

    // Process (will fail and move to DLQ after retries)
    // ... start consuming ...

    // Verify in DLQ
    let dlq_entries = bus.list_dlq_entries(10).await.unwrap();
    assert!(!dlq_entries.is_empty());
}
```

### Unit Tests (Mock Redis)

```rust
#[tokio::test]
async fn config_factory_creates_in_memory() {
    let config = EventBusConfig {
        adapter: EventBusAdapter::InMemory,
        redis_url: None,
        redis: None,
    };

    let bus = create_event_bus(&config).unwrap();
    // Should not panic, should be InMemory type
}

#[tokio::test]
async fn config_factory_requires_redis_url() {
    let config = EventBusConfig {
        adapter: EventBusAdapter::Redis,
        redis_url: None, // Missing!
        redis: None,
    };

    let result = create_event_bus(&config);
    assert!(result.is_err());
}
```

---

## Monitoring & Observability

### Metrics to Track

| Metric | Type | Description |
|--------|------|-------------|
| `events_published_total` | Counter | Total events published |
| `events_processed_total` | Counter | Total events successfully processed |
| `events_failed_total` | Counter | Total events that failed processing |
| `events_dlq_total` | Counter | Total events moved to DLQ |
| `event_processing_duration_ms` | Histogram | Processing time per event |
| `stream_length` | Gauge | Current stream length |
| `pel_length` | Gauge | Pending entries list length |
| `consumer_lag` | Gauge | Messages behind latest |

### Health Check

```rust
pub async fn event_bus_health(bus: &RedisEventBus) -> HealthStatus {
    match bus.ping().await {
        Ok(_) => {
            let lag = bus.get_consumer_lag().await.unwrap_or(0);
            if lag > 1000 {
                HealthStatus::Degraded("High consumer lag")
            } else {
                HealthStatus::Healthy
            }
        }
        Err(e) => HealthStatus::Unhealthy(e.to_string()),
    }
}
```

---

## Dependencies

### Crate Dependencies

```toml
[dependencies]
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
```

### Module Dependencies

- `foundation::events` - EventEnvelope, DomainEvent
- `ports::event_publisher` - EventPublisher trait
- `ports::event_subscriber` - EventSubscriber, EventHandler traits

---

## Related Documents

- **Phase 1:** features/foundation/event-infrastructure.md
- **Checklist:** REQUIREMENTS/CHECKLIST-events.md (Phase 7)
- **Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md

---

*Version: 1.0.0*
*Created: 2026-01-07*
*Phase: 7 of 8*
