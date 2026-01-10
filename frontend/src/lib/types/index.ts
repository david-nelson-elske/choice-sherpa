/**
 * Type exports for the Choice Sherpa frontend.
 */

// Money types
export {
    type Cents,
    cents,
    dollarsToCents,
    centsToDollars,
    formatMoney,
    formatMoneyShort,
    PRICING,
    calculateAnnualSavings,
    calculateAnnualSavingsPercent,
} from './money';

// Tier types
export {
    type MembershipTier,
    type MembershipStatus,
    type TierInfo,
    type TierLimits,
    TIERS,
    getTierInfo,
    getTierLimits,
    formatTierPrice,
    formatMonthlyEquivalent,
    isPaidTier,
    statusHasAccess,
    formatStatus,
    getStatusColor,
    compareTiers,
    canUpgradeTo,
} from './tier';

// Membership types
export {
    type MembershipId,
    type UserId,
    membershipId,
    userId,
    type MembershipView,
    type MembershipSummary,
    type MembershipState,
    type UsageStats,
    type AccessResult,
    type AccessDeniedReason,
    type CreateCheckoutRequest,
    type CheckoutSession,
    type CreatePortalRequest,
    type PortalSession,
    type PromoCodeValidation,
    type MembershipStatistics,
    hasActiveMembership,
    isExpiringSoon,
    formatDaysRemaining,
    getAccessDeniedMessage,
} from './membership';

// WebSocket types
export {
    type WebSocketMessage,
    type ConnectedMessage,
    type DashboardUpdateMessage,
    type DashboardUpdateType,
    type ErrorMessage,
    type PongMessage,
    type ServerMessage,
    type ClientPing,
    type RequestStateMessage,
    type ClientMessage,
    type ComponentCompletedData,
    type ProgressInfo,
    type ConversationMessageData,
    type MessagePreview,
    type AnalysisScoresData,
    type WebSocketConnectionState,
    type UseDashboardLiveOptions,
    INITIAL_CONNECTION_STATE,
} from './websocket';
