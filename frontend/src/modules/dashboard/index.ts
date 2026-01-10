/**
 * Dashboard module public API.
 *
 * Exports all domain types and API functions for dashboard functionality.
 */

// Domain types
export type {
	DashboardOverview,
	ObjectiveSummary,
	AlternativeSummary,
	CompactConsequencesTable,
	ConsequenceCell,
	RecommendationSummary,
	ComponentDetailView,
	CycleComparison,
	CycleComparisonItem,
	CycleProgressSummary,
	ComponentComparisonSummary,
	ComparisonDifference,
	ComparisonSummary
} from './domain/types';

export {
	hasContent,
	getDQCategory,
	formatDQScore,
	getProgressColor,
	hasConversation,
	getSignificanceColor
} from './domain/types';

// API functions
export {
	getDashboardOverview,
	getComponentDetail,
	compareCycles,
	ApiError
} from './api/dashboard-api';
