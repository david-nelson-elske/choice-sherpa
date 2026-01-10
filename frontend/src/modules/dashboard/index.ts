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

// Components
export { default as DashboardLayout } from './components/DashboardLayout.svelte';
export { default as OverviewPanel } from './components/OverviewPanel.svelte';
export { default as CycleTreeSidebar } from './components/CycleTreeSidebar.svelte';
export { default as ComponentDetailDrawer } from './components/ComponentDetailDrawer.svelte';
export { default as DecisionStatement } from './components/DecisionStatement.svelte';
export { default as ObjectivesList } from './components/ObjectivesList.svelte';
export { default as AlternativesPills } from './components/AlternativesPills.svelte';
export { default as ConsequencesMatrix } from './components/ConsequencesMatrix.svelte';
export { default as RecommendationCard } from './components/RecommendationCard.svelte';
export { default as DQScoreBadge } from './components/DQScoreBadge.svelte';
