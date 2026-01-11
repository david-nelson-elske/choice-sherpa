/**
 * Membership tier types and utilities.
 *
 * Mirrors the backend MembershipTier enum and TierLimits.
 *
 * @module tier
 */

import { PRICING, type Cents, formatMoney, formatMoneyShort } from './money';

/** Membership tier levels */
export type MembershipTier = 'free' | 'monthly' | 'annual';

/** Membership status values */
export type MembershipStatus = 'pending' | 'active' | 'past_due' | 'cancelled' | 'expired';

/** AI model tier levels */
export type AiModelTier = 'standard' | 'advanced';

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

/**
 * Complete feature limits for a membership tier.
 *
 * Mirrors the backend TierLimits struct.
 * null values indicate unlimited.
 */
export interface TierLimits {
    // Session & Cycle Limits
    /** Maximum active sessions. null = unlimited. */
    maxActiveSessions: number | null;
    /** Maximum cycles per session. null = unlimited. */
    maxCyclesPerSession: number | null;
    /** Maximum archived sessions. null = unlimited. */
    maxArchivedSessions: number | null;
    /** Session history retention in days. null = forever. */
    sessionHistoryDays: number | null;

    // AI Features
    /** Whether AI conversations are enabled. */
    aiEnabled: boolean;
    /** Maximum AI messages per day. null = unlimited. */
    aiMessagesPerDay: number | null;
    /** AI model tier (standard or advanced). */
    aiModelTier: AiModelTier;

    // Component Access
    /** Whether the Decision Quality component is accessible. */
    dqComponentEnabled: boolean;

    // Analysis Features
    /** Whether full tradeoff analysis is available. */
    fullTradeoffAnalysis: boolean;
    /** Whether DQ scoring is enabled. */
    dqScoringEnabled: boolean;
    /** Whether improvement suggestions are shown. */
    improvementSuggestionsEnabled: boolean;

    // Export & Sharing
    /** Whether PDF export is enabled. */
    pdfExportEnabled: boolean;
    /** Whether share link generation is enabled. */
    shareLinkEnabled: boolean;
    /** Whether API access is enabled. */
    apiAccess: boolean;
}

/** Default limits for Free tier */
const FREE_LIMITS: TierLimits = {
    maxActiveSessions: 3,
    maxCyclesPerSession: 2,
    maxArchivedSessions: 10,
    sessionHistoryDays: 90,

    aiEnabled: true,
    aiMessagesPerDay: 50,
    aiModelTier: 'standard',

    dqComponentEnabled: false,

    fullTradeoffAnalysis: false,
    dqScoringEnabled: false,
    improvementSuggestionsEnabled: false,

    pdfExportEnabled: false,
    shareLinkEnabled: false,
    apiAccess: false,
};

/** Default limits for Monthly (Premium) tier */
const MONTHLY_LIMITS: TierLimits = {
    maxActiveSessions: 10,
    maxCyclesPerSession: 5,
    maxArchivedSessions: 50,
    sessionHistoryDays: 365,

    aiEnabled: true,
    aiMessagesPerDay: 200,
    aiModelTier: 'standard',

    dqComponentEnabled: true,

    fullTradeoffAnalysis: true,
    dqScoringEnabled: true,
    improvementSuggestionsEnabled: true,

    pdfExportEnabled: true,
    shareLinkEnabled: true,
    apiAccess: false,
};

