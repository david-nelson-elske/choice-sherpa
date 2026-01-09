# Feature: Dashboard Data Freshness Model

**Module:** dashboard
**Type:** Caching & Consistency Strategy
**Priority:** P1
**Status:** Specification Complete

> Defines how the dashboard maintains data freshness, handles stale data, and ensures consistency across the read model cache.

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Required |
| Authorization Model | Cached data must be re-authorized on access; session ownership verified |
| Sensitive Data | Cached session/cycle data (Confidential) |
| Rate Limiting | Required - cache refresh rate, manual refresh rate |
| Audit Logging | Cache hits/misses, eviction events, reconciliation events |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| Cached session data | Confidential | Encrypt at rest if using external cache |
| Cached cycle data | Confidential | Same as session data |
| Cached scores | Confidential | Derived from user decisions |
| Cache metadata (timestamps, versions) | Internal | Safe to log |
| Cache metrics (hit rates) | Internal | Safe to aggregate |

### Authorization Model for Cached Data

**Critical**: Cached data does not inherently include authorization information. Authorization must be enforced at access time:

```
1. User requests dashboard data
2. Check: Does user own session_id?
3. If yes: Return cached data (or load from DB)
4. If no: Return 403 Forbidden
```

This means:
- Cache invalidation events do NOT re-verify ownership
- Every read operation MUST verify ownership
- WebSocket event forwarding MUST verify ownership

### Security Events to Log

- Cache warm-up start/complete - Log session count, duration
- Cache eviction - Log session_id, eviction reason (age/size)
- Database fallback - Log session_id, cache miss reason
- Reconciliation detected stale - Log session_id, version mismatch
- Manual refresh request - Log user_id, session_id

### Cache Security Best Practices

1. **No sensitive data in cache keys** - Use UUIDs, not user-identifiable data
2. **Re-authorize on read** - Never assume cached data is authorized
3. **Secure cache invalidation** - Invalidation requests must be authenticated
4. **Memory-only cache** - If using external cache (Redis), encrypt sensitive fields
5. **Cache poisoning prevention** - Validate event payloads before updating cache

### WebSocket Security with Cached Data

- Connection must be authenticated and associated with user_id
- Each session subscription must verify user owns session
- Event forwarding must re-verify ownership (session could be transferred)
- Connection timeout for inactive sessions

---

## Overview

The Dashboard module maintains an in-memory cache populated by domain events. This specification defines:
1. Cache invalidation strategies
2. Freshness indicators for the UI
3. Fallback to database on cache miss
4. Consistency guarantees
5. Memory management

---

## 1. Cache Architecture

### Cache Layers

```
┌─────────────────────────────────────────────────────────────────┐
│                        Dashboard API                             │
└───────────────┬─────────────────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    L1: In-Memory Cache                           │
│                   (DashboardCache struct)                        │
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │  Sessions   │  │   Cycles    │  │ Components  │              │
│  │  HashMap    │  │  HashMap    │  │  HashMap    │              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │ Pugh Scores │  │ DQ Scores   │  │  Messages   │              │
│  │  HashMap    │  │  HashMap    │  │  HashMap    │              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
│                                                                  │
│  TTL: Indefinite (event-driven invalidation)                     │
└───────────────┬─────────────────────────────────────────────────┘
                │ Cache Miss
                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    L2: PostgreSQL Database                       │
│                   (Source of Truth)                              │
│                                                                  │
│  Complex JOINs across: sessions, cycles, components,             │
│  conversations, component_outputs tables                         │
└─────────────────────────────────────────────────────────────────┘
```

### Cache Entry Structure

```rust
/// Cache entry wrapper with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    /// The cached data
    pub data: T,
    /// When this entry was last updated
    pub updated_at: Timestamp,
    /// The event ID that caused this update
    pub last_event_id: EventId,
    /// Whether this entry is marked stale (needs refresh)
    pub is_stale: bool,
    /// Version number for optimistic locking
    pub version: u64,
}

impl<T> CacheEntry<T> {
    pub fn new(data: T, event_id: EventId) -> Self {
        Self {
            data,
            updated_at: Timestamp::now(),
            last_event_id: event_id,
            is_stale: false,
            version: 1,
        }
    }

    pub fn mark_stale(&mut self) {
        self.is_stale = true;
    }

    pub fn age(&self) -> Duration {
        Timestamp::now().duration_since(self.updated_at)
    }
}
```

---

## 2. Freshness Indicators

### Data Age Thresholds

