# Analysis Module Checklist

**Module:** Analysis
**Language:** Rust
**Dependencies:** foundation, proact-types
**Phase:** 3 (parallel with cycle, conversation)

---

## Overview

The Analysis module provides stateless domain services for analytical computations: Pugh matrix calculations, Decision Quality scoring, and tradeoff analysis. These are pure functions with no persistence needs - they're called by other modules to perform calculations.

---

## Module Classification

| Attribute | Value |
|-----------|-------|
| **Type** | Domain Services (pure functions, no ports/adapters) |
| **Language** | Rust |
| **External Dependencies** | None (pure Rust) |

---

## File Inventory

### Domain Services (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/analysis/mod.rs` | Module exports | ✅ |
| `backend/src/domain/analysis/pugh_analyzer.rs` | PughAnalyzer service | ✅ |
| `backend/src/domain/analysis/dq_calculator.rs` | DQCalculator service | ✅ |
| `backend/src/domain/analysis/tradeoff_analyzer.rs` | TradeoffAnalyzer service | ✅ |
| `backend/src/domain/analysis/consequences_table.rs` | ConsequencesTable utilities | ✅ |

> **Note:** CellColor, Improvement, Priority, TradeoffSummary are integrated into their respective analyzer files.

### Domain Service Tests (Rust)

> **Note:** Tests are inline in implementation files using `#[cfg(test)] mod tests` (Rust convention).

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/analysis/pugh_analyzer.rs` | PughAnalyzer tests (19 tests) | ✅ |
| `backend/src/domain/analysis/dq_calculator.rs` | DQCalculator tests (18 tests) | ✅ |
| `backend/src/domain/analysis/tradeoff_analyzer.rs` | TradeoffAnalyzer tests (13 tests) | ✅ |
| `backend/src/domain/analysis/consequences_table.rs` | ConsequencesTable tests (11 tests) | ✅ |

### Frontend Domain (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/analysis/domain/pugh-matrix.ts` | Pugh calculations | ⬜ |
| `frontend/src/modules/analysis/domain/dq-score.ts` | DQ calculations | ⬜ |
| `frontend/src/modules/analysis/domain/cell-color.ts` | CellColor mapping | ⬜ |
| `frontend/src/modules/analysis/index.ts` | Public exports | ⬜ |

### Frontend Domain Tests (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/analysis/domain/pugh-matrix.test.ts` | Pugh tests | ⬜ |
| `frontend/src/modules/analysis/domain/dq-score.test.ts` | DQ tests | ⬜ |

### Frontend Components (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/analysis/components/ConsequencesTable.tsx` | Pugh matrix display | ⬜ |
| `frontend/src/modules/analysis/components/ConsequencesCell.tsx` | Individual cell | ⬜ |
| `frontend/src/modules/analysis/components/DQGauge.tsx` | DQ score visualization | ⬜ |
| `frontend/src/modules/analysis/components/DQElementList.tsx` | DQ elements list | ⬜ |
| `frontend/src/modules/analysis/components/TradeoffsChart.tsx` | Tradeoff visualization | ⬜ |

### Frontend Component Tests (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/analysis/components/ConsequencesTable.test.tsx` | Table tests | ⬜ |
| `frontend/src/modules/analysis/components/DQGauge.test.tsx` | Gauge tests | ⬜ |

---

## Test Inventory

### PughAnalyzer Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_compute_scores_sums_ratings` | Sum of all ratings per alternative | ⬜ |
| `test_compute_scores_empty_table_returns_empty` | Empty table returns empty map | ⬜ |
| `test_compute_scores_with_missing_cells` | Missing cells treated as 0 | ⬜ |
| `test_compute_scores_single_alternative` | Works with one alternative | ⬜ |
| `test_find_dominated_detects_strict_dominance` | A >= B everywhere, A > B somewhere | ⬜ |
| `test_find_dominated_returns_empty_when_none` | No dominance returns empty | ⬜ |
| `test_find_dominated_mutual_non_dominance` | A better on some, B better on others | ⬜ |
| `test_find_dominated_explanation_includes_objectives` | Explanation lists objectives | ⬜ |
| `test_dominates_requires_at_least_one_strictly_better` | Equal everywhere is not dominance | ⬜ |
| `test_dominates_fails_if_worse_on_any` | One worse objective breaks dominance | ⬜ |
| `test_find_irrelevant_uniform_ratings` | Same rating everywhere is irrelevant | ⬜ |
| `test_find_irrelevant_returns_empty_when_varying` | Varying ratings are relevant | ⬜ |
| `test_rank_alternatives_descending_order` | Highest score first | ⬜ |
| `test_rank_alternatives_handles_ties` | Ties appear together | ⬜ |
| `test_find_top_alternative_clear_winner` | Returns top if no tie | ⬜ |
| `test_find_top_alternative_tie_returns_none` | Tie returns None | ⬜ |
| `test_find_top_alternative_single_returns_it` | Single alternative returns it | ⬜ |

