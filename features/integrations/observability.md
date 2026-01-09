# Integration: Observability & Telemetry

**Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md
**Type:** Cross-Cutting Infrastructure
**Priority:** P1 (Required for production deployment)
**Depends On:** foundation module (event infrastructure)

> Comprehensive observability stack with structured logging, distributed tracing, metrics, and alerting.

---

## Overview

Observability enables understanding system behavior in production. For Choice Sherpa, this means tracking user journeys through PrOACT components, monitoring AI costs, debugging conversation flows, and ensuring system health. This specification covers the three pillars of observability: logs, traces, and metrics.

### Three Pillars

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              LOGS                                            │
│   Structured JSON events with context. "What happened?"                      │
│   - Request/response logging                                                 │
│   - Error details with stack traces                                          │
│   - Audit trail for compliance                                               │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                              TRACES                                          │
│   Distributed request flow across services. "How did it flow?"               │
│   - Span hierarchy (request → command → repo → db)                          │
│   - Cross-service correlation                                                │
│   - Latency breakdown                                                        │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                              METRICS                                         │
│   Aggregated numerical measurements. "How is it performing?"                 │
│   - Request rates, latencies, error rates                                   │
│   - AI costs and token usage                                                │
│   - Business metrics (sessions created, cycles completed)                   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Logging** | tracing + tracing-subscriber | Structured logs with spans |
| **Tracing** | OpenTelemetry (OTLP) | Distributed tracing |
| **Metrics** | Prometheus | Time-series metrics |
| **Visualization** | Grafana | Dashboards and alerts |
| **Log Aggregation** | Loki (optional) | Centralized log search |
| **Error Tracking** | Sentry (optional) | Error aggregation |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Application Code                                   │
│   #[instrument] │ info!() │ metrics::counter!()                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                     tracing + OpenTelemetry Layer                            │
│   Span creation │ Context propagation │ Metric recording                    │
└─────────────────────────────────────────────────────────────────────────────┘
          │                         │                         │
          ▼                         ▼                         ▼
┌─────────────────┐   ┌─────────────────────┐   ┌─────────────────────┐
│   JSON Logs     │   │   OTLP Exporter     │   │   Prometheus        │
│   (stdout/file) │   │   (traces)          │   │   (metrics)         │
└─────────────────┘   └─────────────────────┘   └─────────────────────┘
          │                         │                         │
          ▼                         ▼                         ▼
┌─────────────────┐   ┌─────────────────────┐   ┌─────────────────────┐
│   Loki / ELK    │   │   Jaeger / Tempo    │   │   Grafana           │
│   (log search)  │   │   (trace view)      │   │   (dashboards)      │
└─────────────────┘   └─────────────────────┘   └─────────────────────┘
```

---

## Logging

### Structured Logging Configuration

```rust
// backend/src/infrastructure/telemetry/logging.rs

use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub fn init_logging(config: &TelemetryConfig) -> Result<(), TelemetryError> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    let fmt_layer = fmt::layer()
        .json()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_current_span(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    Ok(())
}
```

### Log Format

```json
{
  "timestamp": "2026-01-07T14:32:01.234Z",
  "level": "INFO",
  "target": "choice_sherpa::application::handlers::session",
  "message": "Session created successfully",
  "span": {
    "name": "create_session",
    "session_id": "sess-abc123",
    "user_id": "user-456"
  },
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "span_id": "00f067aa0ba902b7",
  "file": "src/application/handlers/session.rs",
  "line": 42
}
```

### Standard Log Fields

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | ISO 8601 | When the log was emitted |
| `level` | string | TRACE, DEBUG, INFO, WARN, ERROR |
| `target` | string | Module path |
| `message` | string | Human-readable message |
| `trace_id` | string | Distributed trace correlation |
| `span_id` | string | Current span identifier |
| `user_id` | string | Authenticated user (if present) |
| `session_id` | string | Decision session (if applicable) |
| `request_id` | string | HTTP request identifier |

### Logging Macros

```rust
// Structured logging with context
info!(
    session_id = %session.id(),
    user_id = %user.id(),
    title = %session.title(),
    "Session created"
);

