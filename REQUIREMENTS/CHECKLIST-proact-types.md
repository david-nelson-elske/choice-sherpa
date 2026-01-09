# PrOACT Types Module Checklist

**Module:** PrOACT Types
**Language:** Rust
**Dependencies:** foundation
**Phase:** 2 (parallel with session)

---

## Overview

The PrOACT Types module defines the 9 PrOACT component types and their structured outputs. These are domain types used by the `cycle` module for persistence and by the `conversation` module for data extraction. This is a shared domain library, not a full module with ports/adapters.

---

## File Inventory

### Domain Layer (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/proact/mod.rs` | Module exports | ⬜ |
| `backend/src/domain/proact/component.rs` | Component trait + ComponentBase | ⬜ |
| `backend/src/domain/proact/component_variant.rs` | ComponentVariant enum | ⬜ |
| `backend/src/domain/proact/message.rs` | Message, MessageId, Role | ⬜ |
| `backend/src/domain/proact/issue_raising.rs` | IssueRaising component | ⬜ |
| `backend/src/domain/proact/problem_frame.rs` | ProblemFrame component | ⬜ |
| `backend/src/domain/proact/objectives.rs` | Objectives component | ⬜ |
| `backend/src/domain/proact/alternatives.rs` | Alternatives component | ⬜ |
| `backend/src/domain/proact/consequences.rs` | Consequences component | ⬜ |
| `backend/src/domain/proact/tradeoffs.rs` | Tradeoffs component | ⬜ |
| `backend/src/domain/proact/recommendation.rs` | Recommendation component | ⬜ |
| `backend/src/domain/proact/decision_quality.rs` | DecisionQuality component | ⬜ |
| `backend/src/domain/proact/notes_next_steps.rs` | NotesNextSteps component | ⬜ |
| `backend/src/domain/proact/errors.rs` | ComponentError enum | ⬜ |

### Domain Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/proact/component_test.rs` | Component trait tests | ⬜ |
| `backend/src/domain/proact/component_variant_test.rs` | ComponentVariant tests | ⬜ |
| `backend/src/domain/proact/message_test.rs` | Message tests | ⬜ |
| `backend/src/domain/proact/issue_raising_test.rs` | IssueRaising tests | ⬜ |
| `backend/src/domain/proact/problem_frame_test.rs` | ProblemFrame tests | ⬜ |
| `backend/src/domain/proact/objectives_test.rs` | Objectives tests | ⬜ |
| `backend/src/domain/proact/alternatives_test.rs` | Alternatives tests | ⬜ |
| `backend/src/domain/proact/consequences_test.rs` | Consequences tests | ⬜ |
| `backend/src/domain/proact/tradeoffs_test.rs` | Tradeoffs tests | ⬜ |
| `backend/src/domain/proact/recommendation_test.rs` | Recommendation tests | ⬜ |
| `backend/src/domain/proact/decision_quality_test.rs` | DecisionQuality tests | ⬜ |
| `backend/src/domain/proact/notes_next_steps_test.rs` | NotesNextSteps tests | ⬜ |

### Frontend Types (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/shared/proact/component.ts` | Component interface | ⬜ |
| `frontend/src/shared/proact/message.ts` | Message type | ⬜ |
| `frontend/src/shared/proact/issue-raising.ts` | IssueRaising types | ⬜ |
| `frontend/src/shared/proact/problem-frame.ts` | ProblemFrame types | ⬜ |
| `frontend/src/shared/proact/objectives.ts` | Objectives types | ⬜ |
| `frontend/src/shared/proact/alternatives.ts` | Alternatives types | ⬜ |
| `frontend/src/shared/proact/consequences.ts` | Consequences types | ⬜ |
| `frontend/src/shared/proact/tradeoffs.ts` | Tradeoffs types | ⬜ |
| `frontend/src/shared/proact/recommendation.ts` | Recommendation types | ⬜ |
| `frontend/src/shared/proact/decision-quality.ts` | DecisionQuality types | ⬜ |
| `frontend/src/shared/proact/notes-next-steps.ts` | NotesNextSteps types | ⬜ |
| `frontend/src/shared/proact/index.ts` | Public exports | ⬜ |

---

## Test Inventory

### Component Trait Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_component_base_new_sets_not_started` | New component has NotStarted status | ⬜ |
| `test_component_base_new_generates_unique_id` | Each call produces different ID | ⬜ |
| `test_component_base_start_transitions_to_in_progress` | Start changes status | ⬜ |
| `test_component_base_start_from_not_started_succeeds` | Valid transition | ⬜ |
| `test_component_base_start_from_complete_fails` | Invalid transition | ⬜ |
| `test_component_base_complete_transitions_to_complete` | Complete changes status | ⬜ |
| `test_component_base_complete_from_in_progress_succeeds` | Valid transition | ⬜ |
| `test_component_base_complete_from_not_started_fails` | Invalid transition | ⬜ |
| `test_component_base_mark_for_revision_sets_reason` | Reason is stored | ⬜ |
| `test_component_base_start_updates_timestamp` | Start updates updated_at | ⬜ |
| `test_component_base_complete_updates_timestamp` | Complete updates updated_at | ⬜ |

