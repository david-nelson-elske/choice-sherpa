/**
 * Cycle domain type tests.
 */

import { describe, it, expect } from 'vitest';
import {
	COMPONENT_ORDER,
	COMPONENT_LABELS,
	getComponentIndex,
	canStartComponent,
	getComponentLabel,
	calculateProgress,
	type ComponentType,
	type ComponentStatusItem
} from './types';

describe('COMPONENT_ORDER', () => {
	it('has 9 components', () => {
		expect(COMPONENT_ORDER).toHaveLength(9);
	});

	it('starts with issue_raising', () => {
		expect(COMPONENT_ORDER[0]).toBe('issue_raising');
	});

	it('ends with notes_next_steps', () => {
		expect(COMPONENT_ORDER[8]).toBe('notes_next_steps');
	});
});

describe('COMPONENT_LABELS', () => {
	it('has labels for all components', () => {
		for (const type of COMPONENT_ORDER) {
			expect(COMPONENT_LABELS[type]).toBeDefined();
			expect(typeof COMPONENT_LABELS[type]).toBe('string');
		}
	});

	it('has human-readable labels', () => {
		expect(COMPONENT_LABELS.issue_raising).toBe('Issue Raising');
		expect(COMPONENT_LABELS.problem_frame).toBe('Problem Frame');
		expect(COMPONENT_LABELS.decision_quality).toBe('Decision Quality');
	});
});

describe('getComponentIndex', () => {
	it('returns 0 for first component', () => {
		expect(getComponentIndex('issue_raising')).toBe(0);
	});

	it('returns correct index for middle component', () => {
		expect(getComponentIndex('alternatives')).toBe(3);
	});

	it('returns 8 for last component', () => {
		expect(getComponentIndex('notes_next_steps')).toBe(8);
	});
});

describe('canStartComponent', () => {
	const allNotStarted: ComponentStatusItem[] = COMPONENT_ORDER.map((type) => ({
		component_type: type,
		status: 'not_started',
		is_current: type === 'issue_raising'
	}));

	const issueRaisingStarted: ComponentStatusItem[] = COMPONENT_ORDER.map((type) => ({
		component_type: type,
		status: type === 'issue_raising' ? 'in_progress' : 'not_started',
		is_current: type === 'issue_raising'
	}));

	const issueRaisingComplete: ComponentStatusItem[] = COMPONENT_ORDER.map((type) => ({
		component_type: type,
		status: type === 'issue_raising' ? 'complete' : 'not_started',
		is_current: type === 'problem_frame'
	}));

	it('allows starting first component when all not started', () => {
		expect(canStartComponent('issue_raising', allNotStarted)).toBe(true);
	});

	it('prevents starting second component when first not started', () => {
		expect(canStartComponent('problem_frame', allNotStarted)).toBe(false);
	});

	it('allows starting second component when first is in progress', () => {
		expect(canStartComponent('problem_frame', issueRaisingStarted)).toBe(true);
	});

	it('allows starting second component when first is complete', () => {
		expect(canStartComponent('problem_frame', issueRaisingComplete)).toBe(true);
	});

	it('prevents skipping components', () => {
		expect(canStartComponent('alternatives', issueRaisingComplete)).toBe(false);
	});
});

describe('getComponentLabel', () => {
	it('returns correct label for component type', () => {
		expect(getComponentLabel('issue_raising')).toBe('Issue Raising');
		expect(getComponentLabel('tradeoffs')).toBe('Tradeoffs');
	});
});

describe('calculateProgress', () => {
	const allNotStarted: ComponentStatusItem[] = COMPONENT_ORDER.map((type) => ({
		component_type: type,
		status: 'not_started',
		is_current: type === 'issue_raising'
	}));

	const threeComplete: ComponentStatusItem[] = COMPONENT_ORDER.map((type, i) => ({
		component_type: type,
		status: i < 3 ? 'complete' : 'not_started',
		is_current: i === 3
	}));

	const allComplete: ComponentStatusItem[] = COMPONENT_ORDER.map((type) => ({
		component_type: type,
		status: 'complete',
		is_current: false
	}));

	it('returns 0 when no components complete', () => {
		expect(calculateProgress(allNotStarted)).toBe(0);
	});

	it('returns correct percentage for partial completion', () => {
		expect(calculateProgress(threeComplete)).toBe(33); // 3/9 = 33%
	});

	it('returns 100 when all components complete', () => {
		expect(calculateProgress(allComplete)).toBe(100);
	});
});