// Error with details
error!(
    error = %err,
    user_id = %user.id(),
    "Failed to create session"
);

// Span-based logging (automatically includes span context)
#[tracing::instrument(
    name = "create_session",
    skip(self, command),
    fields(
        session_id,
        user_id = %command.user_id
    )
)]
async fn handle(&self, command: CreateSession) -> Result<Session, DomainError> {
    let session = Session::create(command)?;
    tracing::Span::current().record("session_id", &session.id().to_string());
    info!("Session created successfully");
    Ok(session)
}
```

---

## Distributed Tracing

### OpenTelemetry Setup

```rust
// backend/src/infrastructure/telemetry/tracing.rs

use opentelemetry::{
    global,
    sdk::{
        propagation::TraceContextPropagator,
        trace::{self, Sampler, TracerProvider},
        Resource,
    },
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;

pub fn init_tracing(config: &TelemetryConfig) -> Result<(), TelemetryError> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&config.otlp_endpoint);

    let tracer_provider = TracerProvider::builder()
        .with_batch_exporter(exporter.build_span_exporter()?, trace::BatchConfig::default())
        .with_resource(Resource::new(vec![
            KeyValue::new("service.name", "choice-sherpa"),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            KeyValue::new("deployment.environment", &config.environment),
        ]))
        .with_sampler(Sampler::TraceIdRatioBased(config.sample_rate))
        .build();

    global::set_tracer_provider(tracer_provider);

    // Bridge tracing crate spans to OpenTelemetry
    let otel_layer = tracing_opentelemetry::layer()
        .with_tracer(global::tracer("choice-sherpa"));

    // Add to existing subscriber
    // (integrated with logging setup)

    Ok(())
}
```

### Span Hierarchy

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ HTTP Request: POST /api/conversations/{id}/messages                         │
│ trace_id: abc123                                                            │
└─────────────────────────────────────────────────────────────────────────────┘
    │
    ├── send_message_handler (span_id: def456)
    │   ├── attributes: user_id, conversation_id, component_type
    │   ├── events: message_received
    │   │
    │   ├── validate_access (span_id: ghi789)
    │   │   └── membership_check
    │   │
    │   ├── ai_completion (span_id: jkl012)
    │   │   ├── attributes: provider, model, token_count
    │   │   ├── events: request_sent, response_received
    │   │   └── duration: 1.2s
    │   │
    │   ├── save_message (span_id: mno345)
    │   │   └── postgres_query
    │   │
    │   └── publish_event (span_id: pqr678)
    │       └── redis_publish
    │
    └── response (status: 200, duration: 1.4s)
```

### Context Propagation

```rust
// HTTP middleware for trace context propagation
pub async fn trace_context_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Response {
    // Extract trace context from incoming headers
    let parent_context = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(request.headers()))
    });

    // Create span with parent context
    let span = tracing::info_span!(
        "http_request",
        method = %request.method(),
        path = %request.uri().path(),
        trace_id = tracing::field::Empty,
    );

    // Record trace_id
    if let Some(trace_id) = span.context().span().span_context().trace_id() {
        span.record("trace_id", &trace_id.to_string());
    }

    span.in_scope(|| async {
        let response = next.run(request).await;

        // Inject trace context into response headers for debugging
        // (optional - useful for correlating frontend errors)

        response
    }).await
}
```

### Event Correlation

Events include trace context for correlation:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Trace ID for distributed tracing
    pub trace_id: Option<String>,

    /// Span ID that emitted this event
    pub span_id: Option<String>,

    /// Correlation ID for business flow
    pub correlation_id: Option<String>,

    /// Causation ID (event that caused this event)
    pub causation_id: Option<String>,
}

