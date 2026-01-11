//! PostgreSQL implementation of AccessChecker.
//!
//! Provides database-backed access control based on membership status and usage.

use crate::domain::foundation::{DomainError, ErrorCode, SessionId, UserId};
use crate::domain::membership::{MembershipStatus, MembershipTier, TierLimits};
use crate::ports::{AccessChecker, AccessDeniedReason, AccessResult, UsageStats};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of the AccessChecker port.
///
/// Queries the database to determine user access based on membership status,
/// tier limits, and current usage.
pub struct PostgresAccessChecker {
    pool: PgPool,
}

impl PostgresAccessChecker {
    /// Creates a new PostgresAccessChecker with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Result of membership access query.
#[derive(Debug)]
struct MembershipAccess {
    tier: MembershipTier,
    status: MembershipStatus,
    has_access: bool,
}

fn parse_tier(s: &str) -> Result<MembershipTier, DomainError> {
    match s.to_lowercase().as_str() {
        "free" => Ok(MembershipTier::Free),
        "monthly" => Ok(MembershipTier::Monthly),
        "annual" => Ok(MembershipTier::Annual),
        _ => Err(DomainError::new(
            ErrorCode::DatabaseError,
            format!("Invalid tier value: {}", s),
        )),
    }
}

fn parse_status(s: &str) -> Result<MembershipStatus, DomainError> {
    match s.to_lowercase().as_str() {
        "pending" => Ok(MembershipStatus::Pending),
        "active" => Ok(MembershipStatus::Active),
        "past_due" => Ok(MembershipStatus::PastDue),
        "cancelled" => Ok(MembershipStatus::Cancelled),
        "expired" => Ok(MembershipStatus::Expired),
        _ => Err(DomainError::new(
            ErrorCode::DatabaseError,
            format!("Invalid status value: {}", s),
        )),
    }
}

fn parse_user_id_as_uuid(user_id: &UserId) -> Result<Uuid, DomainError> {
    Uuid::parse_str(user_id.as_str()).map_err(|e| {
        DomainError::new(
            ErrorCode::ValidationFailed,
            format!("User ID must be a valid UUID: {}", e),
        )
    })
}

impl PostgresAccessChecker {
    /// Get membership access info for a user.
    async fn get_membership_access(
        &self,
        user_id: &UserId,
    ) -> Result<Option<MembershipAccess>, DomainError> {
        let user_uuid = parse_user_id_as_uuid(user_id)?;
        let now = Utc::now();

        let row: Option<(String, String, Option<DateTime<Utc>>)> = sqlx::query_as(
            r#"
            SELECT tier, status, current_period_end
            FROM memberships
            WHERE user_id = $1
            "#,
        )
        .bind(user_uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Failed to check membership: {}", e),
            )
        })?;

        let Some((tier_str, status_str, period_end)) = row else {
            return Ok(None);
        };

        let tier = parse_tier(&tier_str)?;
        let status = parse_status(&status_str)?;

        // Calculate access based on status and period
        let has_access = if !status.has_access() {
            false
        } else if status == MembershipStatus::Cancelled {
            // Cancelled memberships have access until period end
            period_end.is_some_and(|end| now <= end)
        } else {
            true
        };

