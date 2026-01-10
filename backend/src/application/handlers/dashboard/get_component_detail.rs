//! GetComponentDetailHandler - Query handler for retrieving component details.
//!
//! Returns detailed view of a specific component including structured output,
//! conversation metadata, and navigation context.

use std::sync::Arc;

use crate::domain::dashboard::ComponentDetailView;
use crate::domain::foundation::{ComponentType, CycleId, UserId};
use crate::ports::{DashboardError, DashboardReader};

/// Query to get component detail.
#[derive(Debug, Clone)]
pub struct GetComponentDetailQuery {
    /// The cycle ID containing the component.
    pub cycle_id: CycleId,
    /// The component type to retrieve.
    pub component_type: ComponentType,
    /// User ID for authorization.
    pub user_id: UserId,
}

/// Result of successful component detail query.
pub type GetComponentDetailResult = ComponentDetailView;

/// Handler for retrieving component details.
///
/// Returns full component data for drill-down views.
pub struct GetComponentDetailHandler {
    reader: Arc<dyn DashboardReader>,
}

impl GetComponentDetailHandler {
    pub fn new(reader: Arc<dyn DashboardReader>) -> Self {
        Self { reader }
    }

    pub async fn handle(
        &self,
        query: GetComponentDetailQuery,
    ) -> Result<GetComponentDetailResult, DashboardError> {
        self.reader
            .get_component_detail(query.cycle_id, query.component_type, &query.user_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::dashboard::{ComponentDetailView, DashboardOverview};
    use crate::domain::foundation::{ComponentId, ComponentStatus, ComponentType, CycleId, SessionId, UserId};
    use async_trait::async_trait;
    use serde_json::json;

    // ─────────────────────────────────────────────────────────────────────
    // Mock Implementation
    // ─────────────────────────────────────────────────────────────────────

    struct MockDashboardReader {
        component_detail: Option<ComponentDetailView>,
        should_fail: bool,
        should_unauthorized: bool,
    }

    impl MockDashboardReader {
        fn with_component_detail(detail: ComponentDetailView) -> Self {
            Self {
                component_detail: Some(detail),
                should_fail: false,
                should_unauthorized: false,
            }
        }

        fn failing() -> Self {
            Self {
                component_detail: None,
                should_fail: true,
                should_unauthorized: false,
            }
        }

        fn unauthorized() -> Self {
            Self {
                component_detail: None,
                should_fail: false,
                should_unauthorized: true,
            }
        }
    }

    #[async_trait]
    impl DashboardReader for MockDashboardReader {
        async fn get_overview(
            &self,
            _session_id: SessionId,
            _cycle_id: Option<CycleId>,
            _user_id: &UserId,
        ) -> Result<DashboardOverview, DashboardError> {
            unimplemented!()
        }

        async fn get_component_detail(
            &self,
            _cycle_id: CycleId,
            _component_type: ComponentType,
            _user_id: &UserId,
        ) -> Result<ComponentDetailView, DashboardError> {
            if self.should_unauthorized {
                return Err(DashboardError::Unauthorized);
            }
            if self.should_fail {
                return Err(DashboardError::Database("Simulated failure".to_string()));
            }
            self.component_detail
                .clone()
                .ok_or_else(|| DashboardError::ComponentNotFound(ComponentType::Objectives))
        }

        async fn compare_cycles(
            &self,
            _cycle_ids: &[CycleId],
            _user_id: &UserId,
        ) -> Result<crate::domain::dashboard::CycleComparison, DashboardError> {
            unimplemented!()
        }
    }

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    fn create_test_component_detail() -> ComponentDetailView {
        ComponentDetailView {
            component_id: ComponentId::new(),
            cycle_id: CycleId::new(),
            component_type: ComponentType::Objectives,
            status: ComponentStatus::Complete,
            structured_output: json!({
                "objectives": [{"id": "obj1", "description": "Test objective"}]
            }),
            conversation_message_count: 10,
            last_message_at: Some(chrono::Utc::now()),
            can_branch: true,
            can_revise: true,
            previous_component: Some(ComponentType::ProblemFrame),
            next_component: Some(ComponentType::Alternatives),
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Tests
    // ─────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_detail_returns_component() {
        let detail = create_test_component_detail();
        let reader = Arc::new(MockDashboardReader::with_component_detail(detail.clone()));
        let handler = GetComponentDetailHandler::new(reader);

        let query = GetComponentDetailQuery {
            cycle_id: detail.cycle_id,
            component_type: ComponentType::Objectives,
            user_id: test_user_id(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_ok());

        let returned_detail = result.unwrap();
        assert_eq!(returned_detail.component_type, ComponentType::Objectives);
        assert_eq!(returned_detail.status, ComponentStatus::Complete);
    }

    #[tokio::test]
    async fn test_get_detail_passes_component_type() {
        let detail = create_test_component_detail();
        let reader = Arc::new(MockDashboardReader::with_component_detail(detail.clone()));
        let handler = GetComponentDetailHandler::new(reader);

        let query = GetComponentDetailQuery {
            cycle_id: detail.cycle_id,
            component_type: ComponentType::Alternatives,
            user_id: test_user_id(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_detail_passes_user_id() {
        let detail = create_test_component_detail();
        let user_id = test_user_id();
        let reader = Arc::new(MockDashboardReader::with_component_detail(detail.clone()));
        let handler = GetComponentDetailHandler::new(reader);

        let query = GetComponentDetailQuery {
            cycle_id: detail.cycle_id,
            component_type: ComponentType::Objectives,
            user_id: user_id.clone(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_detail_handles_unauthorized() {
        let reader = Arc::new(MockDashboardReader::unauthorized());
        let handler = GetComponentDetailHandler::new(reader);

        let query = GetComponentDetailQuery {
            cycle_id: CycleId::new(),
            component_type: ComponentType::Objectives,
            user_id: test_user_id(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DashboardError::Unauthorized));
    }

    #[tokio::test]
    async fn test_get_detail_propagates_errors() {
        let reader = Arc::new(MockDashboardReader::failing());
        let handler = GetComponentDetailHandler::new(reader);

        let query = GetComponentDetailQuery {
            cycle_id: CycleId::new(),
            component_type: ComponentType::Objectives,
            user_id: test_user_id(),
        };

        let result = handler.handle(query).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DashboardError::Database(_)));
    }
}
