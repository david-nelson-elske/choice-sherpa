/**
 * API client for Session operations
 */

import type {
  Session,
  SessionList,
  CreateSessionRequest,
  RenameSessionRequest,
  ListSessionsQuery,
  SessionCommandResponse,
} from '../types';

const API_BASE = '/api/sessions';

export class SessionApiError extends Error {
  constructor(
    message: string,
    public code?: string,
    public status?: number
  ) {
    super(message);
    this.name = 'SessionApiError';
  }
}

/**
 * Create a new session
 */
export async function createSession(
  request: CreateSessionRequest
): Promise<SessionCommandResponse> {
  const response = await fetch(API_BASE, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    credentials: 'include',
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: 'Failed to create session' }));
    throw new SessionApiError(
      error.message || 'Failed to create session',
      error.code,
      response.status
    );
  }

  return response.json();
}

/**
 * Get session by ID
 */
export async function getSession(sessionId: string): Promise<Session> {
  const response = await fetch(`${API_BASE}/${sessionId}`, {
    method: 'GET',
    credentials: 'include',
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: 'Failed to fetch session' }));
    throw new SessionApiError(
      error.message || 'Failed to fetch session',
      error.code,
      response.status
    );
  }

  return response.json();
}

/**
 * List sessions for the current user
 */
export async function listSessions(query?: ListSessionsQuery): Promise<SessionList> {
  const params = new URLSearchParams();

  if (query?.page) params.append('page', query.page.toString());
  if (query?.per_page) params.append('per_page', query.per_page.toString());
  if (query?.status) params.append('status', query.status);
  if (query?.include_archived) params.append('include_archived', 'true');

  const url = params.toString() ? `${API_BASE}?${params}` : API_BASE;

  const response = await fetch(url, {
    method: 'GET',
    credentials: 'include',
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: 'Failed to list sessions' }));
    throw new SessionApiError(
      error.message || 'Failed to list sessions',
      error.code,
      response.status
    );
  }

  return response.json();
}

/**
 * Rename a session
 */
export async function renameSession(
  sessionId: string,
  request: RenameSessionRequest
): Promise<SessionCommandResponse> {
  const response = await fetch(`${API_BASE}/${sessionId}/rename`, {
    method: 'PATCH',
    headers: {
      'Content-Type': 'application/json',
    },
    credentials: 'include',
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: 'Failed to rename session' }));
    throw new SessionApiError(
      error.message || 'Failed to rename session',
      error.code,
      response.status
    );
  }

  return response.json();
}

/**
 * Archive a session
 */
export async function archiveSession(sessionId: string): Promise<SessionCommandResponse> {
  const response = await fetch(`${API_BASE}/${sessionId}/archive`, {
    method: 'POST',
    credentials: 'include',
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: 'Failed to archive session' }));
    throw new SessionApiError(
      error.message || 'Failed to archive session',
      error.code,
      response.status
    );
  }

  return response.json();
}