        Ok(Some(MembershipAccess {
            tier,
            status,
            has_access,
        }))
    }

    /// Count active sessions for a user.
    ///
    /// Note: Returns 0 until sessions table is implemented.
    async fn count_active_sessions(&self, user_id: &UserId) -> Result<u32, DomainError> {
        let user_uuid = parse_user_id_as_uuid(user_id)?;

        // Check if sessions table exists and query it
        // For now, returns 0 as sessions table may not exist yet
        let count: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM sessions
            WHERE user_id = $1 AND status != 'archived'
            "#,
        )
        .bind(user_uuid)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None); // Ignore errors (table may not exist)

        Ok(count.map_or(0, |(c,)| c as u32))
    }

    /// Count cycles for a specific session.
    ///
    /// Note: Returns 0 until cycles table is implemented.
    async fn count_session_cycles(&self, session_id: &SessionId) -> Result<u32, DomainError> {
        // Check if cycles table exists and query it
        // For now, returns 0 as cycles table may not exist yet
        let count: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM cycles
            WHERE session_id = $1
            "#,
        )
        .bind(session_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None); // Ignore errors (table may not exist)

        Ok(count.map_or(0, |(c,)| c as u32))
    }

    /// Count total cycles across all sessions for a user.
    ///
    /// Note: Returns 0 until cycles table is implemented.
    async fn count_total_cycles(&self, user_id: &UserId) -> Result<u32, DomainError> {
        let user_uuid = parse_user_id_as_uuid(user_id)?;

        // Check if cycles/sessions tables exist and query them
        // For now, returns 0 as tables may not exist yet
        let count: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM cycles c
            JOIN sessions s ON c.session_id = s.id
            WHERE s.user_id = $1
            "#,
        )
        .bind(user_uuid)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None); // Ignore errors (tables may not exist)

        Ok(count.map_or(0, |(c,)| c as u32))
    }
}

#[async_trait]
impl AccessChecker for PostgresAccessChecker {
    async fn can_create_session(&self, user_id: &UserId) -> Result<AccessResult, DomainError> {
        // Check membership exists and has access
        let Some(membership) = self.get_membership_access(user_id).await? else {
            return Ok(AccessResult::Denied(AccessDeniedReason::NoMembership));
        };

        if !membership.has_access {
            return Ok(match membership.status {
                MembershipStatus::Expired => {
                    AccessResult::Denied(AccessDeniedReason::MembershipExpired)
                }
                MembershipStatus::PastDue => {
                    AccessResult::Denied(AccessDeniedReason::MembershipPastDue)
                }
                MembershipStatus::Cancelled => {
                    AccessResult::Denied(AccessDeniedReason::MembershipExpired)
                }
                _ => AccessResult::Denied(AccessDeniedReason::NoMembership),
            });
        }

        // Check session limits
        let limits = TierLimits::for_tier(membership.tier);
        let active_sessions = self.count_active_sessions(user_id).await?;

        if !limits.can_create_session(active_sessions) {
            return Ok(AccessResult::Denied(AccessDeniedReason::SessionLimitReached {
                current: active_sessions,
                max: limits.max_active_sessions.unwrap_or(0),
            }));
        }

        Ok(AccessResult::Allowed)
    }

    async fn can_create_cycle(
        &self,
        user_id: &UserId,
        session_id: &SessionId,
    ) -> Result<AccessResult, DomainError> {
        // Check membership exists and has access
        let Some(membership) = self.get_membership_access(user_id).await? else {
            return Ok(AccessResult::Denied(AccessDeniedReason::NoMembership));
        };

        if !membership.has_access {
            return Ok(match membership.status {
                MembershipStatus::Expired => {
                    AccessResult::Denied(AccessDeniedReason::MembershipExpired)
                }
                MembershipStatus::PastDue => {
                    AccessResult::Denied(AccessDeniedReason::MembershipPastDue)
                }
                _ => AccessResult::Denied(AccessDeniedReason::NoMembership),
            });
        }

        // Check cycle limits for this session
        let limits = TierLimits::for_tier(membership.tier);
        let session_cycles = self.count_session_cycles(session_id).await?;

        if !limits.can_create_cycle(session_cycles) {
            return Ok(AccessResult::Denied(AccessDeniedReason::CycleLimitReached {
                current: session_cycles,
                max: limits.max_cycles_per_session.unwrap_or(0),
            }));
        }

        Ok(AccessResult::Allowed)
    }

