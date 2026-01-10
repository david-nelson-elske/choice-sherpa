//! Objectives component - fundamental and means objectives with measures.

use serde::{Deserialize, Serialize};

use crate::domain::foundation::{ComponentId, ComponentStatus, ComponentType, Timestamp};

use super::{Component, ComponentBase, ComponentError};

/// How to measure achievement of an objective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMeasure {
    pub description: String,
    pub is_quantitative: bool,
    /// Unit of measurement (e.g., "dollars", "days").
    pub unit: Option<String>,
    /// Direction: "higher_is_better" or "lower_is_better".
    pub direction: String,
}

/// A fundamental objective - what we ultimately care about.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundamentalObjective {
    pub id: String,
    pub description: String,
    pub performance_measure: PerformanceMeasure,
    /// Links to Party.id from ProblemFrame.
    pub affected_party_id: Option<String>,
}

/// A means objective - a way to achieve fundamental objectives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeansObjective {
    pub id: String,
    pub description: String,
    /// Which fundamental objective this supports.
    pub contributes_to_objective_id: String,
}

/// Objectives output structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObjectivesOutput {
    pub fundamental_objectives: Vec<FundamentalObjective>,
    pub means_objectives: Vec<MeansObjective>,
}

/// The Objectives component.
#[derive(Debug, Clone)]
pub struct Objectives {
    base: ComponentBase,
    output: ObjectivesOutput,
}

impl Objectives {
    /// Creates a new Objectives component.
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(ComponentType::Objectives),
            output: ObjectivesOutput::default(),
        }
    }

    /// Reconstitutes an Objectives component from persisted data.
    pub(crate) fn reconstitute(base: ComponentBase, output: ObjectivesOutput) -> Self {
        Self { base, output }
    }

    /// Returns the output.
    pub fn output(&self) -> &ObjectivesOutput {
        &self.output
    }

    /// Sets the output.
    pub fn set_output(&mut self, output: ObjectivesOutput) {
        self.output = output;
        self.base.touch();
    }

    /// Adds a fundamental objective.
    pub fn add_fundamental(&mut self, objective: FundamentalObjective) {
        self.output.fundamental_objectives.push(objective);
        self.base.touch();
    }

    /// Adds a means objective.
    pub fn add_means(&mut self, objective: MeansObjective) {
        self.output.means_objectives.push(objective);
        self.base.touch();
    }

    /// Returns the count of fundamental objectives.
    pub fn fundamental_count(&self) -> usize {
        self.output.fundamental_objectives.len()
    }

    /// Returns the count of means objectives.
    pub fn means_count(&self) -> usize {
        self.output.means_objectives.len()
    }

    /// Finds a fundamental objective by ID.
    pub fn find_fundamental(&self, id: &str) -> Option<&FundamentalObjective> {
        self.output.fundamental_objectives.iter().find(|o| o.id == id)
    }
}

impl Default for Objectives {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Objectives {
    fn id(&self) -> ComponentId {
        self.base.id
    }

    fn component_type(&self) -> ComponentType {
        self.base.component_type
    }

    fn status(&self) -> ComponentStatus {
        self.base.status
    }

    fn created_at(&self) -> Timestamp {
        self.base.created_at
    }

    fn updated_at(&self) -> Timestamp {
        self.base.updated_at
    }

    fn start(&mut self) -> Result<(), ComponentError> {
        self.base.start()
    }

    fn complete(&mut self) -> Result<(), ComponentError> {
        self.base.complete()
    }

    fn mark_for_revision(&mut self, reason: String) -> Result<(), ComponentError> {
        self.base.mark_for_revision(reason)
    }

    fn output_as_value(&self) -> serde_json::Value {
        serde_json::to_value(&self.output).unwrap_or_default()
    }

    fn set_output_from_value(&mut self, value: serde_json::Value) -> Result<(), ComponentError> {
        self.output = serde_json::from_value(value)
            .map_err(|e| ComponentError::InvalidOutput(e.to_string()))?;
        self.base.touch();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn objectives_has_correct_component_type() {
        let obj = Objectives::new();
        assert_eq!(obj.component_type(), ComponentType::Objectives);
    }

    #[test]
    fn add_fundamental_increases_count() {
        let mut obj = Objectives::new();
        assert_eq!(obj.fundamental_count(), 0);

        let fundamental = FundamentalObjective {
            id: "f1".to_string(),
            description: "Maximize profit".to_string(),
            performance_measure: PerformanceMeasure {
                description: "Annual revenue".to_string(),
                is_quantitative: true,
                unit: Some("dollars".to_string()),
                direction: "higher_is_better".to_string(),
            },
            affected_party_id: None,
        };
        obj.add_fundamental(fundamental);

        assert_eq!(obj.fundamental_count(), 1);
    }

    #[test]
    fn add_means_increases_count() {
        let mut obj = Objectives::new();
        let means = MeansObjective {
            id: "m1".to_string(),
            description: "Reduce costs".to_string(),
            contributes_to_objective_id: "f1".to_string(),
        };
        obj.add_means(means);

        assert_eq!(obj.means_count(), 1);
    }

    #[test]
    fn find_fundamental_returns_objective() {
        let mut obj = Objectives::new();
        let fundamental = FundamentalObjective {
            id: "f1".to_string(),
            description: "Test objective".to_string(),
            performance_measure: PerformanceMeasure {
                description: "Test measure".to_string(),
                is_quantitative: false,
                unit: None,
                direction: "higher_is_better".to_string(),
            },
            affected_party_id: None,
        };
        obj.add_fundamental(fundamental);

        let found = obj.find_fundamental("f1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().description, "Test objective");
    }

    #[test]
    fn find_fundamental_returns_none_if_not_found() {
        let obj = Objectives::new();
        assert!(obj.find_fundamental("nonexistent").is_none());
    }

    #[test]
    fn output_roundtrips_through_json() {
        let mut obj = Objectives::new();
        obj.add_fundamental(FundamentalObjective {
            id: "f1".to_string(),
            description: "Test".to_string(),
            performance_measure: PerformanceMeasure {
                description: "Measure".to_string(),
                is_quantitative: true,
                unit: Some("units".to_string()),
                direction: "lower_is_better".to_string(),
            },
            affected_party_id: Some("p1".to_string()),
        });

        let value = obj.output_as_value();
        let mut obj2 = Objectives::new();
        obj2.set_output_from_value(value).unwrap();

        assert_eq!(obj.fundamental_count(), obj2.fundamental_count());
    }
}
