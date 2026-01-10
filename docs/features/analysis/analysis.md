# Analysis Module Specification

## Overview

The Analysis module provides stateless domain services for analytical computations: Pugh matrix calculations, Decision Quality scoring, and tradeoff analysis. These are pure functions with no persistence needs - they're called by other modules to perform calculations.

---

## Module Classification

| Attribute | Value |
|-----------|-------|
| **Type** | Domain Services (pure functions, no ports/adapters) |
| **Language** | Rust |
| **Responsibility** | Pugh matrix, DQ scoring, tradeoff analysis |
| **Domain Dependencies** | foundation, proact-types |
| **External Dependencies** | None (pure Rust) |

---

## Architecture

### Domain Services Pattern

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          ANALYSIS MODULE                                     │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                       DOMAIN SERVICES                                   │ │
│  │                       (Pure Functions)                                  │ │
│  │                                                                         │ │
│  │   ┌────────────────────────────────────────────────────────────────┐   │ │
│  │   │                    PughAnalyzer                                 │   │ │
│  │   │                                                                 │   │ │
│  │   │   + compute_scores(table) -> HashMap<AltId, i32>                │   │ │
│  │   │   + find_dominated(table) -> Vec<DominatedAlternative>          │   │ │
│  │   │   + find_irrelevant_objectives(table) -> Vec<ObjectiveId>       │   │ │
│  │   │   + rank_alternatives(table) -> Vec<(AltId, i32)>               │   │ │
│  │   └────────────────────────────────────────────────────────────────┘   │ │
│  │                                                                         │ │
│  │   ┌────────────────────────────────────────────────────────────────┐   │ │
│  │   │                   DQCalculator                                  │   │ │
│  │   │                                                                 │   │ │
│  │   │   + compute_overall(elements) -> Percentage                     │   │ │
│  │   │   + identify_weakest(elements) -> Option<&DQElement>            │   │ │
│  │   │   + suggest_improvements(elements) -> Vec<Improvement>          │   │ │
│  │   └────────────────────────────────────────────────────────────────┘   │ │
│  │                                                                         │ │
│  │   ┌────────────────────────────────────────────────────────────────┐   │ │
│  │   │                  TradeoffAnalyzer                               │   │ │
│  │   │                                                                 │   │ │
│  │   │   + analyze_tensions(table, dominated) -> Vec<Tension>          │   │ │
│  │   │   + summarize_tradeoffs(tensions) -> TradeoffSummary            │   │ │
│  │   └────────────────────────────────────────────────────────────────┘   │ │
│  │                                                                         │ │
│  │   ┌────────────────────────────────────────────────────────────────┐   │ │
│  │   │                    CellColor                                    │   │ │
│  │   │                   (Value Object)                                │   │ │
│  │   │                                                                 │   │ │
│  │   │   + from_rating(rating) -> CellColor                            │   │ │
│  │   │   + to_css_class() -> &str                                      │   │ │
│  │   │   + to_hex_color() -> &str                                      │   │ │
│  │   └────────────────────────────────────────────────────────────────┘   │ │
│  │                                                                         │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  NOTE: No ports, no adapters, no persistence - pure domain logic            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Domain Services

### PughAnalyzer

Computes Pugh matrix results, identifies dominated alternatives, and ranks options.

