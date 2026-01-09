//! AI provider configuration

use serde::Deserialize;
use std::time::Duration;

use super::error::ValidationError;

/// AI provider configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AiConfig {
    /// OpenAI API key
    pub openai_api_key: Option<String>,

    /// Anthropic API key
    pub anthropic_api_key: Option<String>,

    /// Primary AI provider
    #[serde(default = "default_provider")]
    pub primary_provider: AiProvider,

    /// Fallback AI provider
    pub fallback_provider: Option<AiProvider>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Maximum retries on failure
    #[serde(default = "default_retries")]
    pub max_retries: u32,
}

/// AI provider type
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    OpenAI,
    #[default]
    Anthropic,
}

impl AiConfig {
    /// Get timeout as Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    /// Check if OpenAI is configured
    pub fn has_openai(&self) -> bool {
        self.openai_api_key.as_ref().is_some_and(|k| !k.is_empty())
    }

    /// Check if Anthropic is configured
    pub fn has_anthropic(&self) -> bool {
        self.anthropic_api_key.as_ref().is_some_and(|k| !k.is_empty())
    }

    /// Validate AI configuration
    pub fn validate(&self) -> Result<(), ValidationError> {
        // At least one provider must have an API key
        if !self.has_openai() && !self.has_anthropic() {
            return Err(ValidationError::NoAiProviderConfigured);
        }

        // Primary provider must have an API key
        match self.primary_provider {
            AiProvider::OpenAI if !self.has_openai() => {
                return Err(ValidationError::MissingRequired("OPENAI_API_KEY"));
            }
            AiProvider::Anthropic if !self.has_anthropic() => {
                return Err(ValidationError::MissingRequired("ANTHROPIC_API_KEY"));
            }
            _ => {}
        }

        Ok(())
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            openai_api_key: None,
            anthropic_api_key: None,
            primary_provider: default_provider(),
            fallback_provider: None,
            timeout_secs: default_timeout(),
            max_retries: default_retries(),
        }
    }
}

fn default_provider() -> AiProvider {
    AiProvider::Anthropic
}

fn default_timeout() -> u64 {
    120
}

fn default_retries() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_config_defaults() {
        let config = AiConfig::default();
        assert_eq!(config.primary_provider, AiProvider::Anthropic);
        assert_eq!(config.timeout_secs, 120);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_timeout_duration() {
        let config = AiConfig {
            timeout_secs: 60,
            ..Default::default()
        };
        assert_eq!(config.timeout(), Duration::from_secs(60));
    }

    #[test]
    fn test_has_provider_checks() {
        let config = AiConfig {
            openai_api_key: Some("sk-xxx".to_string()),
            anthropic_api_key: None,
            ..Default::default()
        };
        assert!(config.has_openai());
        assert!(!config.has_anthropic());
    }

    #[test]
    fn test_validation_no_provider() {
        let config = AiConfig::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_primary_missing_key() {
        let config = AiConfig {
            primary_provider: AiProvider::Anthropic,
            openai_api_key: Some("sk-xxx".to_string()),
            anthropic_api_key: None, // Missing key for primary
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_valid_config() {
        let config = AiConfig {
            primary_provider: AiProvider::Anthropic,
            anthropic_api_key: Some("sk-ant-xxx".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validation_with_fallback() {
        let config = AiConfig {
            primary_provider: AiProvider::Anthropic,
            anthropic_api_key: Some("sk-ant-xxx".to_string()),
            fallback_provider: Some(AiProvider::OpenAI),
            openai_api_key: Some("sk-xxx".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
}
