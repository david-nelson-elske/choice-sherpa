/**
 * Dashboard live updates hook.
 *
 * Manages WebSocket connection for real-time dashboard updates.
 * Handles connection lifecycle, reconnection, and heartbeat.
 */

import { writable, derived, type Readable, type Writable } from 'svelte/store';
import type {
    WebSocketConnectionState,
    UseDashboardLiveOptions,
    ServerMessage,
    ClientMessage,
    DashboardUpdateMessage,
    ConnectedMessage,
} from '../types/websocket';
import { dispatchDashboardUpdate } from '../stores/dashboard';

/** Default reconnection interval in milliseconds */
const DEFAULT_RECONNECT_INTERVAL = 3000;

/** Default maximum reconnection attempts */
const DEFAULT_MAX_RECONNECT_ATTEMPTS = 10;

/** Heartbeat interval in milliseconds */
const HEARTBEAT_INTERVAL = 30000;

/** Connection timeout in milliseconds */
const CONNECTION_TIMEOUT = 10000;

/**
 * Result of the useDashboardLive hook.
 */
export interface DashboardLiveResult {
    /** Connection state store */
    state: Readable<WebSocketConnectionState>;
    /** Whether currently connected */
    connected: Readable<boolean>;
    /** Current error if any */
    error: Readable<Error | null>;
    /** Manually connect */
    connect: () => void;
    /** Manually disconnect */
    disconnect: () => void;
    /** Request full state (after reconnection) */
    requestState: () => void;
}

/**
 * Creates initial connection state.
 */
function createInitialState(): WebSocketConnectionState {
    return {
        connected: false,
        clientId: null,
        lastUpdate: null,
        error: null,
        reconnectAttempts: 0,
    };
}

/**
 * Create a dashboard live connection hook.
 *
 * This is a Svelte-idiomatic hook that creates stores and manages
 * the WebSocket connection lifecycle.
 *
 * @param options - Connection options
 * @returns Dashboard live result with stores and methods
 *
 * @example
 * ```svelte
 * <script>
 *   import { useDashboardLive } from '$lib/hooks/use-dashboard-live';
 *
 *   const { state, connected, connect, disconnect } = useDashboardLive({
 *     sessionId: 'abc-123',
 *     onUpdate: (update) => console.log('Update:', update),
 *   });
 *
 *   onMount(() => {
 *     connect();
 *     return () => disconnect();
 *   });
 * </script>
 *
 * {#if $connected}
 *   <span class="status connected">Live</span>
 * {:else}
 *   <span class="status disconnected">Offline</span>
 * {/if}
 * ```
 */
