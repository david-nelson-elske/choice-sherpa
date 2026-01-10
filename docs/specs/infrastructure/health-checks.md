# Infrastructure: Health Checks and Readiness Probes

**Type:** Cross-Cutting Infrastructure
**Priority:** P0 (Required for production deployment)
**Last Updated:** 2026-01-08

> Complete specification for liveness, readiness, and startup probes for Kubernetes/container orchestration deployments.

---

## Overview

Choice Sherpa requires three types of health checks for proper orchestration:

| Probe Type | Purpose | Failure Action |
|------------|---------|----------------|
| **Liveness** | Is the process alive and responsive? | Container restart |
| **Readiness** | Can the service handle traffic? | Remove from load balancer |
| **Startup** | Has the service finished starting? | Wait before other probes |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Kubernetes / Container Runtime                        │
│                                                                              │
│   Probes:                                                                    │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  /health/live    → Liveness    → Restart on failure                 │   │
│   │  /health/ready   → Readiness   → Remove from LB on failure          │   │
│   │  /health/startup → Startup     → Wait for success                   │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
└────────────────────────────────────┼─────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          Choice Sherpa Application                           │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                       Health Check Handler                          │   │
│   │                                                                      │   │
│   │   LivenessChecker:                                                  │   │
│   │     - Process alive                                                 │   │
│   │     - Memory not exhausted                                          │   │
│   │     - No deadlock detected                                          │   │
│   │                                                                      │   │
│   │   ReadinessChecker:                                                 │   │
│   │     - All dependencies available                                    │   │
│   │     - PostgreSQL connected                                          │   │
│   │     - Redis connected (if configured)                               │   │
│   │     - Not under excessive load                                      │   │
│   │                                                                      │   │
│   │   StartupChecker:                                                   │   │
│   │     - Migrations completed                                          │   │
│   │     - Event handlers registered                                     │   │
│   │     - Background tasks started                                      │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   Dependencies:                                                              │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│   │ PostgreSQL  │  │    Redis    │  │  Zitadel    │  │   Stripe    │       │
│   │  (Required) │  │  (Optional) │  │  (Required) │  │  (Optional) │       │
│   └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘       │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Health Check Types

### Health Status Enum

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum HealthStatus {
    /// Service is functioning normally
    Healthy,
    /// Service is functional but degraded
    Degraded { reason: String },
    /// Service is not functional
    Unhealthy { reason: String },
}

