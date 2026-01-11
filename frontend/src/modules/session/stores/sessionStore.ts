/**
 * Svelte store for session state management
 */

import { writable, derived } from 'svelte/store';
import type { Session, SessionList, SessionSummary } from '../types';

interface SessionState {
  currentSession: Session | null;
  sessions: SessionSummary[];
  total: number;
  hasMore: boolean;
  loading: boolean;
  error: string | null;
}

const initialState: SessionState = {
  currentSession: null,
  sessions: [],
  total: 0,
  hasMore: false,
  loading: false,
  error: null,
};

function createSessionStore() {
  const { subscribe, set, update } = writable<SessionState>(initialState);

  return {
    subscribe,

    setCurrentSession: (session: Session | null) => {
      update((state) => ({ ...state, currentSession: session, error: null }));
    },

    setSessionList: (list: SessionList) => {
      update((state) => ({
        ...state,
        sessions: list.items,
        total: list.total,
        hasMore: list.has_more,
        error: null,
      }));
    },

    appendSessions: (list: SessionList) => {
      update((state) => ({
        ...state,
        sessions: [...state.sessions, ...list.items],
        total: list.total,
        hasMore: list.has_more,
        error: null,
      }));
    },

    addSession: (session: SessionSummary) => {
      update((state) => ({
        ...state,
        sessions: [session, ...state.sessions],
        total: state.total + 1,
      }));
    },

    updateSession: (sessionId: string, updates: Partial<SessionSummary>) => {
      update((state) => ({
        ...state,
        sessions: state.sessions.map((s) =>
          s.id === sessionId ? { ...s, ...updates } : s
        ),
      }));
    },

    removeSession: (sessionId: string) => {
      update((state) => ({
        ...state,
        sessions: state.sessions.filter((s) => s.id !== sessionId),
        total: Math.max(0, state.total - 1),
      }));
    },

    setLoading: (loading: boolean) => {
      update((state) => ({ ...state, loading }));
    },

    setError: (error: string | null) => {
      update((state) => ({ ...state, error, loading: false }));
    },

    reset: () => {
      set(initialState);
    },
  };
}

export const sessionStore = createSessionStore();

// Derived stores for convenience
export const currentSession = derived(sessionStore, ($store) => $store.currentSession);
export const sessions = derived(sessionStore, ($store) => $store.sessions);
export const sessionsLoading = derived(sessionStore, ($store) => $store.loading);
export const sessionsError = derived(sessionStore, ($store) => $store.error);
