//! Risk profile types for assessing user risk tolerance

use crate::domain::foundation::{CycleId, Timestamp};
use serde::{Deserialize, Serialize};

/// Overall risk classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskClassification {
    /// Actively pursues high-variance options
    RiskSeeking,
    /// Evaluates options purely on expected value
    RiskNeutral,
    /// Prefers certainty over equivalent expected value
    RiskAverse,
}

impl RiskClassification {
    /// Determine classification from risk score (0.0 - 1.0)
    /// - RiskSeeking: score > 0.6
    /// - RiskNeutral: score 0.4-0.6
    /// - RiskAverse: score < 0.4
    pub fn from_score(score: f32) -> Self {
        if score > 0.6 {
            Self::RiskSeeking
        } else if score >= 0.4 {
            Self::RiskNeutral
        } else {
            Self::RiskAverse
        }
    }
}

impl Default for RiskClassification {
    fn default() -> Self {
        Self::RiskNeutral
    }
}

impl std::fmt::Display for RiskClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RiskSeeking => write!(f, "Risk-Seeking"),
            Self::RiskNeutral => write!(f, "Risk-Neutral"),
            Self::RiskAverse => write!(f, "Risk-Averse"),
        }
    }
}

/// Risk score for a specific dimension (1-5 scale)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskScore {
    /// 1 (very low) to 5 (very high) tolerance
    pub value: u8,
    /// Confidence in this score (0.0 - 1.0)
    pub confidence: f32,
    /// Number of decisions contributing to this score
    pub sample_size: u32,
}

impl RiskScore {
    /// Create a new risk score
    pub fn new(value: u8, confidence: f32, sample_size: u32) -> Result<Self, String> {
        if !(1..=5).contains(&value) {
            return Err("Risk score value must be between 1 and 5".to_string());
        }
        if !(0.0..=1.0).contains(&confidence) {
            return Err("Confidence must be between 0.0 and 1.0".to_string());
        }

        Ok(Self {
            value,
            confidence,
            sample_size,
        })
    }

    /// Create a default score (neutral, low confidence)
    pub fn default_neutral() -> Self {
        Self {
            value: 3,
            confidence: 0.0,
            sample_size: 0,
        }
    }
}

impl Default for RiskScore {
    fn default() -> Self {
        Self::default_neutral()
    }
}

/// Domain-specific risk dimensions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskDimensions {
    /// Tolerance for financial uncertainty (1-5 scale)
    pub financial: RiskScore,
    /// Tolerance for career/professional uncertainty
    pub career: RiskScore,
    /// Tolerance for time-horizon uncertainty
    pub temporal: RiskScore,
    /// Tolerance for relationship/social uncertainty
    pub relational: RiskScore,
    /// Tolerance for health/safety uncertainty
    pub health: RiskScore,
}

impl Default for RiskDimensions {
    fn default() -> Self {
        Self {
            financial: RiskScore::default(),
            career: RiskScore::default(),
            temporal: RiskScore::default(),
            relational: RiskScore::default(),
            health: RiskScore::default(),
        }
    }
}

/// Type of risk indicator observed in user behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskIndicatorType {
    /// Which alternative was chosen
    OptionChoice,
    /// Words used in conversation
    LanguagePattern,
    /// How much info requested before deciding
    InformationSeeking,
    /// How they rated uncertain outcomes
    ConsequenceRating,
    /// Near-term vs long-term focus
    TimePreference,
}

/// Evidence supporting risk classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskEvidence {
    pub decision_id: CycleId,
    pub indicator_type: RiskIndicatorType,
    pub description: String,
    pub weight: f32,
}

impl RiskEvidence {
    pub fn new(
        decision_id: CycleId,
        indicator_type: RiskIndicatorType,
        description: String,
        weight: f32,
    ) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&weight) {
            return Err("Weight must be between 0.0 and 1.0".to_string());
        }

        Ok(Self {
            decision_id,
            indicator_type,
            description,
            weight,
        })
    }
}

/// Complete risk profile for a user
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskProfile {
    /// Overall risk classification
    pub classification: RiskClassification,
    /// Confidence in the classification (0.0 - 1.0)
    pub confidence: f32,
    /// Domain-specific risk tolerances
    pub dimensions: RiskDimensions,
    /// Behavioral evidence supporting classification
    pub evidence: Vec<RiskEvidence>,
    /// Last updated timestamp
    pub updated_at: Timestamp,
}

impl RiskProfile {
    /// Create a new risk profile
    pub fn new(
        classification: RiskClassification,
        confidence: f32,
        dimensions: RiskDimensions,
        evidence: Vec<RiskEvidence>,
        timestamp: Timestamp,
    ) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&confidence) {
            return Err("Confidence must be between 0.0 and 1.0".to_string());
        }

        Ok(Self {
            classification,
            confidence,
            dimensions,
            evidence,
            updated_at: timestamp,
        })
    }

    /// Calculate average risk score across dimensions
    pub fn average_risk_score(&self) -> f32 {
        let sum = self.dimensions.financial.value as f32
            + self.dimensions.career.value as f32
            + self.dimensions.temporal.value as f32
            + self.dimensions.relational.value as f32
            + self.dimensions.health.value as f32;
        sum / 5.0
    }
}

