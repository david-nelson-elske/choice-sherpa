//! PostgreSQL implementation of DashboardReader.
//!
//! Provides read-optimized queries for dashboard aggregation across
//! sessions, cycles, and components.

use async_trait::async_trait;
use serde_json::Value as JsonValue;
use sqlx::{PgPool, Row};

use crate::domain::dashboard::{
    AlternativeSummary, ComparisonSummary, ComponentDetailView, CycleComparison,
    DashboardOverview, ObjectiveSummary,
};
use crate::domain::foundation::{
    ComponentId, ComponentStatus, ComponentType, CycleId, SessionId, UserId,
};
use crate::ports::{DashboardError, DashboardReader};

/// PostgreSQL implementation of DashboardReader.
#[derive(Clone)]
pub struct PostgresDashboardReader {
    pool: PgPool,
}

impl PostgresDashboardReader {
    /// Creates a new PostgresDashboardReader.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Verifies user owns the session.
    async fn verify_session_ownership(
        &self,
        session_id: &SessionId,
        user_id: &UserId,
    ) -> Result<(), DashboardError> {
        let row = sqlx::query(
            r#"
            SELECT user_id FROM sessions WHERE id = $1
            "#,
        )
        .bind(session_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DashboardError::Database(e.to_string()))?;

        match row {
            None => Err(DashboardError::SessionNotFound(*session_id)),
            Some(row) => {
                let owner_id: String = row.get("user_id");
                if owner_id == user_id.as_str() {
                    Ok(())
                } else {
                    Err(DashboardError::Unauthorized)
                }
            }
        }
    }

    /// Gets the active cycle ID for a session (most recently updated).
    async fn get_active_cycle_id(
        &self,
        session_id: &SessionId,
    ) -> Result<Option<CycleId>, DashboardError> {
        let row = sqlx::query(
            r#"
            SELECT id FROM cycles
            WHERE session_id = $1 AND status != 'archived'
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(session_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DashboardError::Database(e.to_string()))?;

        Ok(row.map(|r| {
            let uuid: uuid::Uuid = r.get("id");
            CycleId::from_uuid(uuid)
        }))
    }

    /// Gets component structured output as JSON.
    async fn get_component_output(
        &self,
        cycle_id: &CycleId,
        component_type: ComponentType,
    ) -> Result<Option<JsonValue>, DashboardError> {
        let row = sqlx::query(
            r#"
            SELECT structured_data FROM components
            WHERE cycle_id = $1 AND component_type = $2
            "#,
        )
        .bind(cycle_id.as_uuid())
        .bind(component_type_to_str(component_type))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DashboardError::Database(e.to_string()))?;

        Ok(row.and_then(|r| r.get("structured_data")))
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
        // Verify authorization
        self.verify_session_ownership(&session_id, user_id).await?;

        // Get session info
        let session_row = sqlx::query(
            r#"
            SELECT title, created_at
            FROM sessions WHERE id = $1
            "#,
        )
        .bind(session_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DashboardError::Database(e.to_string()))?
        .ok_or(DashboardError::SessionNotFound(session_id))?;

        let session_title: String = session_row.get("title");

        // Count cycles for this session
        let cycle_count_row = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM cycles WHERE session_id = $1
            "#,
        )
        .bind(session_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DashboardError::Database(e.to_string()))?;

        let cycle_count: i64 = cycle_count_row.get("count");

        // Determine which cycle to use
        let target_cycle_id = match cycle_id {
            Some(id) => id,
            None => {
                self.get_active_cycle_id(&session_id)
                    .await?
                    .ok_or(DashboardError::CycleNotFound(CycleId::new()))?
            }
        };

        // Get decision statement from ProblemFrame component
        let decision_statement = self
            .get_component_output(&target_cycle_id, ComponentType::ProblemFrame)
            .await?
            .and_then(|json| {
                json.get("decision_statement")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            });

