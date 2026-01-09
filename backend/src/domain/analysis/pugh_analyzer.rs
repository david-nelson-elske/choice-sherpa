//! Pugh Analyzer - Score computation, dominance detection, and irrelevant objective identification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ConsequencesTable;

/// An alternative that is dominated by another.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DominatedAlternative {
    pub alternative_id: String,
    pub dominated_by_id: String,
    pub explanation: String,
}

impl DominatedAlternative {
    /// Creates a new dominated alternative record.
    pub fn new(
        alternative_id: impl Into<String>,
        dominated_by_id: impl Into<String>,
    ) -> Self {
        Self {
            alternative_id: alternative_id.into(),
            dominated_by_id: dominated_by_id.into(),
            explanation: String::new(),
        }
    }

    /// Creates with an explanation.
    pub fn with_explanation(
        alternative_id: impl Into<String>,
        dominated_by_id: impl Into<String>,
        explanation: impl Into<String>,
    ) -> Self {
        Self {
            alternative_id: alternative_id.into(),
            dominated_by_id: dominated_by_id.into(),
            explanation: explanation.into(),
        }
    }
}

/// An objective that doesn't distinguish between alternatives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrrelevantObjective {
    pub objective_id: String,
    pub uniform_rating: i8,
    pub reason: String,
}

impl IrrelevantObjective {
    /// Creates a new irrelevant objective record.
    pub fn new(objective_id: impl Into<String>, uniform_rating: i8) -> Self {
        Self {
            objective_id: objective_id.into(),
            uniform_rating,
            reason: "All alternatives have the same rating".to_string(),
        }
    }
}

/// Pugh matrix analysis functions.
pub struct PughAnalyzer;

impl PughAnalyzer {
    /// Computes Pugh scores for each alternative.
    ///
    /// # Algorithm
    /// For each alternative: score = Σ(rating[objective])
    ///
    /// # Edge Cases
    /// - Empty table: Returns empty HashMap
    /// - Single alternative: Returns score for that alternative
    /// - No objectives: Returns 0 for all alternatives
    /// - Missing cells: Treated as 0 (neutral)
    pub fn compute_scores(table: &ConsequencesTable) -> HashMap<String, i32> {
        let mut scores = HashMap::new();

        if table.alternative_ids.is_empty() {
            return scores;
        }

        for alt_id in &table.alternative_ids {
            let mut total: i32 = 0;

            for obj_id in &table.objective_ids {
                let rating = table
                    .get_cell(alt_id, obj_id)
                    .map(|c| c.rating.value() as i32)
                    .unwrap_or(0);
                total += rating;
            }

            scores.insert(alt_id.clone(), total);
        }

        scores
    }

    /// Finds all dominated alternatives.
    ///
    /// Alternative A dominates Alternative B if:
    /// 1. A >= B on ALL objectives
    /// 2. A > B on AT LEAST ONE objective
    ///
    /// # Edge Cases
    /// - Empty table: Returns empty Vec
    /// - Single alternative: Returns empty Vec (can't dominate self)
    /// - Ties: Neither dominates if equal on all objectives
    pub fn find_dominated(table: &ConsequencesTable) -> Vec<DominatedAlternative> {
        let mut dominated = Vec::new();

        if table.alternative_ids.len() < 2 {
            return dominated;
        }

        for candidate in &table.alternative_ids {
            for potential_dominator in &table.alternative_ids {
                if candidate == potential_dominator {
                    continue;
                }

                if Self::dominates(table, potential_dominator, candidate) {
                    dominated.push(DominatedAlternative::with_explanation(
                        candidate.clone(),
                        potential_dominator.clone(),
                        Self::explain_dominance(table, potential_dominator, candidate),
                    ));
                    break; // Only need one dominator per candidate
                }
            }
        }

        dominated
    }

    /// Checks if alternative `a` dominates alternative `b`.
    fn dominates(table: &ConsequencesTable, a: &str, b: &str) -> bool {
        let mut at_least_equal = true;
        let mut strictly_better_on_one = false;

        for obj_id in &table.objective_ids {
            let a_rating = table
                .get_cell(a, obj_id)
                .map(|c| c.rating.value())
                .unwrap_or(0);
            let b_rating = table
                .get_cell(b, obj_id)
                .map(|c| c.rating.value())
                .unwrap_or(0);

            if a_rating < b_rating {
                at_least_equal = false;
                break;
            }

            if a_rating > b_rating {
                strictly_better_on_one = true;
            }
        }

        at_least_equal && strictly_better_on_one
    }

