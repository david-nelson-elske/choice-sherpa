//! CycleProgress value object - Progress tracking for decision cycles.
//!
//! Provides a snapshot of component completion status across a cycle,
//! with utilities for calculating overall progress and finding next steps.

use std::collections::HashMap;

use crate::domain::foundation::{ComponentStatus, ComponentType};
use crate::domain::proact::ComponentSequence;

/// A snapshot of cycle progress across all components.
///
/// This is a read-only value object that provides computed properties
/// about the current state of a cycle's components.
#[derive(Debug, Clone)]
pub struct CycleProgress {
    statuses: HashMap<ComponentType, ComponentStatus>,
}

impl CycleProgress {
    /// Creates a new progress snapshot from component statuses.
    pub fn new(statuses: HashMap<ComponentType, ComponentStatus>) -> Self {
        Self { statuses }
    }

    /// Returns the status of a specific component.
    pub fn status(&self, ct: ComponentType) -> ComponentStatus {
        self.statuses
            .get(&ct)
            .copied()
            .unwrap_or(ComponentStatus::NotStarted)
    }

    /// Returns the number of completed components.
    pub fn completed_count(&self) -> usize {
        self.statuses
            .values()
            .filter(|s| s.is_complete())
            .count()
    }

    /// Returns the total number of required components (excluding optional NotesNextSteps).
    pub fn required_count(&self) -> usize {
        8 // All components except NotesNextSteps are required
    }

    /// Returns the completion percentage (0-100).
    ///
    /// Only counts required components (NotesNextSteps is optional).
    pub fn percent_complete(&self) -> u8 {
        let required_completed = ComponentSequence::all()
            .iter()
            .filter(|ct| **ct != ComponentType::NotesNextSteps)
            .filter(|ct| self.status(**ct).is_complete())
            .count();

        ((required_completed * 100) / self.required_count()) as u8
    }

    /// Returns true if all required components are complete.
    ///
    /// NotesNextSteps is optional and not required for cycle completion.
    pub fn is_complete(&self) -> bool {
        ComponentSequence::all()
            .iter()
            .filter(|ct| **ct != ComponentType::NotesNextSteps)
            .all(|ct| self.status(*ct).is_complete())
    }

    /// Returns the first incomplete component in sequence order.
    ///
    /// Returns None if all required components are complete.
    pub fn first_incomplete(&self) -> Option<ComponentType> {
        ComponentSequence::all()
            .iter()
            .filter(|ct| **ct != ComponentType::NotesNextSteps)
            .find(|ct| !self.status(**ct).is_complete())
            .copied()
    }

    /// Returns a map of all component statuses in sequence order.
    pub fn step_statuses(&self) -> Vec<(ComponentType, ComponentStatus)> {
        ComponentSequence::all()
            .iter()
            .map(|ct| (*ct, self.status(*ct)))
            .collect()
    }

    /// Returns true if any component needs revision.
    pub fn has_revisions_needed(&self) -> bool {
        self.statuses
            .values()
            .any(|s| matches!(s, ComponentStatus::NeedsRevision))
    }

    /// Returns all components that need revision.
    pub fn revisions_needed(&self) -> Vec<ComponentType> {
        ComponentSequence::all()
            .iter()
            .filter(|ct| matches!(self.status(**ct), ComponentStatus::NeedsRevision))
            .copied()
            .collect()
    }

    /// Returns the component currently in progress, if any.
    pub fn current_in_progress(&self) -> Option<ComponentType> {
        ComponentSequence::all()
            .iter()
            .find(|ct| matches!(self.status(**ct), ComponentStatus::InProgress))
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_progress() -> CycleProgress {
        CycleProgress::new(HashMap::new())
    }

    fn progress_with(statuses: Vec<(ComponentType, ComponentStatus)>) -> CycleProgress {
        CycleProgress::new(statuses.into_iter().collect())
    }

    fn all_complete_progress() -> CycleProgress {
        let statuses: HashMap<_, _> = ComponentSequence::all()
            .iter()
            .filter(|ct| **ct != ComponentType::NotesNextSteps)
            .map(|ct| (*ct, ComponentStatus::Complete))
            .collect();
        CycleProgress::new(statuses)
    }

    // ───────────────────────────────────────────────────────────────
    // percent_complete tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn percent_complete_zero_initially() {
        let progress = empty_progress();
        assert_eq!(progress.percent_complete(), 0);
    }

    #[test]
    fn percent_complete_calculates_correctly() {
        // 2 of 8 required components complete = 25%
        let progress = progress_with(vec![
            (ComponentType::IssueRaising, ComponentStatus::Complete),
            (ComponentType::ProblemFrame, ComponentStatus::Complete),
            (ComponentType::Objectives, ComponentStatus::InProgress),
        ]);
        assert_eq!(progress.percent_complete(), 25);
    }

    #[test]
    fn percent_complete_excludes_optional_notes() {
        // All required complete, NotesNextSteps not counted
        let progress = all_complete_progress();
        assert_eq!(progress.percent_complete(), 100);
    }

    #[test]
    fn percent_complete_half_done() {
        // 4 of 8 required components complete = 50%
        let progress = progress_with(vec![
            (ComponentType::IssueRaising, ComponentStatus::Complete),
            (ComponentType::ProblemFrame, ComponentStatus::Complete),
            (ComponentType::Objectives, ComponentStatus::Complete),
            (ComponentType::Alternatives, ComponentStatus::Complete),
        ]);
        assert_eq!(progress.percent_complete(), 50);
    }

