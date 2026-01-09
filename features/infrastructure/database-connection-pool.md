# Infrastructure: PostgreSQL Connection Pool

**Type:** Cross-Cutting Infrastructure
**Priority:** P0 (Required for production deployment)
**Last Updated:** 2026-01-08

> Complete specification for PostgreSQL connection pooling, configuration, transaction management, and failover handling.

---

## Overview

Choice Sherpa uses PostgreSQL as its primary data store. This specification defines:
1. Connection pool configuration
2. Transaction boundaries and isolation levels
3. Retry strategies for transient failures
4. Connection health monitoring
5. Read replica support (future)

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Application Layer                                   │
│                                                                              │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│   │   Session   │  │    Cycle    │  │ Membership  │  │  Dashboard  │       │
│   │   Handler   │  │   Handler   │  │   Handler   │  │   Handler   │       │
│   └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘       │
│          │                │                │                │               │
│          └────────────────┴────────────────┴────────────────┘               │
│                                   │                                          │
│                                   ▼                                          │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     Connection Pool (sqlx::PgPool)                   │   │
│   │                                                                      │   │
│   │   ┌─────────────────────────────────────────────────────────────┐   │   │
│   │   │                    Pool Configuration                        │   │   │
│   │   │                                                              │   │   │
│   │   │   min_connections: 5                                         │   │   │
│   │   │   max_connections: 20                                        │   │   │
│   │   │   acquire_timeout: 30s                                       │   │   │
│   │   │   idle_timeout: 10m                                          │   │   │
│   │   │   max_lifetime: 30m                                          │   │   │
│   │   └─────────────────────────────────────────────────────────────┘   │   │
│   │                                                                      │   │
│   │   Active Connections: [conn1] [conn2] [conn3] [conn4] [conn5]       │   │
│   │   Idle Connections:   [conn6] [conn7] [conn8]                       │   │
│   │   Waiting Requests:   []                                            │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                   │                                          │
└───────────────────────────────────┼──────────────────────────────────────────┘
                                    │
                                    │ TCP/TLS
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           PostgreSQL Server                                   │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                      Connection Settings                             │   │
│   │                                                                      │   │
│   │   max_connections: 100                                               │   │
│   │   work_mem: 64MB                                                     │   │
│   │   shared_buffers: 256MB                                              │   │
│   │   effective_cache_size: 768MB                                        │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Connection Pool Configuration

### Environment-Based Configuration

```rust
use std::time::Duration;
use sqlx::postgres::{PgPoolOptions, PgConnectOptions};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// PostgreSQL connection URL
    pub url: String,
    /// Minimum connections to maintain
    pub min_connections: u32,
    /// Maximum connections allowed
    pub max_connections: u32,
    /// Timeout for acquiring a connection
    pub acquire_timeout: Duration,
    /// Maximum idle time before connection is closed
    pub idle_timeout: Duration,
    /// Maximum lifetime of a connection
    pub max_lifetime: Duration,
    /// Enable TLS
    pub ssl_mode: SslMode,
    /// Statement cache size per connection
    pub statement_cache_capacity: usize,
}

#[derive(Debug, Clone)]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
    VerifyCa,
    VerifyFull,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            min_connections: 5,
            max_connections: 20,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
            ssl_mode: SslMode::Prefer,
            statement_cache_capacity: 100,
        }
    }
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let environment = std::env::var("ENVIRONMENT").unwrap_or_default();

        // SECURITY: Require TLS in production unless explicitly disabled
        let default_ssl = if environment == "production" {
            SslMode::Require
        } else {
            SslMode::Prefer
        };

        Ok(Self {
            url: std::env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingEnv("DATABASE_URL"))?,
            min_connections: std::env::var("DATABASE_MIN_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(20),
            acquire_timeout: std::env::var("DATABASE_ACQUIRE_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .map(Duration::from_secs)
                .unwrap_or(Duration::from_secs(30)),
            idle_timeout: std::env::var("DATABASE_IDLE_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .map(Duration::from_secs)
                .unwrap_or(Duration::from_secs(600)),
            max_lifetime: std::env::var("DATABASE_MAX_LIFETIME_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .map(Duration::from_secs)
                .unwrap_or(Duration::from_secs(1800)),
            ssl_mode: std::env::var("DATABASE_SSL_MODE")
                .ok()
                .map(|v| match v.to_lowercase().as_str() {
                    "disable" => {
                        if environment == "production" {
                            tracing::warn!("SECURITY WARNING: SSL disabled in production!");
                        }
                        SslMode::Disable
                    },
                    "require" => SslMode::Require,
                    "verify-ca" => SslMode::VerifyCa,
                    "verify-full" => SslMode::VerifyFull,
                    _ => default_ssl.clone(),
                })
                .unwrap_or(default_ssl),
            statement_cache_capacity: std::env::var("DATABASE_STATEMENT_CACHE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
        })
    }
}
```

