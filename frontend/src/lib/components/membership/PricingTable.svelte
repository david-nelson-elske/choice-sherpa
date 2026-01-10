<script lang="ts">
    import type { MembershipTier } from '../../types';
    import {
        TIERS,
        formatTierPrice,
        formatMonthlyEquivalent,
        calculateAnnualSavingsPercent,
    } from '../../types';
    import CheckoutButton from './CheckoutButton.svelte';

    interface Props {
        currentTier?: MembershipTier;
        onSelect?: (tier: MembershipTier) => void;
    }

    let { currentTier = 'free', onSelect }: Props = $props();

    const tiers: MembershipTier[] = ['free', 'monthly', 'annual'];
    const savingsPercent = calculateAnnualSavingsPercent();
</script>

<div class="grid grid-cols-1 gap-6 md:grid-cols-3">
    {#each tiers as tier}
        {@const info = TIERS[tier]}
        {@const isCurrent = tier === currentTier}
        {@const isRecommended = info.recommended}

        <div
            class="relative rounded-2xl border-2 p-6 {isRecommended
                ? 'border-blue-500 shadow-lg'
                : 'border-gray-200'} {isCurrent ? 'bg-gray-50' : 'bg-white'}"
        >
            {#if isRecommended}
                <div
                    class="absolute -top-3 left-1/2 -translate-x-1/2 rounded-full bg-blue-500 px-4 py-1 text-sm font-medium text-white"
                >
                    Most Popular
                </div>
            {/if}

            {#if tier === 'annual'}
                <div
                    class="absolute -top-3 right-4 rounded-full bg-green-500 px-3 py-1 text-sm font-medium text-white"
                >
                    Save {savingsPercent}%
                </div>
            {/if}

            <div class="mb-4">
                <h3 class="text-xl font-bold text-gray-900">{info.name}</h3>
                <p class="mt-1 text-sm text-gray-500">{info.description}</p>
            </div>

            <div class="mb-6">
                <div class="text-3xl font-bold text-gray-900">
                    {formatTierPrice(tier)}
                </div>
                {#if tier === 'annual'}
                    <div class="text-sm text-gray-500">
                        {formatMonthlyEquivalent(tier)} equivalent
                    </div>
                {/if}
            </div>

            <ul class="mb-6 space-y-3">
                {#each info.features as feature}
                    <li class="flex items-start">
                        <svg
                            class="mr-2 h-5 w-5 flex-shrink-0 text-green-500"
                            fill="currentColor"
                            viewBox="0 0 20 20"
                        >
                            <path
                                fill-rule="evenodd"
                                d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                                clip-rule="evenodd"
                            />
                        </svg>
                        <span class="text-sm text-gray-600">{feature}</span>
                    </li>
                {/each}
            </ul>

            <div class="mt-auto">
                {#if isCurrent}
                    <button
                        disabled
                        class="w-full rounded-lg bg-gray-100 px-4 py-2 font-medium text-gray-500"
                    >
                        Current Plan
                    </button>
                {:else if tier === 'free'}
                    <button
                        disabled
                        class="w-full rounded-lg bg-gray-100 px-4 py-2 font-medium text-gray-500"
                    >
                        Free Forever
                    </button>
                {:else}
                    <CheckoutButton
                        {tier}
                        variant={isRecommended ? 'primary' : 'secondary'}
                        fullWidth
                    />
                {/if}
            </div>
        </div>
    {/each}
</div>
