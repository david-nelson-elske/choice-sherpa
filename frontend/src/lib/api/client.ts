/**
 * API clients for Choice Sherpa.
 *
 * This module provides two fetch patterns:
 * 1. Session-based auth (authFetch) - for server-side rendering with @auth/sveltekit
 * 2. Token-based auth (apiRequest) - for client-side API calls with localStorage token
 */

import type { Session } from '@auth/sveltekit';

// ============================================================================
// Session-based Authentication (Server-Side)
// ============================================================================

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

// ============================================================================
// Token-based Authentication (Client-Side)
// ============================================================================

/** API configuration */
export interface ApiConfig {
	baseUrl: string;
	timeout?: number;
}

/** API error response for Result pattern */
export interface ApiErrorResponse {
	code: string;
	message: string;
	details?: Record<string, unknown>;
}

/** API result type */
export type ApiResult<T> = { ok: true; data: T } | { ok: false; error: ApiErrorResponse };

/** Default API configuration */
const defaultConfig: ApiConfig = {
	baseUrl: import.meta.env.VITE_API_URL || '/api',
	timeout: 30000,
};

/** Current configuration */
let config: ApiConfig = { ...defaultConfig };

/** Set API configuration */
export function configureApi(newConfig: Partial<ApiConfig>): void {
	config = { ...config, ...newConfig };
}

/** Get current auth token from localStorage */
function getAuthToken(): string | null {
	if (typeof localStorage === 'undefined') return null;
	return localStorage.getItem('auth_token');
}

/** Build headers for API request */
function buildHeaders(contentType?: string): HeadersInit {
	const headers: HeadersInit = {
		Accept: 'application/json',
	};

	if (contentType) {
		headers['Content-Type'] = contentType;
	}

	const token = getAuthToken();
	if (token) {
		headers['Authorization'] = `Bearer ${token}`;
	}

	return headers;
}

/** Parse API error from response */
async function parseApiError(response: Response): Promise<ApiErrorResponse> {
	try {
		const body = await response.json();
		return {
			code: body.code || `HTTP_${response.status}`,
			message: body.message || response.statusText,
			details: body.details,
		};
	} catch {
		return {
			code: `HTTP_${response.status}`,
			message: response.statusText,
		};
	}
}

/** Make API request with Result pattern */
export async function apiRequest<T>(
	method: string,
	path: string,
	body?: unknown
): Promise<ApiResult<T>> {
	const url = `${config.baseUrl}${path}`;

	const controller = new AbortController();
	const timeoutId = setTimeout(() => controller.abort(), config.timeout);

	try {
		const response = await fetch(url, {
			method,
			headers: buildHeaders(body ? 'application/json' : undefined),
			body: body ? JSON.stringify(body) : undefined,
			signal: controller.signal,
		});

		clearTimeout(timeoutId);

		if (!response.ok) {
			const error = await parseApiError(response);
			return { ok: false, error };
		}

		// Handle 204 No Content
		if (response.status === 204) {
			return { ok: true, data: undefined as T };
		}

		const data = await response.json();
		return { ok: true, data: data as T };
	} catch (err) {
		clearTimeout(timeoutId);

		if (err instanceof Error) {
			if (err.name === 'AbortError') {
				return {
					ok: false,
					error: {
						code: 'TIMEOUT',
						message: 'Request timed out',
					},
				};
			}
			return {
				ok: false,
				error: {
					code: 'NETWORK_ERROR',
					message: err.message,
				},
			};
		}

		return {
			ok: false,
			error: {
				code: 'UNKNOWN_ERROR',
				message: 'An unknown error occurred',
			},
		};
	}
}

/** GET request with Result pattern */
export function get<T>(path: string): Promise<ApiResult<T>> {
	return apiRequest<T>('GET', path);
}

/** POST request with Result pattern */
export function post<T>(path: string, body?: unknown): Promise<ApiResult<T>> {
	return apiRequest<T>('POST', path, body);
}

/** PUT request with Result pattern */
export function put<T>(path: string, body?: unknown): Promise<ApiResult<T>> {
	return apiRequest<T>('PUT', path, body);
}

/** DELETE request with Result pattern */
export function del<T>(path: string): Promise<ApiResult<T>> {
	return apiRequest<T>('DELETE', path);
}
