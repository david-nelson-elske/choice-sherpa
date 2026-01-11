//! Decision-making style types for capturing how users approach decisions

use serde::{Deserialize, Serialize};

/// Primary decision-making approach
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StyleClassification {
    /// Data-driven, careful
    AnalyticalCautious,
    /// Data-driven, action-oriented
    AnalyticalDynamic,
    /// Gut-feel, careful
    IntuitiveCautious,
    /// Gut-feel, action-oriented
    IntuitiveDynamic,
    /// Mix of approaches
    Balanced,
}

impl Default for StyleClassification {
    fn default() -> Self {
        Self::Balanced
    }
}

impl std::fmt::Display for StyleClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AnalyticalCautious => write!(f, "Analytical-Cautious"),
            Self::AnalyticalDynamic => write!(f, "Analytical-Dynamic"),
            Self::IntuitiveCautious => write!(f, "Intuitive-Cautious"),
            Self::IntuitiveDynamic => write!(f, "Intuitive-Dynamic"),
            Self::Balanced => write!(f, "Balanced"),
        }
    }
}

/// Dimension level in decision-making
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DimensionLevel {
    VeryLow,
    Low,
    Moderate,
    High,
    VeryHigh,
}

impl DimensionLevel {
    pub fn to_score(&self) -> u8 {
        match self {
            Self::VeryLow => 1,
            Self::Low => 2,
            Self::Moderate => 3,
            Self::High => 4,
            Self::VeryHigh => 5,
        }
    }
}

impl Default for DimensionLevel {
    fn default() -> Self {
        Self::Moderate
    }
}

/// Strength of a dimension observation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrengthLevel {
    Weak,
    Moderate,
    Strong,
}

impl Default for StrengthLevel {
    fn default() -> Self {
        Self::Moderate
    }
}

/// Score for a decision-making dimension
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DimensionScore {
    pub level: DimensionLevel,
    pub strength: StrengthLevel,
    pub notes: Option<String>,
}

impl DimensionScore {
    pub fn new(level: DimensionLevel, strength: StrengthLevel, notes: Option<String>) -> Self {
        Self {
            level,
            strength,
            notes,
        }
    }
}

impl Default for DimensionScore {
    fn default() -> Self {
        Self {
            level: DimensionLevel::default(),
            strength: StrengthLevel::default(),
            notes: None,
        }
    }
}

/// Decision-making style dimensions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StyleDimensions {
    /// How much information is gathered before deciding
    pub information_gathering: DimensionScore,
    /// Tendency to over-analyze
    pub analysis_paralysis_risk: DimensionScore,
    /// Trust in gut feelings
    pub intuition_trust: DimensionScore,
    /// Consideration of others affected
    pub stakeholder_consideration: DimensionScore,
    /// Weight given to reversibility
    pub reversibility_weighting: DimensionScore,
}

impl Default for StyleDimensions {
    fn default() -> Self {
        Self {
            information_gathering: DimensionScore::default(),
            analysis_paralysis_risk: DimensionScore::default(),
            intuition_trust: DimensionScore::default(),
            stakeholder_consideration: DimensionScore::default(),
            reversibility_weighting: DimensionScore::default(),
        }
    }
}

/// Severity of a cognitive pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeverityLevel {
    Mild,
    Moderate,
    Strong,
}

impl Default for SeverityLevel {
    fn default() -> Self {
        Self::Mild
    }
}

/// Types of cognitive biases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitiveBiasType {
    Anchoring,
    LossAversion,
    StatusQuoBias,
    ConfirmationBias,
    OverconfidenceBias,
    AvailabilityBias,
    SunkCostFallacy,
    PlanningFallacy,
}

impl std::fmt::Display for CognitiveBiasType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anchoring => write!(f, "Anchoring"),
            Self::LossAversion => write!(f, "Loss Aversion"),
            Self::StatusQuoBias => write!(f, "Status Quo Bias"),
            Self::ConfirmationBias => write!(f, "Confirmation Bias"),
            Self::OverconfidenceBias => write!(f, "Overconfidence Bias"),
            Self::AvailabilityBias => write!(f, "Availability Bias"),
            Self::SunkCostFallacy => write!(f, "Sunk Cost Fallacy"),
            Self::PlanningFallacy => write!(f, "Planning Fallacy"),
        }
    }
}

/// Identified cognitive pattern/bias
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CognitivePattern {
    pub bias_type: CognitiveBiasType,
    pub severity: SeverityLevel,
    pub evidence: String,
    pub mitigation_prompt: String,
}

impl CognitivePattern {
    pub fn new(
        bias_type: CognitiveBiasType,
        severity: SeverityLevel,
        evidence: String,
        mitigation_prompt: String,
    ) -> Result<Self, String> {
        if evidence.trim().is_empty() {
            return Err("Evidence cannot be empty".to_string());
        }
        if mitigation_prompt.trim().is_empty() {
            return Err("Mitigation prompt cannot be empty".to_string());
        }

        Ok(Self {
            bias_type,
            severity,
            evidence,
            mitigation_prompt,
        })
    }
}

/// Complete decision-making style profile
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionMakingStyle {
    /// Primary decision-making approach
    pub primary_style: StyleClassification,
    /// Dimensional tendencies
    pub dimensions: StyleDimensions,
    /// Identified cognitive biases
    pub cognitive_patterns: Vec<CognitivePattern>,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
}

