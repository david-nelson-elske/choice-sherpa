//! Cycle tree visualization view models.
//!
//! These types support the PrOACT letter-based tree visualization in the UI.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::foundation::{ComponentType, CycleId};

// ════════════════════════════════════════════════════════════════════════════════
// PrOACT Letter Mapping
// ════════════════════════════════════════════════════════════════════════════════

/// A single letter in the PrOACT acronym.
///
/// Maps to one or more ComponentType values for visualization.
///
/// # Letter-to-Component Mapping
/// - **P**: Problem Frame
/// - **R**: Objectives (what Really matters)
/// - **O**: Options/Alternatives
/// - **A**: Analysis/Consequences
/// - **C**: Clear Tradeoffs
/// - **T**: Think Through (Recommendation + Decision Quality)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrOACTLetter {
    /// Problem Frame
    P,
    /// Objectives (what Really matters)
    R,
    /// Options/Alternatives
    O,
    /// Analysis/Consequences
    A,
    /// Clear Tradeoffs
    C,
    /// Think Through (Recommendation + Decision Quality)
    T,
}

impl PrOACTLetter {
    /// Maps this letter to its corresponding component type(s).
    ///
    /// Most letters map to a single component, but T maps to both
    /// Recommendation and DecisionQuality.
    pub fn to_component_types(&self) -> Vec<ComponentType> {
        match self {
            PrOACTLetter::P => vec![ComponentType::ProblemFrame],
            PrOACTLetter::R => vec![ComponentType::Objectives],
            PrOACTLetter::O => vec![ComponentType::Alternatives],
            PrOACTLetter::A => vec![ComponentType::Consequences],
            PrOACTLetter::C => vec![ComponentType::Tradeoffs],
            PrOACTLetter::T => vec![
                ComponentType::Recommendation,
                ComponentType::DecisionQuality,
            ],
        }
    }

    /// Returns all PrOACT letters in order.
    pub fn all() -> &'static [PrOACTLetter] {
        &[
            PrOACTLetter::P,
            PrOACTLetter::R,
            PrOACTLetter::O,
            PrOACTLetter::A,
            PrOACTLetter::C,
            PrOACTLetter::T,
        ]
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Letter Status
// ════════════════════════════════════════════════════════════════════════════════

/// Status of a single letter in the PrOACT visualization.
///
/// Aggregates the status of one or more underlying components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LetterStatus {
    /// Component(s) not yet started
    NotStarted,
    /// At least one component in progress
    InProgress,
    /// All component(s) completed
    Completed,
}

// ════════════════════════════════════════════════════════════════════════════════
// PrOACT Status
// ════════════════════════════════════════════════════════════════════════════════

/// Status for all six PrOACT letters.
///
/// This is the main visualization model shown in each cycle tree node.
///
/// # Example
/// ```text
/// P  r  O  A  C  T
/// ●  ●  ◉  ○  ○  ○
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrOACTStatus {
    /// P - Problem Frame
    pub p: LetterStatus,
    /// r - Objectives (Really matters)
    pub r: LetterStatus,
    /// O - Options/Alternatives
    pub o: LetterStatus,
    /// A - Analysis/Consequences
    pub a: LetterStatus,
    /// C - Clear Tradeoffs
    pub c: LetterStatus,
    /// T - Think Through (Recommendation + DQ)
    pub t: LetterStatus,
}

impl PrOACTStatus {
    /// Creates a new PrOACTStatus with all letters not started.
    pub fn all_not_started() -> Self {
        Self {
            p: LetterStatus::NotStarted,
            r: LetterStatus::NotStarted,
            o: LetterStatus::NotStarted,
            a: LetterStatus::NotStarted,
            c: LetterStatus::NotStarted,
            t: LetterStatus::NotStarted,
        }
    }

    /// Gets the status for a specific letter.
    pub fn get(&self, letter: PrOACTLetter) -> LetterStatus {
        match letter {
            PrOACTLetter::P => self.p,
            PrOACTLetter::R => self.r,
            PrOACTLetter::O => self.o,
            PrOACTLetter::A => self.a,
            PrOACTLetter::C => self.c,
            PrOACTLetter::T => self.t,
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Cycle Tree Node
// ════════════════════════════════════════════════════════════════════════════════

/// A node in the cycle tree visualization.
///
/// Represents a single cycle with its PrOACT status and child branches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleTreeNode {
    /// Unique identifier for this cycle
    pub cycle_id: CycleId,
    /// User-provided or auto-generated label
    pub label: String,
    /// The letter where this cycle branched from parent (None for root)
    pub branch_point: Option<PrOACTLetter>,
    /// Status of all six PrOACT letters
    pub letter_statuses: PrOACTStatus,
    /// Child cycles branched from this one
    pub children: Vec<CycleTreeNode>,
    /// When this cycle was last updated
    pub updated_at: DateTime<Utc>,
}

// ════════════════════════════════════════════════════════════════════════════════
// Branch Metadata
// ════════════════════════════════════════════════════════════════════════════════

/// Metadata for a branched cycle.
///
/// Used to enhance the tree visualization with labels and layout hints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchMetadata {
    /// User-provided label for this branch (e.g., "Remote Option")
    pub branch_label: Option<String>,
    /// Visual position hint for tree layout
    pub position_hint: Option<PositionHint>,
}

impl BranchMetadata {
    /// Creates metadata for a root cycle (no branch).
    pub fn root() -> Self {
        Self {
            branch_label: None,
            position_hint: None,
        }
    }

