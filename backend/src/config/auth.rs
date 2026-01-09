//! Authentication configuration

use serde::Deserialize;
use std::time::Duration;

use super::error::ValidationError;
use super::server::Environment;

/// Authentication configuration (Zitadel OIDC)
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
    /// Get JWKS cache TTL as Duration
    pub fn jwks_cache_ttl(&self) -> Duration {
        Duration::from_secs(self.jwks_cache_ttl_secs)
    }

    /// Validate authentication configuration
    ///
    /// In production, requires HTTPS for the authority URL.
    /// In development, allows localhost with HTTP/HTTPS.
    pub fn validate(&self, environment: &Environment) -> Result<(), ValidationError> {
        if self.zitadel_authority.is_empty() {
            return Err(ValidationError::MissingRequired("ZITADEL_AUTHORITY"));
        }
        if self.zitadel_client_id.is_empty() {
            return Err(ValidationError::MissingRequired("ZITADEL_CLIENT_ID"));
        }
        if self.zitadel_audience.is_empty() {
            return Err(ValidationError::MissingRequired("ZITADEL_AUDIENCE"));
        }

        // In production, require HTTPS
        if *environment == Environment::Production && !self.zitadel_authority.starts_with("https://") {
            return Err(ValidationError::AuthorityMustBeHttps);
        }

        Ok(())
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            zitadel_authority: String::new(),
            zitadel_client_id: String::new(),
            zitadel_audience: String::new(),
            jwks_cache_ttl_secs: default_jwks_cache_ttl(),
        }
    }
}

fn default_jwks_cache_ttl() -> u64 {
    3600
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_defaults() {
        let config = AuthConfig::default();
        assert_eq!(config.jwks_cache_ttl_secs, 3600);
    }

    #[test]
    fn test_jwks_cache_ttl_duration() {
        let config = AuthConfig {
            jwks_cache_ttl_secs: 7200,
            ..Default::default()
        };
        assert_eq!(config.jwks_cache_ttl(), Duration::from_secs(7200));
    }

    #[test]
    fn test_validation_missing_authority() {
        let config = AuthConfig::default();
        assert!(config.validate(&Environment::Development).is_err());
    }

    #[test]
    fn test_validation_missing_client_id() {
        let config = AuthConfig {
            zitadel_authority: "https://auth.example.com".to_string(),
            ..Default::default()
        };
        assert!(config.validate(&Environment::Development).is_err());
    }

    #[test]
    fn test_validation_production_requires_https() {
        let config = AuthConfig {
            zitadel_authority: "http://auth.example.com".to_string(),
            zitadel_client_id: "client-id".to_string(),
            zitadel_audience: "audience".to_string(),
            ..Default::default()
        };
        // Allowed in development
        assert!(config.validate(&Environment::Development).is_ok());
        // Rejected in production
        assert!(config.validate(&Environment::Production).is_err());
    }

    #[test]
    fn test_validation_valid_config() {
        let config = AuthConfig {
            zitadel_authority: "https://auth.example.com".to_string(),
            zitadel_client_id: "choice-sherpa".to_string(),
            zitadel_audience: "choice-sherpa-api".to_string(),
            ..Default::default()
        };
        assert!(config.validate(&Environment::Production).is_ok());
    }
}
