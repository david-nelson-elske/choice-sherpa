# Infrastructure: Application Configuration

**Type:** Cross-Cutting Infrastructure
**Priority:** P0 (Required for all adapters)
**Last Updated:** 2026-01-09

> Complete specification for environment-based configuration loading, validation, and typed config structs.

---

## Overview

Choice Sherpa uses environment variables for configuration with the `config` and `dotenvy` crates. This specification defines:
1. Configuration structure and hierarchy
2. Environment file loading precedence
3. Validation rules for all settings
4. Secrets handling and security
5. Type-safe config access patterns

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Application Startup                                   │
│                                                                              │
│   ┌────────────────┐    ┌────────────────┐    ┌────────────────┐           │
│   │   .env file    │───▶│    dotenvy     │───▶│  Environment   │           │
│   │  (if exists)   │    │    loader      │    │   Variables    │           │
│   └────────────────┘    └────────────────┘    └───────┬────────┘           │
│                                                        │                     │
│                                                        ▼                     │
│                              ┌─────────────────────────────────────────┐    │
│                              │           config crate                   │    │
│                              │                                          │    │
│                              │   Environment::default()                 │    │
│                              │   .prefix("CHOICE_SHERPA")              │    │
│                              │   .separator("_")                        │    │
│                              │   .try_deserialize::<AppConfig>()       │    │
│                              └─────────────────────────┬───────────────┘    │
│                                                        │                     │
│                                                        ▼                     │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                          AppConfig                                   │   │
│   │                                                                      │   │
│   │   ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐       │   │
│   │   │  Server   │  │ Database  │  │   Redis   │  │    Auth   │       │   │
│   │   │  Config   │  │  Config   │  │  Config   │  │  Config   │       │   │
│   │   └───────────┘  └───────────┘  └───────────┘  └───────────┘       │   │
│   │                                                                      │   │
│   │   ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐       │   │
│   │   │    AI     │  │  Payment  │  │   Email   │  │ Features  │       │   │
│   │   │  Config   │  │  Config   │  │  Config   │  │  Flags    │       │   │
│   │   └───────────┘  └───────────┘  └───────────┘  └───────────┘       │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Configuration Structure

### Root Configuration

```rust
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub auth: AuthConfig,
    pub ai: AiConfig,
    pub payment: PaymentConfig,
    pub email: EmailConfig,
    pub features: FeatureFlags,
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn load() -> Result<Self, ConfigError> {
        // Load .env file if present (development)
        dotenvy::dotenv().ok();

        config::Config::builder()
            .add_source(
                config::Environment::default()
                    .prefix("CHOICE_SHERPA")
                    .separator("_")
            )
            .build()?
            .try_deserialize()
    }

    /// Validate all configuration values
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.server.validate()?;
        self.database.validate()?;
        self.redis.validate()?;
        self.auth.validate()?;
        self.ai.validate()?;
        self.payment.validate()?;
        self.email.validate()?;
        Ok(())
    }
}
```

### Server Configuration

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Host address to bind to (default: 0.0.0.0)
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to listen on (default: 8080)
    #[serde(default = "default_port")]
    pub port: u16,

    /// Environment name (development, staging, production)
    #[serde(default = "default_environment")]
    pub environment: Environment,

    /// Rust log filter directive
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Request timeout in seconds
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,

    /// CORS allowed origins (comma-separated)
    pub cors_origins: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl ServerConfig {
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid socket address")
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.port == 0 {
            return Err(ValidationError::InvalidPort);
        }
        if self.request_timeout_secs == 0 || self.request_timeout_secs > 300 {
            return Err(ValidationError::InvalidTimeout);
        }
        Ok(())
    }
}