/** Default limits for Annual (Pro) tier */
const ANNUAL_LIMITS: TierLimits = {
    maxActiveSessions: null, // Unlimited
    maxCyclesPerSession: null, // Unlimited
    maxArchivedSessions: null, // Unlimited
    sessionHistoryDays: null, // Forever

    aiEnabled: true,
    aiMessagesPerDay: null, // Unlimited
    aiModelTier: 'advanced',

    dqComponentEnabled: true,

    fullTradeoffAnalysis: true,
    dqScoringEnabled: true,
    improvementSuggestionsEnabled: true,

    pdfExportEnabled: true,
    shareLinkEnabled: true,
    apiAccess: true,
};

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
            '2 cycles per session',
            'Full PrOACT framework (except DQ)',
            '50 AI messages per day',
            '90-day history retention',
        ],
        limits: FREE_LIMITS,
    },
    monthly: {
        tier: 'monthly',
        name: 'Premium',
        description: 'Full decision support for professionals',
        priceMonthly: PRICING.MONTHLY,
        billingPeriod: 'monthly',
        features: [
            '10 active sessions',
            '5 cycles per session',
            'Decision Quality component',
            '200 AI messages per day',
            'PDF export & sharing',
            '1-year history retention',
        ],
        limits: MONTHLY_LIMITS,
        recommended: true,
    },
    annual: {
        tier: 'annual',
        name: 'Pro',
        description: 'Best value for committed decision-makers',
        priceMonthly: PRICING.ANNUAL_MONTHLY_EQUIVALENT,
        priceAnnual: PRICING.ANNUAL,
        billingPeriod: 'yearly',
        features: [
            'Unlimited sessions & cycles',
            'Advanced AI model',
            'All Premium features',
            'API access',
            'Unlimited history',
            '2 months free',
        ],
        limits: ANNUAL_LIMITS,
    },
};

/**
 * Limits for users without a valid membership.
 * Fail-secure: no access to any features.
 */
export const NO_MEMBERSHIP_LIMITS: TierLimits = {
    maxActiveSessions: 0,
    maxCyclesPerSession: 0,
    maxArchivedSessions: 0,
    sessionHistoryDays: 0,

    aiEnabled: false,
    aiMessagesPerDay: 0,
    aiModelTier: 'standard',

    dqComponentEnabled: false,

    fullTradeoffAnalysis: false,
    dqScoringEnabled: false,
    improvementSuggestionsEnabled: false,

    pdfExportEnabled: false,
    shareLinkEnabled: false,
    apiAccess: false,
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

// ─── Limit Checking Utilities ──────────────────────────────────────────────

/** Check if user can create a new session */
export function canCreateSession(limits: TierLimits, currentActive: number): boolean {
    if (limits.maxActiveSessions === null) return true;
    return currentActive < limits.maxActiveSessions;
}

/** Check if user can create a new cycle */
export function canCreateCycle(limits: TierLimits, currentCycles: number): boolean {
    if (limits.maxCyclesPerSession === null) return true;
    return currentCycles < limits.maxCyclesPerSession;
}

/** Check if user can send an AI message */
export function canSendAiMessage(limits: TierLimits, messagesToday: number): boolean {
    if (!limits.aiEnabled) return false;
    if (limits.aiMessagesPerDay === null) return true;
    return messagesToday < limits.aiMessagesPerDay;
}

/** Check if user can access Decision Quality component */
export function canAccessDQ(limits: TierLimits): boolean {
    return limits.dqComponentEnabled;
}

/** Check if user can export to PDF */
export function canExportPDF(limits: TierLimits): boolean {
    return limits.pdfExportEnabled;
}

/** Check if user can create share links */
export function canShare(limits: TierLimits): boolean {
    return limits.shareLinkEnabled;
}

/** Calculate remaining AI messages for today */
export function aiMessagesRemaining(limits: TierLimits, messagesToday: number): number | null {
    if (limits.aiMessagesPerDay === null) return null; // Unlimited
    return Math.max(0, limits.aiMessagesPerDay - messagesToday);
}

/** Format limit value for display */
export function formatLimit(value: number | null): string {
    return value === null ? 'Unlimited' : value.toString();
}

/** Format history retention for display */
export function formatHistoryRetention(days: number | null): string {
    if (days === null) return 'Forever';
    if (days === 0) return 'None';
    if (days >= 365) {
        const years = Math.floor(days / 365);
        return years === 1 ? '1 year' : `${years} years`;
    }
    return `${days} days`;
}
