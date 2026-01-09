# Component Status Validation & Cycle Branching

**Module:** cycle
**Type:** Feature Specification
**Priority:** P1 (Core workflow)
**Last Updated:** 2026-01-08

> Detailed rules for component status transitions and data inheritance during cycle branching.

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Required |
| Authorization Model | Inherited from cycle ownership (user owns parent session) |
| Sensitive Data | Component outputs (Confidential); Status transitions (Internal) |
| Rate Limiting | Inherited from cycle operations (200 requests/minute per user) |
| Audit Logging | Status transitions, validation failures, branching operations |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| ComponentStatus | Internal | Safe to log state transitions |
| ComponentType | Internal | Safe to log, enum values only |
| Component output (JSON) | Confidential | Contains user decision data, encrypt at rest |
| Validation errors | Internal | Log for debugging, do not expose internal details to client |
| Branch point | Internal | Safe to log |
| Inherited component data | Confidential | Cloned from parent, maintains original classification |

### Security Events to Log

- Status transition success: DEBUG level with component_type, from_status, to_status
- Status transition failure (WARN level): Invalid state transition attempts with reason
- Validation failure (WARN level): Schema or business rule validation failures
- Branch creation (INFO level): Parent cycle, branch point, inherited component count
- Prerequisite violation (WARN level): Attempts to start components out of order

### Authorization Rules

1. **Status Transitions**: User must own the session containing the cycle
2. **Component Output Updates**: User must own the session; component must accept output
3. **Branching**: User must own the session; source component must be started
4. **Navigation**: User must own the session; target must be accessible per rules

### Input Validation Security

1. **Schema Validation**: All component outputs validated against JSON Schema before persistence
2. **Business Rule Validation**: Component-specific rules (e.g., min alternatives, required fields)
3. **State Machine Enforcement**: Only valid status transitions allowed per `can_transition_to()`
4. **Ordering Enforcement**: Prerequisite components must be started before dependent components

---

## Overview

This document specifies the complete rules for:
1. Component status transitions within a cycle
2. Data inheritance when branching cycles
3. Validation rules before status changes
4. Cross-component dependencies

---

## Component Status Machine

```
                    ┌─────────────────┐
                    │  NOT_STARTED    │
                    │                 │
                    │ • Default state │
                    │ • No output     │
                    └────────┬────────┘
                             │
                             │ start_component()
                             │ Precondition: previous component started
                             ▼
                    ┌─────────────────┐
                    │  IN_PROGRESS    │
                    │                 │
                    │ • Active work   │
                    │ • May have      │
                    │   partial data  │
                    └───┬─────────┬───┘
                        │         │
   complete_component() │         │ mark_needs_revision()
   Precondition:        │         │
   valid output         │         │
                        ▼         ▼
          ┌─────────────────┐  ┌─────────────────┐
          │    COMPLETE     │  │ NEEDS_REVISION  │
          │                 │  │                 │
          │ • Final output  │  │ • Output exists │
          │ • Read-only     │  │ • Flagged for   │
          │                 │  │   review        │
          └────────┬────────┘  └────────┬────────┘
                   │                    │
                   │                    │ resume_work()
                   │                    │
                   │                    ▼
                   │           ┌─────────────────┐
                   │           │  IN_PROGRESS    │
                   └───────────┤   (returns)     │
                               └─────────────────┘
```

### Status Definitions

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentStatus {
    /// Component not yet started
    NotStarted,

    /// Actively being worked on
    InProgress,

    /// Work complete, output finalized
    Complete,

    /// Completed but flagged for review (e.g., after branching)
    NeedsRevision,
}

impl ComponentStatus {
    /// Can this component accept new output?
    pub fn accepts_output(&self) -> bool {
        matches!(self, Self::InProgress | Self::NeedsRevision)
    }

    /// Is this component considered "started" for ordering purposes?
    pub fn is_started(&self) -> bool {
        !matches!(self, Self::NotStarted)
    }

    /// Is this component locked for modification?
    pub fn is_locked(&self) -> bool {
        matches!(self, Self::Complete)
    }

