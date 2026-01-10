/**
 * Dashboard store with real-time update handlers.
 *
 * Manages dashboard state and processes WebSocket updates.
 * This store integrates with the useDashboardLive hook via
 * custom browser events.
 */

import { writable, derived, type Readable } from 'svelte/store';
import type {
    DashboardUpdateMessage,
    DashboardUpdateType,
    ComponentCompletedData,
    ConversationMessageData,
    AnalysisScoresData,
    ProgressInfo,
} from '../types/websocket';

// ============================================
// Dashboard Types (placeholder for dashboard module)
// ============================================

/**
 * Overview of a decision session for dashboard display.
 * This is a placeholder - will be replaced by dashboard module types.
 */
export interface DashboardOverview {
    sessionId: string;
    sessionTitle: string;
    sessionDescription: string | null;
    activeCycleId: string | null;
    cycleCount: number;
    progress: ProgressInfo;
    dqScore: number | null;
    lastUpdated: string;
}

/**
 * Conversation message preview.
 */
export interface ConversationMessage {
    id: string;
    role: 'user' | 'assistant';
    contentPreview: string;
    timestamp: string;
}

// ============================================
// Store State
// ============================================

/**
 * Dashboard store state.
 */
interface DashboardStoreState {
    overview: DashboardOverview | null;
    conversationMessages: ConversationMessage[];
    loading: boolean;
    error: string | null;
    lastWebSocketUpdate: DashboardUpdateMessage | null;
}

/**
 * Initial store state.
 */
const initialState: DashboardStoreState = {
    overview: null,
    conversationMessages: [],
    loading: false,
    error: null,
    lastWebSocketUpdate: null,
};

// ============================================
// Update Handlers
// ============================================

/**
 * Handler function type for dashboard updates.
 */
type UpdateHandler = (state: DashboardStoreState, data: unknown) => DashboardStoreState;

/**
 * Registry of update handlers by update type.
 */
const updateHandlers: Partial<Record<DashboardUpdateType, UpdateHandler>> = {
    session_metadata: (state, data) => {
        const payload = data as { title?: string; description?: string };
        if (!state.overview) return state;

        return {
            ...state,
            overview: {
                ...state.overview,
                sessionTitle: payload.title ?? state.overview.sessionTitle,
                sessionDescription: payload.description ?? state.overview.sessionDescription,
                lastUpdated: new Date().toISOString(),
            },
        };
    },

    cycle_created: (state, data) => {
        const payload = data as { cycle_id: string; session_id: string };
        if (!state.overview) return state;

        return {
            ...state,
            overview: {
                ...state.overview,
                cycleCount: state.overview.cycleCount + 1,
                activeCycleId: payload.cycle_id,
                lastUpdated: new Date().toISOString(),
            },
        };
    },

    cycle_progress: (state, data) => {
        const payload = data as { cycle_id: string; progress: ProgressInfo };
        if (!state.overview) return state;

        return {
            ...state,
            overview: {
                ...state.overview,
                progress: payload.progress,
                lastUpdated: new Date().toISOString(),
            },
        };
    },

    component_started: (state, _data) => {
        // Component started - might update progress in future
        return {
            ...state,
            overview: state.overview
                ? { ...state.overview, lastUpdated: new Date().toISOString() }
                : null,
        };
    },

    component_completed: (state, data) => {
        const payload = data as ComponentCompletedData;
        if (!state.overview) return state;

        return {
            ...state,
            overview: {
                ...state.overview,
                progress: payload.progress,
                lastUpdated: new Date().toISOString(),
            },
        };
    },

    component_output: (state, _data) => {
        // Component output updated - could trigger refresh
        return {
            ...state,
            overview: state.overview
                ? { ...state.overview, lastUpdated: new Date().toISOString() }
                : null,
        };
    },

    conversation_message: (state, data) => {
        const payload = data as ConversationMessageData;
        const newMessage: ConversationMessage = {
            id: payload.message.id,
            role: payload.message.role,
            contentPreview: payload.message.contentPreview,
            timestamp: payload.message.timestamp,
        };

        return {
            ...state,
            conversationMessages: [...state.conversationMessages, newMessage],
        };
    },

    analysis_scores: (state, data) => {
        const payload = data as AnalysisScoresData;
        if (!state.overview) return state;

        // Only update DQ score if this is a DQ score update
        const dqScore =
            payload.scoreType === 'dq' ? payload.overallScore ?? null : state.overview.dqScore;

        return {
            ...state,
            overview: {
                ...state.overview,
                dqScore,
                lastUpdated: new Date().toISOString(),
            },
        };
    },

    cycle_completed: (state, data) => {
        const payload = data as { cycle_id: string; dq_score?: number };
        if (!state.overview) return state;

        return {
            ...state,
            overview: {
                ...state.overview,
                dqScore: payload.dq_score ?? state.overview.dqScore,
                progress: { completed: 9, total: 9, percent: 100 },
                lastUpdated: new Date().toISOString(),
            },
        };
    },
};