    /// Creates metadata for a branched cycle.
    pub fn branched(label: Option<String>) -> Self {
        Self {
            branch_label: label,
            position_hint: None,
        }
    }
}

impl Default for BranchMetadata {
    fn default() -> Self {
        Self::root()
    }
}

/// Visual position hint for tree layout algorithms.
///
/// Frontend can use these hints to position nodes, but is not required to.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PositionHint {
    /// Horizontal position
    pub x: f32,
    /// Vertical position
    pub y: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proact_letter_p_maps_to_problem_frame() {
        let letter = PrOACTLetter::P;
        let types = letter.to_component_types();

        assert_eq!(types.len(), 1);
        assert_eq!(types[0], ComponentType::ProblemFrame);
    }

    #[test]
    fn proact_letter_r_maps_to_objectives() {
        let letter = PrOACTLetter::R;
        let types = letter.to_component_types();

        assert_eq!(types.len(), 1);
        assert_eq!(types[0], ComponentType::Objectives);
    }

    #[test]
    fn proact_letter_o_maps_to_alternatives() {
        let letter = PrOACTLetter::O;
        let types = letter.to_component_types();

        assert_eq!(types.len(), 1);
        assert_eq!(types[0], ComponentType::Alternatives);
    }

    #[test]
    fn proact_letter_a_maps_to_consequences() {
        let letter = PrOACTLetter::A;
        let types = letter.to_component_types();

        assert_eq!(types.len(), 1);
        assert_eq!(types[0], ComponentType::Consequences);
    }

    #[test]
    fn proact_letter_c_maps_to_tradeoffs() {
        let letter = PrOACTLetter::C;
        let types = letter.to_component_types();

        assert_eq!(types.len(), 1);
        assert_eq!(types[0], ComponentType::Tradeoffs);
    }

    #[test]
    fn proact_letter_t_maps_to_recommendation_and_decision_quality() {
        let letter = PrOACTLetter::T;
        let types = letter.to_component_types();

        assert_eq!(types.len(), 2);
        assert_eq!(types[0], ComponentType::Recommendation);
        assert_eq!(types[1], ComponentType::DecisionQuality);
    }

    #[test]
    fn letter_status_serializes_as_snake_case() {
        let status = LetterStatus::NotStarted;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"not_started\"");

        let status = LetterStatus::InProgress;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"in_progress\"");

        let status = LetterStatus::Completed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"completed\"");
    }

    #[test]
    fn proact_status_has_all_six_letters() {
        let status = PrOACTStatus {
            p: LetterStatus::Completed,
            r: LetterStatus::Completed,
            o: LetterStatus::InProgress,
            a: LetterStatus::NotStarted,
            c: LetterStatus::NotStarted,
            t: LetterStatus::NotStarted,
        };

        // Should serialize with all fields
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"p\""));
        assert!(json.contains("\"r\""));
        assert!(json.contains("\"o\""));
        assert!(json.contains("\"a\""));
        assert!(json.contains("\"c\""));
        assert!(json.contains("\"t\""));
    }

    #[test]
    fn proact_letter_serializes_as_single_char_uppercase() {
        let letter = PrOACTLetter::P;
        let json = serde_json::to_string(&letter).unwrap();
        assert_eq!(json, "\"P\"");

        let letter = PrOACTLetter::R;
        let json = serde_json::to_string(&letter).unwrap();
        assert_eq!(json, "\"R\"");
    }

    #[test]
    fn proact_status_all_not_started_creates_correctly() {
        let status = PrOACTStatus::all_not_started();

        assert_eq!(status.p, LetterStatus::NotStarted);
        assert_eq!(status.r, LetterStatus::NotStarted);
        assert_eq!(status.o, LetterStatus::NotStarted);
        assert_eq!(status.a, LetterStatus::NotStarted);
        assert_eq!(status.c, LetterStatus::NotStarted);
        assert_eq!(status.t, LetterStatus::NotStarted);
    }

    #[test]
    fn proact_status_get_returns_correct_letter() {
        let status = PrOACTStatus {
            p: LetterStatus::Completed,
            r: LetterStatus::InProgress,
            o: LetterStatus::NotStarted,
            a: LetterStatus::NotStarted,
            c: LetterStatus::NotStarted,
            t: LetterStatus::NotStarted,
        };

        assert_eq!(status.get(PrOACTLetter::P), LetterStatus::Completed);
        assert_eq!(status.get(PrOACTLetter::R), LetterStatus::InProgress);
        assert_eq!(status.get(PrOACTLetter::O), LetterStatus::NotStarted);
    }

    #[test]
    fn branch_metadata_root_creates_no_label() {
        let metadata = BranchMetadata::root();
        assert_eq!(metadata.branch_label, None);
        assert_eq!(metadata.position_hint, None);
    }

    #[test]
    fn branch_metadata_branched_accepts_label() {
        let metadata = BranchMetadata::branched(Some("Test Branch".to_string()));
        assert_eq!(metadata.branch_label, Some("Test Branch".to_string()));
    }

    #[test]
    fn branch_metadata_default_is_root() {
        let metadata = BranchMetadata::default();
        assert_eq!(metadata, BranchMetadata::root());
    }
}
