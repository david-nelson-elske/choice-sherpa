//! Tier-based feature limits configuration.
//!
//! Defines what features and limits are available for each membership tier.
//!
//! # Tier Matrix
//!
//! | Tier | Sessions | Cycles | History | AI/Day | Model | DQ | Export |
//! |------|----------|--------|---------|--------|-------|-----|--------|
//! | Free | 3 | 2 | 90d | 50 | Std | No | No |
//! | Monthly | 10 | 5 | 365d | 200 | Std | Yes | Yes |
//! | Annual | ∞ | ∞ | ∞ | ∞ | Adv | Yes | Yes |

use super::MembershipTier;
use serde::{Deserialize, Serialize};

/// AI model tier levels.
///
/// Determines which AI model is used for conversations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiModelTier {
    /// GPT-4o-mini or equivalent - fast, cost-effective
    Standard,
    /// GPT-4o or equivalent - highest quality
    Advanced,
}

impl AiModelTier {
    /// Get the AI provider model identifier.
    pub fn model_id(&self) -> &'static str {
        match self {
            AiModelTier::Standard => "gpt-4o-mini",
            AiModelTier::Advanced => "gpt-4o",
        }
    }

    /// Get a human-readable name for the model tier.
    pub fn display_name(&self) -> &'static str {
        match self {
            AiModelTier::Standard => "Standard AI",
            AiModelTier::Advanced => "Advanced AI",
        }
    }
}

impl Default for AiModelTier {
    fn default() -> Self {
        AiModelTier::Standard
    }
}

/// Complete feature limits for a membership tier.
///
/// Defines the boundaries of what a user can do based on their subscription.
/// Uses `Option<u32>` where `None` means unlimited.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TierLimits {
    /// The tier these limits apply to.
    pub tier: MembershipTier,

    // ─── Session & Cycle Limits ─────────────────────────────────────

    /// Maximum active sessions. None = unlimited.
    pub max_active_sessions: Option<u32>,
    /// Maximum cycles per session. None = unlimited.
    pub max_cycles_per_session: Option<u32>,
    /// Maximum archived sessions. None = unlimited.
    pub max_archived_sessions: Option<u32>,
    /// Session history retention in days. None = forever.
    pub session_history_days: Option<u32>,

    // ─── AI Features ────────────────────────────────────────────────

    /// Whether AI conversations are enabled.
    pub ai_enabled: bool,
    /// Maximum AI messages per day. None = unlimited.
    pub ai_messages_per_day: Option<u32>,
    /// AI model quality tier.
    pub ai_model_tier: AiModelTier,

    // ─── Component Access ───────────────────────────────────────────

    /// Whether the Decision Quality component is accessible.
    pub dq_component_enabled: bool,

    // ─── Analysis Features ──────────────────────────────────────────

    /// Whether full tradeoff analysis is available (vs. basic).
    pub full_tradeoff_analysis: bool,
    /// Whether DQ scoring is enabled.
    pub dq_scoring_enabled: bool,
    /// Whether improvement suggestions are shown.
    pub improvement_suggestions_enabled: bool,

    // ─── Export & Sharing ───────────────────────────────────────────

    /// Whether PDF/CSV export is enabled.
    pub pdf_export_enabled: bool,
    /// Whether share link generation is enabled.
    pub share_link_enabled: bool,
    /// Whether API access is enabled.
    pub api_access: bool,
}

impl TierLimits {
    /// Get the limits for a specific tier.
    ///
    /// # Tier Configuration
    ///
    /// | Tier | Active Sessions | Cycles/Session | Archived | History |
    /// |------|-----------------|----------------|----------|---------|
    /// | Free | 3 | 2 | 10 | 90 days |
    /// | Monthly | 10 | 5 | 50 | 1 year |
    /// | Annual | Unlimited | Unlimited | Unlimited | Forever |
    ///
    /// | Tier | AI Messages/Day | AI Model | DQ Component |
    /// |------|-----------------|----------|--------------|
    /// | Free | 50 | Standard | No |
    /// | Monthly | 200 | Standard | Yes |
    /// | Annual | Unlimited | Advanced | Yes |
    pub fn for_tier(tier: MembershipTier) -> Self {
        match tier {
            MembershipTier::Free => Self::free(),
            MembershipTier::Monthly => Self::premium(),
            MembershipTier::Annual => Self::pro(),
        }
    }