    /// Generates explanation for why `a` dominates `b`.
    fn explain_dominance(table: &ConsequencesTable, a: &str, b: &str) -> String {
        let mut better_on = Vec::new();

        for obj_id in &table.objective_ids {
            let a_rating = table
                .get_cell(a, obj_id)
                .map(|c| c.rating.value())
                .unwrap_or(0);
            let b_rating = table
                .get_cell(b, obj_id)
                .map(|c| c.rating.value())
                .unwrap_or(0);

            if a_rating > b_rating {
                better_on.push(obj_id.as_str());
            }
        }

        format!(
            "{} is at least as good on all objectives and strictly better on: {}",
            a,
            better_on.join(", ")
        )
    }

    /// Finds objectives that don't distinguish between alternatives.
    ///
    /// An objective is irrelevant if all alternatives have the same rating on it.
    ///
    /// # Edge Cases
    /// - Single alternative: Returns empty Vec (no comparison possible)
    /// - All objectives vary: Returns empty Vec
    pub fn find_irrelevant_objectives(table: &ConsequencesTable) -> Vec<IrrelevantObjective> {
        let mut irrelevant = Vec::new();

        // Need at least 2 alternatives to compare
        if table.alternative_ids.len() < 2 {
            return irrelevant;
        }

        for obj_id in &table.objective_ids {
            let ratings: Vec<i8> = table
                .alternative_ids
                .iter()
                .map(|alt_id| {
                    table
                        .get_cell(alt_id, obj_id)
                        .map(|c| c.rating.value())
                        .unwrap_or(0)
                })
                .collect();

            if !ratings.is_empty() && Self::all_same(&ratings) {
                irrelevant.push(IrrelevantObjective::new(obj_id.clone(), ratings[0]));
            }
        }

        irrelevant
    }

    /// Checks if all values in the slice are the same.
    fn all_same(ratings: &[i8]) -> bool {
        if ratings.is_empty() {
            return true;
        }
        let first = ratings[0];
        ratings.iter().all(|&r| r == first)
    }

    /// Finds the best alternative by total score.
    /// Returns None if empty or if there's a tie for best.
    pub fn find_best(table: &ConsequencesTable) -> Option<String> {
        let scores = Self::compute_scores(table);

        if scores.is_empty() {
            return None;
        }

        let max_score = scores.values().max()?;
        let best: Vec<_> = scores
            .iter()
            .filter(|(_, &score)| score == *max_score)
            .map(|(id, _)| id.clone())
            .collect();

        // Return None on tie
        if best.len() == 1 {
            Some(best.into_iter().next().unwrap())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::Rating;

    fn single_alternative_table() -> ConsequencesTable {
        ConsequencesTable::builder()
            .alternatives(vec!["A"])
            .objectives(vec!["O1", "O2"])
            .cell("A", "O1", Rating::Better)
            .cell("A", "O2", Rating::MuchBetter)
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

    // Pugh Score Tests

    #[test]
    fn compute_scores_empty_table() {
        let table = ConsequencesTable::empty();
        let scores = PughAnalyzer::compute_scores(&table);
        assert!(scores.is_empty());
    }

    #[test]
    fn compute_scores_no_objectives() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["alt-1", "alt-2"])
            .objectives(Vec::<&str>::new())
            .build();

        let scores = PughAnalyzer::compute_scores(&table);
        assert_eq!(scores.get("alt-1"), Some(&0));
        assert_eq!(scores.get("alt-2"), Some(&0));
    }

    #[test]
    fn compute_scores_single_alternative() {
        let table = single_alternative_table();
        let scores = PughAnalyzer::compute_scores(&table);

        assert_eq!(scores.len(), 1);
        assert_eq!(scores.get("A"), Some(&3)); // +1 + +2 = 3
    }

    #[test]
    fn compute_scores_missing_cells() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["alt-1", "alt-2"])
            .objectives(vec!["obj-1", "obj-2", "obj-3"])
            .cell("alt-1", "obj-1", Rating::Better) // +1
            // alt-1/obj-2 missing -> 0
            // alt-1/obj-3 missing -> 0
            .cell("alt-2", "obj-1", Rating::Worse) // -1
            .cell("alt-2", "obj-2", Rating::Same) // 0
            // alt-2/obj-3 missing -> 0
            .build();

