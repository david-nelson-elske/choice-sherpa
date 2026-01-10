//! PostgreSQL implementation of MembershipRepository.
//!
//! Provides persistent storage for Membership aggregates using PostgreSQL.

use crate::domain::foundation::{DomainError, ErrorCode, MembershipId, Timestamp, UserId};
use crate::domain::membership::{Membership, MembershipStatus, MembershipTier};
use crate::ports::MembershipRepository;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of the MembershipRepository port.
///
/// Uses sqlx for type-safe database operations with connection pooling.
pub struct PostgresMembershipRepository {
    pool: PgPool,
}

impl PostgresMembershipRepository {
    /// Creates a new PostgresMembershipRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Database row representation of a membership.
#[derive(Debug, sqlx::FromRow)]
struct MembershipRow {
    id: Uuid,
    user_id: Uuid,
    tier: String,
    status: String,
    stripe_customer_id: Option<String>,
    stripe_subscription_id: Option<String>,
    promo_code: Option<String>,
    current_period_start: Option<DateTime<Utc>>,
    current_period_end: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[allow(dead_code)]
    version: i32,
}

impl TryFrom<MembershipRow> for Membership {
    type Error = DomainError;

    fn try_from(row: MembershipRow) -> Result<Self, Self::Error> {
        let tier = parse_tier(&row.tier)?;
        let status = parse_status(&row.status)?;

        // For period dates, use created_at as fallback
        let period_start = row
            .current_period_start
            .map(Timestamp::from_datetime)
            .unwrap_or_else(|| Timestamp::from_datetime(row.created_at));
        let period_end = row
            .current_period_end
            .map(Timestamp::from_datetime)
            .unwrap_or_else(|| Timestamp::from_datetime(row.created_at));

        Ok(Membership {
            id: MembershipId::from_uuid(row.id),
            user_id: UserId::new(row.user_id.to_string()).map_err(|e| {
                DomainError::new(ErrorCode::DatabaseError, format!("Invalid user_id: {}", e))
            })?,
            tier,
            status,
            current_period_start: period_start,
            current_period_end: period_end,
            promo_code: row.promo_code,
            stripe_customer_id: row.stripe_customer_id,
            stripe_subscription_id: row.stripe_subscription_id,
            created_at: Timestamp::from_datetime(row.created_at),
            updated_at: Timestamp::from_datetime(row.updated_at),
            cancelled_at: None, // Note: cancelled_at is derived from status, not stored separately
        })
    }
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

fn tier_to_string(tier: &MembershipTier) -> &'static str {
    match tier {
        MembershipTier::Free => "free",
        MembershipTier::Monthly => "monthly",
        MembershipTier::Annual => "annual",
    }
}

fn status_to_string(status: &MembershipStatus) -> &'static str {
    match status {
        MembershipStatus::Pending => "pending",
        MembershipStatus::Active => "active",
        MembershipStatus::PastDue => "past_due",
        MembershipStatus::Cancelled => "cancelled",
        MembershipStatus::Expired => "expired",
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

#[async_trait]
impl MembershipRepository for PostgresMembershipRepository {
    async fn save(&self, membership: &Membership) -> Result<(), DomainError> {
        let user_uuid = parse_user_id_as_uuid(&membership.user_id)?;

        sqlx::query(
            r#"
            INSERT INTO memberships (
                id, user_id, tier, status, stripe_customer_id, stripe_subscription_id,
                promo_code, current_period_start, current_period_end, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(membership.id.as_uuid())
        .bind(user_uuid)
        .bind(tier_to_string(&membership.tier))
        .bind(status_to_string(&membership.status))
        .bind(&membership.stripe_customer_id)
        .bind(&membership.stripe_subscription_id)
        .bind(&membership.promo_code)
        .bind(membership.current_period_start.as_datetime())
        .bind(membership.current_period_end.as_datetime())
        .bind(membership.created_at.as_datetime())
        .bind(membership.updated_at.as_datetime())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint() == Some("memberships_user_id_key") {
                    return DomainError::new(
                        ErrorCode::MembershipExists,
                        "User already has a membership",
                    );
                }
            }
            DomainError::new(ErrorCode::DatabaseError, format!("Failed to save membership: {}", e))
        })?;

        Ok(())
    }

    async fn update(&self, membership: &Membership) -> Result<(), DomainError> {
        let result = sqlx::query(
            r#"
            UPDATE memberships SET
                tier = $2,
                status = $3,
                stripe_customer_id = $4,
                stripe_subscription_id = $5,
                promo_code = $6,
                current_period_start = $7,
                current_period_end = $8,
                updated_at = $9,
                version = version + 1
            WHERE id = $1
            "#,
        )
        .bind(membership.id.as_uuid())
        .bind(tier_to_string(&membership.tier))
        .bind(status_to_string(&membership.status))
        .bind(&membership.stripe_customer_id)
        .bind(&membership.stripe_subscription_id)
        .bind(&membership.promo_code)
        .bind(membership.current_period_start.as_datetime())
        .bind(membership.current_period_end.as_datetime())
        .bind(membership.updated_at.as_datetime())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(ErrorCode::DatabaseError, format!("Failed to update membership: {}", e))
        })?;