    /// Can transition to target status?
    pub fn can_transition_to(&self, target: &Self) -> bool {
        match (self, target) {
            // From NotStarted
            (Self::NotStarted, Self::InProgress) => true,

            // From InProgress
            (Self::InProgress, Self::Complete) => true,
            (Self::InProgress, Self::NeedsRevision) => true,

            // From Complete (generally locked)
            (Self::Complete, Self::NeedsRevision) => true, // Via revision request

            // From NeedsRevision
            (Self::NeedsRevision, Self::InProgress) => true,
            (Self::NeedsRevision, Self::Complete) => true,

            // Same status is always allowed (no-op)
            (a, b) if a == b => true,

            _ => false,
        }
    }
}
```

---

## Component Ordering Rules

### PrOACT Component Sequence

```
1. IssueRaising     (entry point)
2. ProblemFrame     (requires: IssueRaising started)
3. Objectives       (requires: ProblemFrame started)
4. Alternatives     (requires: Objectives started)
5. Consequences     (requires: Alternatives started)
6. Tradeoffs        (requires: Consequences started)
7. Recommendation   (requires: Tradeoffs started)
8. DecisionQuality  (requires: Recommendation started)
9. NotesNextSteps   (optional, can start anytime after IssueRaising)
```

### Ordering Implementation

```rust
impl ComponentType {
    /// Returns the component that must be started before this one
    pub fn prerequisite(&self) -> Option<ComponentType> {
        match self {
            Self::IssueRaising => None,
            Self::ProblemFrame => Some(Self::IssueRaising),
            Self::Objectives => Some(Self::ProblemFrame),
            Self::Alternatives => Some(Self::Objectives),
            Self::Consequences => Some(Self::Alternatives),
            Self::Tradeoffs => Some(Self::Consequences),
            Self::Recommendation => Some(Self::Tradeoffs),
            Self::DecisionQuality => Some(Self::Recommendation),
            Self::NotesNextSteps => Some(Self::IssueRaising), // Can start after any
        }
    }

    /// Returns the next component in sequence (excluding NotesNextSteps)
    pub fn next(&self) -> Option<ComponentType> {
        match self {
            Self::IssueRaising => Some(Self::ProblemFrame),
            Self::ProblemFrame => Some(Self::Objectives),
            Self::Objectives => Some(Self::Alternatives),
            Self::Alternatives => Some(Self::Consequences),
            Self::Consequences => Some(Self::Tradeoffs),
            Self::Tradeoffs => Some(Self::Recommendation),
            Self::Recommendation => Some(Self::DecisionQuality),
            Self::DecisionQuality => None,
            Self::NotesNextSteps => None,
        }
    }

    /// Returns the previous component in sequence
    pub fn previous(&self) -> Option<ComponentType> {
        match self {
            Self::IssueRaising => None,
            Self::ProblemFrame => Some(Self::IssueRaising),
            Self::Objectives => Some(Self::ProblemFrame),
            Self::Alternatives => Some(Self::Objectives),
            Self::Consequences => Some(Self::Alternatives),
            Self::Tradeoffs => Some(Self::Consequences),
            Self::Recommendation => Some(Self::Tradeoffs),
            Self::DecisionQuality => Some(Self::Recommendation),
            Self::NotesNextSteps => None, // No specific previous
        }
    }

    /// Check if this component comes before another
    pub fn is_before(&self, other: &ComponentType) -> bool {
        self.ordinal() < other.ordinal()
    }

