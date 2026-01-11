//! Dashboard query handlers.
//!
//! Read-only handlers for aggregating and viewing dashboard data.

mod compare_cycles;
mod get_component_detail;
mod get_dashboard_overview;

pub use compare_cycles::{CompareCyclesHandler, CompareCyclesQuery, CompareCyclesResult};
pub use get_component_detail::{
    GetComponentDetailHandler, GetComponentDetailQuery, GetComponentDetailResult,
};
pub use get_dashboard_overview::{
    GetDashboardOverviewHandler, GetDashboardOverviewQuery, GetDashboardOverviewResult,
};
