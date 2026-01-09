//! Consequences Table - Core data structure for Pugh matrix analysis.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::foundation::Rating;

/// A cell in the consequences table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub alternative_id: String,
    pub objective_id: String,
    pub rating: Rating,
    pub rationale: Option<String>,
}

impl Cell {
    /// Creates a new cell with a rating.
    pub fn new(alternative_id: impl Into<String>, objective_id: impl Into<String>, rating: Rating) -> Self {
        Self {
            alternative_id: alternative_id.into(),
            objective_id: objective_id.into(),
            rating,
            rationale: None,
        }
    }

    /// Creates a cell with rationale.
    pub fn with_rationale(
        alternative_id: impl Into<String>,
        objective_id: impl Into<String>,
        rating: Rating,
        rationale: impl Into<String>,
    ) -> Self {
        Self {
            alternative_id: alternative_id.into(),
            objective_id: objective_id.into(),
            rating,
            rationale: Some(rationale.into()),
        }
    }
}

/// The consequences table mapping alternatives x objectives to ratings.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsequencesTable {
    /// Ordered list of alternative IDs.
    pub alternative_ids: Vec<String>,
    /// Ordered list of objective IDs.
    pub objective_ids: Vec<String>,
    /// Cell data keyed by "alt_id:obj_id".
    pub cells: HashMap<String, Cell>,
}

impl ConsequencesTable {
    /// Creates an empty consequences table.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Creates a builder for constructing a consequences table.
    pub fn builder() -> ConsequencesTableBuilder {
        ConsequencesTableBuilder::new()
    }

    /// Gets a cell by alternative and objective IDs.
    pub fn get_cell(&self, alternative_id: &str, objective_id: &str) -> Option<&Cell> {
        let key = Self::cell_key(alternative_id, objective_id);
        self.cells.get(&key)
    }

    /// Generates the cell key from alternative and objective IDs.
    fn cell_key(alternative_id: &str, objective_id: &str) -> String {
        format!("{}:{}", alternative_id, objective_id)
    }

    /// Returns true if the table has no alternatives.
    pub fn is_empty(&self) -> bool {
        self.alternative_ids.is_empty()
    }

    /// Returns the number of alternatives.
    pub fn alternative_count(&self) -> usize {
        self.alternative_ids.len()
    }

    /// Returns the number of objectives.
    pub fn objective_count(&self) -> usize {
        self.objective_ids.len()
    }
}

/// Builder for constructing ConsequencesTable instances.
#[derive(Debug, Default)]
pub struct ConsequencesTableBuilder {
    alternative_ids: Vec<String>,
    objective_ids: Vec<String>,
    cells: HashMap<String, Cell>,
}

impl ConsequencesTableBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the alternatives.
    pub fn alternatives(mut self, ids: Vec<impl Into<String>>) -> Self {
        self.alternative_ids = ids.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Sets the objectives.
    pub fn objectives(mut self, ids: Vec<impl Into<String>>) -> Self {
        self.objective_ids = ids.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Adds a cell with a rating.
    pub fn cell(
        mut self,
        alternative_id: impl Into<String>,
        objective_id: impl Into<String>,
        rating: Rating,
    ) -> Self {
        let alt_id = alternative_id.into();
        let obj_id = objective_id.into();
        let key = format!("{}:{}", alt_id, obj_id);
        self.cells.insert(key, Cell::new(alt_id, obj_id, rating));
        self
    }

    /// Adds a cell with rating and rationale.
    pub fn cell_with_rationale(
        mut self,
        alternative_id: impl Into<String>,
        objective_id: impl Into<String>,
        rating: Rating,
        rationale: impl Into<String>,
    ) -> Self {
        let alt_id = alternative_id.into();
        let obj_id = objective_id.into();
        let key = format!("{}:{}", alt_id, obj_id);
        self.cells.insert(
            key,
            Cell::with_rationale(alt_id, obj_id, rating, rationale),
        );
        self
    }

    /// Builds the consequences table.
    pub fn build(self) -> ConsequencesTable {
        ConsequencesTable {
            alternative_ids: self.alternative_ids,
            objective_ids: self.objective_ids,
            cells: self.cells,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_table_has_no_alternatives() {
        let table = ConsequencesTable::empty();
        assert!(table.is_empty());
        assert_eq!(table.alternative_count(), 0);
        assert_eq!(table.objective_count(), 0);
    }

    #[test]
    fn builder_creates_table_with_alternatives() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A", "B", "C"])
            .objectives(vec!["O1", "O2"])
            .build();

        assert_eq!(table.alternative_count(), 3);
        assert_eq!(table.objective_count(), 2);
    }

    #[test]
    fn builder_adds_cells() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A", "B"])
            .objectives(vec!["O1", "O2"])
            .cell("A", "O1", Rating::Better)
            .cell("A", "O2", Rating::Worse)
            .cell("B", "O1", Rating::Same)
            .cell("B", "O2", Rating::MuchBetter)
            .build();

        assert_eq!(table.get_cell("A", "O1").unwrap().rating, Rating::Better);
        assert_eq!(table.get_cell("A", "O2").unwrap().rating, Rating::Worse);
        assert_eq!(table.get_cell("B", "O1").unwrap().rating, Rating::Same);
        assert_eq!(table.get_cell("B", "O2").unwrap().rating, Rating::MuchBetter);
    }

    #[test]
    fn get_cell_returns_none_for_missing() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A"])
            .objectives(vec!["O1"])
            .build();

        assert!(table.get_cell("A", "O1").is_none());
        assert!(table.get_cell("B", "O1").is_none());
    }

    #[test]
    fn cell_with_rationale_stores_rationale() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A"])
            .objectives(vec!["O1"])
            .cell_with_rationale("A", "O1", Rating::Better, "Cost savings")
            .build();

        let cell = table.get_cell("A", "O1").unwrap();
        assert_eq!(cell.rating, Rating::Better);
        assert_eq!(cell.rationale.as_deref(), Some("Cost savings"));
    }

    #[test]
    fn table_serializes_to_json() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A"])
            .objectives(vec!["O1"])
            .cell("A", "O1", Rating::Better)
            .build();

        let json = serde_json::to_string(&table).unwrap();
        assert!(json.contains("alternative_ids"));
        assert!(json.contains("objective_ids"));
    }

    #[test]
    fn table_deserializes_from_json() {
        let json = r#"{
            "alternative_ids": ["A", "B"],
            "objective_ids": ["O1"],
            "cells": {
                "A:O1": {
                    "alternative_id": "A",
                    "objective_id": "O1",
                    "rating": "Better",
                    "rationale": null
                }
            }
        }"#;

        let table: ConsequencesTable = serde_json::from_str(json).unwrap();
        assert_eq!(table.alternative_count(), 2);
        assert_eq!(table.get_cell("A", "O1").unwrap().rating, Rating::Better);
    }
}
