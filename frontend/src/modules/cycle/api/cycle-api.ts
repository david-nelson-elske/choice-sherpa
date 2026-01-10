/**
 * Cycle API client.
 *
 * Provides typed methods for interacting with the cycle REST API.
 * Uses the authenticated API client from $lib/api.
 */

import type { Session } from '@auth/sveltekit';
import type {
	CycleView,
	CycleSummary,
	CycleTreeNode,
	ComponentOutputView,
	ComponentType
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
// Request/Response Types
// ─────────────────────────────────────────────────────────────────────

export interface CreateCycleRequest {
	session_id: string;
}

export interface CreateCycleResponse {
	cycle_id: string;
}

export interface BranchCycleRequest {
	parent_cycle_id: string;
	branch_point: ComponentType;
}

export interface BranchCycleResponse {
	cycle_id: string;
}

export interface StartComponentRequest {
	cycle_id: string;
	component_type: ComponentType;
}

export interface CompleteComponentRequest {
	cycle_id: string;
	component_type: ComponentType;
}

export interface UpdateComponentOutputRequest {
	cycle_id: string;
	component_type: ComponentType;
	output: unknown;
}

export interface NavigateToComponentRequest {
	cycle_id: string;
	component_type: ComponentType;
}

// ─────────────────────────────────────────────────────────────────────
// API Functions
// ─────────────────────────────────────────────────────────────────────

const API_BASE = '/api/cycles';

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
// Command Operations (mutations)
// ─────────────────────────────────────────────────────────────────────

/**
 * Create a new cycle for a session.
 */
export async function createCycle(
	session: Session | null,
	request: CreateCycleRequest
): Promise<CreateCycleResponse> {
	const response = await authFetch(API_BASE, session, {
		method: 'POST',
		body: JSON.stringify(request)
	});
	return handleResponse(response);
}

/**
 * Branch from an existing cycle at a specific component.
 */
export async function branchCycle(
	session: Session | null,
	request: BranchCycleRequest
): Promise<BranchCycleResponse> {
	const response = await authFetch(`${API_BASE}/${request.parent_cycle_id}/branch`, session, {
		method: 'POST',
		body: JSON.stringify({ branch_point: request.branch_point })
	});
	return handleResponse(response);
}

/**
 * Start a component within a cycle.
 */
export async function startComponent(
	session: Session | null,
	request: StartComponentRequest
): Promise<void> {
	const response = await authFetch(
		`${API_BASE}/${request.cycle_id}/components/${request.component_type}/start`,
		session,
		{ method: 'POST' }
	);
	return handleResponse(response);
}

/**
 * Complete a component within a cycle.
 */
export async function completeComponent(
	session: Session | null,
	request: CompleteComponentRequest
): Promise<void> {
	const response = await authFetch(
		`${API_BASE}/${request.cycle_id}/components/${request.component_type}/complete`,
		session,
		{ method: 'POST' }
	);
	return handleResponse(response);
}

/**
 * Update a component's output.
 */
export async function updateComponentOutput(
	session: Session | null,
	request: UpdateComponentOutputRequest
): Promise<void> {
	const response = await authFetch(
		`${API_BASE}/${request.cycle_id}/components/${request.component_type}`,
		session,
		{
			method: 'PUT',
			body: JSON.stringify({ output: request.output })
		}
	);
	return handleResponse(response);
}

/**
 * Navigate to a specific component.
 */
export async function navigateToComponent(
	session: Session | null,
	request: NavigateToComponentRequest
): Promise<void> {
	const response = await authFetch(
		`${API_BASE}/${request.cycle_id}/navigate`,
		session,
		{
			method: 'POST',
			body: JSON.stringify({ component_type: request.component_type })
		}
	);
	return handleResponse(response);
}

/**
 * Complete a cycle.
 */
export async function completeCycle(
	session: Session | null,
	cycleId: string
): Promise<void> {
	const response = await authFetch(`${API_BASE}/${cycleId}/complete`, session, {
		method: 'POST'
	});
	return handleResponse(response);
}

/**
 * Archive a cycle.
 */
export async function archiveCycle(
	session: Session | null,
	cycleId: string
): Promise<void> {
	const response = await authFetch(`${API_BASE}/${cycleId}/archive`, session, {
		method: 'POST'
	});
	return handleResponse(response);
}

// ─────────────────────────────────────────────────────────────────────
// Query Operations (reads)
// ─────────────────────────────────────────────────────────────────────

/**
 * Get a cycle by ID.
 */
export async function getCycle(
	session: Session | null,
	cycleId: string
): Promise<CycleView> {
	const response = await authFetch(`${API_BASE}/${cycleId}`, session, {
		method: 'GET'
	});
	return handleResponse(response);
}

/**
 * List cycles for a session.
 */
export async function listCycles(
	session: Session | null,
	sessionId: string
): Promise<CycleSummary[]> {
	const response = await authFetch(`${API_BASE}?session_id=${sessionId}`, session, {
		method: 'GET'
	});
	return handleResponse(response);
}

/**
 * Get the cycle tree for a session.
 */
export async function getCycleTree(
	session: Session | null,
	sessionId: string
): Promise<CycleTreeNode | null> {
	const response = await authFetch(`${API_BASE}/tree?session_id=${sessionId}`, session, {
		method: 'GET'
	});
	return handleResponse(response);
}

/**
 * Get a component's output.
 */
export async function getComponentOutput(
	session: Session | null,
	cycleId: string,
	componentType: ComponentType
): Promise<ComponentOutputView> {
	const response = await authFetch(
		`${API_BASE}/${cycleId}/components/${componentType}`,
		session,
		{ method: 'GET' }
	);
	return handleResponse(response);
}

/**
 * Get a cycle's lineage (ancestors).
 */
export async function getCycleLineage(
	session: Session | null,
	cycleId: string
): Promise<CycleSummary[]> {
	const response = await authFetch(`${API_BASE}/${cycleId}/lineage`, session, {
		method: 'GET'
	});
	return handleResponse(response);
}
