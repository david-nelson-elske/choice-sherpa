//! Blind spots and growth areas tracking

use crate::domain::foundation::Timestamp;
use serde::{Deserialize, Serialize};

/// Identified blind spot in decision-making
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlindSpot {
    pub name: String,
    pub description: String,
    pub evidence: Vec<String>,
    /// What the agent should do to address this
    pub agent_behavior: String,
    pub identified_at: Timestamp,
    pub still_active: bool,
}

impl BlindSpot {
    pub fn new(
        name: String,
        description: String,
        evidence: Vec<String>,
        agent_behavior: String,
        timestamp: Timestamp,
    ) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("Blind spot name cannot be empty".to_string());
        }
        if description.trim().is_empty() {
            return Err("Description cannot be empty".to_string());
        }
        if agent_behavior.trim().is_empty() {
            return Err("Agent behavior cannot be empty".to_string());
        }

        Ok(Self {
            name,
            description,
            evidence,
            agent_behavior,
            identified_at: timestamp,
            still_active: true,
        })
    }

    /// Mark blind spot as resolved
    pub fn resolve(&mut self) {
        self.still_active = false;
    }

    /// Check if blind spot is still active
    pub fn is_active(&self) -> bool {
        self.still_active
    }
}

/// Observed improvement over time
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrowthObservation {
    pub area: String,
    pub before_behavior: String,
    pub after_behavior: String,
    pub trigger: String,
    pub observed_at: Timestamp,
}

impl GrowthObservation {
    pub fn new(
        area: String,
        before_behavior: String,
        after_behavior: String,
        trigger: String,
        timestamp: Timestamp,
    ) -> Result<Self, String> {
        if area.trim().is_empty() {
            return Err("Area cannot be empty".to_string());
        }
        if before_behavior.trim().is_empty() || after_behavior.trim().is_empty() {
            return Err("Behaviors cannot be empty".to_string());
        }
        if trigger.trim().is_empty() {
            return Err("Trigger cannot be empty".to_string());
        }

        Ok(Self {
            area,
            before_behavior,
            after_behavior,
            trigger,
            observed_at: timestamp,
        })
    }
}

/// Blind spots and growth tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlindSpotsGrowth {
    /// Identified blind spots
    pub blind_spots: Vec<BlindSpot>,
    /// Observed improvements over time
    pub growth_areas: Vec<GrowthObservation>,
    /// Suggested focus areas
    pub suggested_focus: Vec<String>,
}

impl BlindSpotsGrowth {
    pub fn new(
        blind_spots: Vec<BlindSpot>,
        growth_areas: Vec<GrowthObservation>,
        suggested_focus: Vec<String>,
    ) -> Self {
        Self {
            blind_spots,
            growth_areas,
            suggested_focus,
        }
    }

    /// Get active blind spots
    pub fn active_blind_spots(&self) -> Vec<&BlindSpot> {
        self.blind_spots
            .iter()
            .filter(|bs| bs.is_active())
            .collect()
    }

    /// Count total blind spots
    pub fn total_blind_spots(&self) -> usize {
        self.blind_spots.len()
    }

    /// Count active blind spots
    pub fn active_count(&self) -> usize {
        self.active_blind_spots().len()
    }

    /// Count growth observations
    pub fn growth_count(&self) -> usize {
        self.growth_areas.len()
    }
}