```rust
use std::collections::HashMap;
use crate::foundation::Rating;
use crate::proact::{ConsequencesTable, Cell, DominatedAlternative, IrrelevantObjective};

/// Stateless service for Pugh matrix analysis
pub struct PughAnalyzer;

impl PughAnalyzer {
    /// Computes total Pugh scores for each alternative
    /// Score = sum of all ratings for an alternative
    pub fn compute_scores(table: &ConsequencesTable) -> HashMap<String, i32> {
        let mut scores = HashMap::new();

        for alt_id in &table.alternative_ids {
            let mut total: i32 = 0;

            for obj_id in &table.objective_ids {
                if let Some(cell) = table.get_cell(alt_id, obj_id) {
                    total += cell.rating.value() as i32;
                }
            }

            scores.insert(alt_id.clone(), total);
        }

        scores
    }

    /// Finds dominated alternatives
    /// Alternative A dominates B if A >= B on all objectives and A > B on at least one
    pub fn find_dominated(table: &ConsequencesTable) -> Vec<DominatedAlternative> {
        let mut dominated = Vec::new();

        for candidate in &table.alternative_ids {
            for dominator in &table.alternative_ids {
                if candidate == dominator {
                    continue;
                }

                if Self::dominates(table, dominator, candidate) {
                    dominated.push(DominatedAlternative {
                        alternative_id: candidate.clone(),
                        dominated_by_id: dominator.clone(),
                        explanation: Self::explain_dominance(table, dominator, candidate),
                    });
                    break; // Only need to find one dominator
                }
            }
        }

        dominated
    }

    /// Checks if alternative A dominates alternative B
    fn dominates(table: &ConsequencesTable, a: &str, b: &str) -> bool {
        let mut at_least_equal = true;
        let mut strictly_better_on_one = false;

        for obj_id in &table.objective_ids {
            let a_rating = table.get_cell(a, obj_id)
                .map(|c| c.rating.value())
                .unwrap_or(0);
            let b_rating = table.get_cell(b, obj_id)
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

    fn explain_dominance(table: &ConsequencesTable, dominator: &str, dominated: &str) -> String {
        let better_on: Vec<String> = table.objective_ids.iter()
            .filter(|obj_id| {
                let dom_rating = table.get_cell(dominator, obj_id)
                    .map(|c| c.rating.value())
                    .unwrap_or(0);
                let sub_rating = table.get_cell(dominated, obj_id)
                    .map(|c| c.rating.value())
                    .unwrap_or(0);
                dom_rating > sub_rating
            })
            .cloned()
            .collect();

        format!(
            "{} is dominated by {} - {} performs better on: {}",
            dominated,
            dominator,
            dominator,
            better_on.join(", ")
        )
    }

    /// Finds objectives that don't distinguish between alternatives
    /// (all alternatives have the same rating)
    pub fn find_irrelevant_objectives(table: &ConsequencesTable) -> Vec<IrrelevantObjective> {
        let mut irrelevant = Vec::new();

        for obj_id in &table.objective_ids {
            let ratings: Vec<i8> = table.alternative_ids.iter()
                .filter_map(|alt_id| {
                    table.get_cell(alt_id, obj_id).map(|c| c.rating.value())
                })
                .collect();

            // If all ratings are the same, objective is irrelevant
            if ratings.len() > 1 && ratings.windows(2).all(|w| w[0] == w[1]) {
                irrelevant.push(IrrelevantObjective {
                    objective_id: obj_id.clone(),
                    reason: "All alternatives have the same rating".to_string(),
                });
            }
        }

        irrelevant
    }

    /// Ranks alternatives by their Pugh scores (highest first)
    pub fn rank_alternatives(table: &ConsequencesTable) -> Vec<(String, i32)> {
        let scores = Self::compute_scores(table);
        let mut ranked: Vec<_> = scores.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1)); // Descending
        ranked
    }

    /// Returns the top alternative (if there's a clear winner)
    pub fn find_top_alternative(table: &ConsequencesTable) -> Option<(String, i32)> {
        let ranked = Self::rank_alternatives(table);

        if ranked.len() < 2 {
            return ranked.into_iter().next();
        }

        let top = &ranked[0];
        let second = &ranked[1];

        // Only return if there's a clear winner (not a tie)
        if top.1 > second.1 {
            Some(top.clone())
        } else {
            None
        }
    }
}
```

### DQCalculator

Computes Decision Quality scores and identifies improvement opportunities.