impl DecisionMakingStyle {
    pub fn new(
        primary_style: StyleClassification,
        dimensions: StyleDimensions,
        cognitive_patterns: Vec<CognitivePattern>,
        confidence: f32,
    ) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&confidence) {
            return Err("Confidence must be between 0.0 and 1.0".to_string());
        }

        Ok(Self {
            primary_style,
            dimensions,
            cognitive_patterns,
            confidence,
        })
    }

    /// Count cognitive patterns by severity
    pub fn severe_patterns(&self) -> usize {
        self.cognitive_patterns
            .iter()
            .filter(|p| matches!(p.severity, SeverityLevel::Strong))
            .count()
    }
}

impl Default for DecisionMakingStyle {
    fn default() -> Self {
        Self {
            primary_style: StyleClassification::default(),
            dimensions: StyleDimensions::default(),
            cognitive_patterns: Vec::new(),
            confidence: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_classification_display() {
        assert_eq!(
            format!("{}", StyleClassification::AnalyticalCautious),
            "Analytical-Cautious"
        );
        assert_eq!(
            format!("{}", StyleClassification::IntuitiveDynamic),
            "Intuitive-Dynamic"
        );
    }

    #[test]
    fn test_dimension_level_to_score() {
        assert_eq!(DimensionLevel::VeryLow.to_score(), 1);
        assert_eq!(DimensionLevel::Low.to_score(), 2);
        assert_eq!(DimensionLevel::Moderate.to_score(), 3);
        assert_eq!(DimensionLevel::High.to_score(), 4);
        assert_eq!(DimensionLevel::VeryHigh.to_score(), 5);
    }

    #[test]
    fn test_dimension_score_creation() {
        let score = DimensionScore::new(
            DimensionLevel::High,
            StrengthLevel::Strong,
            Some("Test note".to_string()),
        );

        assert_eq!(score.level, DimensionLevel::High);
        assert_eq!(score.strength, StrengthLevel::Strong);
        assert_eq!(score.notes, Some("Test note".to_string()));
    }

    #[test]
    fn test_cognitive_bias_type_display() {
        assert_eq!(format!("{}", CognitiveBiasType::Anchoring), "Anchoring");
        assert_eq!(
            format!("{}", CognitiveBiasType::LossAversion),
            "Loss Aversion"
        );
        assert_eq!(
            format!("{}", CognitiveBiasType::SunkCostFallacy),
            "Sunk Cost Fallacy"
        );
    }

    #[test]
    fn test_cognitive_pattern_creation() {
        let pattern = CognitivePattern::new(
            CognitiveBiasType::Anchoring,
            SeverityLevel::Moderate,
            "Tends to anchor on first number mentioned".to_string(),
            "Introduce alternative reference points early".to_string(),
        );

        assert!(pattern.is_ok());
        let p = pattern.unwrap();
        assert_eq!(p.bias_type, CognitiveBiasType::Anchoring);
        assert_eq!(p.severity, SeverityLevel::Moderate);
    }

    #[test]
    fn test_cognitive_pattern_validation() {
        // Empty evidence
        assert!(CognitivePattern::new(
            CognitiveBiasType::Anchoring,
            SeverityLevel::Mild,
            "".to_string(),
            "Mitigation".to_string()
        )
        .is_err());

        // Empty mitigation
        assert!(CognitivePattern::new(
            CognitiveBiasType::Anchoring,
            SeverityLevel::Mild,
            "Evidence".to_string(),
            "".to_string()
        )
        .is_err());
    }

    #[test]
    fn test_decision_making_style_creation() {
        let style = DecisionMakingStyle::new(
            StyleClassification::AnalyticalCautious,
            StyleDimensions::default(),
            vec![],
            0.75,
        );

        assert!(style.is_ok());
        let s = style.unwrap();
        assert_eq!(s.primary_style, StyleClassification::AnalyticalCautious);
        assert_eq!(s.confidence, 0.75);
    }

    #[test]
    fn test_decision_making_style_confidence_validation() {
        assert!(DecisionMakingStyle::new(
            StyleClassification::Balanced,
            StyleDimensions::default(),
            vec![],
            -0.1
        )
        .is_err());

        assert!(DecisionMakingStyle::new(
            StyleClassification::Balanced,
            StyleDimensions::default(),
            vec![],
            1.1
        )
        .is_err());
    }

    #[test]
    fn test_decision_making_style_severe_patterns() {
        let pattern1 = CognitivePattern::new(
            CognitiveBiasType::Anchoring,
            SeverityLevel::Mild,
            "Evidence".to_string(),
            "Mitigation".to_string(),
        )
        .unwrap();

        let pattern2 = CognitivePattern::new(
            CognitiveBiasType::LossAversion,
            SeverityLevel::Strong,
            "Evidence".to_string(),
            "Mitigation".to_string(),
        )
        .unwrap();

        let pattern3 = CognitivePattern::new(
            CognitiveBiasType::ConfirmationBias,
            SeverityLevel::Strong,
            "Evidence".to_string(),
            "Mitigation".to_string(),
        )
        .unwrap();

        let style = DecisionMakingStyle::new(
            StyleClassification::Balanced,
            StyleDimensions::default(),
            vec![pattern1, pattern2, pattern3],
            0.8,
        )
        .unwrap();

        assert_eq!(style.severe_patterns(), 2);
    }

    #[test]
    fn test_decision_making_style_default() {
        let style = DecisionMakingStyle::default();
        assert_eq!(style.primary_style, StyleClassification::Balanced);
        assert_eq!(style.confidence, 0.0);
        assert_eq!(style.cognitive_patterns.len(), 0);
    }
}