    fn ordinal(&self) -> u8 {
        match self {
            Self::IssueRaising => 0,
            Self::ProblemFrame => 1,
            Self::Objectives => 2,
            Self::Alternatives => 3,
            Self::Consequences => 4,
            Self::Tradeoffs => 5,
            Self::Recommendation => 6,
            Self::DecisionQuality => 7,
            Self::NotesNextSteps => 8,
        }
    }
}
```

---

## Pre-Transition Validation

### Start Component Validation

```rust
impl Cycle {
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
                ErrorCode::ComponentAlreadyComplete,
                format!("{:?} is already {:?}", ct, current_status),
            ));
        }

        // 3. Check prerequisite is started
        if let Some(prereq) = ct.prerequisite() {
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
}
```

### Complete Component Validation

```rust
impl Cycle {
    pub fn validate_can_complete(
        &self,
        ct: ComponentType,
        output: &serde_json::Value,
        validator: &dyn ComponentSchemaValidator,
    ) -> Result<(), DomainError> {
        // 1. Check cycle is mutable
        if !self.status.is_mutable() {
            return Err(DomainError::new(
                ErrorCode::CycleArchived,
                "Cannot modify archived or completed cycle",
            ));
        }

        // 2. Check component is in progress
        let current_status = self.component_status(ct);
        if !current_status.accepts_output() {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!("Cannot complete {:?} from {:?} state", ct, current_status),
            ));
        }

        // 3. Validate output against schema
        validator.validate(ct, output)
            .map_err(|e| DomainError::new(
                ErrorCode::InvalidComponentOutput,
                e.to_string(),
            ))?;

        // 4. Check component-specific completion rules
        self.validate_component_completion_rules(ct, output)?;

        Ok(())
    }

    fn validate_component_completion_rules(
        &self,
        ct: ComponentType,
        output: &serde_json::Value,
    ) -> Result<(), DomainError> {
        match ct {
            ComponentType::Alternatives => {
                // Must have at least 2 alternatives including status quo
                let alts = output.get("alternatives")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| DomainError::validation(
                        "alternatives",
                        "Missing alternatives array",
                    ))?;

                if alts.len() < 2 {
                    return Err(DomainError::validation(
                        "alternatives",
                        "Must have at least 2 alternatives",
                    ));
                }

                // Verify status_quo_id exists in alternatives
                let status_quo_id = output.get("status_quo_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError::validation(
                        "status_quo_id",
                        "Missing status quo designation",
                    ))?;

                let has_status_quo = alts.iter().any(|a|
                    a.get("id").and_then(|v| v.as_str()) == Some(status_quo_id)
                );

                if !has_status_quo {
                    return Err(DomainError::validation(
                        "status_quo_id",
                        "Status quo ID not found in alternatives",
                    ));
                }
            }

            ComponentType::Objectives => {
                // Must have at least 1 fundamental objective
                let fundamentals = output.get("fundamental_objectives")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| DomainError::validation(
                        "fundamental_objectives",
                        "Missing fundamental objectives",
                    ))?;

                if fundamentals.is_empty() {
                    return Err(DomainError::validation(
                        "fundamental_objectives",
                        "Must have at least 1 fundamental objective",
                    ));
                }
            }

            ComponentType::Consequences => {
                // All cells must be filled
                let table = output.get("table")
                    .ok_or_else(|| DomainError::validation(
                        "table",
                        "Missing consequence table",
                    ))?;

                let alt_ids: Vec<_> = table.get("alternative_ids")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                    .unwrap_or_default();

                let obj_ids: Vec<_> = table.get("objective_ids")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                    .unwrap_or_default();

                let cells = table.get("cells")
                    .and_then(|v| v.as_object())
                    .ok_or_else(|| DomainError::validation(
                        "cells",
                        "Missing cells object",
                    ))?;

                // Check all combinations exist
                for alt_id in &alt_ids {
                    for obj_id in &obj_ids {
                        let key = format!("{}:{}", alt_id, obj_id);
                        if !cells.contains_key(&key) {
                            return Err(DomainError::validation(
                                "cells",
                                format!("Missing cell for alternative {} and objective {}", alt_id, obj_id),
                            ));
                        }
                    }
                }
            }

            ComponentType::DecisionQuality => {
                // All 7 elements must be scored
                let elements = output.get("elements")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| DomainError::validation(
                        "elements",
                        "Missing DQ elements",
                    ))?;

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
}
```

---

## Cycle Branching Inheritance

### Branching Rules

When creating a branch from a cycle:

```
Parent Cycle at ComponentType::Alternatives
┌────────────────────────────────────────────────────────────┐
│ IssueRaising    [COMPLETE]  → Copied (cloned output)       │
│ ProblemFrame    [COMPLETE]  → Copied (cloned output)       │
│ Objectives      [COMPLETE]  → Copied (cloned output)       │
│ Alternatives    [COMPLETE]  → Copied (branch point)        │
│ Consequences    [IN_PROGRESS] → NOT copied (fresh start)   │
│ Tradeoffs       [NOT_STARTED] → NOT copied (fresh)         │
│ Recommendation  [NOT_STARTED] → NOT copied (fresh)         │
│ DecisionQuality [NOT_STARTED] → NOT copied (fresh)         │
│ NotesNextSteps  [NOT_STARTED] → NOT copied (fresh)         │
└────────────────────────────────────────────────────────────┘
                              │
                              │ branch_at(ComponentType::Alternatives)
                              ▼
Child (Branch) Cycle
┌────────────────────────────────────────────────────────────┐
│ IssueRaising    [COMPLETE]  ← Inherited from parent        │
│ ProblemFrame    [COMPLETE]  ← Inherited from parent        │
│ Objectives      [COMPLETE]  ← Inherited from parent        │
│ Alternatives    [NEEDS_REVISION] ← Branch point, editable  │
│ Consequences    [NOT_STARTED] ← Fresh start               │
│ Tradeoffs       [NOT_STARTED] ← Fresh start               │
│ Recommendation  [NOT_STARTED] ← Fresh start               │
│ DecisionQuality [NOT_STARTED] ← Fresh start               │
│ NotesNextSteps  [NOT_STARTED] ← Fresh start               │
└────────────────────────────────────────────────────────────┘
```

### Branching Implementation

```rust
impl Cycle {
    /// Creates a branch from this cycle at the specified component
    pub fn branch_at(&self, branch_point: ComponentType) -> Result<Cycle, DomainError> {
        // 1. Validate can branch
        self.validate_can_branch_at(&branch_point)?;

        let id = CycleId::new();
        let now = Timestamp::now();

        // 2. Determine which components to copy
        let mut new_components = HashMap::new();

        for ct in ComponentType::all() {
            if ct.is_before(&branch_point) {
                // Components before branch point: copy with COMPLETE status
                let parent_component = self.components.get(ct).unwrap();
                new_components.insert(*ct, parent_component.clone());
            } else if *ct == branch_point {
                // Branch point: copy but mark NEEDS_REVISION
                let mut branch_component = self.components.get(ct).unwrap().clone();
                branch_component.set_status(ComponentStatus::NeedsRevision);
                new_components.insert(*ct, branch_component);
            } else {
                // After branch point: fresh component
                new_components.insert(*ct, ComponentVariant::new(*ct));
            }
        }

        // 3. Create branch cycle
        let mut branch = Cycle {
            id,
            session_id: self.session_id,
            parent_cycle_id: Some(self.id),
            branch_point: Some(branch_point),
            status: CycleStatus::Active,
            current_step: branch_point,
            components: new_components,
            created_at: now,
            updated_at: now,
            domain_events: Vec::new(),
        };

        // 4. Record event
        branch.record_event(CycleEvent::Branched {
            cycle_id: id,
            parent_cycle_id: self.id,
            branch_point,
            created_at: now,
        });

        Ok(branch)
    }