    /// Returns limits for the Free tier.
    pub fn free() -> Self {
        Self {
            tier: MembershipTier::Free,

            // Session & Cycle Limits
            max_active_sessions: Some(3),
            max_cycles_per_session: Some(2),
            max_archived_sessions: Some(10),
            session_history_days: Some(90),

            // AI Features
            ai_enabled: true,
            ai_messages_per_day: Some(50),
            ai_model_tier: AiModelTier::Standard,

            // Component Access
            dq_component_enabled: false,

            // Analysis Features
            full_tradeoff_analysis: false,
            dq_scoring_enabled: false,
            improvement_suggestions_enabled: false,

            // Export & Sharing
            pdf_export_enabled: false,
            share_link_enabled: false,
            api_access: false,
        }
    }

    /// Returns limits for the Monthly (Premium) tier.
    pub fn premium() -> Self {
        Self {
            tier: MembershipTier::Monthly,

            // Session & Cycle Limits
            max_active_sessions: Some(10),
            max_cycles_per_session: Some(5),
            max_archived_sessions: Some(50),
            session_history_days: Some(365),

            // AI Features
            ai_enabled: true,
            ai_messages_per_day: Some(200),
            ai_model_tier: AiModelTier::Standard,

            // Component Access
            dq_component_enabled: true,

            // Analysis Features
            full_tradeoff_analysis: true,
            dq_scoring_enabled: true,
            improvement_suggestions_enabled: true,

            // Export & Sharing
            pdf_export_enabled: true,
            share_link_enabled: true,
            api_access: false,
        }
    }

    /// Returns limits for the Annual (Pro) tier.
    pub fn pro() -> Self {
        Self {
            tier: MembershipTier::Annual,

            // Session & Cycle Limits
            max_active_sessions: None, // Unlimited
            max_cycles_per_session: None, // Unlimited
            max_archived_sessions: None, // Unlimited
            session_history_days: None, // Forever

            // AI Features
            ai_enabled: true,
            ai_messages_per_day: None, // Unlimited
            ai_model_tier: AiModelTier::Advanced,

            // Component Access
            dq_component_enabled: true,

            // Analysis Features
            full_tradeoff_analysis: true,
            dq_scoring_enabled: true,
            improvement_suggestions_enabled: true,

            // Export & Sharing
            pdf_export_enabled: true,
            share_link_enabled: true,
            api_access: true,
        }
    }

    /// Returns limits for users without a valid membership.
    ///
    /// **Security Note:** This is the fail-secure default. Users without
    /// membership get zero access - there is no implicit free tier.
    pub fn no_membership() -> Self {
        Self {
            tier: MembershipTier::Free, // Nominal tier for serialization

            // All access denied
            max_active_sessions: Some(0),
            max_cycles_per_session: Some(0),
            max_archived_sessions: Some(0),
            session_history_days: Some(0),

            ai_enabled: false,
            ai_messages_per_day: Some(0),
            ai_model_tier: AiModelTier::Standard,

            dq_component_enabled: false,

            full_tradeoff_analysis: false,
            dq_scoring_enabled: false,
            improvement_suggestions_enabled: false,

            pdf_export_enabled: false,
            share_link_enabled: false,
            api_access: false,
        }
    }

    // ─── Limit Checking Methods ─────────────────────────────────────

    /// Check if user can create a new session.
    ///
    /// Returns `true` if under the session limit or limit is unlimited.
    pub fn can_create_session(&self, current_active: u32) -> bool {
        match self.max_active_sessions {
            None => true,
            Some(max) => current_active < max,
        }
    }

    /// Check if user can create a new cycle in a session.
    ///
    /// Returns `true` if under the cycle limit or limit is unlimited.
    pub fn can_create_cycle(&self, current_cycles: u32) -> bool {
        match self.max_cycles_per_session {
            None => true,
            Some(max) => current_cycles < max,
        }
    }

    /// Check if user can archive another session.
    ///
    /// Returns `true` if under the archived limit or limit is unlimited.
    pub fn can_archive_session(&self, current_archived: u32) -> bool {
        match self.max_archived_sessions {
            None => true,
            Some(max) => current_archived < max,
        }
    }