impl HealthStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded { .. })
    }

    pub fn http_status_code(&self) -> StatusCode {
        match self {
            HealthStatus::Healthy => StatusCode::OK,
            HealthStatus::Degraded { .. } => StatusCode::OK,
            HealthStatus::Unhealthy { .. } => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub component: String,
    pub status: HealthStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub timestamp: String,
    pub version: String,
    pub checks: Vec<HealthCheckResult>,
}
```

---

## Endpoint Specifications

### Liveness Endpoint

**Purpose:** Detect if the application is in a broken state requiring restart.

```
GET /health/live
```

**Checks:**
- Process is responsive (can handle HTTP request)
- No memory pressure (optional)
- Tokio runtime responsive

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2026-01-08T10:30:00Z",
  "version": "1.0.0",
  "checks": []
}
```

**Implementation:**

```rust
pub async fn liveness_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Simple liveness check - just verify we can respond
    let response = HealthResponse {
        status: HealthStatus::Healthy,
        timestamp: Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks: vec![],
    };

    (StatusCode::OK, Json(response))
}
```

---

### Readiness Endpoint

**Purpose:** Determine if the application can serve traffic.

```
GET /health/ready
```

**Checks:**
- PostgreSQL connection pool available
- Redis connection (if configured)
- Auth service reachable (Zitadel)
- Not under excessive load

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2026-01-08T10:30:00Z",
  "version": "1.0.0",
  "checks": [
    {
      "component": "database",
      "status": "healthy",
      "latency_ms": 5,
      "details": {
        "pool_size": 20,
        "pool_idle": 15,
        "utilization_percent": 25.0
      }
    },
    {
      "component": "redis",
      "status": "healthy",
      "latency_ms": 2
    },
    {
      "component": "auth",
      "status": "healthy",
      "latency_ms": 45
    }
  ]
}
```

**Implementation:**

```rust
pub async fn readiness_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let checks = tokio::join!(
        check_database(&state.db_pool),
        check_redis(&state.redis_client),
        check_auth(&state.auth_service),
    );

    let check_results = vec![checks.0, checks.1, checks.2];

    // Overall status is worst of all checks
    let overall_status = check_results.iter()
        .map(|c| &c.status)
        .fold(HealthStatus::Healthy, |acc, status| {
            match (&acc, status) {
                (_, HealthStatus::Unhealthy { .. }) => status.clone(),
                (HealthStatus::Unhealthy { .. }, _) => acc,
                (_, HealthStatus::Degraded { .. }) => status.clone(),
                (HealthStatus::Degraded { .. }, _) => acc,
                _ => acc,
            }
        });

    let response = HealthResponse {
        status: overall_status.clone(),
        timestamp: Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks: check_results,
    };

    (overall_status.http_status_code(), Json(response))
}
```

---

### Startup Endpoint

**Purpose:** Indicate when the application has completed initialization.

```
GET /health/startup
```

**Checks:**
- Database migrations completed
- Event handlers registered
- Background tasks started
- Initial data loaded (if applicable)

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2026-01-08T10:30:00Z",
  "version": "1.0.0",
  "checks": [
    {
      "component": "migrations",
      "status": "healthy",
      "details": { "applied": 12, "pending": 0 }
    },
    {
      "component": "event_handlers",
      "status": "healthy",
      "details": { "registered": 8 }
    },
    {
      "component": "background_tasks",
      "status": "healthy",
      "details": { "running": 3 }
    }
  ]
}
```

**Implementation:**

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct StartupState {
    pub migrations_complete: AtomicBool,
    pub handlers_registered: AtomicBool,
    pub background_tasks_started: AtomicBool,
}

impl StartupState {
    pub fn new() -> Self {
        Self {
            migrations_complete: AtomicBool::new(false),
            handlers_registered: AtomicBool::new(false),
            background_tasks_started: AtomicBool::new(false),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.migrations_complete.load(Ordering::Relaxed)
            && self.handlers_registered.load(Ordering::Relaxed)
            && self.background_tasks_started.load(Ordering::Relaxed)
    }
}

pub async fn startup_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let startup = &state.startup_state;

    let migrations_ok = startup.migrations_complete.load(Ordering::Relaxed);
    let handlers_ok = startup.handlers_registered.load(Ordering::Relaxed);
    let tasks_ok = startup.background_tasks_started.load(Ordering::Relaxed);

    let checks = vec![
        HealthCheckResult {
            component: "migrations".to_string(),
            status: if migrations_ok {
                HealthStatus::Healthy
            } else {
                HealthStatus::Unhealthy { reason: "Migrations not complete".to_string() }
            },
            latency_ms: None,
            details: None,
        },
        HealthCheckResult {
            component: "event_handlers".to_string(),
            status: if handlers_ok {
                HealthStatus::Healthy
            } else {
                HealthStatus::Unhealthy { reason: "Handlers not registered".to_string() }
            },
            latency_ms: None,
            details: None,
        },
        HealthCheckResult {
            component: "background_tasks".to_string(),
            status: if tasks_ok {
                HealthStatus::Healthy
            } else {
                HealthStatus::Unhealthy { reason: "Tasks not started".to_string() }
            },
            latency_ms: None,
            details: None,
        },
    ];

    let overall_status = if startup.is_ready() {
        HealthStatus::Healthy
    } else {
        HealthStatus::Unhealthy { reason: "Startup not complete".to_string() }
    };

