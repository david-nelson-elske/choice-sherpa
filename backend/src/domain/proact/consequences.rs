//! Consequences component - consequences table with Pugh ratings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::foundation::{ComponentId, ComponentStatus, ComponentType, Rating, Timestamp};

use super::{Component, ComponentBase, ComponentError};

/// A cell in the consequences table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    /// Pugh rating: -2 to +2.
    pub rating: Rating,
    /// Explanation of the rating.
    pub explanation: String,
    /// Quantitative value if available.
    pub quant_value: Option<f64>,
    /// Unit for quantitative value.
    pub quant_unit: Option<String>,
    /// Source/citation.
    pub source: Option<String>,
    /// Flag for uncertain values.
    pub uncertainty: Option<String>,
}

impl Cell {
    /// Creates a new cell with a rating and explanation.
    pub fn new(rating: Rating, explanation: impl Into<String>) -> Self {
        Self {
            rating,
            explanation: explanation.into(),
            quant_value: None,
            quant_unit: None,
            source: None,
            uncertainty: None,
        }
    }

    /// Adds a quantitative value to the cell.
    pub fn with_quantitative(mut self, value: f64, unit: impl Into<String>) -> Self {
        self.quant_value = Some(value);
        self.quant_unit = Some(unit.into());
        self
    }

    /// Adds a source citation.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Marks the cell as uncertain.
    pub fn with_uncertainty(mut self, note: impl Into<String>) -> Self {
        self.uncertainty = Some(note.into());
        self
    }
}

/// An uncertainty that affects consequences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Uncertainty {
    pub id: String,
    pub description: String,
    /// What causes this uncertainty.
    pub driver: String,
    /// Is it worth spending resources to resolve?
    pub worth_resolving: bool,
    /// Can it be reduced within the decision timeframe?
    pub resolvable: bool,
}

/// The consequences table structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsequencesTable {
    /// Column identifiers (alternative IDs).
    pub alternative_ids: Vec<String>,
    /// Row identifiers (objective IDs).
    pub objective_ids: Vec<String>,
    /// Cell data: cells[alt_id][obj_id].
    pub cells: HashMap<String, HashMap<String, Cell>>,
}

/// Consequences output structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsequencesOutput {
    pub table: ConsequencesTable,
    pub uncertainties: Vec<Uncertainty>,
}

/// The Consequences component.
#[derive(Debug, Clone)]
pub struct Consequences {
    base: ComponentBase,
    output: ConsequencesOutput,
}

impl Consequences {
    /// Creates a new Consequences component.
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(ComponentType::Consequences),
            output: ConsequencesOutput::default(),
        }
    }

    /// Reconstitutes a Consequences component from persisted data.
    pub(crate) fn reconstitute(base: ComponentBase, output: ConsequencesOutput) -> Self {
        Self { base, output }
    }

    /// Returns the output.
    pub fn output(&self) -> &ConsequencesOutput {
        &self.output
    }

    /// Sets the output.
    pub fn set_output(&mut self, output: ConsequencesOutput) {
        self.output = output;
        self.base.touch();
    }

    /// Sets the alternative IDs for the table.
    pub fn set_alternative_ids(&mut self, ids: Vec<String>) {
        self.output.table.alternative_ids = ids;
        self.base.touch();
    }

    /// Sets the objective IDs for the table.
    pub fn set_objective_ids(&mut self, ids: Vec<String>) {
        self.output.table.objective_ids = ids;
        self.base.touch();
    }

    /// Sets a cell in the consequences table.
    pub fn set_cell(&mut self, alt_id: &str, obj_id: &str, cell: Cell) {
        self.output
            .table
            .cells
            .entry(alt_id.to_string())
            .or_default()
            .insert(obj_id.to_string(), cell);
        self.base.touch();
    }

    /// Gets a cell from the consequences table.
    pub fn get_cell(&self, alt_id: &str, obj_id: &str) -> Option<&Cell> {
        self.output
            .table
            .cells
            .get(alt_id)
            .and_then(|row| row.get(obj_id))
    }

    /// Adds an uncertainty.
    pub fn add_uncertainty(&mut self, uncertainty: Uncertainty) {
        self.output.uncertainties.push(uncertainty);
        self.base.touch();
    }

    /// Returns the count of filled cells.
    pub fn cell_count(&self) -> usize {
        self.output
            .table
            .cells
            .values()
            .map(|row| row.len())
            .sum()
    }

    /// Returns true if all cells are filled.
    pub fn is_complete(&self) -> bool {
        let expected = self.output.table.alternative_ids.len()
            * self.output.table.objective_ids.len();
        expected > 0 && self.cell_count() == expected
    }
}

