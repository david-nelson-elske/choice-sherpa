//! DecisionQuality component - 7 elements rated 0-100%.

use serde::{Deserialize, Serialize};

use crate::domain::foundation::{ComponentId, ComponentStatus, ComponentType, Percentage, Timestamp};

use super::{Component, ComponentBase, ComponentError};

/// The 7 standard Decision Quality element names.
pub const DQ_ELEMENT_NAMES: &[&str] = &[
    "Helpful Problem Frame",
    "Clear Objectives",
    "Creative Alternatives",
    "Reliable Consequence Information",
    "Logically Correct Reasoning",
    "Clear Tradeoffs",
    "Commitment to Follow Through",
];

/// A single Decision Quality element rating.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DQElement {
    /// One of the 7 standard element names.
    pub name: String,
    /// Score from 0-100.
    pub score: Percentage,
    /// Why this score was given.
    pub rationale: String,
    /// What would improve this element.
    pub improvement: String,
}

/// DecisionQuality output structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionQualityOutput {
    pub elements: Vec<DQElement>,
    /// Minimum of all element scores.
    pub overall_score: Percentage,
    /// What would raise the lowest scores.
    pub improvement_paths: Vec<String>,
}

/// The DecisionQuality component.
#[derive(Debug, Clone)]
pub struct DecisionQuality {
    base: ComponentBase,
    output: DecisionQualityOutput,
}

impl DecisionQuality {
    /// Creates a new DecisionQuality component.
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(ComponentType::DecisionQuality),
            output: DecisionQualityOutput::default(),
        }
    }

    /// Reconstitutes a DecisionQuality component from persisted data.
    pub(crate) fn reconstitute(base: ComponentBase, output: DecisionQualityOutput) -> Self {
        Self { base, output }
    }

    /// Returns the output.
    pub fn output(&self) -> &DecisionQualityOutput {
        &self.output
    }

    /// Sets the output.
    pub fn set_output(&mut self, output: DecisionQualityOutput) {
        self.output = output;
        self.base.touch();
    }

    /// Sets or updates an element.
    pub fn set_element(&mut self, element: DQElement) {
        // Replace if exists, otherwise add
        if let Some(existing) = self.output.elements.iter_mut().find(|e| e.name == element.name) {
            *existing = element;
        } else {
            self.output.elements.push(element);
        }
        self.recalculate_overall();
        self.base.touch();
    }

    /// Recalculates the overall score as minimum of all elements.
    pub fn recalculate_overall(&mut self) {
        if self.output.elements.is_empty() {
            self.output.overall_score = Percentage::ZERO;
        } else {
            let min = self
                .output
                .elements
                .iter()
                .map(|e| e.score.value())
                .min()
                .unwrap_or(0);
            self.output.overall_score = Percentage::new(min);
        }
    }

    /// Adds an improvement path.
    pub fn add_improvement_path(&mut self, path: String) {
        self.output.improvement_paths.push(path);
        self.base.touch();
    }

    /// Returns true if all elements score 100%.
    pub fn is_perfect(&self) -> bool {
        self.output.overall_score.value() == 100
    }

    /// Returns the weakest element (lowest score).
    pub fn weakest_element(&self) -> Option<&DQElement> {
        self.output.elements.iter().min_by_key(|e| e.score.value())
    }

    /// Returns the count of rated elements.
    pub fn element_count(&self) -> usize {
        self.output.elements.len()
    }

    /// Returns true if all 7 standard elements have been rated.
    pub fn is_complete(&self) -> bool {
        DQ_ELEMENT_NAMES.iter().all(|name| {
            self.output.elements.iter().any(|e| e.name == *name)
        })
    }

    /// Returns the average score across all elements.
    pub fn average_score(&self) -> Percentage {
        if self.output.elements.is_empty() {
            return Percentage::ZERO;
        }
        let sum: u32 = self.output.elements.iter().map(|e| e.score.value() as u32).sum();
        let avg = sum / self.output.elements.len() as u32;
        Percentage::new(avg as u8)
    }
}

impl Default for DecisionQuality {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for DecisionQuality {
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
    fn decision_quality_has_correct_component_type() {
        let dq = DecisionQuality::new();
        assert_eq!(dq.component_type(), ComponentType::DecisionQuality);
    }

    #[test]
    fn dq_element_names_has_7_elements() {
        assert_eq!(DQ_ELEMENT_NAMES.len(), 7);
    }

