//! PostgreSQL implementation of CycleReader.
//!
//! Provides read-optimized queries for cycle data.

use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::foundation::{
    ComponentStatus, ComponentType, CycleId, CycleStatus, DomainError, ErrorCode, SessionId,
    Timestamp,
};
use crate::ports::{
    ComponentOutputView, ComponentStatusItem, CycleProgressView, CycleReader, CycleSummary,
    CycleTreeNode, CycleView, NextAction, NextActionType, ProgressStep,
};

/// PostgreSQL implementation of CycleReader.
#[derive(Clone)]
pub struct PostgresCycleReader {
    pool: PgPool,
}

impl PostgresCycleReader {
    /// Creates a new PostgresCycleReader.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CycleReader for PostgresCycleReader {
    async fn get_by_id(&self, id: &CycleId) -> Result<Option<CycleView>, DomainError> {
        // Fetch cycle
        let cycle_row = sqlx::query(
            r#"
            SELECT id, session_id, parent_cycle_id, branch_point, status,
                   current_step, created_at, updated_at
            FROM cycles WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| db_error(&format!("Failed to fetch cycle: {}", e)))?;

        let cycle_row = match cycle_row {
            Some(r) => r,
            None => return Ok(None),
        };

        // Fetch components
        let component_rows = sqlx::query(
            r#"
            SELECT component_type, status
            FROM components
            WHERE cycle_id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| db_error(&format!("Failed to fetch components: {}", e)))?;

        // Count branches
        let branch_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM cycles WHERE parent_cycle_id = $1",
        )
        .bind(id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| db_error(&format!("Failed to count branches: {}", e)))?;

        // Build component statuses
        let current_step: String = cycle_row.get("current_step");
        let current_step_type = str_to_component_type(&current_step)?;

        let mut component_statuses = Vec::new();
        let mut completed_count = 0u8;

        for ct in ComponentType::all() {
            let status = component_rows
                .iter()
                .find(|r| {
                    let ct_str: String = r.get("component_type");
                    str_to_component_type(&ct_str).ok() == Some(*ct)
                })
                .map(|r| {
                    let status_str: String = r.get("status");
                    str_to_component_status(&status_str).unwrap_or(ComponentStatus::NotStarted)
                })
                .unwrap_or(ComponentStatus::NotStarted);

            if status == ComponentStatus::Complete {
                completed_count += 1;
            }

            component_statuses.push(ComponentStatusItem {
                component_type: *ct,
                status,
                is_current: *ct == current_step_type,
            });
        }

        // Calculate progress (8 required components, NotesNextSteps is optional)
        let required_count = 8u8;
        let progress_percent = ((completed_count as f32 / required_count as f32) * 100.0) as u8;
        let is_complete = completed_count >= required_count;

        let status_str: String = cycle_row.get("status");
        let branch_point_str: Option<String> = cycle_row.get("branch_point");
        let parent_id: Option<Uuid> = cycle_row.get("parent_cycle_id");

