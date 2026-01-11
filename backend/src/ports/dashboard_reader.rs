use async_trait::async_trait;
use crate::domain::dashboard::{ComponentDetailView, CycleComparison, DashboardOverview};
use crate::domain::foundation::{ComponentType, CycleId, SessionId, UserId};

/// Read-only port for dashboard queries
#[async_trait]
pub trait DashboardReader: Send + Sync {
    /// Gets the main dashboard overview for a session
    /// If cycle_id is None, uses the most recently updated active cycle
    async fn get_overview(
        &self,
        session_id: SessionId,
        cycle_id: Option<CycleId>,
        user_id: &UserId,
    ) -> Result<DashboardOverview, DashboardError>;

    /// Gets detailed view for a specific component
    async fn get_component_detail(
        &self,
        cycle_id: CycleId,
        component_type: ComponentType,
        user_id: &UserId,
    ) -> Result<ComponentDetailView, DashboardError>;

    /// Compares multiple cycles
    async fn compare_cycles(
        &self,
        cycle_ids: &[CycleId],
        user_id: &UserId,
    ) -> Result<CycleComparison, DashboardError>;
}

/// Errors that can occur during dashboard operations
#[derive(Debug, thiserror::Error)]
pub enum DashboardError {
    #[error("Session not found: {0}")]
    SessionNotFound(SessionId),

    #[error("Cycle not found: {0}")]
    CycleNotFound(CycleId),

    #[error("Component not found: {0:?}")]
    ComponentNotFound(ComponentType),

    #[error("Unauthorized access to session")]
    Unauthorized,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Database error: {0}")]
    Database(String),
}

impl From<sqlx::Error> for DashboardError {
    fn from(err: sqlx::Error) -> Self {
        DashboardError::Database(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::foundation::{SessionId, CycleId, ComponentType};

    // Mock implementation for testing
    struct MockDashboardReader;

    #[async_trait]
    impl DashboardReader for MockDashboardReader {
        async fn get_overview(
            &self,
            _session_id: SessionId,
            _cycle_id: Option<CycleId>,
            _user_id: &UserId,
        ) -> Result<DashboardOverview, DashboardError> {
            unimplemented!("Mock for testing trait only")
        }

        async fn get_component_detail(
            &self,
            _cycle_id: CycleId,
            _component_type: ComponentType,
            _user_id: &UserId,
        ) -> Result<ComponentDetailView, DashboardError> {
            unimplemented!("Mock for testing trait only")
        }

        async fn compare_cycles(
            &self,
            _cycle_ids: &[CycleId],
            _user_id: &UserId,
        ) -> Result<CycleComparison, DashboardError> {
            unimplemented!("Mock for testing trait only")
        }
    }

    #[test]
    fn test_reader_trait_compiles() {
        // This test ensures the trait is properly defined
        let _reader: Box<dyn DashboardReader> = Box::new(MockDashboardReader);
    }

    #[test]
    fn test_error_conversion_from_sqlx() {
        let sqlx_error = sqlx::Error::RowNotFound;
        let dashboard_error: DashboardError = sqlx_error.into();

        match dashboard_error {
            DashboardError::Database(_) => {},
            _ => panic!("Expected Database error"),
        }
    }

    #[test]
    fn test_error_messages() {
        let session_id = SessionId::new();
        let error = DashboardError::SessionNotFound(session_id);
        let msg = format!("{}", error);
        assert!(msg.contains("Session not found"));

        let cycle_id = CycleId::new();
        let error = DashboardError::CycleNotFound(cycle_id);
        let msg = format!("{}", error);
        assert!(msg.contains("Cycle not found"));

        let error = DashboardError::ComponentNotFound(ComponentType::Objectives);
        let msg = format!("{}", error);
        assert!(msg.contains("Component not found"));

        let error = DashboardError::Unauthorized;
        let msg = format!("{}", error);
        assert_eq!(msg, "Unauthorized access to session");
    }
}
