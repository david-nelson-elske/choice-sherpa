/**
 * Dashboard domain types for frontend.
 *
 * These types mirror the backend DTOs and domain types,
 * providing type-safe access to dashboard data.
 */

import type { ComponentType } from '../../cycle/domain/types';

// ─────────────────────────────────────────────────────────────────────
// Dashboard Overview Types
// ─────────────────────────────────────────────────────────────────────

/**
 * Main dashboard view aggregating key decision data.
 */
export interface DashboardOverview {
	session_id: string;
	session_title: string;
	decision_statement: string | null;
	objectives: ObjectiveSummary[];
	alternatives: AlternativeSummary[];
	consequences_table: CompactConsequencesTable | null;
	recommendation: RecommendationSummary | null;
	dq_score: number | null;
	active_cycle_id: string | null;
	cycle_count: number;
	last_updated: string;
}

/**
 * Objective summary for dashboard display.
 */
export interface ObjectiveSummary {
	text: string;
	type: 'fundamental' | 'means';
	measure: string | null;
}

/**
 * Alternative summary for dashboard display.
 */
export interface AlternativeSummary {
	name: string;
	description: string | null;
	is_status_quo: boolean;
}

/**
 * Compact consequences table for dashboard.
 */
export interface CompactConsequencesTable {
	objectives: string[];
	alternatives: string[];
	cells: ConsequenceCell[][];
}

/**
 * Single cell in consequences table.
 */
export interface ConsequenceCell {
	objective_index: number;
	alternative_index: number;
	rating: number | null;
	note: string | null;
}

/**
 * Recommendation summary for dashboard.
 */
export interface RecommendationSummary {
	recommended_alternative: string | null;
	rationale_preview: string | null;
}

// ─────────────────────────────────────────────────────────────────────
// Component Detail Types
// ─────────────────────────────────────────────────────────────────────

/**
 * Detailed view of a single component.
 */
export interface ComponentDetailView {
	component_id: string;
	cycle_id: string;
	component_type: ComponentType;
	status: 'not_started' | 'in_progress' | 'complete';
	structured_output: unknown;
	conversation_message_count: number;
	last_message_at: string | null;
	can_branch: boolean;
	can_revise: boolean;
	previous_component: ComponentType | null;
	next_component: ComponentType | null;
}

// ─────────────────────────────────────────────────────────────────────
// Cycle Comparison Types
// ─────────────────────────────────────────────────────────────────────

/**
 * Comparison view for multiple cycles side-by-side.
 */
export interface CycleComparison {
	cycles: CycleComparisonItem[];
	differences: ComparisonDifference[];
	summary: ComparisonSummary;
}

/**
 * Single cycle in a comparison view.
 */
export interface CycleComparisonItem {
	cycle_id: string;
	branch_point: ComponentType | null;
	progress: CycleProgressSummary;
	component_summaries: ComponentComparisonSummary[];
}

/**
 * Progress summary for a cycle.
 */
export interface CycleProgressSummary {
	completed_count: number;
	total_count: number;
	percent: number;
	current_step: ComponentType | null;
}

/**
 * Component summary for comparison.
 */
export interface ComponentComparisonSummary {
	component_type: ComponentType;
	summary: string;
	differs_from_others: boolean;
}

/**
 * A detected difference between cycles.
 */
export interface ComparisonDifference {
	component_type: ComponentType;
	cycle_id: string;
	description: string;
	significance: 'minor' | 'moderate' | 'major';
}

/**
 * High-level comparison summary.
 */
export interface ComparisonSummary {
	total_cycles: number;
	components_with_differences: number;
	most_different_cycle: string | null;
	recommendation_differs: boolean;
}

// ─────────────────────────────────────────────────────────────────────
// Helper Functions
// ─────────────────────────────────────────────────────────────────────

/**
 * Check if dashboard has meaningful data to display.
 */
export function hasContent(overview: DashboardOverview): boolean {
	return (
		overview.objectives.length > 0 ||
		overview.alternatives.length > 0 ||
		overview.consequences_table !== null ||
		overview.recommendation !== null
	);
}

/**
 * Get DQ score category (0-49: Low, 50-79: Medium, 80-100: High).
 */
export function getDQCategory(score: number | null): 'low' | 'medium' | 'high' | 'none' {
	if (score === null) return 'none';
	if (score < 50) return 'low';
	if (score < 80) return 'medium';
	return 'high';
}

/**
 * Format DQ score for display.
 */
export function formatDQScore(score: number | null): string {
	if (score === null) return 'Not yet assessed';
	return `${score}%`;
}

/**
 * Get progress color class based on percentage.
 */
export function getProgressColor(percent: number): string {
	if (percent < 30) return 'text-red-600';
	if (percent < 70) return 'text-yellow-600';
	return 'text-green-600';
}

/**
 * Check if component detail has conversation history.
 */
export function hasConversation(detail: ComponentDetailView): boolean {
	return detail.conversation_message_count > 0;
}

/**
 * Get significance color for comparison differences.
 */
export function getSignificanceColor(significance: 'minor' | 'moderate' | 'major'): string {
	switch (significance) {
		case 'minor':
			return 'text-gray-600';
		case 'moderate':
			return 'text-yellow-600';
		case 'major':
			return 'text-red-600';
	}
}
