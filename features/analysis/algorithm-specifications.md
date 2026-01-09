# Feature: Analysis Algorithm Specifications

**Module:** analysis
**Type:** Domain Services (Pure Functions)
**Priority:** P0
**Status:** Specification Complete

> Complete algorithmic specifications for Pugh matrix analysis, dominance detection, DQ scoring, and tradeoff analysis - including all edge cases and boundary conditions.

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Not Applicable - pure computation functions |
| Authorization Model | Not Applicable - authorization at API boundary |
| Sensitive Data | Input data is Confidential (validated at boundary) |
| Rate Limiting | Not Applicable - applied at API layer |
| Audit Logging | Not Applicable - logging at API/handler layer |

### Input Validation Requirements

Although analysis functions are pure computations with no direct security responsibilities, **input validation MUST be performed at the API boundary** before invoking these functions:

| Input | Validation Rules |
|-------|------------------|
| Rating values | Must be -2, -1, 0, +1, or +2 (use Rating enum) |
| Percentage values | Must be 0-100 (use Percentage type) |
| Alternative IDs | Must be valid UUIDs, non-empty |
| Objective IDs | Must be valid UUIDs, non-empty |
| DQ element names | Must match DQ_ELEMENT_NAMES constant |
| Table dimensions | Max alternatives: 50, Max objectives: 50 |

### Data Classification

| Input/Output | Classification | Notes |
|--------------|----------------|-------|
| ConsequencesTable | Confidential | Contains user decision data |
| DQElement array | Confidential | User self-assessment |
| Computed scores | Confidential | Derived from user data |
| Dominated alternatives | Confidential | Analysis result |
| Tension analysis | Confidential | Analysis result |

### Security Notes for Implementers

1. **Do not log input data** - These functions receive Confidential user decision data
2. **Validate at boundary** - All validation happens in command handlers/API layer
3. **Type safety** - Use Rating enum and Percentage type to enforce bounds at compile time
4. **No external calls** - Pure functions must not make network calls or access external state
5. **Deterministic** - Same inputs must produce same outputs (no randomness)

---

## Overview

The Analysis module provides stateless computation functions. This specification defines:
1. Complete algorithm pseudocode
2. All edge cases and boundary conditions
3. Single-alternative behavior
4. Tie-breaking rules
5. Invalid input handling

---

## 1. Pugh Score Computation

### Algorithm

```rust
/// Computes Pugh scores for each alternative
///
/// # Algorithm
/// For each alternative:
///   score = Σ(rating[objective] × weight[objective])
///
/// Default weight = 1 for all objectives (unweighted mode)
///
/// # Edge Cases
/// - Empty table: Returns empty HashMap
/// - Single alternative: Returns score for that alternative
/// - No objectives: Returns 0 for all alternatives
/// - Missing cells: Treated as 0 (neutral)
fn compute_scores(table: &ConsequencesTable) -> HashMap<String, i32>
```

### Detailed Pseudocode

```
FUNCTION compute_scores(table):
    // Edge case: empty table
    IF table.alternative_ids.is_empty():
        RETURN empty HashMap

    IF table.objective_ids.is_empty():
        // All alternatives get score of 0
        RETURN alternative_ids.map(id -> (id, 0)).to_hashmap()

    scores = new HashMap<String, i32>

    FOR each alt_id IN table.alternative_ids:
        total = 0

        FOR each obj_id IN table.objective_ids:
            cell = table.get_cell(alt_id, obj_id)

            IF cell IS Some:
                // Rating values: -2, -1, 0, +1, +2
                total += cell.rating.value()
            ELSE:
                // Missing cell treated as neutral (0)
                total += 0
            END IF
        END FOR

        scores.insert(alt_id, total)
    END FOR

    RETURN scores
END FUNCTION
```

### Edge Cases Table

