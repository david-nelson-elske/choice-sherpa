//! PostgreSQL implementation of CycleRepository.
//!
//! Persists Cycle aggregates to PostgreSQL with components stored as JSONB.

use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::cycle::Cycle;
use crate::domain::foundation::{
    ComponentId, ComponentStatus, ComponentType, CycleId, CycleStatus, DomainError, ErrorCode,
    SessionId, Timestamp,
};
use crate::domain::proact::ComponentVariant;
use crate::ports::CycleRepository;

/// PostgreSQL implementation of CycleRepository.
#[derive(Clone)]
pub struct PostgresCycleRepository {
    pool: PgPool,
}

impl PostgresCycleRepository {
    /// Creates a new PostgresCycleRepository.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CycleRepository for PostgresCycleRepository {
    async fn save(&self, cycle: &Cycle) -> Result<(), DomainError> {
        let mut tx = self.pool.begin().await.map_err(|e| {
            DomainError::new(ErrorCode::DatabaseError, format!("Failed to begin transaction: {}", e))
        })?;

        // Insert cycle
        sqlx::query(
            r#"
            INSERT INTO cycles (
                id, session_id, parent_cycle_id, branch_point, status,
                current_step, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(cycle.id().as_uuid())
        .bind(cycle.session_id().as_uuid())
        .bind(cycle.parent_cycle_id().map(|id| *id.as_uuid()))
        .bind(cycle.branch_point().map(component_type_to_str))
        .bind(cycle_status_to_str(cycle.status()))
        .bind(component_type_to_str(cycle.current_step()))
        .bind(cycle.created_at().as_datetime())
        .bind(cycle.updated_at().as_datetime())
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::new(ErrorCode::DatabaseError, format!("Failed to insert cycle: {}", e)))?;

        // Insert all components
        for component_type in ComponentType::all() {
            if let Some(component) = cycle.component(*component_type) {
                save_component(&mut tx, cycle.id(), component).await?;
            }
        }

        tx.commit().await.map_err(|e| {
            DomainError::new(ErrorCode::DatabaseError, format!("Failed to commit transaction: {}", e))
        })?;

        Ok(())
    }

    async fn update(&self, cycle: &Cycle) -> Result<(), DomainError> {
        let mut tx = self.pool.begin().await.map_err(|e| {
            DomainError::new(ErrorCode::DatabaseError, format!("Failed to begin transaction: {}", e))
        })?;

        // Update cycle
        let result = sqlx::query(
            r#"
            UPDATE cycles SET
                status = $2,
                current_step = $3,
                updated_at = $4
            WHERE id = $1
            "#,
        )
        .bind(cycle.id().as_uuid())
        .bind(cycle_status_to_str(cycle.status()))
        .bind(component_type_to_str(cycle.current_step()))
        .bind(cycle.updated_at().as_datetime())
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::new(ErrorCode::DatabaseError, format!("Failed to update cycle: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::CycleNotFound,
                format!("Cycle not found: {}", cycle.id()),
            ));
        }

        // Update all components
        for component_type in ComponentType::all() {
            if let Some(component) = cycle.component(*component_type) {
                update_component(&mut tx, cycle.id(), component).await?;
            }
        }

        tx.commit().await.map_err(|e| {
            DomainError::new(ErrorCode::DatabaseError, format!("Failed to commit transaction: {}", e))
        })?;

        Ok(())
    }