### Pool Creation

```rust
pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool, DatabaseError> {
    let connect_options = config.url.parse::<PgConnectOptions>()
        .map_err(|e| DatabaseError::ConfigError(e.to_string()))?
        .statement_cache_capacity(config.statement_cache_capacity);

    let pool = PgPoolOptions::new()
        .min_connections(config.min_connections)
        .max_connections(config.max_connections)
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(Some(config.idle_timeout))
        .max_lifetime(Some(config.max_lifetime))
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                // Set session-level defaults
                sqlx::query("SET timezone = 'UTC'")
                    .execute(conn)
                    .await?;
                sqlx::query("SET default_transaction_isolation TO 'read committed'")
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .before_acquire(|conn, _meta| {
            Box::pin(async move {
                // Validate connection is healthy before use
                sqlx::query("SELECT 1")
                    .execute(conn)
                    .await
                    .map(|_| true)
            })
        })
        .connect_with(connect_options)
        .await
        .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

    // Verify pool is healthy
    pool.acquire().await
        .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

    tracing::info!(
        min = config.min_connections,
        max = config.max_connections,
        "Database connection pool created"
    );

    Ok(pool)
}
```

---

## Configuration Guidelines

### Development Environment

```bash
# .env.development
DATABASE_URL=postgres://choice_sherpa:dev_password@localhost:5432/choice_sherpa_dev
DATABASE_MIN_CONNECTIONS=2
DATABASE_MAX_CONNECTIONS=5
DATABASE_ACQUIRE_TIMEOUT_SECS=10
DATABASE_SSL_MODE=disable
```

### Production Environment

```bash
# .env.production
# SECURITY: Use sslmode=require (or verify-full for maximum security) for encrypted connections
ENVIRONMENT=production
DATABASE_URL=postgres://choice_sherpa:${DB_PASSWORD}@db.example.com:5432/choice_sherpa_prod?sslmode=require
DATABASE_MIN_CONNECTIONS=10
DATABASE_MAX_CONNECTIONS=50
DATABASE_ACQUIRE_TIMEOUT_SECS=30
DATABASE_IDLE_TIMEOUT_SECS=300
DATABASE_MAX_LIFETIME_SECS=1800
# SECURITY: verify-full recommended; require is minimum for production
DATABASE_SSL_MODE=verify-full
DATABASE_STATEMENT_CACHE=200
```

### Sizing Guidelines

| Deployment Size | Connections | Min | Max | Notes |
|-----------------|-------------|-----|-----|-------|
| Single instance | 20 | 5 | 20 | Development/staging |
| 2 instances | 25 each | 5 | 25 | 50 total, within PG default 100 |
| 4 instances | 20 each | 5 | 20 | 80 total, leaves headroom |
| 8+ instances | 10 each | 3 | 10 | Consider PgBouncer |

**Formula:** `max_connections_per_instance = (pg_max_connections - 20) / instance_count`

The `-20` reserves connections for admin, monitoring, and migrations.

---

## Transaction Management

### Transaction Wrapper

