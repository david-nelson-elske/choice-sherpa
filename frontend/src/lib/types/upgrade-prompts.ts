/**
 * Upgrade prompt configuration for feature gating.
 *
 * Provides user-friendly messages when users attempt to access
 * features that require a higher tier.
 *
 * @module upgrade-prompts
 */

import type { MembershipTier } from './tier';

/** Configuration for an upgrade prompt */
export interface UpgradePromptConfig {
    /** Feature identifier (for tracking/analytics) */
    feature: string;
    /** Minimum tier required for this feature */
    requiredTier: MembershipTier;
    /** Short title for the prompt */
    title: string;
    /** Detailed message explaining the feature and upgrade benefit */
    message: string;
    /** Optional CTA button text */
    ctaText?: string;
}

/**
 * Upgrade prompts for gated features.
 *
 * Keys should match the feature check points in the application.
 */
export const UPGRADE_PROMPTS: Record<string, UpgradePromptConfig> = {
    // Component Access
    dq_component: {
        feature: 'Decision Quality',
        requiredTier: 'monthly',
        title: 'Decision Quality',
        message:
            'Upgrade to Premium to rate your decision quality across 7 key elements and get personalized improvement suggestions.',
        ctaText: 'Unlock Decision Quality',
    },

    // Export Features
    pdf_export: {
        feature: 'PDF Export',
        requiredTier: 'monthly',
        title: 'Export to PDF',
        message: 'Upgrade to Premium to export your decision analysis as a professional PDF document.',
        ctaText: 'Enable Export',
    },

    share_link: {
        feature: 'Share Link',
        requiredTier: 'monthly',
        title: 'Share Your Decision',
        message: 'Upgrade to Premium to create shareable read-only links to your decision analysis.',
        ctaText: 'Enable Sharing',
    },

    // Session Limits
    session_limit: {
        feature: 'More Sessions',
        requiredTier: 'monthly',
        title: 'Session Limit Reached',
        message:
            "You've reached your limit of 3 active sessions. Upgrade to Premium for 10 sessions, or Pro for unlimited.",
        ctaText: 'Get More Sessions',
    },

    session_limit_premium: {
        feature: 'Unlimited Sessions',
        requiredTier: 'annual',
        title: 'Session Limit Reached',
        message:
            "You've reached your limit of 10 active sessions. Upgrade to Pro for unlimited sessions.",
        ctaText: 'Go Unlimited',
    },

    // Cycle Limits
    cycle_limit: {
        feature: 'More Cycles',
        requiredTier: 'monthly',
        title: 'Cycle Limit Reached',
        message:
            "You've reached your limit of 2 cycles per session. Upgrade to Premium for 5 cycles, or Pro for unlimited.",
        ctaText: 'Get More Cycles',
    },

    cycle_limit_premium: {
        feature: 'Unlimited Cycles',
        requiredTier: 'annual',
        title: 'Cycle Limit Reached',
        message:
            "You've reached your limit of 5 cycles per session. Upgrade to Pro for unlimited cycles.",
        ctaText: 'Go Unlimited',
    },

    // AI Limits
    ai_limit: {
        feature: 'More AI Messages',
        requiredTier: 'monthly',
        title: 'Daily AI Limit Reached',
        message:
            "You've used all 50 AI messages for today. Upgrade to Premium for 200 messages/day, or Pro for unlimited.",
        ctaText: 'Get More Messages',
    },

    ai_limit_premium: {
        feature: 'Unlimited AI',
        requiredTier: 'annual',
        title: 'Daily AI Limit Reached',
        message:
            "You've used all 200 AI messages for today. Upgrade to Pro for unlimited AI conversations.",
        ctaText: 'Go Unlimited',
    },

    // Advanced AI
    advanced_ai: {
        feature: 'Advanced AI Model',
        requiredTier: 'annual',
        title: 'Advanced AI',
        message:
            'Upgrade to Pro to access our advanced AI model for deeper insights and more nuanced guidance.',
        ctaText: 'Unlock Advanced AI',
    },

    // Analysis Features
    full_tradeoff: {
        feature: 'Full Tradeoff Analysis',
        requiredTier: 'monthly',
        title: 'Full Tradeoff Analysis',
        message:
            'Upgrade to Premium for comprehensive tradeoff analysis including sensitivity testing and what-if scenarios.',
        ctaText: 'Enable Full Analysis',
    },

    improvement_suggestions: {
        feature: 'Improvement Suggestions',
        requiredTier: 'monthly',
        title: 'Improvement Suggestions',
        message:
            'Upgrade to Premium to receive AI-powered suggestions for improving your decision quality.',
        ctaText: 'Get Suggestions',
    },

    // API Access
    api_access: {
        feature: 'API Access',
        requiredTier: 'annual',
        title: 'API Access',
        message: 'Upgrade to Pro to access the Choice Sherpa API for integrations and automation.',
        ctaText: 'Enable API',
    },
};

/**
 * Get the upgrade prompt for a feature.
 *
 * @param featureKey - The key identifying the gated feature
 * @returns The upgrade prompt configuration, or undefined if not found
 */
export function getUpgradePrompt(featureKey: string): UpgradePromptConfig | undefined {
    return UPGRADE_PROMPTS[featureKey];
}

/**
 * Get the appropriate session limit prompt based on current tier.
 */
export function getSessionLimitPrompt(currentTier: MembershipTier): UpgradePromptConfig {
    return currentTier === 'monthly' ? UPGRADE_PROMPTS.session_limit_premium : UPGRADE_PROMPTS.session_limit;
}

/**
 * Get the appropriate cycle limit prompt based on current tier.
 */
export function getCycleLimitPrompt(currentTier: MembershipTier): UpgradePromptConfig {
    return currentTier === 'monthly' ? UPGRADE_PROMPTS.cycle_limit_premium : UPGRADE_PROMPTS.cycle_limit;
}

/**
 * Get the appropriate AI limit prompt based on current tier.
 */
export function getAiLimitPrompt(currentTier: MembershipTier): UpgradePromptConfig {
    return currentTier === 'monthly' ? UPGRADE_PROMPTS.ai_limit_premium : UPGRADE_PROMPTS.ai_limit;
}
