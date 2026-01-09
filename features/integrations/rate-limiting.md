# Integration: API Rate Limiting

**Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md
**Type:** Cross-Cutting Infrastructure
**Priority:** P1 (Required for production deployment)
**Depends On:** foundation module, membership module

> Multi-layer rate limiting protecting APIs, external services, and AI costs with tier-aware limits.

---

## Overview

Rate limiting protects Choice Sherpa from abuse, controls costs (especially AI API calls), and ensures fair usage across membership tiers. This specification defines a multi-layer rate limiting strategy that operates at different granularities.

### Rate Limiting Layers

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Layer 1: Global                                    │
│   DDoS protection, infrastructure limits (e.g., 10,000 req/min global)      │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Layer 2: Per-IP                                    │
│   Unauthenticated requests, brute-force protection (100 req/min per IP)     │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Layer 3: Per-User                                  │
│   Tier-based limits, fair usage (varies by membership tier)                 │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Layer 4: Per-Resource                              │
│   AI endpoints, expensive operations (separate limits for costly calls)     │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Rate Limit Configuration

### Tier-Based Limits

| Endpoint Category | Free | Monthly | Annual |
|-------------------|------|---------|--------|
| **General API** (req/min) | 60 | 300 | 600 |
| **Session CRUD** (req/hour) | 30 | 100 | 300 |
| **Conversation Messages** (req/min) | 10 | 30 | 60 |
| **AI Completions** (req/min) | 5 | 15 | 30 |
| **AI Tokens** (tokens/day) | 10,000 | 100,000 | 500,000 |
| **Exports** (req/hour) | 0 | 10 | 50 |
| **WebSocket Connections** | 1 | 3 | 10 |

### Global Limits

| Limit | Value | Purpose |
|-------|-------|---------|
| Global requests/min | 10,000 | Infrastructure protection |
| Per-IP requests/min | 100 | Brute-force protection |
| Per-IP auth attempts/hour | 10 | Login brute-force |
| Webhook requests/min | 1,000 | External integration |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Request Flow                                    │
└─────────────────────────────────────────────────────────────────────────────┘

Request → [Global Limiter] → [IP Limiter] → [Auth] → [User Limiter] → Handler
             │                    │                       │
             │                    │                       │
             ▼                    ▼                       ▼
         429 Global          429 IP Rate            429 User Rate
         Rate Limited        Limited                Limited

                    ┌───────────────────┐
                    │   Redis Backend   │
                    │   (Token Bucket)  │
                    └───────────────────┘
```

---

## Port Definition

```rust
// backend/src/ports/rate_limiter.rs

use async_trait::async_trait;

/// Port for rate limiting operations
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if request is allowed, consuming a token if so
    async fn check(&self, key: RateLimitKey) -> Result<RateLimitResult, RateLimitError>;

    /// Get current rate limit status without consuming a token
    async fn status(&self, key: RateLimitKey) -> Result<RateLimitStatus, RateLimitError>;

    /// Reset rate limit for a key (admin operation)
    async fn reset(&self, key: RateLimitKey) -> Result<(), RateLimitError>;
}

/// Key identifying what to rate limit
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct RateLimitKey {
    pub scope: RateLimitScope,
    pub identifier: String,
    pub resource: Option<String>,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum RateLimitScope {
    Global,
    Ip,
    User,
    Resource,
}

impl RateLimitKey {
    pub fn global() -> Self {
        Self {
            scope: RateLimitScope::Global,
            identifier: "global".to_string(),
            resource: None,
        }
    }

    pub fn ip(ip: &str) -> Self {
        Self {
            scope: RateLimitScope::Ip,
            identifier: ip.to_string(),
            resource: None,
        }
    }

    pub fn user(user_id: &UserId) -> Self {
        Self {
            scope: RateLimitScope::User,
            identifier: user_id.to_string(),
            resource: None,
        }
    }

    pub fn user_resource(user_id: &UserId, resource: &str) -> Self {
        Self {
            scope: RateLimitScope::User,
            identifier: user_id.to_string(),
            resource: Some(resource.to_string()),
        }
    }
}

/// Result of a rate limit check
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed(RateLimitStatus),
    Denied(RateLimitDenied),
}

#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: Timestamp,
    pub window_secs: u32,
}