```rust
/// Freshness thresholds for dashboard data
pub struct FreshnessConfig {
    /// Data older than this is considered "warm" (show indicator)
    pub warm_threshold: Duration,
    /// Data older than this is considered "stale" (auto-refresh)
    pub stale_threshold: Duration,
    /// Data older than this triggers cache eviction
    pub expiry_threshold: Duration,
}

impl Default for FreshnessConfig {
    fn default() -> Self {
        Self {
            warm_threshold: Duration::from_secs(30),   // 30 seconds
            stale_threshold: Duration::from_secs(300), // 5 minutes
            expiry_threshold: Duration::from_secs(3600), // 1 hour
        }
    }
}

pub enum FreshnessLevel {
    /// Data is fresh (< warm_threshold)
    Fresh,
    /// Data is warm but still valid (warm_threshold..stale_threshold)
    Warm,
    /// Data needs refresh (> stale_threshold)
    Stale,
    /// Data has expired (> expiry_threshold)
    Expired,
}
```

### API Response with Freshness

```rust
/// Dashboard response includes freshness metadata
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardOverviewResponse {
    // Data
    pub session: SessionInfo,
    pub objectives: Vec<ObjectiveSummary>,
    pub alternatives: Vec<AlternativeSummary>,
    pub consequences_table: Option<CompactConsequencesTable>,
    pub recommendation: Option<RecommendationSummary>,
    pub dq_score: Option<u8>,
    pub cycle: CycleInfo,

    // Freshness metadata
    pub freshness: FreshnessMetadata,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FreshnessMetadata {
    /// When this data was last updated
    pub updated_at: String, // ISO 8601
    /// Freshness level indicator
    pub freshness_level: String, // "fresh" | "warm" | "stale"
    /// Whether background refresh is recommended
    pub should_refresh: bool,
    /// Components that are stale
    pub stale_components: Vec<String>,
}
```

### Frontend Handling

```typescript
// frontend/src/modules/dashboard/hooks/use-dashboard-freshness.ts

export interface FreshnessMetadata {
  updatedAt: Date;
  freshnessLevel: 'fresh' | 'warm' | 'stale';
  shouldRefresh: boolean;
  staleComponents: string[];
}

export function useDashboardFreshness(freshness: FreshnessMetadata) {
  const [isRefreshing, setIsRefreshing] = useState(false);

  useEffect(() => {
    // Auto-refresh if stale
    if (freshness.shouldRefresh && !isRefreshing) {
      setIsRefreshing(true);
      refetchDashboard().finally(() => setIsRefreshing(false));
    }
  }, [freshness.shouldRefresh]);

  // Visual indicator
  const indicator = useMemo(() => {
    switch (freshness.freshnessLevel) {
      case 'fresh':
        return null; // No indicator needed
      case 'warm':
        return { color: 'yellow', message: 'Data may be slightly out of date' };
      case 'stale':
        return { color: 'orange', message: 'Refreshing data...' };
    }
  }, [freshness.freshnessLevel]);

  return { indicator, isRefreshing };
}
```

---

## 3. Cache Invalidation Strategies

### Event-Driven Invalidation

Primary strategy: Events invalidate specific cache entries.

```rust
impl DashboardUpdateHandler {
    /// Invalidation map: event type -> affected cache regions
    fn invalidation_map(&self) -> HashMap<&'static str, Vec<CacheRegion>> {
        HashMap::from([
            ("session.created", vec![CacheRegion::Sessions]),
            ("session.renamed", vec![CacheRegion::Sessions]),
            ("session.archived", vec![CacheRegion::Sessions]),

            ("cycle.created", vec![CacheRegion::Cycles, CacheRegion::Sessions]),
            ("cycle.branched", vec![CacheRegion::Cycles]),
            ("component.started", vec![CacheRegion::Cycles, CacheRegion::Components]),
            ("component.completed", vec![CacheRegion::Cycles, CacheRegion::Components]),
            ("component.output_updated", vec![CacheRegion::Components]),

            ("analysis.pugh_scores_computed", vec![CacheRegion::PughScores]),
            ("analysis.dq_scores_computed", vec![CacheRegion::DQScores]),

            ("message.sent", vec![CacheRegion::RecentMessages]),
        ])
    }

    async fn invalidate_for_event(&self, event_type: &str, entity_ids: &EventEntityIds) {
        if let Some(regions) = self.invalidation_map().get(event_type) {
            for region in regions {
                match region {
                    CacheRegion::Sessions => {
                        if let Some(id) = &entity_ids.session_id {
                            self.cache.mark_session_stale(id).await;
                        }
                    }
                    CacheRegion::Cycles => {
                        if let Some(id) = &entity_ids.cycle_id {
                            self.cache.mark_cycle_stale(id).await;
                        }
                    }
                    // ... other regions
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CacheRegion {
    Sessions,
    Cycles,
    Components,
    PughScores,
    DQScores,
    RecentMessages,
}
```

