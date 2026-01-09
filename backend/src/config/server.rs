//! Server configuration

use serde::Deserialize;
use std::net::SocketAddr;

use super::error::ValidationError;

/// Server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Host address to bind to
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to listen on
    #[serde(default = "default_port")]
    pub port: u16,

    /// Environment name
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

/// Application environment
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    #[default]
    Development,
    Staging,
    Production,
}

impl ServerConfig {
    /// Get the socket address to bind to
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid socket address")
    }

    /// Check if running in production
    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    /// Get CORS origins as a vector
    pub fn cors_origins_list(&self) -> Vec<String> {
        self.cors_origins
            .as_ref()
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default()
    }

    /// Validate server configuration
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

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            environment: default_environment(),
            log_level: default_log_level(),
            request_timeout_secs: default_request_timeout(),
            cors_origins: None,
        }
    }
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_environment() -> Environment {
    Environment::Development
}

fn default_log_level() -> String {
    "info,choice_sherpa=debug,sqlx=warn".to_string()
}

fn default_request_timeout() -> u64 {
    30
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.environment, Environment::Development);
    }

    #[test]
    fn test_socket_addr() {
        let config = ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            ..Default::default()
        };
        let addr = config.socket_addr();
        assert_eq!(addr.to_string(), "127.0.0.1:3000");
    }

    #[test]
    fn test_is_production() {
        let mut config = ServerConfig::default();
        assert!(!config.is_production());

        config.environment = Environment::Production;
        assert!(config.is_production());
    }

    #[test]
    fn test_cors_origins_parsing() {
        let config = ServerConfig {
            cors_origins: Some("http://localhost:5173, http://localhost:3000".to_string()),
            ..Default::default()
        };
        let origins = config.cors_origins_list();
        assert_eq!(origins.len(), 2);
        assert_eq!(origins[0], "http://localhost:5173");
        assert_eq!(origins[1], "http://localhost:3000");
    }

    #[test]
    fn test_validation_invalid_port() {
        let config = ServerConfig {
            port: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_timeout() {
        let config = ServerConfig {
            request_timeout_secs: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        let config = ServerConfig {
            request_timeout_secs: 500,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }
}