    fn validate_can_branch_at(&self, branch_point: &ComponentType) -> Result<(), DomainError> {
        // Must be active cycle
        if !self.status.is_mutable() {
            return Err(DomainError::new(
                ErrorCode::CycleArchived,
                "Cannot branch from archived cycle",
            ));
        }

        // Branch point must be started
        let status = self.component_status(*branch_point);
        if !status.is_started() {
            return Err(DomainError::new(
                ErrorCode::CannotBranch,
                format!("Cannot branch at {:?} - component not started", branch_point),
            ));
        }

        Ok(())
    }
}
```

### Data Reference Handling in Branches

When branching, IDs from inherited components are preserved:

```rust
impl Cycle {
    /// Get objective IDs from Objectives component (used by Consequences)
    pub fn get_objective_ids(&self) -> Vec<Uuid> {
        self.components.get(&ComponentType::Objectives)
            .and_then(|c| c.output())
            .and_then(|o| o.get("fundamental_objectives"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.get("id"))
                    .filter_map(|id| id.as_str())
                    .filter_map(|s| Uuid::parse_str(s).ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get alternative IDs from Alternatives component (used by Consequences)
    pub fn get_alternative_ids(&self) -> Vec<Uuid> {
        self.components.get(&ComponentType::Alternatives)
            .and_then(|c| c.output())
            .and_then(|o| o.get("alternatives"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.get("id"))
                    .filter_map(|id| id.as_str())
                    .filter_map(|s| Uuid::parse_str(s).ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get status quo ID from Alternatives component
    pub fn get_status_quo_id(&self) -> Option<Uuid> {
        self.components.get(&ComponentType::Alternatives)
            .and_then(|c| c.output())
            .and_then(|o| o.get("status_quo_id"))
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
    }
}
```

---

## Navigation Rules

### Navigate To Component

```rust
impl Cycle {
    /// Navigate to a different component (changes current_step)
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
            ComponentStatus::InProgress |
            ComponentStatus::Complete |
            ComponentStatus::NeedsRevision => true,

            // Can navigate to next not-started component
            ComponentStatus::NotStarted => {
                // Check if prerequisite is started
                target.prerequisite()
                    .map(|prereq| self.component_status(prereq).is_started())
                    .unwrap_or(true)
            }
        };

        if !can_navigate {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!("Cannot navigate to {:?} - prerequisite not started", target),
            ));
        }

        // 3. Update current step
        self.current_step = target;
        self.updated_at = Timestamp::now();

        Ok(())
    }
}
```

---

## Cycle Completion

### Complete Cycle Validation

```rust
impl Cycle {
    pub fn complete(&mut self) -> Result<(), DomainError> {
        // 1. Check can transition
        if !self.status.can_transition_to(&CycleStatus::Completed) {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                "Cycle cannot be completed in current state",
            ));
        }

        // 2. Check minimum completion requirements
        // At minimum, DecisionQuality should be complete (user assessed quality)
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
}
```

---

## Tasks

- [x] Implement ComponentStatus enum with transition validation
- [x] Implement prerequisite checking in start_component
- [x] Implement component-specific completion validation rules
- [x] Implement branch_at with proper data inheritance
- [x] Implement NEEDS_REVISION status handling
- [x] Implement navigate_to with accessibility checks
- [x] Write unit tests for all status transitions
- [x] Write unit tests for branching with various branch points
- [ ] Write integration tests for complete cycle workflow

---

## Related Documents

- **Cycle Module:** `docs/modules/cycle.md`
- **Component Schemas:** `features/proact-types/component-schemas.md`
- **Conversation Lifecycle:** `features/conversation/conversation-lifecycle.md`

---

*Version: 1.0.0*
*Created: 2026-01-08*