fn default_host() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 8080 }
fn default_environment() -> Environment { Environment::Development }
fn default_log_level() -> String { "info,choice_sherpa=debug,sqlx=warn".to_string() }
fn default_request_timeout() -> u64 { 30 }
```

### Database Configuration

```rust
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// PostgreSQL connection URL
    pub url: String,

    /// Minimum connections to maintain
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// Maximum connections allowed
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Connection acquire timeout in seconds
    #[serde(default = "default_acquire_timeout")]
    pub acquire_timeout_secs: u64,

    /// Idle connection timeout in seconds
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_secs: u64,

    /// Maximum connection lifetime in seconds
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime_secs: u64,

    /// Run migrations on startup
    #[serde(default = "default_run_migrations")]
    pub run_migrations: bool,
}

impl DatabaseConfig {
    pub fn acquire_timeout(&self) -> Duration {
        Duration::from_secs(self.acquire_timeout_secs)
    }

    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.idle_timeout_secs)
    }

    pub fn max_lifetime(&self) -> Duration {
        Duration::from_secs(self.max_lifetime_secs)
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.url.is_empty() {
            return Err(ValidationError::MissingRequired("DATABASE_URL"));
        }
        if !self.url.starts_with("postgres://") && !self.url.starts_with("postgresql://") {
            return Err(ValidationError::InvalidDatabaseUrl);
        }
        if self.min_connections > self.max_connections {
            return Err(ValidationError::InvalidPoolSize);
        }
        if self.max_connections > 100 {
            return Err(ValidationError::PoolSizeTooLarge);
        }
        Ok(())
    }
}

fn default_min_connections() -> u32 { 5 }
fn default_max_connections() -> u32 { 20 }
fn default_acquire_timeout() -> u64 { 30 }
fn default_idle_timeout() -> u64 { 600 }
fn default_max_lifetime() -> u64 { 1800 }
fn default_run_migrations() -> bool { false }
```

### Redis Configuration

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    /// Redis connection URL
    pub url: String,

    /// Connection pool size
    #[serde(default = "default_redis_pool_size")]
    pub pool_size: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_redis_timeout")]
    pub timeout_secs: u64,
}

impl RedisConfig {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.url.is_empty() {
            return Err(ValidationError::MissingRequired("REDIS_URL"));
        }
        if !self.url.starts_with("redis://") && !self.url.starts_with("rediss://") {
            return Err(ValidationError::InvalidRedisUrl);
        }
        Ok(())
    }
}

fn default_redis_pool_size() -> u32 { 10 }
fn default_redis_timeout() -> u64 { 5 }
```

### Authentication Configuration

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    /// Zitadel authority URL
    pub zitadel_authority: String,

    /// OAuth2 client ID
    pub zitadel_client_id: String,

    /// Expected audience for tokens
    pub zitadel_audience: String,

    /// JWKS cache TTL in seconds
    #[serde(default = "default_jwks_cache_ttl")]
    pub jwks_cache_ttl_secs: u64,
}

impl AuthConfig {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.zitadel_authority.is_empty() {
            return Err(ValidationError::MissingRequired("ZITADEL_AUTHORITY"));
        }
        if self.zitadel_client_id.is_empty() {
            return Err(ValidationError::MissingRequired("ZITADEL_CLIENT_ID"));
        }
        if self.zitadel_audience.is_empty() {
            return Err(ValidationError::MissingRequired("ZITADEL_AUDIENCE"));
        }
        if !self.zitadel_authority.starts_with("https://") {
            return Err(ValidationError::AuthorityMustBeHttps);
        }
        Ok(())
    }
}

