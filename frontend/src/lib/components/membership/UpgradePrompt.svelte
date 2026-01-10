<script lang="ts">
    import type { MembershipTier, AccessDeniedReason } from '../../types';
    import { TIERS, getAccessDeniedMessage } from '../../types';
    import CheckoutButton from './CheckoutButton.svelte';

    interface Props {
        reason?: AccessDeniedReason;
        currentTier?: MembershipTier;
        suggestedTier?: MembershipTier;
        variant?: 'inline' | 'modal' | 'banner';
        onDismiss?: () => void;
    }

    let {
        reason,
        currentTier = 'free',
        suggestedTier = 'monthly',
        variant = 'inline',
        onDismiss,
    }: Props = $props();

    $: message = reason ? getAccessDeniedMessage(reason) : 'Upgrade to unlock more features';
    $: tierInfo = TIERS[suggestedTier];

    const variantClasses = {
        inline: 'rounded-xl border border-blue-200 bg-blue-50 p-6',
        modal: 'rounded-2xl bg-white p-8 shadow-xl',
        banner: 'border-b border-blue-200 bg-blue-50 px-4 py-3',
    };
</script>

<div class={variantClasses[variant]}>
    {#if variant === 'banner'}
        <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
                <svg class="h-5 w-5 text-blue-600" fill="currentColor" viewBox="0 0 20 20">
                    <path
                        fill-rule="evenodd"
                        d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-8.707l-3-3a1 1 0 00-1.414 0l-3 3a1 1 0 001.414 1.414L9 9.414V13a1 1 0 102 0V9.414l1.293 1.293a1 1 0 001.414-1.414z"
                        clip-rule="evenodd"
                    />
                </svg>
                <span class="text-sm text-blue-800">{message}</span>
            </div>
            <div class="flex items-center gap-3">
                <a
                    href="/pricing"
                    class="text-sm font-medium text-blue-600 hover:text-blue-700"
                >
                    View Plans
                </a>
                {#if onDismiss}
                    <button
                        onclick={onDismiss}
                        class="text-blue-400 hover:text-blue-600"
                        aria-label="Dismiss"
                    >
                        <svg class="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
                            <path
                                fill-rule="evenodd"
                                d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                                clip-rule="evenodd"
                            />
                        </svg>
                    </button>
                {/if}
            </div>
        </div>
    {:else}
        <div class="text-center">
            <div class="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-blue-100">
                <svg class="h-6 w-6 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M13 10V3L4 14h7v7l9-11h-7z"
                    />
                </svg>
            </div>

            <h3 class="text-lg font-semibold text-gray-900">
                {currentTier === 'free' ? 'Unlock Your Full Potential' : 'Upgrade Your Plan'}
            </h3>

            <p class="mt-2 text-sm text-gray-600">
                {message}
            </p>

            <div class="mt-4 rounded-lg bg-white p-4 {variant === 'modal' ? 'border border-gray-200' : ''}">
                <div class="text-2xl font-bold text-gray-900">
                    {tierInfo.name}
                </div>
                <p class="mt-1 text-sm text-gray-500">{tierInfo.description}</p>

                <ul class="mt-4 space-y-2 text-left">
                    {#each tierInfo.features.slice(0, 3) as feature}
                        <li class="flex items-center gap-2 text-sm text-gray-600">
                            <svg class="h-4 w-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                                <path
                                    fill-rule="evenodd"
                                    d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                                    clip-rule="evenodd"
                                />
                            </svg>
                            {feature}
                        </li>
                    {/each}
                </ul>
            </div>

            <div class="mt-6 flex flex-col gap-3 sm:flex-row sm:justify-center">
                <CheckoutButton tier={suggestedTier} variant="primary" />

                <a
                    href="/pricing"
                    class="rounded-lg border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
                >
                    Compare All Plans
                </a>
            </div>

            {#if onDismiss && variant === 'modal'}
                <button
                    onclick={onDismiss}
                    class="mt-4 text-sm text-gray-500 hover:text-gray-700"
                >
                    Maybe Later
                </button>
            {/if}
        </div>
    {/if}
</div>