export function useDashboardLive(options: UseDashboardLiveOptions): DashboardLiveResult {
    const {
        sessionId,
        onUpdate,
        onConnectionChange,
        reconnectInterval = DEFAULT_RECONNECT_INTERVAL,
        maxReconnectAttempts = DEFAULT_MAX_RECONNECT_ATTEMPTS,
    } = options;

    // Internal state
    const state: Writable<WebSocketConnectionState> = writable(createInitialState());
    let socket: WebSocket | null = null;
    let heartbeatTimer: ReturnType<typeof setInterval> | null = null;
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    let connectionTimeout: ReturnType<typeof setTimeout> | null = null;
    let isIntentionalDisconnect = false;

    // Derived stores
    const connected: Readable<boolean> = derived(state, ($state) => $state.connected);
    const error: Readable<Error | null> = derived(state, ($state) => $state.error);

    /**
     * Build WebSocket URL for the session.
     */
    function buildWebSocketUrl(): string {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const host = window.location.host;
        return `${protocol}//${host}/api/sessions/${sessionId}/live`;
    }

    /**
     * Send a message to the server.
     */
    function send(message: ClientMessage): void {
        if (socket?.readyState === WebSocket.OPEN) {
            socket.send(JSON.stringify(message));
        }
    }

    /**
     * Start heartbeat timer.
     */
    function startHeartbeat(): void {
        stopHeartbeat();
        heartbeatTimer = setInterval(() => {
            send({ type: 'ping' });
        }, HEARTBEAT_INTERVAL);
    }

    /**
     * Stop heartbeat timer.
     */
    function stopHeartbeat(): void {
        if (heartbeatTimer) {
            clearInterval(heartbeatTimer);
            heartbeatTimer = null;
        }
    }

    /**
     * Clear connection timeout.
     */
    function clearConnectionTimeout(): void {
        if (connectionTimeout) {
            clearTimeout(connectionTimeout);
            connectionTimeout = null;
        }
    }

    /**
     * Clear reconnect timer.
     */
    function clearReconnectTimer(): void {
        if (reconnectTimer) {
            clearTimeout(reconnectTimer);
            reconnectTimer = null;
        }
    }

    /**
     * Schedule reconnection with exponential backoff.
     */
    function scheduleReconnect(): void {
        state.update((s) => {
            const attempts = s.reconnectAttempts + 1;

            if (attempts > maxReconnectAttempts) {
                return {
                    ...s,
                    error: new Error(`Max reconnection attempts (${maxReconnectAttempts}) exceeded`),
                    reconnectAttempts: attempts,
                };
            }

            // Exponential backoff: base * 2^(attempt-1), capped at 30 seconds
            const delay = Math.min(reconnectInterval * Math.pow(2, attempts - 1), 30000);

            reconnectTimer = setTimeout(() => {
                connect();
            }, delay);

            return {
                ...s,
                reconnectAttempts: attempts,
            };
        });
    }

    /**
     * Handle incoming WebSocket message.
     */
    function handleMessage(event: MessageEvent): void {
        try {
            const message: ServerMessage = JSON.parse(event.data);

            switch (message.type) {
                case 'connected': {
                    const connectedMsg = message as ConnectedMessage;
                    clearConnectionTimeout();
                    state.update((s) => ({
                        ...s,
                        connected: true,
                        clientId: connectedMsg.clientId,
                        error: null,
                        reconnectAttempts: 0,
                    }));
                    onConnectionChange?.(true);
                    startHeartbeat();
                    break;
                }

                case 'dashboard.update': {
                    const updateMsg = message as DashboardUpdateMessage;
                    state.update((s) => ({
                        ...s,
                        lastUpdate: updateMsg,
                    }));
                    onUpdate?.(updateMsg);
                    // Dispatch to dashboard store via custom event
                    dispatchDashboardUpdate(updateMsg);
                    break;
                }

                case 'pong':
                    // Heartbeat response - connection is alive
                    break;

                case 'error': {
                    const errorMsg = message as { code: string; message: string };
                    state.update((s) => ({
                        ...s,
                        error: new Error(`${errorMsg.code}: ${errorMsg.message}`),
                    }));
                    break;
                }
            }
        } catch (e) {
            console.error('[useDashboardLive] Failed to parse message:', e);
        }
    }

    /**
     * Handle WebSocket open event.
     */
    function handleOpen(): void {
        // Connection opened, waiting for 'connected' message from server
        // The server sends this message with session and client IDs
    }

    /**
     * Handle WebSocket close event.
     */
    function handleClose(event: CloseEvent): void {
        stopHeartbeat();
        clearConnectionTimeout();

        const wasConnected = socket !== null;
        socket = null;

        state.update((s) => ({
            ...s,
            connected: false,
            clientId: null,
        }));

        if (wasConnected) {
            onConnectionChange?.(false);
        }

        // Attempt reconnection unless intentionally disconnected
        if (!isIntentionalDisconnect && !event.wasClean) {
            scheduleReconnect();
        }
    }

    /**
     * Handle WebSocket error event.
     */
    function handleError(_event: Event): void {
        state.update((s) => ({
            ...s,
            error: new Error('WebSocket connection error'),
        }));
    }

    /**
     * Connect to the WebSocket server.
     */
    function connect(): void {
        // Prevent multiple connections
        if (socket?.readyState === WebSocket.OPEN || socket?.readyState === WebSocket.CONNECTING) {
            return;
        }

        isIntentionalDisconnect = false;
        clearReconnectTimer();

        try {
            const url = buildWebSocketUrl();
            socket = new WebSocket(url);

            socket.onopen = handleOpen;
            socket.onmessage = handleMessage;
            socket.onclose = handleClose;
            socket.onerror = handleError;

            // Set connection timeout
            connectionTimeout = setTimeout(() => {
                if (socket?.readyState !== WebSocket.OPEN) {
                    state.update((s) => ({
                        ...s,
                        error: new Error('Connection timeout'),
                    }));
                    socket?.close();
                }
            }, CONNECTION_TIMEOUT);
        } catch (e) {
            state.update((s) => ({
                ...s,
                error: e instanceof Error ? e : new Error('Failed to create WebSocket'),
            }));
        }
    }

    /**
     * Disconnect from the WebSocket server.
     */
    function disconnect(): void {
        isIntentionalDisconnect = true;
        stopHeartbeat();
        clearConnectionTimeout();
        clearReconnectTimer();

        if (socket) {
            socket.close(1000, 'Client disconnect');
            socket = null;
        }

        state.set(createInitialState());
    }

    /**
     * Request full state from the server.
     *
     * Useful after reconnection to sync with current state.
     */
    function requestState(): void {
        send({ type: 'request.state' });
    }

    return {
        state,
        connected,
        error,
        connect,
        disconnect,
        requestState,
    };
}

/**
 * Calculate exponential backoff delay.
 *
 * @param attempt - Current attempt number (1-based)
 * @param baseDelay - Base delay in milliseconds
 * @param maxDelay - Maximum delay cap in milliseconds
 * @returns Delay in milliseconds
 */
export function calculateBackoff(attempt: number, baseDelay: number, maxDelay: number): number {
    return Math.min(baseDelay * Math.pow(2, attempt - 1), maxDelay);
}
