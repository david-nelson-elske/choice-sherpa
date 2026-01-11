//! GetAgentInstructions - Query handler for retrieving agent personalization instructions.

use std::sync::Arc;

use crate::domain::foundation::{DomainError, UserId};
use crate::domain::user::DecisionDomain;
use crate::ports::{AgentInstructions, ProfileReader};

/// Query to get agent instructions based on profile.
#[derive(Debug, Clone)]
pub struct GetAgentInstructionsQuery {
    pub user_id: UserId,
    pub domain: Option<DecisionDomain>,
}

/// Handler for getting agent instructions.
pub struct GetAgentInstructionsHandler {
    reader: Arc<dyn ProfileReader>,
}

impl GetAgentInstructionsHandler {
    pub fn new(reader: Arc<dyn ProfileReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(
        &self,
        query: GetAgentInstructionsQuery,
    ) -> Result<Option<AgentInstructions>, DomainError> {
        self.reader
            .get_agent_instructions(&query.user_id, query.domain)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockProfileReader {
        instructions: Option<AgentInstructions>,
    }

    #[async_trait]
    impl ProfileReader for MockProfileReader {
        async fn get_summary(
            &self,
            _user_id: &UserId,
        ) -> Result<Option<crate::ports::ProfileSummary>, DomainError> {
            unimplemented!()
        }

        async fn get_agent_instructions(
            &self,
            _user_id: &UserId,
            _domain: Option<DecisionDomain>,
        ) -> Result<Option<AgentInstructions>, DomainError> {
            Ok(self.instructions.clone())
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
            _domain: DecisionDomain,
        ) -> Result<Vec<crate::domain::user::DecisionRecord>, DomainError> {
            unimplemented!()
        }
    }

    fn test_user_id() -> UserId {
        UserId::new("test@example.com".to_string()).unwrap()
    }

    fn test_instructions() -> AgentInstructions {
        AgentInstructions {
            risk_guidance: "Challenge risk aversion when upside is significant".to_string(),
            blind_spot_prompts: vec![
                "What does this look like in 10 years?".to_string(),
                "What are you giving up by choosing this?".to_string(),
            ],
            communication_adjustments: vec![
                "Skip lengthy preambles - get to questions quickly".to_string(),
                "Use devil's advocate approach".to_string(),
            ],
            suggested_questions: vec!["How does this impact your family?".to_string()],
        }
    }

    #[tokio::test]
    async fn test_get_agent_instructions_exists() {
        let reader = Arc::new(MockProfileReader {
            instructions: Some(test_instructions()),
        });
        let handler = GetAgentInstructionsHandler::new(reader);

        let result = handler
            .handle(GetAgentInstructionsQuery {
                user_id: test_user_id(),
                domain: None,
            })
            .await
            .unwrap();

        assert!(result.is_some());
        let instructions = result.unwrap();
        assert_eq!(instructions.blind_spot_prompts.len(), 2);
        assert!(instructions
            .risk_guidance
            .contains("Challenge risk aversion"));
    }

    #[tokio::test]
    async fn test_get_agent_instructions_with_domain() {
        let reader = Arc::new(MockProfileReader {
            instructions: Some(test_instructions()),
        });
        let handler = GetAgentInstructionsHandler::new(reader);

        let result = handler
            .handle(GetAgentInstructionsQuery {
                user_id: test_user_id(),
                domain: Some(DecisionDomain::Career),
            })
            .await
            .unwrap();

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_get_agent_instructions_not_found() {
        let reader = Arc::new(MockProfileReader {
            instructions: None,
        });
        let handler = GetAgentInstructionsHandler::new(reader);

        let result = handler
            .handle(GetAgentInstructionsQuery {
                user_id: test_user_id(),
                domain: None,
            })
            .await
            .unwrap();

        assert!(result.is_none());
    }
}