impl Default for EventMetadata {
    fn default() -> Self {
        // Extract current trace context
        let span = tracing::Span::current();
        let context = span.context();

        Self {
            trace_id: context.span().span_context().trace_id()
                .map(|id| id.to_string()),
            span_id: context.span().span_context().span_id()
                .map(|id| id.to_string()),
            correlation_id: None,
            causation_id: None,
        }
    }
}
```

---

## Metrics

### Prometheus Metrics

```rust
// backend/src/infrastructure/telemetry/metrics.rs

use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::PrometheusBuilder;

pub fn init_metrics(config: &TelemetryConfig) -> Result<PrometheusHandle, TelemetryError> {
    let builder = PrometheusBuilder::new();

    let handle = builder
        .with_http_listener(([0, 0, 0, 0], config.metrics_port))
        .install_recorder()?;

    // Register standard metrics descriptions
    metrics::describe_counter!(
        "http_requests_total",
        "Total number of HTTP requests"
    );
    metrics::describe_histogram!(
        "http_request_duration_seconds",
        "HTTP request duration in seconds"
    );
    metrics::describe_counter!(
        "ai_tokens_total",
        "Total AI tokens consumed"
    );
    metrics::describe_counter!(
        "ai_cost_cents_total",
        "Total AI cost in cents"
    );

    Ok(handle)
}
```

### Metric Categories

#### HTTP Metrics

```rust
// Recorded by middleware
counter!("http_requests_total", "method" => method, "path" => path, "status" => status);
histogram!("http_request_duration_seconds", duration, "method" => method, "path" => path);
```

#### Business Metrics

```rust
// Sessions
counter!("sessions_created_total", "tier" => tier);
counter!("sessions_archived_total");
gauge!("sessions_active", active_count);

// Cycles
counter!("cycles_created_total");
counter!("cycles_branched_total");
counter!("cycles_completed_total");

// Components
counter!("components_started_total", "component_type" => component_type);
counter!("components_completed_total", "component_type" => component_type);
histogram!("component_duration_seconds", duration, "component_type" => component_type);

// Conversations
counter!("messages_sent_total", "role" => role);
histogram!("message_length_chars", length, "role" => role);
```

#### AI Metrics

```rust
// Token consumption
counter!("ai_tokens_total", "provider" => provider, "model" => model, "type" => "prompt");
counter!("ai_tokens_total", "provider" => provider, "model" => model, "type" => "completion");

// Cost tracking
counter!("ai_cost_cents_total", "provider" => provider, "model" => model, "tier" => tier);

// Latency
histogram!("ai_completion_duration_seconds", duration, "provider" => provider, "streaming" => streaming);

// Errors
counter!("ai_errors_total", "provider" => provider, "error_type" => error_type);
```

#### Infrastructure Metrics

```rust
// Event bus
counter!("events_published_total", "event_type" => event_type);
counter!("events_processed_total", "event_type" => event_type, "handler" => handler);
counter!("events_failed_total", "event_type" => event_type, "handler" => handler);
histogram!("event_processing_duration_seconds", duration, "event_type" => event_type);

// Rate limiting
counter!("rate_limit_hits_total", "scope" => scope, "resource" => resource);

// Database
histogram!("db_query_duration_seconds", duration, "operation" => operation);
counter!("db_connections_total");
gauge!("db_pool_size", pool_size);

// WebSocket
gauge!("websocket_connections_active", connections);
counter!("websocket_messages_sent_total");
```

### Metric Recording Patterns

```rust
// Request timing middleware
pub async fn metrics_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let start = Instant::now();

    let response = next.run(request).await;

    let duration = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    counter!("http_requests_total",
        "method" => method.clone(),
        "path" => normalize_path(&path),
        "status" => status
    );

    histogram!("http_request_duration_seconds", duration,
        "method" => method,
        "path" => normalize_path(&path)
    );

    response
}