### ComponentVariant Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_component_variant_new_creates_correct_type` | Factory produces correct variant | ⬜ |
| `test_component_variant_component_type_returns_correct` | Type accessor works | ⬜ |
| `test_component_variant_status_returns_inner_status` | Status accessor works | ⬜ |
| `test_component_variant_all_nine_types_constructible` | All 9 types can be created | ⬜ |

### Message Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_message_id_new_generates_unique` | Each call produces different ID | ⬜ |
| `test_message_user_has_user_role` | Factory sets correct role | ⬜ |
| `test_message_assistant_has_assistant_role` | Factory sets correct role | ⬜ |
| `test_message_system_has_system_role` | Factory sets correct role | ⬜ |
| `test_message_has_timestamp` | Messages have timestamps | ⬜ |
| `test_message_metadata_default_is_empty` | Default metadata | ⬜ |
| `test_role_serializes_snake_case` | "user" not "User" | ⬜ |

### IssueRaising Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_issue_raising_new_has_empty_output` | Initial output is empty | ⬜ |
| `test_issue_raising_add_potential_decision_appends` | Adding works | ⬜ |
| `test_issue_raising_add_decision_updates_timestamp` | Timestamp updates | ⬜ |
| `test_issue_raising_confirm_sets_user_confirmed` | Confirm works | ⬜ |
| `test_issue_raising_output_as_value_serializes` | JSON export works | ⬜ |
| `test_issue_raising_set_output_from_value_deserializes` | JSON import works | ⬜ |

### ProblemFrame Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_problem_frame_new_has_empty_output` | Initial output is empty | ⬜ |
| `test_problem_frame_set_decision_statement_works` | Statement setter | ⬜ |
| `test_problem_frame_add_party_appends` | Adding party works | ⬜ |
| `test_problem_frame_add_constraint_appends` | Adding constraint works | ⬜ |
| `test_linked_decision_serialize_roundtrip` | Nested type serializes | ⬜ |
| `test_decision_hierarchy_serialize_roundtrip` | Nested type serializes | ⬜ |

### Objectives Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_objectives_new_has_empty_output` | Initial output is empty | ⬜ |
| `test_objectives_add_fundamental_appends` | Adding works | ⬜ |
| `test_objectives_add_means_appends` | Adding works | ⬜ |
| `test_objectives_fundamental_count_correct` | Count is accurate | ⬜ |
| `test_performance_measure_serialize_roundtrip` | Nested type serializes | ⬜ |

### Alternatives Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_alternatives_new_has_empty_output` | Initial output is empty | ⬜ |
| `test_alternatives_add_alternative_appends` | Adding works | ⬜ |
| `test_alternatives_add_status_quo_sets_flag` | Status quo flag | ⬜ |
| `test_alternatives_set_strategy_table_works` | Strategy table setter | ⬜ |
| `test_alternatives_count_correct` | Count is accurate | ⬜ |
| `test_strategy_table_serialize_roundtrip` | Nested type serializes | ⬜ |

### Consequences Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_consequences_new_has_empty_table` | Initial table is empty | ⬜ |
| `test_consequences_set_cell_creates_entry` | Setting cell works | ⬜ |
| `test_consequences_get_cell_retrieves_entry` | Getting cell works | ⬜ |
| `test_consequences_get_cell_returns_none_for_missing` | Missing returns None | ⬜ |
| `test_consequences_add_uncertainty_appends` | Adding works | ⬜ |
| `test_cell_rating_validates_range` | Rating validation | ⬜ |
| `test_cell_serialize_roundtrip` | Cell serializes | ⬜ |

### Tradeoffs Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_tradeoffs_new_has_empty_output` | Initial output is empty | ⬜ |
| `test_tradeoffs_add_dominated_appends` | Adding works | ⬜ |
| `test_tradeoffs_add_irrelevant_appends` | Adding works | ⬜ |
| `test_tradeoffs_add_tension_appends` | Adding works | ⬜ |
| `test_tradeoffs_viable_count_subtracts_dominated` | Count calculation | ⬜ |

### Recommendation Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_recommendation_new_has_empty_output` | Initial output is empty | ⬜ |
| `test_recommendation_set_synthesis_works` | Synthesis setter | ⬜ |
| `test_recommendation_set_standout_works` | Standout setter | ⬜ |
| `test_recommendation_add_caveat_appends` | Adding works | ⬜ |
| `test_recommendation_has_standout_returns_true` | Predicate works | ⬜ |
| `test_recommendation_has_standout_returns_false` | Predicate works | ⬜ |

