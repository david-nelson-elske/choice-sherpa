//! Feature flags configuration

use serde::Deserialize;

/// Feature flags for enabling/disabling functionality
#[derive(Debug, Clone, Deserialize)]
pub struct FeatureFlags {
    /// Enable WebSocket streaming for conversations
    #[serde(default)]
    pub enable_streaming: bool,

    /// Enable AI fallback provider on primary failure
    #[serde(default)]
    pub enable_ai_fallback: bool,

    /// Show detailed error messages (disable in production!)
    #[serde(default)]
    pub verbose_errors: bool,

    /// Enable request tracing (defaults to true)
    #[serde(default = "default_enable_tracing")]
    pub enable_tracing: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            enable_streaming: false,
            enable_ai_fallback: false,
            verbose_errors: false,
            enable_tracing: true,
        }
    }
}

fn default_enable_tracing() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_flags_defaults() {
        let flags = FeatureFlags::default();
        assert!(!flags.enable_streaming);
        assert!(!flags.enable_ai_fallback);
        assert!(!flags.verbose_errors);
        assert!(flags.enable_tracing);
    }

    #[test]
    fn test_feature_flags_deserialization() {
        let json = r#"{
            "enable_streaming": true,
            "enable_ai_fallback": true,
            "verbose_errors": false,
            "enable_tracing": true
        }"#;

        let flags: FeatureFlags = serde_json::from_str(json).unwrap();
        assert!(flags.enable_streaming);
        assert!(flags.enable_ai_fallback);
        assert!(!flags.verbose_errors);
        assert!(flags.enable_tracing);
    }
}
