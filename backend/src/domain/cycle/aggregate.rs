//! Cycle aggregate - The root entity for decision cycles.
//!
//! A Cycle owns all components and manages their lifecycle through the PrOACT
//! framework. Cycles can be branched to explore alternative paths.

use std::collections::HashMap;

use crate::domain::foundation::{
    ComponentStatus, ComponentType, CycleId, CycleStatus, DomainError, ErrorCode, SessionId,
    Timestamp,
};
use crate::domain::proact::{ComponentSequence, ComponentVariant};

use super::{BranchMetadata, CycleEvent};

/// The Cycle aggregate root.
///
/// A Cycle represents a complete or partial path through PrOACT.
/// It owns all component instances and manages their state transitions.
#[derive(Debug, Clone)]
pub struct Cycle {
    id: CycleId,
    session_id: SessionId,
    parent_cycle_id: Option<CycleId>,
    branch_point: Option<ComponentType>,
    /// Metadata for branch visualization (label, position hints)
    branch_metadata: BranchMetadata,
    status: CycleStatus,
    current_step: ComponentType,
    components: HashMap<ComponentType, ComponentVariant>,
    created_at: Timestamp,
    updated_at: Timestamp,
    domain_events: Vec<CycleEvent>,
}

impl Cycle {
    /// Creates a new cycle for a session.
    pub fn new(session_id: SessionId) -> Self {
        let id = CycleId::new();
        let now = Timestamp::now();

        // Initialize all 9 components
        let mut components = HashMap::new();
        for ct in ComponentSequence::all() {
            components.insert(*ct, ComponentVariant::new(*ct));
        }

        let mut cycle = Self {
            id,
            session_id,
            parent_cycle_id: None,
            branch_point: None,
            branch_metadata: BranchMetadata::root(),
            status: CycleStatus::Active,
            current_step: ComponentSequence::first(),
            components,
            created_at: now,
            updated_at: now,
            domain_events: Vec::new(),
        };

        cycle.record_event(CycleEvent::Created {
            cycle_id: id,
            created_at: now,
        });

        cycle
    }