fn normalize_path(path: &str) -> String {
    // Replace IDs with placeholders: /sessions/abc123 → /sessions/{id}
    let re = regex::Regex::new(r"/[a-f0-9-]{36}").unwrap();
    re.replace_all(path, "/{id}").to_string()
}
```

---

## Alerting

### Alert Definitions

```yaml
# prometheus/alerts.yml

groups:
  - name: choice-sherpa
    rules:
      # High error rate
      - alert: HighErrorRate
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[5m]))
          /
          sum(rate(http_requests_total[5m]))
          > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value | humanizePercentage }} over the last 5 minutes"

      # AI provider issues
      - alert: AIProviderErrors
        expr: |
          sum(rate(ai_errors_total[5m])) > 0.1
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "AI provider errors increasing"
          description: "{{ $value }} AI errors per second"

      # High AI costs
      - alert: HighAICosts
        expr: |
          sum(increase(ai_cost_cents_total[1h])) > 10000
        labels:
          severity: warning
        annotations:
          summary: "High AI costs in last hour"
          description: "AI costs: ${{ $value | humanize }}"

      # Slow responses
      - alert: SlowResponses
        expr: |
          histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 2
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "95th percentile latency is high"
          description: "P95 latency is {{ $value }}s"

      # Event processing backlog
      - alert: EventBacklog
        expr: |
          sum(events_published_total) - sum(events_processed_total) > 1000
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Event processing backlog growing"
          description: "{{ $value }} events pending"

      # Rate limiting spike
      - alert: RateLimitSpike
        expr: |
          sum(rate(rate_limit_hits_total[5m])) > 10
        for: 5m
        labels:
          severity: info
        annotations:
          summary: "High rate limit hits"
          description: "{{ $value }} rate limit hits per second"
```

---

## Dashboards

### Grafana Dashboard Structure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Choice Sherpa Overview                               │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐ │
│  │ Requests  │  │ Error     │  │ P95       │  │ Active    │  │ AI Cost   │ │
│  │ /sec      │  │ Rate      │  │ Latency   │  │ Sessions  │  │ Today     │ │
│  │   142     │  │   0.2%    │  │   320ms   │  │    89     │  │  $12.34   │ │
│  └───────────┘  └───────────┘  └───────────┘  └───────────┘  └───────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Request Rate                          Error Rate                            │
│  ┌────────────────────────┐            ┌────────────────────────┐           │
│  │    ▁▂▃▅▆▇█▇▆▅▃▂▁       │            │    ─────────▁──────    │           │
│  │                        │            │                        │           │
│  └────────────────────────┘            └────────────────────────┘           │
│                                                                              │
│  Latency Distribution                  AI Token Usage                        │
│  ┌────────────────────────┐            ┌────────────────────────┐           │
│  │  P50: 120ms            │            │    ▁▂▅▇█▇▅▃▂▁          │           │
│  │  P95: 320ms            │            │                        │           │
│  │  P99: 890ms            │            └────────────────────────┘           │
│  └────────────────────────┘                                                  │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                         PrOACT Journey Funnel                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Sessions → Cycles → Issue Raising → Problem Frame → ... → Decision Quality │
│                                                                              │
│  ████████████████████████████████████████████████████████████  100%          │
│  ████████████████████████████████████████████████             82%           │
│  ████████████████████████████████████████                     65%           │
│  ████████████████████████████████                             52%           │
│  █████████████████████████                                    41%           │
│  ███████████████████                                          30%           │
│  █████████████                                                21%           │
│  ████████                                                     13%           │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Dashboard JSON Template

```json
{
  "dashboard": {
    "title": "Choice Sherpa Overview",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [{
          "expr": "sum(rate(http_requests_total[1m]))",
          "legendFormat": "Requests/sec"
        }]
      },
      {
        "title": "Error Rate",
        "type": "graph",
        "targets": [{
          "expr": "sum(rate(http_requests_total{status=~\"5..\"}[1m])) / sum(rate(http_requests_total[1m])) * 100",
          "legendFormat": "Error %"
        }]
      },
      {
        "title": "AI Cost by Provider",
        "type": "graph",
        "targets": [{
          "expr": "sum by (provider) (increase(ai_cost_cents_total[1h])) / 100",
          "legendFormat": "{{provider}}"
        }]
      },
      {
        "title": "Component Completion Funnel",
        "type": "bar",
        "targets": [{
          "expr": "sum by (component_type) (components_completed_total)",
          "legendFormat": "{{component_type}}"
        }]
      }
    ]
  }
}
```

---

## Error Tracking (Sentry)

Optional Sentry integration for error aggregation:

```rust
// backend/src/infrastructure/telemetry/sentry.rs