impl Default for RiskProfile {
    fn default() -> Self {
        Self {
            classification: RiskClassification::default(),
            confidence: 0.0,
            dimensions: RiskDimensions::default(),
            evidence: Vec::new(),
            updated_at: Timestamp::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cycle_id() -> CycleId {
        CycleId::new()
    }

    fn test_timestamp() -> Timestamp {
        Timestamp::from_datetime(chrono::DateTime::from_timestamp(1704326400, 0).unwrap())
    }

    #[test]
    fn test_risk_classification_from_score() {
        assert_eq!(
            RiskClassification::from_score(0.2),
            RiskClassification::RiskAverse
        );
        assert_eq!(
            RiskClassification::from_score(0.39),
            RiskClassification::RiskAverse
        );
        assert_eq!(
            RiskClassification::from_score(0.4),
            RiskClassification::RiskNeutral
        );
        assert_eq!(
            RiskClassification::from_score(0.5),
            RiskClassification::RiskNeutral
        );
        assert_eq!(
            RiskClassification::from_score(0.6),
            RiskClassification::RiskNeutral
        );
        assert_eq!(
            RiskClassification::from_score(0.61),
            RiskClassification::RiskSeeking
        );
        assert_eq!(
            RiskClassification::from_score(0.9),
            RiskClassification::RiskSeeking
        );
    }

    #[test]
    fn test_risk_classification_display() {
        assert_eq!(format!("{}", RiskClassification::RiskSeeking), "Risk-Seeking");
        assert_eq!(format!("{}", RiskClassification::RiskNeutral), "Risk-Neutral");
        assert_eq!(format!("{}", RiskClassification::RiskAverse), "Risk-Averse");
    }

    #[test]
    fn test_risk_score_validation() {
        // Valid scores
        assert!(RiskScore::new(1, 0.5, 10).is_ok());
        assert!(RiskScore::new(3, 0.8, 5).is_ok());
        assert!(RiskScore::new(5, 1.0, 20).is_ok());

        // Invalid value
        assert!(RiskScore::new(0, 0.5, 10).is_err());
        assert!(RiskScore::new(6, 0.5, 10).is_err());

        // Invalid confidence
        assert!(RiskScore::new(3, -0.1, 10).is_err());
        assert!(RiskScore::new(3, 1.1, 10).is_err());
    }

    #[test]
    fn test_risk_score_default() {
        let score = RiskScore::default();
        assert_eq!(score.value, 3);
        assert_eq!(score.confidence, 0.0);
        assert_eq!(score.sample_size, 0);
    }

    #[test]
    fn test_risk_dimensions_default() {
        let dims = RiskDimensions::default();
        assert_eq!(dims.financial.value, 3);
        assert_eq!(dims.career.value, 3);
        assert_eq!(dims.temporal.value, 3);
        assert_eq!(dims.relational.value, 3);
        assert_eq!(dims.health.value, 3);
    }

    #[test]
    fn test_risk_evidence_creation() {
        let cycle_id = test_cycle_id();
        let evidence = RiskEvidence::new(
            cycle_id,
            RiskIndicatorType::OptionChoice,
            "Chose high-variance option".to_string(),
            0.7,
        );

        assert!(evidence.is_ok());
        let e = evidence.unwrap();
        assert_eq!(e.decision_id, cycle_id);
        assert_eq!(e.indicator_type, RiskIndicatorType::OptionChoice);
        assert_eq!(e.weight, 0.7);
    }

    #[test]
    fn test_risk_evidence_weight_validation() {
        let cycle_id = test_cycle_id();

        assert!(RiskEvidence::new(
            cycle_id,
            RiskIndicatorType::OptionChoice,
            "Test".to_string(),
            -0.1
        )
        .is_err());

        assert!(RiskEvidence::new(
            cycle_id,
            RiskIndicatorType::OptionChoice,
            "Test".to_string(),
            1.1
        )
        .is_err());
    }

    #[test]
    fn test_risk_profile_creation() {
        let ts = test_timestamp();
        let profile = RiskProfile::new(
            RiskClassification::RiskAverse,
            0.75,
            RiskDimensions::default(),
            Vec::new(),
            ts,
        );

        assert!(profile.is_ok());
        let p = profile.unwrap();
        assert_eq!(p.classification, RiskClassification::RiskAverse);
        assert_eq!(p.confidence, 0.75);
        assert_eq!(p.updated_at, ts);
    }

    #[test]
    fn test_risk_profile_confidence_validation() {
        let ts = test_timestamp();

        assert!(RiskProfile::new(
            RiskClassification::RiskNeutral,
            -0.1,
            RiskDimensions::default(),
            Vec::new(),
            ts,
        )
        .is_err());

        assert!(RiskProfile::new(
            RiskClassification::RiskNeutral,
            1.1,
            RiskDimensions::default(),
            Vec::new(),
            ts,
        )
        .is_err());
    }

    #[test]
    fn test_risk_profile_average_score() {
        let mut dims = RiskDimensions::default();
        dims.financial = RiskScore::new(2, 0.8, 5).unwrap();
        dims.career = RiskScore::new(3, 0.7, 4).unwrap();
        dims.temporal = RiskScore::new(1, 0.9, 6).unwrap();
        dims.relational = RiskScore::new(4, 0.6, 3).unwrap();
        dims.health = RiskScore::new(5, 0.8, 8).unwrap();

        let profile = RiskProfile::new(
            RiskClassification::RiskNeutral,
            0.75,
            dims,
            Vec::new(),
            test_timestamp(),
        )
        .unwrap();

        // (2+3+1+4+5) / 5 = 15 / 5 = 3.0
        assert_eq!(profile.average_risk_score(), 3.0);
    }

    #[test]
    fn test_risk_profile_default() {
        let profile = RiskProfile::default();
        assert_eq!(profile.classification, RiskClassification::RiskNeutral);
        assert_eq!(profile.confidence, 0.0);
        assert_eq!(profile.average_risk_score(), 3.0);
    }
}
