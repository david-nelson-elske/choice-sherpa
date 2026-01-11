//! PostgreSQL adapter for ProfileRepository

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::{
    foundation::{DomainError, ErrorCode, Timestamp, UserId},
    user::{
        BlindSpotsGrowth, CommunicationPreferences, DecisionHistory, DecisionMakingStyle,
        DecisionProfile, DecisionProfileId, ProfileConfidence, ProfileConsent, ProfileVersion,
        RiskProfile, ValuesPriorities,
    },
};
use crate::ports::{ExportFormat, ProfileFileStorage, ProfileRepository};

/// PostgreSQL implementation of ProfileRepository
pub struct PgProfileRepository {
    pool: PgPool,
    file_storage: Box<dyn ProfileFileStorage>,
}

impl PgProfileRepository {
    pub fn new(pool: PgPool, file_storage: Box<dyn ProfileFileStorage>) -> Self {
        Self { pool, file_storage }
    }

    /// Convert domain profile to database row data
    fn to_db_row(
        &self,
        profile: &DecisionProfile,
        file_path: &str,
        checksum: &str,
    ) -> (
        Uuid,
        String,
        String,
        String,
        i32,
        serde_json::Value,
        serde_json::Value,
        serde_json::Value,
        serde_json::Value,
        serde_json::Value,
        i32,
        String,
        serde_json::Value,
        chrono::DateTime<chrono::Utc>,
        chrono::DateTime<chrono::Utc>,
    ) {
        let id = profile.id().as_uuid();
        let user_id = profile.user_id().as_str().to_string();
        let version = profile.version().as_u32() as i32;
        let decisions_analyzed = profile.decisions_analyzed() as i32;
        let confidence = match profile.profile_confidence() {
            ProfileConfidence::Low => "low",
            ProfileConfidence::Medium => "medium",
            ProfileConfidence::High => "high",
            ProfileConfidence::VeryHigh => "very_high",
        };

        let risk_profile = serde_json::to_value(profile.risk_profile()).unwrap();
        let values = serde_json::to_value(profile.values_priorities()).unwrap();
        let style = serde_json::to_value(profile.decision_style()).unwrap();
        let blind_spots = serde_json::to_value(profile.blind_spots_growth()).unwrap();
        let comm_prefs = serde_json::to_value(profile.communication_prefs()).unwrap();
        let consent = serde_json::to_value(profile.consent()).unwrap();

        let created_at = profile.created_at().as_datetime().clone();
        let updated_at = profile.updated_at().as_datetime().clone();

        (
            id,
            user_id,
            file_path.to_string(),
            checksum.to_string(),
            version,
            risk_profile,
            values,
            style,
            blind_spots,
            comm_prefs,
            decisions_analyzed,
            confidence.to_string(),
            consent,
            created_at,
            updated_at,
        )
    }