fn default_jwks_cache_ttl() -> u64 { 3600 }
```

### AI Provider Configuration

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct AiConfig {
    /// OpenAI API key
    pub openai_api_key: Option<String>,

    /// Anthropic API key
    pub anthropic_api_key: Option<String>,

    /// Primary AI provider (openai or anthropic)
    #[serde(default = "default_ai_provider")]
    pub primary_provider: AiProvider,

    /// Fallback AI provider
    pub fallback_provider: Option<AiProvider>,

    /// Request timeout in seconds
    #[serde(default = "default_ai_timeout")]
    pub timeout_secs: u64,

    /// Maximum retries on failure
    #[serde(default = "default_ai_retries")]
    pub max_retries: u32,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    OpenAI,
    Anthropic,
}

impl AiConfig {
    pub fn validate(&self) -> Result<(), ValidationError> {
        // At least one provider must have an API key
        if self.openai_api_key.is_none() && self.anthropic_api_key.is_none() {
            return Err(ValidationError::NoAiProviderConfigured);
        }

        // Primary provider must have an API key
        match self.primary_provider {
            AiProvider::OpenAI if self.openai_api_key.is_none() => {
                return Err(ValidationError::MissingRequired("OPENAI_API_KEY"));
            }
            AiProvider::Anthropic if self.anthropic_api_key.is_none() => {
                return Err(ValidationError::MissingRequired("ANTHROPIC_API_KEY"));
            }
            _ => {}
        }

        Ok(())
    }
}

fn default_ai_provider() -> AiProvider { AiProvider::Anthropic }
fn default_ai_timeout() -> u64 { 120 }
fn default_ai_retries() -> u32 { 3 }
```

### Payment Configuration

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct PaymentConfig {
    /// Stripe API key
    pub stripe_api_key: String,

    /// Stripe webhook signing secret
    pub stripe_webhook_secret: String,

    /// Stripe price IDs for plans
    pub stripe_monthly_price_id: Option<String>,
    pub stripe_annual_price_id: Option<String>,
}

impl PaymentConfig {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.stripe_api_key.is_empty() {
            return Err(ValidationError::MissingRequired("STRIPE_API_KEY"));
        }
        if self.stripe_webhook_secret.is_empty() {
            return Err(ValidationError::MissingRequired("STRIPE_WEBHOOK_SECRET"));
        }
        // Verify key prefixes for safety
        if !self.stripe_api_key.starts_with("sk_") {
            return Err(ValidationError::InvalidStripeKey);
        }
        if !self.stripe_webhook_secret.starts_with("whsec_") {
            return Err(ValidationError::InvalidStripeWebhookSecret);
        }
        Ok(())
    }

    pub fn is_test_mode(&self) -> bool {
        self.stripe_api_key.starts_with("sk_test_")
    }
}
```

### Email Configuration

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct EmailConfig {
    /// Resend API key
    pub resend_api_key: String,

    /// From email address
    #[serde(default = "default_from_email")]
    pub from_email: String,

    /// From name
    #[serde(default = "default_from_name")]
    pub from_name: String,
}

impl EmailConfig {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.resend_api_key.is_empty() {
            return Err(ValidationError::MissingRequired("RESEND_API_KEY"));
        }
        if !self.resend_api_key.starts_with("re_") {
            return Err(ValidationError::InvalidResendKey);
        }
        if !self.from_email.contains('@') {
            return Err(ValidationError::InvalidFromEmail);
        }
        Ok(())
    }
}

fn default_from_email() -> String { "noreply@choicesherpa.com".to_string() }
fn default_from_name() -> String { "Choice Sherpa".to_string() }
```

### Feature Flags

```rust
#[derive(Debug, Clone, Deserialize, Default)]
pub struct FeatureFlags {
    /// Enable WebSocket streaming for conversations
    #[serde(default)]
    pub enable_streaming: bool,

    /// Enable AI fallback provider
    #[serde(default)]
    pub enable_ai_fallback: bool,

    /// Enable detailed error messages (disable in production)
    #[serde(default)]
    pub verbose_errors: bool,

    /// Enable request tracing
    #[serde(default = "default_enable_tracing")]
    pub enable_tracing: bool,
}

fn default_enable_tracing() -> bool { true }
```

---

## Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration loading failed: {0}")]
    LoadError(#[from] config::ConfigError),

    #[error("Environment variable not found: {0}")]
    MissingEnv(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(#[from] ValidationError),
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Required configuration missing: {0}")]
    MissingRequired(&'static str),

    #[error("Invalid port number")]
    InvalidPort,

    #[error("Invalid request timeout")]
    InvalidTimeout,

    #[error("Invalid database URL format")]
    InvalidDatabaseUrl,

    #[error("Invalid Redis URL format")]
    InvalidRedisUrl,

    #[error("Pool min_connections exceeds max_connections")]
    InvalidPoolSize,

    #[error("Pool size exceeds maximum allowed (100)")]
    PoolSizeTooLarge,

    #[error("Auth authority must use HTTPS")]
    AuthorityMustBeHttps,

    #[error("No AI provider configured")]
    NoAiProviderConfigured,

    #[error("Invalid Stripe API key format")]
    InvalidStripeKey,

    #[error("Invalid Stripe webhook secret format")]
    InvalidStripeWebhookSecret,

    #[error("Invalid Resend API key format")]
    InvalidResendKey,

    #[error("Invalid from email address")]
    InvalidFromEmail,
}
```

---

## Environment Variables Reference

### Required Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `CHOICE_SHERPA_DATABASE_URL` | PostgreSQL connection URL | `postgresql://user:pass@localhost:5432/db` |
| `CHOICE_SHERPA_REDIS_URL` | Redis connection URL | `redis://localhost:6379` |
| `CHOICE_SHERPA_AUTH_ZITADEL_AUTHORITY` | Zitadel OIDC authority | `https://auth.example.com` |
| `CHOICE_SHERPA_AUTH_ZITADEL_CLIENT_ID` | OAuth2 client ID | `choice-sherpa` |
| `CHOICE_SHERPA_AUTH_ZITADEL_AUDIENCE` | Token audience | `choice-sherpa-api` |
| `CHOICE_SHERPA_PAYMENT_STRIPE_API_KEY` | Stripe secret key | `sk_test_xxx` |
| `CHOICE_SHERPA_PAYMENT_STRIPE_WEBHOOK_SECRET` | Webhook signing secret | `whsec_xxx` |
| `CHOICE_SHERPA_EMAIL_RESEND_API_KEY` | Resend API key | `re_xxx` |

### Optional Variables (with defaults)

| Variable | Default | Description |
|----------|---------|-------------|
| `CHOICE_SHERPA_SERVER_HOST` | `0.0.0.0` | Bind address |
| `CHOICE_SHERPA_SERVER_PORT` | `8080` | Listen port |
| `CHOICE_SHERPA_SERVER_ENVIRONMENT` | `development` | Environment name |
| `CHOICE_SHERPA_SERVER_LOG_LEVEL` | `info,choice_sherpa=debug` | Log filter |
| `CHOICE_SHERPA_DATABASE_MIN_CONNECTIONS` | `5` | Pool minimum |
| `CHOICE_SHERPA_DATABASE_MAX_CONNECTIONS` | `20` | Pool maximum |
| `CHOICE_SHERPA_AI_PRIMARY_PROVIDER` | `anthropic` | Primary AI |
| `CHOICE_SHERPA_AI_OPENAI_API_KEY` | None | OpenAI key |
| `CHOICE_SHERPA_AI_ANTHROPIC_API_KEY` | None | Anthropic key |

---

## .env File Format

### Development (.env)

```env
# Server
CHOICE_SHERPA_SERVER_HOST=127.0.0.1
CHOICE_SHERPA_SERVER_PORT=8080
CHOICE_SHERPA_SERVER_ENVIRONMENT=development
CHOICE_SHERPA_SERVER_LOG_LEVEL=debug,choice_sherpa=trace,sqlx=info

# Database
CHOICE_SHERPA_DATABASE_URL=postgresql://choice-sherpa:password@localhost:5432/choice_sherpa
CHOICE_SHERPA_DATABASE_MIN_CONNECTIONS=2
CHOICE_SHERPA_DATABASE_MAX_CONNECTIONS=5
CHOICE_SHERPA_DATABASE_RUN_MIGRATIONS=true

# Redis
CHOICE_SHERPA_REDIS_URL=redis://localhost:6379

# Auth (local Zitadel or mock)
CHOICE_SHERPA_AUTH_ZITADEL_AUTHORITY=https://localhost:8443
CHOICE_SHERPA_AUTH_ZITADEL_CLIENT_ID=choice-sherpa-dev
CHOICE_SHERPA_AUTH_ZITADEL_AUDIENCE=choice-sherpa-api

# AI (at least one required)
CHOICE_SHERPA_AI_PRIMARY_PROVIDER=anthropic
CHOICE_SHERPA_AI_ANTHROPIC_API_KEY=sk-ant-xxx
CHOICE_SHERPA_AI_OPENAI_API_KEY=sk-xxx

# Payment (Stripe test mode)
CHOICE_SHERPA_PAYMENT_STRIPE_API_KEY=sk_test_xxx
CHOICE_SHERPA_PAYMENT_STRIPE_WEBHOOK_SECRET=whsec_xxx

# Email
CHOICE_SHERPA_EMAIL_RESEND_API_KEY=re_xxx

# Features
CHOICE_SHERPA_FEATURES_VERBOSE_ERRORS=true
CHOICE_SHERPA_FEATURES_ENABLE_STREAMING=true
```

---

## Loading Precedence

1. **System environment variables** (highest priority)
2. **`.env` file** (loaded by dotenvy)
3. **Default values** (defined in config structs)

This allows production to use environment variables from container orchestration while development uses `.env` files.

---

## Security Considerations

### Secrets Handling

1. **Never log secrets** - API keys, passwords masked in logs
2. **Don't commit .env** - Add to `.gitignore`
3. **Validate key formats** - Detect accidental placeholder usage
4. **Require HTTPS** for auth in production

### Debug Implementation

```rust
impl std::fmt::Debug for AppConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppConfig")
            .field("server", &self.server)
            .field("database", &MaskedConfig("DATABASE_URL"))
            .field("redis", &MaskedConfig("REDIS_URL"))
            .field("auth", &self.auth)
            .field("ai", &MaskedConfig("API_KEYS"))
            .field("payment", &MaskedConfig("STRIPE_KEYS"))
            .field("email", &MaskedConfig("RESEND_KEY"))
            .field("features", &self.features)
            .finish()
    }
}

struct MaskedConfig(&'static str);
impl std::fmt::Debug for MaskedConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{} MASKED]", self.0)
    }
}
```

---

## File Structure

```
backend/src/
├── config/
│   ├── mod.rs          # Module exports, AppConfig
│   ├── server.rs       # ServerConfig
│   ├── database.rs     # DatabaseConfig
│   ├── redis.rs        # RedisConfig
│   ├── auth.rs         # AuthConfig
│   ├── ai.rs           # AiConfig
│   ├── payment.rs      # PaymentConfig
│   ├── email.rs        # EmailConfig
│   ├── features.rs     # FeatureFlags
│   └── error.rs        # ConfigError, ValidationError
└── lib.rs              # pub mod config;
```

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_server_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_database_config_validation() {
        let mut config = DatabaseConfig {
            url: "postgresql://localhost/test".to_string(),
            min_connections: 10,
            max_connections: 5, // Invalid: min > max
            ..Default::default()
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_ai_config_requires_provider_key() {
        let config = AiConfig {
            primary_provider: AiProvider::Anthropic,
            anthropic_api_key: None, // Missing key for primary
            openai_api_key: Some("sk-xxx".to_string()),
            ..Default::default()
        };

        assert!(config.validate().is_err());
    }
}
```

### Integration Tests

```rust
#[test]
fn test_load_from_environment() {
    // Set required env vars
    env::set_var("CHOICE_SHERPA_DATABASE_URL", "postgresql://test@localhost/test");
    env::set_var("CHOICE_SHERPA_REDIS_URL", "redis://localhost");
    // ... other required vars

    let config = AppConfig::load().expect("Should load config");
    assert!(config.validate().is_ok());
}
```

---

## Related Documents

- **Database Connection Pool**: `features/infrastructure/database-connection-pool.md`
- **Docker Development**: `features/infrastructure/docker-development.md`
- **System Architecture**: `docs/architecture/SYSTEM-ARCHITECTURE.md`

---

*Version: 1.0.0*
*Created: 2026-01-09*