```rust
use sqlx::{PgPool, Postgres, Transaction};

/// Unit of work pattern for transactions
pub struct UnitOfWork<'a> {
    tx: Transaction<'a, Postgres>,
}

impl<'a> UnitOfWork<'a> {
    pub async fn begin(pool: &PgPool) -> Result<UnitOfWork<'_>, DatabaseError> {
        let tx = pool.begin().await
            .map_err(|e| DatabaseError::TransactionFailed(e.to_string()))?;
        Ok(UnitOfWork { tx })
    }

    pub fn transaction(&mut self) -> &mut Transaction<'a, Postgres> {
        &mut self.tx
    }

    pub async fn commit(self) -> Result<(), DatabaseError> {
        self.tx.commit().await
            .map_err(|e| DatabaseError::TransactionFailed(e.to_string()))
    }

    pub async fn rollback(self) -> Result<(), DatabaseError> {
        self.tx.rollback().await
            .map_err(|e| DatabaseError::TransactionFailed(e.to_string()))
    }
}
```

### Transactional Command Handler

```rust
/// Execute command with automatic transaction management
pub async fn execute_transactionally<F, T>(
    pool: &PgPool,
    operation: F,
) -> Result<T, DomainError>
where
    F: for<'a> FnOnce(&'a mut Transaction<'_, Postgres>) -> BoxFuture<'a, Result<T, DomainError>>,
{
    let mut uow = UnitOfWork::begin(pool).await?;

    match operation(uow.transaction()).await {
        Ok(result) => {
            uow.commit().await?;
            Ok(result)
        }
        Err(e) => {
            uow.rollback().await?;
            Err(e)
        }
    }
}

// Usage example:
pub async fn create_session(
    pool: &PgPool,
    cmd: CreateSessionCommand,
) -> Result<SessionId, DomainError> {
    execute_transactionally(pool, |tx| {
        Box::pin(async move {
            // Insert session
            let session = Session::create(cmd.user_id, cmd.title)?;
            sqlx::query!(
                "INSERT INTO sessions (id, user_id, title, created_at) VALUES ($1, $2, $3, $4)",
                session.id.to_string(),
                session.user_id.to_string(),
                session.title,
                session.created_at,
            )
            .execute(&mut **tx)
            .await?;

            // Insert event to outbox
            let event = SessionCreated::from(&session);
            sqlx::query!(
                "INSERT INTO event_outbox (event_id, event_type, aggregate_id, payload) VALUES ($1, $2, $3, $4)",
                event.event_id.to_string(),
                "session.created",
                session.id.to_string(),
                serde_json::to_value(&event)?,
            )
            .execute(&mut **tx)
            .await?;

            Ok(session.id)
        })
    }).await
}
```

### Isolation Levels

| Operation | Isolation Level | Use Case |
|-----------|-----------------|----------|
| Read queries | Read Committed | Default, sufficient for most reads |
| Inventory operations | Repeatable Read | Prevent lost updates |
| Financial operations | Serializable | Critical consistency |

```rust
/// Set isolation level for critical operations
pub async fn execute_with_isolation<F, T>(
    pool: &PgPool,
    isolation: IsolationLevel,
    operation: F,
) -> Result<T, DomainError>
where
    F: for<'a> FnOnce(&'a mut Transaction<'_, Postgres>) -> BoxFuture<'a, Result<T, DomainError>>,
{
    let mut tx = pool.begin().await?;

    // Set isolation level
    let isolation_sql = match isolation {
        IsolationLevel::ReadCommitted => "SET TRANSACTION ISOLATION LEVEL READ COMMITTED",
        IsolationLevel::RepeatableRead => "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ",
        IsolationLevel::Serializable => "SET TRANSACTION ISOLATION LEVEL SERIALIZABLE",
    };
    sqlx::query(isolation_sql).execute(&mut *tx).await?;

    match operation(&mut tx).await {
        Ok(result) => {
            tx.commit().await?;
            Ok(result)
        }
        Err(e) => {
            tx.rollback().await?;
            Err(e)
        }
    }
}

pub enum IsolationLevel {
    ReadCommitted,
    RepeatableRead,
    Serializable,
}
```

