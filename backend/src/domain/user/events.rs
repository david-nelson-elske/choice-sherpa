//! Domain events for the Decision Profile feature

use crate::domain::foundation::{CycleId, Timestamp, UserId};
use serde::{Deserialize, Serialize};

use super::{
    BlindSpot, DecisionProfileId, GrowthObservation, OutcomeRecord, ProfileConsent,
    RiskClassification,
};

/// Domain events for Decision Profile
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProfileEvent {
    /// Profile was created with user consent
    DecisionProfileCreated {
        user_id: UserId,
        profile_id: DecisionProfileId,
        consent: ProfileConsent,
        created_at: Timestamp,
    },

    /// Profile was updated after analyzing a decision
    DecisionProfileUpdated {
        profile_id: DecisionProfileId,
        version: u32,
        decisions_analyzed: u32,
        updated_at: Timestamp,
    },

    /// Risk profile classification changed
    RiskProfileRecalculated {
        profile_id: DecisionProfileId,
        old_classification: RiskClassification,
        new_classification: RiskClassification,
        confidence: f32,
        recalculated_at: Timestamp,
    },

    /// New blind spot identified
    BlindSpotIdentified {
        profile_id: DecisionProfileId,
        blind_spot: BlindSpot,
        identified_at: Timestamp,
    },

    /// Growth observed in a particular area
    GrowthObserved {
        profile_id: DecisionProfileId,
        growth: GrowthObservation,
        observed_at: Timestamp,
    },

    /// Outcome recorded for a past decision
    OutcomeRecorded {
        profile_id: DecisionProfileId,
        cycle_id: CycleId,
        outcome: OutcomeRecord,
        recorded_at: Timestamp,
    },

    /// Consent settings updated
    ConsentUpdated {
        profile_id: DecisionProfileId,
        consent: ProfileConsent,
        updated_at: Timestamp,
    },

    /// Profile deleted by user request
    ProfileDeleted {
        user_id: UserId,
        profile_id: DecisionProfileId,
        deleted_at: Timestamp,
    },

    /// Profile exported
    ProfileExported {
        profile_id: DecisionProfileId,
        format: ExportFormat,
        exported_at: Timestamp,
    },
}

/// Export format for profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Markdown,
    Json,
    Pdf,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Markdown => write!(f, "markdown"),
            Self::Json => write!(f, "json"),
            Self::Pdf => write!(f, "pdf"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_user_id() -> UserId {
        UserId::new("test@example.com".to_string()).unwrap()
    }

    fn test_profile_id() -> DecisionProfileId {
        DecisionProfileId::new()
    }

    fn test_timestamp() -> Timestamp {
        Timestamp::from_datetime(chrono::DateTime::from_timestamp(1704326400, 0).unwrap())
    }

    #[test]
    fn test_profile_created_event() {
        let user_id = test_user_id();
        let profile_id = test_profile_id();
        let ts = test_timestamp();
        let consent = ProfileConsent::full(ts);

        let event = ProfileEvent::DecisionProfileCreated {
            user_id: user_id.clone(),
            profile_id,
            consent: consent.clone(),
            created_at: ts,
        };

        match event {
            ProfileEvent::DecisionProfileCreated {
                user_id: uid,
                profile_id: pid,
                ..
            } => {
                assert_eq!(uid, user_id);
                assert_eq!(pid, profile_id);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_profile_updated_event() {
        let profile_id = test_profile_id();
        let ts = test_timestamp();

        let event = ProfileEvent::DecisionProfileUpdated {
            profile_id,
            version: 2,
            decisions_analyzed: 1,
            updated_at: ts,
        };

        match event {
            ProfileEvent::DecisionProfileUpdated { version, .. } => {
                assert_eq!(version, 2);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_risk_profile_recalculated_event() {
        let profile_id = test_profile_id();
        let ts = test_timestamp();

        let event = ProfileEvent::RiskProfileRecalculated {
            profile_id,
            old_classification: RiskClassification::RiskNeutral,
            new_classification: RiskClassification::RiskAverse,
            confidence: 0.75,
            recalculated_at: ts,
        };

        match event {
            ProfileEvent::RiskProfileRecalculated {
                old_classification,
                new_classification,
                ..
            } => {
                assert_eq!(old_classification, RiskClassification::RiskNeutral);
                assert_eq!(new_classification, RiskClassification::RiskAverse);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_export_format_display() {
        assert_eq!(format!("{}", ExportFormat::Markdown), "markdown");
        assert_eq!(format!("{}", ExportFormat::Json), "json");
        assert_eq!(format!("{}", ExportFormat::Pdf), "pdf");
    }

    #[test]
    fn test_profile_deleted_event() {
        let user_id = test_user_id();
        let profile_id = test_profile_id();
        let ts = test_timestamp();

        let event = ProfileEvent::ProfileDeleted {
            user_id: user_id.clone(),
            profile_id,
            deleted_at: ts,
        };

        match event {
            ProfileEvent::ProfileDeleted { user_id: uid, .. } => {
                assert_eq!(uid, user_id);
            }
            _ => panic!("Wrong event type"),
        }
    }
}
