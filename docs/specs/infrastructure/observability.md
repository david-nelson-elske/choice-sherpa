# Infrastructure: Observability

**Type:** Cross-Cutting Infrastructure
**Priority:** P1 (Required for production operations)
**Last Updated:** 2026-01-08

> Complete specification for metrics, distributed tracing, and structured logging across Choice Sherpa.

---

## Overview

Observability enables understanding system behavior through three pillars:

| Pillar | Purpose | Technology |
|--------|---------|------------|
| **Metrics** | Quantitative measurements over time | Prometheus + Grafana |
| **Tracing** | Request flow across services | OpenTelemetry + Jaeger |
| **Logging** | Event records for debugging | Structured JSON + ELK/Loki |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          Choice Sherpa Application                           │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     Instrumentation Layer                            │   │
│   │                                                                      │   │
│   │   Metrics:     prometheus::Registry                                  │   │
│   │   Tracing:     tracing + opentelemetry                              │   │
│   │   Logging:     tracing-subscriber + JSON formatter                   │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     HTTP Middleware                                  │   │
│   │                                                                      │   │
│   │   tower_http::trace::TraceLayer                                     │   │
│   │   - Request/response logging                                        │   │
│   │   - Latency measurement                                             │   │
│   │   - Status code tracking                                            │   │
│   │   - Trace propagation (W3C Trace Context)                          │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└──────────────┬──────────────────────┬───────────────────────┬───────────────┘
               │                      │                       │
               ▼                      ▼                       ▼
     ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
     │   Prometheus    │    │     Jaeger      │    │    Loki/ELK     │
     │   (Metrics)     │    │   (Tracing)     │    │   (Logging)     │
     └────────┬────────┘    └────────┬────────┘    └────────┬────────┘
              │                      │                       │
              └──────────────────────┼───────────────────────┘
                                     ▼
                          ┌─────────────────┐
                          │     Grafana     │
                          │  (Dashboards)   │
                          └─────────────────┘
```

---

## Metrics

### Metrics Registry

```rust
use prometheus::{
    Counter, CounterVec, Histogram, HistogramVec, Gauge, GaugeVec,
    Opts, HistogramOpts, Registry, Encoder, TextEncoder,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    // HTTP Metrics
    pub static ref HTTP_REQUESTS_TOTAL: CounterVec = CounterVec::new(
        Opts::new("http_requests_total", "Total HTTP requests"),
        &["method", "endpoint", "status"]
    ).unwrap();

    pub static ref HTTP_REQUEST_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "http_request_duration_seconds",
            "HTTP request duration in seconds"
        ).buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
        &["method", "endpoint"]
    ).unwrap();

    pub static ref HTTP_REQUESTS_IN_FLIGHT: Gauge = Gauge::new(
        "http_requests_in_flight",
        "Current number of HTTP requests being processed"
    ).unwrap();

    // Database Metrics
    pub static ref DB_CONNECTIONS_TOTAL: Gauge = Gauge::new(
        "db_connections_total",
        "Total database connections in pool"
    ).unwrap();

    pub static ref DB_CONNECTIONS_IDLE: Gauge = Gauge::new(
        "db_connections_idle",
        "Idle database connections"
    ).unwrap();

    pub static ref DB_QUERY_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "db_query_duration_seconds",
            "Database query duration"
        ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
        &["query_type"]
    ).unwrap();

    pub static ref DB_QUERIES_TOTAL: CounterVec = CounterVec::new(
        Opts::new("db_queries_total", "Total database queries"),
        &["query_type", "result"]
    ).unwrap();

    // Event Bus Metrics
    pub static ref EVENTS_PUBLISHED_TOTAL: CounterVec = CounterVec::new(
        Opts::new("events_published_total", "Total events published"),
        &["event_type"]
    ).unwrap();

    pub static ref EVENTS_PROCESSED_TOTAL: CounterVec = CounterVec::new(
        Opts::new("events_processed_total", "Total events processed"),
        &["event_type", "handler", "result"]
    ).unwrap();

    pub static ref EVENT_PROCESSING_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "event_processing_duration_seconds",
            "Event processing duration"
        ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
        &["event_type", "handler"]
    ).unwrap();

    pub static ref EVENT_OUTBOX_PENDING: Gauge = Gauge::new(
        "event_outbox_pending",
        "Number of pending events in outbox"
    ).unwrap();

    // Business Metrics
    pub static ref SESSIONS_CREATED_TOTAL: Counter = Counter::new(
        "sessions_created_total",
        "Total sessions created"
    ).unwrap();

    pub static ref CYCLES_COMPLETED_TOTAL: Counter = Counter::new(
        "cycles_completed_total",
        "Total PrOACT cycles completed"
    ).unwrap();

    pub static ref AI_REQUESTS_TOTAL: CounterVec = CounterVec::new(
        Opts::new("ai_requests_total", "Total AI API requests"),
        &["provider", "model", "result"]
    ).unwrap();

    pub static ref AI_REQUEST_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "ai_request_duration_seconds",
            "AI API request duration"
        ).buckets(vec![0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0]),
        &["provider", "model"]
    ).unwrap();

    pub static ref AI_TOKENS_USED: CounterVec = CounterVec::new(
        Opts::new("ai_tokens_used_total", "Total AI tokens used"),
        &["provider", "model", "type"]  // type: prompt, completion
    ).unwrap();

    // WebSocket Metrics
    pub static ref WS_CONNECTIONS_ACTIVE: Gauge = Gauge::new(
        "ws_connections_active",
        "Active WebSocket connections"
    ).unwrap();

    pub static ref WS_MESSAGES_SENT_TOTAL: Counter = Counter::new(
        "ws_messages_sent_total",
        "Total WebSocket messages sent"
    ).unwrap();
}

