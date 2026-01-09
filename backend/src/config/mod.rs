//! Application configuration module
//!
//! This module provides type-safe configuration loading from environment variables
//! using the `config` and `dotenvy` crates. Configuration is loaded with the
//! `CHOICE_SHERPA_` prefix and nested values use underscores as separators.
//!
//! # Example
//!
//! ```no_run
//! use choice_sherpa::config::AppConfig;
//!
//! let config = AppConfig::load().expect("Failed to load configuration");
//! config.validate().expect("Invalid configuration");
//!
//! println!("Server running on {}", config.server.socket_addr());
//! ```

mod ai;
mod auth;
mod database;
mod email;
mod error;
mod features;
mod payment;
mod redis;
mod server;

pub use ai::{AiConfig, AiProvider};
pub use auth::AuthConfig;
pub use database::DatabaseConfig;
pub use email::EmailConfig;
pub use error::{ConfigError, ValidationError};
pub use features::FeatureFlags;
pub use payment::PaymentConfig;
pub use redis::RedisConfig;
pub use server::{Environment, ServerConfig};

use serde::Deserialize;

/// Root application configuration
///
/// Contains all configuration sections for the Choice Sherpa application.
/// Load using [`AppConfig::load()`] which reads from environment variables.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Server configuration (host, port, environment)
    #[serde(default)]
    pub server: ServerConfig,

    /// Database configuration (PostgreSQL connection)
    pub database: DatabaseConfig,

    /// Redis configuration (cache/pubsub)
    pub redis: RedisConfig,

    /// Authentication configuration (Zitadel OIDC)
    pub auth: AuthConfig,

    /// AI provider configuration (OpenAI/Anthropic)
    #[serde(default)]
    pub ai: AiConfig,

    /// Payment configuration (Stripe)
    pub payment: PaymentConfig,

    /// Email configuration (Resend)
    pub email: EmailConfig,

    /// Feature flags
    #[serde(default)]
    pub features: FeatureFlags,
}

impl AppConfig {
    /// Load configuration from environment variables
    ///
    /// This function:
    /// 1. Loads `.env` file if present (for development)
    /// 2. Reads environment variables with `CHOICE_SHERPA` prefix
    /// 3. Uses `__` (double underscore) to separate nested values
    /// 4. Deserializes into typed configuration structs
    ///
    /// # Environment Variable Format
    ///
    /// - `CHOICE_SHERPA__SERVER__PORT=8080` -> `server.port = 8080`
    /// - `CHOICE_SHERPA__DATABASE__URL=...` -> `database.url = ...`
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if:
    /// - Required environment variables are missing
    /// - Values cannot be parsed into expected types
    pub fn load() -> Result<Self, ConfigError> {
        // Load .env file if present (development)
        dotenvy::dotenv().ok();

        let config = config::Config::builder()
            .add_source(
                config::Environment::default()
                    .prefix("CHOICE_SHERPA")
                    .separator("__"),
            )
            .build()?
            .try_deserialize()?;

        Ok(config)
    }

    /// Validate all configuration values
    ///
    /// Performs semantic validation of configuration:
    /// - URL formats
    /// - Pool size constraints
    /// - Required API key prefixes
    /// - Production-specific requirements (e.g., HTTPS)
    ///
    /// # Errors
    ///
    /// Returns `ValidationError` if any configuration value is invalid.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.server.validate()?;
        self.database.validate()?;
        self.redis.validate()?;
        self.auth.validate(&self.server.environment)?;
        self.ai.validate()?;
        self.payment.validate()?;
        self.email.validate()?;
        Ok(())
    }

    /// Check if running in production environment
    pub fn is_production(&self) -> bool {
        self.server.is_production()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // Mutex to ensure tests don't run in parallel (env vars are global)
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    /// Helper to set environment variables for testing
    /// Uses double underscores to separate nested config values
    fn set_minimal_env() {
        env::set_var("CHOICE_SHERPA__DATABASE__URL", "postgresql://test@localhost/test");
        env::set_var("CHOICE_SHERPA__REDIS__URL", "redis://localhost:6379");
        env::set_var("CHOICE_SHERPA__AUTH__ZITADEL_AUTHORITY", "https://auth.example.com");
        env::set_var("CHOICE_SHERPA__AUTH__ZITADEL_CLIENT_ID", "client-id");
        env::set_var("CHOICE_SHERPA__AUTH__ZITADEL_AUDIENCE", "audience");
        env::set_var("CHOICE_SHERPA__AI__ANTHROPIC_API_KEY", "sk-ant-xxx");
        env::set_var("CHOICE_SHERPA__PAYMENT__STRIPE_API_KEY", "sk_test_xxx");
        env::set_var("CHOICE_SHERPA__PAYMENT__STRIPE_WEBHOOK_SECRET", "whsec_xxx");
        env::set_var("CHOICE_SHERPA__EMAIL__RESEND_API_KEY", "re_xxx");
    }

    /// Helper to clear environment variables after testing
    fn clear_env() {
        env::remove_var("CHOICE_SHERPA__DATABASE__URL");
        env::remove_var("CHOICE_SHERPA__REDIS__URL");
        env::remove_var("CHOICE_SHERPA__AUTH__ZITADEL_AUTHORITY");
        env::remove_var("CHOICE_SHERPA__AUTH__ZITADEL_CLIENT_ID");
        env::remove_var("CHOICE_SHERPA__AUTH__ZITADEL_AUDIENCE");
        env::remove_var("CHOICE_SHERPA__AI__ANTHROPIC_API_KEY");
        env::remove_var("CHOICE_SHERPA__PAYMENT__STRIPE_API_KEY");
        env::remove_var("CHOICE_SHERPA__PAYMENT__STRIPE_WEBHOOK_SECRET");
        env::remove_var("CHOICE_SHERPA__EMAIL__RESEND_API_KEY");
        env::remove_var("CHOICE_SHERPA__SERVER__PORT");
        env::remove_var("CHOICE_SHERPA__SERVER__ENVIRONMENT");
    }

    #[test]
    fn test_load_from_environment() {
        let _guard = ENV_MUTEX.lock().unwrap();
        set_minimal_env();
        let result = AppConfig::load();
        clear_env();

        assert!(result.is_ok(), "Failed to load config: {:?}", result.err());
        let config = result.unwrap();
        assert_eq!(config.database.url, "postgresql://test@localhost/test");
        assert_eq!(config.redis.url, "redis://localhost:6379");
    }

    #[test]
    fn test_validate_full_config() {
        let _guard = ENV_MUTEX.lock().unwrap();
        set_minimal_env();
        let result = AppConfig::load();
        clear_env();

        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_server_defaults() {
        let _guard = ENV_MUTEX.lock().unwrap();
        set_minimal_env();
        let result = AppConfig::load();
        clear_env();

        let config = result.unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.environment, Environment::Development);
    }

    #[test]
    fn test_is_production() {
        let _guard = ENV_MUTEX.lock().unwrap();
        set_minimal_env();
        env::set_var("CHOICE_SHERPA__SERVER__ENVIRONMENT", "production");
        let result = AppConfig::load();
        clear_env();

        let config = result.unwrap();
        assert!(config.is_production());
    }

    #[test]
    fn test_custom_server_port() {
        let _guard = ENV_MUTEX.lock().unwrap();
        set_minimal_env();
        env::set_var("CHOICE_SHERPA__SERVER__PORT", "3000");
        let result = AppConfig::load();
        clear_env();

        let config = result.unwrap();
        assert_eq!(config.server.port, 3000);
    }
}