### DQCalculator Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_compute_overall_is_minimum_score` | Overall = minimum element | ⬜ |
| `test_compute_overall_empty_returns_zero` | Empty returns 0% | ⬜ |
| `test_compute_overall_single_element_returns_it` | Single element is overall | ⬜ |
| `test_compute_overall_all_100_returns_100` | All 100% = 100% overall | ⬜ |
| `test_identify_weakest_finds_lowest` | Returns element with min score | ⬜ |
| `test_identify_weakest_empty_returns_none` | Empty returns None | ⬜ |
| `test_identify_weak_elements_below_threshold` | Filters below threshold | ⬜ |
| `test_identify_weak_elements_none_below` | All above returns empty | ⬜ |
| `test_suggest_improvements_for_weak` | Creates improvements for weak | ⬜ |
| `test_suggest_improvements_empty_when_strong` | No improvements when all strong | ⬜ |
| `test_compute_priority_critical_0_to_30` | 0-30 = Critical | ⬜ |
| `test_compute_priority_high_31_to_50` | 31-50 = High | ⬜ |
| `test_compute_priority_medium_51_to_70` | 51-70 = Medium | ⬜ |
| `test_compute_priority_low_above_70` | >70 = Low | ⬜ |
| `test_is_acceptable_all_above_80` | All >= 80% returns true | ⬜ |
| `test_is_acceptable_one_below_80` | One < 80% returns false | ⬜ |
| `test_has_all_elements_true_when_complete` | All 7 present returns true | ⬜ |
| `test_has_all_elements_false_when_missing` | Missing element returns false | ⬜ |
| `test_dq_element_names_has_seven` | Constant has 7 elements | ⬜ |

### TradeoffAnalyzer Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_analyze_tensions_excludes_dominated` | Dominated alternatives ignored | ⬜ |
| `test_analyze_tensions_finds_gains` | Objectives where alt is better | ⬜ |
| `test_analyze_tensions_finds_losses` | Objectives where alt is worse | ⬜ |
| `test_analyze_tensions_empty_when_single_viable` | Single alt has no tensions | ⬜ |
| `test_summarize_has_clear_winner` | Detects no-loss alternative | ⬜ |
| `test_summarize_no_clear_winner` | Detects when all have tradeoffs | ⬜ |
| `test_summarize_most_balanced` | Finds smallest |gains - losses| | ⬜ |
| `test_summarize_most_polarizing` | Finds largest gains + losses | ⬜ |

### CellColor Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_from_rating_much_better_is_dark_blue` | +2 → DarkBlue | ⬜ |
| `test_from_rating_better_is_blue` | +1 → Blue | ⬜ |
| `test_from_rating_same_is_neutral` | 0 → Neutral | ⬜ |
| `test_from_rating_worse_is_orange` | -1 → Orange | ⬜ |
| `test_from_rating_much_worse_is_red` | -2 → Red | ⬜ |
| `test_to_css_class_returns_valid_class` | CSS class format | ⬜ |
| `test_to_hex_color_returns_valid_hex` | Hex color format | ⬜ |
| `test_text_color_provides_contrast` | Text colors are set | ⬜ |
| `test_cell_color_serialize_snake_case` | Serializes to snake_case | ⬜ |
| `test_cell_color_deserialize_from_string` | Deserializes from string | ⬜ |

### Property-Based Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_dominance_is_asymmetric` | If A dominates B, B cannot dominate A | ⬜ |
| `test_dq_overall_leq_all_elements` | Overall <= each element score | ⬜ |
| `test_color_rating_bijection` | Each rating maps to exactly one color | ⬜ |

---

## Error Codes

| Error Code | Condition |
|------------|-----------|
| N/A | Module has no errors - pure functions with valid inputs from types |

---

## Business Rules