pub fn register_metrics() {
    REGISTRY.register(Box::new(HTTP_REQUESTS_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(HTTP_REQUEST_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(HTTP_REQUESTS_IN_FLIGHT.clone())).unwrap();
    REGISTRY.register(Box::new(DB_CONNECTIONS_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(DB_CONNECTIONS_IDLE.clone())).unwrap();
    REGISTRY.register(Box::new(DB_QUERY_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(DB_QUERIES_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(EVENTS_PUBLISHED_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(EVENTS_PROCESSED_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(EVENT_PROCESSING_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(EVENT_OUTBOX_PENDING.clone())).unwrap();
    REGISTRY.register(Box::new(SESSIONS_CREATED_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(CYCLES_COMPLETED_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(AI_REQUESTS_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(AI_REQUEST_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(AI_TOKENS_USED.clone())).unwrap();
    REGISTRY.register(Box::new(WS_CONNECTIONS_ACTIVE.clone())).unwrap();
    REGISTRY.register(Box::new(WS_MESSAGES_SENT_TOTAL.clone())).unwrap();
}
```

### Metrics Endpoint

```rust
pub async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        buffer,
    )
}

// Register route
pub fn metrics_routes() -> Router<AppState> {
    Router::new().route("/metrics", get(metrics_handler))
}
```

---

## Metrics Endpoint Security

### Authentication Options

The `/metrics` endpoint exposes operational data that could aid attackers. Choose one:

#### Option 1: Internal Network Only (Recommended)

Bind metrics server to internal network only:

```rust
pub async fn run_metrics_server(config: &MetricsConfig) -> Result<(), Error> {
    let app = Router::new()
        .route("/metrics", get(metrics_handler));

    // SECURITY: Bind to internal network only
    let bind_addr = if config.environment == "production" {
        "127.0.0.1:9090"  // Localhost only
    } else {
        "0.0.0.0:9090"    // All interfaces in dev
    };

    let listener = TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await
}
```

#### Option 2: Authenticated Access

If external access is required, add authentication:

```rust
pub fn metrics_routes() -> Router<AppState> {
    Router::new()
        .route("/metrics", get(metrics_handler))
        .layer(middleware::from_fn(require_metrics_auth))
}

async fn require_metrics_auth(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match auth_token {
        Some(token) if token == std::env::var("METRICS_TOKEN").unwrap_or_default() => {
            Ok(next.run(request).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
```

### Configuration

```bash
# Metrics security configuration
METRICS_BIND_ADDRESS=127.0.0.1:9090  # Internal only (recommended)
METRICS_REQUIRE_AUTH=false           # Or true with METRICS_TOKEN
METRICS_TOKEN=<secure-random-token>  # If auth required
```

### Sensitive Metrics

The following metrics may reveal sensitive operational information:

| Metric | Risk | Mitigation |
|--------|------|------------|
| `ai_tokens_used_total` | Cost/usage patterns | Internal only |
| `http_request_duration_*` | Performance profiling | Internal only |
| `db_connections_*` | Infrastructure sizing | Internal only |
| `sessions_created_total` | Business metrics | Internal only |

---

### HTTP Metrics Middleware

```rust
use axum::{
    middleware::Next,
    http::{Request, StatusCode},
    response::Response,
};
use std::time::Instant;

pub async fn metrics_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    // Normalize path (replace IDs with placeholders)
    let endpoint = normalize_path(&path);

    HTTP_REQUESTS_IN_FLIGHT.inc();
    let start = Instant::now();

    let response = next.run(request).await;

    let status = response.status().as_u16().to_string();
    let duration = start.elapsed().as_secs_f64();

    HTTP_REQUESTS_TOTAL
        .with_label_values(&[&method, &endpoint, &status])
        .inc();

    HTTP_REQUEST_DURATION
        .with_label_values(&[&method, &endpoint])
        .observe(duration);

    HTTP_REQUESTS_IN_FLIGHT.dec();

    response
}

fn normalize_path(path: &str) -> String {
    // Replace UUIDs with :id placeholder
    let uuid_regex = regex::Regex::new(
        r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}"
    ).unwrap();

    uuid_regex.replace_all(path, ":id").to_string()
}
```

---

## Distributed Tracing

### OpenTelemetry Setup

```rust
use opentelemetry::{
    global,
    sdk::{propagation::TraceContextPropagator, trace, Resource},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing(config: &TracingConfig) -> Result<(), TracingError> {
    // Set up trace context propagation
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Configure OTLP exporter
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&config.otlp_endpoint)
        )
        .with_trace_config(
            trace::config()
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", config.service_name.clone()),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                    KeyValue::new("deployment.environment", config.environment.clone()),
                ]))
                .with_sampler(trace::Sampler::TraceIdRatioBased(config.sample_rate))
        )
        .install_batch(opentelemetry::runtime::Tokio)
        .map_err(|e| TracingError::InitFailed(e.to_string()))?;

    // Create OpenTelemetry layer
    let otel_layer = OpenTelemetryLayer::new(tracer);

    // Combine with existing subscriber
    tracing_subscriber::registry()
        .with(otel_layer)
        .with(tracing_subscriber::fmt::layer().json())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    Ok(())
}

#[derive(Debug, Clone)]
pub struct TracingConfig {
    pub service_name: String,
    pub environment: String,
    pub otlp_endpoint: String,
    pub sample_rate: f64,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "choice-sherpa".to_string(),
            environment: "development".to_string(),
            otlp_endpoint: "http://localhost:4317".to_string(),
            sample_rate: 1.0, // Sample everything in development
        }
    }
}
```

### Trace Propagation Middleware

```rust
use axum::{
    http::{Request, HeaderMap},
    middleware::Next,
    response::Response,
};
use opentelemetry::{global, propagation::Extractor};
use tracing::Span;

struct HeaderExtractor<'a>(&'a HeaderMap);

impl<'a> Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

pub async fn trace_propagation_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Response {
    // Extract trace context from incoming headers
    let parent_cx = global::get_text_map_propagator(|prop| {
        prop.extract(&HeaderExtractor(request.headers()))
    });

    // Create span with extracted context
    let span = tracing::info_span!(
        "http_request",
        otel.kind = "server",
        http.method = %request.method(),
        http.url = %request.uri(),
        http.route = tracing::field::Empty,
        http.status_code = tracing::field::Empty,
    );

    span.set_parent(parent_cx);

    // Execute request within span
    let response = {
        let _guard = span.enter();
        next.run(request).await
    };

    // Record response status
    span.record("http.status_code", response.status().as_u16());

    response
}
```

### Instrumented Operations

```rust
use tracing::{instrument, info_span, Instrument};

#[instrument(
    name = "create_session",
    skip(self, pool),
    fields(
        user_id = %cmd.user_id,
        session_id = tracing::field::Empty
    )
)]
pub async fn create_session(
    &self,
    pool: &PgPool,
    cmd: CreateSessionCommand,
) -> Result<SessionId, DomainError> {
    let session = Session::create(cmd.user_id, cmd.title)?;

    // Record session ID in span
    tracing::Span::current().record("session_id", session.id.to_string());

    // Database operation with sub-span
    let insert_span = info_span!("db_insert", table = "sessions");
    sqlx::query!(/* ... */)
        .execute(pool)
        .instrument(insert_span)
        .await?;

    // Event publishing with sub-span
    let publish_span = info_span!("publish_event", event_type = "session.created");
    self.event_publisher
        .publish(SessionCreated::from(&session))
        .instrument(publish_span)
        .await?;

    Ok(session.id)
}
```

---

## Structured Logging

### Log Format

```rust
use tracing_subscriber::{
    fmt::{format::JsonFields, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub fn init_logging(config: &LoggingConfig) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.default_level));

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_timer(UtcTime::rfc_3339())
                .with_current_span(true)
                .with_span_list(true)
                .with_file(true)
                .with_line_number(true)
                .with_thread_ids(true)
                .with_target(true)
                .flatten_event(true)
        );

    subscriber.init();
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub default_level: String,
    pub format: LogFormat,
}

#[derive(Debug, Clone)]
pub enum LogFormat {
    Json,
    Pretty,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            default_level: "info,choice_sherpa=debug,sqlx=warn".to_string(),
            format: LogFormat::Json,
        }
    }
}
```

### Log Output Format

```json
{
  "timestamp": "2026-01-08T10:30:00.123456Z",
  "level": "INFO",
  "target": "choice_sherpa::application::session",
  "message": "Session created successfully",
  "span": {
    "name": "create_session",
    "user_id": "user-123",
    "session_id": "sess-456"
  },
  "spans": [
    { "name": "http_request", "http.method": "POST", "http.url": "/api/sessions" },
    { "name": "create_session", "user_id": "user-123" }
  ],
  "file": "src/application/session/commands.rs",
  "line": 42,
  "thread_id": 7
}
```

### Logging Best Practices

```rust
// Good: Structured context
tracing::info!(
    user_id = %user_id,
    session_id = %session_id,
    "Session created"
);

