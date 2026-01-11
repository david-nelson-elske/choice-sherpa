//! DecisionProfile aggregate root and core value objects

use crate::domain::foundation::{Timestamp, UserId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    BlindSpotsGrowth, CommunicationPreferences, DecisionHistory, DecisionMakingStyle,
    RiskProfile, ValuesPriorities,
};

/// Unique identifier for a decision profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DecisionProfileId(Uuid);

impl DecisionProfileId {
    /// Create a new random profile ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from existing UUID
    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    /// Get inner UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for DecisionProfileId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DecisionProfileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Profile version for tracking updates
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProfileVersion(u32);

impl ProfileVersion {
    /// Create initial version (1)
    pub fn initial() -> Self {
        Self(1)
    }

    /// Create from value
    pub fn from_u32(value: u32) -> Result<Self, &'static str> {
        if value == 0 {
            Err("Profile version must be greater than 0")
        } else {
            Ok(Self(value))
        }
    }

    /// Increment version
    pub fn increment(&self) -> Self {
        Self(self.0 + 1)
    }

    /// Get inner value
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl Default for ProfileVersion {
    fn default() -> Self {
        Self::initial()
    }
}

impl std::fmt::Display for ProfileVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Profile confidence level based on decisions analyzed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileConfidence {
    /// Less than 3 decisions analyzed
    Low,
    /// 3-7 decisions
    Medium,
    /// 8-15 decisions
    High,
    /// 15+ decisions
    VeryHigh,
}

impl ProfileConfidence {
    /// Calculate confidence from number of decisions
    pub fn from_decisions(count: u32) -> Self {
        match count {
            0..=2 => Self::Low,
            3..=7 => Self::Medium,
            8..=15 => Self::High,
            _ => Self::VeryHigh,
        }
    }

    /// Get minimum decisions for this confidence level
    pub fn min_decisions(&self) -> u32 {
        match self {
            Self::Low => 0,
            Self::Medium => 3,
            Self::High => 8,
            Self::VeryHigh => 16,
        }
    }
}

impl std::fmt::Display for ProfileConfidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::VeryHigh => write!(f, "Very High"),
        }
    }
}

/// User consent for profile collection and analysis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileConsent {
    /// Allow data collection
    pub collection_enabled: bool,
    /// Allow profile analysis
    pub analysis_enabled: bool,
    /// Allow agent to access profile
    pub agent_access_enabled: bool,
    /// When consent was given
    pub consented_at: Timestamp,
    /// Last time consent was reviewed/updated
    pub last_reviewed: Timestamp,
}

impl ProfileConsent {
    /// Create new consent with all permissions enabled
    pub fn full(timestamp: Timestamp) -> Self {
        Self {
            collection_enabled: true,
            analysis_enabled: true,
            agent_access_enabled: true,
            consented_at: timestamp,
            last_reviewed: timestamp,
        }
    }

    /// Create limited consent (collection only)
    pub fn limited(timestamp: Timestamp) -> Self {
        Self {
            collection_enabled: true,
            analysis_enabled: false,
            agent_access_enabled: false,
            consented_at: timestamp,
            last_reviewed: timestamp,
        }
    }

    /// Check if profile creation is allowed
    pub fn allows_creation(&self) -> bool {
        self.collection_enabled
    }

    /// Check if analysis is allowed
    pub fn allows_analysis(&self) -> bool {
        self.analysis_enabled
    }

    /// Check if agent access is allowed
    pub fn allows_agent_access(&self) -> bool {
        self.agent_access_enabled
    }

    /// Update consent settings
    pub fn update(
        &mut self,
        collection: bool,
        analysis: bool,
        agent_access: bool,
        timestamp: Timestamp,
    ) {
        self.collection_enabled = collection;
        self.analysis_enabled = analysis;
        self.agent_access_enabled = agent_access;
        self.last_reviewed = timestamp;
    }
}

