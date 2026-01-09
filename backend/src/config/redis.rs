//! Redis configuration

use serde::Deserialize;
use std::time::Duration;

use super::error::ValidationError;

/// Redis configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    /// Redis connection URL
    pub url: String,

    /// Connection pool size
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

impl RedisConfig {
    /// Get timeout as Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    /// Validate Redis configuration
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

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            pool_size: default_pool_size(),
            timeout_secs: default_timeout(),
        }
    }
}

fn default_pool_size() -> u32 {
    10
}

fn default_timeout() -> u64 {
    5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_config_defaults() {
        let config = RedisConfig::default();
        assert_eq!(config.pool_size, 10);
        assert_eq!(config.timeout_secs, 5);
    }

    #[test]
    fn test_timeout_duration() {
        let config = RedisConfig {
            timeout_secs: 10,
            ..Default::default()
        };
        assert_eq!(config.timeout(), Duration::from_secs(10));
    }

    #[test]
    fn test_validation_missing_url() {
        let config = RedisConfig::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_url() {
        let config = RedisConfig {
            url: "http://localhost:6379".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_valid_redis_url() {
        let config = RedisConfig {
            url: "redis://localhost:6379".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validation_valid_rediss_url() {
        let config = RedisConfig {
            url: "rediss://user:pass@redis.example.com:6380".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
}
