//! Payment configuration

use serde::Deserialize;

use super::error::ValidationError;

/// Payment configuration (Stripe)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PaymentConfig {
    /// Stripe API key
    pub stripe_api_key: String,

    /// Stripe webhook signing secret
    pub stripe_webhook_secret: String,

    /// Stripe price ID for monthly plan
    pub stripe_monthly_price_id: Option<String>,

    /// Stripe price ID for annual plan
    pub stripe_annual_price_id: Option<String>,
}

impl PaymentConfig {
    /// Check if using Stripe test mode
    pub fn is_test_mode(&self) -> bool {
        self.stripe_api_key.starts_with("sk_test_")
    }

    /// Check if using Stripe live mode
    pub fn is_live_mode(&self) -> bool {
        self.stripe_api_key.starts_with("sk_live_")
    }

    /// Validate payment configuration
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_test_mode() {
        let config = PaymentConfig {
            stripe_api_key: "sk_test_xxx".to_string(),
            stripe_webhook_secret: "whsec_xxx".to_string(),
            ..Default::default()
        };
        assert!(config.is_test_mode());
        assert!(!config.is_live_mode());
    }

    #[test]
    fn test_is_live_mode() {
        let config = PaymentConfig {
            stripe_api_key: "sk_live_xxx".to_string(),
            stripe_webhook_secret: "whsec_xxx".to_string(),
            ..Default::default()
        };
        assert!(config.is_live_mode());
        assert!(!config.is_test_mode());
    }

    #[test]
    fn test_validation_missing_api_key() {
        let config = PaymentConfig::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_missing_webhook_secret() {
        let config = PaymentConfig {
            stripe_api_key: "sk_test_xxx".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_api_key_prefix() {
        let config = PaymentConfig {
            stripe_api_key: "pk_test_xxx".to_string(), // Wrong prefix
            stripe_webhook_secret: "whsec_xxx".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_webhook_secret_prefix() {
        let config = PaymentConfig {
            stripe_api_key: "sk_test_xxx".to_string(),
            stripe_webhook_secret: "secret_xxx".to_string(), // Wrong prefix
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_valid_config() {
        let config = PaymentConfig {
            stripe_api_key: "sk_test_abcd1234".to_string(),
            stripe_webhook_secret: "whsec_xyz789".to_string(),
            stripe_monthly_price_id: Some("price_monthly".to_string()),
            stripe_annual_price_id: Some("price_annual".to_string()),
        };
        assert!(config.validate().is_ok());
    }
}
