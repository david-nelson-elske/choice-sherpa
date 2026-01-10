/**
 * Authenticated API client for backend communication.
 *
 * This module provides fetch wrappers that automatically attach
 * the Bearer token from the user's session to API requests.
 *
 * @example
 * ```typescript
 * import { authFetch } from '$lib/api/client';
 *
 * // In a load function or component
 * const response = await authFetch('/api/sessions', { session });
 * const sessions = await response.json();
 * ```
 */

import type { Session } from '@auth/sveltekit';

/** Base URL for API requests (empty for same-origin) */
const API_BASE = '';

/**
 * Custom error for API authentication failures.
 */
export class AuthenticationError extends Error {
	constructor(message: string = 'Not authenticated') {
		super(message);
		this.name = 'AuthenticationError';
	}
}

/**
 * Custom error for API request failures.
 */
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

/**
 * Options for authenticated fetch requests.
 */
export interface AuthFetchOptions extends Omit<RequestInit, 'headers'> {
	/** The user's session containing the access token */
	session: Session | null;
	/** Additional headers to include */
	headers?: Record<string, string>;
	/** Skip authentication requirement (allows unauthenticated requests) */
	allowUnauthenticated?: boolean;
}

/**
 * Perform an authenticated fetch request.
 *
 * Automatically attaches the Bearer token from the session to the
 * Authorization header. Throws if not authenticated unless
 * `allowUnauthenticated` is true.
 *
 * @param url - The URL to fetch (relative to API_BASE)
 * @param options - Fetch options including session
 * @returns The fetch Response
 * @throws {AuthenticationError} If not authenticated and auth is required
 *
 * @example
 * ```typescript
 * // In a +page.server.ts load function
 * const response = await authFetch('/api/sessions', {
 *   session: await event.locals.auth(),
 *   method: 'GET'
 * });
 * ```
 */
export async function authFetch(
	url: string,
	options: AuthFetchOptions
): Promise<Response> {
	const { session, headers = {}, allowUnauthenticated = false, ...fetchOptions } = options;

	// Check authentication
	if (!session?.accessToken && !allowUnauthenticated) {
		throw new AuthenticationError();
	}

	// Build headers
	const requestHeaders: Record<string, string> = {
		'Content-Type': 'application/json',
		...headers
	};

	// Add Authorization header if we have a token
	if (session?.accessToken) {
		requestHeaders['Authorization'] = `Bearer ${session.accessToken}`;
	}

	// Perform the request
	const response = await fetch(`${API_BASE}${url}`, {
		...fetchOptions,
		headers: requestHeaders
	});

	return response;
}

/**
 * Perform an authenticated GET request.
 *
 * @param url - The URL to fetch
 * @param session - The user's session
 * @returns The parsed JSON response
 * @throws {AuthenticationError} If not authenticated
 * @throws {ApiError} If the request fails
 */
export async function authGet<T>(url: string, session: Session | null): Promise<T> {
	const response = await authFetch(url, { session, method: 'GET' });

	if (!response.ok) {
		const body = await response.json().catch(() => null);
		throw new ApiError(
			body?.error || `Request failed: ${response.status}`,
			response.status,
			body
		);
	}

	return response.json();
}

/**
 * Perform an authenticated POST request.
 *
 * @param url - The URL to fetch
 * @param session - The user's session
 * @param data - The request body (will be JSON serialized)
 * @returns The parsed JSON response
 * @throws {AuthenticationError} If not authenticated
 * @throws {ApiError} If the request fails
 */
export async function authPost<T>(
	url: string,
	session: Session | null,
	data: unknown
): Promise<T> {
	const response = await authFetch(url, {
		session,
		method: 'POST',
		body: JSON.stringify(data)
	});

	if (!response.ok) {
		const body = await response.json().catch(() => null);
		throw new ApiError(
			body?.error || `Request failed: ${response.status}`,
			response.status,
			body
		);
	}

	return response.json();
}

/**
 * Perform an authenticated PUT request.
 *
 * @param url - The URL to fetch
 * @param session - The user's session
 * @param data - The request body (will be JSON serialized)
 * @returns The parsed JSON response
 */
export async function authPut<T>(
	url: string,
	session: Session | null,
	data: unknown
): Promise<T> {
	const response = await authFetch(url, {
		session,
		method: 'PUT',
		body: JSON.stringify(data)
	});

	if (!response.ok) {
		const body = await response.json().catch(() => null);
		throw new ApiError(
			body?.error || `Request failed: ${response.status}`,
			response.status,
			body
		);
	}

	return response.json();
}

/**
 * Perform an authenticated DELETE request.
 *
 * @param url - The URL to fetch
 * @param session - The user's session
 * @returns The parsed JSON response (if any)
 */
export async function authDelete<T = void>(
	url: string,
	session: Session | null
): Promise<T> {
	const response = await authFetch(url, {
		session,
		method: 'DELETE'
	});

	if (!response.ok) {
		const body = await response.json().catch(() => null);
		throw new ApiError(
			body?.error || `Request failed: ${response.status}`,
			response.status,
			body
		);
	}

	// Some DELETE requests return no content
	if (response.status === 204) {
		return undefined as T;
	}

	return response.json();
}