---

## Retry Strategies

### Retryable Errors

```rust
/// Determines if a database error is retryable
pub fn is_retryable_error(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Database(db_err) => {
            // PostgreSQL error codes that warrant retry
            let code = db_err.code().map(|c| c.to_string());
            matches!(code.as_deref(), Some(
                "40001" |  // serialization_failure
                "40P01" |  // deadlock_detected
                "08006" |  // connection_failure
                "08001" |  // sqlclient_unable_to_establish_sqlconnection
                "08004" |  // sqlserver_rejected_establishment_of_sqlconnection
                "57P01" |  // admin_shutdown
                "57P02" |  // crash_shutdown
                "57P03"    // cannot_connect_now
            ))
        }
        sqlx::Error::PoolTimedOut => true,
        sqlx::Error::PoolClosed => false, // Pool closed deliberately
        sqlx::Error::Io(_) => true, // Network issues
        _ => false,
    }
}
```

### Retry Wrapper

```rust
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub exponential_base: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            exponential_base: 2.0,
        }
    }
}

pub async fn with_retry<F, Fut, T>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, DatabaseError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, sqlx::Error>>,
{
    let mut attempts = 0;
    let mut delay = config.initial_delay;

    loop {
        attempts += 1;

        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                if !is_retryable_error(&err) || attempts >= config.max_attempts {
                    return Err(DatabaseError::QueryFailed(err.to_string()));
                }

                tracing::warn!(
                    attempt = attempts,
                    max_attempts = config.max_attempts,
                    delay_ms = delay.as_millis(),
                    error = %err,
                    "Database operation failed, retrying"
                );

                sleep(delay).await;

                // Exponential backoff with jitter
                let jitter = rand::random::<f64>() * 0.3; // 0-30% jitter
                delay = Duration::from_secs_f64(
                    (delay.as_secs_f64() * config.exponential_base * (1.0 + jitter))
                        .min(config.max_delay.as_secs_f64())
                );
            }
        }
    }
}

// Usage:
pub async fn find_session_with_retry(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Option<Session>, DatabaseError> {
    with_retry(&RetryConfig::default(), || async {
        sqlx::query_as!(
            SessionRow,
            "SELECT * FROM sessions WHERE id = $1",
            session_id.to_string()
        )
        .fetch_optional(pool)
        .await
    })
    .await
    .map(|row| row.map(Session::from))
}
```

---

## Connection Health Monitoring

### Pool Metrics

```rust
use prometheus::{IntGauge, IntCounter, Histogram};

pub struct DatabaseMetrics {
    pub connections_total: IntGauge,
    pub connections_idle: IntGauge,
    pub connections_active: IntGauge,
    pub acquire_wait_time: Histogram,
    pub query_duration: Histogram,
    pub queries_total: IntCounter,
    pub queries_failed: IntCounter,
    pub connection_errors: IntCounter,
}

impl DatabaseMetrics {
    pub fn new() -> Self {
        Self {
            connections_total: IntGauge::new(
                "db_connections_total",
                "Total database connections in pool"
            ).unwrap(),
            connections_idle: IntGauge::new(
                "db_connections_idle",
                "Idle database connections"
            ).unwrap(),
            connections_active: IntGauge::new(
                "db_connections_active",
                "Active database connections"
            ).unwrap(),
            acquire_wait_time: Histogram::with_opts(
                HistogramOpts::new(
                    "db_acquire_wait_seconds",
                    "Time spent waiting for a connection"
                )
                .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0])
            ).unwrap(),
            query_duration: Histogram::with_opts(
                HistogramOpts::new(
                    "db_query_duration_seconds",
                    "Query execution time"
                )
                .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0])
            ).unwrap(),
            queries_total: IntCounter::new(
                "db_queries_total",
                "Total database queries executed"
            ).unwrap(),
            queries_failed: IntCounter::new(
                "db_queries_failed_total",
                "Total failed database queries"
            ).unwrap(),
            connection_errors: IntCounter::new(
                "db_connection_errors_total",
                "Total connection errors"
            ).unwrap(),
        }
    }

    pub fn update_from_pool(&self, pool: &PgPool) {
        let size = pool.size();
        let idle = pool.num_idle();

        self.connections_total.set(size as i64);
        self.connections_idle.set(idle as i64);
        self.connections_active.set((size - idle as u32) as i64);
    }
}

// Background task to update metrics periodically
pub async fn metrics_updater(pool: PgPool, metrics: DatabaseMetrics) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));

    loop {
        interval.tick().await;
        metrics.update_from_pool(&pool);
    }
}
```

