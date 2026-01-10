/**
 * Cycle reactive stores for Svelte.
 *
 * Provides reactive state management for cycle data using Svelte stores.
 * These stores are used by components to reactively display cycle state.
 */

import { writable, derived, type Readable } from 'svelte/store';
import type { Session } from '@auth/sveltekit';
import type {
	CycleView,
	CycleTreeNode,
	ComponentOutputView,
	ComponentType
} from '../domain/types';
import * as api from './cycle-api';

// ─────────────────────────────────────────────────────────────────────
// Store Types
// ─────────────────────────────────────────────────────────────────────

interface LoadingState<T> {
	loading: boolean;
	error: string | null;
	data: T | null;
}

// ─────────────────────────────────────────────────────────────────────
// Cycle Store
// ─────────────────────────────────────────────────────────────────────

function createCycleStore() {
	const { subscribe, set, update } = writable<LoadingState<CycleView>>({
		loading: false,
		error: null,
		data: null
	});

	return {
		subscribe,

		async load(session: Session | null, cycleId: string) {
			update((s) => ({ ...s, loading: true, error: null }));

			try {
				const data = await api.getCycle(session, cycleId);
				set({ loading: false, error: null, data });
			} catch (e) {
				const message = e instanceof Error ? e.message : 'Failed to load cycle';
				set({ loading: false, error: message, data: null });
			}
		},

		async startComponent(session: Session | null, componentType: ComponentType) {
			const state = { loading: false, error: null, data: null as CycleView | null };
			subscribe((s) => (state.data = s.data))();

			if (!state.data) return;

			try {
				await api.startComponent(session, {
					cycle_id: state.data.id,
					component_type: componentType
				});
				// Reload to get updated state
				await this.load(session, state.data.id);
			} catch (e) {
				const message = e instanceof Error ? e.message : 'Failed to start component';
				update((s) => ({ ...s, error: message }));
			}
		},

		async completeComponent(session: Session | null, componentType: ComponentType) {
			const state = { loading: false, error: null, data: null as CycleView | null };
			subscribe((s) => (state.data = s.data))();

			if (!state.data) return;

			try {
				await api.completeComponent(session, {
					cycle_id: state.data.id,
					component_type: componentType
				});
				await this.load(session, state.data.id);
			} catch (e) {
				const message = e instanceof Error ? e.message : 'Failed to complete component';
				update((s) => ({ ...s, error: message }));
			}
		},

		async navigateTo(session: Session | null, componentType: ComponentType) {
			const state = { loading: false, error: null, data: null as CycleView | null };
			subscribe((s) => (state.data = s.data))();

			if (!state.data) return;

			try {
				await api.navigateToComponent(session, {
					cycle_id: state.data.id,
					component_type: componentType
				});
				await this.load(session, state.data.id);
			} catch (e) {
				const message = e instanceof Error ? e.message : 'Failed to navigate';
				update((s) => ({ ...s, error: message }));
			}
		},

		reset() {
			set({ loading: false, error: null, data: null });
		}
	};
}

export const cycleStore = createCycleStore();

// ─────────────────────────────────────────────────────────────────────
// Cycle Tree Store
// ─────────────────────────────────────────────────────────────────────

function createCycleTreeStore() {
	const { subscribe, set, update } = writable<LoadingState<CycleTreeNode>>({
		loading: false,
		error: null,
		data: null
	});

	return {
		subscribe,

		async load(session: Session | null, sessionId: string) {
			update((s) => ({ ...s, loading: true, error: null }));

			try {
				const data = await api.getCycleTree(session, sessionId);
				set({ loading: false, error: null, data });
			} catch (e) {
				const message = e instanceof Error ? e.message : 'Failed to load cycle tree';
				set({ loading: false, error: message, data: null });
			}
		},

		reset() {
			set({ loading: false, error: null, data: null });
		}
	};
}

export const cycleTreeStore = createCycleTreeStore();

// ─────────────────────────────────────────────────────────────────────
// Component Output Store
// ─────────────────────────────────────────────────────────────────────

function createComponentOutputStore() {
	const { subscribe, set, update } = writable<LoadingState<ComponentOutputView>>({
		loading: false,
		error: null,
		data: null
	});

	return {
		subscribe,

		async load(session: Session | null, cycleId: string, componentType: ComponentType) {
			update((s) => ({ ...s, loading: true, error: null }));

			try {
				const data = await api.getComponentOutput(session, cycleId, componentType);
				set({ loading: false, error: null, data });
			} catch (e) {
				const message = e instanceof Error ? e.message : 'Failed to load component';
				set({ loading: false, error: message, data: null });
			}
		},

		async updateOutput(session: Session | null, output: unknown) {
			const state = { loading: false, error: null, data: null as ComponentOutputView | null };
			subscribe((s) => (state.data = s.data))();

			if (!state.data) return;

			try {
				await api.updateComponentOutput(session, {
					cycle_id: state.data.cycle_id,
					component_type: state.data.component_type,
					output
				});
				// Reload to get updated state
				await this.load(session, state.data.cycle_id, state.data.component_type);
			} catch (e) {
				const message = e instanceof Error ? e.message : 'Failed to update component';
				update((s) => ({ ...s, error: message }));
			}
		},

		reset() {
			set({ loading: false, error: null, data: null });
		}
	};
}

export const componentOutputStore = createComponentOutputStore();

// ─────────────────────────────────────────────────────────────────────
// Derived Stores
// ─────────────────────────────────────────────────────────────────────

/** Whether the cycle is loading. */
export const isLoading: Readable<boolean> = derived(cycleStore, ($cycle) => $cycle.loading);

/** The current cycle's progress percentage. */
export const progressPercent: Readable<number> = derived(
	cycleStore,
	($cycle) => $cycle.data?.progress_percent ?? 0
);

/** The current step in the cycle. */
export const currentStep: Readable<ComponentType | null> = derived(
	cycleStore,
	($cycle) => $cycle.data?.current_step ?? null
);

/** Whether the cycle is complete. */
export const isComplete: Readable<boolean> = derived(
	cycleStore,
	($cycle) => $cycle.data?.is_complete ?? false
);