        Ok(Some(CycleView {
            id: *id,
            session_id: SessionId::from_uuid(cycle_row.get("session_id")),
            parent_cycle_id: parent_id.map(CycleId::from_uuid),
            branch_point: branch_point_str
                .map(|s| str_to_component_type(&s))
                .transpose()?,
            status: str_to_cycle_status(&status_str)?,
            current_step: current_step_type,
            component_statuses,
            progress_percent,
            is_complete,
            branch_count: branch_count.0 as u32,
            created_at: Timestamp::from_datetime(cycle_row.get("created_at")),
            updated_at: Timestamp::from_datetime(cycle_row.get("updated_at")),
        }))
    }

    async fn list_by_session_id(
        &self,
        session_id: &SessionId,
    ) -> Result<Vec<CycleSummary>, DomainError> {
        let rows = sqlx::query(
            r#"
            SELECT c.id, c.parent_cycle_id, c.branch_point, c.status,
                   c.current_step, c.created_at,
                   (SELECT COUNT(*) FROM components comp
                    WHERE comp.cycle_id = c.id AND comp.status = 'complete') as completed_count
            FROM cycles c
            WHERE c.session_id = $1
            ORDER BY c.created_at DESC
            "#,
        )
        .bind(session_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| db_error(&format!("Failed to fetch cycles: {}", e)))?;

        let required_count = 8u8;
        let mut summaries = Vec::with_capacity(rows.len());

        for row in rows {
            let parent_id: Option<Uuid> = row.get("parent_cycle_id");
            let branch_point_str: Option<String> = row.get("branch_point");
            let status_str: String = row.get("status");
            let current_step_str: String = row.get("current_step");
            let completed_count: i64 = row.get("completed_count");

            let progress = ((completed_count as f32 / required_count as f32) * 100.0) as u8;

            summaries.push(CycleSummary {
                id: CycleId::from_uuid(row.get("id")),
                is_branch: parent_id.is_some(),
                branch_point: branch_point_str
                    .map(|s| str_to_component_type(&s))
                    .transpose()?,
                status: str_to_cycle_status(&status_str)?,
                current_step: str_to_component_type(&current_step_str)?,
                progress_percent: progress.min(100),
                created_at: Timestamp::from_datetime(row.get("created_at")),
            });
        }

        Ok(summaries)
    }

    async fn get_tree(&self, session_id: &SessionId) -> Result<Option<CycleTreeNode>, DomainError> {
        // Fetch all cycles for session
        let rows = sqlx::query(
            r#"
            SELECT c.id, c.parent_cycle_id, c.branch_point, c.status,
                   c.current_step, c.created_at,
                   (SELECT COUNT(*) FROM components comp
                    WHERE comp.cycle_id = c.id AND comp.status = 'complete') as completed_count
            FROM cycles c
            WHERE c.session_id = $1
            ORDER BY c.created_at ASC
            "#,
        )
        .bind(session_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| db_error(&format!("Failed to fetch cycles for tree: {}", e)))?;

        if rows.is_empty() {
            return Ok(None);
        }

        let required_count = 8u8;

        // Build summaries and parent mapping
        let mut summaries: HashMap<Uuid, CycleSummary> = HashMap::new();
        let mut parent_map: HashMap<Uuid, Option<Uuid>> = HashMap::new();

        for row in &rows {
            let id: Uuid = row.get("id");
            let parent_id: Option<Uuid> = row.get("parent_cycle_id");
            let branch_point_str: Option<String> = row.get("branch_point");
            let status_str: String = row.get("status");
            let current_step_str: String = row.get("current_step");
            let completed_count: i64 = row.get("completed_count");

            let progress = ((completed_count as f32 / required_count as f32) * 100.0) as u8;

            let summary = CycleSummary {
                id: CycleId::from_uuid(id),
                is_branch: parent_id.is_some(),
                branch_point: branch_point_str
                    .map(|s| str_to_component_type(&s))
                    .transpose()?,
                status: str_to_cycle_status(&status_str)?,
                current_step: str_to_component_type(&current_step_str)?,
                progress_percent: progress.min(100),
                created_at: Timestamp::from_datetime(row.get("created_at")),
            };

            summaries.insert(id, summary);
            parent_map.insert(id, parent_id);
        }

        // Find root (no parent)
        let root_id = parent_map
            .iter()
            .find(|(_, parent)| parent.is_none())
            .map(|(id, _)| *id);

        let root_id = match root_id {
            Some(id) => id,
            None => return Ok(None),
        };

        // Build tree recursively
        fn build_node(
            id: Uuid,
            summaries: &HashMap<Uuid, CycleSummary>,
            parent_map: &HashMap<Uuid, Option<Uuid>>,
        ) -> Option<CycleTreeNode> {
            let summary = summaries.get(&id)?.clone();

            // Find children
            let children: Vec<CycleTreeNode> = parent_map
                .iter()
                .filter(|(_, parent)| **parent == Some(id))
                .filter_map(|(child_id, _)| build_node(*child_id, summaries, parent_map))
                .collect();

            Some(CycleTreeNode {
                cycle: summary,
                children,
            })
        }

        Ok(build_node(root_id, &summaries, &parent_map))
    }

    async fn get_progress(&self, id: &CycleId) -> Result<Option<CycleProgressView>, DomainError> {
        // Fetch cycle
        let cycle_row = sqlx::query(
            r#"
            SELECT id, current_step, status
            FROM cycles WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| db_error(&format!("Failed to fetch cycle: {}", e)))?;

        let cycle_row = match cycle_row {
            Some(r) => r,
            None => return Ok(None),
        };

        let current_step_str: String = cycle_row.get("current_step");
        let current_step = str_to_component_type(&current_step_str)?;
        let cycle_status_str: String = cycle_row.get("status");
        let cycle_status = str_to_cycle_status(&cycle_status_str)?;

        // Fetch components
        let component_rows = sqlx::query(
            r#"
            SELECT component_type, status
            FROM components
            WHERE cycle_id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| db_error(&format!("Failed to fetch components: {}", e)))?;

        // Build component status map
        let mut status_map: HashMap<ComponentType, ComponentStatus> = HashMap::new();
        for row in component_rows {
            let ct_str: String = row.get("component_type");
            let status_str: String = row.get("status");
            if let (Ok(ct), Ok(status)) = (
                str_to_component_type(&ct_str),
                str_to_component_status(&status_str),
            ) {
                status_map.insert(ct, status);
            }
        }

        // Build progress steps
        let mut steps = Vec::new();
        let mut completed_count = 0u8;
        let mut has_revisions = false;
        let mut first_incomplete: Option<ComponentType> = None;
        let mut first_revision: Option<ComponentType> = None;

        // Required components (all except NotesNextSteps which is optional)
        let required_components = [
            ComponentType::IssueRaising,
            ComponentType::ProblemFrame,
            ComponentType::Objectives,
            ComponentType::Alternatives,
            ComponentType::Consequences,
            ComponentType::Tradeoffs,
            ComponentType::Recommendation,
            ComponentType::DecisionQuality,
        ];

        for (i, ct) in ComponentType::all().iter().enumerate() {
            let status = status_map
                .get(ct)
                .copied()
                .unwrap_or(ComponentStatus::NotStarted);

            let is_required = required_components.contains(ct);
            if is_required && status == ComponentStatus::Complete {
                completed_count += 1;
            }

            if status == ComponentStatus::NeedsRevision {
                has_revisions = true;
                if first_revision.is_none() {
                    first_revision = Some(*ct);
                }
            }

            if first_incomplete.is_none()
                && is_required
                && status != ComponentStatus::Complete
            {
                first_incomplete = Some(*ct);
            }

            // Accessible if previous step is complete or it's the first step
            let is_accessible = i == 0
                || ComponentType::all()
                    .get(i - 1)
                    .map(|prev| {
                        status_map.get(prev).copied() == Some(ComponentStatus::Complete)
                    })
                    .unwrap_or(false)
                || status != ComponentStatus::NotStarted;

            steps.push(ProgressStep {
                component_type: *ct,
                name: component_display_name(*ct),
                status,
                is_current: *ct == current_step,
                is_accessible,
            });
        }

        let required_count = 8u8;
        let progress_percent = ((completed_count as f32 / required_count as f32) * 100.0) as u8;
        let is_complete = completed_count >= required_count;

        // Determine next action
        let next_action = if cycle_status == CycleStatus::Completed || is_complete {
            Some(NextAction {
                action_type: NextActionType::AlreadyComplete,
                component: None,
                description: "Cycle is complete".to_string(),
            })
        } else if let Some(rev_ct) = first_revision {
            Some(NextAction {
                action_type: NextActionType::ReviseComponent,
                component: Some(rev_ct),
                description: format!("Revise {}", component_display_name(rev_ct)),
            })
        } else if status_map.get(&current_step) == Some(&ComponentStatus::InProgress) {
            Some(NextAction {
                action_type: NextActionType::ContinueCurrent,
                component: Some(current_step),
                description: format!("Continue {}", component_display_name(current_step)),
            })
        } else if let Some(next_ct) = first_incomplete {
            let action_type = if status_map.is_empty() {
                NextActionType::StartFirst
            } else {
                NextActionType::StartNext
            };
            Some(NextAction {
                action_type,
                component: Some(next_ct),
                description: format!("Start {}", component_display_name(next_ct)),
            })
        } else {
            Some(NextAction {
                action_type: NextActionType::CompleteCycle,
                component: None,
                description: "Complete the cycle".to_string(),
            })
        };

        Ok(Some(CycleProgressView {
            cycle_id: *id,
            progress_percent: progress_percent.min(100),
            completed_count,
            required_count,
            is_complete,
            has_revisions,
            steps,
            next_action,
        }))
    }

    async fn get_lineage(&self, id: &CycleId) -> Result<Vec<CycleSummary>, DomainError> {
        // Use recursive CTE to get lineage
        let rows = sqlx::query(
            r#"
            WITH RECURSIVE lineage AS (
                SELECT id, parent_cycle_id, branch_point, status, current_step, created_at, 0 as depth
                FROM cycles WHERE id = $1

                UNION ALL

                SELECT c.id, c.parent_cycle_id, c.branch_point, c.status, c.current_step, c.created_at, l.depth + 1
                FROM cycles c
                JOIN lineage l ON c.id = l.parent_cycle_id
            )
            SELECT l.id, l.parent_cycle_id, l.branch_point, l.status, l.current_step, l.created_at,
                   (SELECT COUNT(*) FROM components comp
                    WHERE comp.cycle_id = l.id AND comp.status = 'complete') as completed_count
            FROM lineage l
            ORDER BY l.depth DESC
            "#,
        )
        .bind(id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| db_error(&format!("Failed to fetch lineage: {}", e)))?;

        let required_count = 8u8;
        let mut summaries = Vec::with_capacity(rows.len());

        for row in rows {
            let parent_id: Option<Uuid> = row.get("parent_cycle_id");
            let branch_point_str: Option<String> = row.get("branch_point");
            let status_str: String = row.get("status");
            let current_step_str: String = row.get("current_step");
            let completed_count: i64 = row.get("completed_count");

            let progress = ((completed_count as f32 / required_count as f32) * 100.0) as u8;

            summaries.push(CycleSummary {
                id: CycleId::from_uuid(row.get("id")),
                is_branch: parent_id.is_some(),
                branch_point: branch_point_str
                    .map(|s| str_to_component_type(&s))
                    .transpose()?,
                status: str_to_cycle_status(&status_str)?,
                current_step: str_to_component_type(&current_step_str)?,
                progress_percent: progress.min(100),
                created_at: Timestamp::from_datetime(row.get("created_at")),
            });
        }

        Ok(summaries)
    }

    async fn get_component_output(
        &self,
        cycle_id: &CycleId,
        component_type: ComponentType,
    ) -> Result<Option<ComponentOutputView>, DomainError> {
        let component_type_str = component_type_to_str(component_type);

        let row = sqlx::query(
            r#"
            SELECT c.id as cycle_id, comp.component_type, comp.status, comp.output, comp.updated_at
            FROM cycles c
            JOIN components comp ON comp.cycle_id = c.id
            WHERE c.id = $1 AND comp.component_type = $2
            "#,
        )
        .bind(cycle_id.as_uuid())
        .bind(component_type_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| db_error(&format!("Failed to fetch component output: {}", e)))?;

        match row {
            Some(row) => {
                let status_str: String = row.get("status");
                let output: serde_json::Value = row.get("output");

                Ok(Some(ComponentOutputView {
                    cycle_id: *cycle_id,
                    component_type,
                    status: str_to_component_status(&status_str)?,
                    output,
                    updated_at: Timestamp::from_datetime(row.get("updated_at")),
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_proact_tree_view(
        &self,
        session_id: &SessionId,
    ) -> Result<Option<crate::domain::cycle::CycleTreeNode>, DomainError> {
        use crate::domain::cycle::{LetterStatus, PrOACTLetter, PrOACTStatus, CycleTreeNode as PrOACTTreeNode};

        // Fetch all cycles with their component statuses
        let cycle_rows = sqlx::query(
            r#"
            SELECT c.id, c.parent_cycle_id, c.branch_point, c.updated_at
            FROM cycles c
            WHERE c.session_id = $1
            ORDER BY c.created_at ASC
            "#,
        )
        .bind(session_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| db_error(&format!("Failed to fetch cycles for PrOACT tree: {}", e)))?;

        if cycle_rows.is_empty() {
            return Ok(None);
        }

        // Fetch all components for all cycles in this session
        let component_rows = sqlx::query(
            r#"
            SELECT comp.cycle_id, comp.component_type, comp.status
            FROM components comp
            JOIN cycles c ON c.id = comp.cycle_id
            WHERE c.session_id = $1
            "#,
        )
        .bind(session_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| db_error(&format!("Failed to fetch components for PrOACT tree: {}", e)))?;

        // Build status map: cycle_id -> component_type -> status
        let mut cycle_components: HashMap<Uuid, HashMap<ComponentType, ComponentStatus>> = HashMap::new();

        for row in component_rows {
            let cycle_id: Uuid = row.get("cycle_id");
            let ct_str: String = row.get("component_type");
            let status_str: String = row.get("status");

            let ct = str_to_component_type(&ct_str)?;
            let status = str_to_component_status(&status_str)?;

            cycle_components
                .entry(cycle_id)
                .or_default()
                .insert(ct, status);
        }

        // Build PrOACT nodes and parent mapping
        let mut nodes: HashMap<Uuid, PrOACTTreeNode> = HashMap::new();
        let mut parent_map: HashMap<Uuid, Option<Uuid>> = HashMap::new();

        for row in &cycle_rows {
            let id: Uuid = row.get("id");
            let parent_id: Option<Uuid> = row.get("parent_cycle_id");
            let branch_point_str: Option<String> = row.get("branch_point");
            let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

            // Get component statuses for this cycle
            let component_statuses = cycle_components.get(&id).cloned().unwrap_or_default();

            // Map component statuses to PrOACT letter statuses
            let letter_statuses = component_statuses_to_proact_status(&component_statuses);

            // Convert branch_point to PrOACTLetter
            let branch_point = branch_point_str
                .and_then(|s| str_to_component_type(&s).ok())
                .and_then(component_type_to_proact_letter);

            let node = PrOACTTreeNode {
                cycle_id: CycleId::from_uuid(id),
                label: format!("Cycle {}", &id.to_string()[..8]),  // Default label, TODO: load from DB
                branch_point,
                letter_statuses,
                children: Vec::new(),  // Will be filled later
                updated_at,
            };

            nodes.insert(id, node);
            parent_map.insert(id, parent_id);
        }

        // Find root (no parent)
        let root_id = parent_map
            .iter()
            .find(|(_, parent)| parent.is_none())
            .map(|(id, _)| *id);

        let root_id = match root_id {
            Some(id) => id,
            None => return Ok(None),
        };

        // Build tree recursively
        fn build_proact_tree(
            id: Uuid,
            nodes: &mut HashMap<Uuid, PrOACTTreeNode>,
            parent_map: &HashMap<Uuid, Option<Uuid>>,
        ) -> Option<PrOACTTreeNode> {
            let mut node = nodes.remove(&id)?;

            // Find children
            let child_ids: Vec<Uuid> = parent_map
                .iter()
                .filter(|(_, parent)| **parent == Some(id))
                .map(|(child_id, _)| *child_id)
                .collect();

            node.children = child_ids
                .into_iter()
                .filter_map(|child_id| build_proact_tree(child_id, nodes, parent_map))
                .collect();

            Some(node)
        }

        Ok(build_proact_tree(root_id, &mut nodes, &parent_map))
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Helper Functions
// ════════════════════════════════════════════════════════════════════════════════

fn db_error(msg: &str) -> DomainError {
    DomainError::new(ErrorCode::DatabaseError, msg.to_string())
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

fn component_display_name(ct: ComponentType) -> String {
    match ct {
        ComponentType::IssueRaising => "Issue Raising".to_string(),
        ComponentType::ProblemFrame => "Problem Frame".to_string(),
        ComponentType::Objectives => "Objectives".to_string(),
        ComponentType::Alternatives => "Alternatives".to_string(),
        ComponentType::Consequences => "Consequences".to_string(),
        ComponentType::Tradeoffs => "Tradeoffs".to_string(),
        ComponentType::Recommendation => "Recommendation".to_string(),
        ComponentType::DecisionQuality => "Decision Quality".to_string(),
        ComponentType::NotesNextSteps => "Notes & Next Steps".to_string(),
    }
}

/// Maps a ComponentType to its corresponding PrOACTLetter.
///
/// Note: IssueRaising and NotesNextSteps don't map to PrOACT letters
/// as they are pre/post steps, not part of the core framework.
fn component_type_to_proact_letter(ct: ComponentType) -> Option<crate::domain::cycle::PrOACTLetter> {
    use crate::domain::cycle::PrOACTLetter;

    match ct {
        ComponentType::ProblemFrame => Some(PrOACTLetter::P),
        ComponentType::Objectives => Some(PrOACTLetter::R),
        ComponentType::Alternatives => Some(PrOACTLetter::O),
        ComponentType::Consequences => Some(PrOACTLetter::A),
        ComponentType::Tradeoffs => Some(PrOACTLetter::C),
        ComponentType::Recommendation | ComponentType::DecisionQuality => Some(PrOACTLetter::T),
        ComponentType::IssueRaising | ComponentType::NotesNextSteps => None,
    }
}

/// Converts a map of component statuses to PrOACT letter statuses.
///
/// Aggregates component statuses by their PrOACT letter. For letter T,
/// which maps to both Recommendation and DecisionQuality, the status is:
/// - Completed: both are complete
/// - InProgress: at least one is in progress
/// - NotStarted: neither is started
fn component_statuses_to_proact_status(
    statuses: &HashMap<ComponentType, ComponentStatus>,
) -> crate::domain::cycle::PrOACTStatus {
    use crate::domain::cycle::{LetterStatus, PrOACTStatus};

    // Helper to convert ComponentStatus to LetterStatus
    fn to_letter_status(status: ComponentStatus) -> LetterStatus {
        match status {
            ComponentStatus::Complete => LetterStatus::Completed,
            ComponentStatus::InProgress | ComponentStatus::NeedsRevision => LetterStatus::InProgress,
            ComponentStatus::NotStarted => LetterStatus::NotStarted,
        }
    }

    // Get status for a single component type
    fn get_status(
        statuses: &HashMap<ComponentType, ComponentStatus>,
        ct: ComponentType,
    ) -> LetterStatus {
        statuses
            .get(&ct)
            .copied()
            .map(to_letter_status)
            .unwrap_or(LetterStatus::NotStarted)
    }

    // Get combined status for T (Recommendation + DecisionQuality)
    fn get_t_status(statuses: &HashMap<ComponentType, ComponentStatus>) -> LetterStatus {
        let rec_status = get_status(statuses, ComponentType::Recommendation);
        let dq_status = get_status(statuses, ComponentType::DecisionQuality);

        match (rec_status, dq_status) {
            (LetterStatus::Completed, LetterStatus::Completed) => LetterStatus::Completed,
            (LetterStatus::NotStarted, LetterStatus::NotStarted) => LetterStatus::NotStarted,
            _ => LetterStatus::InProgress,
        }
    }

    PrOACTStatus {
        p: get_status(statuses, ComponentType::ProblemFrame),
        r: get_status(statuses, ComponentType::Objectives),
        o: get_status(statuses, ComponentType::Alternatives),
        a: get_status(statuses, ComponentType::Consequences),
        c: get_status(statuses, ComponentType::Tradeoffs),
        t: get_t_status(statuses),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_type_conversion_round_trips() {
        let types = [
            ("issue_raising", ComponentType::IssueRaising),
            ("problem_frame", ComponentType::ProblemFrame),
            ("objectives", ComponentType::Objectives),
            ("alternatives", ComponentType::Alternatives),
            ("consequences", ComponentType::Consequences),
            ("tradeoffs", ComponentType::Tradeoffs),
            ("recommendation", ComponentType::Recommendation),
            ("decision_quality", ComponentType::DecisionQuality),
            ("notes_next_steps", ComponentType::NotesNextSteps),
        ];

        for (s, expected) in types {
            let result = str_to_component_type(s).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn cycle_status_conversion_round_trips() {
        let statuses = [
            ("active", CycleStatus::Active),
            ("completed", CycleStatus::Completed),
            ("archived", CycleStatus::Archived),
        ];

        for (s, expected) in statuses {
            let result = str_to_cycle_status(s).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn component_status_conversion_round_trips() {
        let statuses = [
            ("not_started", ComponentStatus::NotStarted),
            ("in_progress", ComponentStatus::InProgress),
            ("complete", ComponentStatus::Complete),
            ("needs_revision", ComponentStatus::NeedsRevision),
        ];

        for (s, expected) in statuses {
            let result = str_to_component_status(s).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn invalid_component_type_returns_error() {
        let result = str_to_component_type("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn component_display_names_are_human_readable() {
        assert_eq!(
            component_display_name(ComponentType::IssueRaising),
            "Issue Raising"
        );
        assert_eq!(
            component_display_name(ComponentType::DecisionQuality),
            "Decision Quality"
        );
        assert_eq!(
            component_display_name(ComponentType::NotesNextSteps),
            "Notes & Next Steps"
        );
    }
}
