/**
 * Membership domain types.
 *
 * Mirrors the backend Membership aggregate and related types.
 */

import type { MembershipTier, MembershipStatus, TierLimits } from './tier';

/** Branded type for MembershipId */
export type MembershipId = string & { readonly _brand: 'MembershipId' };

/** Branded type for UserId */
export type UserId = string & { readonly _brand: 'UserId' };

/** Create a MembershipId from a string */
export function membershipId(id: string): MembershipId {
    return id as MembershipId;
}

/** Create a UserId from a string */
export function userId(id: string): UserId {
    return id as UserId;
}

/**
 * Membership view - read model for displaying membership info.
 *
 * This is what the API returns for membership queries.
 */
export interface MembershipView {
    id: MembershipId;
    userId: UserId;
    tier: MembershipTier;
    status: MembershipStatus;
    hasAccess: boolean;
    daysRemaining: number;
    periodEnd: string; // ISO 8601 timestamp
    promoCode: string | null;
    createdAt: string; // ISO 8601 timestamp
}

/**
 * Membership summary - lightweight view for lists.
 */
export interface MembershipSummary {
    id: MembershipId;
    userId: UserId;
    tier: MembershipTier;
    status: MembershipStatus;
    periodEnd: string;
}

/**
 * Current user's membership state.
 *
 * Used by the membership store.
 */
export interface MembershipState {
    membership: MembershipView | null;
    limits: TierLimits;
    loading: boolean;
    error: string | null;
}

/**
 * Usage statistics for the current user.
 */
export interface UsageStats {
    activeSessions: number;
    totalCycles: number;
    exportsThisMonth: number;
}

/**
 * Access check result.
 */
export interface AccessResult {
    allowed: boolean;
    reason?: AccessDeniedReason;
}

/** Reasons why access might be denied */
export type AccessDeniedReason =
    | { type: 'no_membership' }
    | { type: 'membership_expired' }
    | { type: 'membership_past_due' }
    | { type: 'session_limit_reached'; current: number; max: number }
    | { type: 'cycle_limit_reached'; current: number; max: number }
    | { type: 'feature_not_included'; feature: string; requiredTier: MembershipTier };

/**
 * Checkout session request.
 */
export interface CreateCheckoutRequest {
    tier: MembershipTier;
    promoCode?: string;
    successUrl: string;
    cancelUrl: string;
}

/**
 * Checkout session response.
 */
export interface CheckoutSession {
    sessionId: string;
    url: string;
}

/**
 * Customer portal request.
 */
export interface CreatePortalRequest {
    returnUrl: string;
}

/**
 * Customer portal response.
 */
export interface PortalSession {
    url: string;
}

/**
 * Promo code validation result.
 */
export interface PromoCodeValidation {
    valid: boolean;
    code: string;
    tier?: MembershipTier;
    description?: string;
    error?: string;
}

/**
 * Membership statistics (admin view).
 */
export interface MembershipStatistics {
    totalCount: number;
    activeCount: number;
    byTier: {
        free: number;
        monthly: number;
        annual: number;
    };
    byStatus: {
        pending: number;
        active: number;
        pastDue: number;
        cancelled: number;
        expired: number;
    };
    monthlyRecurringRevenueCents: number;
}

/** Helper to check if user has active membership */
export function hasActiveMembership(membership: MembershipView | null): boolean {
    if (!membership) return false;
    return membership.hasAccess;
}

/** Helper to check if membership is expiring soon */
export function isExpiringSoon(membership: MembershipView | null, days: number = 7): boolean {
    if (!membership) return false;
    return membership.daysRemaining > 0 && membership.daysRemaining <= days;
}

/** Helper to format remaining days */
export function formatDaysRemaining(days: number): string {
    if (days === 0) return 'Expires today';
    if (days === 1) return '1 day remaining';
    if (days < 0) return 'Expired';
    return `${days} days remaining`;
}

/** Get access denied message for user display */
export function getAccessDeniedMessage(reason: AccessDeniedReason): string {
    switch (reason.type) {
        case 'no_membership':
            return 'You need a membership to access this feature.';
        case 'membership_expired':
            return 'Your membership has expired. Please renew to continue.';
        case 'membership_past_due':
            return 'Your payment is past due. Please update your payment method.';
        case 'session_limit_reached':
            return `You've reached your session limit (${reason.current}/${reason.max}). Upgrade for unlimited sessions.`;
        case 'cycle_limit_reached':
            return `You've reached your cycle limit (${reason.current}/${reason.max}). Upgrade for unlimited cycles.`;
        case 'feature_not_included':
            return `${reason.feature} requires a ${reason.requiredTier} plan or higher.`;
    }
}