use sentry::{ClientOptions, IntoDsn};

pub fn init_sentry(config: &TelemetryConfig) -> Option<sentry::ClientInitGuard> {
    config.sentry_dsn.as_ref().map(|dsn| {
        sentry::init((
            dsn.clone(),
            ClientOptions {
                release: Some(env!("CARGO_PKG_VERSION").into()),
                environment: Some(config.environment.clone().into()),
                attach_stacktrace: true,
                send_default_pii: false,  // GDPR compliance
                ..Default::default()
            },
        ))
    })
}

// Error capture with context
pub fn capture_error(error: &dyn std::error::Error, context: ErrorContext) {
    sentry::with_scope(
        |scope| {
            scope.set_tag("module", &context.module);
            scope.set_user(context.user_id.map(|id| sentry::User {
                id: Some(id),
                ..Default::default()
            }));
            if let Some(session_id) = context.session_id {
                scope.set_extra("session_id", session_id.into());
            }
        },
        || sentry::capture_error(error),
    );
}

pub struct ErrorContext {
    pub module: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
}
```

---

## Health Checks

```rust
// backend/src/adapters/http/routes/health.rs

/// GET /health
/// Basic liveness check
pub async fn health() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

/// GET /health/ready
/// Readiness check with dependency verification
pub async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    let mut checks = vec![];

    // Database check
    let db_ok = state.db_pool.acquire().await.is_ok();
    checks.push(("database", db_ok));

    // Redis check
    let redis_ok = state.redis.ping().await.is_ok();
    checks.push(("redis", redis_ok));

    // AI provider check (optional, non-blocking)
    let ai_ok = state.ai_provider.provider_info().name.len() > 0;
    checks.push(("ai_provider", ai_ok));

    let all_ok = checks.iter().all(|(_, ok)| *ok);
    let status = if all_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(json!({
        "status": if all_ok { "ready" } else { "not_ready" },
        "checks": checks.into_iter()
            .map(|(name, ok)| (name, if ok { "ok" } else { "fail" }))
            .collect::<HashMap<_, _>>()
    })))
}

/// GET /metrics
/// Prometheus metrics endpoint (handled by metrics exporter)
```

---

## Configuration

```rust
// backend/src/config/telemetry.rs

#[derive(Debug, Clone, Deserialize)]
pub struct TelemetryConfig {
    /// Log level (TRACE, DEBUG, INFO, WARN, ERROR)
    pub log_level: String,

    /// OTLP endpoint for trace export
    pub otlp_endpoint: String,

    /// Trace sampling rate (0.0 - 1.0)
    pub sample_rate: f64,

    /// Prometheus metrics port
    pub metrics_port: u16,

    /// Environment name (dev, staging, prod)
    pub environment: String,

    /// Sentry DSN (optional)
    pub sentry_dsn: Option<String>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            log_level: "INFO".to_string(),
            otlp_endpoint: "http://localhost:4317".to_string(),
            sample_rate: 0.1,  // 10% sampling in production
            metrics_port: 9090,
            environment: "development".to_string(),
            sentry_dsn: None,
        }
    }
}
```

### Environment Variables

```bash
# .env
LOG_LEVEL=INFO
OTLP_ENDPOINT=http://otel-collector:4317
TRACE_SAMPLE_RATE=0.1
METRICS_PORT=9090
ENVIRONMENT=production
SENTRY_DSN=https://xxx@sentry.io/yyy
```

---

## Docker Compose Stack

```yaml
# docker-compose.observability.yml