#[derive(Debug, Clone)]
pub struct RateLimitDenied {
    pub limit: u32,
    pub retry_after_secs: u32,
    pub scope: RateLimitScope,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("rate limiter unavailable: {0}")]
    Unavailable(String),

    #[error("invalid key: {0}")]
    InvalidKey(String),
}
```

---

## Rate Limit Configuration

```rust
// backend/src/config/rate_limits.rs

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub global: GlobalLimits,
    pub per_ip: IpLimits,
    pub per_tier: HashMap<MembershipTier, TierLimits>,
    pub resources: HashMap<String, ResourceLimits>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GlobalLimits {
    pub requests_per_minute: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IpLimits {
    pub requests_per_minute: u32,
    pub auth_attempts_per_hour: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TierLimits {
    pub general_requests_per_minute: u32,
    pub session_requests_per_hour: u32,
    pub conversation_messages_per_minute: u32,
    pub ai_completions_per_minute: u32,
    pub ai_tokens_per_day: u32,
    pub exports_per_hour: u32,
    pub websocket_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResourceLimits {
    pub requests_per_minute: u32,
    pub window_secs: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            global: GlobalLimits {
                requests_per_minute: 10_000,
            },
            per_ip: IpLimits {
                requests_per_minute: 100,
                auth_attempts_per_hour: 10,
            },
            per_tier: [
                (MembershipTier::Free, TierLimits {
                    general_requests_per_minute: 60,
                    session_requests_per_hour: 30,
                    conversation_messages_per_minute: 10,
                    ai_completions_per_minute: 5,
                    ai_tokens_per_day: 10_000,
                    exports_per_hour: 0,
                    websocket_connections: 1,
                }),
                (MembershipTier::Monthly, TierLimits {
                    general_requests_per_minute: 300,
                    session_requests_per_hour: 100,
                    conversation_messages_per_minute: 30,
                    ai_completions_per_minute: 15,
                    ai_tokens_per_day: 100_000,
                    exports_per_hour: 10,
                    websocket_connections: 3,
                }),
                (MembershipTier::Annual, TierLimits {
                    general_requests_per_minute: 600,
                    session_requests_per_hour: 300,
                    conversation_messages_per_minute: 60,
                    ai_completions_per_minute: 30,
                    ai_tokens_per_day: 500_000,
                    exports_per_hour: 50,
                    websocket_connections: 10,
                }),
            ].into(),
            resources: HashMap::new(),
        }
    }
}
```

---

## Redis Adapter (Token Bucket)

```rust
// backend/src/adapters/rate_limiter/redis.rs

use redis::aio::ConnectionManager;

pub struct RedisRateLimiter {
    redis: ConnectionManager,
    config: RateLimitConfig,
}

impl RedisRateLimiter {
    pub fn new(redis: ConnectionManager, config: RateLimitConfig) -> Self {
        Self { redis, config }
    }

    fn key_for(&self, key: &RateLimitKey) -> String {
        match &key.resource {
            Some(resource) => format!(
                "ratelimit:{}:{}:{}",
                key.scope.as_str(),
                key.identifier,
                resource
            ),
            None => format!(
                "ratelimit:{}:{}",
                key.scope.as_str(),
                key.identifier
            ),
        }
    }

    fn limits_for(&self, key: &RateLimitKey, tier: Option<MembershipTier>) -> (u32, u32) {
        match key.scope {
            RateLimitScope::Global => (self.config.global.requests_per_minute, 60),
            RateLimitScope::Ip => (self.config.per_ip.requests_per_minute, 60),
            RateLimitScope::User => {
                let tier = tier.unwrap_or(MembershipTier::Free);
                let tier_limits = self.config.per_tier.get(&tier)
                    .unwrap_or(&self.config.per_tier[&MembershipTier::Free]);

                match key.resource.as_deref() {
                    Some("ai_completions") => (tier_limits.ai_completions_per_minute, 60),
                    Some("ai_tokens") => (tier_limits.ai_tokens_per_day, 86400),
                    Some("conversation") => (tier_limits.conversation_messages_per_minute, 60),
                    Some("session") => (tier_limits.session_requests_per_hour, 3600),
                    Some("export") => (tier_limits.exports_per_hour, 3600),
                    _ => (tier_limits.general_requests_per_minute, 60),
                }
            }
            RateLimitScope::Resource => {
                let resource = key.resource.as_deref().unwrap_or("default");
                self.config.resources.get(resource)
                    .map(|r| (r.requests_per_minute, r.window_secs))
                    .unwrap_or((100, 60))
            }
        }
    }
}

#[async_trait]
impl RateLimiter for RedisRateLimiter {
    async fn check(&self, key: RateLimitKey) -> Result<RateLimitResult, RateLimitError> {
        let redis_key = self.key_for(&key);
        let (limit, window_secs) = self.limits_for(&key, None);

        // Token bucket algorithm using Redis
        let script = redis::Script::new(r#"
            local key = KEYS[1]
            local limit = tonumber(ARGV[1])
            local window = tonumber(ARGV[2])
            local now = tonumber(ARGV[3])

            local bucket = redis.call('HGETALL', key)
            local tokens = limit
            local last_update = now

            if #bucket > 0 then
                tokens = tonumber(bucket[2]) or limit
                last_update = tonumber(bucket[4]) or now
            end

            -- Refill tokens based on time elapsed
            local elapsed = now - last_update
            local refill = math.floor(elapsed * limit / window)
            tokens = math.min(limit, tokens + refill)

            if tokens > 0 then
                tokens = tokens - 1
                redis.call('HSET', key, 'tokens', tokens, 'last_update', now)
                redis.call('EXPIRE', key, window)
                return {1, tokens, limit}  -- allowed
            else
                local reset_at = last_update + window
                return {0, 0, limit, reset_at}  -- denied
            end
        "#);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let result: Vec<i64> = script
            .key(&redis_key)
            .arg(limit)
            .arg(window_secs)
            .arg(now as i64)
            .invoke_async(&mut self.redis.clone())
            .await
            .map_err(|e| RateLimitError::Unavailable(e.to_string()))?;

        if result[0] == 1 {
            Ok(RateLimitResult::Allowed(RateLimitStatus {
                limit,
                remaining: result[1] as u32,
                reset_at: Timestamp::from_unix_secs(now + window_secs as u64),
                window_secs,
            }))
        } else {
            let retry_after = (result[3] as u64).saturating_sub(now) as u32;
            Ok(RateLimitResult::Denied(RateLimitDenied {
                limit,
                retry_after_secs: retry_after,
                scope: key.scope,
                message: format!(
                    "Rate limit exceeded for {:?}. Retry after {} seconds.",
                    key.scope, retry_after
                ),
            }))
        }
    }

    async fn status(&self, key: RateLimitKey) -> Result<RateLimitStatus, RateLimitError> {
        // Similar to check but doesn't consume a token
        let redis_key = self.key_for(&key);
        let (limit, window_secs) = self.limits_for(&key, None);

        let bucket: Option<(u32, u64)> = redis::cmd("HGET")
            .arg(&redis_key)
            .arg("tokens")
            .query_async(&mut self.redis.clone())
            .await
            .ok();

        let remaining = bucket.map(|(t, _)| t).unwrap_or(limit);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(RateLimitStatus {
            limit,
            remaining,
            reset_at: Timestamp::from_unix_secs(now + window_secs as u64),
            window_secs,
        })
    }

    async fn reset(&self, key: RateLimitKey) -> Result<(), RateLimitError> {
        let redis_key = self.key_for(&key);
        redis::cmd("DEL")
            .arg(&redis_key)
            .query_async(&mut self.redis.clone())
            .await
            .map_err(|e| RateLimitError::Unavailable(e.to_string()))?;
        Ok(())
    }
}

impl RateLimitScope {
    fn as_str(&self) -> &'static str {
        match self {
            RateLimitScope::Global => "global",
            RateLimitScope::Ip => "ip",
            RateLimitScope::User => "user",
            RateLimitScope::Resource => "resource",
        }
    }
}
```

---

## HTTP Middleware

```rust
// backend/src/adapters/http/middleware/rate_limit.rs

use axum::{
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;

pub async fn rate_limit_middleware<B>(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    let ip = addr.ip().to_string();
    let path = request.uri().path();
    let user_id = extract_user_id(&request);
    let tier = user_id.as_ref()
        .and_then(|id| state.membership_service.get_tier(id).ok());

    // Layer 1: Global rate limit
    if let Err(denied) = check_limit(&state.rate_limiter, RateLimitKey::global()).await {
        return rate_limit_response(denied);
    }

    // Layer 2: Per-IP rate limit
    if let Err(denied) = check_limit(&state.rate_limiter, RateLimitKey::ip(&ip)).await {
        return rate_limit_response(denied);
    }

    // Layer 3: Per-user rate limit (if authenticated)
    if let Some(ref user_id) = user_id {
        if let Err(denied) = check_limit(&state.rate_limiter, RateLimitKey::user(user_id)).await {
            return rate_limit_response(denied);
        }

        // Layer 4: Resource-specific limits
        let resource = classify_resource(path);
        if let Some(resource) = resource {
            if let Err(denied) = check_limit(
                &state.rate_limiter,
                RateLimitKey::user_resource(user_id, resource)
            ).await {
                return rate_limit_response(denied);
            }
        }
    }

    // Add rate limit headers to response
    let response = next.run(request).await;
    add_rate_limit_headers(response, &state.rate_limiter, user_id).await
}

async fn check_limit(
    limiter: &Arc<dyn RateLimiter>,
    key: RateLimitKey,
) -> Result<RateLimitStatus, RateLimitDenied> {
    match limiter.check(key.clone()).await {
        Ok(RateLimitResult::Allowed(status)) => Ok(status),
        Ok(RateLimitResult::Denied(denied)) => Err(denied),
        Err(e) => {
            // SECURITY: Fail secure - deny on error
            // Per APPLICATION-SECURITY-STANDARD.md, we must never allow
            // requests when security controls fail. If the rate limiter
            // is unavailable, deny the request rather than allowing
            // potential abuse.
            tracing::error!("Rate limiter error (denying request): {}", e);
            Err(RateLimitDenied {
                limit: 0,
                retry_after_secs: 60,
                scope: key.scope,
                message: "Rate limiter temporarily unavailable".to_string(),
            })
        }
    }
}

fn rate_limit_response(denied: RateLimitDenied) -> Response {
    let headers = [
        ("X-RateLimit-Limit", denied.limit.to_string()),
        ("X-RateLimit-Remaining", "0".to_string()),
        ("Retry-After", denied.retry_after_secs.to_string()),
    ];

    let body = serde_json::json!({
        "error": "rate_limit_exceeded",
        "message": denied.message,
        "retry_after_secs": denied.retry_after_secs,
        "scope": format!("{:?}", denied.scope),
    });

    (StatusCode::TOO_MANY_REQUESTS, headers, Json(body)).into_response()
}

fn classify_resource(path: &str) -> Option<&'static str> {
    if path.contains("/conversations/") && path.contains("/messages") {
        Some("conversation")
    } else if path.contains("/ai/") || path.contains("/completions") {
        Some("ai_completions")
    } else if path.contains("/sessions") {
        Some("session")
    } else if path.contains("/export") {
        Some("export")
    } else {
        None
    }
}

async fn add_rate_limit_headers(
    mut response: Response,
    limiter: &Arc<dyn RateLimiter>,
    user_id: Option<UserId>,
) -> Response {
    if let Some(user_id) = user_id {
        if let Ok(status) = limiter.status(RateLimitKey::user(&user_id)).await {
            response.headers_mut().insert(
                "X-RateLimit-Limit",
                status.limit.to_string().parse().unwrap(),
            );
            response.headers_mut().insert(
                "X-RateLimit-Remaining",
                status.remaining.to_string().parse().unwrap(),
            );
            response.headers_mut().insert(
                "X-RateLimit-Reset",
                status.reset_at.as_unix_secs().to_string().parse().unwrap(),
            );
        }
    }
    response
}
```

---

## AI Token Rate Limiting

Special handling for AI token consumption (daily limits).

```rust
// backend/src/application/handlers/ai_token_rate_limiter.rs

pub struct AITokenRateLimiter {
    rate_limiter: Arc<dyn RateLimiter>,
    config: RateLimitConfig,
}

impl AITokenRateLimiter {
    /// Check if user can consume tokens, returns remaining budget
    pub async fn check_token_budget(
        &self,
        user_id: &UserId,
        tier: MembershipTier,
        requested_tokens: u32,
    ) -> Result<TokenBudgetResult, RateLimitError> {
        let key = RateLimitKey::user_resource(user_id, "ai_tokens");
        let daily_limit = self.config.per_tier.get(&tier)
            .map(|t| t.ai_tokens_per_day)
            .unwrap_or(10_000);

        let status = self.rate_limiter.status(key.clone()).await?;
        let consumed_today = daily_limit - status.remaining;

        if consumed_today + requested_tokens > daily_limit {
            return Ok(TokenBudgetResult::Exceeded {
                limit: daily_limit,
                consumed: consumed_today,
                requested: requested_tokens,
                reset_at: status.reset_at,
            });
        }

        Ok(TokenBudgetResult::Allowed {
            remaining: daily_limit - consumed_today - requested_tokens,
        })
    }

    /// Record token consumption after AI completion
    pub async fn consume_tokens(
        &self,
        user_id: &UserId,
        tokens: u32,
    ) -> Result<(), RateLimitError> {
        // Consume N tokens by making N rate limit checks
        // (In production, use a custom Lua script for atomic multi-consume)
        let key = RateLimitKey::user_resource(user_id, "ai_tokens");

        for _ in 0..tokens {
            self.rate_limiter.check(key.clone()).await?;
        }

        Ok(())
    }
}

pub enum TokenBudgetResult {
    Allowed { remaining: u32 },
    Exceeded {
        limit: u32,
        consumed: u32,
        requested: u32,
        reset_at: Timestamp,
    },
}
```

---

## WebSocket Connection Limiting

```rust
// backend/src/adapters/websocket/connection_limiter.rs

pub struct WebSocketConnectionLimiter {
    connections: RwLock<HashMap<UserId, u32>>,
    config: RateLimitConfig,
}

impl WebSocketConnectionLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
            config,
        }
    }

    pub async fn can_connect(&self, user_id: &UserId, tier: MembershipTier) -> bool {
        let limit = self.config.per_tier.get(&tier)
            .map(|t| t.websocket_connections)
            .unwrap_or(1);

        let connections = self.connections.read().await;
        let current = connections.get(user_id).copied().unwrap_or(0);

        current < limit
    }

    pub async fn register_connection(&self, user_id: &UserId) {
        let mut connections = self.connections.write().await;
        *connections.entry(user_id.clone()).or_insert(0) += 1;
    }

    pub async fn unregister_connection(&self, user_id: &UserId) {
        let mut connections = self.connections.write().await;
        if let Some(count) = connections.get_mut(user_id) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                connections.remove(user_id);
            }
        }
    }
}
```

---

## Events

```rust
// Rate limit events for monitoring and alerting

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitExceeded {
    pub user_id: Option<UserId>,
    pub ip: String,
    pub scope: String,
    pub resource: Option<String>,
    pub limit: u32,
    pub occurred_at: Timestamp,
}