    /// Check if user can send an AI message.
    ///
    /// Returns `true` if AI is enabled and under daily message limit.
    pub fn can_send_ai_message(&self, messages_today: u32) -> bool {
        if !self.ai_enabled {
            return false;
        }
        match self.ai_messages_per_day {
            None => true,
            Some(max) => messages_today < max,
        }
    }

    /// Check if user can access the Decision Quality component.
    pub fn can_access_dq(&self) -> bool {
        self.dq_component_enabled
    }

    /// Check if user can export to PDF.
    pub fn can_export_pdf(&self) -> bool {
        self.pdf_export_enabled
    }

    /// Check if user can create share links.
    pub fn can_share(&self) -> bool {
        self.share_link_enabled
    }

    /// Get the AI model identifier for this tier.
    pub fn ai_model(&self) -> &'static str {
        self.ai_model_tier.model_id()
    }

    /// Calculate remaining AI messages for today.
    ///
    /// Returns `None` if unlimited.
    pub fn ai_messages_remaining(&self, messages_today: u32) -> Option<u32> {
        self.ai_messages_per_day.map(|max| max.saturating_sub(messages_today))
    }

    // ─── Legacy Compatibility ───────────────────────────────────────

    /// Check if the session limit has been reached.
    ///
    /// **Deprecated:** Use `can_create_session()` instead.
    /// Returns `true` if at or over limit.
    #[deprecated(since = "0.2.0", note = "Use can_create_session() instead")]
    pub fn session_limit_reached(&self, current_sessions: u32) -> bool {
        !self.can_create_session(current_sessions)
    }

    /// Check if the cycle limit has been reached for a session.
    ///
    /// **Deprecated:** Use `can_create_cycle()` instead.
    /// Returns `true` if at or over limit.
    #[deprecated(since = "0.2.0", note = "Use can_create_cycle() instead")]
    pub fn cycle_limit_reached(&self, current_cycles: u32) -> bool {
        !self.can_create_cycle(current_cycles)
    }

    /// Legacy accessor for max sessions.
    ///
    /// **Deprecated:** Use `max_active_sessions` instead.
    #[deprecated(since = "0.2.0", note = "Use max_active_sessions instead")]
    pub fn max_sessions(&self) -> Option<u32> {
        self.max_active_sessions
    }

    /// Legacy accessor for export enabled.
    ///
    /// **Deprecated:** Use `pdf_export_enabled` instead.
    #[deprecated(since = "0.2.0", note = "Use pdf_export_enabled instead")]
    pub fn export_enabled(&self) -> bool {
        self.pdf_export_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── AiModelTier Tests ─────────────────────────────────────────

    #[test]
    fn standard_model_id_is_gpt4o_mini() {
        assert_eq!(AiModelTier::Standard.model_id(), "gpt-4o-mini");
    }

    #[test]
    fn advanced_model_id_is_gpt4o() {
        assert_eq!(AiModelTier::Advanced.model_id(), "gpt-4o");
    }

    #[test]
    fn ai_model_tier_default_is_standard() {
        assert_eq!(AiModelTier::default(), AiModelTier::Standard);
    }

    #[test]
    fn ai_model_tier_serializes_to_snake_case() {
        let json = serde_json::to_string(&AiModelTier::Standard).unwrap();
        assert_eq!(json, "\"standard\"");

        let json = serde_json::to_string(&AiModelTier::Advanced).unwrap();
        assert_eq!(json, "\"advanced\"");
    }

    #[test]
    fn ai_model_tier_deserializes_from_snake_case() {
        let tier: AiModelTier = serde_json::from_str("\"standard\"").unwrap();
        assert_eq!(tier, AiModelTier::Standard);

        let tier: AiModelTier = serde_json::from_str("\"advanced\"").unwrap();
        assert_eq!(tier, AiModelTier::Advanced);
    }

    // ─── Free Tier Tests ───────────────────────────────────────────

    #[test]
    fn free_tier_has_3_active_sessions() {
        let limits = TierLimits::free();
        assert_eq!(limits.max_active_sessions, Some(3));
    }

    #[test]
    fn free_tier_has_2_cycles_per_session() {
        let limits = TierLimits::free();
        assert_eq!(limits.max_cycles_per_session, Some(2));
    }

    #[test]
    fn free_tier_has_10_archived_sessions() {
        let limits = TierLimits::free();
        assert_eq!(limits.max_archived_sessions, Some(10));
    }

    #[test]
    fn free_tier_has_90_day_history() {
        let limits = TierLimits::free();
        assert_eq!(limits.session_history_days, Some(90));
    }

    #[test]
    fn free_tier_has_50_ai_messages_per_day() {
        let limits = TierLimits::free();
        assert_eq!(limits.ai_messages_per_day, Some(50));
    }

    #[test]
    fn free_tier_uses_standard_ai() {
        let limits = TierLimits::free();
        assert_eq!(limits.ai_model_tier, AiModelTier::Standard);
    }

    #[test]
    fn free_tier_has_no_dq_access() {
        let limits = TierLimits::free();
        assert!(!limits.dq_component_enabled);
        assert!(!limits.can_access_dq());
    }

    #[test]
    fn free_tier_has_no_export() {
        let limits = TierLimits::free();
        assert!(!limits.pdf_export_enabled);
        assert!(!limits.can_export_pdf());
    }

    #[test]
    fn free_tier_has_no_share() {
        let limits = TierLimits::free();
        assert!(!limits.share_link_enabled);
        assert!(!limits.can_share());
    }

    #[test]
    fn free_tier_has_no_api_access() {
        let limits = TierLimits::free();
        assert!(!limits.api_access);
    }

    // ─── Premium (Monthly) Tier Tests ──────────────────────────────

    #[test]
    fn premium_tier_has_10_active_sessions() {
        let limits = TierLimits::premium();
        assert_eq!(limits.max_active_sessions, Some(10));
    }

    #[test]
    fn premium_tier_has_5_cycles_per_session() {
        let limits = TierLimits::premium();
        assert_eq!(limits.max_cycles_per_session, Some(5));
    }

    #[test]
    fn premium_tier_has_50_archived_sessions() {
        let limits = TierLimits::premium();
        assert_eq!(limits.max_archived_sessions, Some(50));
    }

    #[test]
    fn premium_tier_has_365_day_history() {
        let limits = TierLimits::premium();
        assert_eq!(limits.session_history_days, Some(365));
    }

    #[test]
    fn premium_tier_has_200_ai_messages_per_day() {
        let limits = TierLimits::premium();
        assert_eq!(limits.ai_messages_per_day, Some(200));
    }

    #[test]
    fn premium_tier_uses_standard_ai() {
        let limits = TierLimits::premium();
        assert_eq!(limits.ai_model_tier, AiModelTier::Standard);
    }

    #[test]
    fn premium_tier_has_dq_access() {
        let limits = TierLimits::premium();
        assert!(limits.dq_component_enabled);
        assert!(limits.can_access_dq());
    }

    #[test]
    fn premium_tier_has_export() {
        let limits = TierLimits::premium();
        assert!(limits.pdf_export_enabled);
        assert!(limits.can_export_pdf());
    }

    #[test]
    fn premium_tier_has_share() {
        let limits = TierLimits::premium();
        assert!(limits.share_link_enabled);
        assert!(limits.can_share());
    }

    #[test]
    fn premium_tier_has_no_api_access() {
        let limits = TierLimits::premium();
        assert!(!limits.api_access);
    }

    // ─── Pro (Annual) Tier Tests ───────────────────────────────────

    #[test]
    fn pro_tier_has_unlimited_active_sessions() {
        let limits = TierLimits::pro();
        assert_eq!(limits.max_active_sessions, None);
    }

    #[test]
    fn pro_tier_has_unlimited_cycles() {
        let limits = TierLimits::pro();
        assert_eq!(limits.max_cycles_per_session, None);
    }

    #[test]
    fn pro_tier_has_unlimited_archived_sessions() {
        let limits = TierLimits::pro();
        assert_eq!(limits.max_archived_sessions, None);
    }

    #[test]
    fn pro_tier_has_unlimited_history() {
        let limits = TierLimits::pro();
        assert_eq!(limits.session_history_days, None);
    }

    #[test]
    fn pro_tier_has_unlimited_ai_messages() {
        let limits = TierLimits::pro();
        assert_eq!(limits.ai_messages_per_day, None);
    }

    #[test]
    fn pro_tier_uses_advanced_ai() {
        let limits = TierLimits::pro();
        assert_eq!(limits.ai_model_tier, AiModelTier::Advanced);
    }

    #[test]
    fn pro_tier_has_api_access() {
        let limits = TierLimits::pro();
        assert!(limits.api_access);
    }

    // ─── No Membership Tests ───────────────────────────────────────

    #[test]
    fn no_membership_has_zero_sessions() {
        let limits = TierLimits::no_membership();
        assert_eq!(limits.max_active_sessions, Some(0));
        assert!(!limits.can_create_session(0));
    }

    #[test]
    fn no_membership_has_zero_cycles() {
        let limits = TierLimits::no_membership();
        assert_eq!(limits.max_cycles_per_session, Some(0));
        assert!(!limits.can_create_cycle(0));
    }

    #[test]
    fn no_membership_has_ai_disabled() {
        let limits = TierLimits::no_membership();
        assert!(!limits.ai_enabled);
        assert!(!limits.can_send_ai_message(0));
    }

    #[test]
    fn no_membership_denies_all_features() {
        let limits = TierLimits::no_membership();
        assert!(!limits.can_access_dq());
        assert!(!limits.can_export_pdf());
        assert!(!limits.can_share());
        assert!(!limits.api_access);
    }

    // ─── can_create_session Tests ──────────────────────────────────

    #[test]
    fn can_create_session_when_under_limit() {
        let limits = TierLimits::free();
        assert!(limits.can_create_session(0));
        assert!(limits.can_create_session(1));
        assert!(limits.can_create_session(2));
    }

    #[test]
    fn cannot_create_session_when_at_limit() {
        let limits = TierLimits::free();
        assert!(!limits.can_create_session(3));
    }

    #[test]
    fn cannot_create_session_when_over_limit() {
        let limits = TierLimits::free();
        assert!(!limits.can_create_session(5));
    }

    #[test]
    fn can_always_create_session_when_unlimited() {
        let limits = TierLimits::pro();
        assert!(limits.can_create_session(0));
        assert!(limits.can_create_session(100));
        assert!(limits.can_create_session(1000));
    }

    // ─── can_create_cycle Tests ────────────────────────────────────

    #[test]
    fn can_create_cycle_when_under_limit() {
        let limits = TierLimits::free();
        assert!(limits.can_create_cycle(0));
        assert!(limits.can_create_cycle(1));
    }

    #[test]
    fn cannot_create_cycle_when_at_limit() {
        let limits = TierLimits::free();
        assert!(!limits.can_create_cycle(2));
    }

    #[test]
    fn can_always_create_cycle_when_unlimited() {
        let limits = TierLimits::pro();
        assert!(limits.can_create_cycle(1000));
    }

    // ─── can_archive_session Tests ─────────────────────────────────

    #[test]
    fn can_archive_session_when_under_limit() {
        let limits = TierLimits::free();
        assert!(limits.can_archive_session(9));
    }

    #[test]
    fn cannot_archive_session_when_at_limit() {
        let limits = TierLimits::free();
        assert!(!limits.can_archive_session(10));
    }

    #[test]
    fn can_always_archive_session_when_unlimited() {
        let limits = TierLimits::pro();
        assert!(limits.can_archive_session(1000));
    }

    // ─── can_send_ai_message Tests ─────────────────────────────────

    #[test]
    fn can_send_ai_message_when_under_limit() {
        let limits = TierLimits::free();
        assert!(limits.can_send_ai_message(0));
        assert!(limits.can_send_ai_message(49));
    }

    #[test]
    fn cannot_send_ai_message_when_at_limit() {
        let limits = TierLimits::free();
        assert!(!limits.can_send_ai_message(50));
    }

    #[test]
    fn cannot_send_ai_message_when_disabled() {
        let limits = TierLimits::no_membership();
        assert!(!limits.can_send_ai_message(0));
    }

    #[test]
    fn can_always_send_ai_message_when_unlimited() {
        let limits = TierLimits::pro();
        assert!(limits.can_send_ai_message(10000));
    }

    // ─── ai_messages_remaining Tests ───────────────────────────────

    #[test]
    fn ai_messages_remaining_calculates_correctly() {
        let limits = TierLimits::free();
        assert_eq!(limits.ai_messages_remaining(0), Some(50));
        assert_eq!(limits.ai_messages_remaining(30), Some(20));
        assert_eq!(limits.ai_messages_remaining(50), Some(0));
    }

    #[test]
    fn ai_messages_remaining_saturates_at_zero() {
        let limits = TierLimits::free();
        assert_eq!(limits.ai_messages_remaining(100), Some(0));
    }

    #[test]
    fn ai_messages_remaining_none_when_unlimited() {
        let limits = TierLimits::pro();
        assert_eq!(limits.ai_messages_remaining(1000), None);
    }

    // ─── ai_model Tests ────────────────────────────────────────────

    #[test]
    fn free_tier_ai_model_is_gpt4o_mini() {
        let limits = TierLimits::free();
        assert_eq!(limits.ai_model(), "gpt-4o-mini");
    }

    #[test]
    fn premium_tier_ai_model_is_gpt4o_mini() {
        let limits = TierLimits::premium();
        assert_eq!(limits.ai_model(), "gpt-4o-mini");
    }

    #[test]
    fn pro_tier_ai_model_is_gpt4o() {
        let limits = TierLimits::pro();
        assert_eq!(limits.ai_model(), "gpt-4o");
    }

    // ─── for_tier Tests ────────────────────────────────────────────

    #[test]
    fn for_tier_returns_correct_limits() {
        assert_eq!(TierLimits::for_tier(MembershipTier::Free), TierLimits::free());
        assert_eq!(TierLimits::for_tier(MembershipTier::Monthly), TierLimits::premium());
        assert_eq!(TierLimits::for_tier(MembershipTier::Annual), TierLimits::pro());
    }

    // ─── Serialization Tests ───────────────────────────────────────

    #[test]
    fn tier_limits_serializes_to_json() {
        let limits = TierLimits::free();
        let json = serde_json::to_string(&limits).unwrap();
        assert!(json.contains("\"max_active_sessions\":3"));
        assert!(json.contains("\"ai_model_tier\":\"standard\""));
    }

    #[test]
    fn tier_limits_deserializes_from_json() {
        let json = r#"{
            "tier": "free",
            "max_active_sessions": 3,
            "max_cycles_per_session": 2,
            "max_archived_sessions": 10,
            "session_history_days": 90,
            "ai_enabled": true,
            "ai_messages_per_day": 50,
            "ai_model_tier": "standard",
            "dq_component_enabled": false,
            "full_tradeoff_analysis": false,
            "dq_scoring_enabled": false,
            "improvement_suggestions_enabled": false,
            "pdf_export_enabled": false,
            "share_link_enabled": false,
            "api_access": false
        }"#;
        let limits: TierLimits = serde_json::from_str(json).unwrap();
        assert_eq!(limits.max_active_sessions, Some(3));
        assert_eq!(limits.ai_model_tier, AiModelTier::Standard);
    }

    // ─── Legacy Compatibility Tests ────────────────────────────────

    #[test]
    #[allow(deprecated)]
    fn session_limit_reached_is_inverse_of_can_create() {
        let limits = TierLimits::free();
        assert!(!limits.session_limit_reached(2));
        assert!(limits.session_limit_reached(3));
    }

    #[test]
    #[allow(deprecated)]
    fn cycle_limit_reached_is_inverse_of_can_create() {
        let limits = TierLimits::free();
        assert!(!limits.cycle_limit_reached(1));
        assert!(limits.cycle_limit_reached(2));
    }

    #[test]
    #[allow(deprecated)]
    fn max_sessions_returns_max_active_sessions() {
        let limits = TierLimits::free();
        assert_eq!(limits.max_sessions(), Some(3));
    }

    #[test]
    #[allow(deprecated)]
    fn export_enabled_returns_pdf_export_enabled() {
        let limits = TierLimits::free();
        assert!(!limits.export_enabled());

        let limits = TierLimits::premium();
        assert!(limits.export_enabled());
    }
}
