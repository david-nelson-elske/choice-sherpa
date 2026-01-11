use serde::Serialize;
use crate::domain::foundation::{ComponentType, CycleId};

/// Comparison view for multiple cycles
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleComparison {
    pub cycles: Vec<CycleComparisonItem>,
    pub differences: Vec<ComparisonDifference>,
    pub summary: ComparisonSummary,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleComparisonItem {
    pub cycle_id: CycleId,
    /// Where this cycle branched (if applicable)
    pub branch_point: Option<ComponentType>,
    pub progress: CycleProgressSnapshot,
    /// Component outputs for comparison
    pub component_summaries: Vec<ComponentComparisonSummary>,
}

/// Simplified progress snapshot for comparison view
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleProgressSnapshot {
    pub completed_count: usize,
    pub total_count: usize,
    pub percent_complete: u8,
    pub current_step: Option<ComponentType>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentComparisonSummary {
    pub component_type: ComponentType,
    /// Short summary of output (varies by component)
    pub summary: String,
    /// Key differences from other cycles
    pub differs_from_others: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonDifference {
    pub component_type: ComponentType,
    pub cycle_id: CycleId,
    pub description: String,
    pub significance: DifferenceSignificance,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DifferenceSignificance {
    Minor,
    Moderate,
    Major,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonSummary {
    pub total_cycles: usize,
    pub components_with_differences: usize,
    pub most_different_cycle: Option<CycleId>,
    pub recommendation_differs: bool,
}

#[cfg(test)]
#[path = "cycle_comparison_test.rs"]
mod cycle_comparison_test;
