# Dashboard Module Checklist

**Module:** Dashboard
**Language:** Rust
**Dependencies:** foundation, proact-types, session, cycle, conversation, analysis
**Phase:** 4 (depends on all other modules)

---

## Overview

The Dashboard module provides read models and view compositions for the dashboard interface. It aggregates data from all other modules to provide Overview and Detail views for the user interface. This is a **read-only module** with no commands - only queries.

---

## Module Classification

| Attribute | Value |
|-----------|-------|
| **Type** | Full Module (Ports + Adapters, read-only) |
| **Language** | Rust |
| **Responsibility** | Read models, view aggregation, dashboard composition |
| **External Dependencies** | `async-trait`, `sqlx` |

---

## File Inventory

### Domain Layer - View Models (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/dashboard/mod.rs` | Module exports | ⬜ |
| `backend/src/domain/dashboard/overview.rs` | DashboardOverview view model | ⬜ |
| `backend/src/domain/dashboard/objective_summary.rs` | ObjectiveSummary struct | ⬜ |
| `backend/src/domain/dashboard/alternative_summary.rs` | AlternativeSummary struct | ⬜ |
| `backend/src/domain/dashboard/compact_consequences.rs` | CompactConsequencesTable, CellSummary | ⬜ |
| `backend/src/domain/dashboard/recommendation_summary.rs` | RecommendationSummary struct | ⬜ |
| `backend/src/domain/dashboard/component_detail.rs` | ComponentDetailView | ⬜ |
| `backend/src/domain/dashboard/cycle_comparison.rs` | CycleComparison, ComparisonDifference | ⬜ |
| `backend/src/domain/dashboard/comparison_summary.rs` | ComparisonSummary, DifferenceSignificance | ⬜ |

### Domain Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/dashboard/overview_test.rs` | DashboardOverview tests | ⬜ |
| `backend/src/domain/dashboard/component_detail_test.rs` | ComponentDetailView tests | ⬜ |
| `backend/src/domain/dashboard/cycle_comparison_test.rs` | CycleComparison tests | ⬜ |

### Ports (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/ports/dashboard_reader.rs` | DashboardReader trait | ⬜ |
| `backend/src/ports/dashboard_error.rs` | DashboardError enum | ⬜ |

### Application Layer - Queries (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/queries/get_dashboard_overview.rs` | GetDashboardOverview query + handler | ⬜ |
| `backend/src/application/queries/get_component_detail.rs` | GetComponentDetail query + handler | ⬜ |
| `backend/src/application/queries/compare_cycles.rs` | CompareCycles query + handler | ⬜ |
| `backend/src/application/queries/query_error.rs` | QueryError enum | ⬜ |

### Application Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/queries/get_dashboard_overview_test.rs` | GetDashboardOverview tests | ⬜ |
| `backend/src/application/queries/get_component_detail_test.rs` | GetComponentDetail tests | ⬜ |
| `backend/src/application/queries/compare_cycles_test.rs` | CompareCycles tests | ⬜ |

### Adapters - HTTP (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/http/dashboard/mod.rs` | HTTP module exports | ⬜ |
| `backend/src/adapters/http/dashboard/handlers.rs` | HTTP handlers | ⬜ |
| `backend/src/adapters/http/dashboard/dto.rs` | Request/Response DTOs | ⬜ |
| `backend/src/adapters/http/dashboard/routes.rs` | Route definitions | ⬜ |

### Adapters - HTTP Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/http/dashboard/handlers_test.rs` | Handler tests | ⬜ |

### Adapters - Postgres (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/postgres/dashboard_reader.rs` | PostgresDashboardReader impl | ⬜ |

### Adapters - Postgres Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/postgres/dashboard_reader_test.rs` | PostgresDashboardReader tests | ⬜ |

### Frontend Types (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/dashboard/domain/overview.ts` | DashboardOverview type | ⬜ |
| `frontend/src/modules/dashboard/domain/component-detail.ts` | ComponentDetailView type | ⬜ |
| `frontend/src/modules/dashboard/domain/cycle-comparison.ts` | CycleComparison type | ⬜ |
| `frontend/src/modules/dashboard/index.ts` | Public exports | ⬜ |

