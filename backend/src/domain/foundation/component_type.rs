//! ComponentType enum representing the 9 PrOACT phases.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The 9 PrOACT phases (including Issue Raising and Notes/Next Steps).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    IssueRaising,
    ProblemFrame,
    Objectives,
    Alternatives,
    Consequences,
    Tradeoffs,
    Recommendation,
    DecisionQuality,
    NotesNextSteps,
}

impl ComponentType {
    /// Returns all component types in canonical order.
    pub fn all() -> &'static [ComponentType] {
        &[
            ComponentType::IssueRaising,
            ComponentType::ProblemFrame,
            ComponentType::Objectives,
            ComponentType::Alternatives,
            ComponentType::Consequences,
            ComponentType::Tradeoffs,
            ComponentType::Recommendation,
            ComponentType::DecisionQuality,
            ComponentType::NotesNextSteps,
        ]
    }

    /// Returns the 0-based index of this component in the canonical order.
    pub fn order_index(&self) -> usize {
        Self::all()
            .iter()
            .position(|c| c == self)
            .expect("ComponentType must be in all() array")
    }

    /// Returns the next component in order, if any.
    pub fn next(&self) -> Option<ComponentType> {
        let idx = self.order_index();
        Self::all().get(idx + 1).copied()
    }

    /// Returns the previous component in order, if any.
    pub fn previous(&self) -> Option<ComponentType> {
        let idx = self.order_index();
        if idx == 0 {
            None
        } else {
            Self::all().get(idx - 1).copied()
        }
    }

    /// Returns true if this component comes before another in order.
    pub fn is_before(&self, other: &ComponentType) -> bool {
        self.order_index() < other.order_index()
    }

    /// Returns true if this component comes after another in order.
    pub fn is_after(&self, other: &ComponentType) -> bool {
        self.order_index() > other.order_index()
    }

    /// Returns the display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            ComponentType::IssueRaising => "Issue Raising",
            ComponentType::ProblemFrame => "Problem Frame",
            ComponentType::Objectives => "Objectives",
            ComponentType::Alternatives => "Alternatives",
            ComponentType::Consequences => "Consequences",
            ComponentType::Tradeoffs => "Tradeoffs",
            ComponentType::Recommendation => "Recommendation",
            ComponentType::DecisionQuality => "Decision Quality",
            ComponentType::NotesNextSteps => "Notes & Next Steps",
        }
    }

    /// Returns a short abbreviation (for compact displays).
    pub fn abbreviation(&self) -> &'static str {
        match self {
            ComponentType::IssueRaising => "IR",
            ComponentType::ProblemFrame => "PF",
            ComponentType::Objectives => "OBJ",
            ComponentType::Alternatives => "ALT",
            ComponentType::Consequences => "CON",
            ComponentType::Tradeoffs => "TRD",
            ComponentType::Recommendation => "REC",
            ComponentType::DecisionQuality => "DQ",
            ComponentType::NotesNextSteps => "NNS",
        }
    }
}