| Case | Input | Expected Output |
|------|-------|-----------------|
| Empty table | No alternatives, no objectives | `{}` (empty map) |
| No objectives | 3 alternatives, 0 objectives | `{a: 0, b: 0, c: 0}` |
| No alternatives | 0 alternatives, 3 objectives | `{}` (empty map) |
| Single alternative | 1 alternative, 3 objectives | Map with single entry |
| Missing cells | Sparse consequence table | Treat missing as 0 |
| All neutral ratings | All cells are 0 | All scores are 0 |
| Extreme positive | All cells are +2 | Max score = obj_count × 2 |
| Extreme negative | All cells are -2 | Min score = obj_count × -2 |

### Test Specifications

```rust
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
        .objectives(vec![]) // No objectives
        .build();

    let scores = PughAnalyzer::compute_scores(&table);
    assert_eq!(scores.get("alt-1"), Some(&0));
    assert_eq!(scores.get("alt-2"), Some(&0));
}

#[test]
fn compute_scores_single_alternative() {
    let table = ConsequencesTable::builder()
        .alternatives(vec!["alt-1"])
        .objectives(vec!["obj-1", "obj-2"])
        .cell("alt-1", "obj-1", Rating::Better)   // +1
        .cell("alt-1", "obj-2", Rating::MuchBetter) // +2
        .build();

    let scores = PughAnalyzer::compute_scores(&table);
    assert_eq!(scores.len(), 1);
    assert_eq!(scores.get("alt-1"), Some(&3));
}

#[test]
fn compute_scores_missing_cells() {
    let table = ConsequencesTable::builder()
        .alternatives(vec!["alt-1", "alt-2"])
        .objectives(vec!["obj-1", "obj-2", "obj-3"])
        // Only some cells filled
        .cell("alt-1", "obj-1", Rating::Better)   // +1
        // alt-1/obj-2 missing -> 0
        // alt-1/obj-3 missing -> 0
        .cell("alt-2", "obj-1", Rating::Worse)    // -1
        .cell("alt-2", "obj-2", Rating::Same)     // 0
        // alt-2/obj-3 missing -> 0
        .build();

    let scores = PughAnalyzer::compute_scores(&table);
    assert_eq!(scores.get("alt-1"), Some(&1));  // 1 + 0 + 0
    assert_eq!(scores.get("alt-2"), Some(&-1)); // -1 + 0 + 0
}
```

---

## 2. Dominance Detection

### Definition

**Alternative A dominates Alternative B** if and only if:
1. A ≥ B on ALL objectives (A is at least as good as B everywhere)
2. A > B on AT LEAST ONE objective (A is strictly better somewhere)

### Algorithm

```rust
/// Finds all dominated alternatives
///
/// # Returns
/// List of dominated alternatives with their dominator
///
/// # Edge Cases
/// - Empty table: Returns empty Vec
/// - Single alternative: Returns empty Vec (can't dominate self)
/// - Ties: Neither dominates if equal on all objectives
/// - Transitive: If A→B and B→C, only report A→B and B→C (not A→C)
fn find_dominated(table: &ConsequencesTable) -> Vec<DominatedAlternative>
```

### Detailed Pseudocode

```
FUNCTION find_dominated(table):
    // Edge case: fewer than 2 alternatives
    IF table.alternative_ids.len() < 2:
        RETURN empty Vec

    dominated = new Vec<DominatedAlternative>

    FOR each candidate IN table.alternative_ids:
        FOR each potential_dominator IN table.alternative_ids:
            IF candidate == potential_dominator:
                CONTINUE  // Can't dominate self

            IF dominates(table, potential_dominator, candidate):
                dominated.push(DominatedAlternative {
                    alternative_id: candidate,
                    dominated_by_id: potential_dominator,
                    explanation: explain_dominance(table, potential_dominator, candidate),
                })
                BREAK  // Only need to find ONE dominator per candidate
            END IF
        END FOR
    END FOR

    RETURN dominated
END FUNCTION

FUNCTION dominates(table, a, b):
    at_least_equal = true
    strictly_better_on_one = false

    FOR each obj_id IN table.objective_ids:
        a_rating = table.get_cell(a, obj_id)?.rating.value() OR 0
        b_rating = table.get_cell(b, obj_id)?.rating.value() OR 0

        IF a_rating < b_rating:
            at_least_equal = false
            BREAK  // A is worse somewhere - can't dominate
        END IF

        IF a_rating > b_rating:
            strictly_better_on_one = true
        END IF
    END FOR

    RETURN at_least_equal AND strictly_better_on_one
END FUNCTION
```