### Frontend API (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/dashboard/api/dashboard-api.ts` | API client | ⬜ |
| `frontend/src/modules/dashboard/api/use-dashboard.ts` | Dashboard hook | ⬜ |
| `frontend/src/modules/dashboard/api/use-component-detail.ts` | Component detail hook | ⬜ |

### Frontend Components (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/dashboard/components/DashboardLayout.tsx` | Main layout | ⬜ |
| `frontend/src/modules/dashboard/components/OverviewPanel.tsx` | Overview panel | ⬜ |
| `frontend/src/modules/dashboard/components/DecisionStatement.tsx` | Decision statement display | ⬜ |
| `frontend/src/modules/dashboard/components/ObjectivesList.tsx` | Objectives list | ⬜ |
| `frontend/src/modules/dashboard/components/AlternativesPills.tsx` | Alternatives as pills | ⬜ |
| `frontend/src/modules/dashboard/components/ConsequencesMatrix.tsx` | Consequences table | ⬜ |
| `frontend/src/modules/dashboard/components/RecommendationCard.tsx` | Recommendation card | ⬜ |
| `frontend/src/modules/dashboard/components/DQScoreBadge.tsx` | DQ score badge | ⬜ |
| `frontend/src/modules/dashboard/components/CycleTreeSidebar.tsx` | Cycle tree sidebar | ⬜ |
| `frontend/src/modules/dashboard/components/ComponentDetailDrawer.tsx` | Detail drawer | ⬜ |

### Frontend Pages (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/dashboard/pages/DashboardPage.tsx` | Main dashboard page | ⬜ |

### Frontend Tests (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/dashboard/components/DashboardLayout.test.tsx` | Layout tests | ⬜ |
| `frontend/src/modules/dashboard/pages/DashboardPage.test.tsx` | Page tests | ⬜ |

---

## Test Inventory

### DashboardOverview Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_overview_serializes_all_fields` | JSON serialization works | ⬜ |
| `test_overview_handles_empty_components` | Works with no completed components | ⬜ |
| `test_overview_includes_session_info` | Session id and title present | ⬜ |
| `test_overview_includes_cycle_info` | Active cycle and count present | ⬜ |
| `test_objective_summary_serializes` | ObjectiveSummary serializes | ⬜ |
| `test_alternative_summary_includes_rank` | Rank field present | ⬜ |
| `test_alternative_summary_includes_dominated` | Dominated flag present | ⬜ |
| `test_cell_summary_has_color` | CellSummary includes color | ⬜ |
| `test_recommendation_summary_has_preview` | Synthesis preview truncated | ⬜ |

### ComponentDetailView Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_component_detail_has_type` | Component type is set | ⬜ |
| `test_component_detail_has_status` | Status field present | ⬜ |
| `test_component_detail_has_structured_output` | Output is JSON value | ⬜ |
| `test_component_detail_has_navigation` | Previous/next components set | ⬜ |
| `test_component_detail_display_name` | Display name method works | ⬜ |
| `test_component_detail_is_started` | Is started predicate | ⬜ |
| `test_component_detail_is_complete` | Is complete predicate | ⬜ |
| `test_component_detail_can_branch` | Can branch flag | ⬜ |
| `test_component_detail_can_revise` | Can revise flag | ⬜ |

### CycleComparison Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_comparison_includes_all_cycles` | All requested cycles present | ⬜ |
| `test_comparison_identifies_differences` | Differences list populated | ⬜ |
| `test_comparison_summary_counts` | Summary counts correct | ⬜ |
| `test_comparison_item_has_progress` | CycleComparisonItem has progress | ⬜ |
| `test_comparison_item_has_branch_point` | Branch point set if applicable | ⬜ |
| `test_difference_has_significance` | Significance enum present | ⬜ |
| `test_difference_significance_values` | Minor/Moderate/Major values | ⬜ |

### DashboardReader Port Tests (with mock)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_reader_get_overview_returns_result` | Returns DashboardOverview | ⬜ |
| `test_reader_get_component_detail_returns_result` | Returns ComponentDetailView | ⬜ |
| `test_reader_compare_cycles_returns_result` | Returns CycleComparison | ⬜ |

