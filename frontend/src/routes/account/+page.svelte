<script lang="ts">
    import { MembershipStatus, UpgradePrompt, MembershipBadge } from '$lib/components/membership';
    import { membershipStore, currentTier, isPaid, canUpgrade, isExpiringSoon } from '$lib/stores';
    import type { UsageStats } from '$lib/types';
    import { getTierLimits } from '$lib/types';
    import { onMount } from 'svelte';

    let usage: UsageStats | null = null;
    let loadingUsage = true;

    onMount(async () => {
        await membershipStore.fetchAll();
        loadingUsage = false;
    });

    $: membership = $membershipStore.membership;
    $: limits = $membershipStore.limits;
    $: usageData = $membershipStore.usage;
</script>

<svelte:head>
    <title>Account - Choice Sherpa</title>
</svelte:head>

<div class="min-h-screen bg-gray-50 py-8">
    <div class="mx-auto max-w-4xl px-4 sm:px-6 lg:px-8">
        <!-- Header -->
        <div class="mb-8">
            <h1 class="text-3xl font-bold text-gray-900">Account</h1>
            <p class="mt-1 text-gray-600">Manage your membership and view usage</p>
        </div>

        <!-- Expiring Soon Alert -->
        {#if $isExpiringSoon}
            <div class="mb-6">
                <UpgradePrompt
                    currentTier={$currentTier}
                    variant="banner"
                    reason={{ type: 'membership_expired' }}
                />
            </div>
        {/if}

        <div class="grid gap-6 lg:grid-cols-2">
            <!-- Membership Card -->
            <div class="lg:col-span-2">
                {#if $membershipStore.loading}
                    <div class="animate-pulse rounded-xl border border-gray-200 bg-white p-6">
                        <div class="h-6 w-1/3 rounded bg-gray-200"></div>
                        <div class="mt-4 h-8 w-1/4 rounded bg-gray-200"></div>
                        <div class="mt-4 grid grid-cols-2 gap-4">
                            <div class="h-16 rounded bg-gray-200"></div>
                            <div class="h-16 rounded bg-gray-200"></div>
                        </div>
                    </div>
                {:else if membership}
                    <MembershipStatus {membership} />
                {:else}
                    <div class="rounded-xl border border-gray-200 bg-white p-6 text-center">
                        <p class="text-gray-600">No active membership</p>
                        <a
                            href="/pricing"
                            class="mt-4 inline-block rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
                        >
                            View Plans
                        </a>
                    </div>
                {/if}
            </div>

            <!-- Usage Card -->
            <div class="rounded-xl border border-gray-200 bg-white p-6">
                <h3 class="text-lg font-semibold text-gray-900">Usage</h3>

                {#if loadingUsage}
                    <div class="mt-4 animate-pulse space-y-4">
                        <div class="h-4 rounded bg-gray-200"></div>
                        <div class="h-4 rounded bg-gray-200"></div>
                        <div class="h-4 rounded bg-gray-200"></div>
                    </div>
                {:else}
                    <dl class="mt-4 space-y-4">
                        <div>
                            <dt class="text-sm text-gray-500">Active Sessions</dt>
                            <dd class="mt-1 flex items-baseline gap-2">
                                <span class="text-2xl font-semibold text-gray-900">
                                    {usageData?.activeSessions || 0}
                                </span>
                                {#if limits.maxSessions !== null}
                                    <span class="text-sm text-gray-500">
                                        / {limits.maxSessions}
                                    </span>
                                {:else}
                                    <span class="text-sm text-gray-500">unlimited</span>
                                {/if}
                            </dd>
                            {#if limits.maxSessions !== null}
                                <div class="mt-1 h-2 overflow-hidden rounded-full bg-gray-200">
                                    <div
                                        class="h-full bg-blue-600"
                                        style="width: {Math.min(100, ((usageData?.activeSessions || 0) / limits.maxSessions) * 100)}%"
                                    ></div>
                                </div>
                            {/if}
                        </div>

                        <div>
                            <dt class="text-sm text-gray-500">Total Cycles</dt>
                            <dd class="mt-1 text-2xl font-semibold text-gray-900">
                                {usageData?.totalCycles || 0}
                            </dd>
                        </div>

                        {#if limits.exportEnabled}
                            <div>
                                <dt class="text-sm text-gray-500">Exports This Month</dt>
                                <dd class="mt-1 text-2xl font-semibold text-gray-900">
                                    {usageData?.exportsThisMonth || 0}
                                </dd>
                            </div>
                        {/if}
                    </dl>
                {/if}
            </div>

            <!-- Limits Card -->
            <div class="rounded-xl border border-gray-200 bg-white p-6">
                <h3 class="text-lg font-semibold text-gray-900">Plan Limits</h3>

                <dl class="mt-4 space-y-4">
                    <div class="flex justify-between">
                        <dt class="text-sm text-gray-500">Sessions</dt>
                        <dd class="text-sm font-medium text-gray-900">
                            {limits.maxSessions === null ? 'Unlimited' : limits.maxSessions}
                        </dd>
                    </div>

                    <div class="flex justify-between">
                        <dt class="text-sm text-gray-500">Cycles per Session</dt>
                        <dd class="text-sm font-medium text-gray-900">
                            {limits.maxCyclesPerSession === null ? 'Unlimited' : limits.maxCyclesPerSession}
                        </dd>
                    </div>

                    <div class="flex justify-between">
                        <dt class="text-sm text-gray-500">Export</dt>
                        <dd class="text-sm font-medium text-gray-900">
                            {limits.exportEnabled ? 'Enabled' : 'Not included'}
                        </dd>
                    </div>

                    <div class="flex justify-between">
                        <dt class="text-sm text-gray-500">History Retention</dt>
                        <dd class="text-sm font-medium text-gray-900">
                            {limits.historyRetentionDays === null
                                ? 'Forever'
                                : `${limits.historyRetentionDays} days`}
                        </dd>
                    </div>
                </dl>

                {#if $canUpgrade}
                    <div class="mt-6 border-t border-gray-100 pt-4">
                        <a
                            href="/pricing"
                            class="block w-full rounded-lg bg-blue-600 px-4 py-2 text-center text-sm font-medium text-white hover:bg-blue-700"
                        >
                            Upgrade for More
                        </a>
                    </div>
                {/if}
            </div>
        </div>

        <!-- Upgrade Prompt for Free Users -->
        {#if $currentTier === 'free'}
            <div class="mt-8">
                <UpgradePrompt
                    currentTier="free"
                    suggestedTier="monthly"
                    variant="inline"
                />
            </div>
        {/if}
    </div>
</div>
