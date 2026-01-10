# Infrastructure: HTTP Router

**Type:** Cross-Cutting Infrastructure
**Priority:** P0 (Required for API)
**Last Updated:** 2026-01-09

> Complete specification for Axum HTTP router setup, middleware stack, and error handling.

---

## Overview

Choice Sherpa uses Axum as its HTTP framework. This specification defines:
1. Router architecture and route organization
2. Middleware stack configuration
3. Error handling and response formats
4. Request/response types
5. WebSocket integration for streaming

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           HTTP Request Flow                                  │
│                                                                              │
│   Client Request                                                             │
│        │                                                                     │
│        ▼                                                                     │
│   ┌────────────────────────────────────────────────────────────────────┐    │
│   │                        Middleware Stack                             │    │
│   │                                                                     │    │
│   │   1. RequestId         - Assign unique request ID                  │    │
│   │   2. TraceLayer        - Distributed tracing                       │    │
│   │   3. Timeout           - Request timeout (30s default)             │    │
│   │   4. CORS              - Cross-origin handling                     │    │
│   │   5. Compression       - Gzip response compression                 │    │
│   │   6. ErrorHandler      - Convert errors to HTTP responses          │    │
│   └────────────────────────────────────────────────────────────────────┘    │
│        │                                                                     │
│        ▼                                                                     │
│   ┌────────────────────────────────────────────────────────────────────┐    │
│   │                          Router                                     │    │
│   │                                                                     │    │
│   │   /health              → HealthHandler                             │    │
│   │   /api/v1/auth/*       → AuthRoutes (Zitadel callbacks)           │    │
│   │   /api/v1/memberships  → MembershipRoutes [Auth Required]         │    │
│   │   /api/v1/sessions     → SessionRoutes [Auth Required]            │    │
│   │   /api/v1/cycles       → CycleRoutes [Auth Required]              │    │
│   │   /api/v1/conversations→ ConversationRoutes [Auth Required]       │    │
│   │   /ws/conversations/*  → WebSocket Handler [Auth Required]        │    │
│   └────────────────────────────────────────────────────────────────────┘    │
│        │                                                                     │
│        ▼                                                                     │
│   ┌────────────────────────────────────────────────────────────────────┐    │
│   │                       Handler Layer                                 │    │
│   │                                                                     │    │
│   │   Extract<Path, Query, Json> → Command/Query → Result → Response   │    │
│   └────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Router Setup

### Application State

```rust
use axum::extract::FromRef;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db: PgPool,
    pub redis: redis::Client,
    pub config: Arc<AppConfig>,
    pub ai_provider: Arc<dyn AiProvider>,
    pub payment_provider: Arc<dyn PaymentProvider>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Result<Self, StartupError> {
        let db = create_pool(&config.database).await?;
        let redis = redis::Client::open(config.redis.url.clone())?;
        let ai_provider = create_ai_provider(&config.ai)?;
        let payment_provider = create_payment_provider(&config.payment)?;

        Ok(Self {
            db,
            redis,
            config: Arc::new(config),
            ai_provider,
            payment_provider,
        })
    }
}
```

### Main Router

```rust
use axum::{Router, routing::{get, post, put, delete}};
use tower_http::{
    trace::TraceLayer,
    timeout::TimeoutLayer,
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    compression::CompressionLayer,
};
use std::time::Duration;

pub fn create_router(state: AppState) -> Router {
    let api_routes = Router::new()
        .nest("/auth", auth_routes())
        .nest("/memberships", membership_routes())
        .nest("/sessions", session_routes())
        .nest("/cycles", cycle_routes())
        .nest("/conversations", conversation_routes());

    Router::new()
        // Health check (no auth)
        .route("/health", get(health_handler))
        .route("/ready", get(readiness_handler))

        // API routes (versioned)
        .nest("/api/v1", api_routes)

        // WebSocket routes
        .route("/ws/conversations/:id", get(ws_handler))

        // Apply middleware
        .layer(middleware_stack(&state.config))

        // Application state
        .with_state(state)
}
```

### Middleware Stack

```rust
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

fn middleware_stack(config: &AppConfig) -> impl tower::Layer<...> {
    let cors = CorsLayer::new()
        .allow_origin(parse_cors_origins(&config.server.cors_origins))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600));

    ServiceBuilder::new()
        // Request ID propagation
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())

        // Tracing
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    let request_id = request.headers()
                        .get("x-request-id")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("unknown");

                    tracing::info_span!(
                        "http_request",
                        request_id = %request_id,
                        method = %request.method(),
                        uri = %request.uri(),
                    )
                })
                .on_response(|response: &Response<_>, latency: Duration, _span: &Span| {
                    tracing::info!(
                        status = %response.status().as_u16(),
                        latency_ms = %latency.as_millis(),
                        "response"
                    );
                })
        )

        // Timeout
        .layer(TimeoutLayer::new(Duration::from_secs(
            config.server.request_timeout_secs
        )))

        // CORS
        .layer(cors)

        // Compression
        .layer(CompressionLayer::new())
}
```

---

## Route Modules

### Membership Routes

```rust
pub fn membership_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_membership))
        .route("/me", get(get_current_membership))
        .route("/me", put(update_membership))
        .route("/me/cancel", post(cancel_membership))
        .route("/webhook", post(stripe_webhook))
        // Auth middleware for all except webhook
        .layer(axum::middleware::from_fn(require_auth))
}

async fn create_membership(
    State(state): State<AppState>,
    claims: AuthClaims,
    Json(cmd): Json<CreateMembershipRequest>,
) -> Result<Json<MembershipResponse>, ApiError> {
    let membership = create_membership_command(&state.db, claims.user_id, cmd).await?;
    Ok(Json(membership.into()))
}
```

### Session Routes

```rust
pub fn session_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_sessions))
        .route("/", post(create_session))
        .route("/:id", get(get_session))
        .route("/:id", put(update_session))
        .route("/:id/archive", post(archive_session))
        .layer(axum::middleware::from_fn(require_auth))
}

async fn create_session(
    State(state): State<AppState>,
    claims: AuthClaims,
    Json(cmd): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<SessionResponse>), ApiError> {
    // Check access via AccessChecker
    let can_create = state.access_checker
        .can_create_session(&claims.user_id)
        .await?;

    if !can_create {
        return Err(ApiError::Forbidden("Session limit reached"));
    }

    let session = create_session_command(&state.db, claims.user_id, cmd).await?;
    Ok((StatusCode::CREATED, Json(session.into())))
}
```

### Cycle Routes

```rust
pub fn cycle_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_cycle))
        .route("/:id", get(get_cycle))
        .route("/:id/components/:type", get(get_component))
        .route("/:id/components/:type", put(update_component))
        .route("/:id/components/:type/start", post(start_component))
        .route("/:id/components/:type/complete", post(complete_component))
        .route("/:id/branch", post(create_branch))
        .layer(axum::middleware::from_fn(require_auth))
}
```

### Conversation Routes

```rust
pub fn conversation_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_conversation))
        .route("/:id", get(get_conversation))
        .route("/:id/messages", get(list_messages))
        .route("/:id/messages", post(send_message))
        .layer(axum::middleware::from_fn(require_auth))
}
```

---

## Authentication Middleware

### JWT Validation

```rust
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let auth_header = request.headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(ApiError::Unauthorized("Missing authorization header"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(ApiError::Unauthorized("Invalid authorization format"))?;

    let claims = validate_token(&state.config.auth, token).await?;

    // Insert claims into request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthClaims {
    pub sub: String,
    pub user_id: UserId,
    pub email: Option<String>,
    pub exp: u64,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthClaims
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions
            .get::<AuthClaims>()
            .cloned()
            .ok_or(ApiError::Unauthorized("Not authenticated"))
    }
}
```

---

## Error Handling

### Error Types

```rust
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Unauthorized: {0}")]
    Unauthorized(&'static str),

    #[error("Forbidden: {0}")]
    Forbidden(&'static str),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", *msg),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", *msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.as_str()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.as_str()),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.as_str()),
            ApiError::UnprocessableEntity(msg) => (StatusCode::UNPROCESSABLE_ENTITY, "UNPROCESSABLE_ENTITY", msg.as_str()),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.as_str()),
            ApiError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, "SERVICE_UNAVAILABLE", msg.as_str()),
        };

        let body = Json(ErrorResponse {
            error: ErrorBody {
                code: error_code.to_string(),
                message: message.to_string(),
            },
        });

        (status, body).into_response()
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}
```

### Domain Error Conversion

```rust
impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::NotFound(msg) => ApiError::NotFound(msg),
            DomainError::ValidationFailed(msg) => ApiError::UnprocessableEntity(msg),
            DomainError::Unauthorized(msg) => ApiError::Forbidden(msg.to_string()),
            DomainError::Conflict(msg) => ApiError::Conflict(msg),
            DomainError::Internal(msg) => ApiError::Internal(msg),
        }
    }
}

impl From<DatabaseError> for ApiError {
    fn from(err: DatabaseError) -> Self {
        match err {
            DatabaseError::ConstraintViolation(msg) => ApiError::Conflict(msg),
            DatabaseError::PoolExhausted => ApiError::ServiceUnavailable("Database unavailable".into()),
            _ => ApiError::Internal("Database error".into()),
        }
    }
}
```

---

## Request/Response Types

### Standard Response Format

```rust
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
}

#[derive(Serialize)]
pub struct ResponseMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Serialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
}
```

### Pagination

```rust
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

impl PaginationParams {
    pub fn offset(&self) -> u64 {
        ((self.page.saturating_sub(1)) * self.per_page) as u64
    }

    pub fn limit(&self) -> u64 {
        self.per_page.min(100) as u64
    }
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 20 }
```

---

## WebSocket Handler

### Streaming Conversations

```rust
use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade}, Path},
    response::Response,
};
use futures::{StreamExt, SinkExt};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    claims: AuthClaims,
    Path(conversation_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    // Verify user owns the conversation
    verify_conversation_access(&state.db, &claims.user_id, &conversation_id).await?;

    Ok(ws.on_upgrade(move |socket| handle_ws(socket, state, claims, conversation_id)))
}

async fn handle_ws(
    socket: WebSocket,
    state: AppState,
    claims: AuthClaims,
    conversation_id: Uuid,
) {
    let (mut sender, mut receiver) = socket.split();

    // Spawn task to handle incoming messages
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(ws::Message::Text(text)) => {
                    // Parse and handle user message
                    if let Ok(user_msg) = serde_json::from_str::<UserMessage>(&text) {
                        // Stream AI response back
                        let stream = state.ai_provider
                            .stream_response(&conversation_id, &user_msg)
                            .await;

                        while let Some(chunk) = stream.next().await {
                            let response = serde_json::to_string(&chunk).unwrap();
                            sender.send(ws::Message::Text(response)).await.ok();
                        }
                    }
                }
                Ok(ws::Message::Close(_)) => break,
                Err(_) => break,
                _ => {}
            }
        }
    });

    recv_task.await.ok();
}
```

---

## Health Endpoints

### Liveness Check

```rust
async fn health_handler() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
```

### Readiness Check

```rust
async fn readiness_handler(State(state): State<AppState>) -> impl IntoResponse {
    let db_health = check_database_health(&state.db).await;
    let redis_health = check_redis_health(&state.redis).await;

    let all_healthy = db_health.is_healthy() && redis_health.is_healthy();
    let status = if all_healthy { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };

    (status, Json(json!({
        "status": if all_healthy { "ready" } else { "not_ready" },
        "checks": {
            "database": db_health,
            "redis": redis_health,
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
```

---

## Server Startup

### Main Entry Point

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    // Load configuration
    let config = AppConfig::load()?;
    config.validate()?;

    tracing::info!(
        host = %config.server.host,
        port = %config.server.port,
        environment = ?config.server.environment,
        "Starting Choice Sherpa API"
    );

    // Create application state
    let state = AppState::new(config.clone()).await?;

    // Run migrations if configured
    if config.database.run_migrations {
        sqlx::migrate!("./migrations")
            .run(&state.db)
            .await?;
    }

    // Create router
    let app = create_router(state);

    // Start server
    let addr = config.server.socket_addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C handler");

    tracing::info!("Shutdown signal received, starting graceful shutdown");
}
```

---

## File Structure

```
backend/src/
├── adapters/
│   └── http/
│       ├── mod.rs              # Module exports
│       ├── router.rs           # Router setup
│       ├── middleware.rs       # Auth, tracing middleware
│       ├── error.rs            # ApiError, IntoResponse
│       ├── extractors.rs       # AuthClaims, Pagination
│       ├── responses.rs        # Response types
│       ├── handlers/
│       │   ├── health.rs
│       │   ├── membership.rs
│       │   ├── session.rs
│       │   ├── cycle.rs
│       │   └── conversation.rs
│       └── websocket.rs        # WebSocket handler
└── main.rs                     # Server entry point
```

---

## Related Documents

- **Configuration**: `features/infrastructure/configuration.md`
- **WebSocket Event Bridge**: `features/infrastructure/websocket-event-bridge.md`
- **Health Checks**: `features/infrastructure/health-checks.md`
- **Security Standard**: `docs/architecture/APPLICATION-SECURITY-STANDARD.md`

---

*Version: 1.0.0*
*Created: 2026-01-09*
