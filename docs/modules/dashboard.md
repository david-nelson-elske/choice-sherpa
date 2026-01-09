# Dashboard Module Specification

## Overview

The Dashboard module provides read models and view compositions for the dashboard interface. It aggregates data from all other modules to provide Overview and Detail views for the user interface. This is a **read-only module** with no commands - only queries.

---

## Module Classification

| Attribute | Value |
|-----------|-------|
| **Type** | Full Module (Ports + Adapters, read-only) |
| **Language** | Rust |
| **Responsibility** | Read models, view aggregation, dashboard composition |
| **Domain Dependencies** | foundation, proact-types, session, cycle, conversation, analysis |
| **External Dependencies** | `async-trait`, `sqlx` |

---

## Architecture

### Hexagonal Structure (Read-Only)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         DASHBOARD MODULE                                     │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         DOMAIN LAYER                                    │ │
│  │                         (View Models)                                   │ │
│  │                                                                         │ │
│  │   ┌────────────────────────────────────────────────────────────────┐   │ │
│  │   │                  DashboardOverview                              │   │ │
│  │   │                                                                 │   │ │
│  │   │   - session_id, title                                           │   │ │
│  │   │   - decision_statement (from ProblemFrame)                      │   │ │
│  │   │   - objectives_summary                                          │   │ │
│  │   │   - alternatives_list                                           │   │ │
│  │   │   - consequences_table (compact)                                │   │ │
│  │   │   - recommendation_summary                                      │   │ │
│  │   │   - dq_score                                                    │   │ │
│  │   │   - active_cycle_id                                             │   │ │
│  │   └────────────────────────────────────────────────────────────────┘   │ │
│  │                                                                         │ │
│  │   ┌────────────────────┐  ┌────────────────────────────────────────┐   │ │
│  │   │ ComponentDetail    │  │        CycleComparison                 │   │ │
│  │   │ View               │  │        View                            │   │ │
│  │   └────────────────────┘  └────────────────────────────────────────┘   │ │
│  │                                                                         │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
│                                      ▼                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                          PORT LAYER                                     │ │
│  │                       (Read Operations Only)                            │ │
│  │                                                                         │ │
│  │   ┌─────────────────────────────────────────────────────────────────┐  │ │
│  │   │                    DashboardReader                               │  │ │
│  │   │                                                                  │  │ │
│  │   │   + get_overview(session_id, cycle_id?) -> DashboardOverview     │  │ │
│  │   │   + get_component_detail(cycle_id, type) -> ComponentDetail      │  │ │
│  │   │   + compare_cycles(cycle_ids[]) -> CycleComparison               │  │ │
│  │   └─────────────────────────────────────────────────────────────────┘  │ │
│  │                                                                         │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
│                                      ▼                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                        ADAPTER LAYER                                    │ │
│  │                                                                         │ │
│  │   ┌─────────────────────────────────────────────────────────────────┐  │ │
│  │   │              PostgresDashboardReader                             │  │ │
│  │   │   (Complex JOINs across sessions, cycles, components)            │  │ │
│  │   └─────────────────────────────────────────────────────────────────┘  │ │
│  │                                                                         │ │
│  │   ┌─────────────────────────────────────────────────────────────────┐  │ │
│  │   │                    HTTP Handlers                                 │  │ │
│  │   │   GET /sessions/:id/dashboard                                    │  │ │
│  │   │   GET /cycles/:id/components/:type/detail                        │  │ │
│  │   │   GET /sessions/:id/compare?cycles=...                           │  │ │
│  │   └─────────────────────────────────────────────────────────────────┘  │ │
│  │                                                                         │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Domain Layer (View Models)

### DashboardOverview

The main dashboard view, aggregating data from all completed components.

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::foundation::{CycleId, Percentage, SessionId};
use crate::analysis::CellColor;

/// The main dashboard overview - aggregates all component data
#[derive(Debug, Clone, Serialize)]
pub struct DashboardOverview {
    /// Session information
    pub session_id: SessionId,
    pub session_title: String,

    /// From ProblemFrame component
    pub decision_statement: Option<String>,

