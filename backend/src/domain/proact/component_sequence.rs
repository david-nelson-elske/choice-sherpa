//! ComponentSequence - Centralized ordering logic for PrOACT components.
//!
//! The PrOACT framework has a defined progression through 9 components. This module
//! consolidates all ordering logic into a single location to avoid duplication
//! across the codebase.
//!
//! # Component Order
//!
//! 1. IssueRaising → 2. ProblemFrame → 3. Objectives → 4. Alternatives →
//! 5. Consequences → 6. Tradeoffs → 7. Recommendation → 8. DecisionQuality →
//! 9. NotesNextSteps
//!
//! # Usage
//!
//! ```ignore
//! use crate::domain::proact::ComponentSequence;
//! use crate::domain::foundation::ComponentType;
//!
//! // Get the standard order
//! let all = ComponentSequence::all();
//!
//! // Navigation
//! let next = ComponentSequence::next(ComponentType::Objectives); // Some(Alternatives)
//! let prev = ComponentSequence::previous(ComponentType::Objectives); // Some(ProblemFrame)
//!
//! // Queries
//! let idx = ComponentSequence::order_index(ComponentType::Tradeoffs); // 5
//! let is_before = ComponentSequence::is_before(ComponentType::Objectives, ComponentType::Consequences); // true
//!
//! // Get all components up to and including a specific point
//! let up_to = ComponentSequence::components_up_to(ComponentType::Alternatives);
//! // [IssueRaising, ProblemFrame, Objectives, Alternatives]
//! ```

use crate::domain::foundation::ComponentType;

/// Central location for component ordering logic.
///
/// This struct provides static methods for querying the PrOACT component sequence.
/// All ordering-related logic should go through this type to maintain DRY.
pub struct ComponentSequence;

impl ComponentSequence {
    /// The canonical order of PrOACT components.
    pub const ORDER: [ComponentType; 9] = [
        ComponentType::IssueRaising,
        ComponentType::ProblemFrame,
        ComponentType::Objectives,
        ComponentType::Alternatives,
        ComponentType::Consequences,
        ComponentType::Tradeoffs,
        ComponentType::Recommendation,
        ComponentType::DecisionQuality,
        ComponentType::NotesNextSteps,
    ];

