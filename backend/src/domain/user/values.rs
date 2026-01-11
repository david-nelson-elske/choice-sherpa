//! Values and priorities types for tracking consistent objectives

use crate::domain::foundation::{CycleId, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::history::DecisionDomain;

/// Weight given to an objective
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveWeight {
    Low,
    Medium,
    High,
    Critical,
}

impl ObjectiveWeight {
    /// Convert to numeric score (1-4)
    pub fn to_score(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }

    /// Create from frequency and importance patterns
    pub fn from_score(score: u8) -> Option<Self> {
        match score {
            1 => Some(Self::Low),
            2 => Some(Self::Medium),
            3 => Some(Self::High),
            4 => Some(Self::Critical),
            _ => None,
        }
    }
}

impl std::fmt::Display for ObjectiveWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// An objective that appears consistently across decisions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsistentObjective {
    /// Name of the objective
    pub name: String,
    /// Frequency across decisions (0.0 - 1.0)
    pub frequency: f32,
    /// Typical weight given to this objective
    pub typical_weight: ObjectiveWeight,
    /// First time this objective appeared
    pub first_seen: Timestamp,
    /// Most recent appearance
    pub last_seen: Timestamp,
}

impl ConsistentObjective {
    pub fn new(
        name: String,
        frequency: f32,
        typical_weight: ObjectiveWeight,
        first_seen: Timestamp,
        last_seen: Timestamp,
    ) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("Objective name cannot be empty".to_string());
        }
        if !(0.0..=1.0).contains(&frequency) {
            return Err("Frequency must be between 0.0 and 1.0".to_string());
        }

        Ok(Self {
            name,
            frequency,
            typical_weight,
            first_seen,
            last_seen,
        })
    }

    /// Check if this is a high-frequency objective (appears in 60%+ of decisions)
    pub fn is_core_value(&self) -> bool {
        self.frequency >= 0.6
    }
}

/// A tension between two values with resolution pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueTension {
    /// First value in tension
    pub value_a: String,
    /// Second value in tension
    pub value_b: String,
    /// How this tension is typically resolved
    pub resolution_pattern: String,
    /// Example decisions showing this tension
    pub examples: Vec<CycleId>,
}

impl ValueTension {
    pub fn new(
        value_a: String,
        value_b: String,
        resolution_pattern: String,
        examples: Vec<CycleId>,
    ) -> Result<Self, String> {
        if value_a.trim().is_empty() || value_b.trim().is_empty() {
            return Err("Value names cannot be empty".to_string());
        }
        if resolution_pattern.trim().is_empty() {
            return Err("Resolution pattern cannot be empty".to_string());
        }

        Ok(Self {
            value_a,
            value_b,
            resolution_pattern,
            examples,
        })
    }
}

/// Values and priorities identified across decisions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValuesPriorities {
    /// Objectives that appear frequently
    pub consistent_objectives: Vec<ConsistentObjective>,
    /// Observed tensions between values
    pub value_tensions: Vec<ValueTension>,
    /// Domain-specific value patterns (objective names by domain)
    pub domain_patterns: HashMap<DecisionDomain, Vec<String>>,
}

impl ValuesPriorities {
    pub fn new(
        consistent_objectives: Vec<ConsistentObjective>,
        value_tensions: Vec<ValueTension>,
        domain_patterns: HashMap<DecisionDomain, Vec<String>>,
    ) -> Self {
        Self {
            consistent_objectives,
            value_tensions,
            domain_patterns,
        }
    }

    /// Get core values (frequency >= 60%)
    pub fn core_values(&self) -> Vec<&ConsistentObjective> {
        self.consistent_objectives
            .iter()
            .filter(|obj| obj.is_core_value())
            .collect()
    }

    /// Count total consistent objectives
    pub fn objective_count(&self) -> usize {
        self.consistent_objectives.len()
    }

    /// Count value tensions
    pub fn tension_count(&self) -> usize {
        self.value_tensions.len()
    }
}