    let response = HealthResponse {
        status: overall_status.clone(),
        timestamp: Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks,
    };

    (overall_status.http_status_code(), Json(response))
}
```

---

## Component Health Checkers

### Database Health Check

```rust
pub async fn check_database(pool: &PgPool) -> HealthCheckResult {
    let start = std::time::Instant::now();

    match sqlx::query("SELECT 1").fetch_one(pool).await {
        Ok(_) => {
            let latency = start.elapsed();
            let size = pool.size();
            let idle = pool.num_idle();
            let utilization = ((size - idle as u32) as f64 / size as f64) * 100.0;

            let status = if utilization > 90.0 {
                HealthStatus::Degraded {
                    reason: format!("Pool utilization: {:.1}%", utilization),
                }
            } else if latency > Duration::from_millis(500) {
                HealthStatus::Degraded {
                    reason: format!("High latency: {}ms", latency.as_millis()),
                }
            } else {
                HealthStatus::Healthy
            };

            HealthCheckResult {
                component: "database".to_string(),
                status,
                latency_ms: Some(latency.as_millis() as u64),
                details: Some(serde_json::json!({
                    "pool_size": size,
                    "pool_idle": idle,
                    "utilization_percent": utilization,
                })),
            }
        }
        Err(e) => HealthCheckResult {
            component: "database".to_string(),
            status: HealthStatus::Unhealthy { reason: e.to_string() },
            latency_ms: Some(start.elapsed().as_millis() as u64),
            details: None,
        },
    }
}
```

### Redis Health Check

```rust
pub async fn check_redis(client: &Option<redis::Client>) -> HealthCheckResult {
    let Some(client) = client else {
        return HealthCheckResult {
            component: "redis".to_string(),
            status: HealthStatus::Healthy, // Redis is optional
            latency_ms: None,
            details: Some(serde_json::json!({ "configured": false })),
        };
    };

    let start = std::time::Instant::now();

    match client.get_async_connection().await {
        Ok(mut conn) => {
            match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
                Ok(_) => HealthCheckResult {
                    component: "redis".to_string(),
                    status: HealthStatus::Healthy,
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                    details: Some(serde_json::json!({ "configured": true })),
                },
                Err(e) => HealthCheckResult {
                    component: "redis".to_string(),
                    status: HealthStatus::Unhealthy { reason: e.to_string() },
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                    details: None,
                },
            }
        }
        Err(e) => HealthCheckResult {
            component: "redis".to_string(),
            status: HealthStatus::Unhealthy { reason: e.to_string() },
            latency_ms: Some(start.elapsed().as_millis() as u64),
            details: None,
        },
    }
}
```

### Auth Service Health Check

```rust
pub async fn check_auth(auth_service: &dyn AuthService) -> HealthCheckResult {
    let start = std::time::Instant::now();

    // Check if we can reach the auth service's well-known endpoint
    match auth_service.health_check().await {
        Ok(_) => HealthCheckResult {
            component: "auth".to_string(),
            status: HealthStatus::Healthy,
            latency_ms: Some(start.elapsed().as_millis() as u64),
            details: None,
        },
        Err(e) => {
            // Auth is critical - if it's down, we can't authenticate new requests
            // But existing sessions with cached tokens might still work
            HealthCheckResult {
                component: "auth".to_string(),
                status: HealthStatus::Degraded {
                    reason: format!("Auth service unreachable: {}", e),
                },
                latency_ms: Some(start.elapsed().as_millis() as u64),
                details: None,
            }
        }
    }
}
```

---

## Kubernetes Configuration

### Pod Spec

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: choice-sherpa
spec:
  containers:
    - name: app
      image: choice-sherpa:1.0.0
      ports:
        - containerPort: 3000

      # Startup probe - wait for initialization
      startupProbe:
        httpGet:
          path: /health/startup
          port: 3000
        initialDelaySeconds: 5
        periodSeconds: 5
        failureThreshold: 30  # 5 * 30 = 150s max startup time
        timeoutSeconds: 5

      # Liveness probe - restart if dead
      livenessProbe:
        httpGet:
          path: /health/live
          port: 3000
        initialDelaySeconds: 0  # Start after startup probe succeeds
        periodSeconds: 10
        failureThreshold: 3
        timeoutSeconds: 5

      # Readiness probe - traffic routing
      readinessProbe:
        httpGet:
          path: /health/ready
          port: 3000
        initialDelaySeconds: 0  # Start after startup probe succeeds
        periodSeconds: 5
        failureThreshold: 3
        successThreshold: 1
        timeoutSeconds: 10

      resources:
        requests:
          memory: "256Mi"
          cpu: "100m"
        limits:
          memory: "1Gi"
          cpu: "1000m"
```