        let scores = PughAnalyzer::compute_scores(&table);
        assert_eq!(scores.get("alt-1"), Some(&1)); // 1 + 0 + 0
        assert_eq!(scores.get("alt-2"), Some(&-1)); // -1 + 0 + 0
    }

    #[test]
    fn compute_scores_all_neutral() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A", "B"])
            .objectives(vec!["O1", "O2"])
            .cell("A", "O1", Rating::Same)
            .cell("A", "O2", Rating::Same)
            .cell("B", "O1", Rating::Same)
            .cell("B", "O2", Rating::Same)
            .build();

        let scores = PughAnalyzer::compute_scores(&table);
        assert_eq!(scores.get("A"), Some(&0));
        assert_eq!(scores.get("B"), Some(&0));
    }

    #[test]
    fn compute_scores_extreme_positive() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A"])
            .objectives(vec!["O1", "O2", "O3"])
            .cell("A", "O1", Rating::MuchBetter)
            .cell("A", "O2", Rating::MuchBetter)
            .cell("A", "O3", Rating::MuchBetter)
            .build();

        let scores = PughAnalyzer::compute_scores(&table);
        assert_eq!(scores.get("A"), Some(&6)); // 3 objectives × +2
    }

    #[test]
    fn compute_scores_extreme_negative() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A"])
            .objectives(vec!["O1", "O2", "O3"])
            .cell("A", "O1", Rating::MuchWorse)
            .cell("A", "O2", Rating::MuchWorse)
            .cell("A", "O3", Rating::MuchWorse)
            .build();

        let scores = PughAnalyzer::compute_scores(&table);
        assert_eq!(scores.get("A"), Some(&-6)); // 3 objectives × -2
    }

    // Dominance Detection Tests

    #[test]
    fn find_dominated_empty_table() {
        let table = ConsequencesTable::empty();
        let dominated = PughAnalyzer::find_dominated(&table);
        assert!(dominated.is_empty());
    }

    #[test]
    fn find_dominated_single_alternative() {
        let table = single_alternative_table();
        let dominated = PughAnalyzer::find_dominated(&table);
        assert!(dominated.is_empty());
    }

    #[test]
    fn find_dominated_all_tied() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A", "B", "C"])
            .objectives(vec!["O1", "O2"])
            .cell("A", "O1", Rating::Same)
            .cell("A", "O2", Rating::Same)
            .cell("B", "O1", Rating::Same)
            .cell("B", "O2", Rating::Same)
            .cell("C", "O1", Rating::Same)
            .cell("C", "O2", Rating::Same)
            .build();

        let dominated = PughAnalyzer::find_dominated(&table);
        assert!(dominated.is_empty(), "No alternative dominates when all tied");
    }

    #[test]
    fn find_dominated_clear_dominance() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A", "B"])
            .objectives(vec!["O1", "O2", "O3"])
            // A is better on everything
            .cell("A", "O1", Rating::MuchBetter)
            .cell("A", "O2", Rating::Better)
            .cell("A", "O3", Rating::Better)
            // B is worse or equal on everything
            .cell("B", "O1", Rating::Same)
            .cell("B", "O2", Rating::Same)
            .cell("B", "O3", Rating::Worse)
            .build();

        let dominated = PughAnalyzer::find_dominated(&table);
        assert_eq!(dominated.len(), 1);
        assert_eq!(dominated[0].alternative_id, "B");
        assert_eq!(dominated[0].dominated_by_id, "A");
    }

    #[test]
    fn find_dominated_tradeoffs_no_dominance() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A", "B"])
            .objectives(vec!["Cost", "Quality"])
            // A: Low cost, low quality
            .cell("A", "Cost", Rating::MuchBetter)
            .cell("A", "Quality", Rating::Worse)
            // B: High cost, high quality
            .cell("B", "Cost", Rating::Worse)
            .cell("B", "Quality", Rating::MuchBetter)
            .build();

        let dominated = PughAnalyzer::find_dominated(&table);
        assert!(
            dominated.is_empty(),
            "Tradeoff alternatives don't dominate each other"
        );
    }

    #[test]
    fn find_dominated_weak_dominance() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A", "B"])
            .objectives(vec!["O1", "O2", "O3"])
            // A: Better on one, same on rest
            .cell("A", "O1", Rating::Better)
            .cell("A", "O2", Rating::Same)
            .cell("A", "O3", Rating::Same)
            // B: Same on everything
            .cell("B", "O1", Rating::Same)
            .cell("B", "O2", Rating::Same)
            .cell("B", "O3", Rating::Same)
            .build();

        let dominated = PughAnalyzer::find_dominated(&table);
        assert_eq!(dominated.len(), 1);
        assert_eq!(dominated[0].alternative_id, "B");
    }

    #[test]
    fn find_dominated_near_dominance() {
        // A almost dominates B but is worse on one objective
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A", "B"])
            .objectives(vec!["O1", "O2", "O3"])
            .cell("A", "O1", Rating::MuchBetter)
            .cell("A", "O2", Rating::Better)
            .cell("A", "O3", Rating::Worse) // A is worse here
            .cell("B", "O1", Rating::Same)
            .cell("B", "O2", Rating::Same)
            .cell("B", "O3", Rating::Better)
            .build();

        let dominated = PughAnalyzer::find_dominated(&table);
        assert!(dominated.is_empty(), "Near-dominance is not dominance");
    }

    // Irrelevant Objectives Tests

    #[test]
    fn irrelevant_single_alternative() {
        let table = single_alternative_table();
        let irrelevant = PughAnalyzer::find_irrelevant_objectives(&table);
        assert!(irrelevant.is_empty());
    }

    #[test]
    fn irrelevant_all_same_rating() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A", "B", "C"])
            .objectives(vec!["Cost", "Quality"])
            // Cost: All same
            .cell("A", "Cost", Rating::Same)
            .cell("B", "Cost", Rating::Same)
            .cell("C", "Cost", Rating::Same)
            // Quality: Varies
            .cell("A", "Quality", Rating::Better)
            .cell("B", "Quality", Rating::Same)
            .cell("C", "Quality", Rating::Worse)
            .build();

        let irrelevant = PughAnalyzer::find_irrelevant_objectives(&table);
        assert_eq!(irrelevant.len(), 1);
        assert_eq!(irrelevant[0].objective_id, "Cost");
        assert_eq!(irrelevant[0].uniform_rating, 0);
    }

    #[test]
    fn irrelevant_all_objectives_vary() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A", "B"])
            .objectives(vec!["O1", "O2"])
            .cell("A", "O1", Rating::Better)
            .cell("A", "O2", Rating::Better)
            .cell("B", "O1", Rating::Worse)
            .cell("B", "O2", Rating::Worse)
            .build();

        let irrelevant = PughAnalyzer::find_irrelevant_objectives(&table);
        assert!(irrelevant.is_empty());
    }

    #[test]
    fn irrelevant_with_missing_cells() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A", "B"])
            .objectives(vec!["O1"])
            // Both missing -> both treated as 0 -> uniform
            .build();

        let irrelevant = PughAnalyzer::find_irrelevant_objectives(&table);
        assert_eq!(irrelevant.len(), 1);
        assert_eq!(irrelevant[0].uniform_rating, 0);
    }

    // Find Best Tests

    #[test]
    fn find_best_empty_table() {
        let table = ConsequencesTable::empty();
        assert!(PughAnalyzer::find_best(&table).is_none());
    }

    #[test]
    fn find_best_single_alternative() {
        let table = single_alternative_table();
        assert_eq!(PughAnalyzer::find_best(&table), Some("A".to_string()));
    }

    #[test]
    fn find_best_clear_winner() {
        let table = two_alternative_table();
        assert_eq!(PughAnalyzer::find_best(&table), Some("A".to_string()));
    }

    #[test]
    fn find_best_tie_returns_none() {
        let table = ConsequencesTable::builder()
            .alternatives(vec!["A", "B"])
            .objectives(vec!["O1", "O2"])
            .cell("A", "O1", Rating::Better)
            .cell("A", "O2", Rating::Worse)
            .cell("B", "O1", Rating::Worse)
            .cell("B", "O2", Rating::Better)
            .build();

        // Both score 0
        assert!(PughAnalyzer::find_best(&table).is_none());
    }
}