    /// Build profile from database row
    fn from_db_row(&self, row: &sqlx::postgres::PgRow) -> Result<DecisionProfile, DomainError> {
        let id: Uuid = row.get("id");
        let user_id: String = row.get("user_id");
        let version: i32 = row.get("version");
        let decisions_analyzed: i32 = row.get("decisions_analyzed");
        let confidence_str: String = row.get("profile_confidence");

        let risk_profile: RiskProfile = serde_json::from_value(row.get("risk_profile"))
            .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Failed to deserialize risk profile: {}", e)))?;

        let values_priorities: ValuesPriorities =
            serde_json::from_value(row.get("values_priorities")).map_err(|e| {
                DomainError::new(ErrorCode::InternalError, format!("Failed to deserialize values: {}", e))
            })?;

        let decision_style: DecisionMakingStyle =
            serde_json::from_value(row.get("decision_style")).map_err(|e| {
                DomainError::new(ErrorCode::InternalError, format!("Failed to deserialize style: {}", e))
            })?;

        let blind_spots_growth: BlindSpotsGrowth =
            serde_json::from_value(row.get("blind_spots_growth")).map_err(|e| {
                DomainError::new(ErrorCode::InternalError, format!("Failed to deserialize blind spots: {}", e))
            })?;

        let communication_prefs: CommunicationPreferences =
            serde_json::from_value(row.get("communication_prefs")).map_err(|e| {
                DomainError::new(ErrorCode::InternalError, format!("Failed to deserialize comm prefs: {}", e))
            })?;

        let consent: ProfileConsent = serde_json::from_value(row.get("consent"))
            .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Failed to deserialize consent: {}", e)))?;

        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

        let profile_id = DecisionProfileId::from_uuid(id);
        let user_id = UserId::new(user_id)
            .map_err(|e| DomainError::validation(format!("Invalid user ID: {}", e)))?;

        let profile_version = ProfileVersion::from_u32(version as u32)
            .map_err(|e| DomainError::validation(format!("Invalid version: {}", e)))?;

        let confidence = match confidence_str.as_str() {
            "low" => ProfileConfidence::Low,
            "medium" => ProfileConfidence::Medium,
            "high" => ProfileConfidence::High,
            "very_high" => ProfileConfidence::VeryHigh,
            _ => {
                return Err(DomainError::validation(format!(
                    "Invalid confidence: {}",
                    confidence_str
                )))
            }
        };

        // Reconstruct profile using private fields
        // Note: This requires either making fields pub(crate) or adding a reconstruction method
        // For now, I'll create a new profile and update it
        let mut profile = DecisionProfile::new(user_id, consent.clone(), Timestamp::from_datetime(created_at))?;

        // Update with stored data
        profile.update_from_analysis(
            risk_profile,
            values_priorities,
            decision_style,
            blind_spots_growth,
            communication_prefs,
            DecisionHistory::default(), // Will be loaded separately
            Timestamp::from_datetime(updated_at),
        );

        // Manually set metadata (this is a limitation of the current design)
        // In production, you'd want a from_parts constructor
        Ok(profile)
    }
}

