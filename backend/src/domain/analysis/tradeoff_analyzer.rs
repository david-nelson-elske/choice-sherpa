//! Tradeoff Analyzer - Tension analysis for non-dominated alternatives.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashSet;

use super::{ConsequencesTable, DominatedAlternative};

/// Tension analysis for a single alternative.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tension {
    pub alternative_id: String,
    /// Objectives where this alternative outperforms at least one other non-dominated alternative.
    pub gains: Vec<String>,
    /// Objectives where this alternative underperforms at least one other non-dominated alternative.
    pub losses: Vec<String>,
    /// Optional uncertainty impact assessment.
    pub uncertainty_impact: Option<String>,
}

impl Tension {
    /// Creates a new tension record.
    pub fn new(alternative_id: impl Into<String>) -> Self {
        Self {
            alternative_id: alternative_id.into(),
            gains: Vec::new(),
            losses: Vec::new(),
            uncertainty_impact: None,
        }
    }

    /// Creates a tension with gains and losses.
    pub fn with_tradeoffs(
        alternative_id: impl Into<String>,
        gains: Vec<String>,
        losses: Vec<String>,
    ) -> Self {
        Self {
            alternative_id: alternative_id.into(),
            gains,
            losses,
            uncertainty_impact: None,
        }
    }

    /// Returns true if this alternative has no losses.
    pub fn is_clear_winner(&self) -> bool {
        !self.gains.is_empty() && self.losses.is_empty()
    }

    /// Returns true if this alternative has pure tradeoffs (both gains and losses).
    pub fn has_tradeoffs(&self) -> bool {
        !self.gains.is_empty() && !self.losses.is_empty()
    }
}

/// Summary of tradeoff analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeoffSummary {
    pub total_alternatives: usize,
    pub has_clear_winner: bool,
    pub most_balanced: Option<String>,
    pub most_polarizing: Option<String>,
}

/// Analyzer for alternative tensions and tradeoffs.
pub struct TradeoffAnalyzer;

impl TradeoffAnalyzer {
    /// Analyzes tensions for non-dominated alternatives.
    ///
    /// A tension exists when choosing one alternative means:
    /// - Gaining on some objectives (where this alt is better than others)
    /// - Losing on other objectives (where this alt is worse than others)
    ///
    /// # Edge Cases
    /// - All dominated: Returns empty Vec
    /// - Single non-dominated: Returns single tension with empty gains/losses
    pub fn analyze_tensions(
        table: &ConsequencesTable,
        dominated: &[DominatedAlternative],
    ) -> Vec<Tension> {
        let dominated_ids: HashSet<_> = dominated.iter().map(|d| d.alternative_id.as_str()).collect();

        let viable: Vec<_> = table
            .alternative_ids
            .iter()
            .filter(|id| !dominated_ids.contains(id.as_str()))
            .collect();

        // Edge case: fewer than 2 viable alternatives
        if viable.is_empty() {
            return Vec::new();
        }

        if viable.len() == 1 {
            return vec![Tension::new(viable[0].clone())];
        }

        let mut tensions = Vec::new();

        for alt_id in &viable {
            let mut gains = HashSet::new();
            let mut losses = HashSet::new();

            // Compare against OTHER non-dominated alternatives
            for other_id in &viable {
                if *other_id == *alt_id {
                    continue;
                }

                for obj_id in &table.objective_ids {
                    let my_rating = table
                        .get_cell(alt_id, obj_id)
                        .map(|c| c.rating.value())
                        .unwrap_or(0);
                    let other_rating = table
                        .get_cell(other_id, obj_id)
                        .map(|c| c.rating.value())
                        .unwrap_or(0);

                    match my_rating.cmp(&other_rating) {
                        Ordering::Greater => {
                            gains.insert(obj_id.clone());
                        }
                        Ordering::Less => {
                            losses.insert(obj_id.clone());
                        }
                        Ordering::Equal => {}
                    }
                }
            }

            tensions.push(Tension::with_tradeoffs(
                (*alt_id).clone(),
                gains.into_iter().collect(),
                losses.into_iter().collect(),
            ));
        }

        tensions
    }

