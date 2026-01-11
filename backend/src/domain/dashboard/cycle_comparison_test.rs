#[cfg(test)]
mod tests {
    use crate::domain::foundation::{ComponentType, CycleId};
    use crate::domain::dashboard::cycle_comparison::*;

    fn create_test_comparison() -> CycleComparison {
        let cycle1 = CycleId::new();
        let cycle2 = CycleId::new();

        CycleComparison {
            cycles: vec![
                CycleComparisonItem {
                    cycle_id: cycle1,
                    branch_point: None,
                    progress: CycleProgressSnapshot {
                        completed_count: 5,
                        total_count: 9,
                        percent_complete: 55,
                        current_step: Some(ComponentType::Tradeoffs),
                    },
                    component_summaries: vec![
                        ComponentComparisonSummary {
                            component_type: ComponentType::Objectives,
                            summary: "3 objectives defined".to_string(),
                            differs_from_others: false,
                        },
                    ],
                },
                CycleComparisonItem {
                    cycle_id: cycle2,
                    branch_point: Some(ComponentType::Alternatives),
                    progress: CycleProgressSnapshot {
                        completed_count: 4,
                        total_count: 9,
                        percent_complete: 44,
                        current_step: Some(ComponentType::Consequences),
                    },
                    component_summaries: vec![
                        ComponentComparisonSummary {
                            component_type: ComponentType::Objectives,
                            summary: "3 objectives defined".to_string(),
                            differs_from_others: false,
                        },
                    ],
                },
            ],
            differences: vec![
                ComparisonDifference {
                    component_type: ComponentType::Alternatives,
                    cycle_id: cycle2,
                    description: "Different alternatives selected".to_string(),
                    significance: DifferenceSignificance::Major,
                },
            ],
            summary: ComparisonSummary {
                total_cycles: 2,
                components_with_differences: 1,
                most_different_cycle: Some(cycle2),
                recommendation_differs: false,
            },
        }
    }

    #[test]
    fn test_comparison_includes_all_cycles() {
        let comparison = create_test_comparison();
        assert_eq!(comparison.cycles.len(), 2);
        assert_eq!(comparison.summary.total_cycles, 2);
    }

    #[test]
    fn test_comparison_identifies_differences() {
        let comparison = create_test_comparison();
        assert_eq!(comparison.differences.len(), 1);
        assert_eq!(comparison.summary.components_with_differences, 1);
    }

    #[test]
    fn test_comparison_summary_counts() {
        let comparison = create_test_comparison();
        assert_eq!(comparison.summary.total_cycles, 2);
        assert_eq!(comparison.summary.components_with_differences, 1);
        assert!(!comparison.summary.recommendation_differs);
    }

    #[test]
    fn test_comparison_item_has_progress() {
        let comparison = create_test_comparison();
        let item = &comparison.cycles[0];
        assert_eq!(item.progress.completed_count, 5);
        assert_eq!(item.progress.total_count, 9);
        assert_eq!(item.progress.percent_complete, 55);
    }

    #[test]
    fn test_comparison_item_has_branch_point() {
        let comparison = create_test_comparison();
        let item1 = &comparison.cycles[0];
        let item2 = &comparison.cycles[1];

        assert_eq!(item1.branch_point, None);
        assert_eq!(item2.branch_point, Some(ComponentType::Alternatives));
    }

    #[test]
    fn test_difference_has_significance() {
        let comparison = create_test_comparison();
        let diff = &comparison.differences[0];
        assert_eq!(diff.significance, DifferenceSignificance::Major);
    }

    #[test]
    fn test_difference_significance_values() {
        assert_eq!(DifferenceSignificance::Minor, DifferenceSignificance::Minor);
        assert_ne!(DifferenceSignificance::Minor, DifferenceSignificance::Major);
        assert_ne!(DifferenceSignificance::Moderate, DifferenceSignificance::Major);
    }
}