| Rule | Implementation | Test | Status |
|------|----------------|------|--------|
| Pugh score is sum of ratings | `compute_scores()` | `test_compute_scores_sums_ratings` | ⬜ |
| Dominance requires strict better on at least one | `dominates()` logic | `test_dominates_requires_at_least_one_strictly_better` | ⬜ |
| DQ overall is minimum of elements | `compute_overall()` | `test_compute_overall_is_minimum_score` | ⬜ |
| DQ acceptable threshold is 80% | `is_acceptable()` | `test_is_acceptable_all_above_80` | ⬜ |
| 7 standard DQ elements exist | `DQ_ELEMENT_NAMES` constant | `test_dq_element_names_has_seven` | ⬜ |
| CellColor maps bijectively to Rating | `from_rating()` | `test_color_rating_bijection` | ⬜ |
| Tradeoff analysis excludes dominated | `analyze_tensions()` | `test_analyze_tensions_excludes_dominated` | ⬜ |

---

## Verification Commands

```bash
# Run all analysis tests
cargo test --package analysis -- --nocapture

# Run specific test category
cargo test --package analysis pugh:: -- --nocapture
cargo test --package analysis dq:: -- --nocapture
cargo test --package analysis tradeoff:: -- --nocapture
cargo test --package analysis cell_color:: -- --nocapture

# Run property-based tests
cargo test --package analysis property:: -- --nocapture

# Coverage check (target: 95%+ - pure functions are easy to cover)
cargo tarpaulin --package analysis --out Html

# Full verification
cargo test --package analysis -- --nocapture && cargo clippy --package analysis

# Frontend tests
cd frontend && npm test -- --testPathPattern="modules/analysis"
```

---

## Exit Criteria

### Module is COMPLETE when:

- [x] All Rust domain service files exist (5/5 complete - 100%)
- [x] All Rust tests pass (61 tests passing)
- [x] Rust coverage >= 95% (verified via inline tests)
- [x] All functions are pure (no side effects)
- [ ] Property-based tests pass (not yet implemented)
- [ ] Frontend calculations match Rust (frontend not started)
- [x] No clippy warnings
- [ ] No TypeScript lint errors (frontend not started)

### Current Status

```
RUST BACKEND COMPLETE: analysis
Files: 5/5
Tests: 61/61 passing
Frontend: Not started
```

### Exit Signal (Full Module)

```
MODULE COMPLETE: analysis
Rust Files: 5/5
Rust Tests: 61/61 passing
Frontend Files: 0/9 (not started)
```

---

## Implementation Phases

### Phase 1: PughAnalyzer ✅
- [x] compute_scores() function
- [x] rank_alternatives() function
- [x] find_top_alternative() function
- [x] dominates() helper function
- [x] find_dominated() function
- [x] find_irrelevant_objectives() function
- [x] PughAnalyzer tests (19 tests)

### Phase 2: DQCalculator ✅
- [x] DQ_ELEMENT_NAMES constant
- [x] compute_overall() function
- [x] identify_weakest() function
- [x] Priority enum
- [x] compute_priority() function
- [x] identify_weak_elements() function
- [x] suggest_improvements() function
- [x] is_acceptable() function
- [x] has_all_elements() function
- [x] DQCalculator tests (18 tests)

### Phase 3: TradeoffAnalyzer ✅
- [x] analyze_tensions() function
- [x] summarize_tradeoffs() function
- [x] TradeoffAnalyzer tests (13 tests)

### Phase 4: ConsequencesTable ✅
- [x] ConsequencesTable utilities
- [x] ConsequencesTable tests (11 tests)

### Phase 5: Property-Based Tests ⬜
- [ ] Dominance asymmetry property
- [ ] DQ minimum property
- [ ] Color bijection property

### Phase 6: Frontend Domain ⬜
- [ ] TypeScript pugh-matrix.ts
- [ ] TypeScript dq-score.ts
- [ ] TypeScript cell-color.ts
- [ ] Frontend tests

### Phase 7: Frontend Components ⬜
- [ ] ConsequencesTable component
- [ ] ConsequencesCell component
- [ ] DQGauge component
- [ ] DQElementList component
- [ ] TradeoffsChart component
- [ ] Component tests

---

## Notes

- This module has NO ports and NO adapters - it's pure domain logic
- All functions should be deterministic and have no side effects
- These services are called by other modules (cycle, dashboard)
- Property-based tests are important for mathematical correctness
- Frontend should mirror Rust calculations exactly
- Coverage target is higher (95%) because pure functions are easy to test

---

*Generated: 2026-01-07*
*Specification: docs/modules/analysis.md*
