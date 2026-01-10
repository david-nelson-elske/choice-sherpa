<script lang="ts">
    import type { MembershipTier } from '../../types';
    import { redirectToCheckout } from '../../api';

    interface Props {
        tier: MembershipTier;
        promoCode?: string;
        variant?: 'primary' | 'secondary';
        fullWidth?: boolean;
        label?: string;
    }

    let {
        tier,
        promoCode,
        variant = 'primary',
        fullWidth = false,
        label,
    }: Props = $props();

    let loading = $state(false);
    let error = $state<string | null>(null);

    const buttonLabel = label || (tier === 'monthly' ? 'Subscribe Monthly' : 'Subscribe Annually');

    const variantClasses = {
        primary: 'bg-blue-600 text-white hover:bg-blue-700 focus:ring-blue-500',
        secondary: 'bg-white text-gray-900 border border-gray-300 hover:bg-gray-50 focus:ring-gray-500',
    };

    async function handleClick() {
        loading = true;
        error = null;

        try {
            await redirectToCheckout(tier, { promoCode });
        } catch (err) {
            error = err instanceof Error ? err.message : 'Failed to start checkout';
            loading = false;
        }
    }
</script>

<button
    onclick={handleClick}
    disabled={loading}
    class="rounded-lg px-4 py-2 font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2
           {variantClasses[variant]}
           {fullWidth ? 'w-full' : ''}
           {loading ? 'cursor-wait opacity-75' : ''}"
>
    {#if loading}
        <span class="inline-flex items-center">
            <svg
                class="-ml-1 mr-2 h-4 w-4 animate-spin"
                fill="none"
                viewBox="0 0 24 24"
            >
                <circle
                    class="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    stroke-width="4"
                />
                <path
                    class="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
                />
            </svg>
            Processing...
        </span>
    {:else}
        {buttonLabel}
    {/if}
</button>

{#if error}
    <p class="mt-2 text-sm text-red-600">{error}</p>
{/if}
