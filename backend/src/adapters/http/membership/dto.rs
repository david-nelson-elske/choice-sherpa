//! HTTP DTOs (Data Transfer Objects) for membership endpoints.
//!
//! These types define the JSON request/response structure for the membership API.
//! They serve as the boundary between HTTP and the application layer.

use crate::domain::foundation::{MembershipId, UserId};
use crate::domain::membership::{MembershipStatus, MembershipTier, TierLimits};
use crate::ports::{MembershipStatistics, MembershipView};
use serde::{Deserialize, Serialize};

// ════════════════════════════════════════════════════════════════════════════════
// Request DTOs
// ════════════════════════════════════════════════════════════════════════════════

/// Request to create a free membership with promo code.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateFreeMembershipRequest {
    /// The promo code for free tier access.
    pub promo_code: String,
}

/// Request to initiate paid membership checkout.
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePaidMembershipRequest {
    /// User's email for Stripe customer.
    pub email: String,
    /// The tier to subscribe to (monthly or annual).
    pub tier: MembershipTier,
    /// URL to redirect after successful checkout.
    pub success_url: String,
    /// URL to redirect after cancelled checkout.
    pub cancel_url: String,
    /// Optional promo code for discount.
    #[serde(default)]
    pub promo_code: Option<String>,
}

/// Request to cancel a membership.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelMembershipRequest {
    /// Whether to cancel immediately or at period end.
    #[serde(default)]
    pub immediate: bool,
}

// ════════════════════════════════════════════════════════════════════════════════
// Response DTOs
// ════════════════════════════════════════════════════════════════════════════════

/// Response for membership details.
#[derive(Debug, Clone, Serialize)]
pub struct MembershipResponse {
    /// The membership details, or null if none exists.
    #[serde(flatten)]
    pub membership: Option<MembershipViewResponse>,
}

/// Detailed membership view for API response.
#[derive(Debug, Clone, Serialize)]
pub struct MembershipViewResponse {
    /// Membership ID.
    pub id: String,
    /// User ID.
    pub user_id: String,
    /// Subscription tier.
    pub tier: MembershipTier,
    /// Current status.
    pub status: MembershipStatus,
    /// Whether user currently has access.
    pub has_access: bool,
    /// Days remaining in current period.
    pub days_remaining: u32,
    /// End of current billing period (ISO 8601).
    pub period_end: String,
    /// Promo code used (if any).
    pub promo_code: Option<String>,
    /// When the membership was created (ISO 8601).
    pub created_at: String,
}

impl From<MembershipView> for MembershipViewResponse {
    fn from(view: MembershipView) -> Self {
        Self {
            id: view.id.to_string(),
            user_id: view.user_id.to_string(),
            tier: view.tier,
            status: view.status,
            has_access: view.has_access,
            days_remaining: view.days_remaining,
            period_end: view.period_end.as_datetime().to_rfc3339(),
            promo_code: view.promo_code,
            created_at: view.created_at.as_datetime().to_rfc3339(),
        }
    }
}

/// Response for tier limits.
#[derive(Debug, Clone, Serialize)]
pub struct TierLimitsResponse {
    /// The membership tier.
    pub tier: MembershipTier,
    /// Maximum active sessions (null = unlimited).
    pub max_sessions: Option<u32>,
    /// Maximum cycles per session (null = unlimited).
    pub max_cycles_per_session: Option<u32>,
    /// Whether PDF/CSV export is enabled.
    pub export_enabled: bool,
    /// Whether API access is enabled.
    pub api_access: bool,
}

impl From<TierLimits> for TierLimitsResponse {
    fn from(limits: TierLimits) -> Self {
        Self {
            tier: limits.tier,
            max_sessions: limits.max_sessions,
            max_cycles_per_session: limits.max_cycles_per_session,
            export_enabled: limits.export_enabled,
            api_access: limits.api_access,
        }
    }
}

/// Response for access check.
#[derive(Debug, Clone, Serialize)]
pub struct AccessCheckResponse {
    /// Whether the user has access.
    pub has_access: bool,
}

/// Response for checkout initiation.
#[derive(Debug, Clone, Serialize)]
pub struct CheckoutResponse {
    /// The Stripe checkout session URL.
    pub checkout_url: String,
}

/// Response for customer portal.
#[derive(Debug, Clone, Serialize)]
pub struct PortalResponse {
    /// The Stripe customer portal URL.
    pub portal_url: String,
}

/// Response for membership statistics (admin).
#[derive(Debug, Clone, Serialize)]
pub struct MembershipStatsResponse {
    /// Total number of memberships.
    pub total_count: u64,
    /// Number of active memberships.
    pub active_count: u64,
    /// Count by tier.
    pub by_tier: TierCountsResponse,
    /// Count by status.
    pub by_status: StatusCountsResponse,
    /// Monthly recurring revenue in cents.
    pub monthly_recurring_revenue_cents: i64,
}

/// Tier counts for stats response.
#[derive(Debug, Clone, Serialize)]
pub struct TierCountsResponse {
    pub free: u64,
    pub monthly: u64,
    pub annual: u64,
}

/// Status counts for stats response.
#[derive(Debug, Clone, Serialize)]
pub struct StatusCountsResponse {
    pub pending: u64,
    pub active: u64,
    pub past_due: u64,
    pub cancelled: u64,
    pub expired: u64,
}