        // Get objectives from Objectives component
        let objectives = self
            .get_component_output(&target_cycle_id, ComponentType::Objectives)
            .await?
            .and_then(|json| {
                json.get("objectives").and_then(|obj_array| {
                    obj_array.as_array().map(|arr| {
                        arr.iter()
                            .filter_map(|obj| {
                                Some(ObjectiveSummary {
                                    id: obj.get("id")?.as_str()?.to_string(),
                                    description: obj.get("description")?.as_str()?.to_string(),
                                    is_fundamental: obj
                                        .get("is_fundamental")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false),
                                    measure: obj
                                        .get("measure")
                                        .and_then(|v| v.as_str())
                                        .map(String::from),
                                })
                            })
                            .collect()
                    })
                })
            })
            .unwrap_or_default();

        // Get alternatives from Alternatives component
        let alternatives = self
            .get_component_output(&target_cycle_id, ComponentType::Alternatives)
            .await?
            .and_then(|json| {
                json.get("alternatives").and_then(|alt_array| {
                    alt_array.as_array().map(|arr| {
                        arr.iter()
                            .filter_map(|alt| {
                                Some(AlternativeSummary {
                                    id: alt.get("id")?.as_str()?.to_string(),
                                    name: alt.get("name")?.as_str()?.to_string(),
                                    is_status_quo: alt
                                        .get("is_status_quo")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false),
                                    pugh_score: None, // TODO: Calculate from Consequences
                                    rank: None,       // TODO: Calculate from scores
                                    is_dominated: false, // TODO: Calculate from analysis
                                })
                            })
                            .collect()
                    })
                })
            })
            .unwrap_or_default();

        // TODO: Build consequences table from Consequences component
        let consequences_table = None;

        // TODO: Build recommendation summary from Recommendation component
        let recommendation = None;

        // TODO: Get DQ score from DecisionQuality component
        let dq_score = None;

        Ok(DashboardOverview {
            session_id,
            session_title,
            decision_statement,
            objectives,
            alternatives,
            consequences_table,
            recommendation,
            dq_score,
            active_cycle_id: Some(target_cycle_id),
            cycle_count: cycle_count as usize,
            last_updated: chrono::Utc::now(),
        })
    }

    async fn get_component_detail(
        &self,
        cycle_id: CycleId,
        component_type: ComponentType,
        user_id: &UserId,
    ) -> Result<ComponentDetailView, DashboardError> {
        // Get cycle's session_id for authorization
        let cycle_row = sqlx::query(
            r#"
            SELECT session_id FROM cycles WHERE id = $1
            "#,
        )
        .bind(cycle_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DashboardError::Database(e.to_string()))?
        .ok_or(DashboardError::CycleNotFound(cycle_id))?;

        let session_uuid: uuid::Uuid = cycle_row.get("session_id");
        let session_id = SessionId::from_uuid(session_uuid);

        // Verify authorization
        self.verify_session_ownership(&session_id, user_id).await?;

        // Get component data
        let component_row = sqlx::query(
            r#"
            SELECT id, status, structured_data
            FROM components
            WHERE cycle_id = $1 AND component_type = $2
            "#,
        )
        .bind(cycle_id.as_uuid())
        .bind(component_type_to_str(component_type))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DashboardError::Database(e.to_string()))?
        .ok_or(DashboardError::ComponentNotFound(component_type))?;

        let component_uuid: uuid::Uuid = component_row.get("id");
        let component_id = ComponentId::from_uuid(component_uuid);

        let status_str: String = component_row.get("status");
        let status = str_to_component_status(&status_str)
            .map_err(|_| DashboardError::Database(format!("Invalid status: {}", status_str)))?;

        let structured_output: JsonValue = component_row
            .get("structured_data");

        // TODO: Get conversation metadata
        let conversation_message_count = 0;
        let last_message_at = None;

        // TODO: Determine navigation context
        let previous_component = component_type.previous();
        let next_component = component_type.next();

        // TODO: Determine action flags
        let can_branch = status == ComponentStatus::Complete;
        let can_revise = status == ComponentStatus::Complete;

        Ok(ComponentDetailView {
            component_id,
            cycle_id,
            component_type,
            status,
            structured_output,
            conversation_message_count,
            last_message_at,
            can_branch,
            can_revise,
            previous_component,
            next_component,
        })
    }

    async fn compare_cycles(
        &self,
        cycle_ids: &[CycleId],
        user_id: &UserId,
    ) -> Result<CycleComparison, DashboardError> {
        if cycle_ids.is_empty() {
            return Err(DashboardError::InvalidInput(
                "At least one cycle required".to_string(),
            ));
        }

        // Verify all cycles belong to sessions owned by user
        for cycle_id in cycle_ids {
            let cycle_row = sqlx::query(
                r#"
                SELECT session_id FROM cycles WHERE id = $1
                "#,
            )
            .bind(cycle_id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DashboardError::Database(e.to_string()))?
            .ok_or(DashboardError::CycleNotFound(*cycle_id))?;

            let session_uuid: uuid::Uuid = cycle_row.get("session_id");
            let session_id = SessionId::from_uuid(session_uuid);
            self.verify_session_ownership(&session_id, user_id).await?;
        }

        // TODO: Build comparison items for each cycle
        let cycles = vec![];

        // TODO: Identify differences between cycles
        let differences = vec![];

        // TODO: Build comparison summary
        let summary = ComparisonSummary {
            total_cycles: cycle_ids.len(),
            components_with_differences: 0,
            most_different_cycle: None,
            recommendation_differs: false,
        };

        Ok(CycleComparison {
            cycles,
            differences,
            summary,
        })
    }
}

// Helper functions

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

fn str_to_component_status(s: &str) -> Result<ComponentStatus, String> {
    match s {
        "not_started" => Ok(ComponentStatus::NotStarted),
        "in_progress" => Ok(ComponentStatus::InProgress),
        "complete" => Ok(ComponentStatus::Complete),
        _ => Err(format!("Unknown component status: {}", s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_to_component_status() {
        assert_eq!(
            str_to_component_status("not_started").unwrap(),
            ComponentStatus::NotStarted
        );
        assert_eq!(
            str_to_component_status("in_progress").unwrap(),
            ComponentStatus::InProgress
        );
        assert_eq!(
            str_to_component_status("complete").unwrap(),
            ComponentStatus::Complete
        );
        assert!(str_to_component_status("invalid").is_err());
    }
}