    /// Reconstitutes a cycle from persisted data.
    ///
    /// This is used by repository implementations to reconstruct domain objects
    /// from database records. It bypasses domain event recording.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: CycleId,
        session_id: SessionId,
        parent_cycle_id: Option<CycleId>,
        branch_point: Option<ComponentType>,
        branch_metadata: BranchMetadata,
        status: CycleStatus,
        current_step: ComponentType,
        components: HashMap<ComponentType, ComponentVariant>,
        created_at: Timestamp,
        updated_at: Timestamp,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            session_id,
            parent_cycle_id,
            branch_point,
            branch_metadata,
            status,
            current_step,
            components,
            created_at,
            updated_at,
            domain_events: Vec::new(),
        })
    }

    // ───────────────────────────────────────────────────────────────
    // Accessors
    // ───────────────────────────────────────────────────────────────

    /// Returns the cycle ID.
    pub fn id(&self) -> CycleId {
        self.id
    }

    /// Returns the session ID this cycle belongs to.
    pub fn session_id(&self) -> SessionId {
        self.session_id
    }

    /// Returns the parent cycle ID if this is a branch.
    pub fn parent_cycle_id(&self) -> Option<CycleId> {
        self.parent_cycle_id
    }

    /// Returns the component type where branching occurred.
    pub fn branch_point(&self) -> Option<ComponentType> {
        self.branch_point
    }

    /// Returns the branch metadata (label, position hints).
    pub fn branch_metadata(&self) -> &BranchMetadata {
        &self.branch_metadata
    }

    /// Returns the cycle status.
    pub fn status(&self) -> CycleStatus {
        self.status
    }

    /// Returns the current step (active component).
    pub fn current_step(&self) -> ComponentType {
        self.current_step
    }

    /// Returns when this cycle was created.
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    /// Returns when this cycle was last updated.
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    /// Returns the status of a specific component.
    pub fn component_status(&self, ct: ComponentType) -> ComponentStatus {
        self.components
            .get(&ct)
            .map(|c| c.status())
            .unwrap_or(ComponentStatus::NotStarted)
    }

    /// Returns a reference to a component.
    pub fn component(&self, ct: ComponentType) -> Option<&ComponentVariant> {
        self.components.get(&ct)
    }

    /// Returns a mutable reference to a component.
    pub fn component_mut(&mut self, ct: ComponentType) -> Option<&mut ComponentVariant> {
        self.components.get_mut(&ct)
    }

    /// Returns true if this cycle is a branch.
    pub fn is_branch(&self) -> bool {
        self.parent_cycle_id.is_some()
    }

    /// Takes accumulated domain events, clearing the internal buffer.
    pub fn take_events(&mut self) -> Vec<CycleEvent> {
        std::mem::take(&mut self.domain_events)
    }

    // ───────────────────────────────────────────────────────────────
    // Component Status Transitions
    // ───────────────────────────────────────────────────────────────

    /// Validates that a component can be started.
    ///
    /// Checks:
    /// 1. Cycle is mutable (Active)
    /// 2. Component is not already started
    /// 3. Prerequisite component is started
    pub fn validate_can_start(&self, ct: ComponentType) -> Result<(), DomainError> {
        // 1. Check cycle is mutable
        if !self.status.is_mutable() {
            return Err(DomainError::new(
                ErrorCode::CycleArchived,
                "Cannot modify archived or completed cycle",
            ));
        }

        // 2. Check component not already started
        let current_status = self.component_status(ct);
        if current_status.is_started() {
            return Err(DomainError::new(
                ErrorCode::ComponentAlreadyStarted,
                format!("{:?} is already {:?}", ct, current_status),
            ));
        }

        // 3. Check prerequisite is started
        if let Some(prereq) = ComponentSequence::prerequisite(ct) {
            let prereq_status = self.component_status(prereq);
            if !prereq_status.is_started() {
                return Err(DomainError::new(
                    ErrorCode::PreviousComponentRequired,
                    format!("Cannot start {:?} before {:?} is started", ct, prereq),
                ));
            }
        }

        Ok(())
    }

    /// Starts work on a component.
    ///
    /// Transitions the component from NotStarted to InProgress.
    pub fn start_component(&mut self, ct: ComponentType) -> Result<(), DomainError> {
        self.validate_can_start(ct)?;

        let component = self
            .components
            .get_mut(&ct)
            .ok_or_else(|| DomainError::new(ErrorCode::ComponentNotFound, "Component not found"))?;

        component
            .start()
            .map_err(|e| DomainError::new(ErrorCode::InvalidStateTransition, e.to_string()))?;

        self.current_step = ct;
        self.updated_at = Timestamp::now();

        self.record_event(CycleEvent::ComponentStarted {
            cycle_id: self.id,
            component_type: ct,
        });

        Ok(())
    }

    /// Completes a component without validation.
    ///
    /// Use `validate_can_complete` separately for full validation with schema checking.
    pub fn complete_component(&mut self, ct: ComponentType) -> Result<(), DomainError> {
        // Check cycle is mutable
        if !self.status.is_mutable() {
            return Err(DomainError::new(
                ErrorCode::CycleArchived,
                "Cannot modify archived or completed cycle",
            ));
        }

        // Check component accepts output
        let current_status = self.component_status(ct);
        if !current_status.accepts_output() {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!("Cannot complete {:?} from {:?} state", ct, current_status),
            ));
        }

        let component = self
            .components
            .get_mut(&ct)
            .ok_or_else(|| DomainError::new(ErrorCode::ComponentNotFound, "Component not found"))?;

        component
            .complete()
            .map_err(|e| DomainError::new(ErrorCode::InvalidStateTransition, e.to_string()))?;

        self.updated_at = Timestamp::now();

        self.record_event(CycleEvent::ComponentCompleted {
            cycle_id: self.id,
            component_type: ct,
        });

        Ok(())
    }

    /// Updates the output of a component.
    ///
    /// The component must be in a state that accepts output (InProgress or NeedsRevision).
    pub fn update_component_output(
        &mut self,
        ct: ComponentType,
        output: serde_json::Value,
    ) -> Result<(), DomainError> {
        // Check cycle is mutable
        if !self.status.is_mutable() {
            return Err(DomainError::new(
                ErrorCode::CycleArchived,
                "Cannot modify archived or completed cycle",
            ));
        }

        // Check component accepts output
        let current_status = self.component_status(ct);
        if !current_status.accepts_output() {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!(
                    "Cannot update output for {:?} in {:?} state",
                    ct, current_status
                ),
            ));
        }

        let component = self
            .components
            .get_mut(&ct)
            .ok_or_else(|| DomainError::new(ErrorCode::ComponentNotFound, "Component not found"))?;

        component
            .set_output_from_value(output)
            .map_err(|e| DomainError::new(ErrorCode::InvalidFormat, e.to_string()))?;

        self.updated_at = Timestamp::now();

        self.record_event(CycleEvent::ComponentOutputUpdated {
            cycle_id: self.id,
            component_type: ct,
        });

        Ok(())
    }

    /// Marks a component for revision.
    pub fn mark_component_for_revision(
        &mut self,
        ct: ComponentType,
        reason: String,
    ) -> Result<(), DomainError> {
        // Check cycle is mutable
        if !self.status.is_mutable() {
            return Err(DomainError::new(
                ErrorCode::CycleArchived,
                "Cannot modify archived or completed cycle",
            ));
        }

        let component = self
            .components
            .get_mut(&ct)
            .ok_or_else(|| DomainError::new(ErrorCode::ComponentNotFound, "Component not found"))?;

        component
            .mark_for_revision(reason.clone())
            .map_err(|e| DomainError::new(ErrorCode::InvalidStateTransition, e.to_string()))?;

        self.current_step = ct;
        self.updated_at = Timestamp::now();

        self.record_event(CycleEvent::ComponentMarkedForRevision {
            cycle_id: self.id,
            component_type: ct,
            reason,
        });

        Ok(())
    }

    // ───────────────────────────────────────────────────────────────
    // Completion Validation (Component-Specific Rules)
    // ───────────────────────────────────────────────────────────────

    /// Validates component-specific completion rules.
    ///
    /// This checks business rules beyond schema validation:
    /// - Alternatives: >= 2 alternatives, valid status_quo_id
    /// - Objectives: >= 1 fundamental objective
    /// - Consequences: all cells filled
    /// - DecisionQuality: exactly 7 elements scored
    pub fn validate_component_completion_rules(
        &self,
        ct: ComponentType,
        output: &serde_json::Value,
    ) -> Result<(), DomainError> {
        match ct {
            ComponentType::Alternatives => {
                // Must have at least 2 alternatives including status quo
                let alts = output
                    .get("alternatives")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        DomainError::validation("alternatives", "Missing alternatives array")
                    })?;

                if alts.len() < 2 {
                    return Err(DomainError::validation(
                        "alternatives",
                        "Must have at least 2 alternatives",
                    ));
                }

                // Verify status_quo_id exists in alternatives
                let status_quo_id = output
                    .get("status_quo_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        DomainError::validation("status_quo_id", "Missing status quo designation")
                    })?;

                let has_status_quo = alts
                    .iter()
                    .any(|a| a.get("id").and_then(|v| v.as_str()) == Some(status_quo_id));

                if !has_status_quo {
                    return Err(DomainError::validation(
                        "status_quo_id",
                        "Status quo ID not found in alternatives",
                    ));
                }
            }

            ComponentType::Objectives => {
                // Must have at least 1 fundamental objective
                let fundamentals = output
                    .get("fundamental_objectives")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        DomainError::validation(
                            "fundamental_objectives",
                            "Missing fundamental objectives",
                        )
                    })?;

                if fundamentals.is_empty() {
                    return Err(DomainError::validation(
                        "fundamental_objectives",
                        "Must have at least 1 fundamental objective",
                    ));
                }
            }

            ComponentType::Consequences => {
                // All cells must be filled
                let table = output.get("table").ok_or_else(|| {
                    DomainError::validation("table", "Missing consequence table")
                })?;

                let alt_ids: Vec<_> = table
                    .get("alternative_ids")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                    .unwrap_or_default();

                let obj_ids: Vec<_> = table
                    .get("objective_ids")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                    .unwrap_or_default();

                let cells = table
                    .get("cells")
                    .and_then(|v| v.as_object())
                    .ok_or_else(|| DomainError::validation("cells", "Missing cells object"))?;

                // Check all combinations exist
                for alt_id in &alt_ids {
                    for obj_id in &obj_ids {
                        let key = format!("{}:{}", alt_id, obj_id);
                        if !cells.contains_key(&key) {
                            return Err(DomainError::validation(
                                "cells",
                                format!(
                                    "Missing cell for alternative {} and objective {}",
                                    alt_id, obj_id
                                ),
                            ));
                        }
                    }
                }
            }

            ComponentType::DecisionQuality => {
                // All 7 elements must be scored
                let elements = output
                    .get("elements")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        DomainError::validation("elements", "Missing DQ elements")
                    })?;

                if elements.len() != 7 {
                    return Err(DomainError::validation(
                        "elements",
                        "Must have exactly 7 DQ elements",
                    ));
                }
            }

            _ => {} // Other components have no special rules beyond schema
        }

        Ok(())
    }

    // ───────────────────────────────────────────────────────────────
    // Branching
    // ───────────────────────────────────────────────────────────────

    /// Validates that branching is possible at the specified component.
    fn validate_can_branch_at(&self, branch_point: ComponentType) -> Result<(), DomainError> {
        // Must be active cycle
        if !self.status.is_mutable() {
            return Err(DomainError::new(
                ErrorCode::CycleArchived,
                "Cannot branch from archived cycle",
            ));
        }

        // Branch point must be started
        let status = self.component_status(branch_point);
        if !status.is_started() {
            return Err(DomainError::new(
                ErrorCode::CannotBranch,
                format!(
                    "Cannot branch at {:?} - component not started",
                    branch_point
                ),
            ));
        }

        Ok(())
    }

    /// Creates a branch from this cycle at the specified component.
    ///
    /// Components before the branch point are copied with Complete status.
    /// The branch point component is copied with NeedsRevision status.
    /// Components after the branch point start fresh.
    ///
    /// Optionally accepts a branch label for visualization purposes.
    pub fn branch_at(
        &self,
        branch_point: ComponentType,
        branch_label: Option<String>,
    ) -> Result<Cycle, DomainError> {
        self.validate_can_branch_at(branch_point)?;

        let id = CycleId::new();
        let now = Timestamp::now();

        // Determine which components to copy
        let mut new_components = HashMap::new();

        for ct in ComponentSequence::all() {
            if ComponentSequence::is_before(*ct, branch_point) {
                // Components before branch point: copy as-is (already Complete)
                if let Some(parent_component) = self.components.get(ct) {
                    new_components.insert(*ct, parent_component.clone());
                }
            } else if *ct == branch_point {
                // Branch point: copy but mark NeedsRevision
                if let Some(parent_component) = self.components.get(ct) {
                    let mut branch_component = parent_component.clone();
                    // Mark for revision with a standard reason
                    let _ = branch_component.mark_for_revision("Branched for exploration".to_string());
                    new_components.insert(*ct, branch_component);
                }
            } else {
                // After branch point: fresh component
                new_components.insert(*ct, ComponentVariant::new(*ct));
            }
        }

        let mut branch = Cycle {
            id,
            session_id: self.session_id,
            parent_cycle_id: Some(self.id),
            branch_point: Some(branch_point),
            branch_metadata: BranchMetadata::branched(branch_label),
            status: CycleStatus::Active,
            current_step: branch_point,
            components: new_components,
            created_at: now,
            updated_at: now,
            domain_events: Vec::new(),
        };

        branch.record_event(CycleEvent::Branched {
            cycle_id: id,
            parent_cycle_id: self.id,
            branch_point,
            created_at: now,
        });

        Ok(branch)
    }

    // ───────────────────────────────────────────────────────────────
    // Navigation
    // ───────────────────────────────────────────────────────────────

    /// Navigate to a different component (changes current_step).
    ///
    /// Navigation is allowed to:
    /// - Any started component (InProgress, Complete, NeedsRevision)
    /// - The next not-started component if its prerequisite is started
    pub fn navigate_to(&mut self, target: ComponentType) -> Result<(), DomainError> {
        // 1. Check cycle is mutable
        if !self.status.is_mutable() {
            return Err(DomainError::new(
                ErrorCode::CycleArchived,
                "Cannot navigate in archived cycle",
            ));
        }

        // 2. Check target is accessible
        let target_status = self.component_status(target);
        let can_navigate = match target_status {
            // Can navigate to started components
            ComponentStatus::InProgress
            | ComponentStatus::Complete
            | ComponentStatus::NeedsRevision => true,

            // Can navigate to next not-started component if prerequisite started
            ComponentStatus::NotStarted => {
                ComponentSequence::prerequisite(target)
                    .map(|prereq| self.component_status(prereq).is_started())
                    .unwrap_or(true) // IssueRaising has no prerequisite
            }
        };

        if !can_navigate {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!(
                    "Cannot navigate to {:?} - prerequisite not started",
                    target
                ),
            ));
        }

        // 3. Update current step
        self.current_step = target;
        self.updated_at = Timestamp::now();

        self.record_event(CycleEvent::NavigatedTo {
            cycle_id: self.id,
            component_type: target,
        });

        Ok(())
    }

    // ───────────────────────────────────────────────────────────────
    // Cycle Completion
    // ───────────────────────────────────────────────────────────────

    /// Completes the cycle.
    ///
    /// Requires DecisionQuality to be complete (user has assessed decision quality).
    pub fn complete(&mut self) -> Result<(), DomainError> {
        // 1. Check can transition
        if !self.status.can_transition_to(&CycleStatus::Completed) {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                "Cycle cannot be completed in current state",
            ));
        }

        // 2. Check minimum completion requirements
        let dq_status = self.component_status(ComponentType::DecisionQuality);
        if !matches!(dq_status, ComponentStatus::Complete) {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                "DecisionQuality must be complete before completing cycle",
            ));
        }

        // 3. Complete
        self.status = CycleStatus::Completed;
        self.updated_at = Timestamp::now();

        self.record_event(CycleEvent::Completed { cycle_id: self.id });

        Ok(())
    }

    /// Archives the cycle.
    pub fn archive(&mut self) -> Result<(), DomainError> {
        if !self.status.can_transition_to(&CycleStatus::Archived) {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                "Cycle cannot be archived in current state",
            ));
        }

        self.status = CycleStatus::Archived;
        self.updated_at = Timestamp::now();

        self.record_event(CycleEvent::Archived { cycle_id: self.id });

        Ok(())
    }

    // ───────────────────────────────────────────────────────────────
    // Internal Helpers
    // ───────────────────────────────────────────────────────────────

    fn record_event(&mut self, event: CycleEvent) {
        self.domain_events.push(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cycle() -> Cycle {
        Cycle::new(SessionId::new())
    }

    // ───────────────────────────────────────────────────────────────
    // Creation Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn new_cycle_is_active() {
        let cycle = create_test_cycle();
        assert_eq!(cycle.status(), CycleStatus::Active);
    }

    #[test]
    fn new_cycle_has_all_components_not_started() {
        let cycle = create_test_cycle();
        for ct in ComponentSequence::all() {
            assert_eq!(
                cycle.component_status(*ct),
                ComponentStatus::NotStarted,
                "Component {:?} should be NotStarted",
                ct
            );
        }
    }

    #[test]
    fn new_cycle_current_step_is_issue_raising() {
        let cycle = create_test_cycle();
        assert_eq!(cycle.current_step(), ComponentType::IssueRaising);
    }

    #[test]
    fn new_cycle_is_not_a_branch() {
        let cycle = create_test_cycle();
        assert!(!cycle.is_branch());
        assert!(cycle.parent_cycle_id().is_none());
        assert!(cycle.branch_point().is_none());
    }

    #[test]
    fn new_cycle_records_created_event() {
        let mut cycle = create_test_cycle();
        let events = cycle.take_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], CycleEvent::Created { .. }));
    }

    // ───────────────────────────────────────────────────────────────
    // Start Component Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn can_start_issue_raising() {
        let mut cycle = create_test_cycle();
        assert!(cycle.start_component(ComponentType::IssueRaising).is_ok());
        assert_eq!(
            cycle.component_status(ComponentType::IssueRaising),
            ComponentStatus::InProgress
        );
    }

    #[test]
    fn starting_component_updates_current_step() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        assert_eq!(cycle.current_step(), ComponentType::IssueRaising);
    }

    #[test]
    fn cannot_start_problem_frame_before_issue_raising() {
        let cycle = create_test_cycle();
        let result = cycle.validate_can_start(ComponentType::ProblemFrame);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            ErrorCode::PreviousComponentRequired
        );
    }

    #[test]
    fn can_start_problem_frame_after_issue_raising_started() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        assert!(cycle.start_component(ComponentType::ProblemFrame).is_ok());
    }

    #[test]
    fn cannot_start_already_started_component() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        let result = cycle.start_component(ComponentType::IssueRaising);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::ComponentAlreadyStarted);
    }

    #[test]
    fn starting_component_records_event() {
        let mut cycle = create_test_cycle();
        cycle.take_events(); // Clear creation event
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        let events = cycle.take_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            CycleEvent::ComponentStarted {
                component_type: ComponentType::IssueRaising,
                ..
            }
        ));
    }

    // ───────────────────────────────────────────────────────────────
    // Complete Component Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn can_complete_in_progress_component() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        assert!(cycle
            .complete_component(ComponentType::IssueRaising)
            .is_ok());
        assert_eq!(
            cycle.component_status(ComponentType::IssueRaising),
            ComponentStatus::Complete
        );
    }

    #[test]
    fn cannot_complete_not_started_component() {
        let mut cycle = create_test_cycle();
        let result = cycle.complete_component(ComponentType::IssueRaising);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::InvalidStateTransition);
    }

    #[test]
    fn completing_component_records_event() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle.take_events();
        cycle
            .complete_component(ComponentType::IssueRaising)
            .unwrap();
        let events = cycle.take_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            CycleEvent::ComponentCompleted {
                component_type: ComponentType::IssueRaising,
                ..
            }
        ));
    }

    // ───────────────────────────────────────────────────────────────
    // Mark for Revision Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn can_mark_complete_component_for_revision() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle
            .complete_component(ComponentType::IssueRaising)
            .unwrap();
        assert!(cycle
            .mark_component_for_revision(
                ComponentType::IssueRaising,
                "Needs more detail".to_string()
            )
            .is_ok());
        assert_eq!(
            cycle.component_status(ComponentType::IssueRaising),
            ComponentStatus::NeedsRevision
        );
    }

    #[test]
    fn marking_for_revision_updates_current_step() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle
            .complete_component(ComponentType::IssueRaising)
            .unwrap();
        cycle.start_component(ComponentType::ProblemFrame).unwrap();

        cycle
            .mark_component_for_revision(
                ComponentType::IssueRaising,
                "Needs review".to_string(),
            )
            .unwrap();

        assert_eq!(cycle.current_step(), ComponentType::IssueRaising);
    }

    // ───────────────────────────────────────────────────────────────
    // Component Output Update Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn can_update_output_for_in_progress_component() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle.take_events();

        let output = serde_json::json!({
            "potential_decisions": ["Option A"],
            "objectives": [],
            "uncertainties": [],
            "considerations": [],
            "user_confirmed": false
        });

        assert!(cycle
            .update_component_output(ComponentType::IssueRaising, output)
            .is_ok());
    }

    #[test]
    fn update_output_records_event() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle.take_events();

        let output = serde_json::json!({
            "potential_decisions": ["Option A"],
            "objectives": [],
            "uncertainties": [],
            "considerations": [],
            "user_confirmed": false
        });

        cycle
            .update_component_output(ComponentType::IssueRaising, output)
            .unwrap();

        let events = cycle.take_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            CycleEvent::ComponentOutputUpdated { .. }
        ));
    }

    #[test]
    fn cannot_update_output_for_not_started_component() {
        let mut cycle = create_test_cycle();

        let output = serde_json::json!({
            "potential_decisions": [],
            "objectives": [],
            "uncertainties": [],
            "considerations": [],
            "user_confirmed": false
        });

        let result = cycle.update_component_output(ComponentType::IssueRaising, output);
        assert!(result.is_err());
    }

    #[test]
    fn cannot_update_output_for_archived_cycle() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle.archive().unwrap();
        cycle.take_events();

        let output = serde_json::json!({
            "potential_decisions": [],
            "objectives": [],
            "uncertainties": []
        });

        let result = cycle.update_component_output(ComponentType::IssueRaising, output);
        assert!(result.is_err());
    }

    // ───────────────────────────────────────────────────────────────
    // Navigation Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn can_navigate_to_started_component() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle
            .complete_component(ComponentType::IssueRaising)
            .unwrap();
        cycle.start_component(ComponentType::ProblemFrame).unwrap();

        assert!(cycle.navigate_to(ComponentType::IssueRaising).is_ok());
        assert_eq!(cycle.current_step(), ComponentType::IssueRaising);
    }

    #[test]
    fn can_navigate_to_next_not_started_component_if_prereq_started() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();

        // ProblemFrame is not started, but IssueRaising (its prereq) is started
        assert!(cycle.navigate_to(ComponentType::ProblemFrame).is_ok());
    }

    #[test]
    fn cannot_navigate_to_not_started_component_without_prereq() {
        let cycle = create_test_cycle();
        // Alternatives requires Objectives to be started first
        let result = cycle.validate_can_start(ComponentType::Alternatives);
        assert!(result.is_err());
    }

    #[test]
    fn navigating_records_event() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle.take_events();
        cycle.navigate_to(ComponentType::ProblemFrame).unwrap();
        let events = cycle.take_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            CycleEvent::NavigatedTo {
                component_type: ComponentType::ProblemFrame,
                ..
            }
        ));
    }

    // ───────────────────────────────────────────────────────────────
    // Branching Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn can_branch_at_started_component() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle
            .complete_component(ComponentType::IssueRaising)
            .unwrap();
        cycle.start_component(ComponentType::ProblemFrame).unwrap();
        cycle
            .complete_component(ComponentType::ProblemFrame)
            .unwrap();
        cycle.start_component(ComponentType::Objectives).unwrap();

        let branch = cycle.branch_at(ComponentType::Objectives, None).unwrap();

        assert!(branch.is_branch());
        assert_eq!(branch.parent_cycle_id(), Some(cycle.id()));
        assert_eq!(branch.branch_point(), Some(ComponentType::Objectives));
    }

    #[test]
    fn branch_inherits_components_before_branch_point() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle
            .complete_component(ComponentType::IssueRaising)
            .unwrap();
        cycle.start_component(ComponentType::ProblemFrame).unwrap();
        cycle
            .complete_component(ComponentType::ProblemFrame)
            .unwrap();
        cycle.start_component(ComponentType::Objectives).unwrap();

        let branch = cycle.branch_at(ComponentType::Objectives, None).unwrap();

        // Components before branch point should be Complete
        assert_eq!(
            branch.component_status(ComponentType::IssueRaising),
            ComponentStatus::Complete
        );
        assert_eq!(
            branch.component_status(ComponentType::ProblemFrame),
            ComponentStatus::Complete
        );
    }

    #[test]
    fn branch_point_component_marked_needs_revision() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle
            .complete_component(ComponentType::IssueRaising)
            .unwrap();
        cycle.start_component(ComponentType::ProblemFrame).unwrap();

        let branch = cycle.branch_at(ComponentType::ProblemFrame, None).unwrap();

        // Branch point should be NeedsRevision
        assert_eq!(
            branch.component_status(ComponentType::ProblemFrame),
            ComponentStatus::NeedsRevision
        );
    }

    #[test]
    fn branch_components_after_branch_point_are_fresh() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle
            .complete_component(ComponentType::IssueRaising)
            .unwrap();
        cycle.start_component(ComponentType::ProblemFrame).unwrap();

        let branch = cycle.branch_at(ComponentType::ProblemFrame, None).unwrap();

        // Components after branch point should be NotStarted
        assert_eq!(
            branch.component_status(ComponentType::Objectives),
            ComponentStatus::NotStarted
        );
        assert_eq!(
            branch.component_status(ComponentType::Alternatives),
            ComponentStatus::NotStarted
        );
    }

    #[test]
    fn branch_current_step_is_branch_point() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();
        cycle
            .complete_component(ComponentType::IssueRaising)
            .unwrap();
        cycle.start_component(ComponentType::ProblemFrame).unwrap();

        let branch = cycle.branch_at(ComponentType::ProblemFrame, None).unwrap();

        assert_eq!(branch.current_step(), ComponentType::ProblemFrame);
    }

    #[test]
    fn cannot_branch_at_not_started_component() {
        let cycle = create_test_cycle();
        let result = cycle.branch_at(ComponentType::ProblemFrame, None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::CannotBranch);
    }

    #[test]
    fn branch_records_event() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();

        let mut branch = cycle.branch_at(ComponentType::IssueRaising, None).unwrap();
        let events = branch.take_events();

        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], CycleEvent::Branched { .. }));
    }

    // ───────────────────────────────────────────────────────────────
    // Cycle Completion Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn cannot_complete_cycle_without_decision_quality() {
        let mut cycle = create_test_cycle();
        cycle.start_component(ComponentType::IssueRaising).unwrap();

        let result = cycle.complete();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::InvalidStateTransition);
    }

    #[test]
    fn can_complete_cycle_with_decision_quality_complete() {
        let mut cycle = create_test_cycle();

        // Progress through all required components
        for ct in ComponentSequence::all() {
            if *ct == ComponentType::NotesNextSteps {
                continue; // Optional component
            }
            cycle.start_component(*ct).unwrap();
            cycle.complete_component(*ct).unwrap();
        }

        assert!(cycle.complete().is_ok());
        assert_eq!(cycle.status(), CycleStatus::Completed);
    }

    #[test]
    fn completing_cycle_records_event() {
        let mut cycle = create_test_cycle();

        for ct in ComponentSequence::all() {
            if *ct == ComponentType::NotesNextSteps {
                continue;
            }
            cycle.start_component(*ct).unwrap();
            cycle.complete_component(*ct).unwrap();
        }

        cycle.take_events();
        cycle.complete().unwrap();
        let events = cycle.take_events();

        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], CycleEvent::Completed { .. }));
    }

    // ───────────────────────────────────────────────────────────────
    // Archive Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn can_archive_active_cycle() {
        let mut cycle = create_test_cycle();
        assert!(cycle.archive().is_ok());
        assert_eq!(cycle.status(), CycleStatus::Archived);
    }

    #[test]
    fn can_archive_completed_cycle() {
        let mut cycle = create_test_cycle();

        for ct in ComponentSequence::all() {
            if *ct == ComponentType::NotesNextSteps {
                continue;
            }
            cycle.start_component(*ct).unwrap();
            cycle.complete_component(*ct).unwrap();
        }

        cycle.complete().unwrap();
        assert!(cycle.archive().is_ok());
        assert_eq!(cycle.status(), CycleStatus::Archived);
    }

    #[test]
    fn cannot_modify_archived_cycle() {
        let mut cycle = create_test_cycle();
        cycle.archive().unwrap();

        let result = cycle.start_component(ComponentType::IssueRaising);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::CycleArchived);
    }

    // ───────────────────────────────────────────────────────────────
    // Completion Validation Rule Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn alternatives_validation_requires_at_least_two() {
        let cycle = create_test_cycle();
        let output = serde_json::json!({
            "alternatives": [
                {"id": "alt-1", "name": "Option A"}
            ],
            "status_quo_id": "alt-1"
        });

        let result = cycle.validate_component_completion_rules(ComponentType::Alternatives, &output);
        assert!(result.is_err());
    }

    #[test]
    fn alternatives_validation_requires_valid_status_quo() {
        let cycle = create_test_cycle();
        let output = serde_json::json!({
            "alternatives": [
                {"id": "alt-1", "name": "Option A"},
                {"id": "alt-2", "name": "Option B"}
            ],
            "status_quo_id": "nonexistent"
        });

        let result = cycle.validate_component_completion_rules(ComponentType::Alternatives, &output);
        assert!(result.is_err());
    }

    #[test]
    fn alternatives_validation_passes_with_valid_data() {
        let cycle = create_test_cycle();
        let output = serde_json::json!({
            "alternatives": [
                {"id": "alt-1", "name": "Status Quo"},
                {"id": "alt-2", "name": "Option B"}
            ],
            "status_quo_id": "alt-1"
        });

        let result = cycle.validate_component_completion_rules(ComponentType::Alternatives, &output);
        assert!(result.is_ok());
    }

    #[test]
    fn objectives_validation_requires_at_least_one_fundamental() {
        let cycle = create_test_cycle();
        let output = serde_json::json!({
            "fundamental_objectives": []
        });

        let result = cycle.validate_component_completion_rules(ComponentType::Objectives, &output);
        assert!(result.is_err());
    }

    #[test]
    fn decision_quality_validation_requires_seven_elements() {
        let cycle = create_test_cycle();
        let output = serde_json::json!({
            "elements": [
                {"name": "Frame", "score": 80},
                {"name": "Values", "score": 70}
            ]
        });

        let result = cycle.validate_component_completion_rules(ComponentType::DecisionQuality, &output);
        assert!(result.is_err());
    }
}