impl From<MembershipStatistics> for MembershipStatsResponse {
    fn from(stats: MembershipStatistics) -> Self {
        Self {
            total_count: stats.total_count,
            active_count: stats.active_count,
            by_tier: TierCountsResponse {
                free: stats.by_tier.free,
                monthly: stats.by_tier.monthly,
                annual: stats.by_tier.annual,
            },
            by_status: StatusCountsResponse {
                pending: stats.by_status.pending,
                active: stats.by_status.active,
                past_due: stats.by_status.past_due,
                cancelled: stats.by_status.cancelled,
                expired: stats.by_status.expired,
            },
            monthly_recurring_revenue_cents: stats.monthly_recurring_revenue_cents,
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Error Response DTO
// ════════════════════════════════════════════════════════════════════════════════

/// Standard error response for API errors.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    /// Error code for programmatic handling.
    pub error_code: String,
    /// Human-readable error message.
    pub message: String,
    /// Additional details (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    /// Create a new error response.
    pub fn new(error_code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error_code: error_code.into(),
            message: message.into(),
            details: None,
        }
    }

    /// Create an error response with details.
    pub fn with_details(
        error_code: impl Into<String>,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            error_code: error_code.into(),
            message: message.into(),
            details: Some(details),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::Timestamp;
    use crate::ports::{StatusCounts, TierCounts};

    // ════════════════════════════════════════════════════════════════════════════
    // Request DTO Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn create_free_membership_request_deserializes() {
        let json = r#"{"promo_code": "WORKSHOP2026-XYZ"}"#;
        let request: CreateFreeMembershipRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.promo_code, "WORKSHOP2026-XYZ");
    }

    #[test]
    fn create_paid_membership_request_deserializes() {
        let json = r#"{
            "email": "user@example.com",
            "tier": "monthly",
            "success_url": "https://example.com/success",
            "cancel_url": "https://example.com/cancel"
        }"#;
        let request: CreatePaidMembershipRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.email, "user@example.com");
        assert_eq!(request.tier, MembershipTier::Monthly);
        assert_eq!(request.success_url, "https://example.com/success");
        assert!(request.promo_code.is_none());
    }

    #[test]
    fn create_paid_membership_request_with_promo_code() {
        let json = r#"{
            "email": "user@example.com",
            "tier": "annual",
            "success_url": "https://example.com/success",
            "cancel_url": "https://example.com/cancel",
            "promo_code": "SAVE10"
        }"#;
        let request: CreatePaidMembershipRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.promo_code, Some("SAVE10".to_string()));
    }

    #[test]
    fn cancel_membership_request_defaults_immediate_to_false() {
        let json = r#"{}"#;
        let request: CancelMembershipRequest = serde_json::from_str(json).unwrap();
        assert!(!request.immediate);
    }

    #[test]
    fn cancel_membership_request_parses_immediate() {
        let json = r#"{"immediate": true}"#;
        let request: CancelMembershipRequest = serde_json::from_str(json).unwrap();
        assert!(request.immediate);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Response DTO Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn membership_view_response_from_view() {
        let view = MembershipView {
            id: MembershipId::new(),
            user_id: UserId::new("user-123").unwrap(),
            tier: MembershipTier::Annual,
            status: MembershipStatus::Active,
            has_access: true,
            days_remaining: 300,
            period_end: Timestamp::now(),
            promo_code: Some("PROMO".to_string()),
            created_at: Timestamp::now(),
        };

        let response = MembershipViewResponse::from(view.clone());
        assert_eq!(response.id, view.id.to_string());
        assert_eq!(response.tier, MembershipTier::Annual);
        assert!(response.has_access);
    }

    #[test]
    fn tier_limits_response_from_limits() {
        let limits = TierLimits::for_tier(MembershipTier::Free);
        let response = TierLimitsResponse::from(limits);

        assert_eq!(response.tier, MembershipTier::Free);
        assert_eq!(response.max_sessions, Some(3));
        assert!(!response.export_enabled);
    }

    #[test]
    fn tier_limits_response_serializes_null_for_unlimited() {
        let limits = TierLimits::for_tier(MembershipTier::Annual);
        let response = TierLimitsResponse::from(limits);

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains(r#""max_sessions":null"#));
    }

    #[test]
    fn access_check_response_serializes() {
        let response = AccessCheckResponse { has_access: true };
        let json = serde_json::to_string(&response).unwrap();
        assert_eq!(json, r#"{"has_access":true}"#);
    }

    #[test]
    fn membership_stats_response_from_statistics() {
        let stats = MembershipStatistics {
            total_count: 100,
            active_count: 80,
            by_tier: TierCounts {
                free: 20,
                monthly: 50,
                annual: 30,
            },
            by_status: StatusCounts {
                pending: 5,
                active: 80,
                past_due: 3,
                cancelled: 7,
                expired: 5,
            },
            monthly_recurring_revenue_cents: 150000,
        };

        let response = MembershipStatsResponse::from(stats);
        assert_eq!(response.total_count, 100);
        assert_eq!(response.by_tier.monthly, 50);
        assert_eq!(response.monthly_recurring_revenue_cents, 150000);
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Error Response Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn error_response_new_creates_response() {
        let response = ErrorResponse::new("VALIDATION_FAILED", "Invalid promo code");
        assert_eq!(response.error_code, "VALIDATION_FAILED");
        assert_eq!(response.message, "Invalid promo code");
        assert!(response.details.is_none());
    }

    #[test]
    fn error_response_with_details_includes_details() {
        let details = serde_json::json!({"field": "promo_code"});
        let response = ErrorResponse::with_details("VALIDATION_FAILED", "Invalid", details.clone());
        assert_eq!(response.details, Some(details));
    }

    #[test]
    fn error_response_serializes_without_details_when_none() {
        let response = ErrorResponse::new("NOT_FOUND", "Not found");
        let json = serde_json::to_string(&response).unwrap();
        assert!(!json.contains("details"));
    }

    #[test]
    fn error_response_serializes_with_details_when_present() {
        let details = serde_json::json!({"id": "123"});
        let response = ErrorResponse::with_details("NOT_FOUND", "Not found", details);
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("details"));
    }
}