        if result.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::MembershipNotFound,
                "Membership not found",
            ));
        }

        Ok(())
    }

    async fn find_by_id(&self, id: &MembershipId) -> Result<Option<Membership>, DomainError> {
        let row: Option<MembershipRow> = sqlx::query_as(
            r#"
            SELECT id, user_id, tier, status, stripe_customer_id, stripe_subscription_id,
                   promo_code, current_period_start, current_period_end, created_at, updated_at, version
            FROM memberships
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(ErrorCode::DatabaseError, format!("Failed to find membership: {}", e))
        })?;

        row.map(Membership::try_from).transpose()
    }

    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Option<Membership>, DomainError> {
        let user_uuid = parse_user_id_as_uuid(user_id)?;

        let row: Option<MembershipRow> = sqlx::query_as(
            r#"
            SELECT id, user_id, tier, status, stripe_customer_id, stripe_subscription_id,
                   promo_code, current_period_start, current_period_end, created_at, updated_at, version
            FROM memberships
            WHERE user_id = $1
            "#,
        )
        .bind(user_uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(ErrorCode::DatabaseError, format!("Failed to find membership: {}", e))
        })?;

        row.map(Membership::try_from).transpose()
    }

    async fn find_expiring_within_days(&self, days: u32) -> Result<Vec<Membership>, DomainError> {
        let now = Utc::now();
        let expiry_threshold = now + chrono::Duration::days(i64::from(days));

        let rows: Vec<MembershipRow> = sqlx::query_as(
            r#"
            SELECT id, user_id, tier, status, stripe_customer_id, stripe_subscription_id,
                   promo_code, current_period_start, current_period_end, created_at, updated_at, version
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
                format!("Failed to find expiring memberships: {}", e),
            )
        })?;

        rows.into_iter().map(Membership::try_from).collect()
    }

    async fn delete(&self, id: &MembershipId) -> Result<(), DomainError> {
        let result = sqlx::query("DELETE FROM memberships WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| {
                DomainError::new(ErrorCode::DatabaseError, format!("Failed to delete membership: {}", e))
            })?;

        if result.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::MembershipNotFound,
                "Membership not found",
            ));
        }

        Ok(())
    }

    async fn find_by_stripe_subscription_id(
        &self,
        subscription_id: &str,
    ) -> Result<Option<Membership>, DomainError> {
        let row: Option<MembershipRow> = sqlx::query_as(
            r#"
            SELECT id, user_id, tier, status, stripe_customer_id, stripe_subscription_id,
                   promo_code, current_period_start, current_period_end, created_at, updated_at, version
            FROM memberships
            WHERE stripe_subscription_id = $1
            "#,
        )
        .bind(subscription_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(ErrorCode::DatabaseError, format!("Failed to find membership: {}", e))
        })?;

        row.map(Membership::try_from).transpose()
    }

    async fn find_by_stripe_customer_id(
        &self,
        customer_id: &str,
    ) -> Result<Option<Membership>, DomainError> {
        let row: Option<MembershipRow> = sqlx::query_as(
            r#"
            SELECT id, user_id, tier, status, stripe_customer_id, stripe_subscription_id,
                   promo_code, current_period_start, current_period_end, created_at, updated_at, version
            FROM memberships
            WHERE stripe_customer_id = $1
            "#,
        )
        .bind(customer_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            DomainError::new(ErrorCode::DatabaseError, format!("Failed to find membership: {}", e))
        })?;

        row.map(Membership::try_from).transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tier_works_for_all_values() {
        assert_eq!(parse_tier("free").unwrap(), MembershipTier::Free);
        assert_eq!(parse_tier("monthly").unwrap(), MembershipTier::Monthly);
        assert_eq!(parse_tier("annual").unwrap(), MembershipTier::Annual);
        assert_eq!(parse_tier("FREE").unwrap(), MembershipTier::Free);
        assert_eq!(parse_tier("Monthly").unwrap(), MembershipTier::Monthly);
    }

    #[test]
    fn parse_tier_rejects_invalid_values() {
        assert!(parse_tier("invalid").is_err());
        assert!(parse_tier("").is_err());
    }

    #[test]
    fn parse_status_works_for_all_values() {
        assert_eq!(parse_status("pending").unwrap(), MembershipStatus::Pending);
        assert_eq!(parse_status("active").unwrap(), MembershipStatus::Active);
        assert_eq!(parse_status("past_due").unwrap(), MembershipStatus::PastDue);
        assert_eq!(parse_status("cancelled").unwrap(), MembershipStatus::Cancelled);
        assert_eq!(parse_status("expired").unwrap(), MembershipStatus::Expired);
    }

    #[test]
    fn parse_status_rejects_invalid_values() {
        assert!(parse_status("invalid").is_err());
        assert!(parse_status("").is_err());
    }

    #[test]
    fn tier_to_string_is_consistent() {
        assert_eq!(tier_to_string(&MembershipTier::Free), "free");
        assert_eq!(tier_to_string(&MembershipTier::Monthly), "monthly");
        assert_eq!(tier_to_string(&MembershipTier::Annual), "annual");
    }

    #[test]
    fn status_to_string_is_consistent() {
        assert_eq!(status_to_string(&MembershipStatus::Pending), "pending");
        assert_eq!(status_to_string(&MembershipStatus::Active), "active");
        assert_eq!(status_to_string(&MembershipStatus::PastDue), "past_due");
        assert_eq!(status_to_string(&MembershipStatus::Cancelled), "cancelled");
        assert_eq!(status_to_string(&MembershipStatus::Expired), "expired");
    }

    #[test]
    fn parse_user_id_as_uuid_accepts_valid_uuid() {
        let user_id = UserId::new("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let result = parse_user_id_as_uuid(&user_id);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_user_id_as_uuid_rejects_invalid_uuid() {
        let user_id = UserId::new("not-a-uuid").unwrap();
        let result = parse_user_id_as_uuid(&user_id);
        assert!(result.is_err());
    }

    #[test]
    fn roundtrip_tier_conversion() {
        for tier in [
            MembershipTier::Free,
            MembershipTier::Monthly,
            MembershipTier::Annual,
        ] {
            let s = tier_to_string(&tier);
            let parsed = parse_tier(s).unwrap();
            assert_eq!(tier, parsed);
        }
    }

    #[test]
    fn roundtrip_status_conversion() {
        for status in [
            MembershipStatus::Pending,
            MembershipStatus::Active,
            MembershipStatus::PastDue,
            MembershipStatus::Cancelled,
            MembershipStatus::Expired,
        ] {
            let s = status_to_string(&status);
            let parsed = parse_status(s).unwrap();
            assert_eq!(status, parsed);
        }
    }
}