impl Default for ValuesPriorities {
    fn default() -> Self {
        Self {
            consistent_objectives: Vec::new(),
            value_tensions: Vec::new(),
            domain_patterns: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_timestamp() -> Timestamp {
        Timestamp::from_datetime(chrono::DateTime::from_timestamp(1704326400, 0).unwrap())
    }

    fn test_timestamp_later() -> Timestamp {
        Timestamp::from_datetime(chrono::DateTime::from_timestamp(1704412800, 0).unwrap())
    }

    #[test]
    fn test_objective_weight_to_score() {
        assert_eq!(ObjectiveWeight::Low.to_score(), 1);
        assert_eq!(ObjectiveWeight::Medium.to_score(), 2);
        assert_eq!(ObjectiveWeight::High.to_score(), 3);
        assert_eq!(ObjectiveWeight::Critical.to_score(), 4);
    }

    #[test]
    fn test_objective_weight_from_score() {
        assert_eq!(ObjectiveWeight::from_score(1), Some(ObjectiveWeight::Low));
        assert_eq!(
            ObjectiveWeight::from_score(2),
            Some(ObjectiveWeight::Medium)
        );
        assert_eq!(ObjectiveWeight::from_score(3), Some(ObjectiveWeight::High));
        assert_eq!(
            ObjectiveWeight::from_score(4),
            Some(ObjectiveWeight::Critical)
        );
        assert_eq!(ObjectiveWeight::from_score(0), None);
        assert_eq!(ObjectiveWeight::from_score(5), None);
    }

    #[test]
    fn test_consistent_objective_creation() {
        let ts1 = test_timestamp();
        let ts2 = test_timestamp_later();

        let obj = ConsistentObjective::new(
            "Work-life balance".to_string(),
            0.75,
            ObjectiveWeight::High,
            ts1,
            ts2,
        );

        assert!(obj.is_ok());
        let o = obj.unwrap();
        assert_eq!(o.name, "Work-life balance");
        assert_eq!(o.frequency, 0.75);
        assert_eq!(o.typical_weight, ObjectiveWeight::High);
    }

    #[test]
    fn test_consistent_objective_validation() {
        let ts = test_timestamp();

        // Empty name
        assert!(ConsistentObjective::new(
            "".to_string(),
            0.5,
            ObjectiveWeight::Medium,
            ts,
            ts
        )
        .is_err());

        // Invalid frequency
        assert!(ConsistentObjective::new(
            "Test".to_string(),
            -0.1,
            ObjectiveWeight::Medium,
            ts,
            ts
        )
        .is_err());

        assert!(ConsistentObjective::new(
            "Test".to_string(),
            1.1,
            ObjectiveWeight::Medium,
            ts,
            ts
        )
        .is_err());
    }

    #[test]
    fn test_consistent_objective_is_core_value() {
        let ts = test_timestamp();

        let low_freq = ConsistentObjective::new(
            "Rare objective".to_string(),
            0.3,
            ObjectiveWeight::Medium,
            ts,
            ts,
        )
        .unwrap();
        assert!(!low_freq.is_core_value());

        let high_freq = ConsistentObjective::new(
            "Core value".to_string(),
            0.75,
            ObjectiveWeight::High,
            ts,
            ts,
        )
        .unwrap();
        assert!(high_freq.is_core_value());

        let threshold = ConsistentObjective::new(
            "At threshold".to_string(),
            0.6,
            ObjectiveWeight::High,
            ts,
            ts,
        )
        .unwrap();
        assert!(threshold.is_core_value());
    }

    #[test]
    fn test_value_tension_creation() {
        let tension = ValueTension::new(
            "Growth".to_string(),
            "Stability".to_string(),
            "Usually chooses stability unless growth gap is large".to_string(),
            vec![CycleId::new(), CycleId::new()],
        );

        assert!(tension.is_ok());
        let t = tension.unwrap();
        assert_eq!(t.value_a, "Growth");
        assert_eq!(t.value_b, "Stability");
        assert_eq!(t.examples.len(), 2);
    }

    #[test]
    fn test_value_tension_validation() {
        // Empty value names
        assert!(ValueTension::new(
            "".to_string(),
            "B".to_string(),
            "Pattern".to_string(),
            vec![]
        )
        .is_err());

        assert!(ValueTension::new(
            "A".to_string(),
            "".to_string(),
            "Pattern".to_string(),
            vec![]
        )
        .is_err());

        // Empty resolution pattern
        assert!(ValueTension::new(
            "A".to_string(),
            "B".to_string(),
            "".to_string(),
            vec![]
        )
        .is_err());
    }

    #[test]
    fn test_values_priorities_default() {
        let vp = ValuesPriorities::default();
        assert_eq!(vp.objective_count(), 0);
        assert_eq!(vp.tension_count(), 0);
        assert_eq!(vp.core_values().len(), 0);
    }

    #[test]
    fn test_values_priorities_core_values() {
        let ts = test_timestamp();

        let obj1 = ConsistentObjective::new(
            "Core value 1".to_string(),
            0.8,
            ObjectiveWeight::High,
            ts,
            ts,
        )
        .unwrap();

        let obj2 = ConsistentObjective::new(
            "Not core".to_string(),
            0.4,
            ObjectiveWeight::Medium,
            ts,
            ts,
        )
        .unwrap();

        let obj3 = ConsistentObjective::new(
            "Core value 2".to_string(),
            0.65,
            ObjectiveWeight::High,
            ts,
            ts,
        )
        .unwrap();

        let vp = ValuesPriorities::new(vec![obj1, obj2, obj3], vec![], HashMap::new());

        let core = vp.core_values();
        assert_eq!(core.len(), 2);
        assert!(core.iter().any(|o| o.name == "Core value 1"));
        assert!(core.iter().any(|o| o.name == "Core value 2"));
    }

    #[test]
    fn test_values_priorities_domain_patterns() {
        let mut patterns = HashMap::new();
        patterns.insert(
            DecisionDomain::Career,
            vec!["Growth".to_string(), "Learning".to_string()],
        );
        patterns.insert(
            DecisionDomain::Family,
            vec!["Quality time".to_string(), "Stability".to_string()],
        );

        let vp = ValuesPriorities::new(vec![], vec![], patterns);

        assert_eq!(vp.domain_patterns.len(), 2);
        assert!(vp.domain_patterns.contains_key(&DecisionDomain::Career));
        assert!(vp.domain_patterns.contains_key(&DecisionDomain::Family));
    }
}
