//! Decision Quality Calculator - Scoring and analysis of DQ elements.

use serde::{Deserialize, Serialize};

use crate::domain::foundation::Percentage;

/// The seven standard Decision Quality elements.
pub const DQ_ELEMENT_NAMES: &[&str] = &[
    "Helpful Problem Frame",
    "Clear Objectives",
    "Creative Alternatives",
    "Reliable Consequence Information",
    "Logically Correct Reasoning",
    "Clear Tradeoffs",
    "Commitment to Follow Through",
];

/// Threshold for "acceptable" decision quality (all elements >= 80%).
pub const DQ_ACCEPTABLE_THRESHOLD: u8 = 80;

/// A scored Decision Quality element.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DQElement {
    pub name: String,
    pub score: Percentage,
    pub rationale: Option<String>,
    pub improvement_path: Option<String>,
}

impl DQElement {
    /// Creates a new DQ element with a score.
    pub fn new(name: impl Into<String>, score: u8) -> Self {
        Self {
            name: name.into(),
            score: Percentage::new(score),
            rationale: None,
            improvement_path: None,
        }
    }

    /// Creates a DQ element with rationale and improvement path.
    pub fn with_details(
        name: impl Into<String>,
        score: u8,
        rationale: impl Into<String>,
        improvement_path: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            score: Percentage::new(score),
            rationale: Some(rationale.into()),
            improvement_path: Some(improvement_path.into()),
        }
    }
}

/// Priority level for improvement actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

impl Priority {
    /// Returns the display label for this priority.
    pub fn label(&self) -> &'static str {
        match self {
            Priority::Critical => "Critical",
            Priority::High => "High",
            Priority::Medium => "Medium",
            Priority::Low => "Low",
        }
    }
}

/// Calculator for Decision Quality scores and analysis.
pub struct DQCalculator;

impl DQCalculator {
    /// Computes the overall DQ score as the minimum of all element scores.
    ///
    /// # Edge Cases
    /// - Empty elements: Returns 0%
    /// - Single element: Returns that element's score
    pub fn compute_overall(elements: &[DQElement]) -> Percentage {
        if elements.is_empty() {
            return Percentage::ZERO;
        }

        let min_score = elements
            .iter()
            .map(|e| e.score.value())
            .min()
            .unwrap_or(0);

        Percentage::new(min_score)
    }

    /// Checks if all 7 standard DQ elements are present.
    pub fn has_all_elements(elements: &[DQElement]) -> bool {
        DQ_ELEMENT_NAMES.iter().all(|required_name| {
            elements.iter().any(|e| e.name == *required_name)
        })
    }