### Time-Based Expiration

Secondary strategy: Entries expire after TTL even without invalidation events.

```rust
impl DashboardCache {
    /// Background task to evict expired entries
    pub async fn eviction_task(&self, config: FreshnessConfig) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;
            self.evict_expired(config.expiry_threshold).await;
        }
    }

    async fn evict_expired(&self, threshold: Duration) {
        let now = Timestamp::now();

        // Sessions
        let mut sessions = self.sessions.write().unwrap();
        sessions.retain(|_, entry| {
            now.duration_since(entry.updated_at) < threshold
        });

        // Cycles
        let mut cycles = self.cycles.write().unwrap();
        cycles.retain(|_, entry| {
            now.duration_since(entry.updated_at) < threshold
        });

        // ... other caches
    }
}
```

### Manual Refresh

User-triggered refresh bypasses cache entirely.

```rust
impl DashboardReader for PostgresDashboardReader {
    async fn get_overview_fresh(
        &self,
        session_id: SessionId,
        user_id: &UserId,
    ) -> Result<DashboardOverview, DashboardError> {
        // Bypass cache, query database directly
        let overview = self.build_overview_from_db(session_id, user_id).await?;

        // Update cache with fresh data
        self.cache.update_session_entry(session_id, overview.clone()).await;

        Ok(overview)
    }
}
```

---

## 4. Cache Miss Handling

### Read-Through Pattern

```rust
impl DashboardCache {
    /// Gets session from cache or falls back to database
    pub async fn get_session_or_load(
        &self,
        session_id: SessionId,
        db_loader: &impl SessionLoader,
    ) -> Result<SessionCacheEntry, CacheError> {
        // Try cache first
        if let Some(entry) = self.get_session(session_id).await {
            if !entry.is_stale {
                return Ok(entry.data);
            }
            // Stale - fall through to load fresh
        }

        // Cache miss or stale - load from database
        let fresh = db_loader.load_session(session_id).await
            .map_err(|e| CacheError::LoadFailed(e.to_string()))?;

        // Populate cache
        self.create_session_entry(fresh.clone()).await;

        Ok(fresh)
    }
}

#[async_trait]
pub trait SessionLoader: Send + Sync {
    async fn load_session(&self, id: SessionId) -> Result<SessionCacheEntry, DbError>;
}
```

### Cold Start Strategy

On application startup, cache is empty. Strategy:

```rust
impl DashboardCache {
    /// Warm cache on startup with recent/active sessions
    pub async fn warm_cache(
        &self,
        db_loader: &impl BulkLoader,
        config: WarmCacheConfig,
    ) -> Result<WarmCacheStats, CacheError> {
        let mut stats = WarmCacheStats::default();

        // Load active sessions (limit to prevent memory spike)
        let sessions = db_loader
            .load_recent_active_sessions(config.max_sessions)
            .await?;

        for session in sessions {
            self.create_session_entry(session).await;
            stats.sessions_loaded += 1;

            // Load cycles for each session
            let cycles = db_loader
                .load_cycles_for_session(session.session_id, config.max_cycles_per_session)
                .await?;

            for cycle in cycles {
                self.create_cycle_entry(cycle).await;
                stats.cycles_loaded += 1;
            }
        }

        Ok(stats)
    }
}

pub struct WarmCacheConfig {
    pub max_sessions: usize,       // Default: 100
    pub max_cycles_per_session: usize, // Default: 5
}

#[derive(Default)]
pub struct WarmCacheStats {
    pub sessions_loaded: usize,
    pub cycles_loaded: usize,
    pub duration_ms: u64,
}
```

---

## 5. Consistency Guarantees

### Eventual Consistency Model

The dashboard cache provides **eventual consistency**, not strong consistency:

| Guarantee | Description |
|-----------|-------------|
| **Event ordering** | Events processed in order they're received |
| **At-least-once delivery** | Events may be delivered multiple times (idempotency required) |
| **Eventual freshness** | Cache reflects latest state within stale_threshold |
| **No partial writes** | Atomic updates within single cache region |

### Concurrency Handling

```rust
impl DashboardCache {
    /// Update with optimistic locking
    pub async fn update_session_with_version(
        &self,
        id: SessionId,
        expected_version: u64,
        updater: impl FnOnce(&mut SessionCacheEntry),
    ) -> Result<(), ConcurrencyError> {
        let mut sessions = self.sessions.write().unwrap();

        let entry = sessions.get_mut(&id)
            .ok_or(ConcurrencyError::NotFound)?;

        if entry.version != expected_version {
            return Err(ConcurrencyError::VersionMismatch {
                expected: expected_version,
                actual: entry.version,
            });
        }

        updater(&mut entry.data);
        entry.version += 1;
        entry.updated_at = Timestamp::now();

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConcurrencyError {
    #[error("Entry not found")]
    NotFound,
    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u64, actual: u64 },
}
```