    /// Summarizes tradeoff analysis results.
    pub fn summarize_tradeoffs(tensions: &[Tension]) -> TradeoffSummary {
        if tensions.is_empty() {
            return TradeoffSummary {
                total_alternatives: 0,
                has_clear_winner: false,
                most_balanced: None,
                most_polarizing: None,
            };
        }

        // Clear winner: has gains but NO losses
        let has_clear_winner = tensions.iter().any(|t| t.is_clear_winner());

        // Most balanced: smallest absolute difference between gains and losses
        let most_balanced = tensions
            .iter()
            .min_by_key(|t| (t.gains.len() as i32 - t.losses.len() as i32).abs())
            .map(|t| t.alternative_id.clone());

        // Most polarizing: largest total of gains + losses
        let most_polarizing = tensions
            .iter()
            .max_by_key(|t| t.gains.len() + t.losses.len())
            .map(|t| t.alternative_id.clone());

        TradeoffSummary {
            total_alternatives: tensions.len(),
            has_clear_winner,
            most_balanced,
            most_polarizing,
        }
    }

    /// Finds alternatives with no losses (potential clear winners).
    pub fn find_clear_winners(tensions: &[Tension]) -> Vec<&Tension> {
        tensions.iter().filter(|t| t.is_clear_winner()).collect()
    }

    /// Calculates the "tradeoff intensity" for an alternative.
    /// Higher values mean more objectives are in contention.
    pub fn tradeoff_intensity(tension: &Tension) -> usize {
        tension.gains.len() + tension.losses.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::Rating;

    fn three_alternative_table() -> ConsequencesTable {
        ConsequencesTable::builder()
            .alternatives(vec!["A", "B", "C"])
            .objectives(vec!["O1", "O2"])
            .cell("A", "O1", Rating::Better)
            .cell("A", "O2", Rating::Same)
            .cell("B", "O1", Rating::Same)
            .cell("B", "O2", Rating::Better)
            .cell("C", "O1", Rating::Worse)
            .cell("C", "O2", Rating::Worse)
            .build()
    }

    fn two_alternative_table() -> ConsequencesTable {
        ConsequencesTable::builder()
            .alternatives(vec!["A", "B"])
            .objectives(vec!["O1", "O2"])
            .cell("A", "O1", Rating::Better)
            .cell("A", "O2", Rating::Better)
            .cell("B", "O1", Rating::Same)
            .cell("B", "O2", Rating::Same)
            .build()
    }

    #[test]
    fn tradeoffs_all_dominated() {
        let table = three_alternative_table();
        // Pretend all are dominated
        let dominated = vec![
            DominatedAlternative::new("A", "X"),
            DominatedAlternative::new("B", "X"),
            DominatedAlternative::new("C", "X"),
        ];

        let tensions = TradeoffAnalyzer::analyze_tensions(&table, &dominated);
        assert!(tensions.is_empty());
    }

    #[test]
    fn tradeoffs_single_non_dominated() {
        let table = two_alternative_table();
        let dominated = vec![DominatedAlternative::new("B", "A")];

        let tensions = TradeoffAnalyzer::analyze_tensions(&table, &dominated);
        assert_eq!(tensions.len(), 1);
        assert_eq!(tensions[0].alternative_id, "A");
        assert!(tensions[0].gains.is_empty());
        assert!(tensions[0].losses.is_empty());
    }

    #[test]
    fn tradeoffs_clear_winner() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A", "B", "C"])
            .objectives(vec!["O1", "O2", "O3"])
            // A: Best on everything vs B and C
            .cell("A", "O1", Rating::MuchBetter)
            .cell("A", "O2", Rating::Better)
            .cell("A", "O3", Rating::Better)
            // B: Medium
            .cell("B", "O1", Rating::Same)
            .cell("B", "O2", Rating::Better) // B better than C on O2
            .cell("B", "O3", Rating::Worse) // B worse than C on O3
            // C: Medium (trades with B)
            .cell("C", "O1", Rating::Same)
            .cell("C", "O2", Rating::Worse)
            .cell("C", "O3", Rating::Better)
            .build();

        let dominated = vec![]; // No dominated alternatives
        let tensions = TradeoffAnalyzer::analyze_tensions(&table, &dominated);
        let summary = TradeoffAnalyzer::summarize_tradeoffs(&tensions);

        assert!(summary.has_clear_winner);

        // A has gains on O1, O2, O3 vs B and C; no losses
        let a_tension = tensions.iter().find(|t| t.alternative_id == "A").unwrap();
        assert!(!a_tension.gains.is_empty());
        assert!(a_tension.losses.is_empty());
        assert!(a_tension.is_clear_winner());
    }

