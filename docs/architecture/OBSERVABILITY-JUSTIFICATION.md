# Observability Stack Selection: Sentry + Prometheus/Grafana

> **Decision:** Sentry (errors) + Prometheus/Grafana (metrics)
> **Date:** 2026-01-07

---

## Summary

Dual observability stack selected to provide comprehensive error tracking via Sentry and AI token metrics via Prometheus/Grafana. AI cost visibility is critical for an LLM-centric application from MVP onward.

---

## Requirements

### Error Tracking

| Need | Priority |
|------|----------|
| Crash/exception reporting | Must |
| Stack traces with context | Must |
| Frontend error capture | Should |
| Performance transactions | Should |

### AI Metrics (Critical)

| Metric | Purpose |
|--------|---------|
| Token usage by model | Cost tracking |
| Token usage by component type | Optimization targeting |
| Request latency | Performance monitoring |
| Error rates by type | Reliability monitoring |
| Cost in dollars | Budget management |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Rust Backend (axum)                       │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  AI Adapter (OpenAI)                                │    │
│  │  - Records token metrics after each call            │    │
│  │  - Reports errors to Sentry                         │    │
│  └─────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  /metrics endpoint (Prometheus format)              │    │
│  └─────────────────────────────────────────────────────┘    │
└───────────┬─────────────────────────────────┬───────────────┘
            │ errors                          │ scrape
            ▼                                 ▼
     ┌─────────────┐                   ┌─────────────┐
     │   Sentry    │                   │ Prometheus  │
     │  - Errors   │                   │  - Metrics  │
     │  - Traces   │                   └──────┬──────┘
     └─────────────┘                          │ query
                                              ▼
                                       ┌─────────────┐
                                       │   Grafana   │
                                       │ - Dashboards│
                                       │ - Alerts    │
                                       └─────────────┘
```

---

## Tool Selection

### Sentry (Errors + Performance)

**Purpose:** Error tracking, crash reporting, performance monitoring

**Why Sentry:**
- Best-in-class error tracking UI
- Excellent Rust support (`sentry`, `sentry-tower`)
- SvelteKit SDK available
- Free tier: 5K errors/month, 10K transactions

**Rust Integration:**
```toml
[dependencies]
sentry = "0.34"
sentry-tower = "0.34"
sentry-tracing = "0.34"
```

### Prometheus + Grafana (Metrics)

**Purpose:** AI token metrics, cost tracking, custom counters

**Why Prometheus:**
- Industry standard for metrics
- Native Rust support (`metrics` crate)
- Flexible querying (PromQL)
- Grafana Cloud free tier: 10K metrics

**Rust Integration:**
```toml
[dependencies]
metrics = "0.23"
metrics-exporter-prometheus = "0.15"
```

---

## AI Metrics Specification

### Counters

| Metric | Labels | Description |
|--------|--------|-------------|
| `ai_tokens_input_total` | model, component_type | Total input tokens |
| `ai_tokens_output_total` | model, component_type | Total output tokens |
| `ai_request_errors_total` | model, error_type | Failed requests |
| `ai_cost_dollars_total` | model | Estimated cost |

### Histograms

| Metric | Labels | Description |
|--------|--------|-------------|
| `ai_request_duration_seconds` | model, component_type | Request latency |

### Labels

| Label | Values |
|-------|--------|
| `model` | gpt-4o, gpt-4o-mini, whisper-1 |
| `component_type` | issue_raising, problem_frame, objectives, ... |
| `error_type` | rate_limit, timeout, invalid_request, server_error |

---

## Implementation

### Directory Structure

Observability is cross-cutting infrastructure, not a specific external system adapter:

```
backend/internal/
├── domain/                 # ❌ No observability imports
├── ports/                  # ❌ No observability imports
├── application/            # ❌ No observability imports
├── adapters/
│   └── ai/
│       └── openai.rs       # ✅ Calls infrastructure/observability
└── infrastructure/
    └── observability/
        ├── mod.rs
        ├── metrics.rs      # Prometheus setup + recording helpers
        └── sentry.rs       # Sentry initialization
```

### Metrics Recording Module

```rust
// infrastructure/observability/metrics.rs

use metrics::{counter, histogram, describe_counter, describe_histogram};

pub fn init_metrics() {
    describe_counter!(
        "ai_tokens_input_total",
        "Total input tokens sent to AI providers"
    );
    describe_counter!(
        "ai_tokens_output_total",
        "Total output tokens received from AI providers"
    );
    describe_counter!(
        "ai_request_errors_total",
        "Total failed AI requests"
    );
    describe_counter!(
        "ai_cost_dollars_total",
        "Estimated cost in USD"
    );
    describe_histogram!(
        "ai_request_duration_seconds",
        "AI request latency in seconds"
    );
}

pub fn record_ai_request(
    model: &str,
    component_type: &str,
    input_tokens: u64,
    output_tokens: u64,
    duration_secs: f64,
    cost_dollars: f64,
) {
    let labels = [
        ("model", model.to_string()),
        ("component_type", component_type.to_string()),
    ];

    counter!("ai_tokens_input_total", &labels).increment(input_tokens);
    counter!("ai_tokens_output_total", &labels).increment(output_tokens);
    histogram!("ai_request_duration_seconds", &labels).record(duration_secs);
    counter!("ai_cost_dollars_total", "model" => model.to_string())
        .increment((cost_dollars * 1000.0) as u64); // Store as millidollars
}

pub fn record_ai_error(model: &str, error_type: &str) {
    counter!(
        "ai_request_errors_total",
        "model" => model.to_string(),
        "error_type" => error_type.to_string()
    ).increment(1);
}
```

### Prometheus Endpoint

```rust
// main.rs

