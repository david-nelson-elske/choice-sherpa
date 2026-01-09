//! Database configuration

use serde::Deserialize;
use std::time::Duration;

use super::error::ValidationError;

/// Database configuration
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
    #[serde(default)]
    pub run_migrations: bool,
}

impl DatabaseConfig {
    /// Get acquire timeout as Duration
    pub fn acquire_timeout(&self) -> Duration {
        Duration::from_secs(self.acquire_timeout_secs)
    }

    /// Get idle timeout as Duration
    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.idle_timeout_secs)
    }

    /// Get max lifetime as Duration
    pub fn max_lifetime(&self) -> Duration {
        Duration::from_secs(self.max_lifetime_secs)
    }

    /// Validate database configuration
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

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            min_connections: default_min_connections(),
            max_connections: default_max_connections(),
            acquire_timeout_secs: default_acquire_timeout(),
            idle_timeout_secs: default_idle_timeout(),
            max_lifetime_secs: default_max_lifetime(),
            run_migrations: false,
        }
    }
}

fn default_min_connections() -> u32 {
    5
}

fn default_max_connections() -> u32 {
    20
}

fn default_acquire_timeout() -> u64 {
    30
}

fn default_idle_timeout() -> u64 {
    600
}

fn default_max_lifetime() -> u64 {
    1800
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_defaults() {
        let config = DatabaseConfig::default();
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.max_connections, 20);
        assert!(!config.run_migrations);
    }

    #[test]
    fn test_timeout_durations() {
        let config = DatabaseConfig {
            acquire_timeout_secs: 10,
            idle_timeout_secs: 300,
            max_lifetime_secs: 600,
            ..Default::default()
        };
        assert_eq!(config.acquire_timeout(), Duration::from_secs(10));
        assert_eq!(config.idle_timeout(), Duration::from_secs(300));
        assert_eq!(config.max_lifetime(), Duration::from_secs(600));
    }

    #[test]
    fn test_validation_missing_url() {
        let config = DatabaseConfig::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_url() {
        let config = DatabaseConfig {
            url: "mysql://localhost/test".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_pool_size() {
        let config = DatabaseConfig {
            url: "postgresql://localhost/test".to_string(),
            min_connections: 10,
            max_connections: 5,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_pool_too_large() {
        let config = DatabaseConfig {
            url: "postgresql://localhost/test".to_string(),
            max_connections: 150,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_valid_config() {
        let config = DatabaseConfig {
            url: "postgresql://user:pass@localhost:5432/test".to_string(),
            min_connections: 5,
            max_connections: 20,
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
}
