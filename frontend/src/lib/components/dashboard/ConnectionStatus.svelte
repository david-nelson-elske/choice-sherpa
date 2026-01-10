<script lang="ts">
    import type { Readable } from 'svelte/store';

    interface Props {
        /** Whether connected to the WebSocket server */
        connected: Readable<boolean>;
        /** Current error if any */
        error?: Readable<Error | null>;
        /** Number of reconnection attempts */
        reconnectAttempts?: number;
        /** Maximum reconnection attempts before giving up */
        maxReconnectAttempts?: number;
        /** Callback to manually reconnect */
        onReconnect?: () => void;
        /** Size variant */
        size?: 'sm' | 'md';
        /** Whether to show detailed status */
        detailed?: boolean;
    }

    let {
        connected,
        error,
        reconnectAttempts = 0,
        maxReconnectAttempts = 10,
        onReconnect,
        size = 'md',
        detailed = false,
    }: Props = $props();

    // Derive the connection status text
    let statusText = $derived.by(() => {
        if ($connected) return 'Live';
        if ($error && reconnectAttempts >= maxReconnectAttempts) return 'Disconnected';
        if (reconnectAttempts > 0) return 'Reconnecting...';
        return 'Connecting...';
    });

    // Derive the status indicator color
    let statusColor = $derived.by(() => {
        if ($connected) return 'bg-green-500';
        if ($error && reconnectAttempts >= maxReconnectAttempts) return 'bg-red-500';
        return 'bg-yellow-500';
    });

    // Derive animation class
    let animationClass = $derived.by(() => {
        if ($connected) return '';
        if (reconnectAttempts > 0 && reconnectAttempts < maxReconnectAttempts) {
            return 'animate-pulse';
        }
        return '';
    });

    // Size-based classes
    const sizeClasses = {
        sm: {
            container: 'text-xs',
            dot: 'w-2 h-2',
            gap: 'gap-1.5',
        },
        md: {
            container: 'text-sm',
            dot: 'w-2.5 h-2.5',
            gap: 'gap-2',
        },
    };

    let containerClass = $derived(sizeClasses[size].container);
    let dotClass = $derived(sizeClasses[size].dot);
    let gapClass = $derived(sizeClasses[size].gap);

    // Whether to show reconnect button
    let showReconnectButton = $derived(
        !$connected && reconnectAttempts >= maxReconnectAttempts && onReconnect
    );
</script>

<div class="inline-flex items-center {gapClass} {containerClass}">
    <!-- Status indicator dot -->
    <span
        class="rounded-full {statusColor} {dotClass} {animationClass}"
        aria-hidden="true"
    ></span>

    <!-- Status text -->
    <span class="font-medium text-gray-700">
        {statusText}
    </span>

    {#if detailed}
        <!-- Detailed reconnection info -->
        {#if !$connected && reconnectAttempts > 0 && reconnectAttempts < maxReconnectAttempts}
            <span class="text-gray-500">
                (attempt {reconnectAttempts}/{maxReconnectAttempts})
            </span>
        {/if}

        <!-- Error message -->
        {#if $error && error}
            <span class="text-red-600" title={$error.message}>
                - {$error.message}
            </span>
        {/if}
    {/if}

    <!-- Reconnect button -->
    {#if showReconnectButton}
        <button
            type="button"
            onclick={() => onReconnect?.()}
            class="ml-2 px-2 py-0.5 text-xs font-medium text-blue-600 hover:text-blue-800
                   bg-blue-50 hover:bg-blue-100 rounded transition-colors"
        >
            Reconnect
        </button>
    {/if}
</div>