### Edge Cases Table

| Case | Input | Expected Output |
|------|-------|-----------------|
| Empty table | No alternatives | `[]` |
| Single alternative | 1 alternative | `[]` |
| All tied | All alternatives equal on all objectives | `[]` |
| Clear dominance | A > B on all | `[B dominated by A]` |
| Partial dominance | A > B on some, A = B on rest | `[B dominated by A]` |
| No dominance (tradeoffs) | A > B on obj-1, B > A on obj-2 | `[]` |
| Transitive | A→B and B→C | `[B by A, C by B]` (not C by A) |
| Multiple dominators | A→C and B→C | Either `[C by A]` or `[C by B]` (first found) |
| Missing cells | Sparse table | Treat missing as 0 |

### Special Cases

#### Tie on All Objectives
```
Alternative A: [0, 0, 0]
Alternative B: [0, 0, 0]
Result: Neither dominates (at_least_equal=true, strictly_better=false)
```

#### Weak Dominance (Equal with One Better)
```
Alternative A: [+1, 0, 0]
Alternative B: [ 0, 0, 0]
Result: A dominates B (A >= B everywhere, A > B on obj-1)
```

#### Near-Dominance (One Worse)
```
Alternative A: [+2, +1, -1]
Alternative B: [+1, +1, +1]
Result: Neither dominates (A worse on obj-3)
```

### Test Specifications

```rust
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
        // All Same (0) ratings
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
        .cell("A", "O1", Rating::MuchBetter)  // +2
        .cell("A", "O2", Rating::Better)       // +1
        .cell("A", "O3", Rating::Better)       // +1
        // B is worse on everything
        .cell("B", "O1", Rating::Same)         // 0
        .cell("B", "O2", Rating::Same)         // 0
        .cell("B", "O3", Rating::Worse)        // -1
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
        .cell("A", "Cost", Rating::MuchBetter)    // +2
        .cell("A", "Quality", Rating::Worse)       // -1
        // B: High cost, high quality
        .cell("B", "Cost", Rating::Worse)          // -1
        .cell("B", "Quality", Rating::MuchBetter)  // +2
        .build();

    let dominated = PughAnalyzer::find_dominated(&table);
    assert!(dominated.is_empty(), "Tradeoff alternatives don't dominate each other");
}

#[test]
fn find_dominated_weak_dominance() {
    let table = ConsequencesTable::builder()
        .alternatives(vec!["A", "B"])
        .objectives(vec!["O1", "O2", "O3"])
        // A: Better on one, same on rest
        .cell("A", "O1", Rating::Better)  // +1
        .cell("A", "O2", Rating::Same)    // 0
        .cell("A", "O3", Rating::Same)    // 0
        // B: Same on everything
        .cell("B", "O1", Rating::Same)    // 0
        .cell("B", "O2", Rating::Same)    // 0
        .cell("B", "O3", Rating::Same)    // 0
        .build();

    let dominated = PughAnalyzer::find_dominated(&table);
    assert_eq!(dominated.len(), 1);
    assert_eq!(dominated[0].alternative_id, "B");
}
```

---

## 3. Irrelevant Objectives Detection

### Definition

An **irrelevant objective** is one where all alternatives have the same rating. Such objectives don't help distinguish between alternatives.

### Algorithm

```rust
/// Finds objectives that don't distinguish between alternatives
///
/// # Edge Cases
/// - No objectives: Returns empty Vec
/// - Single alternative: All objectives are "irrelevant" but we return empty
///   (irrelevant detection only matters when comparing alternatives)
/// - All different: Returns empty Vec
fn find_irrelevant_objectives(table: &ConsequencesTable) -> Vec<IrrelevantObjective>
```

### Detailed Pseudocode

