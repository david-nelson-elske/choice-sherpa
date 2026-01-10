<script lang="ts">
    import { PricingTable, PromoCodeInput } from '$lib/components/membership';
    import { membershipStore, currentTier } from '$lib/stores';
    import { calculateAnnualSavingsPercent } from '$lib/types';
    import { onMount } from 'svelte';

    const savingsPercent = calculateAnnualSavingsPercent();

    onMount(() => {
        membershipStore.fetchMembership();
    });
</script>

<svelte:head>
    <title>Pricing - Choice Sherpa</title>
    <meta
        name="description"
        content="Choose the plan that's right for your decision-making journey. Start free, upgrade anytime."
    />
</svelte:head>

<div class="min-h-screen bg-gray-50">
    <!-- Hero Section -->
    <div class="bg-white">
        <div class="mx-auto max-w-7xl px-4 py-16 sm:px-6 lg:px-8">
            <div class="text-center">
                <h1 class="text-4xl font-bold tracking-tight text-gray-900 sm:text-5xl">
                    Simple, Transparent Pricing
                </h1>
                <p class="mx-auto mt-4 max-w-2xl text-lg text-gray-600">
                    Choose the plan that fits your decision-making needs.
                    Start free and upgrade when you're ready.
                </p>
            </div>

            <!-- Trust badges -->
            <div class="mt-8 flex flex-wrap items-center justify-center gap-8 text-sm text-gray-500">
                <div class="flex items-center gap-2">
                    <svg class="h-5 w-5 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                        <path
                            fill-rule="evenodd"
                            d="M2.166 4.999A11.954 11.954 0 0010 1.944 11.954 11.954 0 0017.834 5c.11.65.166 1.32.166 2.001 0 5.225-3.34 9.67-8 11.317C5.34 16.67 2 12.225 2 7c0-.682.057-1.35.166-2.001zm11.541 3.708a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                            clip-rule="evenodd"
                        />
                    </svg>
                    <span>Secure payments</span>
                </div>
                <div class="flex items-center gap-2">
                    <svg class="h-5 w-5 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                        <path
                            fill-rule="evenodd"
                            d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                            clip-rule="evenodd"
                        />
                    </svg>
                    <span>Cancel anytime</span>
                </div>
                <div class="flex items-center gap-2">
                    <svg class="h-5 w-5 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                        <path
                            fill-rule="evenodd"
                            d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                            clip-rule="evenodd"
                        />
                    </svg>
                    <span>No hidden fees</span>
                </div>
            </div>
        </div>
    </div>

    <!-- Pricing Cards -->
    <div class="mx-auto max-w-7xl px-4 py-12 sm:px-6 lg:px-8">
        <PricingTable currentTier={$currentTier} />
    </div>

    <!-- Promo Code Section -->
    <div class="bg-white py-12">
        <div class="mx-auto max-w-md px-4 sm:px-6 lg:px-8">
            <div class="text-center">
                <h2 class="text-lg font-semibold text-gray-900">Have a Promo Code?</h2>
                <p class="mt-1 text-sm text-gray-500">
                    Enter your code below to unlock special access.
                </p>
            </div>
            <div class="mt-6">
                <PromoCodeInput canApplyDirectly={true} />
            </div>
        </div>
    </div>

    <!-- FAQ Section -->
    <div class="mx-auto max-w-3xl px-4 py-16 sm:px-6 lg:px-8">
        <h2 class="text-center text-2xl font-bold text-gray-900">
            Frequently Asked Questions
        </h2>

        <dl class="mt-8 space-y-6">
            <div>
                <dt class="text-lg font-medium text-gray-900">
                    Can I switch plans later?
                </dt>
                <dd class="mt-2 text-gray-600">
                    Yes! You can upgrade or downgrade your plan at any time.
                    When upgrading, you'll be charged a prorated amount.
                    When downgrading, the change takes effect at the end of your billing period.
                </dd>
            </div>

            <div>
                <dt class="text-lg font-medium text-gray-900">
                    What happens when I cancel?
                </dt>
                <dd class="mt-2 text-gray-600">
                    You'll retain access until the end of your current billing period.
                    Your data is preserved, so you can re-subscribe anytime.
                </dd>
            </div>

            <div>
                <dt class="text-lg font-medium text-gray-900">
                    Is there a refund policy?
                </dt>
                <dd class="mt-2 text-gray-600">
                    We offer a 14-day money-back guarantee on all paid plans.
                    If you're not satisfied, contact us within 14 days for a full refund.
                </dd>
            </div>

            <div>
                <dt class="text-lg font-medium text-gray-900">
                    What payment methods do you accept?
                </dt>
                <dd class="mt-2 text-gray-600">
                    We accept all major credit cards (Visa, MasterCard, American Express)
                    processed securely through Stripe.
                </dd>
            </div>

            <div>
                <dt class="text-lg font-medium text-gray-900">
                    How much do I save with annual billing?
                </dt>
                <dd class="mt-2 text-gray-600">
                    Annual billing saves you {savingsPercent}% compared to monthly billing -
                    that's 2 months free!
                </dd>
            </div>
        </dl>
    </div>

    <!-- CTA Section -->
    <div class="bg-blue-600">
        <div class="mx-auto max-w-7xl px-4 py-12 sm:px-6 lg:px-8">
            <div class="text-center">
                <h2 class="text-2xl font-bold text-white">
                    Ready to make better decisions?
                </h2>
                <p class="mt-2 text-blue-100">
                    Start with our free plan - no credit card required.
                </p>
                <div class="mt-6">
                    <a
                        href="/register"
                        class="inline-block rounded-lg bg-white px-6 py-3 font-medium text-blue-600 hover:bg-blue-50"
                    >
                        Get Started Free
                    </a>
                </div>
            </div>
        </div>
    </div>
</div>