version: '3.8'

services:
  # OpenTelemetry Collector
  otel-collector:
    image: otel/opentelemetry-collector:latest
    command: ["--config=/etc/otel-collector-config.yaml"]
    volumes:
      - ./otel-collector-config.yaml:/etc/otel-collector-config.yaml
    ports:
      - "4317:4317"   # OTLP gRPC
      - "4318:4318"   # OTLP HTTP

  # Jaeger for trace visualization
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686" # UI
      - "14250:14250" # gRPC

  # Prometheus for metrics
  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - ./alerts.yml:/etc/prometheus/alerts.yml
    ports:
      - "9091:9090"

  # Grafana for dashboards
  grafana:
    image: grafana/grafana:latest
    volumes:
      - ./grafana/provisioning:/etc/grafana/provisioning
      - ./grafana/dashboards:/var/lib/grafana/dashboards
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin

  # Loki for log aggregation (optional)
  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"
```

---

## Testing Observability

```rust
#[tokio::test]
async fn test_request_metrics_recorded() {
    let app = create_test_app().await;

    // Make requests
    app.oneshot(Request::get("/api/sessions").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Verify metrics
    let metrics = app.state().metrics_handle.render();
    assert!(metrics.contains("http_requests_total"));
    assert!(metrics.contains("http_request_duration_seconds"));
}

#[tokio::test]
async fn test_trace_context_propagated() {
    let app = create_test_app().await;

    let trace_id = "4bf92f3577b34da6a3ce929d0e0e4736";
    let response = app
        .oneshot(
            Request::get("/api/sessions")
                .header("traceparent", format!("00-{}-00f067aa0ba902b7-01", trace_id))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // Verify trace context preserved in logs
    // (check captured log output)
}

#[tokio::test]
async fn test_error_includes_context() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::get("/api/sessions/nonexistent")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Verify error logged with context
    // (check captured log includes user_id, trace_id)
}
```

---

## Implementation Phases

### Phase 1: Structured Logging

- [ ] Configure tracing-subscriber with JSON format
- [ ] Add request/response logging middleware
- [ ] Standardize log fields across modules
- [ ] Add #[instrument] to key handlers
- [ ] Write unit tests for log format

### Phase 2: Distributed Tracing

- [ ] Configure OpenTelemetry exporter
- [ ] Add trace context propagation middleware
- [ ] Include trace_id in EventMetadata
- [ ] Set up Jaeger locally
- [ ] Verify trace visibility

### Phase 3: Metrics

- [ ] Configure Prometheus exporter
- [ ] Add HTTP metrics middleware
- [ ] Implement business metrics recording
- [ ] Add AI cost metrics
- [ ] Set up Prometheus locally

### Phase 4: Dashboards & Alerts

- [ ] Create Grafana overview dashboard
- [ ] Create PrOACT journey dashboard
- [ ] Create AI cost dashboard
- [ ] Define alerting rules
- [ ] Test alert triggers

### Phase 5: Production Stack

- [ ] Deploy OpenTelemetry Collector
- [ ] Configure trace sampling
- [ ] Set up log aggregation (Loki or similar)
- [ ] Configure Sentry (optional)
- [ ] Document runbook for common issues

---

## Exit Criteria

1. **Structured logs**: All logs JSON-formatted with standard fields
2. **Traces visible**: Request flow traceable in Jaeger/Tempo
3. **Metrics exported**: Prometheus scrapes all key metrics
4. **Dashboards functional**: Overview dashboard shows system health
5. **Alerts configured**: Critical alerts fire appropriately
6. **Context propagated**: trace_id visible across logs, traces, events
7. **Documentation**: Runbook for common debugging scenarios
