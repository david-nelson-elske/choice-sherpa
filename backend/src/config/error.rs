//! Configuration error types

use thiserror::Error;

/// Errors that can occur during configuration loading
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration loading failed: {0}")]
    LoadError(#[from] config::ConfigError),

    #[error("Validation failed: {0}")]
    ValidationFailed(#[from] ValidationError),
}

/// Errors that can occur during configuration validation
#[derive(Debug, Error)]
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

    #[error("Auth authority must use HTTPS in production")]
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