impl DomainEvent for RateLimitExceeded {
    fn event_type(&self) -> &str { "system.rate_limit_exceeded.v1" }
    fn schema_version(&self) -> u32 { 1 }
    fn aggregate_type(&self) -> &str { "System" }
    fn aggregate_id(&self) -> String { "rate_limiter".to_string() }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AITokenBudgetExhausted {
    pub user_id: UserId,
    pub tier: String,
    pub daily_limit: u32,
    pub occurred_at: Timestamp,
}

impl DomainEvent for AITokenBudgetExhausted {
    fn event_type(&self) -> &str { "membership.ai_token_budget_exhausted.v1" }
    fn schema_version(&self) -> u32 { 1 }
    fn aggregate_type(&self) -> &str { "Membership" }
    fn aggregate_id(&self) -> String { self.user_id.to_string() }
}
```

---

## Dashboard Integration

```rust
// backend/src/adapters/http/routes/rate_limits.rs

/// GET /api/rate-limits
/// Returns current user's rate limit status across all resources
pub async fn get_rate_limit_status(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> impl IntoResponse {
    let user_id = &user.id;
    let tier = state.membership_service.get_tier(user_id).await?;

    let general = state.rate_limiter
        .status(RateLimitKey::user(user_id))
        .await?;

    let ai_tokens = state.rate_limiter
        .status(RateLimitKey::user_resource(user_id, "ai_tokens"))
        .await?;

    let conversation = state.rate_limiter
        .status(RateLimitKey::user_resource(user_id, "conversation"))
        .await?;

    Json(RateLimitStatusResponse {
        tier: tier.to_string(),
        limits: vec![
            ResourceLimit {
                resource: "general".to_string(),
                limit: general.limit,
                remaining: general.remaining,
                reset_at: general.reset_at,
                window: "per minute".to_string(),
            },
            ResourceLimit {
                resource: "ai_tokens".to_string(),
                limit: ai_tokens.limit,
                remaining: ai_tokens.remaining,
                reset_at: ai_tokens.reset_at,
                window: "per day".to_string(),
            },
            ResourceLimit {
                resource: "conversation_messages".to_string(),
                limit: conversation.limit,
                remaining: conversation.remaining,
                reset_at: conversation.reset_at,
                window: "per minute".to_string(),
            },
        ],
    })
}

#[derive(Serialize)]
struct RateLimitStatusResponse {
    tier: String,
    limits: Vec<ResourceLimit>,
}

#[derive(Serialize)]
struct ResourceLimit {
    resource: String,
    limit: u32,
    remaining: u32,
    reset_at: Timestamp,
    window: String,
}
```

---

## Frontend Integration

```typescript
// frontend/src/lib/api/rate-limits.ts

interface RateLimitHeaders {
  limit: number;
  remaining: number;
  resetAt: Date;
}

function extractRateLimitHeaders(response: Response): RateLimitHeaders | null {
  const limit = response.headers.get('X-RateLimit-Limit');
  const remaining = response.headers.get('X-RateLimit-Remaining');
  const reset = response.headers.get('X-RateLimit-Reset');

  if (!limit || !remaining || !reset) return null;

  return {
    limit: parseInt(limit, 10),
    remaining: parseInt(remaining, 10),
    resetAt: new Date(parseInt(reset, 10) * 1000),
  };
}

// Rate limit status store
export const rateLimitStore = writable<RateLimitHeaders | null>(null);

// Wrapper for fetch that handles rate limits
export async function fetchWithRateLimit(
  url: string,
  options?: RequestInit
): Promise<Response> {
  const response = await fetch(url, options);

  // Update rate limit store
  const limits = extractRateLimitHeaders(response);
  if (limits) {
    rateLimitStore.set(limits);
  }

  // Handle 429
  if (response.status === 429) {
    const body = await response.json();
    throw new RateLimitError(body.message, body.retry_after_secs);
  }

  return response;
}

export class RateLimitError extends Error {
  constructor(
    message: string,
    public retryAfterSecs: number
  ) {
    super(message);
    this.name = 'RateLimitError';
  }
}
```

```svelte
<!-- frontend/src/lib/components/RateLimitIndicator.svelte -->
<script lang="ts">
  import { rateLimitStore } from '$lib/api/rate-limits';

  $: percentRemaining = $rateLimitStore
    ? ($rateLimitStore.remaining / $rateLimitStore.limit) * 100
    : 100;

  $: statusColor = percentRemaining > 50 ? 'green' :
                   percentRemaining > 20 ? 'yellow' : 'red';
</script>

{#if $rateLimitStore}
  <div class="rate-limit-indicator" class:warning={percentRemaining < 20}>
    <div class="bar" style="width: {percentRemaining}%; background: {statusColor};" />
    <span>{$rateLimitStore.remaining}/{$rateLimitStore.limit} requests</span>
  </div>
{/if}
```

---

## Testing Strategy

### Unit Tests

```rust
#[tokio::test]
async fn test_token_bucket_allows_within_limit() {
    let limiter = InMemoryRateLimiter::new(RateLimitConfig::default());
    let key = RateLimitKey::user(&UserId::new("user-1"));

    // First 60 requests should succeed (default limit)
    for i in 0..60 {
        let result = limiter.check(key.clone()).await.unwrap();
        assert!(matches!(result, RateLimitResult::Allowed(_)),
            "Request {} should be allowed", i);
    }

    // 61st should be denied
    let result = limiter.check(key.clone()).await.unwrap();
    assert!(matches!(result, RateLimitResult::Denied(_)));
}

#[tokio::test]
async fn test_token_bucket_refills_over_time() {
    let limiter = InMemoryRateLimiter::new(RateLimitConfig::default());
    let key = RateLimitKey::user(&UserId::new("user-1"));

    // Exhaust limit
    for _ in 0..60 {
        limiter.check(key.clone()).await.unwrap();
    }

    // Wait for refill (simulate time passing)
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Should have refilled 1 token
    let result = limiter.check(key.clone()).await.unwrap();
    assert!(matches!(result, RateLimitResult::Allowed(_)));
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_middleware_adds_headers() {
    let app = create_test_app().await;

    let response = app
        .oneshot(Request::get("/api/sessions").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert!(response.headers().contains_key("X-RateLimit-Limit"));
    assert!(response.headers().contains_key("X-RateLimit-Remaining"));
    assert!(response.headers().contains_key("X-RateLimit-Reset"));
}

#[tokio::test]
async fn test_middleware_returns_429_when_exceeded() {
    let app = create_test_app().await;

    // Exhaust rate limit
    for _ in 0..100 {
        app.clone()
            .oneshot(Request::get("/api/sessions").body(Body::empty()).unwrap())
            .await
            .unwrap();
    }

    // Next request should be 429
    let response = app
        .oneshot(Request::get("/api/sessions").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(response.headers().contains_key("Retry-After"));
}
```

---

## Implementation Phases

### Phase 1: Core Infrastructure

- [ ] Define RateLimiter port
- [ ] Implement InMemoryRateLimiter for testing
- [ ] Create rate limit configuration types
- [ ] Write unit tests for token bucket algorithm

### Phase 2: Redis Adapter

- [ ] Implement RedisRateLimiter with Lua scripts
- [ ] Token bucket with atomic operations
- [ ] Integration tests with Redis
- [ ] Benchmark performance

### Phase 3: HTTP Middleware

- [ ] Implement rate_limit_middleware
- [ ] Global, IP, and user layers
- [ ] Resource classification
- [ ] Rate limit headers
- [ ] 429 response formatting

### Phase 4: AI Token Limiting

- [ ] AITokenRateLimiter with daily quotas
- [ ] Integration with conversation handlers
- [ ] TokenBudgetExhausted event
- [ ] Dashboard display

### Phase 5: WebSocket Limiting

- [ ] WebSocketConnectionLimiter
- [ ] Per-user connection limits
- [ ] Tier-based limits
- [ ] Graceful rejection

### Phase 6: Monitoring & Alerting

- [ ] RateLimitExceeded events
- [ ] Dashboard rate limit status endpoint
- [ ] Frontend rate limit indicator
- [ ] Alerting on high rate limit hit rates

---

## Security Considerations

### Fail-Secure Principle

Per APPLICATION-SECURITY-STANDARD.md, the rate limiter implements **fail-secure** behavior:

> **When security controls fail, deny access rather than allowing it.**

This is critical for rate limiting because:

1. **DoS Protection**: If Redis is unavailable, allowing all requests could enable denial-of-service attacks
2. **Cost Control**: AI endpoints have real costs; a failed rate limiter could result in significant unexpected charges
3. **Abuse Prevention**: Attackers could intentionally disrupt Redis to bypass rate limits

**Implementation:**

```rust
// CORRECT: Fail secure - deny on error
Err(e) => {
    tracing::error!("Rate limiter error (denying request): {}", e);
    Err(RateLimitDenied {
        limit: 0,
        retry_after_secs: 60,
        scope: key.scope,
        message: "Rate limiter temporarily unavailable".to_string(),
    })
}
```

**Anti-pattern (NEVER do this):**

```rust
// WRONG: Fail open - allows abuse when rate limiter is down
Err(e) => {
    tracing::error!("Rate limiter error: {}", e);
    Ok(RateLimitStatus { /* allow request */ })
}
```

### Operational Considerations

When the rate limiter fails secure:

1. **Monitoring**: Alert on `RateLimitError` logs to detect infrastructure issues
2. **Retry-After**: Use a reasonable retry period (60 seconds) to allow recovery
3. **User Communication**: Clear error message indicates temporary unavailability
4. **Graceful Degradation**: Frontend should handle 429 responses gracefully

---

## Exit Criteria

1. **All layers functional**: Global, IP, user, resource limits enforced
2. **Tier-aware**: Different limits for Free/Monthly/Annual tiers
3. **AI costs controlled**: Daily token limits prevent runaway costs
4. **Headers present**: All responses include rate limit headers
5. **429 responses correct**: Include Retry-After and clear error messages
6. **Frontend integrated**: Rate limit indicator shows remaining requests
7. **Events emitted**: RateLimitExceeded events for monitoring
8. **Fail-secure**: Rate limiter errors result in request denial, not allowance
