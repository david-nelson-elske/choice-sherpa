//! Zitadel OIDC adapter for JWT validation.
//!
//! This adapter implements the `SessionValidator` port using Zitadel as the
//! identity provider. It validates JWTs by:
//!
//! 1. Fetching JWKS from Zitadel's well-known endpoint
//! 2. Validating JWT signature against the public keys
//! 3. Validating issuer, audience, and expiry claims (OWASP A07)
//! 4. Mapping claims to domain `AuthenticatedUser` type
//!
//! # Security
//!
//! Per APPLICATION-SECURITY-STANDARD.md A07, this adapter validates:
//! - **Issuer (iss)**: Must match expected Zitadel URL
//! - **Audience (aud)**: Must contain our application identifier
//! - **Expiry (exp)**: Must be in the future
//!
//! # Example
//!
//! ```ignore
//! use std::sync::Arc;
//! use choice_sherpa::adapters::auth::ZitadelSessionValidator;
//! use choice_sherpa::ports::SessionValidator;
//!
//! let config = ZitadelConfig {
//!     issuer_url: "https://auth.choicesherpa.com".to_string(),
//!     audience: "choice-sherpa-api".to_string(),
//! };
//!
//! let validator = ZitadelSessionValidator::new(config).await?;
//! let user = validator.validate("eyJ...").await?;
//! ```

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use jsonwebtoken::{
    decode, decode_header, jwk::JwkSet, Algorithm, DecodingKey, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::domain::foundation::{AuthError, AuthenticatedUser, UserId};
use crate::ports::SessionValidator;

/// Configuration for the Zitadel OIDC adapter.
#[derive(Debug, Clone)]
pub struct ZitadelConfig {
    /// The issuer URL (e.g., "https://auth.choicesherpa.com")
    /// Used for JWKS discovery and JWT issuer validation.
    pub issuer_url: String,

    /// Expected audience claim in JWTs.
    /// Tokens must contain this audience to be accepted.
    pub audience: String,

    /// Optional: How long to cache JWKS before refetching.
    /// Defaults to 1 hour if not specified.
    pub jwks_cache_duration: Option<Duration>,
}

impl ZitadelConfig {
    /// Create a new configuration with required fields.
    pub fn new(issuer_url: impl Into<String>, audience: impl Into<String>) -> Self {
        Self {
            issuer_url: issuer_url.into(),
            audience: audience.into(),
            jwks_cache_duration: None,
        }
    }

    /// Set custom JWKS cache duration.
    pub fn with_cache_duration(mut self, duration: Duration) -> Self {
        self.jwks_cache_duration = Some(duration);
        self
    }

    /// Get the JWKS URL for this issuer.
    fn jwks_url(&self) -> String {
        format!("{}/.well-known/jwks.json", self.issuer_url.trim_end_matches('/'))
    }
}

/// JWT claims structure for Zitadel tokens.
#[derive(Debug, Serialize, Deserialize)]
struct ZitadelClaims {
    /// Subject - the user ID
    sub: String,

    /// Issuer URL
    iss: String,

    /// Audience - array or single string
    #[serde(default)]
    aud: Audience,

    /// Expiry timestamp (Unix epoch seconds)
    exp: i64,

    /// Issued at timestamp
    #[serde(default)]
    iat: Option<i64>,

    /// User's email address
    #[serde(default)]
    email: Option<String>,

    /// Whether email is verified
    #[serde(default)]
    email_verified: Option<bool>,

    /// User's display name
    #[serde(default)]
    name: Option<String>,

    /// User's preferred username
    #[serde(default)]
    preferred_username: Option<String>,
}

/// Audience can be a single string or array of strings in JWTs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(untagged)]
enum Audience {
    #[default]
    None,
    Single(String),
    Multiple(Vec<String>),
}

impl Audience {
    fn contains(&self, expected: &str) -> bool {
        match self {
            Audience::None => false,
            Audience::Single(s) => s == expected,
            Audience::Multiple(v) => v.iter().any(|s| s == expected),
        }
    }
}

/// Cached JWKS with expiry tracking.
struct JwksCache {
    jwks: JwkSet,
    fetched_at: Instant,
    cache_duration: Duration,
}

impl JwksCache {
    fn new(jwks: JwkSet, cache_duration: Duration) -> Self {
        Self {
            jwks,
            fetched_at: Instant::now(),
            cache_duration,
        }
    }

    fn is_expired(&self) -> bool {
        self.fetched_at.elapsed() > self.cache_duration
    }
}

/// Zitadel OIDC session validator.
///
/// Validates JWTs against Zitadel's JWKS and extracts user information.
/// This is the production implementation of `SessionValidator`.
pub struct ZitadelSessionValidator {
    config: ZitadelConfig,
    http_client: reqwest::Client,
    jwks_cache: Arc<RwLock<Option<JwksCache>>>,
}

impl ZitadelSessionValidator {
    /// Create a new Zitadel validator.
    ///
    /// This does NOT fetch JWKS immediately - keys are fetched lazily on first
    /// validation to avoid blocking during startup.
    pub fn new(config: ZitadelConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
            jwks_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Fetch JWKS from Zitadel.
    async fn fetch_jwks(&self) -> Result<JwkSet, AuthError> {
        let url = self.config.jwks_url();

        tracing::debug!("Fetching JWKS from {}", url);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch JWKS: {}", e);
                AuthError::ServiceUnavailable(format!("Failed to fetch JWKS: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::error!("JWKS endpoint returned {}", status);
            return Err(AuthError::ServiceUnavailable(format!(
                "JWKS endpoint returned {}",
                status
            )));
        }

        let jwks: JwkSet = response.json().await.map_err(|e| {
            tracing::error!("Failed to parse JWKS: {}", e);
            AuthError::ServiceUnavailable(format!("Failed to parse JWKS: {}", e))
        })?;

        tracing::debug!("Fetched {} keys from JWKS", jwks.keys.len());

        Ok(jwks)
    }

    /// Get JWKS, using cache if available and not expired.
    async fn get_jwks(&self) -> Result<JwkSet, AuthError> {
        // Check cache first
        {
            let cache = self.jwks_cache.read().await;
            if let Some(ref cached) = *cache {
                if !cached.is_expired() {
                    return Ok(cached.jwks.clone());
                }
            }
        }

        // Cache miss or expired - fetch new JWKS
        let jwks = self.fetch_jwks().await?;

        // Update cache
        {
            let mut cache = self.jwks_cache.write().await;
            let duration = self
                .config
                .jwks_cache_duration
                .unwrap_or(Duration::from_secs(3600)); // Default 1 hour
            *cache = Some(JwksCache::new(jwks.clone(), duration));
        }

        Ok(jwks)
    }

    /// Find the decoding key for a JWT.
    fn find_decoding_key<'a>(
        &self,
        header: &jsonwebtoken::Header,
        jwks: &'a JwkSet,
    ) -> Result<(DecodingKey, Algorithm), AuthError> {
        // Get the key ID from the JWT header
        let kid = header.kid.as_ref().ok_or_else(|| {
            tracing::warn!("JWT missing 'kid' header");
            AuthError::InvalidToken
        })?;

        // Find matching key in JWKS
        let jwk = jwks.find(kid).ok_or_else(|| {
            tracing::warn!("No matching key found for kid: {}", kid);
            AuthError::InvalidToken
        })?;

        // Determine algorithm
        let algorithm = match jwk.common.key_algorithm {
            Some(jsonwebtoken::jwk::KeyAlgorithm::RS256) => Algorithm::RS256,
            Some(jsonwebtoken::jwk::KeyAlgorithm::RS384) => Algorithm::RS384,
            Some(jsonwebtoken::jwk::KeyAlgorithm::RS512) => Algorithm::RS512,
            Some(jsonwebtoken::jwk::KeyAlgorithm::ES256) => Algorithm::ES256,
            Some(jsonwebtoken::jwk::KeyAlgorithm::ES384) => Algorithm::ES384,
            Some(other) => {
                tracing::warn!("Unsupported algorithm: {:?}", other);
                return Err(AuthError::InvalidToken);
            }
            None => {
                // Default to RS256 if not specified (common for OIDC)
                Algorithm::RS256
            }
        };

        // Create decoding key
        let decoding_key = DecodingKey::from_jwk(jwk).map_err(|e| {
            tracing::warn!("Failed to create decoding key: {}", e);
            AuthError::InvalidToken
        })?;

        Ok((decoding_key, algorithm))
    }

    /// Validate a JWT and extract claims.
    fn validate_token(
        &self,
        token: &str,
        decoding_key: &DecodingKey,
        algorithm: Algorithm,
    ) -> Result<TokenData<ZitadelClaims>, AuthError> {
        let mut validation = Validation::new(algorithm);

        // SECURITY (A07): Validate issuer
        validation.set_issuer(&[&self.config.issuer_url]);

        // SECURITY (A07): Validate audience
        validation.set_audience(&[&self.config.audience]);

        // SECURITY (A07): Validate expiry (enabled by default)
        validation.validate_exp = true;

        // Require these claims to be present
        validation.set_required_spec_claims(&["exp", "iss", "sub"]);

        decode::<ZitadelClaims>(token, decoding_key, &validation).map_err(|e| {
            use jsonwebtoken::errors::ErrorKind;
            match e.kind() {
                ErrorKind::ExpiredSignature => {
                    tracing::debug!("Token expired");
                    AuthError::TokenExpired
                }
                ErrorKind::InvalidIssuer => {
                    tracing::warn!("Invalid issuer in token");
                    AuthError::InvalidToken
                }
                ErrorKind::InvalidAudience => {
                    tracing::warn!("Invalid audience in token");
                    AuthError::InvalidToken
                }
                _ => {
                    tracing::warn!("Token validation failed: {}", e);
                    AuthError::InvalidToken
                }
            }
        })
    }
}

#[async_trait]
impl SessionValidator for ZitadelSessionValidator {
    async fn validate(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
        // Decode header to get key ID
        let header = decode_header(token).map_err(|e| {
            tracing::debug!("Failed to decode JWT header: {}", e);
            AuthError::InvalidToken
        })?;

        // Get JWKS (cached or fresh)
        let jwks = self.get_jwks().await?;

        // Find the matching key
        let (decoding_key, algorithm) = self.find_decoding_key(&header, &jwks)?;

        // Validate token and extract claims
        let token_data = self.validate_token(token, &decoding_key, algorithm)?;
        let claims = token_data.claims;

        // SECURITY: Double-check issuer (defense in depth)
        if claims.iss != self.config.issuer_url {
            tracing::warn!(
                "Issuer mismatch after validation: expected '{}', got '{}'",
                self.config.issuer_url,
                claims.iss
            );
            return Err(AuthError::InvalidToken);
        }

        // SECURITY: Double-check audience (defense in depth)
        if !claims.aud.contains(&self.config.audience) {
            tracing::warn!(
                "Audience mismatch after validation: expected '{}', got '{:?}'",
                self.config.audience,
                claims.aud
            );
            return Err(AuthError::InvalidToken);
        }

        // Extract email - required for our domain
        let email = claims.email.ok_or_else(|| {
            tracing::warn!("Token missing email claim");
            AuthError::InvalidToken
        })?;

        // Create user ID from subject
        let user_id = UserId::new(&claims.sub).map_err(|_| {
            tracing::warn!("Invalid user ID in token: {}", claims.sub);
            AuthError::InvalidToken
        })?;

        Ok(AuthenticatedUser::new(
            user_id,
            email,
            claims.name.or(claims.preferred_username),
            claims.email_verified.unwrap_or(false),
        ))
    }
}

impl std::fmt::Debug for ZitadelSessionValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZitadelSessionValidator")
            .field("issuer_url", &self.config.issuer_url)
            .field("audience", &self.config.audience)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ════════════════════════════════════════════════════════════════════════════
    // Configuration Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn config_builds_correct_jwks_url() {
        let config = ZitadelConfig::new("https://auth.example.com", "my-api");
        assert_eq!(
            config.jwks_url(),
            "https://auth.example.com/.well-known/jwks.json"
        );
    }

    #[test]
    fn config_handles_trailing_slash() {
        let config = ZitadelConfig::new("https://auth.example.com/", "my-api");
        assert_eq!(
            config.jwks_url(),
            "https://auth.example.com/.well-known/jwks.json"
        );
    }

    #[test]
    fn config_with_custom_cache_duration() {
        let config = ZitadelConfig::new("https://auth.example.com", "my-api")
            .with_cache_duration(Duration::from_secs(300));
        assert_eq!(config.jwks_cache_duration, Some(Duration::from_secs(300)));
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Audience Parsing Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn audience_single_string_contains() {
        let aud = Audience::Single("my-api".to_string());
        assert!(aud.contains("my-api"));
        assert!(!aud.contains("other-api"));
    }

    #[test]
    fn audience_multiple_contains() {
        let aud = Audience::Multiple(vec!["api-1".to_string(), "api-2".to_string()]);
        assert!(aud.contains("api-1"));
        assert!(aud.contains("api-2"));
        assert!(!aud.contains("api-3"));
    }

    #[test]
    fn audience_none_contains_nothing() {
        let aud = Audience::None;
        assert!(!aud.contains("anything"));
    }

    // ════════════════════════════════════════════════════════════════════════════
    // JWKS Cache Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn jwks_cache_not_expired_initially() {
        let jwks = JwkSet { keys: vec![] };
        let cache = JwksCache::new(jwks, Duration::from_secs(3600));
        assert!(!cache.is_expired());
    }

    #[test]
    fn jwks_cache_expires_after_duration() {
        let jwks = JwkSet { keys: vec![] };
        let cache = JwksCache::new(jwks, Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(10));
        assert!(cache.is_expired());
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Type Safety Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn zitadel_validator_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ZitadelSessionValidator>();
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Integration Tests (require network, marked ignore)
    // ════════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    #[ignore = "Requires live Zitadel instance"]
    async fn integration_test_fetch_jwks() {
        // This test requires a running Zitadel instance
        // Set ZITADEL_ISSUER_URL environment variable to test
        let issuer = std::env::var("ZITADEL_ISSUER_URL")
            .unwrap_or_else(|_| "https://auth.choicesherpa.com".to_string());

        let config = ZitadelConfig::new(&issuer, "test-audience");
        let validator = ZitadelSessionValidator::new(config);

        let result = validator.fetch_jwks().await;
        assert!(result.is_ok(), "Failed to fetch JWKS: {:?}", result.err());

        let jwks = result.unwrap();
        assert!(!jwks.keys.is_empty(), "JWKS should contain at least one key");
    }
}