### Health Check

```rust
/// Database health check for liveness/readiness probes
pub async fn check_database_health(pool: &PgPool) -> HealthCheckResult {
    let start = std::time::Instant::now();

    // Quick connectivity check
    match sqlx::query("SELECT 1").execute(pool).await {
        Ok(_) => {
            let latency = start.elapsed();

            // Check pool utilization
            let size = pool.size();
            let idle = pool.num_idle();
            let utilization = if size > 0 {
                ((size - idle as u32) as f64 / size as f64) * 100.0
            } else {
                0.0
            };

            let status = if utilization > 90.0 {
                HealthStatus::Degraded(format!(
                    "Pool utilization high: {:.1}%",
                    utilization
                ))
            } else if latency > Duration::from_millis(500) {
                HealthStatus::Degraded(format!(
                    "Query latency high: {}ms",
                    latency.as_millis()
                ))
            } else {
                HealthStatus::Healthy
            };

            HealthCheckResult {
                component: "database",
                status,
                latency_ms: latency.as_millis() as u64,
                details: serde_json::json!({
                    "pool_size": size,
                    "pool_idle": idle,
                    "utilization_percent": utilization,
                }),
            }
        }
        Err(e) => HealthCheckResult {
            component: "database",
            status: HealthStatus::Unhealthy(e.to_string()),
            latency_ms: start.elapsed().as_millis() as u64,
            details: serde_json::Value::Null,
        },
    }
}
```

---

## Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Pool exhausted")]
    PoolExhausted,

    #[error("Serialization conflict")]
    SerializationConflict,

    #[error("Deadlock detected")]
    Deadlock,

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::Database(db_err) => {
                let code = db_err.code().map(|c| c.to_string());
                match code.as_deref() {
                    Some("40001") => DatabaseError::SerializationConflict,
                    Some("40P01") => DatabaseError::Deadlock,
                    Some(c) if c.starts_with("23") => {
                        DatabaseError::ConstraintViolation(db_err.message().to_string())
                    }
                    _ => DatabaseError::QueryFailed(err.to_string()),
                }
            }
            sqlx::Error::PoolTimedOut => DatabaseError::PoolExhausted,
            _ => DatabaseError::QueryFailed(err.to_string()),
        }
    }
}
```

---

## Migrations

### Migration Runner

```rust
pub async fn run_migrations(pool: &PgPool) -> Result<(), DatabaseError> {
    tracing::info!("Running database migrations...");

    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| DatabaseError::ConnectionFailed(format!("Migration failed: {}", e)))?;

    tracing::info!("Migrations completed successfully");
    Ok(())
}
```

### Migration File Structure

```
backend/migrations/
├── 20260108000000_create_sessions.sql
├── 20260108000001_create_cycles.sql
├── 20260108000002_create_components.sql
├── 20260108000003_create_conversations.sql
├── 20260108000004_create_memberships.sql
├── 20260108000005_create_event_outbox.sql
├── 20260108000006_create_processed_events.sql
└── 20260108000007_create_dashboard_views.sql
```

---

## Testing

### Test Database Setup

```rust
use sqlx::PgPool;

/// Create isolated test database
pub async fn setup_test_database() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://test:test@localhost:5432/choice_sherpa_test".to_string());

    let pool = PgPool::connect(&database_url).await.unwrap();

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    // Clear all tables
    sqlx::query("TRUNCATE sessions, cycles, components, memberships CASCADE")
        .execute(&pool)
        .await
        .unwrap();

    pool
}