    async fn can_export(&self, user_id: &UserId) -> Result<AccessResult, DomainError> {
        // Check membership exists and has access
        let Some(membership) = self.get_membership_access(user_id).await? else {
            return Ok(AccessResult::Denied(AccessDeniedReason::NoMembership));
        };

        if !membership.has_access {
            return Ok(AccessResult::Denied(AccessDeniedReason::MembershipExpired));
        }

        // Check if tier allows export
        let limits = TierLimits::for_tier(membership.tier);
        if !limits.can_export_pdf() {
            return Ok(AccessResult::Denied(AccessDeniedReason::FeatureNotIncluded {
                feature: "Export".to_string(),
                required_tier: MembershipTier::Monthly,
            }));
        }

        Ok(AccessResult::Allowed)
    }

    async fn get_tier_limits(&self, user_id: &UserId) -> Result<TierLimits, DomainError> {
        let membership = self.get_membership_access(user_id).await?;

        let tier = membership.map_or(MembershipTier::Free, |m| m.tier);
        Ok(TierLimits::for_tier(tier))
    }

    async fn get_usage(&self, user_id: &UserId) -> Result<UsageStats, DomainError> {
        let active_sessions = self.count_active_sessions(user_id).await?;
        let total_cycles = self.count_total_cycles(user_id).await?;

        // Export count tracking not implemented yet - would need separate table
        let exports_this_month = 0;

        Ok(UsageStats {
            active_sessions,
            total_cycles,
            exports_this_month,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tier_all_values() {
        assert_eq!(parse_tier("free").unwrap(), MembershipTier::Free);
        assert_eq!(parse_tier("monthly").unwrap(), MembershipTier::Monthly);
        assert_eq!(parse_tier("annual").unwrap(), MembershipTier::Annual);
    }

    #[test]
    fn parse_tier_case_insensitive() {
        assert_eq!(parse_tier("FREE").unwrap(), MembershipTier::Free);
        assert_eq!(parse_tier("Monthly").unwrap(), MembershipTier::Monthly);
        assert_eq!(parse_tier("ANNUAL").unwrap(), MembershipTier::Annual);
    }

    #[test]
    fn parse_tier_rejects_invalid() {
        assert!(parse_tier("invalid").is_err());
        assert!(parse_tier("").is_err());
        assert!(parse_tier("premium").is_err());
    }

    #[test]
    fn parse_status_all_values() {
        assert_eq!(parse_status("pending").unwrap(), MembershipStatus::Pending);
        assert_eq!(parse_status("active").unwrap(), MembershipStatus::Active);
        assert_eq!(parse_status("past_due").unwrap(), MembershipStatus::PastDue);
        assert_eq!(parse_status("cancelled").unwrap(), MembershipStatus::Cancelled);
        assert_eq!(parse_status("expired").unwrap(), MembershipStatus::Expired);
    }

    #[test]
    fn parse_status_case_insensitive() {
        assert_eq!(parse_status("ACTIVE").unwrap(), MembershipStatus::Active);
        assert_eq!(parse_status("Pending").unwrap(), MembershipStatus::Pending);
    }

    #[test]
    fn parse_status_rejects_invalid() {
        assert!(parse_status("invalid").is_err());
        assert!(parse_status("").is_err());
        assert!(parse_status("unknown").is_err());
    }

    #[test]
    fn parse_user_id_accepts_valid_uuid() {
        let user_id = UserId::new("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert!(parse_user_id_as_uuid(&user_id).is_ok());
    }

    #[test]
    fn parse_user_id_rejects_invalid_uuid() {
        let user_id = UserId::new("not-a-valid-uuid").unwrap();
        assert!(parse_user_id_as_uuid(&user_id).is_err());
    }

    #[test]
    fn membership_access_derives_debug() {
        let access = MembershipAccess {
            tier: MembershipTier::Monthly,
            status: MembershipStatus::Active,
            has_access: true,
        };
        let debug_str = format!("{:?}", access);
        assert!(debug_str.contains("Monthly"));
        assert!(debug_str.contains("Active"));
    }
}
