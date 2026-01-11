/**
 * TypeScript types for Session module
 */

export type SessionStatus = 'active' | 'archived';

export interface Session {
  id: string;
  user_id: string;
  title: string;
  description?: string;
  status: SessionStatus;
  cycle_count: number;
  created_at: string;
  updated_at: string;
}

export interface SessionSummary {
  id: string;
  title: string;
  status: SessionStatus;
  cycle_count: number;
  updated_at: string;
}

export interface SessionList {
  items: SessionSummary[];
  total: number;
  has_more: boolean;
}

export interface CreateSessionRequest {
  title: string;
  description?: string;
}

export interface RenameSessionRequest {
  title: string;
}

export interface UpdateDescriptionRequest {
  description?: string;
}

export interface ListSessionsQuery {
  page?: number;
  per_page?: number;
  status?: SessionStatus;
  include_archived?: boolean;
}

export interface SessionCommandResponse {
  session_id: string;
  message: string;
}