```
FUNCTION find_irrelevant_objectives(table):
    // Need at least 2 alternatives to compare
    IF table.alternative_ids.len() < 2:
        RETURN empty Vec

    irrelevant = new Vec<IrrelevantObjective>

    FOR each obj_id IN table.objective_ids:
        ratings = []

        FOR each alt_id IN table.alternative_ids:
            cell = table.get_cell(alt_id, obj_id)
            rating = cell?.rating.value() OR 0
            ratings.push(rating)
        END FOR

        // Check if all ratings are the same
        IF ratings.len() > 1 AND all_same(ratings):
            irrelevant.push(IrrelevantObjective {
                objective_id: obj_id,
                uniform_rating: ratings[0],
                reason: "All alternatives have the same rating",
            })
        END IF
    END FOR

    RETURN irrelevant
END FUNCTION

FUNCTION all_same(ratings):
    IF ratings.is_empty():
        RETURN true

    first = ratings[0]
    RETURN ratings.all(r -> r == first)
END FUNCTION
```

### Edge Cases Table

| Case | Input | Expected Output |
|------|-------|-----------------|
| Single alternative | 1 alt, any objectives | `[]` (no comparison possible) |
| All objectives vary | Each objective distinguishes | `[]` |
| One irrelevant | One objective uniform, rest vary | That objective only |
| All irrelevant | All objectives uniform | All objectives |
| Mixed with missing | Some cells missing | Treat missing as 0 |

### Test Specifications

```rust
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
}
```

---

## 4. Decision Quality Scoring

### The Seven DQ Elements

```rust
pub const DQ_ELEMENT_NAMES: &[&str] = &[
    "Helpful Problem Frame",
    "Clear Objectives",
    "Creative Alternatives",
    "Reliable Consequence Information",
    "Logically Correct Reasoning",
    "Clear Tradeoffs",
    "Commitment to Follow Through",
];
```

### Overall Score Algorithm

The overall DQ score is the **minimum** of all element scores. Decision quality is only as strong as its weakest element.

```rust
/// Computes overall DQ score
///
/// # Algorithm
/// overall = MIN(element_scores)
///
/// # Edge Cases
/// - No elements: Returns Percentage::ZERO
/// - One element: Returns that element's score
/// - Missing elements: Not validated here (see has_all_elements)
fn compute_overall(elements: &[DQElement]) -> Percentage
```

### Detailed Pseudocode

```
FUNCTION compute_overall(elements):
    // Edge case: no elements
    IF elements.is_empty():
        RETURN Percentage::ZERO  // 0%

    // Find minimum score
    min_score = 100  // Start with max possible

    FOR each element IN elements:
        IF element.score.value() < min_score:
            min_score = element.score.value()
        END IF
    END FOR

    RETURN Percentage::new(min_score)
END FUNCTION
```

### Element Validation

```rust
/// Checks if all 7 standard elements are present
fn has_all_elements(elements: &[DQElement]) -> bool {
    DQ_ELEMENT_NAMES.iter().all(|required_name| {
        elements.iter().any(|e| e.name == *required_name)
    })
}

/// Returns missing element names
fn missing_elements(elements: &[DQElement]) -> Vec<&'static str> {
    DQ_ELEMENT_NAMES.iter()
        .filter(|required| !elements.iter().any(|e| e.name == **required))
        .copied()
        .collect()
}
```

### Acceptability Threshold

```rust
/// Checks if decision quality is "acceptable"
/// Threshold: >= 80% on ALL elements
fn is_acceptable(elements: &[DQElement]) -> bool {
    !elements.is_empty() && elements.iter().all(|e| e.score.value() >= 80)
}

/// Returns the acceptability threshold
const DQ_ACCEPTABLE_THRESHOLD: u8 = 80;
```

### Improvement Priority Algorithm

```rust
/// Computes improvement priority based on score
fn compute_priority(score: Percentage) -> Priority {
    match score.value() {
        0..=30   => Priority::Critical,  // Urgent: needs immediate attention
        31..=50  => Priority::High,      // Important: should address soon
        51..=70  => Priority::Medium,    // Moderate: room for improvement
        71..=100 => Priority::Low,       // Good: minor enhancements possible
    }
}
```