impl Default for BlindSpotsGrowth {
    fn default() -> Self {
        Self {
            blind_spots: Vec::new(),
            growth_areas: Vec::new(),
            suggested_focus: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_timestamp() -> Timestamp {
        Timestamp::from_datetime(chrono::DateTime::from_timestamp(1704326400, 0).unwrap())
    }

    #[test]
    fn test_blind_spot_creation() {
        let ts = test_timestamp();
        let blind_spot = BlindSpot::new(
            "Underweights long-term".to_string(),
            "Focuses on 1-2 year horizon".to_string(),
            vec!["Evidence 1".to_string(), "Evidence 2".to_string()],
            "Prompt: What does this look like in 10 years?".to_string(),
            ts,
        );

        assert!(blind_spot.is_ok());
        let bs = blind_spot.unwrap();
        assert_eq!(bs.name, "Underweights long-term");
        assert!(bs.is_active());
        assert_eq!(bs.evidence.len(), 2);
    }

    #[test]
    fn test_blind_spot_validation() {
        let ts = test_timestamp();

        // Empty name
        assert!(BlindSpot::new(
            "".to_string(),
            "Description".to_string(),
            vec![],
            "Agent behavior".to_string(),
            ts
        )
        .is_err());

        // Empty description
        assert!(BlindSpot::new(
            "Name".to_string(),
            "".to_string(),
            vec![],
            "Agent behavior".to_string(),
            ts
        )
        .is_err());

        // Empty agent behavior
        assert!(BlindSpot::new(
            "Name".to_string(),
            "Description".to_string(),
            vec![],
            "".to_string(),
            ts
        )
        .is_err());
    }

    #[test]
    fn test_blind_spot_resolve() {
        let ts = test_timestamp();
        let mut blind_spot = BlindSpot::new(
            "Test".to_string(),
            "Description".to_string(),
            vec![],
            "Behavior".to_string(),
            ts,
        )
        .unwrap();

        assert!(blind_spot.is_active());

        blind_spot.resolve();
        assert!(!blind_spot.is_active());
    }

    #[test]
    fn test_growth_observation_creation() {
        let ts = test_timestamp();
        let growth = GrowthObservation::new(
            "Stakeholder consideration".to_string(),
            "Often forgot spouse input".to_string(),
            "Now automatic".to_string(),
            "Explicit prompt after job decision".to_string(),
            ts,
        );

        assert!(growth.is_ok());
        let g = growth.unwrap();
        assert_eq!(g.area, "Stakeholder consideration");
        assert_eq!(g.before_behavior, "Often forgot spouse input");
    }

    #[test]
    fn test_growth_observation_validation() {
        let ts = test_timestamp();

        // Empty area
        assert!(GrowthObservation::new(
            "".to_string(),
            "Before".to_string(),
            "After".to_string(),
            "Trigger".to_string(),
            ts
        )
        .is_err());

        // Empty behaviors
        assert!(GrowthObservation::new(
            "Area".to_string(),
            "".to_string(),
            "After".to_string(),
            "Trigger".to_string(),
            ts
        )
        .is_err());

        assert!(GrowthObservation::new(
            "Area".to_string(),
            "Before".to_string(),
            "".to_string(),
            "Trigger".to_string(),
            ts
        )
        .is_err());

        // Empty trigger
        assert!(GrowthObservation::new(
            "Area".to_string(),
            "Before".to_string(),
            "After".to_string(),
            "".to_string(),
            ts
        )
        .is_err());
    }

    #[test]
    fn test_blind_spots_growth_default() {
        let bg = BlindSpotsGrowth::default();
        assert_eq!(bg.total_blind_spots(), 0);
        assert_eq!(bg.growth_count(), 0);
        assert_eq!(bg.active_count(), 0);
    }

    #[test]
    fn test_blind_spots_growth_active_filtering() {
        let ts = test_timestamp();

        let mut bs1 = BlindSpot::new(
            "Active 1".to_string(),
            "Desc".to_string(),
            vec![],
            "Behavior".to_string(),
            ts,
        )
        .unwrap();

        let mut bs2 = BlindSpot::new(
            "Resolved".to_string(),
            "Desc".to_string(),
            vec![],
            "Behavior".to_string(),
            ts,
        )
        .unwrap();
        bs2.resolve();

        let bs3 = BlindSpot::new(
            "Active 2".to_string(),
            "Desc".to_string(),
            vec![],
            "Behavior".to_string(),
            ts,
        )
        .unwrap();

        let bg = BlindSpotsGrowth::new(vec![bs1, bs2, bs3], vec![], vec![]);

        assert_eq!(bg.total_blind_spots(), 3);
        assert_eq!(bg.active_count(), 2);

        let active = bg.active_blind_spots();
        assert_eq!(active.len(), 2);
        assert!(active.iter().any(|bs| bs.name == "Active 1"));
        assert!(active.iter().any(|bs| bs.name == "Active 2"));
    }

    #[test]
    fn test_blind_spots_growth_with_observations() {
        let ts = test_timestamp();

        let growth1 = GrowthObservation::new(
            "Area 1".to_string(),
            "Before".to_string(),
            "After".to_string(),
            "Trigger".to_string(),
            ts,
        )
        .unwrap();

        let growth2 = GrowthObservation::new(
            "Area 2".to_string(),
            "Before".to_string(),
            "After".to_string(),
            "Trigger".to_string(),
            ts,
        )
        .unwrap();

        let bg = BlindSpotsGrowth::new(vec![], vec![growth1, growth2], vec![]);

        assert_eq!(bg.growth_count(), 2);
    }
}