### DecisionQuality Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_dq_new_has_zero_overall` | Initial score is 0 | ⬜ |
| `test_dq_set_element_adds_element` | Adding works | ⬜ |
| `test_dq_set_element_replaces_existing` | Replacement works | ⬜ |
| `test_dq_recalculate_overall_uses_minimum` | Overall = min of elements | ⬜ |
| `test_dq_is_perfect_for_all_100` | Perfect detection | ⬜ |
| `test_dq_weakest_element_returns_lowest` | Weakest finder | ⬜ |
| `test_dq_element_names_constant_has_seven` | 7 standard elements | ⬜ |

### NotesNextSteps Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_notes_new_has_empty_output` | Initial output is empty | ⬜ |
| `test_notes_add_action_appends` | Adding works | ⬜ |
| `test_notes_add_open_question_appends` | Adding works | ⬜ |
| `test_notes_set_affirmation_works` | Affirmation setter | ⬜ |
| `test_notes_action_count_correct` | Count is accurate | ⬜ |
| `test_planned_action_serialize_roundtrip` | Nested type serializes | ⬜ |

---

## Error Codes

| Error Code | Condition |
|------------|-----------|
| `INVALID_TRANSITION` | Invalid status transition attempt |
| `INVALID_OUTPUT` | Output data failed validation/parsing |
| `NOT_STARTED` | Action requires started component |
| `ALREADY_COMPLETE` | Action not allowed on complete component |

---

## Business Rules

| Rule | Implementation | Test | Status |
|------|----------------|------|--------|
| Status transitions must be valid | `can_transition_to()` check | `test_component_base_start_from_complete_fails` | ⬜ |
| DQ overall score is minimum | `recalculate_overall()` | `test_dq_recalculate_overall_uses_minimum` | ⬜ |
| Only 9 component types exist | Enum variants | `test_component_variant_all_nine_types_constructible` | ⬜ |
| Messages have timestamps | Constructor sets now | `test_message_has_timestamp` | ⬜ |
| ComponentBase updates timestamp on mutation | All mutators call Timestamp::now() | `test_component_base_start_updates_timestamp` | ⬜ |

---

## Verification Commands

```bash
# Run all proact tests
cargo test --package proact-types -- --nocapture

# Run specific component tests
cargo test --package proact-types issue_raising:: -- --nocapture
cargo test --package proact-types problem_frame:: -- --nocapture
cargo test --package proact-types objectives:: -- --nocapture
cargo test --package proact-types alternatives:: -- --nocapture
cargo test --package proact-types consequences:: -- --nocapture
cargo test --package proact-types tradeoffs:: -- --nocapture
cargo test --package proact-types recommendation:: -- --nocapture
cargo test --package proact-types decision_quality:: -- --nocapture
cargo test --package proact-types notes_next_steps:: -- --nocapture

# Coverage check (target: 90%+)
cargo tarpaulin --package proact-types --out Html

# Full verification
cargo test --package proact-types -- --nocapture && cargo clippy --package proact-types
```

---

## Exit Criteria

### Module is COMPLETE when:

- [ ] All 38 files in File Inventory exist
- [ ] All 72 tests in Test Inventory pass
- [ ] Rust coverage >= 90%
- [ ] TypeScript types match Rust definitions
- [ ] All 9 component types implement Component trait
- [ ] ComponentVariant covers all types
- [ ] JSON serialization roundtrips work
- [ ] No clippy warnings
- [ ] No TypeScript lint errors

### Exit Signal

```
MODULE COMPLETE: proact-types
Files: 38/38
Tests: 72/72 passing
Coverage: 92%
```

---

## Implementation Phases

### Phase 1: Core Types
- [ ] Component trait
- [ ] ComponentBase implementation
- [ ] Message and MessageId types
- [ ] ComponentError enum
- [ ] Core trait tests

### Phase 2: Simple Components
- [ ] IssueRaising with output
- [ ] Recommendation with output
- [ ] NotesNextSteps with output
- [ ] Simple component tests

### Phase 3: Complex Components
- [ ] ProblemFrame with nested types (Party, Constraint, etc.)
- [ ] Objectives with nested types (FundamentalObjective, etc.)
- [ ] Alternatives with nested types (StrategyTable, etc.)
- [ ] Complex component tests

### Phase 4: Analysis Components
- [ ] Consequences with ConsequencesTable and Cell
- [ ] Tradeoffs with analysis types
- [ ] DecisionQuality with DQ elements
- [ ] Analysis component tests

### Phase 5: ComponentVariant & Integration
- [ ] ComponentVariant enum
- [ ] Factory methods
- [ ] Type dispatching
- [ ] Integration tests

### Phase 6: Frontend Types
- [ ] TypeScript interface definitions
- [ ] Type validation alignment
- [ ] Frontend type tests

---

## Notes

- This is a shared domain library (no ports/adapters)
- Components are owned and persisted by the cycle module
- All output types must be serde serializable
- Frontend types should match Rust structures exactly
- Rating type from foundation used in Consequences
- Percentage type from foundation used in DecisionQuality

---

*Generated: 2026-01-07*
*Specification: docs/modules/proact-types.md*
