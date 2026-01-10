<script lang="ts">
    import { validatePromoCode, applyPromoCode } from '../../api';
    import type { PromoCodeValidation } from '../../types';

    interface Props {
        onApplied?: (code: string) => void;
        canApplyDirectly?: boolean; // For free tier promo codes
    }

    let { onApplied, canApplyDirectly = false }: Props = $props();

    let code = $state('');
    let loading = $state(false);
    let validation = $state<PromoCodeValidation | null>(null);
    let error = $state<string | null>(null);
    let applied = $state(false);

    async function handleValidate() {
        if (!code.trim()) return;

        loading = true;
        error = null;
        validation = null;

        try {
            const response = await validatePromoCode(code.trim());

            if (response.ok && response.data) {
                validation = response.data;
                if (!validation.valid) {
                    error = validation.error || 'Invalid promo code';
                }
            } else {
                error = response.error?.message || 'Failed to validate code';
            }
        } catch (err) {
            error = err instanceof Error ? err.message : 'Failed to validate code';
        } finally {
            loading = false;
        }
    }

    async function handleApply() {
        if (!validation?.valid || !canApplyDirectly) return;

        loading = true;
        error = null;

        try {
            const response = await applyPromoCode(code.trim());

            if (response.ok) {
                applied = true;
                onApplied?.(code.trim());
            } else {
                error = response.error?.message || 'Failed to apply code';
            }
        } catch (err) {
            error = err instanceof Error ? err.message : 'Failed to apply code';
        } finally {
            loading = false;
        }
    }

    function handleInput() {
        // Reset validation when code changes
        validation = null;
        error = null;
        applied = false;
    }
</script>

<div class="space-y-2">
    <label for="promo-code" class="block text-sm font-medium text-gray-700">
        Promo Code
    </label>

    <div class="flex gap-2">
        <input
            id="promo-code"
            type="text"
            bind:value={code}
            oninput={handleInput}
            placeholder="Enter code"
            disabled={loading || applied}
            class="flex-1 rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500
                   {applied ? 'bg-green-50' : ''}"
        />

        {#if !validation?.valid && !applied}
            <button
                onclick={handleValidate}
                disabled={loading || !code.trim()}
                class="rounded-lg bg-gray-100 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 disabled:opacity-50"
            >
                {loading ? 'Checking...' : 'Validate'}
            </button>
        {:else if validation?.valid && canApplyDirectly && !applied}
            <button
                onclick={handleApply}
                disabled={loading}
                class="rounded-lg bg-green-600 px-4 py-2 text-sm font-medium text-white hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-offset-2 disabled:opacity-50"
            >
                {loading ? 'Applying...' : 'Apply'}
            </button>
        {/if}
    </div>

    {#if validation?.valid && !applied}
        <div class="flex items-center gap-2 text-sm text-green-600">
            <svg class="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
                <path
                    fill-rule="evenodd"
                    d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                    clip-rule="evenodd"
                />
            </svg>
            <span>
                Valid code: {validation.description || `Unlocks ${validation.tier} tier`}
            </span>
        </div>
    {/if}

    {#if applied}
        <div class="flex items-center gap-2 text-sm text-green-600">
            <svg class="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
                <path
                    fill-rule="evenodd"
                    d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                    clip-rule="evenodd"
                />
            </svg>
            <span>Code applied successfully!</span>
        </div>
    {/if}

    {#if error}
        <p class="text-sm text-red-600">{error}</p>
    {/if}
</div>
