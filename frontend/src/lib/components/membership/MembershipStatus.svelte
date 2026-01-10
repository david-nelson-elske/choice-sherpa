<script lang="ts">
    import type { MembershipView } from '../../types';
    import {
        formatStatus,
        getStatusColor,
        formatTierPrice,
        formatDaysRemaining,
        TIERS,
    } from '../../types';
    import { redirectToPortal } from '../../api';
    import MembershipBadge from './MembershipBadge.svelte';

    interface Props {
        membership: MembershipView;
    }

    let { membership }: Props = $props();

    let loadingPortal = $state(false);
    let portalError = $state<string | null>(null);

    $: tierInfo = TIERS[membership.tier];
    $: isExpiringSoon = membership.daysRemaining > 0 && membership.daysRemaining <= 7;
    $: periodEndDate = new Date(membership.periodEnd).toLocaleDateString('en-CA', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
    });

    async function handleManageSubscription() {
        loadingPortal = true;
        portalError = null;

        try {
            await redirectToPortal();
        } catch (err) {
            portalError = err instanceof Error ? err.message : 'Failed to open portal';
            loadingPortal = false;
        }
    }
</script>

<div class="rounded-xl border border-gray-200 bg-white p-6">
    <div class="flex items-start justify-between">
        <div>
            <h3 class="text-lg font-semibold text-gray-900">Your Membership</h3>
            <div class="mt-2">
                <MembershipBadge tier={membership.tier} status={membership.status} size="md" />
            </div>
        </div>

        <div class="text-right">
            <div class="text-2xl font-bold text-gray-900">
                {formatTierPrice(membership.tier)}
            </div>
            {#if membership.tier !== 'free'}
                <div class="text-sm text-gray-500">
                    {tierInfo.billingPeriod === 'yearly' ? 'per year' : 'per month'}
                </div>
            {/if}
        </div>
    </div>

    {#if membership.tier !== 'free'}
        <div class="mt-4 border-t border-gray-100 pt-4">
            <dl class="grid grid-cols-2 gap-4 text-sm">
                <div>
                    <dt class="text-gray-500">Status</dt>
                    <dd class="mt-1 font-medium">
                        <span class="inline-flex items-center rounded-full px-2 py-0.5 {getStatusColor(membership.status)}">
                            {formatStatus(membership.status)}
                        </span>
                    </dd>
                </div>

                <div>
                    <dt class="text-gray-500">
                        {membership.status === 'cancelled' ? 'Access Until' : 'Next Billing'}
                    </dt>
                    <dd class="mt-1 font-medium text-gray-900">
                        {periodEndDate}
                    </dd>
                </div>

                {#if membership.status !== 'expired'}
                    <div class="col-span-2">
                        <dt class="text-gray-500">Time Remaining</dt>
                        <dd class="mt-1 font-medium {isExpiringSoon ? 'text-orange-600' : 'text-gray-900'}">
                            {formatDaysRemaining(membership.daysRemaining)}
                        </dd>
                    </div>
                {/if}
            </dl>
        </div>
    {/if}

    {#if membership.promoCode}
        <div class="mt-4 rounded-lg bg-purple-50 p-3">
            <div class="flex items-center gap-2 text-sm text-purple-700">
                <svg class="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
                    <path
                        fill-rule="evenodd"
                        d="M5 2a2 2 0 00-2 2v14l3.5-2 3.5 2 3.5-2 3.5 2V4a2 2 0 00-2-2H5zm2.5 3a1.5 1.5 0 100 3 1.5 1.5 0 000-3zm6.207.293a1 1 0 00-1.414 0l-6 6a1 1 0 101.414 1.414l6-6a1 1 0 000-1.414zM12.5 10a1.5 1.5 0 100 3 1.5 1.5 0 000-3z"
                        clip-rule="evenodd"
                    />
                </svg>
                <span>Promo code applied: <strong>{membership.promoCode}</strong></span>
            </div>
        </div>
    {/if}

    {#if membership.tier !== 'free' && membership.status !== 'expired'}
        <div class="mt-6">
            <button
                onclick={handleManageSubscription}
                disabled={loadingPortal}
                class="w-full rounded-lg border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50"
            >
                {loadingPortal ? 'Loading...' : 'Manage Subscription'}
            </button>

            {#if portalError}
                <p class="mt-2 text-sm text-red-600">{portalError}</p>
            {/if}
        </div>
    {/if}

    {#if isExpiringSoon && membership.status !== 'cancelled'}
        <div class="mt-4 rounded-lg bg-orange-50 p-3">
            <div class="flex items-start gap-2">
                <svg class="h-5 w-5 flex-shrink-0 text-orange-500" fill="currentColor" viewBox="0 0 20 20">
                    <path
                        fill-rule="evenodd"
                        d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                        clip-rule="evenodd"
                    />
                </svg>
                <div class="text-sm text-orange-700">
                    <p class="font-medium">Your subscription is expiring soon</p>
                    <p class="mt-1">
                        Please update your payment method to avoid losing access.
                    </p>
                </div>
            </div>
        </div>
    {/if}
</div>