    async fn find_by_id(&self, id: &CycleId) -> Result<Option<Cycle>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, session_id, parent_cycle_id, branch_point, status,
                   current_step, created_at, updated_at
            FROM cycles WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::DatabaseError, format!("Failed to fetch cycle: {}", e)))?;

        match row {
            Some(row) => {
                let components = load_components(&self.pool, id).await?;
                let cycle = row_to_cycle(row, components)?;
                Ok(Some(cycle))
            }
            None => Ok(None),
        }
    }

    async fn exists(&self, id: &CycleId) -> Result<bool, DomainError> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM cycles WHERE id = $1")
            .bind(id.as_uuid())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::new(ErrorCode::DatabaseError, format!("Failed to check cycle existence: {}", e)))?;

        Ok(result.0 > 0)
    }

    async fn find_by_session_id(&self, session_id: &SessionId) -> Result<Vec<Cycle>, DomainError> {
        let rows = sqlx::query(
            r#"
            SELECT id, session_id, parent_cycle_id, branch_point, status,
                   current_step, created_at, updated_at
            FROM cycles
            WHERE session_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(session_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::DatabaseError, format!("Failed to fetch cycles: {}", e)))?;

        let mut cycles = Vec::with_capacity(rows.len());
        for row in rows {
            let id: Uuid = row.get("id");
            let cycle_id = CycleId::from_uuid(id);
            let components = load_components(&self.pool, &cycle_id).await?;
            let cycle = row_to_cycle(row, components)?;
            cycles.push(cycle);
        }

        Ok(cycles)
    }

    async fn find_primary_by_session_id(
        &self,
        session_id: &SessionId,
    ) -> Result<Option<Cycle>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, session_id, parent_cycle_id, branch_point, status,
                   current_step, created_at, updated_at
            FROM cycles
            WHERE session_id = $1 AND parent_cycle_id IS NULL
            ORDER BY created_at ASC
            LIMIT 1
            "#,
        )
        .bind(session_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::DatabaseError, format!("Failed to fetch primary cycle: {}", e)))?;

        match row {
            Some(row) => {
                let id: Uuid = row.get("id");
                let cycle_id = CycleId::from_uuid(id);
                let components = load_components(&self.pool, &cycle_id).await?;
                let cycle = row_to_cycle(row, components)?;
                Ok(Some(cycle))
            }
            None => Ok(None),
        }
    }

    async fn find_branches(&self, parent_id: &CycleId) -> Result<Vec<Cycle>, DomainError> {
        let rows = sqlx::query(
            r#"
            SELECT id, session_id, parent_cycle_id, branch_point, status,
                   current_step, created_at, updated_at
            FROM cycles
            WHERE parent_cycle_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(parent_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::DatabaseError, format!("Failed to fetch branches: {}", e)))?;

        let mut cycles = Vec::with_capacity(rows.len());
        for row in rows {
            let id: Uuid = row.get("id");
            let cycle_id = CycleId::from_uuid(id);
            let components = load_components(&self.pool, &cycle_id).await?;
            let cycle = row_to_cycle(row, components)?;
            cycles.push(cycle);
        }

        Ok(cycles)
    }

    async fn count_by_session_id(&self, session_id: &SessionId) -> Result<u32, DomainError> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM cycles WHERE session_id = $1")
            .bind(session_id.as_uuid())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::new(ErrorCode::DatabaseError, format!("Failed to count cycles: {}", e)))?;

        Ok(result.0 as u32)
    }

    async fn delete(&self, id: &CycleId) -> Result<(), DomainError> {
        let mut tx = self.pool.begin().await.map_err(|e| {
            DomainError::new(ErrorCode::DatabaseError, format!("Failed to begin transaction: {}", e))
        })?;

        // Delete components first (foreign key constraint)
        sqlx::query("DELETE FROM components WHERE cycle_id = $1")
            .bind(id.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::new(ErrorCode::DatabaseError, format!("Failed to delete components: {}", e)))?;

        // Delete cycle
        let result = sqlx::query("DELETE FROM cycles WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::new(ErrorCode::DatabaseError, format!("Failed to delete cycle: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::CycleNotFound,
                format!("Cycle not found: {}", id),
            ));
        }

        tx.commit().await.map_err(|e| {
            DomainError::new(ErrorCode::DatabaseError, format!("Failed to commit transaction: {}", e))
        })?;

        Ok(())
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Helper Functions
// ════════════════════════════════════════════════════════════════════════════════