### Edge Cases Table

| Case | Input | Overall Score | Notes |
|------|-------|---------------|-------|
| No elements | `[]` | 0% | Invalid state, should validate |
| All 100% | 7 elements at 100% | 100% | Perfect decision quality |
| One weak | 6×90%, 1×40% | 40% | Overall = weakest |
| Mixed | 80%, 60%, 90%, 70% | 60% | Minimum of all |
| Single element | 1 element at 75% | 75% | Single element = overall |
| All zeros | 7 elements at 0% | 0% | Completely inadequate |

### Test Specifications

```rust
#[test]
fn dq_overall_empty_elements() {
    let elements: Vec<DQElement> = vec![];
    let overall = DQCalculator::compute_overall(&elements);
    assert_eq!(overall.value(), 0);
}

#[test]
fn dq_overall_is_minimum() {
    let elements = vec![
        DQElement::new("Element A", 90),
        DQElement::new("Element B", 60),  // Minimum
        DQElement::new("Element C", 85),
    ];

    let overall = DQCalculator::compute_overall(&elements);
    assert_eq!(overall.value(), 60);
}

#[test]
fn dq_overall_single_element() {
    let elements = vec![DQElement::new("Only Element", 75)];
    let overall = DQCalculator::compute_overall(&elements);
    assert_eq!(overall.value(), 75);
}

#[test]
fn dq_has_all_elements_complete() {
    let elements = DQ_ELEMENT_NAMES.iter()
        .map(|name| DQElement::new(name, 80))
        .collect::<Vec<_>>();

    assert!(DQCalculator::has_all_elements(&elements));
}

#[test]
fn dq_has_all_elements_missing_one() {
    let elements = DQ_ELEMENT_NAMES[0..6].iter()  // Missing last element
        .map(|name| DQElement::new(name, 80))
        .collect::<Vec<_>>();

    assert!(!DQCalculator::has_all_elements(&elements));
}

#[test]
fn dq_is_acceptable_all_above_threshold() {
    let elements = vec![
        DQElement::new("A", 80),
        DQElement::new("B", 85),
        DQElement::new("C", 90),
    ];

    assert!(DQCalculator::is_acceptable(&elements));
}

#[test]
fn dq_is_acceptable_one_below_threshold() {
    let elements = vec![
        DQElement::new("A", 80),
        DQElement::new("B", 79),  // Below 80%
        DQElement::new("C", 90),
    ];

    assert!(!DQCalculator::is_acceptable(&elements));
}
```

---

## 5. Tradeoff Analysis

### Tension Computation Algorithm

```rust
/// Analyzes tensions for non-dominated alternatives
///
/// A tension exists when choosing one alternative means:
/// - Gaining on some objectives (where this alt is better than others)
/// - Losing on other objectives (where this alt is worse than others)
fn analyze_tensions(
    table: &ConsequencesTable,
    dominated: &[DominatedAlternative],
) -> Vec<Tension>
```

### Detailed Pseudocode

```
FUNCTION analyze_tensions(table, dominated):
    // Get IDs of dominated alternatives
    dominated_ids = dominated.map(d -> d.alternative_id).to_set()

    // Filter to non-dominated alternatives only
    viable = table.alternative_ids
        .filter(id -> !dominated_ids.contains(id))
        .collect()

    // Edge case: fewer than 2 viable alternatives
    IF viable.len() < 2:
        // Return empty tensions (no tradeoffs with < 2 options)
        // Or single tension with no gains/losses if exactly 1
        IF viable.len() == 1:
            RETURN vec![Tension {
                alternative_id: viable[0],
                gains: [],
                losses: [],
                uncertainty_impact: None,
            }]
        END IF
        RETURN []

    tensions = []

    FOR each alt_id IN viable:
        gains = Set::new()
        losses = Set::new()

        // Compare against OTHER non-dominated alternatives
        FOR each other_id IN viable:
            IF other_id == alt_id:
                CONTINUE

            FOR each obj_id IN table.objective_ids:
                my_rating = table.get_cell(alt_id, obj_id)?.rating.value() OR 0
                other_rating = table.get_cell(other_id, obj_id)?.rating.value() OR 0

                IF my_rating > other_rating:
                    gains.insert(obj_id)
                ELSE IF my_rating < other_rating:
                    losses.insert(obj_id)
                END IF
            END FOR
        END FOR

        tensions.push(Tension {
            alternative_id: alt_id,
            gains: gains.to_vec(),
            losses: losses.to_vec(),
            uncertainty_impact: None,  // Future: integrate uncertainty
        })
    END FOR

    RETURN tensions
END FUNCTION
```

