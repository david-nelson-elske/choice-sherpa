#[cfg(test)]
mod tests {
    use crate::domain::foundation::{ComponentId, ComponentStatus, ComponentType, CycleId};
    use crate::domain::dashboard::component_detail::ComponentDetailView;
    use serde_json::json;

    fn create_test_component_detail() -> ComponentDetailView {
        ComponentDetailView {
            component_id: ComponentId::new(),
            cycle_id: CycleId::new(),
            component_type: ComponentType::Objectives,
            status: ComponentStatus::Complete,
            structured_output: json!({
                "objectives": [
                    {"id": "obj1", "description": "Minimize cost"}
                ]
            }),
            conversation_message_count: 5,
            last_message_at: Some(chrono::Utc::now()),
            can_branch: true,
            can_revise: true,
            previous_component: Some(ComponentType::ProblemFrame),
            next_component: Some(ComponentType::Alternatives),
        }
    }

    #[test]
    fn test_component_detail_has_type() {
        let detail = create_test_component_detail();
        assert_eq!(detail.component_type, ComponentType::Objectives);
    }

    #[test]
    fn test_component_detail_has_status() {
        let detail = create_test_component_detail();
        assert_eq!(detail.status, ComponentStatus::Complete);
    }

    #[test]
    fn test_component_detail_has_structured_output() {
        let detail = create_test_component_detail();
        assert!(detail.structured_output.is_object());
        assert!(detail.structured_output.get("objectives").is_some());
    }

    #[test]
    fn test_component_detail_has_navigation() {
        let detail = create_test_component_detail();
        assert_eq!(detail.previous_component, Some(ComponentType::ProblemFrame));
        assert_eq!(detail.next_component, Some(ComponentType::Alternatives));
    }

    #[test]
    fn test_component_detail_display_name() {
        let detail = create_test_component_detail();
        let name = detail.display_name();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_component_detail_is_started() {
        let detail = create_test_component_detail();
        assert!(detail.is_started());
    }

    #[test]
    fn test_component_detail_is_complete() {
        let detail = create_test_component_detail();
        assert!(detail.is_complete());
    }

    #[test]
    fn test_component_detail_can_branch() {
        let detail = create_test_component_detail();
        assert!(detail.can_branch);
    }

    #[test]
    fn test_component_detail_can_revise() {
        let detail = create_test_component_detail();
        assert!(detail.can_revise);
    }
}
