//! Alternatives component - options and strategy tables.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::foundation::{ComponentId, ComponentStatus, ComponentType, Timestamp};

use super::{Component, ComponentBase, ComponentError};

/// A single alternative/option.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub id: String,
    pub name: String,
    pub description: String,
    pub assumptions: Vec<String>,
    pub is_status_quo: bool,
}

/// Column in a strategy table representing one decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionColumn {
    pub decision_name: String,
    pub options: Vec<String>,
}

/// A strategy combining choices across multiple decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub id: String,
    pub name: String,
    /// Maps decision_name -> chosen option.
    pub choices: HashMap<String, String>,
}

/// Strategy table for multiple focal decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyTable {
    pub decisions: Vec<DecisionColumn>,
    pub strategies: Vec<Strategy>,
}

/// Alternatives output structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlternativesOutput {
    pub options: Vec<Alternative>,
    /// Strategy table for multiple focal decisions.
    pub strategy_table: Option<StrategyTable>,
    pub has_status_quo: bool,
}

/// The Alternatives component.
#[derive(Debug, Clone)]
pub struct Alternatives {
    base: ComponentBase,
    output: AlternativesOutput,
}

impl Alternatives {
    /// Creates a new Alternatives component.
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(ComponentType::Alternatives),
            output: AlternativesOutput::default(),
        }
    }

    /// Reconstitutes an Alternatives component from persisted data.
    pub(crate) fn reconstitute(base: ComponentBase, output: AlternativesOutput) -> Self {
        Self { base, output }
    }

    /// Returns the output.
    pub fn output(&self) -> &AlternativesOutput {
        &self.output
    }

    /// Sets the output.
    pub fn set_output(&mut self, output: AlternativesOutput) {
        self.output = output;
        self.base.touch();
    }

    /// Adds an alternative.
    pub fn add_alternative(&mut self, alt: Alternative) {
        if alt.is_status_quo {
            self.output.has_status_quo = true;
        }
        self.output.options.push(alt);
        self.base.touch();
    }

    /// Sets the strategy table.
    pub fn set_strategy_table(&mut self, table: StrategyTable) {
        self.output.strategy_table = Some(table);
        self.base.touch();
    }

    /// Returns the count of alternatives.
    pub fn alternatives_count(&self) -> usize {
        self.output.options.len()
    }

    /// Returns true if there is a status quo alternative.
    pub fn has_status_quo(&self) -> bool {
        self.output.has_status_quo
    }

    /// Finds an alternative by ID.
    pub fn find_alternative(&self, id: &str) -> Option<&Alternative> {
        self.output.options.iter().find(|a| a.id == id)
    }

    /// Returns IDs of all alternatives.
    pub fn alternative_ids(&self) -> Vec<&str> {
        self.output.options.iter().map(|a| a.id.as_str()).collect()
    }
}

impl Default for Alternatives {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Alternatives {
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
    fn alternatives_has_correct_component_type() {
        let alt = Alternatives::new();
        assert_eq!(alt.component_type(), ComponentType::Alternatives);
    }

    #[test]
    fn add_alternative_increases_count() {
        let mut alt = Alternatives::new();
        assert_eq!(alt.alternatives_count(), 0);

        let option = Alternative {
            id: "a1".to_string(),
            name: "Option A".to_string(),
            description: "First option".to_string(),
            assumptions: vec!["Assumption 1".to_string()],
            is_status_quo: false,
        };
        alt.add_alternative(option);

        assert_eq!(alt.alternatives_count(), 1);
    }

    #[test]
    fn add_status_quo_sets_flag() {
        let mut alt = Alternatives::new();
        assert!(!alt.has_status_quo());

        let status_quo = Alternative {
            id: "sq".to_string(),
            name: "Do nothing".to_string(),
            description: "Maintain current state".to_string(),
            assumptions: vec![],
            is_status_quo: true,
        };
        alt.add_alternative(status_quo);

        assert!(alt.has_status_quo());
    }

    #[test]
    fn find_alternative_returns_option() {
        let mut alt = Alternatives::new();
        alt.add_alternative(Alternative {
            id: "a1".to_string(),
            name: "Option A".to_string(),
            description: "Test".to_string(),
            assumptions: vec![],
            is_status_quo: false,
        });

        let found = alt.find_alternative("a1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Option A");
    }

    #[test]
    fn alternative_ids_returns_all_ids() {
        let mut alt = Alternatives::new();
        alt.add_alternative(Alternative {
            id: "a1".to_string(),
            name: "A".to_string(),
            description: "".to_string(),
            assumptions: vec![],
            is_status_quo: false,
        });
        alt.add_alternative(Alternative {
            id: "a2".to_string(),
            name: "B".to_string(),
            description: "".to_string(),
            assumptions: vec![],
            is_status_quo: false,
        });

        let ids = alt.alternative_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"a1"));
        assert!(ids.contains(&"a2"));
    }

    #[test]
    fn set_strategy_table_updates_output() {
        let mut alt = Alternatives::new();
        let table = StrategyTable {
            decisions: vec![DecisionColumn {
                decision_name: "Location".to_string(),
                options: vec!["NYC".to_string(), "SF".to_string()],
            }],
            strategies: vec![Strategy {
                id: "s1".to_string(),
                name: "West Coast".to_string(),
                choices: {
                    let mut m = HashMap::new();
                    m.insert("Location".to_string(), "SF".to_string());
                    m
                },
            }],
        };
        alt.set_strategy_table(table);

        assert!(alt.output().strategy_table.is_some());
    }

    #[test]
    fn output_roundtrips_through_json() {
        let mut alt = Alternatives::new();
        alt.add_alternative(Alternative {
            id: "a1".to_string(),
            name: "Test".to_string(),
            description: "Description".to_string(),
            assumptions: vec!["Assumption".to_string()],
            is_status_quo: true,
        });

        let value = alt.output_as_value();
        let mut alt2 = Alternatives::new();
        alt2.set_output_from_value(value).unwrap();

        assert_eq!(alt.alternatives_count(), alt2.alternatives_count());
        assert_eq!(alt.has_status_quo(), alt2.has_status_quo());
    }
}
