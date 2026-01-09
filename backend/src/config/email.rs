//! Email configuration

use serde::Deserialize;

use super::error::ValidationError;

/// Email configuration (Resend)
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
    /// Get formatted "From" header value
    pub fn from_header(&self) -> String {
        format!("{} <{}>", self.from_name, self.from_email)
    }

    /// Validate email configuration
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

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            resend_api_key: String::new(),
            from_email: default_from_email(),
            from_name: default_from_name(),
        }
    }
}

fn default_from_email() -> String {
    "noreply@choicesherpa.com".to_string()
}

fn default_from_name() -> String {
    "Choice Sherpa".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_config_defaults() {
        let config = EmailConfig::default();
        assert_eq!(config.from_email, "noreply@choicesherpa.com");
        assert_eq!(config.from_name, "Choice Sherpa");
    }

    #[test]
    fn test_from_header() {
        let config = EmailConfig {
            from_email: "support@example.com".to_string(),
            from_name: "Support Team".to_string(),
            ..Default::default()
        };
        assert_eq!(config.from_header(), "Support Team <support@example.com>");
    }

    #[test]
    fn test_validation_missing_api_key() {
        let config = EmailConfig::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_api_key_prefix() {
        let config = EmailConfig {
            resend_api_key: "sk_xxx".to_string(), // Wrong prefix
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_from_email() {
        let config = EmailConfig {
            resend_api_key: "re_xxx".to_string(),
            from_email: "invalid-email".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_valid_config() {
        let config = EmailConfig {
            resend_api_key: "re_abcd1234".to_string(),
            from_email: "noreply@choicesherpa.com".to_string(),
            from_name: "Choice Sherpa".to_string(),
        };
        assert!(config.validate().is_ok());
    }
}
