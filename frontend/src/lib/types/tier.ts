/**
 * Membership tier types and utilities.
 *
 * Mirrors the backend MembershipTier enum and TierLimits.
 */

import { PRICING, type Cents, formatMoney, formatMoneyShort } from './money';

/** Membership tier levels */
export type MembershipTier = 'free' | 'monthly' | 'annual';

/** Membership status values */
export type MembershipStatus = 'pending' | 'active' | 'past_due' | 'cancelled' | 'expired';

/** Tier display information */
export interface TierInfo {
    tier: MembershipTier;
    name: string;
    description: string;
    priceMonthly: Cents;
    priceAnnual?: Cents;
    billingPeriod: 'free' | 'monthly' | 'yearly';
    features: string[];
    limits: TierLimits;
    recommended?: boolean;
}

/** Usage limits for a tier */
export interface TierLimits {
    maxSessions: number | null; // null = unlimited
    maxCyclesPerSession: number | null;
    exportEnabled: boolean;
    historyRetentionDays: number | null;
}

/** Tier configuration */
export const TIERS: Record<MembershipTier, TierInfo> = {
    free: {
        tier: 'free',
        name: 'Free',
        description: 'Get started with structured decision-making',
        priceMonthly: PRICING.FREE,
        billingPeriod: 'free',
        features: [
            '3 active sessions',
            '5 cycles per session',
            'Full PrOACT framework',
            'AI-guided conversations',
            '7-day history retention',
        ],
        limits: {
            maxSessions: 3,
            maxCyclesPerSession: 5,
            exportEnabled: false,
            historyRetentionDays: 7,
        },
    },
    monthly: {
        tier: 'monthly',
        name: 'Monthly',
        description: 'Unlimited decision support',
        priceMonthly: PRICING.MONTHLY,
        billingPeriod: 'monthly',
        features: [
            'Unlimited sessions',
            'Unlimited cycles',
            'Export to PDF/JSON',
            'Unlimited history',
            'Priority support',
        ],
        limits: {
            maxSessions: null,
            maxCyclesPerSession: null,
            exportEnabled: true,
            historyRetentionDays: null,
        },
        recommended: true,
    },
    annual: {
        tier: 'annual',
        name: 'Annual',
        description: 'Best value for committed decision-makers',
        priceMonthly: PRICING.ANNUAL_MONTHLY_EQUIVALENT,
        priceAnnual: PRICING.ANNUAL,
        billingPeriod: 'yearly',
        features: [
            'Everything in Monthly',
            '2 months free',
            'Early access to new features',
            'Annual billing',
        ],
        limits: {
            maxSessions: null,
            maxCyclesPerSession: null,
            exportEnabled: true,
            historyRetentionDays: null,
        },
    },
};

/** Get tier info by tier level */
export function getTierInfo(tier: MembershipTier): TierInfo {
    return TIERS[tier];
}

/** Get tier limits by tier level */
export function getTierLimits(tier: MembershipTier): TierLimits {
    return TIERS[tier].limits;
}

/** Format price for display */
export function formatTierPrice(tier: MembershipTier): string {
    const info = TIERS[tier];
    if (info.tier === 'free') {
        return 'Free';
    }
    if (info.billingPeriod === 'yearly') {
        return `${formatMoneyShort(info.priceAnnual!)}/year`;
    }
    return `${formatMoneyShort(info.priceMonthly)}/month`;
}

/** Format monthly equivalent price */
export function formatMonthlyEquivalent(tier: MembershipTier): string {
    const info = TIERS[tier];
    if (info.tier === 'free') {
        return 'Free forever';
    }
    return `${formatMoney(info.priceMonthly)}/month`;
}

/** Check if tier is paid */
export function isPaidTier(tier: MembershipTier): boolean {
    return tier !== 'free';
}

/** Check if status grants access */
export function statusHasAccess(status: MembershipStatus): boolean {
    return status === 'active' || status === 'past_due' || status === 'cancelled';
}

/** Get display name for status */
export function formatStatus(status: MembershipStatus): string {
    const statusNames: Record<MembershipStatus, string> = {
        pending: 'Pending',
        active: 'Active',
        past_due: 'Past Due',
        cancelled: 'Cancelled',
        expired: 'Expired',
    };
    return statusNames[status];
}

/** Get status badge color class */
export function getStatusColor(status: MembershipStatus): string {
    const colors: Record<MembershipStatus, string> = {
        pending: 'bg-yellow-100 text-yellow-800',
        active: 'bg-green-100 text-green-800',
        past_due: 'bg-orange-100 text-orange-800',
        cancelled: 'bg-gray-100 text-gray-800',
        expired: 'bg-red-100 text-red-800',
    };
    return colors[status];
}

/** Compare tiers (returns positive if a > b) */
export function compareTiers(a: MembershipTier, b: MembershipTier): number {
    const order: Record<MembershipTier, number> = { free: 0, monthly: 1, annual: 2 };
    return order[a] - order[b];
}

/** Check if upgrade is available from current tier */
export function canUpgradeTo(current: MembershipTier, target: MembershipTier): boolean {
    return compareTiers(target, current) > 0;
}