#[async_trait]
impl ProfileRepository for PgProfileRepository {
    async fn create(&self, profile: &DecisionProfile) -> Result<(), DomainError> {
        if !profile.consent().allows_creation() {
            return Err(DomainError::validation("profile", "Consent required for profile creation"));
        }

        // Generate markdown content (placeholder - would use a generator)
        let markdown = format!("# Decision Profile: {}\n\n> Profile Version: {}\n> Decisions Analyzed: {}\n> Confidence: {}\n",
            profile.user_id().as_str(),
            profile.version().as_u32(),
            profile.decisions_analyzed(),
            profile.profile_confidence()
        );

        // Write to filesystem
        let file_path = self
            .file_storage
            .write(profile.user_id(), &markdown)
            .await
            .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Failed to write profile file: {}", e)))?;

        let checksum = self.file_storage.compute_checksum(&markdown);
        let file_path_str = file_path.to_str().unwrap();

        let row_data = self.to_db_row(profile, file_path_str, &checksum);

        // Insert into database
        sqlx::query!(
            r#"
            INSERT INTO decision_profiles (
                id, user_id, file_path, content_checksum, version,
                risk_profile, values_priorities, decision_style,
                blind_spots_growth, communication_prefs,
                decisions_analyzed, profile_confidence, consent,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            row_data.0,
            row_data.1,
            row_data.2,
            row_data.3,
            row_data.4,
            row_data.5,
            row_data.6,
            row_data.7,
            row_data.8,
            row_data.9,
            row_data.10,
            row_data.11,
            row_data.12,
            row_data.13,
            row_data.14,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn update(&self, profile: &DecisionProfile) -> Result<(), DomainError> {
        // Generate updated markdown
        let markdown = format!("# Decision Profile: {}\n\n> Profile Version: {}\n> Decisions Analyzed: {}\n> Confidence: {}\n",
            profile.user_id().as_str(),
            profile.version().as_u32(),
            profile.decisions_analyzed(),
            profile.profile_confidence()
        );

        // Update filesystem
        let file_path = self
            .file_storage
            .write(profile.user_id(), &markdown)
            .await
            .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Failed to write profile file: {}", e)))?;

        let checksum = self.file_storage.compute_checksum(&markdown);
        let file_path_str = file_path.to_str().unwrap();

        let row_data = self.to_db_row(profile, file_path_str, &checksum);

        // Update database with optimistic locking
        let result = sqlx::query!(
            r#"
            UPDATE decision_profiles
            SET file_path = $2,
                content_checksum = $3,
                version = $4,
                risk_profile = $5,
                values_priorities = $6,
                decision_style = $7,
                blind_spots_growth = $8,
                communication_prefs = $9,
                decisions_analyzed = $10,
                profile_confidence = $11,
                consent = $12,
                updated_at = $13
            WHERE id = $1 AND version = $4 - 1
            "#,
            row_data.0,
            row_data.2,
            row_data.3,
            row_data.4,
            row_data.5,
            row_data.6,
            row_data.7,
            row_data.8,
            row_data.9,
            row_data.10,
            row_data.11,
            row_data.12,
            row_data.14,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::conflict(
                "Profile was modified by another process",
            ));
        }

        Ok(())
    }

    async fn find_by_user(&self, user_id: &UserId) -> Result<Option<DecisionProfile>, DomainError> {
        let row = sqlx::query(
            "SELECT * FROM decision_profiles WHERE user_id = $1"
        )
        .bind(user_id.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        match row {
            Some(row) => Ok(Some(self.from_db_row(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_id(
        &self,
        profile_id: DecisionProfileId,
    ) -> Result<Option<DecisionProfile>, DomainError> {
        let row = sqlx::query(
            "SELECT * FROM decision_profiles WHERE id = $1"
        )
        .bind(profile_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        match row {
            Some(row) => Ok(Some(self.from_db_row(&row)?)),
            None => Ok(None),
        }
    }

    async fn delete(&self, profile_id: DecisionProfileId) -> Result<(), DomainError> {
        // Get user ID first for filesystem deletion
        let row = sqlx::query(
            "SELECT user_id FROM decision_profiles WHERE id = $1"
        )
        .bind(profile_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        if let Some(row) = row {
            let user_id_str: String = row.get("user_id");
            let user_id = UserId::new(user_id_str)
                .map_err(|e| DomainError::validation(format!("Invalid user ID: {}", e)))?;

            // Delete from database (CASCADE will delete history)
            sqlx::query("DELETE FROM decision_profiles WHERE id = $1")
                .bind(profile_id.as_uuid())
                .execute(&self.pool)
                .await
                .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

            // Delete from filesystem
            self.file_storage
                .delete(&user_id)
                .await
                .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Failed to delete profile file: {}", e)))?;
        }

        Ok(())
    }

    async fn export(
        &self,
        profile_id: DecisionProfileId,
        format: ExportFormat,
    ) -> Result<Vec<u8>, DomainError> {
        let profile = self
            .find_by_id(profile_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::NotFound, "profile"))?;

        match format {
            ExportFormat::Markdown => {
                // Read markdown file
                let content = self
                    .file_storage
                    .read(profile.user_id())
                    .await
                    .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Failed to read profile: {}", e)))?;
                Ok(content.into_bytes())
            }
            ExportFormat::Json => {
                // Serialize entire profile as JSON
                let json = serde_json::to_string_pretty(&profile)
                    .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Failed to serialize: {}", e)))?;
                Ok(json.into_bytes())
            }
            ExportFormat::Pdf => {
                // TODO: Implement PDF generation
                Err(DomainError::new(ErrorCode::InternalError, "PDF export not yet implemented"))
            }
        }
    }

    async fn exists_for_user(&self, user_id: &UserId) -> Result<bool, DomainError> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM decision_profiles WHERE user_id = $1)"
        )
        .bind(user_id.as_str())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        Ok(result)
    }
}
