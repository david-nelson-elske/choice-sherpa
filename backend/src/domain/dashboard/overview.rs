use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::domain::foundation::{CycleId, Percentage, SessionId};

/// The main dashboard overview - aggregates all component data
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardOverview {
    /// Session information
    pub session_id: SessionId,
    pub session_title: String,

    /// From ProblemFrame component
    pub decision_statement: Option<String>,

    /// Summary of objectives
    pub objectives: Vec<ObjectiveSummary>,

    /// List of alternatives with scores
    pub alternatives: Vec<AlternativeSummary>,

    /// Compact consequences table
    pub consequences_table: Option<CompactConsequencesTable>,

    /// Recommendation summary
    pub recommendation: Option<RecommendationSummary>,

    /// Decision Quality score
    pub dq_score: Option<Percentage>,

    /// Active cycle information
    pub active_cycle_id: Option<CycleId>,
    pub cycle_count: usize,

    /// Timestamps
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectiveSummary {
    pub id: String,
    pub description: String,
    pub is_fundamental: bool,
    /// Performance measure abbreviation
    pub measure: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct CompactConsequencesTable {
    /// Column headers (alternative names)
    pub alternative_names: Vec<String>,
    /// Row headers (objective names)
    pub objective_names: Vec<String>,
    /// Cell data [objective_index][alternative_index]
    pub cells: Vec<Vec<CellSummary>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellSummary {
    pub rating: i8,
    pub color: CellColor,
    /// Truncated explanation (first 50 chars)
    pub explanation_preview: Option<String>,
}

/// Cell color based on rating
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CellColor {
    Red,    // -2, -1
    Yellow, // 0
    Green,  // +1, +2
}

impl From<i8> for CellColor {
    fn from(rating: i8) -> Self {
        match rating {
            -2 | -1 => CellColor::Red,
            0 => CellColor::Yellow,
            1 | 2 => CellColor::Green,
            _ => CellColor::Yellow, // Default for invalid ratings
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
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

#[cfg(test)]
#[path = "overview_test.rs"]
mod overview_test;
