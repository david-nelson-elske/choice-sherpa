<script lang="ts">
    import { membershipStore } from '$lib/stores';
    import { MembershipBadge } from '$lib/components/membership';
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';

    let loading = true;
    let error: string | null = null;

    onMount(async () => {
        // Fetch updated membership after Stripe checkout
        try {
            await membershipStore.fetchMembership();
            loading = false;

            // Auto-redirect to account page after 5 seconds
            setTimeout(() => {
                goto('/account');
            }, 5000);
        } catch (err) {
            error = 'Failed to load membership status';
            loading = false;
        }
    });

    $: membership = $membershipStore.membership;
</script>

<svelte:head>
    <title>Welcome! - Choice Sherpa</title>
</svelte:head>

<div class="flex min-h-screen items-center justify-center bg-gray-50 px-4">
    <div class="w-full max-w-md text-center">
        {#if loading}
            <div class="animate-pulse">
                <div class="mx-auto h-16 w-16 rounded-full bg-gray-200"></div>
                <div class="mt-4 h-8 rounded bg-gray-200"></div>
                <div class="mt-2 h-4 rounded bg-gray-200"></div>
            </div>
        {:else if error}
            <div class="rounded-xl border border-red-200 bg-red-50 p-8">
                <div class="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-red-100">
                    <svg class="h-8 w-8 text-red-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M6 18L18 6M6 6l12 12"
                        />
                    </svg>
                </div>
                <h1 class="mt-4 text-2xl font-bold text-gray-900">Something Went Wrong</h1>
                <p class="mt-2 text-gray-600">{error}</p>
                <a
                    href="/account"
                    class="mt-6 inline-block rounded-lg bg-blue-600 px-6 py-2 font-medium text-white hover:bg-blue-700"
                >
                    Go to Account
                </a>
            </div>
        {:else}
            <div class="rounded-xl border border-gray-200 bg-white p-8 shadow-lg">
                <div class="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-green-100">
                    <svg class="h-8 w-8 text-green-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M5 13l4 4L19 7"
                        />
                    </svg>
                </div>

                <h1 class="mt-4 text-2xl font-bold text-gray-900">Welcome to Choice Sherpa!</h1>

                {#if membership}
                    <div class="mt-4">
                        <MembershipBadge tier={membership.tier} status={membership.status} size="lg" />
                    </div>

                    <p class="mt-4 text-gray-600">
                        Your subscription is now active. You have full access to all features.
                    </p>

                    <div class="mt-6 space-y-3">
                        <a
                            href="/dashboard"
                            class="block w-full rounded-lg bg-blue-600 px-6 py-3 font-medium text-white hover:bg-blue-700"
                        >
                            Start Making Decisions
                        </a>
                        <a
                            href="/account"
                            class="block w-full rounded-lg border border-gray-300 bg-white px-6 py-3 font-medium text-gray-700 hover:bg-gray-50"
                        >
                            View Account Details
                        </a>
                    </div>
                {:else}
                    <p class="mt-4 text-gray-600">
                        Your payment was successful! Setting up your account...
                    </p>
                {/if}

                <p class="mt-6 text-sm text-gray-500">
                    Redirecting to your account in 5 seconds...
                </p>
            </div>
        {/if}
    </div>
</div>