// ============================================
// Store Creation
// ============================================

/**
 * Create the dashboard store.
 */
function createDashboardStore() {
    const { subscribe, set, update } = writable<DashboardStoreState>(initialState);

    return {
        subscribe,

        /**
         * Process a WebSocket dashboard update.
         */
        processUpdate(updateMessage: DashboardUpdateMessage): void {
            update((state) => {
                const handler = updateHandlers[updateMessage.updateType];
                const newState = handler
                    ? handler(state, updateMessage.data)
                    : state;

                return {
                    ...newState,
                    lastWebSocketUpdate: updateMessage,
                };
            });
        },

        /**
         * Set the initial dashboard overview.
         */
        setOverview(overview: DashboardOverview): void {
            update((state) => ({
                ...state,
                overview,
                loading: false,
                error: null,
            }));
        },

        /**
         * Set loading state.
         */
        setLoading(loading: boolean): void {
            update((state) => ({ ...state, loading }));
        },

        /**
         * Set error state.
         */
        setError(error: string | null): void {
            update((state) => ({ ...state, error, loading: false }));
        },

        /**
         * Clear conversation messages.
         */
        clearConversation(): void {
            update((state) => ({ ...state, conversationMessages: [] }));
        },

        /**
         * Reset store to initial state.
         */
        reset(): void {
            set(initialState);
        },
    };
}

/**
 * Dashboard store singleton instance.
 */
export const dashboardStore = createDashboardStore();

// ============================================
// Derived Stores
// ============================================

/**
 * Current dashboard overview (or null).
 */
export const dashboardOverview: Readable<DashboardOverview | null> = derived(
    dashboardStore,
    ($store) => $store.overview
);

/**
 * Current progress information.
 */
export const currentProgress: Readable<ProgressInfo> = derived(
    dashboardStore,
    ($store) =>
        $store.overview?.progress ?? { completed: 0, total: 9, percent: 0 }
);

/**
 * Whether a recommendation exists (cycle completed).
 */
export const hasRecommendation: Readable<boolean> = derived(
    dashboardStore,
    ($store) => ($store.overview?.progress.percent ?? 0) === 100
);

/**
 * Current DQ score.
 */
export const dqScore: Readable<number | null> = derived(
    dashboardStore,
    ($store) => $store.overview?.dqScore ?? null
);

/**
 * Recent conversation messages.
 */
export const conversationMessages: Readable<ConversationMessage[]> = derived(
    dashboardStore,
    ($store) => $store.conversationMessages
);

/**
 * Whether the dashboard is loading.
 */
export const isDashboardLoading: Readable<boolean> = derived(
    dashboardStore,
    ($store) => $store.loading
);

/**
 * Current error message.
 */
export const dashboardError: Readable<string | null> = derived(
    dashboardStore,
    ($store) => $store.error
);

// ============================================
// Browser Event Integration
// ============================================

/**
 * Custom event name for dashboard updates.
 */
export const DASHBOARD_UPDATE_EVENT = 'dashboard:update';

/**
 * Initialize dashboard store event listener.
 *
 * This connects the store to WebSocket updates dispatched
 * by the useDashboardLive hook.
 *
 * Call this once in a layout or root component:
 * ```typescript
 * import { initDashboardEventListener } from '$lib/stores/dashboard';
 * onMount(() => initDashboardEventListener());
 * ```
 */
export function initDashboardEventListener(): () => void {
    if (typeof window === 'undefined') {
        return () => {};
    }

    const handler = (event: Event) => {
        const customEvent = event as CustomEvent<DashboardUpdateMessage>;
        dashboardStore.processUpdate(customEvent.detail);
    };

    window.addEventListener(DASHBOARD_UPDATE_EVENT, handler);

    return () => {
        window.removeEventListener(DASHBOARD_UPDATE_EVENT, handler);
    };
}

/**
 * Dispatch a dashboard update event.
 *
 * Used by useDashboardLive hook to notify the store.
 */
export function dispatchDashboardUpdate(update: DashboardUpdateMessage): void {
    if (typeof window === 'undefined') return;

    window.dispatchEvent(
        new CustomEvent(DASHBOARD_UPDATE_EVENT, { detail: update })
    );
}