/// DecisionProfile aggregate root
///
/// A user-owned artifact that captures decision-making patterns across sessions
#[derive(Debug, Clone)]
pub struct DecisionProfile {
    // Identity
    id: DecisionProfileId,
    user_id: UserId,

    // Profile components
    risk_profile: RiskProfile,
    values_priorities: ValuesPriorities,
    decision_style: DecisionMakingStyle,
    blind_spots_growth: BlindSpotsGrowth,
    communication_prefs: CommunicationPreferences,
    decision_history: DecisionHistory,

    // Metadata
    version: ProfileVersion,
    created_at: Timestamp,
    updated_at: Timestamp,
    decisions_analyzed: u32,
    profile_confidence: ProfileConfidence,

    // Privacy
    consent: ProfileConsent,
}

impl DecisionProfile {
    /// Create a new profile with consent
    pub fn new(user_id: UserId, consent: ProfileConsent, timestamp: Timestamp) -> Result<Self, String> {
        if !consent.allows_creation() {
            return Err("Consent required for profile creation".to_string());
        }

        Ok(Self {
            id: DecisionProfileId::new(),
            user_id,
            risk_profile: RiskProfile::default(),
            values_priorities: ValuesPriorities::default(),
            decision_style: DecisionMakingStyle::default(),
            blind_spots_growth: BlindSpotsGrowth::default(),
            communication_prefs: CommunicationPreferences::default(),
            decision_history: DecisionHistory::default(),
            version: ProfileVersion::initial(),
            created_at: timestamp,
            updated_at: timestamp,
            decisions_analyzed: 0,
            profile_confidence: ProfileConfidence::Low,
            consent,
        })
    }

