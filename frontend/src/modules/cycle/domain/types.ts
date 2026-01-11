/**
 * Cycle domain types for frontend.
 *
 * These types mirror the backend DTOs and domain types,
 * providing type-safe access to cycle data.
 */

// ─────────────────────────────────────────────────────────────────────
// Component Types (matches backend ComponentType enum)
// ─────────────────────────────────────────────────────────────────────

export type ComponentType =
	| 'issue_raising'
	| 'problem_frame'
	| 'objectives'
	| 'alternatives'
	| 'consequences'
	| 'tradeoffs'
	| 'recommendation'
	| 'decision_quality'
	| 'notes_next_steps';

export const COMPONENT_ORDER: ComponentType[] = [
	'issue_raising',
	'problem_frame',
	'objectives',
	'alternatives',
	'consequences',
	'tradeoffs',
	'recommendation',
	'decision_quality',
	'notes_next_steps'
];

export const COMPONENT_LABELS: Record<ComponentType, string> = {
	issue_raising: 'Issue Raising',
	problem_frame: 'Problem Frame',
	objectives: 'Objectives',
	alternatives: 'Alternatives',
	consequences: 'Consequences',
	tradeoffs: 'Tradeoffs',
	recommendation: 'Recommendation',
	decision_quality: 'Decision Quality',
	notes_next_steps: 'Notes & Next Steps'
};

// ─────────────────────────────────────────────────────────────────────
// Status Types
// ─────────────────────────────────────────────────────────────────────

export type CycleStatus = 'active' | 'completed' | 'archived';

export type ComponentStatus = 'not_started' | 'in_progress' | 'complete';

// ─────────────────────────────────────────────────────────────────────
// Cycle Types
// ─────────────────────────────────────────────────────────────────────

/** Status of a single component. */
export interface ComponentStatusItem {
	component_type: ComponentType;
	status: ComponentStatus;
	is_current: boolean;
}

/** Full cycle view for display. */
export interface CycleView {
	id: string;
	session_id: string;
	parent_cycle_id: string | null;
	branch_point: ComponentType | null;
	status: CycleStatus;
	current_step: ComponentType;
	component_statuses: ComponentStatusItem[];
	progress_percent: number;
	is_complete: boolean;
	branch_count: number;
	created_at: string;
	updated_at: string;
}

/** Summary of a cycle for lists and trees. */
export interface CycleSummary {
	id: string;
	is_branch: boolean;
	branch_point: ComponentType | null;
	status: CycleStatus;
	current_step: ComponentType;
	progress_percent: number;
	created_at: string;
}

// ─────────────────────────────────────────────────────────────────────
// Progress Types
// ─────────────────────────────────────────────────────────────────────

/** Detailed progress view. */
export interface CycleProgressView {
	cycle_id: string;
	completed_count: number;
	total_count: number;
	percent: number;
	current_step: ComponentType;
	component_statuses: ComponentStatusItem[];
}

// ─────────────────────────────────────────────────────────────────────
// Tree Types
// ─────────────────────────────────────────────────────────────────────

/** Node in the cycle tree. */
export interface CycleTreeNode {
	cycle: CycleSummary;
	children: CycleTreeNode[];
}

// ─────────────────────────────────────────────────────────────────────
// PrOACT Tree Types (specialized visualization)
// ─────────────────────────────────────────────────────────────────────

/** A single letter in the PrOACT acronym. */
export type PrOACTLetter = 'P' | 'R' | 'O' | 'A' | 'C' | 'T';

/** Status of a single PrOACT letter. */
export type LetterStatus = 'not_started' | 'in_progress' | 'completed';

/** Status for all six PrOACT letters. */
export interface PrOACTStatus {
	p: LetterStatus; // Problem Frame
	r: LetterStatus; // Objectives (what Really matters)
	o: LetterStatus; // Options/Alternatives
	a: LetterStatus; // Analysis/Consequences
	c: LetterStatus; // Clear Tradeoffs
	t: LetterStatus; // Think Through (Recommendation + Decision Quality)
}

/** Node in the PrOACT tree visualization. */
export interface PrOACTTreeNode {
	cycle_id: string;
	label: string;
	branch_point: PrOACTLetter | null;
	letter_statuses: PrOACTStatus;
	children: PrOACTTreeNode[];
	updated_at: string;
}

// ─────────────────────────────────────────────────────────────────────
// Component Output Types
// ─────────────────────────────────────────────────────────────────────

/** Component output view. */
export interface ComponentOutputView {
	cycle_id: string;
	component_type: ComponentType;
	status: ComponentStatus;
	output: unknown;
	updated_at: string;
}

// ─────────────────────────────────────────────────────────────────────
// Helper Functions
// ─────────────────────────────────────────────────────────────────────

/**
 * Get the index of a component in the standard order.
 */
export function getComponentIndex(type: ComponentType): number {
	return COMPONENT_ORDER.indexOf(type);
}

/**
 * Check if a component can be started (all prior components at least started).
 */
export function canStartComponent(
	type: ComponentType,
	statuses: ComponentStatusItem[]
): boolean {
	const targetIndex = getComponentIndex(type);
	if (targetIndex === 0) return true;

	const priorComponent = COMPONENT_ORDER[targetIndex - 1];
	const priorStatus = statuses.find((s) => s.component_type === priorComponent);

	return priorStatus?.status !== 'not_started';
}

/**
 * Get the display label for a component type.
 */
export function getComponentLabel(type: ComponentType): string {
	return COMPONENT_LABELS[type];
}

/**
 * Calculate progress percentage from component statuses.
 */
export function calculateProgress(statuses: ComponentStatusItem[]): number {
	const completed = statuses.filter((s) => s.status === 'complete').length;
	return Math.round((completed / statuses.length) * 100);
}