/// Use test transaction that rolls back
pub async fn with_test_transaction<F, Fut, T>(pool: &PgPool, test: F) -> T
where
    F: FnOnce(Transaction<'_, Postgres>) -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let mut tx = pool.begin().await.unwrap();
    let result = test(tx).await;
    // Transaction drops and rolls back automatically
    result
}
```

---

## File Structure

```
backend/src/
├── infrastructure/
│   ├── database/
│   │   ├── mod.rs              # Module exports
│   │   ├── config.rs           # DatabaseConfig
│   │   ├── pool.rs             # Pool creation
│   │   ├── transaction.rs      # UnitOfWork, transaction helpers
│   │   ├── retry.rs            # Retry logic
│   │   ├── health.rs           # Health checks
│   │   ├── metrics.rs          # Pool metrics
│   │   └── error.rs            # DatabaseError
│   └── mod.rs
└── migrations/
    └── *.sql
```

---

## Row-Level Security Integration

### Overview

Row-Level Security (RLS) ensures users can only access their own data at the database level,
providing defense-in-depth beyond application-layer authorization.

### Session Variable Setup

Each database transaction must set the current user context for RLS policies:

```rust
/// Set user context for RLS policies
pub async fn set_user_context<'a>(
    tx: &mut Transaction<'a, Postgres>,
    user_id: &UserId,
) -> Result<(), DatabaseError> {
    sqlx::query(&format!(
        "SET LOCAL app.current_user_id = '{}'",
        user_id.as_str()
    ))
    .execute(&mut **tx)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

/// Execute operation with RLS user context
pub async fn with_user_context<F, T>(
    pool: &PgPool,
    user_id: &UserId,
    operation: F,
) -> Result<T, DomainError>
where
    F: for<'a> FnOnce(&'a mut Transaction<'_, Postgres>) -> BoxFuture<'a, Result<T, DomainError>>,
{
    let mut tx = pool.begin().await?;

    // Set user context for RLS
    set_user_context(&mut tx, user_id).await?;

    match operation(&mut tx).await {
        Ok(result) => {
            tx.commit().await?;
            Ok(result)
        }
        Err(e) => {
            tx.rollback().await?;
            Err(e)
        }
    }
}
```

### Database User Privileges

Follow the principle of least privilege:

```sql
-- Application user (read-write, RLS enforced)
CREATE USER app_user WITH PASSWORD '${APP_DB_PASSWORD}';
GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA public TO app_user;
-- Note: No DELETE granted - use soft deletes

-- Read-only user for reporting/analytics
CREATE USER app_readonly WITH PASSWORD '${READONLY_DB_PASSWORD}';
GRANT SELECT ON ALL TABLES IN SCHEMA public TO app_readonly;

-- Migration user (schema changes only, used by CI/CD)
CREATE USER app_migrations WITH PASSWORD '${MIGRATIONS_DB_PASSWORD}';
GRANT ALL ON SCHEMA public TO app_migrations;
```

### RLS Policy Examples

```sql
-- Enable RLS on user-owned tables
ALTER TABLE sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE cycles ENABLE ROW LEVEL SECURITY;
ALTER TABLE memberships ENABLE ROW LEVEL SECURITY;

-- Sessions: users can only see their own
CREATE POLICY session_owner_policy ON sessions
    USING (user_id = current_setting('app.current_user_id', true));

-- Cycles: through session ownership
CREATE POLICY cycle_owner_policy ON cycles
    USING (session_id IN (
        SELECT id FROM sessions
        WHERE user_id = current_setting('app.current_user_id', true)
    ));
```

---

## Related Documents

- **System Architecture**: `docs/architecture/SYSTEM-ARCHITECTURE.md`
- **Event Infrastructure**: `features/foundation/event-infrastructure.md` (transactional outbox)
- **Health Checks**: `features/infrastructure/health-checks.md`
- **Security Standard**: `docs/architecture/APPLICATION-SECURITY-STANDARD.md`

---

*Version: 1.0.0*
*Created: 2026-01-08*