### Tradeoff Summary Algorithm

```rust
/// Summarizes tradeoff analysis results
fn summarize_tradeoffs(tensions: &[Tension]) -> TradeoffSummary {
    // Clear winner: has gains but NO losses
    let has_clear_winner = tensions.iter()
        .any(|t| !t.gains.is_empty() && t.losses.is_empty());

    // Most balanced: smallest absolute difference between gains and losses
    let most_balanced = tensions.iter()
        .min_by_key(|t| (t.gains.len() as i32 - t.losses.len() as i32).abs())
        .map(|t| t.alternative_id.clone());

    // Most polarizing: largest total of gains + losses
    let most_polarizing = tensions.iter()
        .max_by_key(|t| t.gains.len() + t.losses.len())
        .map(|t| t.alternative_id.clone());

    TradeoffSummary {
        total_alternatives: tensions.len(),
        has_clear_winner,
        most_balanced,
        most_polarizing,
    }
}
```

### Edge Cases Table

| Case | Input | Expected Output |
|------|-------|-----------------|
| All dominated | All alternatives dominated | Empty tensions |
| Single non-dominated | 1 viable alternative | Single tension, empty gains/losses |
| Clear winner | One alt better on all vs non-dominated peers | `has_clear_winner = true` |
| Pure tradeoffs | Each alt trades gains for losses | No clear winner |
| No objectives | Viable alts but no objectives | Empty gains/losses for all |

### Test Specifications

```rust
#[test]
fn tradeoffs_all_dominated() {
    let table = three_alternative_table();
    // All alternatives dominated by hypothetical fourth
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
        // A: Best on everything vs B and C (but B and C trade off)
        .cell("A", "O1", Rating::MuchBetter)
        .cell("A", "O2", Rating::Better)
        .cell("A", "O3", Rating::Better)
        // B: Medium
        .cell("B", "O1", Rating::Same)
        .cell("B", "O2", Rating::Better)  // B better than C on O2
        .cell("B", "O3", Rating::Worse)   // B worse than C on O3
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
    }
}
```

---

## 6. Single Alternative Behavior

When only one alternative exists, the analysis module should handle gracefully:

### Pugh Scores
```rust
// Single alternative: compute score normally
let scores = compute_scores(table); // Returns { "only_alt": score }
```

### Dominance Detection
```rust
// Single alternative: cannot be dominated (need 2+ to compare)
let dominated = find_dominated(table); // Returns []
```

### Irrelevant Objectives
```rust
// Single alternative: all objectives are "irrelevant" for comparison
// but we return empty because irrelevancy only matters when comparing
let irrelevant = find_irrelevant_objectives(table); // Returns []
```

### Tradeoff Analysis
```rust
// Single alternative: no tradeoffs possible
let tensions = analyze_tensions(table, &[]);
// Returns [Tension { alt_id: "only", gains: [], losses: [] }]
```

