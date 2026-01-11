#[cfg(test)]
mod tests {
    use crate::domain::foundation::{SessionId, CycleId};
    use crate::domain::dashboard::overview::DashboardOverview;

    #[test]
    fn test_overview_serializes_all_fields() {
        let overview = DashboardOverview {
            session_id: SessionId::new(),
            session_title: "My Decision".to_string(),
            decision_statement: Some("Should we expand to new markets?".to_string()),
            cycle_count: 3,
            active_cycle_id: Some(CycleId::new()),
            objectives: vec![],
            alternatives: vec![],
            consequences_table: None,
            recommendation: None,
            dq_score: None,
            last_updated: chrono::Utc::now(),
        };

        // Should serialize to JSON without error
        let json = serde_json::to_string(&overview);
        assert!(json.is_ok());

        let json_value = json.unwrap();
        // Check for camelCase field names (serde rename_all = "camelCase")
        assert!(json_value.contains("sessionId"));
        assert!(json_value.contains("My Decision"));
    }

    #[test]
    fn test_overview_handles_empty_components() {
        let overview = DashboardOverview {
            session_id: SessionId::new(),
            session_title: "Empty Decision".to_string(),
            decision_statement: None,
            cycle_count: 1,
            active_cycle_id: None,
            objectives: vec![],
            alternatives: vec![],
            consequences_table: None,
            recommendation: None,
            dq_score: None,
            last_updated: chrono::Utc::now(),
        };

        assert_eq!(overview.objectives.len(), 0);
        assert_eq!(overview.alternatives.len(), 0);
        assert!(overview.consequences_table.is_none());
        assert!(overview.recommendation.is_none());
    }

    #[test]
    fn test_overview_includes_session_info() {
        let session_id = SessionId::new();
        let overview = DashboardOverview {
            session_id,
            session_title: "Test Session".to_string(),
            decision_statement: None,
            cycle_count: 2,
            active_cycle_id: None,
            objectives: vec![],
            alternatives: vec![],
            consequences_table: None,
            recommendation: None,
            dq_score: None,
            last_updated: chrono::Utc::now(),
        };

        assert_eq!(overview.session_id, session_id);
        assert_eq!(overview.session_title, "Test Session");
    }

    #[test]
    fn test_overview_includes_cycle_info() {
        let cycle_id = CycleId::new();
        let overview = DashboardOverview {
            session_id: SessionId::new(),
            session_title: "Test".to_string(),
            decision_statement: None,
            cycle_count: 5,
            active_cycle_id: Some(cycle_id),
            objectives: vec![],
            alternatives: vec![],
            consequences_table: None,
            recommendation: None,
            dq_score: None,
            last_updated: chrono::Utc::now(),
        };

        assert_eq!(overview.cycle_count, 5);
        assert_eq!(overview.active_cycle_id, Some(cycle_id));
    }
}