    // ───────────────────────────────────────────────────────────────
    // is_complete tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn is_complete_false_when_empty() {
        let progress = empty_progress();
        assert!(!progress.is_complete());
    }

    #[test]
    fn is_complete_true_when_all_required_done() {
        let progress = all_complete_progress();
        assert!(progress.is_complete());
    }

    #[test]
    fn is_complete_false_with_one_incomplete() {
        let mut statuses: HashMap<_, _> = ComponentSequence::all()
            .iter()
            .filter(|ct| **ct != ComponentType::NotesNextSteps)
            .map(|ct| (*ct, ComponentStatus::Complete))
            .collect();
        // Mark one as incomplete
        statuses.insert(ComponentType::Tradeoffs, ComponentStatus::InProgress);

        let progress = CycleProgress::new(statuses);
        assert!(!progress.is_complete());
    }

    // ───────────────────────────────────────────────────────────────
    // first_incomplete tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn first_incomplete_finds_correct_component() {
        let progress = progress_with(vec![
            (ComponentType::IssueRaising, ComponentStatus::Complete),
            (ComponentType::ProblemFrame, ComponentStatus::Complete),
            // Objectives is not complete, so it should be found
        ]);
        assert_eq!(
            progress.first_incomplete(),
            Some(ComponentType::Objectives)
        );
    }

    #[test]
    fn first_incomplete_returns_issue_raising_initially() {
        let progress = empty_progress();
        assert_eq!(
            progress.first_incomplete(),
            Some(ComponentType::IssueRaising)
        );
    }

    #[test]
    fn first_incomplete_returns_none_when_complete() {
        let progress = all_complete_progress();
        assert_eq!(progress.first_incomplete(), None);
    }

    // ───────────────────────────────────────────────────────────────
    // step_statuses tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn step_statuses_maps_all_components() {
        let progress = empty_progress();
        let statuses = progress.step_statuses();

        // Should have all 9 components
        assert_eq!(statuses.len(), 9);

        // Should be in sequence order
        assert_eq!(statuses[0].0, ComponentType::IssueRaising);
        assert_eq!(statuses[8].0, ComponentType::NotesNextSteps);
    }

    #[test]
    fn step_statuses_reflects_actual_statuses() {
        let progress = progress_with(vec![
            (ComponentType::IssueRaising, ComponentStatus::Complete),
            (ComponentType::ProblemFrame, ComponentStatus::InProgress),
        ]);
        let statuses = progress.step_statuses();

        assert_eq!(statuses[0].1, ComponentStatus::Complete);
        assert_eq!(statuses[1].1, ComponentStatus::InProgress);
        assert_eq!(statuses[2].1, ComponentStatus::NotStarted);
    }

    // ───────────────────────────────────────────────────────────────
    // Revision tracking tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn has_revisions_needed_detects_revision() {
        let progress = progress_with(vec![
            (ComponentType::IssueRaising, ComponentStatus::Complete),
            (ComponentType::ProblemFrame, ComponentStatus::NeedsRevision),
        ]);
        assert!(progress.has_revisions_needed());
    }

    #[test]
    fn has_revisions_needed_false_when_none() {
        let progress = progress_with(vec![
            (ComponentType::IssueRaising, ComponentStatus::Complete),
            (ComponentType::ProblemFrame, ComponentStatus::InProgress),
        ]);
        assert!(!progress.has_revisions_needed());
    }

    #[test]
    fn revisions_needed_returns_all_revision_components() {
        let progress = progress_with(vec![
            (ComponentType::IssueRaising, ComponentStatus::NeedsRevision),
            (ComponentType::ProblemFrame, ComponentStatus::Complete),
            (ComponentType::Objectives, ComponentStatus::NeedsRevision),
        ]);

        let revisions = progress.revisions_needed();
        assert_eq!(revisions.len(), 2);
        assert!(revisions.contains(&ComponentType::IssueRaising));
        assert!(revisions.contains(&ComponentType::Objectives));
    }

    // ───────────────────────────────────────────────────────────────
    // Current in progress tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn current_in_progress_finds_component() {
        let progress = progress_with(vec![
            (ComponentType::IssueRaising, ComponentStatus::Complete),
            (ComponentType::ProblemFrame, ComponentStatus::InProgress),
        ]);
        assert_eq!(
            progress.current_in_progress(),
            Some(ComponentType::ProblemFrame)
        );
    }

    #[test]
    fn current_in_progress_returns_none_when_none_active() {
        let progress = progress_with(vec![
            (ComponentType::IssueRaising, ComponentStatus::Complete),
            (ComponentType::ProblemFrame, ComponentStatus::Complete),
        ]);
        assert_eq!(progress.current_in_progress(), None);
    }

    // ───────────────────────────────────────────────────────────────
    // Additional edge cases
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn completed_count_counts_correctly() {
        let progress = progress_with(vec![
            (ComponentType::IssueRaising, ComponentStatus::Complete),
            (ComponentType::ProblemFrame, ComponentStatus::Complete),
            (ComponentType::Objectives, ComponentStatus::InProgress),
            (ComponentType::Alternatives, ComponentStatus::NeedsRevision),
        ]);
        assert_eq!(progress.completed_count(), 2);
    }

    #[test]
    fn required_count_is_eight() {
        let progress = empty_progress();
        assert_eq!(progress.required_count(), 8);
    }
}