### GetDashboardOverview Query Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_get_overview_calls_reader` | Calls reader with params | ⬜ |
| `test_get_overview_uses_specified_cycle` | Uses cycle_id if provided | ⬜ |
| `test_get_overview_defaults_to_active_cycle` | Uses active if cycle_id None | ⬜ |
| `test_get_overview_passes_user_id` | User ID passed for auth | ⬜ |
| `test_get_overview_propagates_errors` | Errors bubble up | ⬜ |

### GetComponentDetail Query Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_get_detail_calls_reader` | Calls reader with params | ⬜ |
| `test_get_detail_passes_component_type` | Component type passed | ⬜ |
| `test_get_detail_passes_user_id` | User ID passed for auth | ⬜ |
| `test_get_detail_propagates_errors` | Errors bubble up | ⬜ |

### CompareCycles Query Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_compare_requires_two_cycles` | Error if < 2 cycles | ⬜ |
| `test_compare_calls_reader` | Calls reader with IDs | ⬜ |
| `test_compare_passes_user_id` | User ID passed for auth | ⬜ |
| `test_compare_propagates_errors` | Errors bubble up | ⬜ |

### PostgresDashboardReader Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_postgres_get_overview_returns_data` | Integration: returns overview | ⬜ |
| `test_postgres_get_overview_authorizes_user` | Rejects unauthorized user | ⬜ |
| `test_postgres_get_overview_session_not_found` | Returns error for missing | ⬜ |
| `test_postgres_get_overview_fetches_components` | Fetches all components | ⬜ |
| `test_postgres_get_overview_computes_pugh_scores` | Pugh scores calculated | ⬜ |
| `test_postgres_get_overview_computes_dq_score` | DQ score calculated | ⬜ |
| `test_postgres_get_detail_returns_data` | Integration: returns detail | ⬜ |
| `test_postgres_get_detail_authorizes_user` | Rejects unauthorized | ⬜ |
| `test_postgres_get_detail_includes_conversation_count` | Message count included | ⬜ |
| `test_postgres_compare_returns_comparison` | Integration: returns comparison | ⬜ |
| `test_postgres_compare_finds_differences` | Identifies differences | ⬜ |

### HTTP Handler Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_dashboard_endpoint_returns_json` | Content-Type is JSON | ⬜ |
| `test_dashboard_endpoint_parses_cycle_id` | Query param parsed | ⬜ |
| `test_dashboard_endpoint_requires_auth` | 401 without auth | ⬜ |
| `test_dashboard_endpoint_404_for_missing` | 404 for missing session | ⬜ |
| `test_detail_endpoint_returns_json` | Content-Type is JSON | ⬜ |
| `test_detail_endpoint_parses_component_type` | Path param parsed | ⬜ |
| `test_compare_endpoint_parses_cycle_ids` | Query param parsed | ⬜ |
| `test_compare_endpoint_requires_two_ids` | 400 for single ID | ⬜ |

---

## Error Codes

| Error Code | Condition |
|------------|-----------|
| `SESSION_NOT_FOUND` | Session does not exist |
| `CYCLE_NOT_FOUND` | Cycle does not exist |
| `COMPONENT_NOT_FOUND` | Component type not found in cycle |
| `UNAUTHORIZED` | User doesn't own the session |
| `INVALID_INPUT` | Invalid query parameters |
| `DATABASE_ERROR` | Database operation failed |

---

## Business Rules

| Rule | Implementation | Test | Status |
|------|----------------|------|--------|
| User must own session | `authorize_session()` check | `test_postgres_get_overview_authorizes_user` | ⬜ |
| Default to active cycle | Query uses most recent active | `test_get_overview_defaults_to_active_cycle` | ⬜ |
| Pugh scores from Analysis module | Call `PughAnalyzer::compute_scores()` | `test_postgres_get_overview_computes_pugh_scores` | ⬜ |
| DQ score from Analysis module | Call `DQCalculator::compute_overall()` | `test_postgres_get_overview_computes_dq_score` | ⬜ |
| Comparison requires 2+ cycles | Validation in handler | `test_compare_requires_two_cycles` | ⬜ |
| Read-only module | No save/update methods | Port interface design | ⬜ |