    /// Returns all component types in order.
    pub fn all() -> &'static [ComponentType; 9] {
        &Self::ORDER
    }

    /// Returns the 0-based index of a component type in the sequence.
    ///
    /// # Panics
    ///
    /// This function will never panic because all ComponentType variants are in ORDER.
    #[inline]
    pub fn order_index(ct: ComponentType) -> usize {
        Self::ORDER
            .iter()
            .position(|&c| c == ct)
            .expect("All ComponentType variants must be in ORDER")
    }

    /// Returns the next component in the sequence, or None if at the end.
    ///
    /// # Example
    ///
    /// ```ignore
    /// assert_eq!(
    ///     ComponentSequence::next(ComponentType::Objectives),
    ///     Some(ComponentType::Alternatives)
    /// );
    /// assert_eq!(
    ///     ComponentSequence::next(ComponentType::NotesNextSteps),
    ///     None
    /// );
    /// ```
    pub fn next(ct: ComponentType) -> Option<ComponentType> {
        let idx = Self::order_index(ct);
        Self::ORDER.get(idx + 1).copied()
    }

    /// Returns the previous component in the sequence, or None if at the start.
    ///
    /// # Example
    ///
    /// ```ignore
    /// assert_eq!(
    ///     ComponentSequence::previous(ComponentType::Objectives),
    ///     Some(ComponentType::ProblemFrame)
    /// );
    /// assert_eq!(
    ///     ComponentSequence::previous(ComponentType::IssueRaising),
    ///     None
    /// );
    /// ```
    pub fn previous(ct: ComponentType) -> Option<ComponentType> {
        let idx = Self::order_index(ct);
        if idx > 0 {
            Self::ORDER.get(idx - 1).copied()
        } else {
            None
        }
    }

    /// Returns true if component `a` comes before component `b` in the sequence.
    ///
    /// # Example
    ///
    /// ```ignore
    /// assert!(ComponentSequence::is_before(
    ///     ComponentType::Objectives,
    ///     ComponentType::Consequences
    /// ));
    /// assert!(!ComponentSequence::is_before(
    ///     ComponentType::Tradeoffs,
    ///     ComponentType::Alternatives
    /// ));
    /// ```
    pub fn is_before(a: ComponentType, b: ComponentType) -> bool {
        Self::order_index(a) < Self::order_index(b)
    }

    /// Returns true if component `a` comes after component `b` in the sequence.
    pub fn is_after(a: ComponentType, b: ComponentType) -> bool {
        Self::order_index(a) > Self::order_index(b)
    }

    /// Returns all components up to and including the specified component.
    ///
    /// Useful for determining which components should be copied when branching a cycle.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let up_to = ComponentSequence::components_up_to(ComponentType::Alternatives);
    /// assert_eq!(up_to, vec![
    ///     ComponentType::IssueRaising,
    ///     ComponentType::ProblemFrame,
    ///     ComponentType::Objectives,
    ///     ComponentType::Alternatives,
    /// ]);
    /// ```
    pub fn components_up_to(ct: ComponentType) -> Vec<ComponentType> {
        let idx = Self::order_index(ct);
        Self::ORDER[..=idx].to_vec()
    }

    /// Returns all components after the specified component.
    ///
    /// Useful for determining which components remain to be completed.
    pub fn components_after(ct: ComponentType) -> Vec<ComponentType> {
        let idx = Self::order_index(ct);
        if idx + 1 < Self::ORDER.len() {
            Self::ORDER[idx + 1..].to_vec()
        } else {
            Vec::new()
        }
    }

    /// Returns the prerequisite component (the one that must be completed before this one).
    ///
    /// This is an alias for `previous()` that makes intent clearer in business logic.
    pub fn prerequisite(ct: ComponentType) -> Option<ComponentType> {
        Self::previous(ct)
    }

    /// Returns the first component in the sequence.
    pub fn first() -> ComponentType {
        Self::ORDER[0]
    }

    /// Returns the last component in the sequence.
    pub fn last() -> ComponentType {
        Self::ORDER[Self::ORDER.len() - 1]
    }

    /// Returns true if this is the first component in the sequence.
    pub fn is_first(ct: ComponentType) -> bool {
        ct == Self::first()
    }

    /// Returns true if this is the last component in the sequence.
    pub fn is_last(ct: ComponentType) -> bool {
        ct == Self::last()
    }

    /// Returns the distance (number of steps) between two components.
    ///
    /// Returns a positive value if `to` is after `from`, negative if before, 0 if same.
    pub fn distance(from: ComponentType, to: ComponentType) -> i32 {
        Self::order_index(to) as i32 - Self::order_index(from) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_contains_all_nine_components() {
        assert_eq!(ComponentSequence::ORDER.len(), 9);
    }

    #[test]
    fn all_returns_order() {
        assert_eq!(ComponentSequence::all(), &ComponentSequence::ORDER);
    }

    #[test]
    fn order_index_returns_correct_position() {
        assert_eq!(ComponentSequence::order_index(ComponentType::IssueRaising), 0);
        assert_eq!(ComponentSequence::order_index(ComponentType::ProblemFrame), 1);
        assert_eq!(ComponentSequence::order_index(ComponentType::Objectives), 2);
        assert_eq!(ComponentSequence::order_index(ComponentType::Alternatives), 3);
        assert_eq!(ComponentSequence::order_index(ComponentType::Consequences), 4);
        assert_eq!(ComponentSequence::order_index(ComponentType::Tradeoffs), 5);
        assert_eq!(ComponentSequence::order_index(ComponentType::Recommendation), 6);
        assert_eq!(ComponentSequence::order_index(ComponentType::DecisionQuality), 7);
        assert_eq!(ComponentSequence::order_index(ComponentType::NotesNextSteps), 8);
    }

    #[test]
    fn next_returns_subsequent_component() {
        assert_eq!(
            ComponentSequence::next(ComponentType::IssueRaising),
            Some(ComponentType::ProblemFrame)
        );
        assert_eq!(
            ComponentSequence::next(ComponentType::Objectives),
            Some(ComponentType::Alternatives)
        );
        assert_eq!(
            ComponentSequence::next(ComponentType::DecisionQuality),
            Some(ComponentType::NotesNextSteps)
        );
    }

    #[test]
    fn next_returns_none_for_last_component() {
        assert_eq!(ComponentSequence::next(ComponentType::NotesNextSteps), None);
    }

    #[test]
    fn previous_returns_preceding_component() {
        assert_eq!(
            ComponentSequence::previous(ComponentType::ProblemFrame),
            Some(ComponentType::IssueRaising)
        );
        assert_eq!(
            ComponentSequence::previous(ComponentType::NotesNextSteps),
            Some(ComponentType::DecisionQuality)
        );
    }

    #[test]
    fn previous_returns_none_for_first_component() {
        assert_eq!(ComponentSequence::previous(ComponentType::IssueRaising), None);
    }

    #[test]
    fn is_before_correctly_compares() {
        assert!(ComponentSequence::is_before(
            ComponentType::IssueRaising,
            ComponentType::ProblemFrame
        ));
        assert!(ComponentSequence::is_before(
            ComponentType::Objectives,
            ComponentType::Tradeoffs
        ));
        assert!(!ComponentSequence::is_before(
            ComponentType::Tradeoffs,
            ComponentType::Objectives
        ));
        assert!(!ComponentSequence::is_before(
            ComponentType::Objectives,
            ComponentType::Objectives
        ));
    }

    #[test]
    fn is_after_correctly_compares() {
        assert!(ComponentSequence::is_after(
            ComponentType::Tradeoffs,
            ComponentType::Objectives
        ));
        assert!(!ComponentSequence::is_after(
            ComponentType::Objectives,
            ComponentType::Tradeoffs
        ));
    }

    #[test]
    fn components_up_to_returns_inclusive_slice() {
        let up_to = ComponentSequence::components_up_to(ComponentType::Alternatives);
        assert_eq!(up_to.len(), 4);
        assert_eq!(up_to[0], ComponentType::IssueRaising);
        assert_eq!(up_to[3], ComponentType::Alternatives);
    }

    #[test]
    fn components_up_to_first_returns_single() {
        let up_to = ComponentSequence::components_up_to(ComponentType::IssueRaising);
        assert_eq!(up_to.len(), 1);
        assert_eq!(up_to[0], ComponentType::IssueRaising);
    }

    #[test]
    fn components_up_to_last_returns_all() {
        let up_to = ComponentSequence::components_up_to(ComponentType::NotesNextSteps);
        assert_eq!(up_to.len(), 9);
    }

    #[test]
    fn components_after_returns_remaining() {
        let after = ComponentSequence::components_after(ComponentType::Consequences);
        assert_eq!(after.len(), 4);
        assert_eq!(after[0], ComponentType::Tradeoffs);
        assert_eq!(after[3], ComponentType::NotesNextSteps);
    }

    #[test]
    fn components_after_last_returns_empty() {
        let after = ComponentSequence::components_after(ComponentType::NotesNextSteps);
        assert!(after.is_empty());
    }

    #[test]
    fn prerequisite_is_alias_for_previous() {
        assert_eq!(
            ComponentSequence::prerequisite(ComponentType::Objectives),
            ComponentSequence::previous(ComponentType::Objectives)
        );
    }

    #[test]
    fn first_returns_issue_raising() {
        assert_eq!(ComponentSequence::first(), ComponentType::IssueRaising);
    }

    #[test]
    fn last_returns_notes_next_steps() {
        assert_eq!(ComponentSequence::last(), ComponentType::NotesNextSteps);
    }

    #[test]
    fn is_first_and_is_last_work_correctly() {
        assert!(ComponentSequence::is_first(ComponentType::IssueRaising));
        assert!(!ComponentSequence::is_first(ComponentType::ProblemFrame));
        assert!(ComponentSequence::is_last(ComponentType::NotesNextSteps));
        assert!(!ComponentSequence::is_last(ComponentType::DecisionQuality));
    }

    #[test]
    fn distance_calculates_correctly() {
        assert_eq!(
            ComponentSequence::distance(ComponentType::IssueRaising, ComponentType::Objectives),
            2
        );
        assert_eq!(
            ComponentSequence::distance(ComponentType::Objectives, ComponentType::IssueRaising),
            -2
        );
        assert_eq!(
            ComponentSequence::distance(ComponentType::Alternatives, ComponentType::Alternatives),
            0
        );
        assert_eq!(
            ComponentSequence::distance(ComponentType::IssueRaising, ComponentType::NotesNextSteps),
            8
        );
    }
}