### Service Configuration

```yaml
apiVersion: v1
kind: Service
metadata:
  name: choice-sherpa
spec:
  selector:
    app: choice-sherpa
  ports:
    - port: 80
      targetPort: 3000
  # Only route to pods that pass readiness probe
  type: ClusterIP
```

---

## Docker Health Checks

For Docker Compose or standalone Docker deployments:

```dockerfile
FROM rust:1.75 as builder
# ... build steps ...

FROM debian:bookworm-slim
COPY --from=builder /app/choice-sherpa /usr/local/bin/

HEALTHCHECK --interval=30s --timeout=5s --start-period=60s --retries=3 \
  CMD curl -f http://localhost:3000/health/ready || exit 1

CMD ["choice-sherpa"]
```

Docker Compose:

```yaml
services:
  app:
    image: choice-sherpa:1.0.0
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health/ready"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 60s
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
```

---

## Graceful Shutdown

### Shutdown Handler

```rust
use tokio::signal;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct ShutdownController {
    sender: broadcast::Sender<()>,
}

impl ShutdownController {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.sender.subscribe()
    }

    pub fn trigger(&self) {
        let _ = self.sender.send(());
    }
}

pub async fn graceful_shutdown(
    shutdown: Arc<ShutdownController>,
    db_pool: PgPool,
    redis_client: Option<redis::Client>,
) {
    // Wait for shutdown signal
    let ctrl_c = async {
        signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, starting graceful shutdown");

    // Notify all subscribers
    shutdown.trigger();

    // Mark service as not ready (fail readiness probe)
    // This removes us from load balancer

    // Wait for in-flight requests (configurable grace period)
    tracing::info!("Waiting for in-flight requests to complete...");
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Close database connections
    tracing::info!("Closing database connections...");
    db_pool.close().await;

    // Close Redis connections
    if let Some(_client) = redis_client {
        tracing::info!("Closing Redis connections...");
        // Redis client cleanup
    }

    tracing::info!("Graceful shutdown complete");
}
```

### Integration with Axum

```rust
pub async fn run_server(config: ServerConfig) -> Result<(), AppError> {
    let shutdown = Arc::new(ShutdownController::new());
    let shutdown_clone = shutdown.clone();

    // Create app state
    let state = AppState::new(config).await?;
    let db_pool = state.db_pool.clone();
    let redis_client = state.redis_client.clone();

    // Build router with health endpoints
    let app = Router::new()
        .route("/health/live", get(liveness_handler))
        .route("/health/ready", get(readiness_handler))
        .route("/health/startup", get(startup_handler))
        .nest("/api", api_routes())
        .with_state(state);

    // Mark startup complete
    state.startup_state.migrations_complete.store(true, Ordering::Relaxed);
    state.startup_state.handlers_registered.store(true, Ordering::Relaxed);
    state.startup_state.background_tasks_started.store(true, Ordering::Relaxed);

    let listener = TcpListener::bind(&config.bind_address).await?;
    tracing::info!("Server listening on {}", config.bind_address);

    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let mut rx = shutdown_clone.subscribe();
            let _ = rx.recv().await;
        })
        .await?;

    // Perform cleanup
    graceful_shutdown(shutdown, db_pool, redis_client).await;

    Ok(())
}
```