impl fmt::Display for ComponentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_9_components() {
        assert_eq!(ComponentType::all().len(), 9);
    }

    #[test]
    fn all_returns_components_in_order() {
        let all = ComponentType::all();
        assert_eq!(all[0], ComponentType::IssueRaising);
        assert_eq!(all[1], ComponentType::ProblemFrame);
        assert_eq!(all[2], ComponentType::Objectives);
        assert_eq!(all[3], ComponentType::Alternatives);
        assert_eq!(all[4], ComponentType::Consequences);
        assert_eq!(all[5], ComponentType::Tradeoffs);
        assert_eq!(all[6], ComponentType::Recommendation);
        assert_eq!(all[7], ComponentType::DecisionQuality);
        assert_eq!(all[8], ComponentType::NotesNextSteps);
    }

    #[test]
    fn order_index_returns_correct_values() {
        assert_eq!(ComponentType::IssueRaising.order_index(), 0);
        assert_eq!(ComponentType::ProblemFrame.order_index(), 1);
        assert_eq!(ComponentType::Objectives.order_index(), 2);
        assert_eq!(ComponentType::Alternatives.order_index(), 3);
        assert_eq!(ComponentType::Consequences.order_index(), 4);
        assert_eq!(ComponentType::Tradeoffs.order_index(), 5);
        assert_eq!(ComponentType::Recommendation.order_index(), 6);
        assert_eq!(ComponentType::DecisionQuality.order_index(), 7);
        assert_eq!(ComponentType::NotesNextSteps.order_index(), 8);
    }

    #[test]
    fn next_returns_correct_component() {
        assert_eq!(
            ComponentType::IssueRaising.next(),
            Some(ComponentType::ProblemFrame)
        );
        assert_eq!(
            ComponentType::DecisionQuality.next(),
            Some(ComponentType::NotesNextSteps)
        );
    }

    #[test]
    fn next_returns_none_for_last() {
        assert_eq!(ComponentType::NotesNextSteps.next(), None);
    }

    #[test]
    fn previous_returns_correct_component() {
        assert_eq!(
            ComponentType::ProblemFrame.previous(),
            Some(ComponentType::IssueRaising)
        );
        assert_eq!(
            ComponentType::NotesNextSteps.previous(),
            Some(ComponentType::DecisionQuality)
        );
    }

    #[test]
    fn previous_returns_none_for_first() {
        assert_eq!(ComponentType::IssueRaising.previous(), None);
    }

    #[test]
    fn is_before_works_correctly() {
        assert!(ComponentType::IssueRaising.is_before(&ComponentType::ProblemFrame));
        assert!(ComponentType::Objectives.is_before(&ComponentType::Consequences));
        assert!(!ComponentType::Tradeoffs.is_before(&ComponentType::Alternatives));
        assert!(!ComponentType::Objectives.is_before(&ComponentType::Objectives)); // Same component
    }

    #[test]
    fn is_after_works_correctly() {
        assert!(ComponentType::ProblemFrame.is_after(&ComponentType::IssueRaising));
        assert!(ComponentType::Consequences.is_after(&ComponentType::Objectives));
        assert!(!ComponentType::Alternatives.is_after(&ComponentType::Tradeoffs));
    }

    #[test]
    fn display_name_returns_readable_text() {
        assert_eq!(ComponentType::IssueRaising.display_name(), "Issue Raising");
        assert_eq!(ComponentType::DecisionQuality.display_name(), "Decision Quality");
        assert_eq!(ComponentType::NotesNextSteps.display_name(), "Notes & Next Steps");
    }

    #[test]
    fn abbreviation_returns_short_code() {
        assert_eq!(ComponentType::IssueRaising.abbreviation(), "IR");
        assert_eq!(ComponentType::DecisionQuality.abbreviation(), "DQ");
        assert_eq!(ComponentType::NotesNextSteps.abbreviation(), "NNS");
    }

    #[test]
    fn display_uses_display_name() {
        assert_eq!(format!("{}", ComponentType::Objectives), "Objectives");
    }

    #[test]
    fn serializes_to_snake_case_json() {
        let json = serde_json::to_string(&ComponentType::IssueRaising).unwrap();
        assert_eq!(json, "\"issue_raising\"");

        let json = serde_json::to_string(&ComponentType::DecisionQuality).unwrap();
        assert_eq!(json, "\"decision_quality\"");
    }

    #[test]
    fn deserializes_from_snake_case_json() {
        let ct: ComponentType = serde_json::from_str("\"problem_frame\"").unwrap();
        assert_eq!(ct, ComponentType::ProblemFrame);

        let ct: ComponentType = serde_json::from_str("\"notes_next_steps\"").unwrap();
        assert_eq!(ct, ComponentType::NotesNextSteps);
    }
}
