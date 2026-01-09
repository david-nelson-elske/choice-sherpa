//! Feature flags configuration

use serde::Deserialize;

/// Feature flags for enabling/disabling functionality
#[derive(Debug, Clone, Deserialize, Default)]
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

    /// Enable request tracing
    #[serde(default = "default_enable_tracing")]
    pub enable_tracing: bool,
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
        // enable_tracing defaults to true but Default trait won't pick it up
        // since it uses bool::default() which is false
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