```rust
use crate::foundation::Percentage;
use crate::proact::DQElement;

/// Stateless service for Decision Quality calculations
pub struct DQCalculator;

/// The 7 standard Decision Quality element names
pub const DQ_ELEMENT_NAMES: &[&str] = &[
    "Helpful Problem Frame",
    "Clear Objectives",
    "Creative Alternatives",
    "Reliable Consequence Information",
    "Logically Correct Reasoning",
    "Clear Tradeoffs",
    "Commitment to Follow Through",
];

impl DQCalculator {
    /// Computes overall DQ score as minimum of all element scores
    /// Decision quality is only as strong as its weakest element
    pub fn compute_overall(elements: &[DQElement]) -> Percentage {
        if elements.is_empty() {
            return Percentage::ZERO;
        }

        let min_score = elements.iter()
            .map(|e| e.score.value())
            .min()
            .unwrap_or(0);

        Percentage::new(min_score)
    }

    /// Identifies the weakest element (lowest score)
    pub fn identify_weakest<'a>(elements: &'a [DQElement]) -> Option<&'a DQElement> {
        elements.iter().min_by_key(|e| e.score.value())
    }

    /// Identifies elements below a threshold (default: 70%)
    pub fn identify_weak_elements(
        elements: &[DQElement],
        threshold: Percentage,
    ) -> Vec<&DQElement> {
        elements.iter()
            .filter(|e| e.score.value() < threshold.value())
            .collect()
    }

    /// Suggests improvements based on weak elements
    pub fn suggest_improvements(elements: &[DQElement]) -> Vec<Improvement> {
        let threshold = Percentage::new(70);
        let weak = Self::identify_weak_elements(elements, threshold);

        weak.into_iter()
            .map(|e| Improvement {
                element_name: e.name.clone(),
                current_score: e.score,
                suggestion: e.improvement.clone(),
                priority: Self::compute_priority(&e.score),
            })
            .collect()
    }

    fn compute_priority(score: &Percentage) -> Priority {
        match score.value() {
            0..=30 => Priority::Critical,
            31..=50 => Priority::High,
            51..=70 => Priority::Medium,
            _ => Priority::Low,
        }
    }

    /// Checks if decision quality is "good enough" (>= 80% on all elements)
    pub fn is_acceptable(elements: &[DQElement]) -> bool {
        elements.iter().all(|e| e.score.value() >= 80)
    }

    /// Checks if all 7 standard elements are present
    pub fn has_all_elements(elements: &[DQElement]) -> bool {
        DQ_ELEMENT_NAMES.iter().all(|name| {
            elements.iter().any(|e| e.name == *name)
        })
    }
}

#[derive(Debug, Clone)]
pub struct Improvement {
    pub element_name: String,
    pub current_score: Percentage,
    pub suggestion: String,
    pub priority: Priority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}
```

### TradeoffAnalyzer

Identifies tensions between alternatives for non-dominated options.

