//! PostgreSQL implementation of MembershipReader.
//!
//! Provides read-optimized queries for membership data.

use crate::domain::foundation::{DomainError, ErrorCode, MembershipId, Timestamp, UserId};
use crate::domain::membership::{MembershipStatus, MembershipTier};
use crate::ports::{
    MembershipReader, MembershipStatistics, MembershipSummary, MembershipView, StatusCounts,
    TierCounts,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of the MembershipReader port.
///
/// Provides read-optimized queries for membership views and statistics.
pub struct PostgresMembershipReader {
    pool: PgPool,
}

impl PostgresMembershipReader {
    /// Creates a new PostgresMembershipReader with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Row for full membership view queries.
#[derive(Debug, sqlx::FromRow)]
struct MembershipViewRow {
    id: Uuid,
    user_id: Uuid,
    tier: String,
    status: String,
    current_period_end: Option<DateTime<Utc>>,
    promo_code: Option<String>,
    created_at: DateTime<Utc>,
}

/// Row for membership summary queries.
#[derive(Debug, sqlx::FromRow)]
struct MembershipSummaryRow {
    id: Uuid,
    user_id: Uuid,
    tier: String,
    status: String,
    current_period_end: Option<DateTime<Utc>>,
}

/// Row for tier statistics query.
#[derive(Debug, sqlx::FromRow)]
struct TierCountRow {
    tier: String,
    count: i64,
}

/// Row for status statistics query.
#[derive(Debug, sqlx::FromRow)]
struct StatusCountRow {
    status: String,
    count: i64,
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

fn calculate_days_remaining(period_end: Option<DateTime<Utc>>) -> u32 {
    let Some(end) = period_end else {
        return 0;
    };

    let now = Utc::now();
    if now >= end {
        return 0;
    }

    let duration = end.signed_duration_since(now);
    duration.num_days().max(0) as u32
}

fn calculate_has_access(status: &MembershipStatus, period_end: Option<DateTime<Utc>>) -> bool {
    if !status.has_access() {
        return false;
    }

    // For cancelled status, check if period has ended
    if *status == MembershipStatus::Cancelled {
        if let Some(end) = period_end {
            return Utc::now() <= end;
        }
        return false;
    }

    true
}

impl TryFrom<MembershipViewRow> for MembershipView {
    type Error = DomainError;

    fn try_from(row: MembershipViewRow) -> Result<Self, Self::Error> {
        let tier = parse_tier(&row.tier)?;
        let status = parse_status(&row.status)?;
        let days_remaining = calculate_days_remaining(row.current_period_end);
        let has_access = calculate_has_access(&status, row.current_period_end);

        Ok(MembershipView {
            id: MembershipId::from_uuid(row.id),
            user_id: UserId::new(row.user_id.to_string()).map_err(|e| {
                DomainError::new(ErrorCode::DatabaseError, format!("Invalid user_id: {}", e))
            })?,
            tier,
            status,
            has_access,
            days_remaining,
            period_end: row
                .current_period_end
                .map(Timestamp::from_datetime)
                .unwrap_or_else(Timestamp::now),
            promo_code: row.promo_code,
            created_at: Timestamp::from_datetime(row.created_at),
        })
    }
}

impl TryFrom<MembershipSummaryRow> for MembershipSummary {
    type Error = DomainError;

    fn try_from(row: MembershipSummaryRow) -> Result<Self, Self::Error> {
        let tier = parse_tier(&row.tier)?;
        let status = parse_status(&row.status)?;

        Ok(MembershipSummary {
            id: MembershipId::from_uuid(row.id),
            user_id: UserId::new(row.user_id.to_string()).map_err(|e| {
                DomainError::new(ErrorCode::DatabaseError, format!("Invalid user_id: {}", e))
            })?,
            tier,
            status,
            period_end: row
                .current_period_end
                .map(Timestamp::from_datetime)
                .unwrap_or_else(Timestamp::now),
        })
    }
}

#[async_trait]
impl MembershipReader for PostgresMembershipReader {
    async fn get_by_user(&self, user_id: &UserId) -> Result<Option<MembershipView>, DomainError> {
        let user_uuid = parse_user_id_as_uuid(user_id)?;

        let row: Option<MembershipViewRow> = sqlx::query_as(
            r#"
            SELECT id, user_id, tier, status, current_period_end, promo_code, created_at
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
                format!("Failed to get membership: {}", e),
            )
        })?;

        row.map(MembershipView::try_from).transpose()
    }

    async fn check_access(&self, user_id: &UserId) -> Result<bool, DomainError> {
        let user_uuid = parse_user_id_as_uuid(user_id)?;
        let now = Utc::now();

        // Single query to check access status
        let row: Option<(String, Option<DateTime<Utc>>)> = sqlx::query_as(
            r#"
            SELECT status, current_period_end
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
                format!("Failed to check access: {}", e),
            )
        })?;

        let Some((status_str, period_end)) = row else {
            return Ok(false);
        };

        let status = parse_status(&status_str)?;

        // Check if status grants access
        if !status.has_access() {
            return Ok(false);
        }

        // For cancelled status, verify period hasn't ended
        if status == MembershipStatus::Cancelled {
            if let Some(end) = period_end {
                return Ok(now <= end);
            }
            return Ok(false);
        }

        Ok(true)
    }

    async fn get_tier(&self, user_id: &UserId) -> Result<Option<MembershipTier>, DomainError> {
        let user_uuid = parse_user_id_as_uuid(user_id)?;

        let row: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT tier
            FROM memberships
            WHERE user_id = $1
            "#,
        )
        .bind(user_uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(ErrorCode::DatabaseError, format!("Failed to get tier: {}", e))
        })?;

        row.map(|(tier_str,)| parse_tier(&tier_str)).transpose()
    }

    async fn list_expiring(&self, days: u32) -> Result<Vec<MembershipSummary>, DomainError> {
        let now = Utc::now();
        let expiry_threshold = now + chrono::Duration::days(i64::from(days));

        let rows: Vec<MembershipSummaryRow> = sqlx::query_as(
            r#"
            SELECT id, user_id, tier, status, current_period_end
            FROM memberships
            WHERE status IN ('active', 'cancelled')
              AND current_period_end IS NOT NULL
              AND current_period_end > $1
              AND current_period_end <= $2
            ORDER BY current_period_end ASC
            "#,
        )
        .bind(now)
        .bind(expiry_threshold)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Failed to list expiring memberships: {}", e),
            )
        })?;

        rows.into_iter().map(MembershipSummary::try_from).collect()
    }

    async fn get_statistics(&self) -> Result<MembershipStatistics, DomainError> {
        // Get total and active counts
        let (total_count, active_count): (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status IN ('active', 'past_due', 'cancelled')) as active
            FROM memberships
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Failed to get membership counts: {}", e),
            )
        })?;

        // Get counts by tier
        let tier_rows: Vec<TierCountRow> = sqlx::query_as(
            r#"
            SELECT tier, COUNT(*) as count
            FROM memberships
            GROUP BY tier
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Failed to get tier counts: {}", e),
            )
        })?;

        let mut by_tier = TierCounts::default();
        for row in tier_rows {
            match row.tier.to_lowercase().as_str() {
                "free" => by_tier.free = row.count as u64,
                "monthly" => by_tier.monthly = row.count as u64,
                "annual" => by_tier.annual = row.count as u64,
                _ => {}
            }
        }

        // Get counts by status
        let status_rows: Vec<StatusCountRow> = sqlx::query_as(
            r#"
            SELECT status, COUNT(*) as count
            FROM memberships
            GROUP BY status
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(
                ErrorCode::DatabaseError,
                format!("Failed to get status counts: {}", e),
            )
        })?;

        let mut by_status = StatusCounts::default();
        for row in status_rows {
            match row.status.to_lowercase().as_str() {
                "pending" => by_status.pending = row.count as u64,
                "active" => by_status.active = row.count as u64,
                "past_due" => by_status.past_due = row.count as u64,
                "cancelled" => by_status.cancelled = row.count as u64,
                "expired" => by_status.expired = row.count as u64,
                _ => {}
            }
        }

        // Calculate MRR (Monthly Recurring Revenue)
        // Pricing in cents (CAD):
        // - Monthly: $19.99 = 1999 cents
        // - Annual: $149.99 = 14999 cents, monthly equivalent = 14999 / 12 = 1249 cents
        const MONTHLY_PRICE_CENTS: i64 = 1999;
        const ANNUAL_MONTHLY_EQUIVALENT_CENTS: i64 = 14999 / 12; // ~1249 cents

        let mrr = (by_tier.monthly as i64 * MONTHLY_PRICE_CENTS)
            + (by_tier.annual as i64 * ANNUAL_MONTHLY_EQUIVALENT_CENTS);

        Ok(MembershipStatistics {
            total_count: total_count as u64,
            active_count: active_count as u64,
            by_tier,
            by_status,
            monthly_recurring_revenue_cents: mrr,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_days_remaining_returns_zero_for_none() {
        assert_eq!(calculate_days_remaining(None), 0);
    }

    #[test]
    fn calculate_days_remaining_returns_zero_for_past() {
        let past = Utc::now() - chrono::Duration::days(5);
        assert_eq!(calculate_days_remaining(Some(past)), 0);
    }

    #[test]
    fn calculate_days_remaining_returns_days_for_future() {
        let future = Utc::now() + chrono::Duration::days(10);
        let days = calculate_days_remaining(Some(future));
        assert!(days >= 9 && days <= 10); // Allow for timing
    }

    #[test]
    fn calculate_has_access_false_for_expired() {
        assert!(!calculate_has_access(&MembershipStatus::Expired, None));
        assert!(!calculate_has_access(
            &MembershipStatus::Expired,
            Some(Utc::now())
        ));
    }

    #[test]
    fn calculate_has_access_false_for_pending() {
        assert!(!calculate_has_access(&MembershipStatus::Pending, None));
    }

    #[test]
    fn calculate_has_access_true_for_active() {
        assert!(calculate_has_access(&MembershipStatus::Active, None));
        assert!(calculate_has_access(
            &MembershipStatus::Active,
            Some(Utc::now())
        ));
    }

    #[test]
    fn calculate_has_access_true_for_past_due() {
        assert!(calculate_has_access(&MembershipStatus::PastDue, None));
    }

    #[test]
    fn calculate_has_access_true_for_cancelled_within_period() {
        let future = Utc::now() + chrono::Duration::days(10);
        assert!(calculate_has_access(
            &MembershipStatus::Cancelled,
            Some(future)
        ));
    }

    #[test]
    fn calculate_has_access_false_for_cancelled_past_period() {
        let past = Utc::now() - chrono::Duration::days(1);
        assert!(!calculate_has_access(
            &MembershipStatus::Cancelled,
            Some(past)
        ));
    }

    #[test]
    fn calculate_has_access_false_for_cancelled_no_period() {
        assert!(!calculate_has_access(&MembershipStatus::Cancelled, None));
    }

    #[test]
    fn parse_tier_case_insensitive() {
        assert_eq!(parse_tier("FREE").unwrap(), MembershipTier::Free);
        assert_eq!(parse_tier("Free").unwrap(), MembershipTier::Free);
        assert_eq!(parse_tier("free").unwrap(), MembershipTier::Free);
    }

    #[test]
    fn parse_status_case_insensitive() {
        assert_eq!(parse_status("ACTIVE").unwrap(), MembershipStatus::Active);
        assert_eq!(parse_status("Active").unwrap(), MembershipStatus::Active);
        assert_eq!(parse_status("active").unwrap(), MembershipStatus::Active);
    }
}
