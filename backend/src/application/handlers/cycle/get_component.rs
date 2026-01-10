//! GetComponentHandler - Query handler for retrieving component output.
//!
//! Returns the structured output data and status of a specific component
//! within a cycle. The output schema varies by component type.

use std::sync::Arc;

use crate::domain::foundation::{ComponentType, CycleId, DomainError};
use crate::ports::{ComponentOutputView, CycleReader};

/// Query to get a component's output from a cycle.
#[derive(Debug, Clone)]
pub struct GetComponentQuery {
    /// The cycle containing the component.
    pub cycle_id: CycleId,
    /// The component type to retrieve.
    pub component_type: ComponentType,
}

/// Result of successful component query.
pub type GetComponentResult = Option<ComponentOutputView>;

/// Handler for retrieving component output.
///
/// Returns the component's structured output and status,
/// or `None` if the cycle is not found.
pub struct GetComponentHandler {
    reader: Arc<dyn CycleReader>,
}

impl GetComponentHandler {
    pub fn new(reader: Arc<dyn CycleReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(&self, query: GetComponentQuery) -> Result<GetComponentResult, DomainError> {
        self.reader
            .get_component_output(&query.cycle_id, query.component_type)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{ComponentStatus, SessionId, Timestamp};
    use crate::ports::{CycleProgressView, CycleSummary, CycleTreeNode, CycleView};
    use async_trait::async_trait;
    use serde_json::json;

    // ─────────────────────────────────────────────────────────────────────
    // Mock Implementation
    // ─────────────────────────────────────────────────────────────────────

    struct MockCycleReader {
        outputs: Vec<ComponentOutputView>,
        fail_read: bool,
    }

    impl MockCycleReader {
        fn new() -> Self {
            Self {
                outputs: Vec::new(),
                fail_read: false,
            }
        }

        fn with_output(output: ComponentOutputView) -> Self {
            Self {
                outputs: vec![output],
                fail_read: false,
            }
        }

        fn failing() -> Self {
            Self {
                outputs: Vec::new(),
                fail_read: true,
            }
        }
    }

    #[async_trait]
    impl CycleReader for MockCycleReader {
        async fn get_by_id(&self, _id: &CycleId) -> Result<Option<CycleView>, DomainError> {
            Ok(None)
        }

        async fn list_by_session_id(
            &self,
            _session_id: &SessionId,
        ) -> Result<Vec<CycleSummary>, DomainError> {
            Ok(vec![])
        }

        async fn get_tree(
            &self,
            _session_id: &SessionId,
        ) -> Result<Option<CycleTreeNode>, DomainError> {
            Ok(None)
        }

        async fn get_progress(&self, _id: &CycleId) -> Result<Option<CycleProgressView>, DomainError> {
            Ok(None)
        }

        async fn get_lineage(&self, _id: &CycleId) -> Result<Vec<CycleSummary>, DomainError> {
            Ok(vec![])
        }

        async fn get_component_output(
            &self,
            cycle_id: &CycleId,
            component_type: ComponentType,
        ) -> Result<Option<ComponentOutputView>, DomainError> {
            if self.fail_read {
                return Err(DomainError::new(
                    crate::domain::foundation::ErrorCode::DatabaseError,
                    "Simulated read failure",
                ));
            }
            Ok(self
                .outputs
                .iter()
                .find(|o| o.cycle_id == *cycle_id && o.component_type == component_type)
                .cloned())
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Test Helpers
    // ─────────────────────────────────────────────────────────────────────

    fn create_test_output(cycle_id: CycleId) -> ComponentOutputView {
        ComponentOutputView {
            cycle_id,
            component_type: ComponentType::IssueRaising,
            status: ComponentStatus::InProgress,
            output: json!({
                "potential_decisions": ["Should we expand?"],
                "objectives": ["Increase revenue"],
                "uncertainties": ["Market conditions"],
                "considerations": ["Budget constraints"],
                "user_confirmed": false
            }),
            updated_at: Timestamp::now(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Tests
    // ─────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn returns_output_when_found() {
        let cycle_id = CycleId::new();
        let output = create_test_output(cycle_id);

        let reader = Arc::new(MockCycleReader::with_output(output));
        let handler = GetComponentHandler::new(reader);

        let query = GetComponentQuery {
            cycle_id,
            component_type: ComponentType::IssueRaising,
        };
        let result = handler.handle(query).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_some());
        let output = result.unwrap();
        assert_eq!(output.cycle_id, cycle_id);
        assert_eq!(output.component_type, ComponentType::IssueRaising);
    }

    #[tokio::test]
    async fn returns_none_when_not_found() {
        let reader = Arc::new(MockCycleReader::new());
        let handler = GetComponentHandler::new(reader);

        let query = GetComponentQuery {
            cycle_id: CycleId::new(),
            component_type: ComponentType::IssueRaising,
        };
        let result = handler.handle(query).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn returns_error_on_read_failure() {
        let reader = Arc::new(MockCycleReader::failing());
        let handler = GetComponentHandler::new(reader);

        let query = GetComponentQuery {
            cycle_id: CycleId::new(),
            component_type: ComponentType::IssueRaising,
        };
        let result = handler.handle(query).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn returns_none_for_different_component_type() {
        let cycle_id = CycleId::new();
        let output = create_test_output(cycle_id);

        let reader = Arc::new(MockCycleReader::with_output(output));
        let handler = GetComponentHandler::new(reader);

        // Query for a different component type
        let query = GetComponentQuery {
            cycle_id,
            component_type: ComponentType::ProblemFrame, // Different from IssueRaising
        };
        let result = handler.handle(query).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn includes_status_and_output_data() {
        let cycle_id = CycleId::new();
        let output = create_test_output(cycle_id);

        let reader = Arc::new(MockCycleReader::with_output(output));
        let handler = GetComponentHandler::new(reader);

        let query = GetComponentQuery {
            cycle_id,
            component_type: ComponentType::IssueRaising,
        };
        let result = handler.handle(query).await.unwrap().unwrap();

        assert_eq!(result.status, ComponentStatus::InProgress);
        assert!(result.output.get("potential_decisions").is_some());
        assert!(result.output.get("objectives").is_some());
    }
}