async fn save_component(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    cycle_id: CycleId,
    component: &ComponentVariant,
) -> Result<(), DomainError> {
    sqlx::query(
        r#"
        INSERT INTO components (
            id, cycle_id, component_type, status, output, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(component.id().as_uuid())
    .bind(cycle_id.as_uuid())
    .bind(component_type_to_str(component.component_type()))
    .bind(component_status_to_str(component.status()))
    .bind(component.output_as_value())
    .bind(component.created_at().as_datetime())
    .bind(component.updated_at().as_datetime())
    .execute(&mut **tx)
    .await
    .map_err(|e| DomainError::new(ErrorCode::DatabaseError, format!("Failed to insert component: {}", e)))?;

    Ok(())
}

async fn update_component(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    cycle_id: CycleId,
    component: &ComponentVariant,
) -> Result<(), DomainError> {
    sqlx::query(
        r#"
        UPDATE components SET
            status = $3,
            output = $4,
            updated_at = $5
        WHERE cycle_id = $1 AND component_type = $2
        "#,
    )
    .bind(cycle_id.as_uuid())
    .bind(component_type_to_str(component.component_type()))
    .bind(component_status_to_str(component.status()))
    .bind(component.output_as_value())
    .bind(component.updated_at().as_datetime())
    .execute(&mut **tx)
    .await
    .map_err(|e| DomainError::new(ErrorCode::DatabaseError, format!("Failed to update component: {}", e)))?;

    Ok(())
}

async fn load_components(
    pool: &PgPool,
    cycle_id: &CycleId,
) -> Result<HashMap<ComponentType, ComponentVariant>, DomainError> {
    let rows = sqlx::query(
        r#"
        SELECT id, component_type, status, output, created_at, updated_at
        FROM components
        WHERE cycle_id = $1
        "#,
    )
    .bind(cycle_id.as_uuid())
    .fetch_all(pool)
    .await
    .map_err(|e| DomainError::new(ErrorCode::DatabaseError, format!("Failed to load components: {}", e)))?;

    let mut components = HashMap::new();
    for row in rows {
        let component_type_str: String = row.get("component_type");
        let component_type = str_to_component_type(&component_type_str)?;
        let component = row_to_component(row, component_type)?;
        components.insert(component_type, component);
    }

    Ok(components)
}

fn row_to_cycle(
    row: sqlx::postgres::PgRow,
    components: HashMap<ComponentType, ComponentVariant>,
) -> Result<Cycle, DomainError> {
    let id: Uuid = row.get("id");
    let session_id: Uuid = row.get("session_id");
    let parent_cycle_id: Option<Uuid> = row.get("parent_cycle_id");
    let branch_point: Option<String> = row.get("branch_point");
    let status: String = row.get("status");
    let current_step: String = row.get("current_step");
    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

    // Reconstruct the cycle using the internal constructor
    Cycle::reconstitute(
        CycleId::from_uuid(id),
        SessionId::from_uuid(session_id),
        parent_cycle_id.map(CycleId::from_uuid),
        branch_point.map(|s| str_to_component_type(&s)).transpose()?,
        str_to_cycle_status(&status)?,
        str_to_component_type(&current_step)?,
        components,
        Timestamp::from_datetime(created_at),
        Timestamp::from_datetime(updated_at),
    )
}

fn row_to_component(
    row: sqlx::postgres::PgRow,
    component_type: ComponentType,
) -> Result<ComponentVariant, DomainError> {
    let id: Uuid = row.get("id");
    let status: String = row.get("status");
    let output: serde_json::Value = row.get("output");
    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

    // Reconstitute the component
    ComponentVariant::reconstitute(
        ComponentId::from_uuid(id),
        component_type,
        str_to_component_status(&status)?,
        output,
        Timestamp::from_datetime(created_at),
        Timestamp::from_datetime(updated_at),
    )
}

// ════════════════════════════════════════════════════════════════════════════════
// Type Conversions
// ════════════════════════════════════════════════════════════════════════════════

fn component_type_to_str(ct: ComponentType) -> &'static str {
    match ct {
        ComponentType::IssueRaising => "issue_raising",
        ComponentType::ProblemFrame => "problem_frame",
        ComponentType::Objectives => "objectives",
        ComponentType::Alternatives => "alternatives",
        ComponentType::Consequences => "consequences",
        ComponentType::Tradeoffs => "tradeoffs",
        ComponentType::Recommendation => "recommendation",
        ComponentType::DecisionQuality => "decision_quality",
        ComponentType::NotesNextSteps => "notes_next_steps",
    }
}

fn str_to_component_type(s: &str) -> Result<ComponentType, DomainError> {
    match s {
        "issue_raising" => Ok(ComponentType::IssueRaising),
        "problem_frame" => Ok(ComponentType::ProblemFrame),
        "objectives" => Ok(ComponentType::Objectives),
        "alternatives" => Ok(ComponentType::Alternatives),
        "consequences" => Ok(ComponentType::Consequences),
        "tradeoffs" => Ok(ComponentType::Tradeoffs),
        "recommendation" => Ok(ComponentType::Recommendation),
        "decision_quality" => Ok(ComponentType::DecisionQuality),
        "notes_next_steps" => Ok(ComponentType::NotesNextSteps),
        _ => Err(DomainError::new(
            ErrorCode::InvalidFormat,
            format!("Invalid component type: {}", s),
        )),
    }
}

fn cycle_status_to_str(status: CycleStatus) -> &'static str {
    match status {
        CycleStatus::Active => "active",
        CycleStatus::Completed => "completed",
        CycleStatus::Archived => "archived",
    }
}