---

## Verification Commands

```bash
# Run all dashboard tests
cargo test --package dashboard -- --nocapture

# Run specific test category
cargo test --package dashboard overview:: -- --nocapture
cargo test --package dashboard component_detail:: -- --nocapture
cargo test --package dashboard comparison:: -- --nocapture
cargo test --package dashboard reader:: -- --nocapture
cargo test --package dashboard handlers:: -- --nocapture

# Coverage check (target: 85%+)
cargo tarpaulin --package dashboard --out Html

# Full verification
cargo test --package dashboard -- --nocapture && cargo clippy --package dashboard

# Frontend tests
cd frontend && npm test -- --testPathPattern="modules/dashboard"
```

---

## Exit Criteria

### Module is COMPLETE when:

- [ ] All 53 files in File Inventory exist
- [ ] All 62 tests in Test Inventory pass
- [ ] Rust coverage >= 85%
- [ ] PostgresDashboardReader uses Analysis module
- [ ] All endpoints require authentication
- [ ] Cycle comparison works with 2+ cycles
- [ ] Frontend components render all view data
- [ ] No clippy warnings
- [ ] No TypeScript lint errors

### Exit Signal

```
MODULE COMPLETE: dashboard
Files: 53/53
Tests: 62/62 passing
Coverage: 87%
```

---

## Implementation Phases

### Phase 1: View Models - Overview
- [ ] DashboardOverview struct
- [ ] ObjectiveSummary struct
- [ ] AlternativeSummary struct
- [ ] CompactConsequencesTable, CellSummary
- [ ] RecommendationSummary struct
- [ ] Overview tests

### Phase 2: View Models - Detail & Comparison
- [ ] ComponentDetailView struct
- [ ] CycleComparison struct
- [ ] CycleComparisonItem struct
- [ ] ComparisonDifference, DifferenceSignificance
- [ ] ComparisonSummary struct
- [ ] Detail and comparison tests

### Phase 3: Ports
- [ ] DashboardReader trait
- [ ] DashboardError enum
- [ ] Port trait tests (with mock)

### Phase 4: Query Handlers
- [ ] GetDashboardOverviewQuery + Handler
- [ ] GetComponentDetailQuery + Handler
- [ ] CompareCyclesQuery + Handler
- [ ] QueryError enum
- [ ] Query handler tests

### Phase 5: PostgresDashboardReader - Overview
- [ ] authorize_session() method
- [ ] get_overview() implementation
- [ ] Component parsing helpers
- [ ] Analysis module integration
- [ ] Overview integration tests

### Phase 6: PostgresDashboardReader - Detail & Compare
- [ ] get_component_detail() implementation
- [ ] compare_cycles() implementation
- [ ] Difference detection logic
- [ ] Integration tests

### Phase 7: HTTP Layer
- [ ] Request/Response DTOs
- [ ] HTTP handlers
- [ ] Route definitions
- [ ] Handler tests

### Phase 8: Frontend Types
- [ ] TypeScript view model types
- [ ] API client
- [ ] Hooks (useDashboard, useComponentDetail)

### Phase 9: Frontend Components
- [ ] DashboardLayout
- [ ] OverviewPanel
- [ ] DecisionStatement
- [ ] ObjectivesList
- [ ] AlternativesPills
- [ ] ConsequencesMatrix
- [ ] RecommendationCard
- [ ] DQScoreBadge

### Phase 10: Frontend - Advanced Components
- [ ] CycleTreeSidebar
- [ ] ComponentDetailDrawer
- [ ] DashboardPage
- [ ] Component tests
- [ ] Page tests

---

## Notes

- This is a read-only module - no commands, only queries
- Depends on ALL other modules (built last)
- Uses Analysis module for Pugh scores and DQ calculations
- PostgresDashboardReader has complex JOINs across tables
- Authorization checked on every query
- Frontend is the main consumer of dashboard data
- View models are optimized for UI rendering (summaries, previews)

---

*Generated: 2026-01-07*
*Specification: docs/modules/dashboard.md*