    /// Summary of objectives
    pub objectives_summary: Vec<ObjectiveSummary>,

    /// List of alternatives with scores
    pub alternatives_list: Vec<AlternativeSummary>,

    /// Compact consequences table
    pub consequences_table: Option<CompactConsequencesTable>,

    /// Recommendation summary
    pub recommendation: Option<RecommendationSummary>,

    /// Decision Quality score
    pub dq_score: Option<Percentage>,

    /// Active cycle information
    pub active_cycle_id: CycleId,
    pub cycle_count: usize,

    /// Timestamps
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObjectiveSummary {
    pub id: String,
    pub description: String,
    pub is_fundamental: bool,
    /// Performance measure abbreviation
    pub measure: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlternativeSummary {
    pub id: String,
    pub name: String,
    pub is_status_quo: bool,
    /// Pugh score (computed if consequences exist)
    pub pugh_score: Option<i32>,
    /// Rank among alternatives (1 = best)
    pub rank: Option<u8>,
    /// Whether this alternative is dominated
    pub is_dominated: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompactConsequencesTable {
    /// Column headers (alternative names)
    pub alternative_names: Vec<String>,
    /// Row headers (objective names)
    pub objective_names: Vec<String>,
    /// Cell data [objective_index][alternative_index]
    pub cells: Vec<Vec<CellSummary>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CellSummary {
    pub rating: i8,
    pub color: CellColor,
    /// Truncated explanation (first 50 chars)
    pub explanation_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecommendationSummary {
    /// Whether there's a standout option
    pub has_standout: bool,
    /// Name of standout option (if any)
    pub standout_name: Option<String>,
    /// First 200 chars of synthesis
    pub synthesis_preview: String,
    /// Number of caveats
    pub caveat_count: usize,
}
```

### ComponentDetailView

Detailed view for drilling into a specific component.

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::foundation::{ComponentId, ComponentStatus, ComponentType, CycleId};

/// Detailed view of a single component
#[derive(Debug, Clone, Serialize)]
pub struct ComponentDetailView {
    pub component_id: ComponentId,
    pub cycle_id: CycleId,
    pub component_type: ComponentType,
    pub status: ComponentStatus,

    /// Full structured output (type-specific JSON)
    pub structured_output: serde_json::Value,

    /// Conversation metadata
    pub conversation_message_count: usize,
    pub last_message_at: Option<DateTime<Utc>>,

    /// Actions
    pub can_branch: bool,
    pub can_revise: bool,

    /// Navigation context
    pub previous_component: Option<ComponentType>,
    pub next_component: Option<ComponentType>,
}

impl ComponentDetailView {
    /// Returns display name for the component
    pub fn display_name(&self) -> &'static str {
        self.component_type.display_name()
    }

    /// Returns true if component has been started
    pub fn is_started(&self) -> bool {
        self.status.is_started()
    }

    /// Returns true if component is complete
    pub fn is_complete(&self) -> bool {
        self.status.is_complete()
    }
}
```

### CycleComparison

View for comparing multiple cycles side-by-side.

```rust
use serde::Serialize;
use crate::foundation::{ComponentType, CycleId};
use crate::cycle::CycleProgress;

/// Comparison view for multiple cycles
#[derive(Debug, Clone, Serialize)]
pub struct CycleComparison {
    pub cycles: Vec<CycleComparisonItem>,
    pub differences: Vec<ComparisonDifference>,
    pub summary: ComparisonSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct CycleComparisonItem {
    pub cycle_id: CycleId,
    /// Where this cycle branched (if applicable)
    pub branch_point: Option<ComponentType>,
    pub progress: CycleProgress,
    /// Component outputs for comparison
    pub component_summaries: Vec<ComponentComparisonSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentComparisonSummary {
    pub component_type: ComponentType,
    /// Short summary of output (varies by component)
    pub summary: String,
    /// Key differences from other cycles
    pub differs_from_others: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComparisonDifference {
    pub component_type: ComponentType,
    pub cycle_id: CycleId,
    pub description: String,
    pub significance: DifferenceSignificance,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum DifferenceSignificance {
    Minor,
    Moderate,
    Major,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComparisonSummary {
    pub total_cycles: usize,
    pub components_with_differences: usize,
    pub most_different_cycle: Option<CycleId>,
    pub recommendation_differs: bool,
}
```

---

## Ports

### DashboardReader (Query Only)

```rust
use async_trait::async_trait;
use crate::foundation::{ComponentType, CycleId, SessionId, UserId};

/// Read-only port for dashboard queries
#[async_trait]
pub trait DashboardReader: Send + Sync {
    /// Gets the main dashboard overview for a session
    /// If cycle_id is None, uses the most recently updated active cycle
    async fn get_overview(
        &self,
        session_id: SessionId,
        cycle_id: Option<CycleId>,
        user_id: &UserId,
    ) -> Result<DashboardOverview, DashboardError>;

    /// Gets detailed view for a specific component
    async fn get_component_detail(
        &self,
        cycle_id: CycleId,
        component_type: ComponentType,
        user_id: &UserId,
    ) -> Result<ComponentDetailView, DashboardError>;

    /// Compares multiple cycles
    async fn compare_cycles(
        &self,
        cycle_ids: &[CycleId],
        user_id: &UserId,
    ) -> Result<CycleComparison, DashboardError>;
}

#[derive(Debug, thiserror::Error)]
pub enum DashboardError {
    #[error("Session not found: {0}")]
    SessionNotFound(SessionId),

    #[error("Cycle not found: {0}")]
    CycleNotFound(CycleId),

    #[error("Component not found: {0:?}")]
    ComponentNotFound(ComponentType),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

---

## Application Layer (Queries Only)

### GetDashboardOverview Query

```rust
use std::sync::Arc;
use crate::ports::DashboardReader;
use crate::foundation::{CycleId, SessionId, UserId};

#[derive(Debug, Clone)]
pub struct GetDashboardOverviewQuery {
    pub session_id: SessionId,
    pub cycle_id: Option<CycleId>,
    pub user_id: UserId,
}

pub struct GetDashboardOverviewHandler {
    reader: Arc<dyn DashboardReader>,
}

impl GetDashboardOverviewHandler {
    pub fn new(reader: Arc<dyn DashboardReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(
        &self,
        query: GetDashboardOverviewQuery,
    ) -> Result<DashboardOverview, QueryError> {
        self.reader
            .get_overview(query.session_id, query.cycle_id, &query.user_id)
            .await
            .map_err(QueryError::from)
    }
}
```

### GetComponentDetail Query

```rust
#[derive(Debug, Clone)]
pub struct GetComponentDetailQuery {
    pub cycle_id: CycleId,
    pub component_type: ComponentType,
    pub user_id: UserId,
}

pub struct GetComponentDetailHandler {
    reader: Arc<dyn DashboardReader>,
}

impl GetComponentDetailHandler {
    pub async fn handle(
        &self,
        query: GetComponentDetailQuery,
    ) -> Result<ComponentDetailView, QueryError> {
        self.reader
            .get_component_detail(query.cycle_id, query.component_type, &query.user_id)
            .await
            .map_err(QueryError::from)
    }
}
```

### CompareCycles Query

```rust
#[derive(Debug, Clone)]
pub struct CompareCyclesQuery {
    pub cycle_ids: Vec<CycleId>,
    pub user_id: UserId,
}

pub struct CompareCyclesHandler {
    reader: Arc<dyn DashboardReader>,
}

impl CompareCyclesHandler {
    pub async fn handle(
        &self,
        query: CompareCyclesQuery,
    ) -> Result<CycleComparison, QueryError> {
        if query.cycle_ids.len() < 2 {
            return Err(QueryError::InvalidInput(
                "At least 2 cycles required for comparison".to_string(),
            ));
        }

        self.reader
            .compare_cycles(&query.cycle_ids, &query.user_id)
            .await
            .map_err(QueryError::from)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Dashboard error: {0}")]
    Dashboard(#[from] DashboardError),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
```

---

## Adapters

### HTTP Endpoints

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| `GET` | `/api/sessions/:id/dashboard` | GetDashboardOverview | Main dashboard |
| `GET` | `/api/sessions/:id/dashboard?cycleId=:cid` | GetDashboardOverview | Specific cycle |
| `GET` | `/api/cycles/:id/components/:type/detail` | GetComponentDetail | Component detail |
| `GET` | `/api/sessions/:id/compare?cycles=:id1,:id2` | CompareCycles | Cycle comparison |

#### Request/Response DTOs

```rust
use serde::{Deserialize, Serialize};

// === Requests ===

#[derive(Debug, Deserialize)]
pub struct DashboardQuery {
    #[serde(rename = "cycleId")]
    pub cycle_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CompareQuery {
    /// Comma-separated cycle IDs
    pub cycles: String,
}

// === Responses ===

#[derive(Debug, Serialize)]
pub struct DashboardOverviewResponse {
    pub session: SessionInfo,
    pub decision_statement: Option<String>,
    pub objectives: Vec<ObjectiveSummaryResponse>,
    pub alternatives: Vec<AlternativeSummaryResponse>,
    pub consequences_table: Option<ConsequencesTableResponse>,
    pub recommendation: Option<RecommendationResponse>,
    pub dq_score: Option<u8>,
    pub cycle: CycleInfo,
}

#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct CycleInfo {
    pub id: String,
    pub total_cycles: usize,
    pub last_updated: String,
}

#[derive(Debug, Serialize)]
pub struct ObjectiveSummaryResponse {
    pub id: String,
    pub description: String,
    pub is_fundamental: bool,
}

#[derive(Debug, Serialize)]
pub struct AlternativeSummaryResponse {
    pub id: String,
    pub name: String,
    pub is_status_quo: bool,
    pub pugh_score: Option<i32>,
    pub rank: Option<u8>,
    pub is_dominated: bool,
}

#[derive(Debug, Serialize)]
pub struct ConsequencesTableResponse {
    pub alternatives: Vec<String>,
    pub objectives: Vec<String>,
    pub cells: Vec<Vec<CellResponse>>,
}

#[derive(Debug, Serialize)]
pub struct CellResponse {
    pub rating: i8,
    pub color: String,
}

#[derive(Debug, Serialize)]
pub struct RecommendationResponse {
    pub has_standout: bool,
    pub standout_name: Option<String>,
    pub synthesis_preview: String,
    pub caveat_count: usize,
}
```

### PostgresDashboardReader

Complex query adapter that joins across multiple tables.

```rust
use sqlx::PgPool;
use async_trait::async_trait;
use crate::ports::{DashboardReader, DashboardError};
use crate::domain::{DashboardOverview, ComponentDetailView, CycleComparison};
use crate::foundation::{ComponentType, CycleId, SessionId, UserId};
use crate::analysis::{PughAnalyzer, DQCalculator, CellColor};

pub struct PostgresDashboardReader {
    pool: PgPool,
}

impl PostgresDashboardReader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// SECURITY: Authorizes user access to session with audit logging
    async fn authorize_session(
        &self,
        session_id: SessionId,
        user_id: &UserId,
    ) -> Result<(), DashboardError> {
        let row = sqlx::query!(
            r#"SELECT user_id FROM sessions WHERE id = $1"#,
            session_id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) if r.user_id == user_id.as_str() => {
                tracing::debug!(
                    user_id = %user_id,
                    session_id = %session_id,
                    "Dashboard authorization successful"
                );
                Ok(())
            }
            Some(r) => {
                // SECURITY: Log authorization failures with owner context for audit
                tracing::warn!(
                    user_id = %user_id,
                    session_id = %session_id,
                    owner_id = %r.user_id,
                    "Dashboard authorization failed - user does not own session"
                );
                Err(DashboardError::Unauthorized)
            }
            None => {
                tracing::warn!(
                    user_id = %user_id,
                    session_id = %session_id,
                    "Dashboard authorization failed - session not found"
                );
                Err(DashboardError::SessionNotFound(session_id))
            }
        }
    }
}

#[async_trait]
impl DashboardReader for PostgresDashboardReader {
    async fn get_overview(
        &self,
        session_id: SessionId,
        cycle_id: Option<CycleId>,
        user_id: &UserId,
    ) -> Result<DashboardOverview, DashboardError> {
        // 1. Authorize
        self.authorize_session(session_id, user_id).await?;

        // 2. Get session info
        let session = sqlx::query!(
            r#"SELECT title FROM sessions WHERE id = $1"#,
            session_id.as_uuid()
        )
        .fetch_one(&self.pool)
        .await?;

        // 3. Get cycle (specified or most recent active)
        let cycle_row = match cycle_id {
            Some(id) => {
                sqlx::query!(
                    r#"SELECT id, updated_at FROM cycles WHERE id = $1"#,
                    id.as_uuid()
                )
                .fetch_optional(&self.pool)
                .await?
                .ok_or(DashboardError::CycleNotFound(id))?
            }
            None => {
                sqlx::query!(
                    r#"
                    SELECT id, updated_at FROM cycles
                    WHERE session_id = $1 AND status = 'active'
                    ORDER BY updated_at DESC
                    LIMIT 1
                    "#,
                    session_id.as_uuid()
                )
                .fetch_optional(&self.pool)
                .await?
                .ok_or(DashboardError::SessionNotFound(session_id))?
            }
        };

        let active_cycle_id = CycleId::from_uuid(cycle_row.id);

        // 4. Get component data
        let components = sqlx::query!(
            r#"
            SELECT component_type, structured_data
            FROM components
            WHERE cycle_id = $1
            "#,
            cycle_row.id
        )
        .fetch_all(&self.pool)
        .await?;

        // 5. Build overview from components
        let mut decision_statement = None;
        let mut objectives_summary = Vec::new();
        let mut alternatives_list = Vec::new();
        let mut consequences_table = None;
        let mut recommendation = None;
        let mut dq_score = None;

        for comp in components {
            match comp.component_type.as_str() {
                "problem_frame" => {
                    if let Some(stmt) = comp.structured_data.get("decision_statement") {
                        decision_statement = stmt.as_str().map(String::from);
                    }
                }
                "objectives" => {
                    // Parse objectives from structured_data
                    objectives_summary = self.parse_objectives(&comp.structured_data);
                }
                "alternatives" => {
                    alternatives_list = self.parse_alternatives(&comp.structured_data);
                }
                "consequences" => {
                    consequences_table = self.build_consequences_table(&comp.structured_data);
                    // Calculate Pugh scores
                    if let Some(ref table) = consequences_table {
                        let scores = PughAnalyzer::compute_scores(&/* ... */);
                        // Update alternatives_list with scores
                    }
                }
                "recommendation" => {
                    recommendation = self.parse_recommendation(&comp.structured_data);
                }
                "decision_quality" => {
                    if let Some(elements) = comp.structured_data.get("elements") {
                        // Calculate overall DQ
                        dq_score = Some(/* DQCalculator::compute_overall(...) */);
                    }
                }
                _ => {}
            }
        }

        // 6. Get cycle count
        let cycle_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM cycles WHERE session_id = $1"#,
            session_id.as_uuid()
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0) as usize;

        Ok(DashboardOverview {
            session_id,
            session_title: session.title,
            decision_statement,
            objectives_summary,
            alternatives_list,
            consequences_table,
            recommendation,
            dq_score,
            active_cycle_id,
            cycle_count,
            last_updated: cycle_row.updated_at,
        })
    }

    async fn get_component_detail(
        &self,
        cycle_id: CycleId,
        component_type: ComponentType,
        user_id: &UserId,
    ) -> Result<ComponentDetailView, DashboardError> {
        // Implementation: fetch component, conversation count, navigation context
        todo!()
    }

    async fn compare_cycles(
        &self,
        cycle_ids: &[CycleId],
        user_id: &UserId,
    ) -> Result<CycleComparison, DashboardError> {
        // Implementation: fetch all cycles, compare components, find differences
        todo!()
    }
}
```

---

## File Structure

```
backend/src/domain/dashboard/
├── mod.rs                      # Module exports
├── overview.rs                 # DashboardOverview view model
├── component_detail.rs         # ComponentDetailView
└── cycle_comparison.rs         # CycleComparison view model

backend/src/ports/
└── dashboard_reader.rs         # DashboardReader trait

backend/src/application/queries/
├── get_dashboard_overview.rs
├── get_dashboard_overview_test.rs
├── get_component_detail.rs
└── compare_cycles.rs

backend/src/adapters/
├── http/dashboard/
│   ├── handlers.rs
│   ├── handlers_test.rs
│   ├── dto.rs
│   └── routes.rs
└── postgres/
    ├── dashboard_reader.rs
    └── dashboard_reader_test.rs

frontend/src/modules/dashboard/
├── domain/
│   ├── overview.ts
│   ├── component-detail.ts
│   └── cycle-comparison.ts
├── api/
│   ├── dashboard-api.ts
│   ├── use-dashboard.ts
│   └── use-component-detail.ts
├── components/
│   ├── DashboardLayout.tsx
│   ├── DashboardLayout.test.tsx
│   ├── OverviewPanel.tsx
│   ├── DecisionStatement.tsx
│   ├── ObjectivesList.tsx
│   ├── AlternativesPills.tsx
│   ├── ConsequencesMatrix.tsx
│   ├── RecommendationCard.tsx
│   ├── DQScoreBadge.tsx
│   ├── CycleTreeSidebar.tsx
│   └── ComponentDetailDrawer.tsx
├── pages/
│   ├── DashboardPage.tsx
│   └── DashboardPage.test.tsx
└── index.ts
```

---

## Data Flow

```
┌───────────────────────────────────────────────────────────────────┐
│                         Frontend Request                           │
│                    GET /sessions/:id/dashboard                      │
└───────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌───────────────────────────────────────────────────────────────────┐
│                          HTTP Handler                              │
│                   Parse request, extract user_id                   │
└───────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌───────────────────────────────────────────────────────────────────┐
│                    GetDashboardOverviewHandler                     │
│                       (Application Layer)                          │
└───────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌───────────────────────────────────────────────────────────────────┐
│                      DashboardReader Port                          │
│                   (Called via trait object)                        │
└───────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌───────────────────────────────────────────────────────────────────┐
│                   PostgresDashboardReader                          │
│                                                                    │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │  1. Authorize user via sessions.user_id                     │  │
│  │  2. Fetch session title                                     │  │
│  │  3. Fetch cycle (specified or most recent active)           │  │
│  │  4. Fetch all components for cycle                          │  │
│  │  5. Parse each component's structured_data                  │  │
│  │  6. Call Analysis services for Pugh scores, DQ              │  │
│  │  7. Assemble DashboardOverview                              │  │
│  └─────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌───────────────────────────────────────────────────────────────────┐
│                     DashboardOverview                              │
│              (Returned to HTTP handler for JSON)                   │
└───────────────────────────────────────────────────────────────────┘
```

---

## Invariants

| Invariant | Enforcement |
|-----------|-------------|
| Read-only operations | No save/update methods in port |
| User authorization | Checked in every query |
| Valid component types | Parsed from enum |
| Cycle belongs to session | JOIN constraint in queries |
| Analysis calculations correct | Delegate to Analysis module |

---

## Test Categories

### Unit Tests

| Category | Example Tests |
|----------|---------------|
| View models | `dashboard_overview_serializes_correctly` |
| View models | `component_detail_has_navigation` |
| Comparison | `comparison_finds_differences` |

### Integration Tests

| Category | Example Tests |
|----------|---------------|
| Reader | `get_overview_returns_complete_data` |
| Reader | `get_overview_authorizes_user` |
| Reader | `compare_cycles_identifies_differences` |
| HTTP | `dashboard_endpoint_returns_json` |

---

*Module Version: 1.0.0*
*Based on: SYSTEM-ARCHITECTURE.md v1.1.0*
*Language: Rust*
