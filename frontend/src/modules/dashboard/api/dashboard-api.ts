/**
 * Dashboard API client.
 *
 * Provides typed methods for interacting with the dashboard REST API.
 * Uses the authenticated API client for all requests.
 */

import type { Session } from '@auth/sveltekit';
import type { ComponentType } from '../../cycle/domain/types';
import type {
	DashboardOverview,
	ComponentDetailView,
	CycleComparison
} from '../domain/types';

// ─────────────────────────────────────────────────────────────────────
// API Error Types
// ─────────────────────────────────────────────────────────────────────

export class ApiError extends Error {
	constructor(
		message: string,
		public status: number,
		public body?: unknown
	) {
		super(message);
		this.name = 'ApiError';
	}
}

// ─────────────────────────────────────────────────────────────────────
// Helper Functions
// ─────────────────────────────────────────────────────────────────────

async function authFetch(
	url: string,
	session: Session | null,
	options: RequestInit = {}
): Promise<Response> {
	if (!session?.accessToken) {
		throw new Error('Not authenticated');
	}

	return fetch(url, {
		...options,
		headers: {
			'Content-Type': 'application/json',
			Authorization: `Bearer ${session.accessToken}`,
			...options.headers
		}
	});
}

async function handleResponse<T>(response: Response): Promise<T> {
	if (!response.ok) {
		const body = await response.json().catch(() => null);
		throw new ApiError(
			body?.error || `Request failed: ${response.status}`,
			response.status,
			body
		);
	}

	if (response.status === 204) {
		return undefined as T;
	}

	return response.json();
}

// ─────────────────────────────────────────────────────────────────────
// Dashboard Queries
// ─────────────────────────────────────────────────────────────────────

/**
 * Get dashboard overview for a session.
 * @param session - Auth session
 * @param sessionId - Session ID
 * @param cycleId - Optional specific cycle ID (defaults to active cycle)
 */
export async function getDashboardOverview(
	session: Session | null,
	sessionId: string,
	cycleId?: string
): Promise<DashboardOverview> {
	const url = cycleId
		? `/api/sessions/${sessionId}/dashboard?cycle_id=${cycleId}`
		: `/api/sessions/${sessionId}/dashboard`;

	const response = await authFetch(url, session, {
		method: 'GET'
	});
	return handleResponse(response);
}

/**
 * Get detailed view of a specific component.
 * @param session - Auth session
 * @param cycleId - Cycle ID
 * @param componentType - Component type (e.g., 'objectives', 'alternatives')
 */
export async function getComponentDetail(
	session: Session | null,
	cycleId: string,
	componentType: ComponentType
): Promise<ComponentDetailView> {
	const response = await authFetch(
		`/api/cycles/${cycleId}/components/${componentType}/detail`,
		session,
		{ method: 'GET' }
	);
	return handleResponse(response);
}

/**
 * Compare multiple cycles side-by-side.
 * @param session - Auth session
 * @param sessionId - Session ID
 * @param cycleIds - Array of cycle IDs to compare (min 2)
 */
export async function compareCycles(
	session: Session | null,
	sessionId: string,
	cycleIds: string[]
): Promise<CycleComparison> {
	if (cycleIds.length < 2) {
		throw new Error('At least 2 cycle IDs required for comparison');
	}

	const cyclesParam = cycleIds.join(',');
	const response = await authFetch(
		`/api/sessions/${sessionId}/compare?cycles=${cyclesParam}`,
		session,
		{ method: 'GET' }
	);
	return handleResponse(response);
}