---

## Metrics and Alerting

### Health Check Metrics

```rust
use prometheus::{IntCounter, IntGauge, Histogram};

pub struct HealthMetrics {
    pub checks_total: IntCounter,
    pub checks_failed: IntCounter,
    pub check_duration: Histogram,
    pub ready: IntGauge,
}

impl HealthMetrics {
    pub fn new() -> Self {
        Self {
            checks_total: IntCounter::new(
                "health_checks_total",
                "Total health checks performed"
            ).unwrap(),
            checks_failed: IntCounter::new(
                "health_checks_failed_total",
                "Total failed health checks"
            ).unwrap(),
            check_duration: Histogram::with_opts(
                HistogramOpts::new(
                    "health_check_duration_seconds",
                    "Health check duration"
                )
            ).unwrap(),
            ready: IntGauge::new(
                "app_ready",
                "Whether the application is ready (1=ready, 0=not ready)"
            ).unwrap(),
        }
    }
}
```

### Alert Rules

```yaml
groups:
  - name: choice-sherpa-health
    rules:
      - alert: AppNotReady
        expr: app_ready == 0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Choice Sherpa not ready"
          description: "Application has been not ready for 5 minutes"

      - alert: HealthChecksFailing
        expr: rate(health_checks_failed_total[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Health checks failing"
          description: "More than 10% of health checks failing"

      - alert: DatabaseConnectionPoolExhausted
        expr: db_connections_active / db_connections_total > 0.9
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Database connection pool exhausted"
          description: "Database pool utilization > 90%"
```

---

## File Structure

```
backend/src/
├── infrastructure/
│   ├── health/
│   │   ├── mod.rs              # Module exports
│   │   ├── types.rs            # HealthStatus, HealthCheckResult
│   │   ├── handlers.rs         # HTTP handlers
│   │   ├── checkers.rs         # Component checkers
│   │   ├── startup.rs          # StartupState
│   │   └── shutdown.rs         # Graceful shutdown
│   └── mod.rs
└── adapters/
    └── http/
        └── routes/
            └── health.rs       # Route registration
```

---

## Related Documents

- **Database Pool**: `features/infrastructure/database-connection-pool.md`
- **Observability**: `features/infrastructure/observability.md`
- **System Architecture**: `docs/architecture/SYSTEM-ARCHITECTURE.md`

---

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Conditional - see detail filtering below |
| Authorization Model | Public endpoints return minimal info; detailed info requires internal access |
| Sensitive Data | Pool metrics (Internal), Connection details (Internal) |
| Rate Limiting | Not Required (low-cost endpoints) |
| Audit Logging | Not Required (operational endpoints) |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| Basic health status | Public | Safe to expose externally |
| Component latencies | Public | Safe to expose externally |
| Pool size/utilization | Internal | Only expose to internal monitoring |
| Connection strings | Restricted | Never expose |
| Error messages (detailed) | Internal | Sanitize for external responses |

### Detail Filtering for External vs Internal

Health endpoints MUST support an `include_details` parameter to control information exposure:

```rust
#[derive(Debug, Deserialize)]
pub struct HealthQueryParams {
    /// Include detailed metrics (pool sizes, etc.)
    /// Only honored for internal requests
    #[serde(default)]
    pub include_details: bool,
}

pub async fn readiness_handler(
    State(state): State<AppState>,
    Query(params): Query<HealthQueryParams>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Check if request is from internal network
    let is_internal = is_internal_request(&headers, &state.config);

    // Only include details if internal AND requested
    let include_details = params.include_details && is_internal;

    let checks = tokio::join!(
        check_database(&state.db_pool, include_details),
        check_redis(&state.redis_client, include_details),
        check_auth(&state.auth_service),
    );

    // ... build response
}

fn is_internal_request(headers: &HeaderMap, config: &Config) -> bool {
    // Check for internal network header (set by ingress/load balancer)
    if let Some(internal_header) = headers.get("X-Internal-Request") {
        if internal_header == config.internal_request_secret.as_bytes() {
            return true;
        }
    }

    // Or check source IP against internal CIDR ranges
    // (implementation depends on infrastructure)

    false
}
```