### Cross-Cache Consistency

When updating related caches, use batch updates:

```rust
impl DashboardUpdateHandler {
    async fn handle_component_completed(&self, event: &EventEnvelope) -> Result<(), DomainError> {
        let payload: ComponentCompleted = event.payload_as()?;

        // Update both cycle and component in single "transaction"
        self.cache.batch_update(|batch| {
            batch.update_cycle(payload.cycle_id, |cycle| {
                cycle.progress = payload.progress.clone();
            });

            batch.update_component_status(
                payload.cycle_id,
                payload.component_type,
                ComponentStatus::Complete,
            );
        }).await;

        Ok(())
    }
}

impl DashboardCache {
    pub async fn batch_update<F>(&self, updates: F)
    where
        F: FnOnce(&mut BatchUpdater),
    {
        let mut updater = BatchUpdater::new();
        updates(&mut updater);

        // Apply all updates atomically
        for update in updater.cycle_updates {
            // ... apply cycle update
        }
        for update in updater.component_updates {
            // ... apply component update
        }
    }
}
```

---

## 6. Memory Management

### Size Limits

```rust
pub struct CacheConfig {
    /// Maximum sessions in cache
    pub max_sessions: usize,        // Default: 1000
    /// Maximum cycles per session
    pub max_cycles_per_session: usize, // Default: 10
    /// Maximum total cycles
    pub max_total_cycles: usize,    // Default: 5000
    /// Maximum messages per cycle
    pub max_messages_per_cycle: usize, // Default: 50
    /// Maximum total memory (bytes)
    pub max_memory_bytes: usize,    // Default: 100MB
}
```

### Eviction Policy

LRU (Least Recently Used) eviction when limits exceeded:

```rust
impl DashboardCache {
    pub async fn ensure_within_limits(&self, config: &CacheConfig) {
        // Evict oldest sessions if over limit
        let mut sessions = self.sessions.write().unwrap();
        if sessions.len() > config.max_sessions {
            let excess = sessions.len() - config.max_sessions;
            let mut entries: Vec<_> = sessions.iter().collect();
            entries.sort_by_key(|(_, e)| e.updated_at);

            for (id, _) in entries.into_iter().take(excess) {
                sessions.remove(id);
            }
        }

        // Similar for cycles, messages, etc.
    }

    /// Estimate current memory usage
    pub fn estimated_memory(&self) -> usize {
        let sessions = self.sessions.read().unwrap();
        let cycles = self.cycles.read().unwrap();
        // Rough estimate: 1KB per session, 2KB per cycle
        sessions.len() * 1024 + cycles.len() * 2048
    }
}
```

---

## 7. Monitoring & Observability

### Cache Metrics

```rust
#[derive(Debug, Default)]
pub struct CacheMetrics {
    /// Cache hit count
    pub hits: AtomicU64,
    /// Cache miss count
    pub misses: AtomicU64,
    /// Stale data served count
    pub stale_served: AtomicU64,
    /// Eviction count
    pub evictions: AtomicU64,
    /// Database fallback count
    pub db_fallbacks: AtomicU64,
}

impl CacheMetrics {
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let total = hits + self.misses.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        hits as f64 / total as f64
    }
}
```

### Health Check

```rust
/// Cache health for /health endpoint
#[derive(Debug, Serialize)]
pub struct CacheHealth {
    pub status: HealthStatus,
    pub session_count: usize,
    pub cycle_count: usize,
    pub hit_rate: f64,
    pub memory_usage_mb: f64,
    pub oldest_entry_age_secs: u64,
}

pub async fn check_cache_health(cache: &DashboardCache) -> CacheHealth {
    CacheHealth {
        status: HealthStatus::Healthy,
        session_count: cache.session_count().await,
        cycle_count: cache.cycle_count().await,
        hit_rate: cache.metrics.hit_rate(),
        memory_usage_mb: cache.estimated_memory() as f64 / 1_000_000.0,
        oldest_entry_age_secs: cache.oldest_entry_age().await.as_secs(),
    }
}
```

---

## 8. Edge Cases

### Missing Event Scenarios

