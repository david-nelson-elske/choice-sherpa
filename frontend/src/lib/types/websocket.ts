/**
 * WebSocket types for real-time dashboard updates.
 *
 * These types match the backend message protocol.
 */

// ============================================
// Base Message Types
// ============================================

/**
 * Base interface for all server messages.
 */
export interface WebSocketMessage {
    type: string;
    timestamp?: string;
    correlationId?: string;
}

// ============================================
// Server → Client Messages
// ============================================

/**
 * Connection established message.
 */
export interface ConnectedMessage extends WebSocketMessage {
    type: 'connected';
    sessionId: string;
    clientId: string;
}

/**
 * Dashboard update notification.
 */
export interface DashboardUpdateMessage extends WebSocketMessage {
    type: 'dashboard.update';
    updateType: DashboardUpdateType;
    data: unknown;
}

/**
 * Types of dashboard updates.
 */
export type DashboardUpdateType =
    | 'session_metadata'    // Session title/description changed
    | 'cycle_created'       // New cycle added
    | 'cycle_progress'      // Cycle progress changed
    | 'component_started'   // Component work began
    | 'component_completed' // Component finished
    | 'component_output'    // Component output updated
    | 'conversation_message' // New chat message
    | 'analysis_scores'     // Pugh/DQ scores computed
    | 'cycle_completed';    // Cycle finished

/**
 * Error message from server.
 */
export interface ErrorMessage extends WebSocketMessage {
    type: 'error';
    code: string;
    message: string;
}

/**
 * Heartbeat response.
 */
export interface PongMessage extends WebSocketMessage {
    type: 'pong';
}

/**
 * Union type of all server messages.
 */
export type ServerMessage =
    | ConnectedMessage
    | DashboardUpdateMessage
    | ErrorMessage
    | PongMessage;

// ============================================
// Client → Server Messages
// ============================================

/**
 * Client ping (keepalive).
 */
export interface ClientPing {
    type: 'ping';
}

/**
 * Request full state (after reconnection).
 */
export interface RequestStateMessage {
    type: 'request.state';
}

/**
 * Union type of all client messages.
 */
export type ClientMessage = ClientPing | RequestStateMessage;

// ============================================
// Dashboard Update Payloads
// ============================================

/**
 * Payload for component completion updates.
 */
export interface ComponentCompletedData {
    cycleId: string;
    componentType: string;
    completedAt: string;
    progress: ProgressInfo;
}

/**
 * Progress information for a cycle.
 */
export interface ProgressInfo {
    completed: number;
    total: number;
    percent: number;
}

/**
 * Payload for new conversation message updates.
 */
export interface ConversationMessageData {
    cycleId: string;
    componentType: string;
    message: MessagePreview;
}

/**
 * Preview of a message (truncated for safety).
 */
export interface MessagePreview {
    id: string;
    role: 'user' | 'assistant';
    contentPreview: string;
    timestamp: string;
}

/**
 * Payload for analysis score updates.
 */
export interface AnalysisScoresData {
    cycleId: string;
    scoreType: 'pugh' | 'dq';
    scores: Record<string, number>;
    overallScore?: number;
}

// ============================================
// Connection State
// ============================================

/**
 * WebSocket connection state.
 */
export interface WebSocketConnectionState {
    /** Whether connected to the server */
    connected: boolean;
    /** Client ID assigned by server */
    clientId: string | null;
    /** Most recent update received */
    lastUpdate: DashboardUpdateMessage | null;
    /** Current error (if any) */
    error: Error | null;
    /** Number of reconnection attempts */
    reconnectAttempts: number;
}

/**
 * Options for the dashboard live hook.
 */
export interface UseDashboardLiveOptions {
    /** Session ID to connect to */
    sessionId: string;
    /** Callback for updates */
    onUpdate?: (update: DashboardUpdateMessage) => void;
    /** Callback for connection state changes */
    onConnectionChange?: (connected: boolean) => void;
    /** Reconnection interval in ms (default: 3000) */
    reconnectInterval?: number;
    /** Max reconnection attempts (default: 10) */
    maxReconnectAttempts?: number;
}

/**
 * Initial connection state.
 */
export const INITIAL_CONNECTION_STATE: WebSocketConnectionState = {
    connected: false,
    clientId: null,
    lastUpdate: null,
    error: null,
    reconnectAttempts: 0,
};
