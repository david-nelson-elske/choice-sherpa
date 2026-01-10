/**
 * Membership API client.
 *
 * Provides functions for membership-related API calls.
 */

import { get, post } from './client';
import type {
    MembershipView,
    UsageStats,
    AccessResult,
    CreateCheckoutRequest,
    CheckoutSession,
    CreatePortalRequest,
    PortalSession,
    PromoCodeValidation,
    TierLimits,
    MembershipTier,
} from '../types';

/** API response wrapper */
interface ApiResponse<T> {
    ok: boolean;
    data?: T;
    error?: { code: string; message: string };
}

/**
 * Get current user's membership.
 */
export async function getMembership(): Promise<ApiResponse<MembershipView>> {
    const result = await get<MembershipView>('/membership/me');
    return result.ok
        ? { ok: true, data: result.data }
        : { ok: false, error: result.error };
}

/**
 * Get current user's usage statistics.
 */
export async function getUsage(): Promise<ApiResponse<UsageStats>> {
    const result = await get<UsageStats>('/membership/me/usage');
    return result.ok
        ? { ok: true, data: result.data }
        : { ok: false, error: result.error };
}

/**
 * Get current user's tier limits.
 */
export async function getTierLimits(): Promise<ApiResponse<TierLimits>> {
    const result = await get<TierLimits>('/membership/me/limits');
    return result.ok
        ? { ok: true, data: result.data }
        : { ok: false, error: result.error };
}

/**
 * Check if user can create a session.
 */
export async function checkCanCreateSession(): Promise<ApiResponse<AccessResult>> {
    const result = await get<AccessResult>('/membership/me/access/session');
    return result.ok
        ? { ok: true, data: result.data }
        : { ok: false, error: result.error };
}

/**
 * Check if user can create a cycle in a session.
 */
export async function checkCanCreateCycle(sessionId: string): Promise<ApiResponse<AccessResult>> {
    const result = await get<AccessResult>(`/membership/me/access/cycle/${sessionId}`);
    return result.ok
        ? { ok: true, data: result.data }
        : { ok: false, error: result.error };
}

/**
 * Check if user can export.
 */
export async function checkCanExport(): Promise<ApiResponse<AccessResult>> {
    const result = await get<AccessResult>('/membership/me/access/export');
    return result.ok
        ? { ok: true, data: result.data }
        : { ok: false, error: result.error };
}

/**
 * Create a Stripe checkout session.
 */
export async function createCheckout(
    tier: MembershipTier,
    options?: { promoCode?: string; successUrl?: string; cancelUrl?: string }
): Promise<ApiResponse<CheckoutSession>> {
    const request: CreateCheckoutRequest = {
        tier,
        promoCode: options?.promoCode,
        successUrl: options?.successUrl || `${window.location.origin}/membership/success`,
        cancelUrl: options?.cancelUrl || `${window.location.origin}/pricing`,
    };

    const result = await post<CheckoutSession>('/membership/checkout', request);
    return result.ok
        ? { ok: true, data: result.data }
        : { ok: false, error: result.error };
}

/**
 * Create a Stripe customer portal session.
 */
export async function createPortalSession(
    returnUrl?: string
): Promise<ApiResponse<PortalSession>> {
    const request: CreatePortalRequest = {
        returnUrl: returnUrl || `${window.location.origin}/account`,
    };

    const result = await post<PortalSession>('/membership/portal', request);
    return result.ok
        ? { ok: true, data: result.data }
        : { ok: false, error: result.error };
}

/**
 * Validate a promo code.
 */
export async function validatePromoCode(code: string): Promise<ApiResponse<PromoCodeValidation>> {
    const result = await get<PromoCodeValidation>(`/membership/promo/${encodeURIComponent(code)}`);
    return result.ok
        ? { ok: true, data: result.data }
        : { ok: false, error: result.error };
}

/**
 * Apply a promo code (for free tier upgrades).
 */
export async function applyPromoCode(code: string): Promise<ApiResponse<MembershipView>> {
    const result = await post<MembershipView>('/membership/promo/apply', { code });
    return result.ok
        ? { ok: true, data: result.data }
        : { ok: false, error: result.error };
}

/**
 * Redirect to Stripe checkout.
 */
export async function redirectToCheckout(
    tier: MembershipTier,
    options?: { promoCode?: string }
): Promise<void> {
    const response = await createCheckout(tier, options);

    if (response.ok && response.data) {
        window.location.href = response.data.url;
    } else {
        throw new Error(response.error?.message || 'Failed to create checkout session');
    }
}

/**
 * Redirect to Stripe customer portal.
 */
export async function redirectToPortal(): Promise<void> {
    const response = await createPortalSession();

    if (response.ok && response.data) {
        window.location.href = response.data.url;
    } else {
        throw new Error(response.error?.message || 'Failed to create portal session');
    }
}
