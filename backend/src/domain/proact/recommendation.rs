//! Recommendation component - synthesis of analysis.

use serde::{Deserialize, Serialize};

use crate::domain::foundation::{ComponentId, ComponentStatus, ComponentType, Timestamp};

use super::{Component, ComponentBase, ComponentError};

/// Recommendation output structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecommendationOutput {
    /// AlternativeID if one stands out.
    pub standout_option: Option<String>,
    /// Summary of the analysis.
    pub synthesis: String,
    /// Important qualifications.
    pub caveats: Vec<String>,
    /// What additional information might help.
    pub additional_info: Vec<String>,
}

/// The Recommendation component.
#[derive(Debug, Clone)]
pub struct Recommendation {
    base: ComponentBase,
    output: RecommendationOutput,
}

impl Recommendation {
    /// Creates a new Recommendation component.
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(ComponentType::Recommendation),
            output: RecommendationOutput::default(),
        }
    }

    /// Returns the output.
    pub fn output(&self) -> &RecommendationOutput {
        &self.output
    }

    /// Sets the output.
    pub fn set_output(&mut self, output: RecommendationOutput) {
        self.output = output;
        self.base.touch();
    }

    /// Sets the synthesis.
    pub fn set_synthesis(&mut self, synthesis: String) {
        self.output.synthesis = synthesis;
        self.base.touch();
    }

    /// Sets the standout option.
    pub fn set_standout(&mut self, alternative_id: String) {
        self.output.standout_option = Some(alternative_id);
        self.base.touch();
    }

    /// Clears the standout option.
    pub fn clear_standout(&mut self) {
        self.output.standout_option = None;
        self.base.touch();
    }

    /// Adds a caveat.
    pub fn add_caveat(&mut self, caveat: String) {
        self.output.caveats.push(caveat);
        self.base.touch();
    }

    /// Adds additional info need.
    pub fn add_additional_info(&mut self, info: String) {
        self.output.additional_info.push(info);
        self.base.touch();
    }

    /// Returns true if there's a standout option.
    pub fn has_standout(&self) -> bool {
        self.output.standout_option.is_some()
    }

    /// Returns the number of caveats.
    pub fn caveat_count(&self) -> usize {
        self.output.caveats.len()
    }
}

impl Default for Recommendation {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Recommendation {
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
    fn recommendation_has_correct_component_type() {
        let rec = Recommendation::new();
        assert_eq!(rec.component_type(), ComponentType::Recommendation);
    }

    #[test]
    fn set_synthesis_updates_output() {
        let mut rec = Recommendation::new();
        rec.set_synthesis("Based on the analysis, Option A is preferred.".to_string());

        assert!(!rec.output().synthesis.is_empty());
    }

    #[test]
    fn set_standout_updates_output() {
        let mut rec = Recommendation::new();
        assert!(!rec.has_standout());

        rec.set_standout("a1".to_string());

        assert!(rec.has_standout());
        assert_eq!(rec.output().standout_option, Some("a1".to_string()));
    }

    #[test]
    fn clear_standout_removes_standout() {
        let mut rec = Recommendation::new();
        rec.set_standout("a1".to_string());
        assert!(rec.has_standout());

        rec.clear_standout();
        assert!(!rec.has_standout());
    }

    #[test]
    fn add_caveat_adds_to_list() {
        let mut rec = Recommendation::new();
        rec.add_caveat("Market conditions may change".to_string());
        rec.add_caveat("Depends on competitor response".to_string());

        assert_eq!(rec.caveat_count(), 2);
    }

    #[test]
    fn add_additional_info_adds_to_list() {
        let mut rec = Recommendation::new();
        rec.add_additional_info("Need cost estimates".to_string());

        assert_eq!(rec.output().additional_info.len(), 1);
    }

    #[test]
    fn output_roundtrips_through_json() {
        let mut rec = Recommendation::new();
        rec.set_synthesis("Test synthesis".to_string());
        rec.set_standout("a1".to_string());
        rec.add_caveat("Caveat 1".to_string());

        let value = rec.output_as_value();
        let mut rec2 = Recommendation::new();
        rec2.set_output_from_value(value).unwrap();

        assert_eq!(rec.output().synthesis, rec2.output().synthesis);
        assert_eq!(rec.output().standout_option, rec2.output().standout_option);
        assert_eq!(rec.caveat_count(), rec2.caveat_count());
    }
}