    #[test]
    fn tradeoffs_pure_tradeoff() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A", "B"])
            .objectives(vec!["Cost", "Quality"])
            // A: Good cost, bad quality
            .cell("A", "Cost", Rating::Better)
            .cell("A", "Quality", Rating::Worse)
            // B: Bad cost, good quality
            .cell("B", "Cost", Rating::Worse)
            .cell("B", "Quality", Rating::Better)
            .build();

        let dominated = vec![];
        let tensions = TradeoffAnalyzer::analyze_tensions(&table, &dominated);
        let summary = TradeoffAnalyzer::summarize_tradeoffs(&tensions);

        assert!(!summary.has_clear_winner);

        // Both alternatives have both gains and losses
        for tension in &tensions {
            assert!(!tension.gains.is_empty());
            assert!(!tension.losses.is_empty());
            assert!(tension.has_tradeoffs());
        }
    }

    #[test]
    fn tradeoffs_no_alternatives() {
        let table = ConsequencesTable::empty();
        let tensions = TradeoffAnalyzer::analyze_tensions(&table, &[]);
        assert!(tensions.is_empty());

        let summary = TradeoffAnalyzer::summarize_tradeoffs(&tensions);
        assert_eq!(summary.total_alternatives, 0);
        assert!(!summary.has_clear_winner);
    }

    #[test]
    fn summary_finds_most_balanced() {
        let tensions = vec![
            Tension::with_tradeoffs("A", vec!["O1".into(), "O2".into()], vec!["O3".into()]),
            Tension::with_tradeoffs("B", vec!["O3".into()], vec!["O1".into()]), // Most balanced: |1-1| = 0
            Tension::with_tradeoffs("C", vec![], vec!["O1".into(), "O2".into(), "O3".into()]),
        ];

        let summary = TradeoffAnalyzer::summarize_tradeoffs(&tensions);
        assert_eq!(summary.most_balanced, Some("B".to_string()));
    }

    #[test]
    fn summary_finds_most_polarizing() {
        let tensions = vec![
            Tension::with_tradeoffs("A", vec!["O1".into()], vec!["O2".into()]), // 2 total
            Tension::with_tradeoffs("B", vec!["O1".into(), "O2".into()], vec!["O3".into(), "O4".into()]), // 4 total - most polarizing
            Tension::with_tradeoffs("C", vec![], vec!["O1".into()]), // 1 total
        ];

        let summary = TradeoffAnalyzer::summarize_tradeoffs(&tensions);
        assert_eq!(summary.most_polarizing, Some("B".to_string()));
    }

    #[test]
    fn find_clear_winners_returns_only_winners() {
        let tensions = vec![
            Tension::with_tradeoffs("A", vec!["O1".into()], vec![]), // Clear winner
            Tension::with_tradeoffs("B", vec!["O2".into()], vec!["O1".into()]), // Has losses
            Tension::with_tradeoffs("C", vec![], vec!["O2".into()]), // No gains
        ];

        let winners = TradeoffAnalyzer::find_clear_winners(&tensions);
        assert_eq!(winners.len(), 1);
        assert_eq!(winners[0].alternative_id, "A");
    }

    #[test]
    fn tradeoff_intensity_calculates_correctly() {
        let low = Tension::with_tradeoffs("A", vec!["O1".into()], vec![]);
        let high = Tension::with_tradeoffs(
            "B",
            vec!["O1".into(), "O2".into()],
            vec!["O3".into(), "O4".into()],
        );

        assert_eq!(TradeoffAnalyzer::tradeoff_intensity(&low), 1);
        assert_eq!(TradeoffAnalyzer::tradeoff_intensity(&high), 4);
    }

    #[test]
    fn tension_serializes() {
        let tension = Tension::with_tradeoffs("A", vec!["O1".into()], vec!["O2".into()]);
        let json = serde_json::to_string(&tension).unwrap();
        assert!(json.contains("\"alternative_id\":\"A\""));
        assert!(json.contains("\"gains\":[\"O1\"]"));
    }
}