// Good: Error with context
tracing::error!(
    error = %e,
    user_id = %user_id,
    operation = "create_session",
    "Failed to create session"
);

// Good: Span for operation
let span = tracing::info_span!(
    "process_payment",
    membership_id = %membership_id,
    amount_cents = amount,
);
let _guard = span.enter();

// Bad: Unstructured logging
tracing::info!("Created session {} for user {}", session_id, user_id);

// Bad: Missing context
tracing::error!("Something went wrong");
```

### Log Levels Guide

| Level | Use Case | Examples |
|-------|----------|----------|
| ERROR | Failures requiring attention | DB connection failed, Payment processing error |
| WARN | Recoverable issues | Retry succeeded, Rate limit approaching |
| INFO | Significant business events | Session created, User logged in |
| DEBUG | Detailed flow information | Query executed, Cache hit/miss |
| TRACE | Very detailed debugging | Request/response bodies, Internal state |

---

## Configuration

### Environment Variables

```bash
# Metrics
METRICS_ENABLED=true
METRICS_PORT=9090

# Tracing
TRACING_ENABLED=true
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=choice-sherpa
OTEL_TRACES_SAMPLER=parentbased_traceidratio
OTEL_TRACES_SAMPLER_ARG=0.1  # 10% sampling in production

# Logging
RUST_LOG=info,choice_sherpa=debug,sqlx=warn
LOG_FORMAT=json  # or "pretty" for development
```

### Production Configuration

```bash
# Production settings
OTEL_TRACES_SAMPLER_ARG=0.01  # 1% sampling
RUST_LOG=info,choice_sherpa=info,sqlx=warn