impl Default for Consequences {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Consequences {
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
    fn consequences_has_correct_component_type() {
        let con = Consequences::new();
        assert_eq!(con.component_type(), ComponentType::Consequences);
    }

    #[test]
    fn set_cell_adds_to_table() {
        let mut con = Consequences::new();
        con.set_alternative_ids(vec!["a1".to_string()]);
        con.set_objective_ids(vec!["o1".to_string()]);

        let cell = Cell::new(Rating::Better, "Good for this objective");
        con.set_cell("a1", "o1", cell);

        assert_eq!(con.cell_count(), 1);
    }

    #[test]
    fn get_cell_retrieves_cell() {
        let mut con = Consequences::new();
        let cell = Cell::new(Rating::MuchBetter, "Excellent");
        con.set_cell("a1", "o1", cell);

        let retrieved = con.get_cell("a1", "o1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().rating, Rating::MuchBetter);
    }

    #[test]
    fn get_cell_returns_none_if_missing() {
        let con = Consequences::new();
        assert!(con.get_cell("a1", "o1").is_none());
    }

    #[test]
    fn cell_with_quantitative_sets_values() {
        let cell = Cell::new(Rating::Same, "Neutral")
            .with_quantitative(1000.0, "dollars");

        assert_eq!(cell.quant_value, Some(1000.0));
        assert_eq!(cell.quant_unit, Some("dollars".to_string()));
    }

    #[test]
    fn cell_with_source_sets_citation() {
        let cell = Cell::new(Rating::Better, "Good")
            .with_source("Market research 2024");

        assert_eq!(cell.source, Some("Market research 2024".to_string()));
    }

    #[test]
    fn cell_with_uncertainty_sets_flag() {
        let cell = Cell::new(Rating::Worse, "Bad")
            .with_uncertainty("Market conditions may change");

        assert!(cell.uncertainty.is_some());
    }

    #[test]
    fn is_complete_returns_false_when_empty() {
        let mut con = Consequences::new();
        con.set_alternative_ids(vec!["a1".to_string()]);
        con.set_objective_ids(vec!["o1".to_string()]);

        assert!(!con.is_complete());
    }

    #[test]
    fn is_complete_returns_true_when_all_filled() {
        let mut con = Consequences::new();
        con.set_alternative_ids(vec!["a1".to_string(), "a2".to_string()]);
        con.set_objective_ids(vec!["o1".to_string()]);

        con.set_cell("a1", "o1", Cell::new(Rating::Same, ""));
        con.set_cell("a2", "o1", Cell::new(Rating::Better, ""));

        assert!(con.is_complete());
    }

    #[test]
    fn add_uncertainty_adds_to_list() {
        let mut con = Consequences::new();
        let uncertainty = Uncertainty {
            id: "u1".to_string(),
            description: "Market volatility".to_string(),
            driver: "Economic conditions".to_string(),
            worth_resolving: true,
            resolvable: false,
        };
        con.add_uncertainty(uncertainty);

        assert_eq!(con.output().uncertainties.len(), 1);
    }

    #[test]
    fn output_roundtrips_through_json() {
        let mut con = Consequences::new();
        con.set_alternative_ids(vec!["a1".to_string()]);
        con.set_objective_ids(vec!["o1".to_string()]);
        con.set_cell("a1", "o1", Cell::new(Rating::Better, "Test"));

        let value = con.output_as_value();
        let mut con2 = Consequences::new();
        con2.set_output_from_value(value).unwrap();

        assert_eq!(con.cell_count(), con2.cell_count());
    }
}
