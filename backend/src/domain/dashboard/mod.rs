pub mod component_detail;
pub mod cycle_comparison;
pub mod overview;

pub use component_detail::ComponentDetailView;
pub use cycle_comparison::{
    ComparisonDifference, ComparisonSummary, ComponentComparisonSummary, CycleComparison,
    CycleComparisonItem, CycleProgressSnapshot, DifferenceSignificance,
};
pub use overview::{
    AlternativeSummary, CellColor, CellSummary, CompactConsequencesTable, DashboardOverview,
    ObjectiveSummary, RecommendationSummary,
};