    // Getters
    pub fn id(&self) -> DecisionProfileId {
        self.id
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn risk_profile(&self) -> &RiskProfile {
        &self.risk_profile
    }

    pub fn values_priorities(&self) -> &ValuesPriorities {
        &self.values_priorities
    }

    pub fn decision_style(&self) -> &DecisionMakingStyle {
        &self.decision_style
    }

    pub fn blind_spots_growth(&self) -> &BlindSpotsGrowth {
        &self.blind_spots_growth
    }

    pub fn communication_prefs(&self) -> &CommunicationPreferences {
        &self.communication_prefs
    }

    pub fn decision_history(&self) -> &DecisionHistory {
        &self.decision_history
    }

    pub fn version(&self) -> ProfileVersion {
        self.version
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    pub fn decisions_analyzed(&self) -> u32 {
        self.decisions_analyzed
    }

    pub fn profile_confidence(&self) -> ProfileConfidence {
        self.profile_confidence
    }

    pub fn consent(&self) -> &ProfileConsent {
        &self.consent
    }

    /// Update profile after analyzing a decision
    pub fn update_from_analysis(
        &mut self,
        risk_profile: RiskProfile,
        values_priorities: ValuesPriorities,
        decision_style: DecisionMakingStyle,
        blind_spots_growth: BlindSpotsGrowth,
        communication_prefs: CommunicationPreferences,
        decision_history: DecisionHistory,
        timestamp: Timestamp,
    ) {
        self.risk_profile = risk_profile;
        self.values_priorities = values_priorities;
        self.decision_style = decision_style;
        self.blind_spots_growth = blind_spots_growth;
        self.communication_prefs = communication_prefs;
        self.decision_history = decision_history;
        self.decisions_analyzed += 1;
        self.profile_confidence = ProfileConfidence::from_decisions(self.decisions_analyzed);
        self.version = self.version.increment();
        self.updated_at = timestamp;
    }

    /// Update consent settings
    pub fn update_consent(&mut self, consent: ProfileConsent, timestamp: Timestamp) {
        self.consent = consent;
        self.updated_at = timestamp;
        self.version = self.version.increment();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_user_id() -> UserId {
        UserId::new("test-user@example.com".to_string()).unwrap()
    }

    fn test_timestamp() -> Timestamp {
        Timestamp::from_datetime(chrono::DateTime::from_timestamp(1704326400, 0).unwrap()) // 2024-01-04
    }

    #[test]
    fn test_profile_id_creation() {
        let id1 = DecisionProfileId::new();
        let id2 = DecisionProfileId::new();
        assert_ne!(id1, id2, "Profile IDs should be unique");
    }

    #[test]
    fn test_profile_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id = DecisionProfileId::from_uuid(uuid);
        assert_eq!(id.as_uuid(), uuid);
    }

    #[test]
    fn test_profile_version_initial() {
        let version = ProfileVersion::initial();
        assert_eq!(version.as_u32(), 1);
    }

    #[test]
    fn test_profile_version_from_u32() {
        assert!(ProfileVersion::from_u32(0).is_err());
        assert_eq!(ProfileVersion::from_u32(1).unwrap().as_u32(), 1);
        assert_eq!(ProfileVersion::from_u32(42).unwrap().as_u32(), 42);
    }

    #[test]
    fn test_profile_version_increment() {
        let v1 = ProfileVersion::initial();
        let v2 = v1.increment();
        assert_eq!(v2.as_u32(), 2);

        let v3 = v2.increment();
        assert_eq!(v3.as_u32(), 3);
    }

    #[test]
    fn test_profile_confidence_from_decisions() {
        assert_eq!(ProfileConfidence::from_decisions(0), ProfileConfidence::Low);
        assert_eq!(ProfileConfidence::from_decisions(2), ProfileConfidence::Low);
        assert_eq!(ProfileConfidence::from_decisions(3), ProfileConfidence::Medium);
        assert_eq!(ProfileConfidence::from_decisions(7), ProfileConfidence::Medium);
        assert_eq!(ProfileConfidence::from_decisions(8), ProfileConfidence::High);
        assert_eq!(ProfileConfidence::from_decisions(15), ProfileConfidence::High);
        assert_eq!(ProfileConfidence::from_decisions(16), ProfileConfidence::VeryHigh);
        assert_eq!(ProfileConfidence::from_decisions(100), ProfileConfidence::VeryHigh);
    }

    #[test]
    fn test_profile_confidence_min_decisions() {
        assert_eq!(ProfileConfidence::Low.min_decisions(), 0);
        assert_eq!(ProfileConfidence::Medium.min_decisions(), 3);
        assert_eq!(ProfileConfidence::High.min_decisions(), 8);
        assert_eq!(ProfileConfidence::VeryHigh.min_decisions(), 16);
    }

    #[test]
    fn test_profile_consent_full() {
        let ts = test_timestamp();
        let consent = ProfileConsent::full(ts);

        assert!(consent.collection_enabled);
        assert!(consent.analysis_enabled);
        assert!(consent.agent_access_enabled);
        assert_eq!(consent.consented_at, ts);
        assert_eq!(consent.last_reviewed, ts);
    }

    #[test]
    fn test_profile_consent_limited() {
        let ts = test_timestamp();
        let consent = ProfileConsent::limited(ts);

        assert!(consent.collection_enabled);
        assert!(!consent.analysis_enabled);
        assert!(!consent.agent_access_enabled);
    }

    #[test]
    fn test_profile_consent_allows() {
        let ts = test_timestamp();
        let full = ProfileConsent::full(ts);
        assert!(full.allows_creation());
        assert!(full.allows_analysis());
        assert!(full.allows_agent_access());

        let limited = ProfileConsent::limited(ts);
        assert!(limited.allows_creation());
        assert!(!limited.allows_analysis());
        assert!(!limited.allows_agent_access());
    }

    #[test]
    fn test_profile_consent_update() {
        let ts1 = test_timestamp();
        let ts2 = Timestamp::from_datetime(chrono::DateTime::from_timestamp(1704412800, 0).unwrap()); // next day
        let mut consent = ProfileConsent::full(ts1);

        consent.update(true, false, false, ts2);

        assert!(consent.collection_enabled);
        assert!(!consent.analysis_enabled);
        assert!(!consent.agent_access_enabled);
        assert_eq!(consent.last_reviewed, ts2);
        assert_eq!(consent.consented_at, ts1); // Original consent time unchanged
    }

    #[test]
    fn test_decision_profile_new_requires_consent() {
        let user_id = test_user_id();
        let ts = test_timestamp();
        let no_consent = ProfileConsent {
            collection_enabled: false,
            analysis_enabled: false,
            agent_access_enabled: false,
            consented_at: ts,
            last_reviewed: ts,
        };

        let result = DecisionProfile::new(user_id, no_consent, ts);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Consent required"));
    }

    #[test]
    fn test_decision_profile_new_with_consent() {
        let user_id = test_user_id();
        let ts = test_timestamp();
        let consent = ProfileConsent::full(ts);

        let profile = DecisionProfile::new(user_id.clone(), consent, ts).unwrap();

        assert_eq!(profile.user_id(), &user_id);
        assert_eq!(profile.version().as_u32(), 1);
        assert_eq!(profile.decisions_analyzed(), 0);
        assert_eq!(profile.profile_confidence(), ProfileConfidence::Low);
        assert_eq!(profile.created_at(), ts);
        assert_eq!(profile.updated_at(), ts);
    }

    #[test]
    fn test_decision_profile_update_from_analysis() {
        let user_id = test_user_id();
        let ts1 = test_timestamp();
        let ts2 = Timestamp::from_datetime(chrono::DateTime::from_timestamp(1704412800, 0).unwrap());
        let consent = ProfileConsent::full(ts1);

        let mut profile = DecisionProfile::new(user_id, consent, ts1).unwrap();
        let initial_version = profile.version();

        profile.update_from_analysis(
            RiskProfile::default(),
            ValuesPriorities::default(),
            DecisionMakingStyle::default(),
            BlindSpotsGrowth::default(),
            CommunicationPreferences::default(),
            DecisionHistory::default(),
            ts2,
        );

        assert_eq!(profile.decisions_analyzed(), 1);
        assert_eq!(profile.version(), initial_version.increment());
        assert_eq!(profile.updated_at(), ts2);
    }

    #[test]
    fn test_decision_profile_confidence_increases() {
        let user_id = test_user_id();
        let ts = test_timestamp();
        let consent = ProfileConsent::full(ts);

        let mut profile = DecisionProfile::new(user_id, consent, ts).unwrap();
        assert_eq!(profile.profile_confidence(), ProfileConfidence::Low);

        // Add 3 decisions -> Medium
        for _ in 0..3 {
            profile.update_from_analysis(
                RiskProfile::default(),
                ValuesPriorities::default(),
                DecisionMakingStyle::default(),
                BlindSpotsGrowth::default(),
                CommunicationPreferences::default(),
                DecisionHistory::default(),
                ts,
            );
        }
        assert_eq!(profile.profile_confidence(), ProfileConfidence::Medium);

        // Add 5 more (total 8) -> High
        for _ in 0..5 {
            profile.update_from_analysis(
                RiskProfile::default(),
                ValuesPriorities::default(),
                DecisionMakingStyle::default(),
                BlindSpotsGrowth::default(),
                CommunicationPreferences::default(),
                DecisionHistory::default(),
                ts,
            );
        }
        assert_eq!(profile.profile_confidence(), ProfileConfidence::High);
    }

    #[test]
    fn test_decision_profile_update_consent() {
        let user_id = test_user_id();
        let ts1 = test_timestamp();
        let ts2 = Timestamp::from_datetime(chrono::DateTime::from_timestamp(1704412800, 0).unwrap());
        let consent = ProfileConsent::full(ts1);

        let mut profile = DecisionProfile::new(user_id, consent, ts1).unwrap();
        let initial_version = profile.version();

        let new_consent = ProfileConsent::limited(ts2);
        profile.update_consent(new_consent, ts2);

        assert!(!profile.consent().allows_analysis());
        assert_eq!(profile.version(), initial_version.increment());
        assert_eq!(profile.updated_at(), ts2);
    }
}
