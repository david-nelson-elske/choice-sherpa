/**
 * Membership Svelte store.
 *
 * Provides reactive state management for membership data.
 */

import { writable, derived, type Readable } from 'svelte/store';
import type { MembershipView, UsageStats, TierLimits, MembershipTier } from '../types';
import { getTierLimits as getDefaultTierLimits, TIERS } from '../types';
import * as api from '../api';

/** Membership state */
interface MembershipStoreState {
    membership: MembershipView | null;
    usage: UsageStats | null;
    limits: TierLimits;
    loading: boolean;
    error: string | null;
}

/** Initial state */
const initialState: MembershipStoreState = {
    membership: null,
    usage: null,
    limits: getDefaultTierLimits('free'),
    loading: false,
    error: null,
};

/** Create the membership store */
function createMembershipStore() {
    const { subscribe, set, update } = writable<MembershipStoreState>(initialState);

    return {
        subscribe,

        /** Fetch current user's membership */
        async fetchMembership(): Promise<void> {
            update((state) => ({ ...state, loading: true, error: null }));

            const response = await api.getMembership();

            if (response.ok && response.data) {
                update((state) => ({
                    ...state,
                    membership: response.data!,
                    limits: getDefaultTierLimits(response.data!.tier),
                    loading: false,
                }));
            } else {
                update((state) => ({
                    ...state,
                    membership: null,
                    limits: getDefaultTierLimits('free'),
                    loading: false,
                    error: response.error?.message || 'Failed to fetch membership',
                }));
            }
        },

        /** Fetch current user's usage */
        async fetchUsage(): Promise<void> {
            const response = await api.getUsage();

            if (response.ok && response.data) {
                update((state) => ({
                    ...state,
                    usage: response.data!,
                }));
            }
        },

        /** Fetch all membership data */
        async fetchAll(): Promise<void> {
            update((state) => ({ ...state, loading: true, error: null }));

            const [membershipResponse, usageResponse] = await Promise.all([
                api.getMembership(),
                api.getUsage(),
            ]);

            if (membershipResponse.ok && membershipResponse.data) {
                update((state) => ({
                    ...state,
                    membership: membershipResponse.data!,
                    limits: getDefaultTierLimits(membershipResponse.data!.tier),
                    usage: usageResponse.ok ? usageResponse.data! : state.usage,
                    loading: false,
                }));
            } else {
                update((state) => ({
                    ...state,
                    membership: null,
                    limits: getDefaultTierLimits('free'),
                    loading: false,
                    error: membershipResponse.error?.message || 'Failed to fetch membership',
                }));
            }
        },

        /** Clear membership state (on logout) */
        clear(): void {
            set(initialState);
        },

        /** Set error */
        setError(error: string): void {
            update((state) => ({ ...state, error }));
        },

        /** Clear error */
        clearError(): void {
            update((state) => ({ ...state, error: null }));
        },
    };
}

/** Membership store instance */
export const membershipStore = createMembershipStore();

/** Derived store: current tier */
export const currentTier: Readable<MembershipTier> = derived(
    membershipStore,
    ($store) => $store.membership?.tier || 'free'
);

/** Derived store: has active membership */
export const hasAccess: Readable<boolean> = derived(
    membershipStore,
    ($store) => $store.membership?.hasAccess || false
);

/** Derived store: is paid user */
export const isPaid: Readable<boolean> = derived(
    membershipStore,
    ($store) => {
        const tier = $store.membership?.tier;
        return tier === 'monthly' || tier === 'annual';
    }
);

/** Derived store: is loading */
export const isLoading: Readable<boolean> = derived(
    membershipStore,
    ($store) => $store.loading
);

/** Derived store: current error */
export const membershipError: Readable<string | null> = derived(
    membershipStore,
    ($store) => $store.error
);

/** Derived store: days remaining */
export const daysRemaining: Readable<number> = derived(
    membershipStore,
    ($store) => $store.membership?.daysRemaining || 0
);

/** Derived store: is expiring soon (within 7 days) */
export const isExpiringSoon: Readable<boolean> = derived(
    membershipStore,
    ($store) => {
        const days = $store.membership?.daysRemaining || 0;
        const status = $store.membership?.status;
        return days > 0 && days <= 7 && status !== 'cancelled';
    }
);

/** Derived store: can upgrade */
export const canUpgrade: Readable<boolean> = derived(
    membershipStore,
    ($store) => {
        const tier = $store.membership?.tier;
        return tier === 'free' || tier === 'monthly';
    }
);

/** Derived store: tier info */
export const tierInfo: Readable<typeof TIERS[MembershipTier]> = derived(
    currentTier,
    ($tier) => TIERS[$tier]
);