    #[test]
    fn overall_score_is_minimum() {
        let mut dq = DecisionQuality::new();
        dq.set_element(DQElement {
            name: "Helpful Problem Frame".to_string(),
            score: Percentage::new(80),
            rationale: "Good".to_string(),
            improvement: "More detail".to_string(),
        });
        dq.set_element(DQElement {
            name: "Clear Objectives".to_string(),
            score: Percentage::new(60),
            rationale: "OK".to_string(),
            improvement: "Add measures".to_string(),
        });
        dq.set_element(DQElement {
            name: "Creative Alternatives".to_string(),
            score: Percentage::new(90),
            rationale: "Excellent".to_string(),
            improvement: "None needed".to_string(),
        });

        assert_eq!(dq.output().overall_score.value(), 60);
    }

    #[test]
    fn set_element_updates_existing() {
        let mut dq = DecisionQuality::new();
        dq.set_element(DQElement {
            name: "Clear Objectives".to_string(),
            score: Percentage::new(50),
            rationale: "".to_string(),
            improvement: "".to_string(),
        });
        dq.set_element(DQElement {
            name: "Clear Objectives".to_string(),
            score: Percentage::new(75),
            rationale: "Improved".to_string(),
            improvement: "".to_string(),
        });

        assert_eq!(dq.element_count(), 1);
        assert_eq!(dq.output().elements[0].score.value(), 75);
    }

    #[test]
    fn weakest_element_returns_lowest_score() {
        let mut dq = DecisionQuality::new();
        dq.set_element(DQElement {
            name: "Element A".to_string(),
            score: Percentage::new(80),
            rationale: "".to_string(),
            improvement: "".to_string(),
        });
        dq.set_element(DQElement {
            name: "Element B".to_string(),
            score: Percentage::new(40),
            rationale: "".to_string(),
            improvement: "".to_string(),
        });

        let weakest = dq.weakest_element().unwrap();
        assert_eq!(weakest.name, "Element B");
        assert_eq!(weakest.score.value(), 40);
    }

    #[test]
    fn is_perfect_returns_true_when_all_100() {
        let mut dq = DecisionQuality::new();
        dq.set_element(DQElement {
            name: "Test".to_string(),
            score: Percentage::new(100),
            rationale: "".to_string(),
            improvement: "".to_string(),
        });

        assert!(dq.is_perfect());
    }

    #[test]
    fn is_perfect_returns_false_when_below_100() {
        let mut dq = DecisionQuality::new();
        dq.set_element(DQElement {
            name: "Test".to_string(),
            score: Percentage::new(99),
            rationale: "".to_string(),
            improvement: "".to_string(),
        });

        assert!(!dq.is_perfect());
    }

    #[test]
    fn is_complete_checks_all_7_elements() {
        let mut dq = DecisionQuality::new();
        assert!(!dq.is_complete());

        for name in DQ_ELEMENT_NAMES {
            dq.set_element(DQElement {
                name: name.to_string(),
                score: Percentage::new(75),
                rationale: "".to_string(),
                improvement: "".to_string(),
            });
        }

        assert!(dq.is_complete());
    }

    #[test]
    fn average_score_calculates_correctly() {
        let mut dq = DecisionQuality::new();
        dq.set_element(DQElement {
            name: "A".to_string(),
            score: Percentage::new(60),
            rationale: "".to_string(),
            improvement: "".to_string(),
        });
        dq.set_element(DQElement {
            name: "B".to_string(),
            score: Percentage::new(80),
            rationale: "".to_string(),
            improvement: "".to_string(),
        });

        assert_eq!(dq.average_score().value(), 70);
    }

    #[test]
    fn output_roundtrips_through_json() {
        let mut dq = DecisionQuality::new();
        dq.set_element(DQElement {
            name: "Helpful Problem Frame".to_string(),
            score: Percentage::new(85),
            rationale: "Well defined".to_string(),
            improvement: "Add constraints".to_string(),
        });
        dq.add_improvement_path("Focus on problem frame".to_string());

        let value = dq.output_as_value();
        let mut dq2 = DecisionQuality::new();
        dq2.set_output_from_value(value).unwrap();

        assert_eq!(dq.element_count(), dq2.element_count());
        assert_eq!(
            dq.output().overall_score.value(),
            dq2.output().overall_score.value()
        );
    }
}