use metrics_exporter_prometheus::PrometheusBuilder;

#[tokio::main]
async fn main() {
    // Initialize Sentry
    let _sentry = sentry::init((
        std::env::var("SENTRY_DSN").ok(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            traces_sample_rate: 0.2,
            ..Default::default()
        },
    ));

    // Initialize Prometheus
    let prometheus_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus recorder");

    init_metrics();

    // Routes
    let app = Router::new()
        .route("/metrics", get({
            let handle = prometheus_handle.clone();
            move || async move { handle.render() }
        }))
        .route("/api/sessions", get(list_sessions))
        .layer(sentry_tower::NewSentryLayer::new_from_top())
        .layer(sentry_tower::SentryHttpLayer::with_transaction());

    // ...
}
```

### AI Adapter Integration

```rust
// adapters/ai/openai.rs

use crate::observability::metrics::{record_ai_request, record_ai_error};

impl ChatProvider for OpenAIAdapter {
    async fn stream(
        &self,
        req: ChatRequest
    ) -> Result<impl Stream<Item = ChatChunk>, ChatError> {
        let start = Instant::now();
        let component_type = req.component_type.clone();

        let response = self.client
            .chat()
            .create_stream(req.into())
            .await;

        match &response {
            Ok(stream) => {
                // Token counting happens when stream completes
                // (simplified - actual impl tracks via stream wrapper)
            }
            Err(e) => {
                record_ai_error(&self.model, &e.to_error_type());
                sentry::capture_error(e);
            }
        }

        response.map_err(ChatError::from)
    }
}
```

---

## Grafana Dashboard

### AI Cost Overview

```
┌────────────────────────────────────────────────────────────┐
│  AI Cost Dashboard                              [30 days ▼]│
├────────────────────────────────────────────────────────────┤
│                                                            │
│  Total Cost        Token Usage         Error Rate          │
│  ┌──────────┐     ┌──────────┐        ┌──────────┐        │
│  │  $47.23  │     │  334K    │        │  0.3%    │        │
│  │  ▲ 12%   │     │  ▲ 8%    │        │  ▼ 0.1%  │        │
│  └──────────┘     └──────────┘        └──────────┘        │
│                                                            │
│  Cost by Component                                         │
│  ┌──────────────────────────────────────────────────────┐ │
│  │ Consequences     ████████████████████████  $18.70    │ │
│  │ Issue Raising    ████████████████          $12.40    │ │
│  │ Problem Frame    ████████                  $8.20     │ │
│  │ Tradeoffs        ██████                    $4.50     │ │
│  │ Other            ████                      $3.43     │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                            │
│  Token Usage Over Time                                     │
│  ┌──────────────────────────────────────────────────────┐ │
│  │     ╭─╮                                              │ │
│  │    ╭╯ ╰╮   ╭──╮                    ╭╮               │ │
│  │ ╭──╯   ╰───╯  ╰────────╮     ╭────╯╰╮              │ │
│  │─╯                      ╰─────╯      ╰──────────    │ │
│  └──────────────────────────────────────────────────────┘ │
│  Jan 1        Jan 7        Jan 14       Jan 21    Jan 28  │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

### Key Queries (PromQL)

```promql
# Total cost (last 30 days)
sum(increase(ai_cost_dollars_total[30d])) / 1000

# Cost by component type
sum by (component_type) (increase(ai_cost_dollars_total[30d])) / 1000

# Token usage rate
rate(ai_tokens_input_total[5m]) + rate(ai_tokens_output_total[5m])

# Error rate
rate(ai_request_errors_total[5m]) / rate(ai_tokens_input_total[5m])

# P95 latency by model
histogram_quantile(0.95, rate(ai_request_duration_seconds_bucket[5m]))
```

---

## Configuration

### Environment Variables

```bash
# Sentry
SENTRY_DSN=https://xxx@sentry.io/xxx
SENTRY_ENVIRONMENT=production
SENTRY_TRACES_SAMPLE_RATE=0.2

# Prometheus (Grafana Cloud)
# No config needed - Grafana scrapes /metrics endpoint
```

### Grafana Cloud Setup

1. Create Grafana Cloud account (free tier)
2. Add Prometheus data source
3. Configure scrape target: `https://api.choicesherpa.com/metrics`
4. Import AI cost dashboard

---

## Deployment Options

| Option | Sentry | Prometheus | Grafana |
|--------|--------|------------|---------|
| **Fully Managed** | Sentry.io | Grafana Cloud | Grafana Cloud |
| **Self-Hosted** | Self-hosted Sentry | Prometheus | Grafana |
| **Hybrid** | Sentry.io | Self-hosted | Self-hosted |

**Recommendation:** Start with managed (Sentry.io + Grafana Cloud) for MVP, migrate to self-hosted if costs warrant.

---

## Free Tier Limits

| Service | Free Tier |
|---------|-----------|
| Sentry | 5K errors/month, 10K transactions |
| Grafana Cloud | 10K active metrics, 50GB logs |

Sufficient for MVP and early growth.

---

## Alternatives Considered

| Alternative | Rejection Reason |
|-------------|------------------|
| Sentry only | No custom metrics for AI tokens |
| Datadog | Expensive, overkill for MVP |
| New Relic | Expensive, complex pricing |
| Self-hosted only | Operational burden for MVP |
| Axiom | Less mature Prometheus ecosystem |

---

## Sources

- [Sentry Rust SDK](https://docs.sentry.io/platforms/rust/)
- [sentry-tower crate](https://crates.io/crates/sentry-tower)
- [metrics crate](https://crates.io/crates/metrics)
- [Grafana Cloud](https://grafana.com/products/cloud/)
- [Prometheus Best Practices](https://prometheus.io/docs/practices/naming/)
