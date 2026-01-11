//! PostgreSQL adapter for ProfileReader

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::{
    foundation::{DomainError, ErrorCode, UserId},
    user::{DecisionDomain, DecisionRecord, ProfileConfidence, RiskClassification, StyleClassification},
};
use crate::ports::{AgentInstructions, ProfileReader, ProfileSummary};

/// PostgreSQL implementation of ProfileReader
pub struct PgProfileReader {
    pool: PgPool,
}

impl PgProfileReader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProfileReader for PgProfileReader {
    async fn get_summary(&self, user_id: &UserId) -> Result<Option<ProfileSummary>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT
                risk_profile,
                values_priorities,
                decision_style,
                blind_spots_growth,
                decisions_analyzed,
                profile_confidence
            FROM decision_profiles
            WHERE user_id = $1
            "#
        )
        .bind(user_id.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        let Some(row) = row else {
            return Ok(None);
        };

        // Parse JSONB fields
        let risk_profile_json: serde_json::Value = row.get("risk_profile");
        let values_json: serde_json::Value = row.get("values_priorities");
        let style_json: serde_json::Value = row.get("decision_style");
        let blind_spots_json: serde_json::Value = row.get("blind_spots_growth");
        let decisions_analyzed: i32 = row.get("decisions_analyzed");
        let confidence_str: String = row.get("profile_confidence");

        // Extract risk classification
        let risk_classification = risk_profile_json
            .get("classification")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "risk_seeking" => Some(RiskClassification::RiskSeeking),
                "risk_neutral" => Some(RiskClassification::RiskNeutral),
                "risk_averse" => Some(RiskClassification::RiskAverse),
                _ => None,
            })
            .unwrap_or(RiskClassification::RiskNeutral);

        let risk_confidence = risk_profile_json
            .get("confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;

        // Extract top values
        let top_values: Vec<String> = values_json
            .get("consistent_objectives")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|obj| obj.get("name").and_then(|n| n.as_str().map(String::from)))
                    .take(5)
                    .collect()
            })
            .unwrap_or_default();

        // Extract decision style
        let decision_style = style_json
            .get("primary_style")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "analytical_cautious" => Some(StyleClassification::AnalyticalCautious),
                "analytical_dynamic" => Some(StyleClassification::AnalyticalDynamic),
                "intuitive_cautious" => Some(StyleClassification::IntuitiveCautious),
                "intuitive_dynamic" => Some(StyleClassification::IntuitiveDynamic),
                "balanced" => Some(StyleClassification::Balanced),
                _ => None,
            })
            .unwrap_or(StyleClassification::Balanced);

        // Extract active blind spots
        let active_blind_spots: Vec<String> = blind_spots_json
            .get("blind_spots")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter(|bs| bs.get("still_active").and_then(|v| v.as_bool()).unwrap_or(false))
                    .filter_map(|bs| bs.get("name").and_then(|n| n.as_str().map(String::from)))
                    .collect()
            })
            .unwrap_or_default();

        let profile_confidence = match confidence_str.as_str() {
            "low" => ProfileConfidence::Low,
            "medium" => ProfileConfidence::Medium,
            "high" => ProfileConfidence::High,
            "very_high" => ProfileConfidence::VeryHigh,
            _ => ProfileConfidence::Low,
        };

        Ok(Some(ProfileSummary {
            risk_classification,
            risk_confidence,
            decisions_analyzed: decisions_analyzed as u32,
            profile_confidence,
            top_values,
            decision_style,
            active_blind_spots,
        }))
    }

    async fn get_agent_instructions(
        &self,
        user_id: &UserId,
        domain: Option<DecisionDomain>,
    ) -> Result<Option<AgentInstructions>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT
                risk_profile,
                blind_spots_growth,
                communication_prefs,
                values_priorities
            FROM decision_profiles
            WHERE user_id = $1
            "#
        )
        .bind(user_id.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let risk_profile_json: serde_json::Value = row.get("risk_profile");
        let blind_spots_json: serde_json::Value = row.get("blind_spots_growth");
        let comm_prefs_json: serde_json::Value = row.get("communication_prefs");
        let values_json: serde_json::Value = row.get("values_priorities");

        // Build risk guidance
        let risk_classification = risk_profile_json
            .get("classification")
            .and_then(|v| v.as_str())
            .unwrap_or("risk_neutral");

        let risk_guidance = match risk_classification {
            "risk_seeking" => "User tends to seek high-variance options. Encourage consideration of downside scenarios.".to_string(),
            "risk_averse" => "User prefers certainty and safety. Challenge risk-averse defaults when potential upside is significant.".to_string(),
            _ => "User evaluates options on expected value. Provide balanced perspective on risk/reward.".to_string(),
        };

        // Build blind spot prompts
        let blind_spot_prompts: Vec<String> = blind_spots_json
            .get("blind_spots")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter(|bs| bs.get("still_active").and_then(|v| v.as_bool()).unwrap_or(false))
                    .filter_map(|bs| {
                        bs.get("agent_behavior")
                            .and_then(|v| v.as_str().map(String::from))
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Build communication adjustments
        let mut communication_adjustments = Vec::new();

        if let Some(preamble) = comm_prefs_json
            .get("interaction_style")
            .and_then(|s| s.get("preamble_preference"))
            .and_then(|v| v.as_str())
        {
            match preamble {
                "minimal" => communication_adjustments.push("Keep preambles minimal - get to questions quickly".to_string()),
                "extensive" => communication_adjustments.push("Provide thorough context before questions".to_string()),
                _ => {}
            }
        }

        if let Some(challenge) = comm_prefs_json
            .get("interaction_style")
            .and_then(|s| s.get("challenge_style"))
            .and_then(|v| v.as_str())
        {
            match challenge {
                "devils_advocate" => communication_adjustments.push("Use devil's advocate approach - user responds well to pushback".to_string()),
                "gentle" => communication_adjustments.push("Challenge gently and supportively".to_string()),
                _ => {}
            }
        }

        // Build suggested questions based on values
        let mut suggested_questions = Vec::new();

        // Extract frequent objectives
        if let Some(objectives) = values_json
            .get("consistent_objectives")
            .and_then(|v| v.as_array())
        {
            for obj in objectives.iter().take(3) {
                if let Some(name) = obj.get("name").and_then(|n| n.as_str()) {
                    suggested_questions.push(format!("How does this decision impact {}?", name));
                }
            }
        }

        // Domain-specific questions
        if let Some(domain) = domain {
            match domain {
                DecisionDomain::Career => {
                    suggested_questions.push("What does this look like in 10 years?".to_string());
                    suggested_questions.push("How does this align with your long-term career vision?".to_string());
                }
                DecisionDomain::Financial => {
                    suggested_questions.push("What's the opportunity cost of this choice?".to_string());
                    suggested_questions.push("How does this fit into your overall financial plan?".to_string());
                }
                DecisionDomain::Family => {
                    suggested_questions.push("How do your family members feel about this?".to_string());
                    suggested_questions.push("What impact will this have on family time/dynamics?".to_string());
                }
                _ => {}
            }
        }

        Ok(Some(AgentInstructions {
            risk_guidance,
            blind_spot_prompts,
            communication_adjustments,
            suggested_questions,
        }))
    }

    async fn get_decision_history(
        &self,
        user_id: &UserId,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<DecisionRecord>, DomainError> {
        // First get profile ID
        let profile_id_row = sqlx::query(
            "SELECT id FROM decision_profiles WHERE user_id = $1"
        )
        .bind(user_id.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        let Some(profile_id_row) = profile_id_row else {
            return Ok(Vec::new());
        };

        let profile_id: uuid::Uuid = profile_id_row.get("id");

        // Get decision history
        let rows = sqlx::query(
            r#"
            SELECT
                cycle_id,
                decision_date,
                title,
                domain,
                dq_score,
                key_tradeoff,
                chosen_alternative,
                outcome_recorded_at,
                satisfaction,
                actual_consequences,
                would_decide_same
            FROM profile_decision_history
            WHERE profile_id = $1
            ORDER BY decision_date DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(profile_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        let mut records = Vec::new();

        for row in rows {
            let cycle_id: uuid::Uuid = row.get("cycle_id");
            let decision_date: chrono::DateTime<chrono::Utc> = row.get("decision_date");
            let title: String = row.get("title");
            let domain_str: String = row.get("domain");
            let dq_score: Option<i32> = row.get("dq_score");
            let key_tradeoff: String = row.get("key_tradeoff");
            let chosen_alternative: String = row.get("chosen_alternative");

            let domain = match domain_str.as_str() {
                "career" => DecisionDomain::Career,
                "financial" => DecisionDomain::Financial,
                "family" => DecisionDomain::Family,
                "health" => DecisionDomain::Health,
                "relationship" => DecisionDomain::Relationship,
                "education" => DecisionDomain::Education,
                "housing" => DecisionDomain::Housing,
                "lifestyle" => DecisionDomain::Lifestyle,
                "business" => DecisionDomain::Business,
                _ => DecisionDomain::Other,
            };

            let outcome = if row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("outcome_recorded_at").ok().flatten().is_some() {
                let outcome_recorded_at: chrono::DateTime<chrono::Utc> = row.get("outcome_recorded_at");
                let satisfaction_str: String = row.get("satisfaction");
                let actual_consequences: String = row.get("actual_consequences");
                let would_decide_same: bool = row.get("would_decide_same");

                let satisfaction = match satisfaction_str.as_str() {
                    "very_dissatisfied" => crate::domain::user::SatisfactionLevel::VeryDissatisfied,
                    "dissatisfied" => crate::domain::user::SatisfactionLevel::Dissatisfied,
                    "neutral" => crate::domain::user::SatisfactionLevel::Neutral,
                    "satisfied" => crate::domain::user::SatisfactionLevel::Satisfied,
                    "very_satisfied" => crate::domain::user::SatisfactionLevel::VerySatisfied,
                    _ => crate::domain::user::SatisfactionLevel::Neutral,
                };

                Some(crate::domain::user::OutcomeRecord::new(
                    crate::domain::foundation::Timestamp::from_datetime(outcome_recorded_at),
                    satisfaction,
                    actual_consequences,
                    vec![], // Surprises not stored in this query
                    would_decide_same,
                )?)
            } else {
                None
            };

            let record = DecisionRecord::new(
                crate::domain::foundation::CycleId::from_uuid(cycle_id),
                crate::domain::foundation::Timestamp::from_datetime(decision_date),
                title,
                domain,
                dq_score.map(|s| s as u8),
                key_tradeoff,
                chosen_alternative,
            )?;

            records.push(record);
        }

        Ok(records)
    }

    async fn get_decisions_by_domain(
        &self,
        user_id: &UserId,
        domain: DecisionDomain,
    ) -> Result<Vec<DecisionRecord>, DomainError> {
        let domain_str = match domain {
            DecisionDomain::Career => "career",
            DecisionDomain::Financial => "financial",
            DecisionDomain::Family => "family",
            DecisionDomain::Health => "health",
            DecisionDomain::Relationship => "relationship",
            DecisionDomain::Education => "education",
            DecisionDomain::Housing => "housing",
            DecisionDomain::Lifestyle => "lifestyle",
            DecisionDomain::Business => "business",
            DecisionDomain::Other => "other",
        };

        // Get profile ID
        let profile_id_row = sqlx::query(
            "SELECT id FROM decision_profiles WHERE user_id = $1"
        )
        .bind(user_id.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        let Some(profile_id_row) = profile_id_row else {
            return Ok(Vec::new());
        };

        let profile_id: uuid::Uuid = profile_id_row.get("id");

        // Get filtered decisions
        let rows = sqlx::query(
            r#"
            SELECT
                cycle_id,
                decision_date,
                title,
                domain,
                dq_score,
                key_tradeoff,
                chosen_alternative
            FROM profile_decision_history
            WHERE profile_id = $1 AND domain = $2
            ORDER BY decision_date DESC
            "#
        )
        .bind(profile_id)
        .bind(domain_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::new(ErrorCode::InternalError, format!("Database error: {}", e)))?;

        let mut records = Vec::new();

        for row in rows {
            let cycle_id: uuid::Uuid = row.get("cycle_id");
            let decision_date: chrono::DateTime<chrono::Utc> = row.get("decision_date");
            let title: String = row.get("title");
            let dq_score: Option<i32> = row.get("dq_score");
            let key_tradeoff: String = row.get("key_tradeoff");
            let chosen_alternative: String = row.get("chosen_alternative");

            let record = DecisionRecord::new(
                crate::domain::foundation::CycleId::from_uuid(cycle_id),
                crate::domain::foundation::Timestamp::from_datetime(decision_date),
                title,
                domain,
                dq_score.map(|s| s as u8),
                key_tradeoff,
                chosen_alternative,
            )?;

            records.push(record);
        }

        Ok(records)
    }
}