| Scenario | Detection | Recovery |
|----------|-----------|----------|
| Event dropped | Data age > stale_threshold | Database refresh |
| Event reordering | Version check fails | Re-fetch from database |
| Duplicate event | Event ID in processed set | Skip processing |
| Partial event | Deserialization fails | Log error, skip |

### Recovery Procedures

```rust
impl DashboardUpdateHandler {
    /// Periodic consistency check
    pub async fn reconcile_with_database(&self, db: &impl DashboardReader) {
        // For each cached session, verify against database
        let cached_sessions = self.cache.all_session_ids().await;

        for session_id in cached_sessions {
            let cached = self.cache.get_session(session_id).await;
            let db_version = db.get_session_version(session_id).await;

            if let (Some(cached), Ok(db_ver)) = (cached, db_version) {
                if cached.version < db_ver {
                    // Cache is behind - refresh
                    let fresh = db.get_session(session_id).await;
                    if let Ok(fresh) = fresh {
                        self.cache.replace_session(session_id, fresh).await;
                    }
                }
            }
        }
    }
}
```

---

## 9. WebSocket Real-Time Updates

### Push vs Pull

- **Push (preferred)**: Events forwarded to WebSocket immediately
- **Pull (fallback)**: Frontend polls if WebSocket disconnected

```rust
impl WebSocketEventBridge {
    /// Forward event to connected clients for this session
    pub async fn forward_event(&self, event: &EventEnvelope) {
        // Extract session_id from event
        if let Some(session_id) = event.session_id() {
            // Get all connections for this session
            let connections = self.connections_for_session(session_id).await;

            // Send event to each connection
            let message = WebSocketMessage::Event {
                event_type: event.event_type.clone(),
                payload: event.payload.clone(),
            };

            for conn in connections {
                if let Err(e) = conn.send(message.clone()).await {
                    // Connection lost - remove from pool
                    self.remove_connection(conn.id).await;
                }
            }
        }
    }
}
```

### Frontend WebSocket Integration

```typescript
// frontend/src/modules/dashboard/hooks/use-dashboard-socket.ts

export function useDashboardSocket(sessionId: string) {
  const queryClient = useQueryClient();

  useEffect(() => {
    const ws = new WebSocket(`${WS_URL}/sessions/${sessionId}/events`);

    ws.onmessage = (event) => {
      const { eventType, payload } = JSON.parse(event.data);

      // Invalidate relevant queries based on event type
      switch (eventType) {
        case 'component.completed':
          queryClient.invalidateQueries(['dashboard', sessionId]);
          queryClient.invalidateQueries(['cycle', payload.cycleId]);
          break;

        case 'analysis.pugh_scores_computed':
          queryClient.invalidateQueries(['consequences', payload.cycleId]);
          break;

        case 'analysis.dq_scores_computed':
          queryClient.invalidateQueries(['dq-score', payload.cycleId]);
          break;
      }
    };

    return () => ws.close();
  }, [sessionId, queryClient]);
}
```

---

## Acceptance Criteria

### AC1: Fresh Data Indication
**Given** dashboard data was updated within 30 seconds
**When** API response is returned
**Then** `freshnessLevel: "fresh"` and `shouldRefresh: false`

### AC2: Warm Data Indication
**Given** dashboard data was updated 2 minutes ago
**When** API response is returned
**Then** `freshnessLevel: "warm"` and `shouldRefresh: false`

### AC3: Stale Data Auto-Refresh
**Given** dashboard data was updated 6 minutes ago
**When** API response is returned
**Then** `freshnessLevel: "stale"` and `shouldRefresh: true`

### AC4: Cache Miss Fallback
**Given** session not in cache
**When** dashboard overview requested
**Then** data loaded from database and cached

### AC5: Event-Driven Updates
**Given** `component.completed` event received
**When** event is processed
**Then** cycle progress and component status updated in cache

### AC6: Memory Limit Enforcement
**Given** cache at max_sessions limit
**When** new session cached
**Then** oldest session evicted (LRU)

---

## File Structure

```
backend/src/adapters/cache/
├── mod.rs
├── dashboard_cache.rs          # Core cache implementation
├── cache_entry.rs              # CacheEntry wrapper
├── freshness.rs                # Freshness config and levels
├── eviction.rs                 # LRU eviction logic
├── metrics.rs                  # Cache metrics
└── tests/
    ├── cache_test.rs
    ├── freshness_test.rs
    └── eviction_test.rs

frontend/src/modules/dashboard/
├── hooks/
│   ├── use-dashboard-freshness.ts
│   └── use-dashboard-socket.ts
├── components/
│   └── FreshnessIndicator.tsx
└── types/
    └── freshness.ts
```

---

*Version: 1.0.0*
*Created: 2026-01-08*
*Module: dashboard*