# Send to centralized collector
OTEL_EXPORTER_OTLP_ENDPOINT=https://otel-collector.internal:4317
```

---

## Grafana Dashboards

### Application Overview Dashboard

```json
{
  "title": "Choice Sherpa Overview",
  "panels": [
    {
      "title": "Request Rate",
      "type": "graph",
      "targets": [{
        "expr": "sum(rate(http_requests_total[5m])) by (endpoint)",
        "legendFormat": "{{endpoint}}"
      }]
    },
    {
      "title": "Request Latency (p99)",
      "type": "graph",
      "targets": [{
        "expr": "histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))",
        "legendFormat": "p99"
      }]
    },
    {
      "title": "Error Rate",
      "type": "graph",
      "targets": [{
        "expr": "sum(rate(http_requests_total{status=~\"5..\"}[5m])) / sum(rate(http_requests_total[5m]))",
        "legendFormat": "Error %"
      }]
    },
    {
      "title": "Database Connections",
      "type": "gauge",
      "targets": [{
        "expr": "db_connections_total - db_connections_idle",
        "legendFormat": "Active"
      }]
    },
    {
      "title": "Event Processing Lag",
      "type": "graph",
      "targets": [{
        "expr": "event_outbox_pending",
        "legendFormat": "Pending Events"
      }]
    },
    {
      "title": "AI API Latency",
      "type": "graph",
      "targets": [{
        "expr": "histogram_quantile(0.95, rate(ai_request_duration_seconds_bucket[5m]))",
        "legendFormat": "p95"
      }]
    }
  ]
}
```

---

## Alert Rules

```yaml
groups:
  - name: choice-sherpa-alerts
    rules:
      # High Error Rate
      - alert: HighErrorRate
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[5m]))
          / sum(rate(http_requests_total[5m])) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value | humanizePercentage }}"

      # High Latency
      - alert: HighLatency
        expr: |
          histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m])) > 2
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High request latency"
          description: "p99 latency is {{ $value }}s"

      # Event Processing Backlog
      - alert: EventBacklog
        expr: event_outbox_pending > 1000
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Event outbox backlog"
          description: "{{ $value }} events pending"

      # Database Connection Pool Exhaustion
      - alert: DBPoolExhausted
        expr: (db_connections_total - db_connections_idle) / db_connections_total > 0.9
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Database pool nearly exhausted"
          description: "Pool utilization is {{ $value | humanizePercentage }}"

      # AI API Errors
      - alert: AIAPIErrors
        expr: rate(ai_requests_total{result="error"}[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "AI API errors"
          description: "AI error rate: {{ $value }}/s"
```

---

## File Structure

```
backend/src/
├── infrastructure/
│   ├── observability/
│   │   ├── mod.rs              # Module exports
│   │   ├── metrics.rs          # Prometheus metrics
│   │   ├── tracing.rs          # OpenTelemetry setup
│   │   ├── logging.rs          # Structured logging config
│   │   └── middleware.rs       # HTTP instrumentation
│   └── mod.rs
└── adapters/
    └── http/
        └── routes/
            └── metrics.rs      # /metrics endpoint

monitoring/
├── grafana/
│   └── dashboards/
│       ├── overview.json
│       ├── database.json
│       └── events.json
├── prometheus/
│   └── rules/
│       └── alerts.yaml
└── docker-compose.monitoring.yaml
```

---

## Related Documents

- **Health Checks**: `features/infrastructure/health-checks.md`
- **Database Pool**: `features/infrastructure/database-connection-pool.md`
- **Event Infrastructure**: `features/foundation/event-infrastructure.md`
- **System Architecture**: `docs/architecture/SYSTEM-ARCHITECTURE.md`

---

*Version: 1.0.0*
*Created: 2026-01-08*