```rust
use crate::proact::{ConsequencesTable, DominatedAlternative, Tension};

/// Stateless service for tradeoff analysis
pub struct TradeoffAnalyzer;

impl TradeoffAnalyzer {
    /// Analyzes tensions for non-dominated alternatives
    pub fn analyze_tensions(
        table: &ConsequencesTable,
        dominated: &[DominatedAlternative],
    ) -> Vec<Tension> {
        let dominated_ids: Vec<_> = dominated.iter()
            .map(|d| &d.alternative_id)
            .collect();

        // Only analyze non-dominated alternatives
        let viable: Vec<_> = table.alternative_ids.iter()
            .filter(|id| !dominated_ids.contains(id))
            .collect();

        viable.into_iter()
            .map(|alt_id| Self::analyze_single_tension(table, alt_id, &dominated_ids))
            .collect()
    }

    fn analyze_single_tension(
        table: &ConsequencesTable,
        alt_id: &str,
        dominated_ids: &[&String],
    ) -> Tension {
        let mut gains = Vec::new();
        let mut losses = Vec::new();

        // Compare against other non-dominated alternatives
        for other_id in &table.alternative_ids {
            if other_id == alt_id || dominated_ids.contains(&other_id) {
                continue;
            }

            for obj_id in &table.objective_ids {
                let my_rating = table.get_cell(alt_id, obj_id)
                    .map(|c| c.rating.value())
                    .unwrap_or(0);
                let other_rating = table.get_cell(other_id, obj_id)
                    .map(|c| c.rating.value())
                    .unwrap_or(0);

                if my_rating > other_rating && !gains.contains(obj_id) {
                    gains.push(obj_id.clone());
                } else if my_rating < other_rating && !losses.contains(obj_id) {
                    losses.push(obj_id.clone());
                }
            }
        }

        Tension {
            alternative_id: alt_id.to_string(),
            gains,
            losses,
            uncertainty_impact: None,
        }
    }

    /// Summarizes tradeoffs in human-readable form
    pub fn summarize_tradeoffs(tensions: &[Tension]) -> TradeoffSummary {
        let total_alternatives = tensions.len();
        let has_clear_winner = tensions.iter()
            .any(|t| t.losses.is_empty() && !t.gains.is_empty());

        let most_balanced = tensions.iter()
            .min_by_key(|t| (t.gains.len() as i32 - t.losses.len() as i32).abs())
            .map(|t| t.alternative_id.clone());

        let most_polarizing = tensions.iter()
            .max_by_key(|t| t.gains.len() + t.losses.len())
            .map(|t| t.alternative_id.clone());

        TradeoffSummary {
            total_alternatives,
            has_clear_winner,
            most_balanced,
            most_polarizing,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TradeoffSummary {
    pub total_alternatives: usize,
    pub has_clear_winner: bool,
    pub most_balanced: Option<String>,
    pub most_polarizing: Option<String>,
}
```

### CellColor Value Object

Maps Pugh ratings to visual colors.

```rust
use crate::foundation::Rating;
use serde::{Deserialize, Serialize};

/// Visual color for Pugh matrix cells
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellColor {
    DarkBlue,   // +2, Much Better
    Blue,       // +1, Better
    Neutral,    // 0, Same
    Orange,     // -1, Worse
    Red,        // -2, Much Worse
}

impl CellColor {
    /// Converts a Pugh rating to a cell color
    pub fn from_rating(rating: Rating) -> Self {
        match rating {
            Rating::MuchBetter => CellColor::DarkBlue,
            Rating::Better => CellColor::Blue,
            Rating::Same => CellColor::Neutral,
            Rating::Worse => CellColor::Orange,
            Rating::MuchWorse => CellColor::Red,
        }
    }

    /// Returns CSS class name for styling
    pub fn to_css_class(&self) -> &'static str {
        match self {
            CellColor::DarkBlue => "cell-much-better",
            CellColor::Blue => "cell-better",
            CellColor::Neutral => "cell-same",
            CellColor::Orange => "cell-worse",
            CellColor::Red => "cell-much-worse",
        }
    }

    /// Returns hex color code
    pub fn to_hex_color(&self) -> &'static str {
        match self {
            CellColor::DarkBlue => "#1e40af",  // Blue-800
            CellColor::Blue => "#3b82f6",      // Blue-500
            CellColor::Neutral => "#9ca3af",   // Gray-400
            CellColor::Orange => "#f97316",    // Orange-500
            CellColor::Red => "#dc2626",       // Red-600
        }
    }

    /// Returns WCAG-compliant text color for contrast
    pub fn text_color(&self) -> &'static str {
        match self {
            CellColor::DarkBlue => "#ffffff",
            CellColor::Blue => "#ffffff",
            CellColor::Neutral => "#1f2937",
            CellColor::Orange => "#1f2937",
            CellColor::Red => "#ffffff",
        }
    }
}
```