fn str_to_cycle_status(s: &str) -> Result<CycleStatus, DomainError> {
    match s {
        "active" => Ok(CycleStatus::Active),
        "completed" => Ok(CycleStatus::Completed),
        "archived" => Ok(CycleStatus::Archived),
        _ => Err(DomainError::new(
            ErrorCode::InvalidFormat,
            format!("Invalid cycle status: {}", s),
        )),
    }
}

fn component_status_to_str(status: ComponentStatus) -> &'static str {
    match status {
        ComponentStatus::NotStarted => "not_started",
        ComponentStatus::InProgress => "in_progress",
        ComponentStatus::Complete => "complete",
        ComponentStatus::NeedsRevision => "needs_revision",
    }
}

fn str_to_component_status(s: &str) -> Result<ComponentStatus, DomainError> {
    match s {
        "not_started" => Ok(ComponentStatus::NotStarted),
        "in_progress" => Ok(ComponentStatus::InProgress),
        "complete" => Ok(ComponentStatus::Complete),
        "needs_revision" => Ok(ComponentStatus::NeedsRevision),
        _ => Err(DomainError::new(
            ErrorCode::InvalidFormat,
            format!("Invalid component status: {}", s),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_type_round_trips() {
        for ct in ComponentType::all() {
            let s = component_type_to_str(*ct);
            let back = str_to_component_type(s).unwrap();
            assert_eq!(*ct, back);
        }
    }

    #[test]
    fn cycle_status_round_trips() {
        let statuses = [CycleStatus::Active, CycleStatus::Completed, CycleStatus::Archived];
        for status in statuses {
            let s = cycle_status_to_str(status);
            let back = str_to_cycle_status(s).unwrap();
            assert_eq!(status, back);
        }
    }

    #[test]
    fn component_status_round_trips() {
        let statuses = [
            ComponentStatus::NotStarted,
            ComponentStatus::InProgress,
            ComponentStatus::Complete,
            ComponentStatus::NeedsRevision,
        ];
        for status in statuses {
            let s = component_status_to_str(status);
            let back = str_to_component_status(s).unwrap();
            assert_eq!(status, back);
        }
    }

    #[test]
    fn invalid_component_type_returns_error() {
        let result = str_to_component_type("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_cycle_status_returns_error() {
        let result = str_to_cycle_status("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_component_status_returns_error() {
        let result = str_to_component_status("invalid");
        assert!(result.is_err());
    }
}