### Public vs Internal Response Examples

**External Response (default):**
```json
{
  "status": "healthy",
  "timestamp": "2026-01-08T10:30:00Z",
  "version": "1.0.0",
  "checks": [
    { "component": "database", "status": "healthy", "latency_ms": 5 },
    { "component": "redis", "status": "healthy", "latency_ms": 2 },
    { "component": "auth", "status": "healthy", "latency_ms": 45 }
  ]
}
```

**Internal Response (include_details=true from internal network):**
```json
{
  "status": "healthy",
  "timestamp": "2026-01-08T10:30:00Z",
  "version": "1.0.0",
  "checks": [
    {
      "component": "database",
      "status": "healthy",
      "latency_ms": 5,
      "details": {
        "pool_size": 20,
        "pool_idle": 15,
        "pool_active": 5,
        "utilization_percent": 25.0,
        "max_lifetime_seconds": 1800
      }
    },
    {
      "component": "redis",
      "status": "healthy",
      "latency_ms": 2,
      "details": {
        "configured": true,
        "cluster_mode": false
      }
    },
    {
      "component": "auth",
      "status": "healthy",
      "latency_ms": 45
    }
  ]
}
```

### Implementation for Detail Filtering

```rust
pub async fn check_database(pool: &PgPool, include_details: bool) -> HealthCheckResult {
    let start = std::time::Instant::now();

    match sqlx::query("SELECT 1").fetch_one(pool).await {
        Ok(_) => {
            let latency = start.elapsed();

            // Compute metrics internally for status determination
            let size = pool.size();
            let idle = pool.num_idle();
            let utilization = ((size - idle as u32) as f64 / size as f64) * 100.0;

            let status = if utilization > 90.0 {
                HealthStatus::Degraded {
                    reason: "High pool utilization".to_string(), // Sanitized
                }
            } else {
                HealthStatus::Healthy
            };

            // Only include details if requested AND internal
            let details = if include_details {
                Some(serde_json::json!({
                    "pool_size": size,
                    "pool_idle": idle,
                    "pool_active": size - idle as u32,
                    "utilization_percent": utilization,
                }))
            } else {
                None
            };

            HealthCheckResult {
                component: "database".to_string(),
                status,
                latency_ms: Some(latency.as_millis() as u64),
                details,
            }
        }
        Err(e) => HealthCheckResult {
            component: "database".to_string(),
            status: HealthStatus::Unhealthy {
                // Sanitize error message for external responses
                reason: if include_details {
                    e.to_string()
                } else {
                    "Database unavailable".to_string()
                },
            },
            latency_ms: Some(start.elapsed().as_millis() as u64),
            details: None,
        },
    }
}
```

### Security Guidelines

1. **External Exposure**: Only `/health/live` and `/health/ready` (without details) should be exposed externally. Configure ingress to block `include_details=true` from external requests.

2. **Error Sanitization**: Error messages in unhealthy status MUST NOT expose:
   - Connection strings or credentials
   - Internal hostnames or IPs
   - Stack traces or internal paths
   - Database schema information

3. **Metrics Endpoint Separation**: For detailed operational metrics, use a separate `/metrics` endpoint protected by authentication, rather than exposing via health checks.

4. **Startup Probe**: The `/health/startup` endpoint reveals initialization state. Consider restricting to internal access only in production.

---

*Version: 1.0.0*
*Created: 2026-01-08*