### Dashboard Implications
- DQ score computed normally (single alternative doesn't affect DQ)
- Consequences table shows single column
- No dominated alternatives badge
- Recommendation should note "single alternative - consider generating more"

---

## 7. Frontend/Backend Parity

### Shared Type Definitions

Frontend TypeScript types MUST mirror backend Rust types exactly:

```typescript
// frontend/src/modules/analysis/domain/pugh-matrix.ts

export type Rating = -2 | -1 | 0 | 1 | 2;

export interface ConsequencesTable {
  alternativeIds: string[];
  objectiveIds: string[];
  cells: Map<string, Map<string, Cell>>;
}

export interface Cell {
  rating: Rating;
  explanation?: string;
}

export interface DominatedAlternative {
  alternativeId: string;
  dominatedById: string;
  explanation: string;
}

export interface IrrelevantObjective {
  objectiveId: string;
  uniformRating: Rating;
  reason: string;
}

export interface Tension {
  alternativeId: string;
  gains: string[];
  losses: string[];
  uncertaintyImpact?: string;
}

export interface TradeoffSummary {
  totalAlternatives: number;
  hasClearWinner: boolean;
  mostBalanced?: string;
  mostPolarizing?: string;
}

// frontend/src/modules/analysis/domain/dq-score.ts

export const DQ_ELEMENT_NAMES = [
  "Helpful Problem Frame",
  "Clear Objectives",
  "Creative Alternatives",
  "Reliable Consequence Information",
  "Logically Correct Reasoning",
  "Clear Tradeoffs",
  "Commitment to Follow Through",
] as const;

export type DQElementName = typeof DQ_ELEMENT_NAMES[number];

export interface DQElement {
  name: DQElementName;
  score: number; // 0-100
  rationale: string;
  improvement: string;
}

export interface DQScores {
  elements: DQElement[];
  overall: number;
  weakestElement: string;
  isAcceptable: boolean;
}

export type ImprovementPriority = 'critical' | 'high' | 'medium' | 'low';
```

### Frontend Computation Functions

Frontend MAY perform calculations for immediate UI responsiveness, but backend is authoritative:

```typescript
// frontend/src/modules/analysis/domain/pugh-matrix.ts

export function computeScores(table: ConsequencesTable): Map<string, number> {
  const scores = new Map<string, number>();

  for (const altId of table.alternativeIds) {
    let total = 0;
    for (const objId of table.objectiveIds) {
      const rating = table.cells.get(altId)?.get(objId)?.rating ?? 0;
      total += rating;
    }
    scores.set(altId, total);
  }

  return scores;
}

// DQ calculation
export function computeOverallDQ(elements: DQElement[]): number {
  if (elements.length === 0) return 0;
  return Math.min(...elements.map(e => e.score));
}

export function isAcceptableDQ(elements: DQElement[]): boolean {
  return elements.length > 0 && elements.every(e => e.score >= 80);
}
```

### API Response Matching

Backend API responses use camelCase for JSON (matching frontend):

```rust
// Backend serialization
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PughScoresResponse {
    pub alternative_scores: HashMap<String, i32>,
    pub dominated_alternatives: Vec<DominatedAlternativeDto>,
    pub irrelevant_objectives: Vec<String>,
    pub best_alternative_id: Option<String>,
}
```

### Test Parity

Both frontend and backend should produce identical results for identical inputs:

```typescript
// frontend/src/modules/analysis/domain/__tests__/pugh-parity.test.ts

describe('Pugh Matrix Frontend/Backend Parity', () => {
  const testCases = [
    {
      name: 'empty table',
      table: emptyTable(),
      expectedScores: new Map(),
    },
    {
      name: 'single alternative',
      table: singleAlternativeTable(),
      expectedScores: new Map([['alt-1', 3]]),
    },
    // ... more cases matching backend tests
  ];

  for (const tc of testCases) {
    it(`computes same scores as backend: ${tc.name}`, () => {
      const scores = computeScores(tc.table);
      expect(scores).toEqual(tc.expectedScores);
    });
  }
});
```

---

## Invariants Summary

| Invariant | Enforcement |
|-----------|-------------|
| Pure functions | No state, no side effects, deterministic |
| Rating bounds | Rating enum constrains to -2..+2 |
| Percentage bounds | Percentage type constrains to 0..100 |
| Score is sum | No weights applied (future enhancement) |
| Overall DQ = minimum | Never average, always minimum |
| Dominance requires 2+ | Single alternative cannot be dominated |
| Irrelevancy requires 2+ | Single alternative has no irrelevant objectives |
| Missing cells = 0 | Sparse tables treated as neutral |

---

*Version: 1.0.0*
*Created: 2026-01-08*
*Module: analysis*
