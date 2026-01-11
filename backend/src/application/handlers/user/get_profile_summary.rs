//! GetProfileSummary - Query handler for retrieving profile summary.

use std::sync::Arc;

use crate::domain::foundation::{DomainError, UserId};
use crate::ports::{ProfileReader, ProfileSummary};

/// Query to get profile summary for UI display.
#[derive(Debug, Clone)]
pub struct GetProfileSummaryQuery {
    pub user_id: UserId,
}

/// Handler for getting profile summary.
pub struct GetProfileSummaryHandler {
    reader: Arc<dyn ProfileReader>,
}

impl GetProfileSummaryHandler {
    pub fn new(reader: Arc<dyn ProfileReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(
        &self,
        query: GetProfileSummaryQuery,
    ) -> Result<Option<ProfileSummary>, DomainError> {
        self.reader.get_summary(&query.user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::{ProfileConfidence, RiskClassification, StyleClassification};
    use async_trait::async_trait;

    struct MockProfileReader {
        summary: Option<ProfileSummary>,
    }

    #[async_trait]
    impl ProfileReader for MockProfileReader {
        async fn get_summary(
            &self,
            _user_id: &UserId,
        ) -> Result<Option<ProfileSummary>, DomainError> {
            Ok(self.summary.clone())
        }

        async fn get_agent_instructions(
            &self,
            _user_id: &UserId,
            _domain: Option<crate::domain::user::DecisionDomain>,
        ) -> Result<Option<crate::ports::AgentInstructions>, DomainError> {
            unimplemented!()
        }

        async fn get_decision_history(
            &self,
            _user_id: &UserId,
            _limit: usize,
            _offset: usize,
        ) -> Result<Vec<crate::domain::user::DecisionRecord>, DomainError> {
            unimplemented!()
        }

        async fn get_decisions_by_domain(
            &self,
            _user_id: &UserId,
            _domain: crate::domain::user::DecisionDomain,
        ) -> Result<Vec<crate::domain::user::DecisionRecord>, DomainError> {
            unimplemented!()
        }
    }

    fn test_user_id() -> UserId {
        UserId::new("test@example.com".to_string()).unwrap()
    }

    fn test_summary() -> ProfileSummary {
        ProfileSummary {
            risk_classification: RiskClassification::RiskAverse,
            risk_confidence: 0.75,
            decisions_analyzed: 5,
            profile_confidence: ProfileConfidence::Medium,
            top_values: vec![
                "Work-life balance".to_string(),
                "Financial security".to_string(),
            ],
            decision_style: StyleClassification::AnalyticalCautious,
            active_blind_spots: vec!["Underweights long-term thinking".to_string()],
        }
    }

    #[tokio::test]
    async fn test_get_profile_summary_exists() {
        let reader = Arc::new(MockProfileReader {
            summary: Some(test_summary()),
        });
        let handler = GetProfileSummaryHandler::new(reader);

        let result = handler
            .handle(GetProfileSummaryQuery {
                user_id: test_user_id(),
            })
            .await
            .unwrap();

        assert!(result.is_some());
        let summary = result.unwrap();
        assert_eq!(summary.decisions_analyzed, 5);
        assert_eq!(
            summary.risk_classification,
            RiskClassification::RiskAverse
        );
    }

    #[tokio::test]
    async fn test_get_profile_summary_not_found() {
        let reader = Arc::new(MockProfileReader { summary: None });
        let handler = GetProfileSummaryHandler::new(reader);

        let result = handler
            .handle(GetProfileSummaryQuery {
                user_id: test_user_id(),
            })
            .await
            .unwrap();

        assert!(result.is_none());
    }
}