---

## File Structure

```
backend/src/domain/analysis/
├── mod.rs                      # Module exports
├── pugh_analyzer.rs            # PughAnalyzer service
├── pugh_analyzer_test.rs       # Pugh tests
├── dq_calculator.rs            # DQCalculator service
├── dq_calculator_test.rs       # DQ tests
├── tradeoff_analyzer.rs        # TradeoffAnalyzer service
├── tradeoff_analyzer_test.rs   # Tradeoff tests
├── cell_color.rs               # CellColor value object
└── cell_color_test.rs          # CellColor tests

frontend/src/modules/analysis/
├── domain/
│   ├── pugh-matrix.ts          # TypeScript Pugh calculations
│   ├── pugh-matrix.test.ts
│   ├── dq-score.ts             # TypeScript DQ calculations
│   ├── dq-score.test.ts
│   └── cell-color.ts           # CellColor mapping
├── components/
│   ├── ConsequencesTable.tsx   # Pugh matrix display
│   ├── ConsequencesTable.test.tsx
│   ├── ConsequencesCell.tsx    # Individual cell with color
│   ├── DQGauge.tsx             # DQ score visualization
│   ├── DQGauge.test.tsx
│   ├── DQElementList.tsx       # List of DQ elements
│   └── TradeoffsChart.tsx      # Visual tradeoff display
└── index.ts
```

---

## Usage by Other Modules

### Cycle Module

```rust
// In cycle domain, after completing Consequences component
use crate::analysis::PughAnalyzer;

let consequences = cycle.get_component(ComponentType::Consequences);
let table = &consequences.output().table;

let dominated = PughAnalyzer::find_dominated(table);
let irrelevant = PughAnalyzer::find_irrelevant_objectives(table);
let scores = PughAnalyzer::compute_scores(table);

// Populate Tradeoffs component
let tensions = TradeoffAnalyzer::analyze_tensions(table, &dominated);
```

### Dashboard Module

```rust
// In dashboard for DQ badge
use crate::analysis::DQCalculator;

let dq_component = cycle.get_component(ComponentType::DecisionQuality);
let elements = &dq_component.output().elements;

let overall = DQCalculator::compute_overall(elements);
let is_acceptable = DQCalculator::is_acceptable(elements);
let improvements = DQCalculator::suggest_improvements(elements);
```

---

## Test Categories

### Unit Tests

| Category | Example Tests |
|----------|---------------|
| Pugh scores | `compute_scores_sums_ratings` |
| Pugh scores | `empty_table_returns_zero_scores` |
| Dominated | `find_dominated_detects_strict_dominance` |
| Dominated | `non_dominated_returns_empty` |
| Irrelevant | `uniform_ratings_marks_irrelevant` |
| DQ overall | `overall_is_minimum_score` |
| DQ overall | `empty_elements_returns_zero` |
| Weakest | `identify_weakest_finds_lowest` |
| Improvements | `suggest_improvements_for_weak_elements` |
| Tensions | `analyze_tensions_finds_gains_losses` |
| CellColor | `from_rating_maps_correctly` |

### Property-Based Tests

| Property | Description |
|----------|-------------|
| Dominance asymmetry | If A dominates B, B cannot dominate A |
| Transitivity | If A dominates B and B dominates C, A dominates C |
| DQ minimum | Overall score <= each element score |
| Color bijectivity | Each rating maps to exactly one color |

---

## Invariants

| Invariant | Enforcement |
|-----------|-------------|
| Pure functions | No state, no side effects |
| Deterministic | Same input always produces same output |
| No persistence | Services have no repository dependencies |
| Rating bounds | Uses Rating enum (compile-time) |
| Percentage bounds | Uses Percentage type (validated) |

---

*Module Version: 1.0.0*
*Based on: SYSTEM-ARCHITECTURE.md v1.1.0*
*Language: Rust*