    /// Returns the names of missing standard DQ elements.
    pub fn missing_elements(elements: &[DQElement]) -> Vec<&'static str> {
        DQ_ELEMENT_NAMES
            .iter()
            .filter(|required| !elements.iter().any(|e| e.name == **required))
            .copied()
            .collect()
    }

    /// Checks if decision quality is "acceptable" (all elements >= 80%).
    pub fn is_acceptable(elements: &[DQElement]) -> bool {
        !elements.is_empty() && elements.iter().all(|e| e.score.value() >= DQ_ACCEPTABLE_THRESHOLD)
    }

    /// Computes the improvement priority based on score.
    pub fn compute_priority(score: Percentage) -> Priority {
        match score.value() {
            0..=30 => Priority::Critical,
            31..=50 => Priority::High,
            51..=70 => Priority::Medium,
            // Percentage is constrained to 0-100, so 71+ is always Low
            _ => Priority::Low,
        }
    }

    /// Finds the weakest element (lowest score).
    pub fn find_weakest(elements: &[DQElement]) -> Option<&DQElement> {
        elements.iter().min_by_key(|e| e.score.value())
    }

    /// Returns elements sorted by score (lowest first).
    pub fn sorted_by_priority(elements: &[DQElement]) -> Vec<&DQElement> {
        let mut sorted: Vec<_> = elements.iter().collect();
        sorted.sort_by_key(|e| e.score.value());
        sorted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dq_overall_empty_elements() {
        let elements: Vec<DQElement> = vec![];
        let overall = DQCalculator::compute_overall(&elements);
        assert_eq!(overall.value(), 0);
    }

    #[test]
    fn dq_overall_is_minimum() {
        let elements = vec![
            DQElement::new("Element A", 90),
            DQElement::new("Element B", 60), // Minimum
            DQElement::new("Element C", 85),
        ];

        let overall = DQCalculator::compute_overall(&elements);
        assert_eq!(overall.value(), 60);
    }

    #[test]
    fn dq_overall_single_element() {
        let elements = vec![DQElement::new("Only Element", 75)];
        let overall = DQCalculator::compute_overall(&elements);
        assert_eq!(overall.value(), 75);
    }

    #[test]
    fn dq_overall_all_100() {
        let elements = vec![
            DQElement::new("A", 100),
            DQElement::new("B", 100),
            DQElement::new("C", 100),
        ];

        let overall = DQCalculator::compute_overall(&elements);
        assert_eq!(overall.value(), 100);
    }

    #[test]
    fn dq_overall_one_zero() {
        let elements = vec![
            DQElement::new("A", 90),
            DQElement::new("B", 0), // Zero pulls overall down
            DQElement::new("C", 85),
        ];

        let overall = DQCalculator::compute_overall(&elements);
        assert_eq!(overall.value(), 0);
    }

    #[test]
    fn dq_has_all_elements_complete() {
        let elements: Vec<DQElement> = DQ_ELEMENT_NAMES
            .iter()
            .map(|name| DQElement::new(*name, 80))
            .collect();

        assert!(DQCalculator::has_all_elements(&elements));
    }

    #[test]
    fn dq_has_all_elements_missing_one() {
        let elements: Vec<DQElement> = DQ_ELEMENT_NAMES[0..6]
            .iter()
            .map(|name| DQElement::new(*name, 80))
            .collect();

        assert!(!DQCalculator::has_all_elements(&elements));
    }

    #[test]
    fn dq_missing_elements_finds_missing() {
        let elements = vec![
            DQElement::new("Helpful Problem Frame", 80),
            DQElement::new("Clear Objectives", 80),
        ];

        let missing = DQCalculator::missing_elements(&elements);
        assert_eq!(missing.len(), 5);
        assert!(missing.contains(&"Creative Alternatives"));
        assert!(missing.contains(&"Commitment to Follow Through"));
    }

    #[test]
    fn dq_missing_elements_none_when_complete() {
        let elements: Vec<DQElement> = DQ_ELEMENT_NAMES
            .iter()
            .map(|name| DQElement::new(*name, 80))
            .collect();

        let missing = DQCalculator::missing_elements(&elements);
        assert!(missing.is_empty());
    }

    #[test]
    fn dq_is_acceptable_all_above_threshold() {
        let elements = vec![
            DQElement::new("A", 80),
            DQElement::new("B", 85),
            DQElement::new("C", 90),
        ];

        assert!(DQCalculator::is_acceptable(&elements));
    }

    #[test]
    fn dq_is_acceptable_one_below_threshold() {
        let elements = vec![
            DQElement::new("A", 80),
            DQElement::new("B", 79), // Below 80%
            DQElement::new("C", 90),
        ];

        assert!(!DQCalculator::is_acceptable(&elements));
    }

    #[test]
    fn dq_is_acceptable_empty_is_false() {
        let elements: Vec<DQElement> = vec![];
        assert!(!DQCalculator::is_acceptable(&elements));
    }

    #[test]
    fn dq_is_acceptable_exactly_80() {
        let elements = vec![
            DQElement::new("A", 80),
            DQElement::new("B", 80),
            DQElement::new("C", 80),
        ];

        assert!(DQCalculator::is_acceptable(&elements));
    }

    #[test]
    fn dq_compute_priority_critical() {
        assert_eq!(DQCalculator::compute_priority(Percentage::new(0)), Priority::Critical);
        assert_eq!(DQCalculator::compute_priority(Percentage::new(15)), Priority::Critical);
        assert_eq!(DQCalculator::compute_priority(Percentage::new(30)), Priority::Critical);
    }

    #[test]
    fn dq_compute_priority_high() {
        assert_eq!(DQCalculator::compute_priority(Percentage::new(31)), Priority::High);
        assert_eq!(DQCalculator::compute_priority(Percentage::new(40)), Priority::High);
        assert_eq!(DQCalculator::compute_priority(Percentage::new(50)), Priority::High);
    }

    #[test]
    fn dq_compute_priority_medium() {
        assert_eq!(DQCalculator::compute_priority(Percentage::new(51)), Priority::Medium);
        assert_eq!(DQCalculator::compute_priority(Percentage::new(60)), Priority::Medium);
        assert_eq!(DQCalculator::compute_priority(Percentage::new(70)), Priority::Medium);
    }

    #[test]
    fn dq_compute_priority_low() {
        assert_eq!(DQCalculator::compute_priority(Percentage::new(71)), Priority::Low);
        assert_eq!(DQCalculator::compute_priority(Percentage::new(85)), Priority::Low);
        assert_eq!(DQCalculator::compute_priority(Percentage::new(100)), Priority::Low);
    }

    #[test]
    fn dq_find_weakest_returns_lowest() {
        let elements = vec![
            DQElement::new("A", 90),
            DQElement::new("B", 60),
            DQElement::new("C", 85),
        ];

        let weakest = DQCalculator::find_weakest(&elements).unwrap();
        assert_eq!(weakest.name, "B");
        assert_eq!(weakest.score.value(), 60);
    }

    #[test]
    fn dq_find_weakest_empty() {
        let elements: Vec<DQElement> = vec![];
        assert!(DQCalculator::find_weakest(&elements).is_none());
    }

    #[test]
    fn dq_sorted_by_priority_orders_lowest_first() {
        let elements = vec![
            DQElement::new("A", 90),
            DQElement::new("B", 50),
            DQElement::new("C", 70),
        ];

        let sorted = DQCalculator::sorted_by_priority(&elements);
        assert_eq!(sorted[0].name, "B"); // 50
        assert_eq!(sorted[1].name, "C"); // 70
        assert_eq!(sorted[2].name, "A"); // 90
    }

    #[test]
    fn dq_element_with_details_stores_all() {
        let element = DQElement::with_details(
            "Helpful Problem Frame",
            75,
            "The problem is well-defined",
            "Consider broader stakeholder impacts",
        );

        assert_eq!(element.name, "Helpful Problem Frame");
        assert_eq!(element.score.value(), 75);
        assert_eq!(element.rationale.as_deref(), Some("The problem is well-defined"));
        assert_eq!(
            element.improvement_path.as_deref(),
            Some("Consider broader stakeholder impacts")
        );
    }

    #[test]
    fn dq_element_serializes() {
        let element = DQElement::new("Test Element", 80);
        let json = serde_json::to_string(&element).unwrap();
        assert!(json.contains("Test Element"));
        assert!(json.contains("80"));
    }
}
