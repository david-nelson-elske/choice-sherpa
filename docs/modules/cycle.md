# Cycle Module Specification

## Overview

The Cycle module manages the Cycle aggregate - a complete or partial path through PrOACT. **The Cycle is the aggregate root that owns and persists all components as child entities.** This module supports branching for "what-if" exploration without losing work.

---

## Module Classification

| Attribute | Value |
|-----------|-------|
| **Type** | Full Module (Ports + Adapters) |
| **Language** | Rust |
| **Responsibility** | Cycle lifecycle, component ownership, branching, navigation |
| **Domain Dependencies** | foundation, proact-types, session |
| **External Dependencies** | `async-trait`, `sqlx`, `tokio`, `serde_json` |

---

## Architecture

### Hexagonal Structure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            CYCLE MODULE                                      │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         DOMAIN LAYER                                    │ │
│  │                                                                         │ │
│  │   ┌────────────────────────────────────────────────────────────────┐   │ │
│  │   │                    Cycle Aggregate                              │   │ │
│  │   │                                                                 │   │ │
│  │   │   - id: CycleId                                                 │   │ │
│  │   │   - session_id: SessionId                                       │   │ │
│  │   │   - parent_cycle_id: Option<CycleId>                            │   │ │
│  │   │   - branch_point: Option<ComponentType>                         │   │ │
│  │   │   - status: CycleStatus                                         │   │ │
│  │   │   - current_step: ComponentType                                 │   │ │
│  │   │   - components: HashMap<ComponentType, ComponentVariant>        │   │ │
│  │   │                                                                 │   │ │
│  │   │   + new(session_id) -> Cycle                                    │   │ │
│  │   │   + branch_at(component) -> Cycle                               │   │ │
│  │   │   + start_component(type) -> Result<()>                         │   │ │
│  │   │   + complete_component(type) -> Result<()>                      │   │ │
│  │   │   + navigate_to(type) -> Result<()>                             │   │ │
│  │   └────────────────────────────────────────────────────────────────┘   │ │
│  │                                                                         │ │
│  │   ┌────────────────────┐  ┌────────────────────────────────────────┐   │ │
│  │   │   CycleProgress    │  │           Domain Events                │   │ │
│  │   │   (Value Object)   │  │   CycleCreated, ComponentStarted, etc. │   │ │
│  │   └────────────────────┘  └────────────────────────────────────────┘   │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
│                                      ▼                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                          PORT LAYER                                     │ │
│  │                                                                         │ │
│  │   ┌─────────────────────────────┐  ┌─────────────────────────────────┐ │ │
│  │   │    CycleRepository          │  │    CycleReader                   │ │ │
│  │   │    (Write operations)       │  │    (Query operations - CQRS)    │ │ │
│  │   │                             │  │                                  │ │ │
│  │   │    + save(cycle)            │  │    + get_by_id(id) -> CycleView  │ │ │
│  │   │    + update(cycle)          │  │    + get_cycle_tree(session_id)  │ │ │
│  │   │    + find_by_id(id)         │  │    + get_component_view(...)     │ │ │
│  │   │    + find_by_session(id)    │  │                                  │ │ │
│  │   └─────────────────────────────┘  └─────────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
│                                      ▼                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                        ADAPTER LAYER                                    │ │
│  │                                                                         │ │
│  │   ┌─────────────────┐  ┌─────────────────┐  ┌──────────────────────┐   │ │
│  │   │ PostgresCycle   │  │ PostgresCycle   │  │ ComponentMapper      │   │ │
│  │   │ Repository      │  │ Reader          │  │ (JSONB <-> Rust)     │   │ │
│  │   └─────────────────┘  └─────────────────┘  └──────────────────────┘   │ │
│  │                                                                         │ │
│  │   ┌─────────────────────────────────────────────────────────────────┐  │ │
│  │   │                    HTTP Handlers                                 │  │ │
│  │   │   POST /cycles, POST /cycles/:id/branch, etc.                    │  │ │
│  │   └─────────────────────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Domain Layer

### Cycle Aggregate

```rust
use std::collections::HashMap;
use crate::foundation::{
    ComponentId, ComponentStatus, ComponentType, CycleId, CycleStatus,
    DomainError, ErrorCode, SessionId, Timestamp,
};
use crate::proact::{Component, ComponentVariant};

/// The Cycle aggregate - owns all components as child entities
#[derive(Debug, Clone)]
pub struct Cycle {
    id: CycleId,
    session_id: SessionId,
    parent_cycle_id: Option<CycleId>,
    branch_point: Option<ComponentType>,
    status: CycleStatus,
    current_step: ComponentType,
    components: HashMap<ComponentType, ComponentVariant>,
    /// Component versions for fine-grained optimistic locking.
    /// Each component has independent versioning to reduce contention
    /// when multiple users edit different components simultaneously.
    component_versions: HashMap<ComponentType, u32>,
    created_at: Timestamp,
    updated_at: Timestamp,
    domain_events: Vec<CycleEvent>,
}

impl Cycle {
    /// Creates a new root cycle for a session
    pub fn new(session_id: SessionId) -> Self {
        let id = CycleId::new();
        let now = Timestamp::now();

        // Initialize all 9 components with version 1
        let components = ComponentType::all()
            .iter()
            .map(|ct| (*ct, ComponentVariant::new(*ct)))
            .collect();

        let component_versions = ComponentType::all()
            .iter()
            .map(|ct| (*ct, 1u32))
            .collect();

        let mut cycle = Self {
            id,
            session_id,
            parent_cycle_id: None,
            branch_point: None,
            status: CycleStatus::Active,
            current_step: ComponentType::IssueRaising,
            components,
            component_versions,
            created_at: now,
            updated_at: now,
            domain_events: Vec::new(),
        };

        cycle.record_event(CycleEvent::Created {
            cycle_id: id,
            session_id,
            created_at: now,
        });

        cycle
    }

    /// Creates a branch from this cycle at the specified component
    pub fn branch_at(&self, branch_point: ComponentType) -> Result<Self, DomainError> {
        // Can only branch at started/completed components
        if !self.can_branch_at(&branch_point) {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!("Cannot branch at {:?} - component not started", branch_point),
            ));
        }

        let id = CycleId::new();
        let now = Timestamp::now();

        // Copy components up to and including branch point
        let mut new_components = HashMap::new();
        for ct in ComponentType::all() {
            if ct.is_before(&branch_point) || *ct == branch_point {
                // Clone the component from parent
                new_components.insert(*ct, self.components.get(ct).unwrap().clone());
            } else {
                // Fresh component for remaining steps
                new_components.insert(*ct, ComponentVariant::new(*ct));
            }
        }

        let mut branch = Self {
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

        branch.record_event(CycleEvent::Branched {
            cycle_id: id,
            parent_cycle_id: self.id,
            branch_point,
            created_at: now,
        });

        Ok(branch)
    }

    /// Reconstitutes a cycle from persistence
    pub fn reconstitute(
        id: CycleId,
        session_id: SessionId,
        parent_cycle_id: Option<CycleId>,
        branch_point: Option<ComponentType>,
        status: CycleStatus,
        current_step: ComponentType,
        components: HashMap<ComponentType, ComponentVariant>,
        component_versions: HashMap<ComponentType, u32>,
        created_at: Timestamp,
        updated_at: Timestamp,
    ) -> Self {
        Self {
            id,
            session_id,
            parent_cycle_id,
            branch_point,
            status,
            current_step,
            components,
            component_versions,
            created_at,
            updated_at,
            domain_events: Vec::new(),
        }
    }

    // === Accessors ===

    pub fn id(&self) -> CycleId { self.id }
    pub fn session_id(&self) -> SessionId { self.session_id }
    pub fn parent_cycle_id(&self) -> Option<CycleId> { self.parent_cycle_id }
    pub fn branch_point(&self) -> Option<ComponentType> { self.branch_point }
    pub fn status(&self) -> CycleStatus { self.status }
    pub fn current_step(&self) -> ComponentType { self.current_step }
    pub fn created_at(&self) -> Timestamp { self.created_at }
    pub fn updated_at(&self) -> Timestamp { self.updated_at }

    pub fn is_root(&self) -> bool {
        self.parent_cycle_id.is_none()
    }

    pub fn is_branch(&self) -> bool {
        self.parent_cycle_id.is_some()
    }

    // === Component Access ===

    /// Gets a component by type
    pub fn get_component(&self, ct: ComponentType) -> Option<&ComponentVariant> {
        self.components.get(&ct)
    }

    /// Gets a mutable component by type
    pub fn get_component_mut(&mut self, ct: ComponentType) -> Option<&mut ComponentVariant> {
        self.components.get_mut(&ct)
    }

    /// Gets the current component
    pub fn current_component(&self) -> &ComponentVariant {
        self.components.get(&self.current_step).unwrap()
    }

    // === Component Lifecycle ===

    /// Starts work on a component
    pub fn start_component(&mut self, ct: ComponentType) -> Result<(), DomainError> {
        self.ensure_mutable()?;
        self.validate_can_start(&ct)?;

        let component = self.components.get_mut(&ct)
            .ok_or_else(|| DomainError::new(
                ErrorCode::ComponentNotFound,
                format!("Component {:?} not found", ct),
            ))?;

        // Start the component using the trait method
        match component {
            ComponentVariant::IssueRaising(c) => c.start()?,
            ComponentVariant::ProblemFrame(c) => c.start()?,
            ComponentVariant::Objectives(c) => c.start()?,
            ComponentVariant::Alternatives(c) => c.start()?,
            ComponentVariant::Consequences(c) => c.start()?,
            ComponentVariant::Tradeoffs(c) => c.start()?,
            ComponentVariant::Recommendation(c) => c.start()?,
            ComponentVariant::DecisionQuality(c) => c.start()?,
            ComponentVariant::NotesNextSteps(c) => c.start()?,
        }

        self.current_step = ct;
        self.updated_at = Timestamp::now();

        self.record_event(CycleEvent::ComponentStarted {
            cycle_id: self.id,
            component_type: ct,
        });

        Ok(())
    }

    /// Completes work on a component
    pub fn complete_component(&mut self, ct: ComponentType) -> Result<(), DomainError> {
        self.ensure_mutable()?;

        let component = self.components.get_mut(&ct)
            .ok_or_else(|| DomainError::new(
                ErrorCode::ComponentNotFound,
                format!("Component {:?} not found", ct),
            ))?;

        // Complete the component
        match component {
            ComponentVariant::IssueRaising(c) => c.complete()?,
            ComponentVariant::ProblemFrame(c) => c.complete()?,
            ComponentVariant::Objectives(c) => c.complete()?,
            ComponentVariant::Alternatives(c) => c.complete()?,
            ComponentVariant::Consequences(c) => c.complete()?,
            ComponentVariant::Tradeoffs(c) => c.complete()?,
            ComponentVariant::Recommendation(c) => c.complete()?,
            ComponentVariant::DecisionQuality(c) => c.complete()?,
            ComponentVariant::NotesNextSteps(c) => c.complete()?,
        }

        self.updated_at = Timestamp::now();

        self.record_event(CycleEvent::ComponentCompleted {
            cycle_id: self.id,
            component_type: ct,
        });

        // Auto-advance to next component if available
        if let Some(next) = ct.next() {
            self.current_step = next;
        }

        Ok(())
    }

    /// Updates structured output for a component with optimistic locking.
    ///
    /// Uses component-level versioning to allow concurrent edits to different
    /// components without conflict. Returns the new version on success.
    pub fn update_component_output(
        &mut self,
        ct: ComponentType,
        output: serde_json::Value,
        expected_version: u32,
    ) -> Result<u32, DomainError> {
        self.ensure_mutable()?;

        // Check version for optimistic locking
        let current_version = self.component_versions.get(&ct).copied().unwrap_or(1);
        if current_version != expected_version {
            return Err(DomainError::new(
                ErrorCode::ConcurrencyConflict,
                format!(
                    "Component {} was modified (expected version {}, found {})",
                    ct, expected_version, current_version
                ),
            ));
        }

        let component = self.components.get_mut(&ct)
            .ok_or_else(|| DomainError::new(
                ErrorCode::ComponentNotFound,
                format!("Component {:?} not found", ct),
            ))?;

        // Set output using the trait method
        match component {
            ComponentVariant::IssueRaising(c) => {
                c.set_output_from_value(output)?;
            }
            ComponentVariant::ProblemFrame(c) => {
                c.set_output_from_value(output)?;
            }
            ComponentVariant::Objectives(c) => {
                c.set_output_from_value(output)?;
            }
            ComponentVariant::Alternatives(c) => {
                c.set_output_from_value(output)?;
            }
            ComponentVariant::Consequences(c) => {
                c.set_output_from_value(output)?;
            }
            ComponentVariant::Tradeoffs(c) => {
                c.set_output_from_value(output)?;
            }
            ComponentVariant::Recommendation(c) => {
                c.set_output_from_value(output)?;
            }
            ComponentVariant::DecisionQuality(c) => {
                c.set_output_from_value(output)?;
            }
            ComponentVariant::NotesNextSteps(c) => {
                c.set_output_from_value(output)?;
            }
        }

        // Increment version
        let new_version = current_version + 1;
        self.component_versions.insert(ct, new_version);
        self.updated_at = Timestamp::now();

        self.record_event(CycleEvent::ComponentOutputUpdated {
            cycle_id: self.id,
            component_type: ct,
            version: new_version,
        });

        Ok(new_version)
    }

    /// Get the current version of a component
    pub fn get_component_version(&self, ct: ComponentType) -> u32 {
        self.component_versions.get(&ct).copied().unwrap_or(1)
    }

    // === Navigation ===

    /// Navigates to a specific component (changes current step)
    pub fn navigate_to(&mut self, ct: ComponentType) -> Result<(), DomainError> {
        self.ensure_mutable()?;

        // Can navigate to any started component or the next not-started one
        let component_status = self.components.get(&ct)
            .map(|c| c.status())
            .ok_or_else(|| DomainError::new(
                ErrorCode::ComponentNotFound,
                format!("Component {:?} not found", ct),
            ))?;

        let can_navigate = component_status.is_started() ||
            ct.previous().map(|prev| {
                self.components.get(&prev)
                    .map(|c| c.status().is_started())
                    .unwrap_or(false)
            }).unwrap_or(ct == ComponentType::IssueRaising);

        if !can_navigate {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!("Cannot navigate to {:?} - previous component not started", ct),
            ));
        }

        self.current_step = ct;
        self.updated_at = Timestamp::now();

        Ok(())
    }

    // === Progress ===

    /// Calculates current progress through the cycle
    pub fn get_progress(&self) -> CycleProgress {
        let total = ComponentType::all().len();
        let completed = self.components.values()
            .filter(|c| c.status().is_complete())
            .count();

        let step_statuses: HashMap<ComponentType, ComponentStatus> = self.components
            .iter()
            .map(|(ct, c)| (*ct, c.status()))
            .collect();

        CycleProgress {
            total_steps: total,
            completed_steps: completed,
            current_step: self.current_step,
            step_statuses,
        }
    }

    // === Lifecycle ===

    /// Marks the cycle as completed
    pub fn complete(&mut self) -> Result<(), DomainError> {
        if !self.status.can_transition_to(&CycleStatus::Completed) {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                "Cycle cannot be completed in current state",
            ));
        }

        self.status = CycleStatus::Completed;
        self.updated_at = Timestamp::now();

        self.record_event(CycleEvent::Completed { cycle_id: self.id });

        Ok(())
    }

    /// Archives the cycle
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

    // === Branching ===

    /// Checks if branching is allowed at a component
    pub fn can_branch_at(&self, ct: &ComponentType) -> bool {
        if !self.status.is_mutable() {
            return false;
        }
        self.components.get(ct)
            .map(|c| c.status().is_started())
            .unwrap_or(false)
    }

    // === Domain Events ===

    pub fn pull_domain_events(&mut self) -> Vec<CycleEvent> {
        std::mem::take(&mut self.domain_events)
    }

    // === Private Helpers ===

    fn ensure_mutable(&self) -> Result<(), DomainError> {
        if !self.status.is_mutable() {
            return Err(DomainError::new(
                ErrorCode::CycleArchived,
                "Cannot modify a completed or archived cycle",
            ));
        }
        Ok(())
    }

    fn validate_can_start(&self, ct: &ComponentType) -> Result<(), DomainError> {
        // Can start if:
        // 1. It's the first component (IssueRaising)
        // 2. Previous component has been started

        if *ct == ComponentType::IssueRaising {
            return Ok(());
        }

        let prev = ct.previous().unwrap();
        let prev_started = self.components.get(&prev)
            .map(|c| c.status().is_started())
            .unwrap_or(false);

        if !prev_started {
            return Err(DomainError::new(
                ErrorCode::InvalidStateTransition,
                format!("Cannot start {:?} before {:?}", ct, prev),
            ));
        }

        Ok(())
    }

    fn record_event(&mut self, event: CycleEvent) {
        self.domain_events.push(event);
    }
}
```

### CycleProgress Value Object

```rust
use std::collections::HashMap;
use crate::foundation::{ComponentStatus, ComponentType};
use serde::Serialize;

/// Represents progress through a cycle
#[derive(Debug, Clone, Serialize)]
pub struct CycleProgress {
    pub total_steps: usize,
    pub completed_steps: usize,
    pub current_step: ComponentType,
    pub step_statuses: HashMap<ComponentType, ComponentStatus>,
}

impl CycleProgress {
    /// Returns completion percentage (0-100)
    pub fn percent_complete(&self) -> u8 {
        if self.total_steps == 0 {
            return 0;
        }
        ((self.completed_steps * 100) / self.total_steps) as u8
    }

    /// Returns true if all steps are complete
    pub fn is_complete(&self) -> bool {
        self.completed_steps == self.total_steps
    }

    /// Returns the first incomplete step
    pub fn first_incomplete(&self) -> Option<ComponentType> {
        ComponentType::all()
            .iter()
            .find(|ct| {
                self.step_statuses.get(ct)
                    .map(|s| !s.is_complete())
                    .unwrap_or(true)
            })
            .copied()
    }
}
```

### Domain Events

```rust
use crate::foundation::{ComponentType, CycleId, SessionId, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CycleEvent {
    Created {
        cycle_id: CycleId,
        session_id: SessionId,
        created_at: Timestamp,
    },
    Branched {
        cycle_id: CycleId,
        parent_cycle_id: CycleId,
        branch_point: ComponentType,
        created_at: Timestamp,
    },
    ComponentStarted {
        cycle_id: CycleId,
        component_type: ComponentType,
    },
    ComponentCompleted {
        cycle_id: CycleId,
        component_type: ComponentType,
    },
    ComponentOutputUpdated {
        cycle_id: CycleId,
        component_type: ComponentType,
        version: u32,
    },
    Completed {
        cycle_id: CycleId,
    },
    Archived {
        cycle_id: CycleId,
    },
}

impl CycleEvent {
    pub fn cycle_id(&self) -> CycleId {
        match self {
            CycleEvent::Created { cycle_id, .. } => *cycle_id,
            CycleEvent::Branched { cycle_id, .. } => *cycle_id,
            CycleEvent::ComponentStarted { cycle_id, .. } => *cycle_id,
            CycleEvent::ComponentCompleted { cycle_id, .. } => *cycle_id,
            CycleEvent::ComponentOutputUpdated { cycle_id, .. } => *cycle_id,
            CycleEvent::Completed { cycle_id } => *cycle_id,
            CycleEvent::Archived { cycle_id } => *cycle_id,
        }
    }
}
```

---

## Ports

### CycleRepository (Write)

```rust
use async_trait::async_trait;
use crate::foundation::{CycleId, SessionId};
use super::Cycle;

#[async_trait]
pub trait CycleRepository: Send + Sync {
    /// Persists a new cycle with all its components
    async fn save(&self, cycle: &Cycle) -> Result<(), RepositoryError>;

    /// Updates an existing cycle and its components
    async fn update(&self, cycle: &Cycle) -> Result<(), RepositoryError>;

    /// Finds a cycle by ID with all components
    async fn find_by_id(&self, id: CycleId) -> Result<Option<Cycle>, RepositoryError>;

    /// Finds all cycles for a session
    async fn find_by_session(&self, session_id: SessionId) -> Result<Vec<Cycle>, RepositoryError>;

    /// Checks if a cycle exists
    async fn exists(&self, id: CycleId) -> Result<bool, RepositoryError>;
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Cycle not found: {0}")]
    NotFound(CycleId),

    #[error("Duplicate cycle ID: {0}")]
    DuplicateId(CycleId),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
```

### CycleReader (Query - CQRS)

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::foundation::{ComponentId, ComponentType, CycleId, CycleStatus, SessionId};

#[async_trait]
pub trait CycleReader: Send + Sync {
    /// Gets a cycle view by ID
    async fn get_by_id(&self, id: CycleId) -> Result<Option<CycleView>, ReaderError>;

    /// Gets the cycle tree for a session
    async fn get_cycle_tree(&self, session_id: SessionId) -> Result<CycleTree, ReaderError>;

    /// Gets a specific component view
    async fn get_component_view(
        &self,
        cycle_id: CycleId,
        component_type: ComponentType,
    ) -> Result<Option<ComponentView>, ReaderError>;

    // === Cross-Module Lookup Methods ===
    // These methods enable authorization in other modules (conversation, dashboard)
    // by allowing lookup from component ID back to cycle and session.

    /// Find the cycle containing a specific component
    /// Used by conversation module for authorization through parent chain
    async fn find_by_component(
        &self,
        component_id: &ComponentId,
    ) -> Result<Option<CycleView>, ReaderError>;

    /// Get cycle_id for a component (lightweight lookup without full cycle data)
    /// Optimized for frequent authorization checks
    async fn get_cycle_id_for_component(
        &self,
        component_id: &ComponentId,
    ) -> Result<Option<CycleId>, ReaderError>;
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CycleView {
    pub id: CycleId,
    pub session_id: SessionId,
    pub parent_cycle_id: Option<CycleId>,
    pub branch_point: Option<ComponentType>,
    pub status: CycleStatus,
    pub progress: CycleProgress,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CycleTree {
    pub root_cycles: Vec<CycleNode>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CycleNode {
    pub cycle_id: CycleId,
    pub status: CycleStatus,
    pub progress: CycleProgress,
    pub branch_point: Option<ComponentType>,
    pub children: Vec<CycleNode>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ComponentView {
    pub id: ComponentId,
    pub cycle_id: CycleId,
    pub component_type: ComponentType,
    pub status: ComponentStatus,
    pub structured_output: serde_json::Value,
    /// Version for optimistic locking. Client should include this
    /// in update requests to detect concurrent modifications.
    pub version: u32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum ReaderError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

---

## Application Layer

### Commands

| Command | Description |
|---------|-------------|
| `CreateCycle` | Start new root cycle in session |
| `BranchCycle` | Create branch from existing cycle |
| `StartComponent` | Begin work on a component |
| `CompleteComponent` | Mark component as done |
| `UpdateComponentOutput` | Save structured data |
| `NavigateToComponent` | Change current step |
| `CompleteCycle` | Mark cycle as finished |
| `ArchiveCycle` | Archive a cycle |

#### CreateCycle Command

```rust
use std::sync::Arc;
use crate::foundation::{CycleId, SessionId, UserId};
use crate::ports::{CycleRepository, SessionRepository, DomainEventPublisher};
use crate::domain::Cycle;

#[derive(Debug, Clone)]
pub struct CreateCycleCommand {
    pub session_id: SessionId,
    pub user_id: UserId,
}

pub struct CreateCycleHandler {
    cycle_repo: Arc<dyn CycleRepository>,
    session_repo: Arc<dyn SessionRepository>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl CreateCycleHandler {
    pub async fn handle(&self, cmd: CreateCycleCommand) -> Result<CycleId, CommandError> {
        // Verify session exists and user owns it
        let mut session = self.session_repo
            .find_by_id(cmd.session_id)
            .await?
            .ok_or(CommandError::SessionNotFound(cmd.session_id))?;

        session.authorize(&cmd.user_id)
            .map_err(|_| CommandError::Unauthorized)?;

        // Create cycle
        let mut cycle = Cycle::new(cmd.session_id);
        let cycle_id = cycle.id();

        // Link cycle to session
        session.add_cycle(cycle_id)?;

        // Persist
        self.cycle_repo.save(&cycle).await?;
        self.session_repo.update(&session).await?;

        // Publish events
        let cycle_events = cycle.pull_domain_events();
        let session_events = session.pull_domain_events();
        self.publisher.publish_cycle_events(cycle_events).await?;
        self.publisher.publish_session_events(session_events).await?;

        Ok(cycle_id)
    }
}
```

#### BranchCycle Command

```rust
#[derive(Debug, Clone)]
pub struct BranchCycleCommand {
    pub cycle_id: CycleId,
    pub branch_point: ComponentType,
    pub user_id: UserId,
}

pub struct BranchCycleHandler {
    cycle_repo: Arc<dyn CycleRepository>,
    session_repo: Arc<dyn SessionRepository>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl BranchCycleHandler {
    pub async fn handle(&self, cmd: BranchCycleCommand) -> Result<CycleId, CommandError> {
        // Load parent cycle
        let parent = self.cycle_repo
            .find_by_id(cmd.cycle_id)
            .await?
            .ok_or(CommandError::CycleNotFound(cmd.cycle_id))?;

        // Authorize via session
        let mut session = self.session_repo
            .find_by_id(parent.session_id())
            .await?
            .ok_or(CommandError::SessionNotFound(parent.session_id()))?;

        session.authorize(&cmd.user_id)
            .map_err(|_| CommandError::Unauthorized)?;

        // Create branch
        let mut branch = parent.branch_at(cmd.branch_point)?;
        let branch_id = branch.id();

        // Link to session
        session.add_cycle(branch_id)?;

        // Persist
        self.cycle_repo.save(&branch).await?;
        self.session_repo.update(&session).await?;

        // Publish events
        let events = branch.pull_domain_events();
        self.publisher.publish_cycle_events(events).await?;

        Ok(branch_id)
    }
}
```

#### StartComponent Command

```rust
#[derive(Debug, Clone)]
pub struct StartComponentCommand {
    pub cycle_id: CycleId,
    pub component_type: ComponentType,
    pub user_id: UserId,
}

pub struct StartComponentHandler {
    cycle_repo: Arc<dyn CycleRepository>,
    session_repo: Arc<dyn SessionRepository>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl StartComponentHandler {
    pub async fn handle(&self, cmd: StartComponentCommand) -> Result<(), CommandError> {
        // Load cycle
        let mut cycle = self.cycle_repo
            .find_by_id(cmd.cycle_id)
            .await?
            .ok_or(CommandError::CycleNotFound(cmd.cycle_id))?;

        // SECURITY: Load parent session and verify ownership (IDOR prevention)
        let session = self.session_repo
            .find_by_id(cycle.session_id())
            .await?
            .ok_or(CommandError::SessionNotFound(cycle.session_id()))?;

        // CRITICAL: Verify user owns the session (A01 Broken Access Control)
        session.authorize(&cmd.user_id)
            .map_err(|_| CommandError::Unauthorized)?;

        // Execute business logic
        cycle.start_component(cmd.component_type)?;

        // Persist
        self.cycle_repo.update(&cycle).await?;

        // Publish events
        let events = cycle.pull_domain_events();
        self.publisher.publish_cycle_events(events).await?;

        Ok(())
    }
}
```

#### CompleteComponent Command

```rust
#[derive(Debug, Clone)]
pub struct CompleteComponentCommand {
    pub cycle_id: CycleId,
    pub component_type: ComponentType,
    pub user_id: UserId,
}

pub struct CompleteComponentHandler {
    cycle_repo: Arc<dyn CycleRepository>,
    session_repo: Arc<dyn SessionRepository>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl CompleteComponentHandler {
    pub async fn handle(&self, cmd: CompleteComponentCommand) -> Result<(), CommandError> {
        // Load cycle
        let mut cycle = self.cycle_repo
            .find_by_id(cmd.cycle_id)
            .await?
            .ok_or(CommandError::CycleNotFound(cmd.cycle_id))?;

        // SECURITY: Load parent session and verify ownership (IDOR prevention)
        let session = self.session_repo
            .find_by_id(cycle.session_id())
            .await?
            .ok_or(CommandError::SessionNotFound(cycle.session_id()))?;

        // CRITICAL: Verify user owns the session (A01 Broken Access Control)
        session.authorize(&cmd.user_id)
            .map_err(|_| CommandError::Unauthorized)?;

        // Execute business logic
        cycle.complete_component(cmd.component_type)?;

        // Persist
        self.cycle_repo.update(&cycle).await?;

        // Publish events
        let events = cycle.pull_domain_events();
        self.publisher.publish_cycle_events(events).await?;

        Ok(())
    }
}
```

#### UpdateComponentOutput Command

```rust
#[derive(Debug, Clone)]
pub struct UpdateComponentOutputCommand {
    pub cycle_id: CycleId,
    pub component_type: ComponentType,
    pub output: serde_json::Value,
    /// Expected version for optimistic locking.
    /// Client must include the version from the last read.
    pub expected_version: u32,
    pub user_id: UserId,
}

/// Result of updating a component, includes new version for client
#[derive(Debug, Clone, serde::Serialize)]
pub struct UpdateComponentResult {
    pub cycle_id: CycleId,
    pub component_type: ComponentType,
    pub new_version: u32,
}

pub struct UpdateComponentOutputHandler {
    cycle_repo: Arc<dyn CycleRepository>,
    session_repo: Arc<dyn SessionRepository>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl UpdateComponentOutputHandler {
    pub async fn handle(&self, cmd: UpdateComponentOutputCommand) -> Result<UpdateComponentResult, CommandError> {
        // Load cycle
        let mut cycle = self.cycle_repo
            .find_by_id(cmd.cycle_id)
            .await?
            .ok_or(CommandError::CycleNotFound(cmd.cycle_id))?;

        // SECURITY: Load parent session and verify ownership (IDOR prevention)
        let session = self.session_repo
            .find_by_id(cycle.session_id())
            .await?
            .ok_or(CommandError::SessionNotFound(cycle.session_id()))?;

        // CRITICAL: Verify user owns the session (A01 Broken Access Control)
        session.authorize(&cmd.user_id)
            .map_err(|_| CommandError::Unauthorized)?;

        // Execute business logic with optimistic locking
        let new_version = cycle.update_component_output(
            cmd.component_type,
            cmd.output,
            cmd.expected_version,
        )?;

        // Persist with version check at DB level
        self.cycle_repo.update(&cycle).await?;

        // Publish events
        let events = cycle.pull_domain_events();
        self.publisher.publish_cycle_events(events).await?;

        Ok(UpdateComponentResult {
            cycle_id: cmd.cycle_id,
            component_type: cmd.component_type,
            new_version,
        })
    }
}
```

### Queries

| Query | Returns |
|-------|---------|
| `GetCycle` | Full cycle with progress |
| `GetCycleTree` | Session's cycle hierarchy |
| `GetComponent` | Single component view |

---

## Adapters

### HTTP Endpoints

| Method | Path | Handler |
|--------|------|---------|
| `POST` | `/api/sessions/:sessionId/cycles` | CreateCycle |
| `GET` | `/api/sessions/:sessionId/cycles` | GetCycleTree |
| `GET` | `/api/cycles/:id` | GetCycle |
| `POST` | `/api/cycles/:id/branch` | BranchCycle |
| `POST` | `/api/cycles/:id/navigate` | NavigateToComponent |
| `POST` | `/api/cycles/:id/complete` | CompleteCycle |
| `GET` | `/api/cycles/:id/components/:type` | GetComponent |
| `POST` | `/api/cycles/:id/components/:type/start` | StartComponent |
| `POST` | `/api/cycles/:id/components/:type/complete` | CompleteComponent |
| `PUT` | `/api/cycles/:id/components/:type` | UpdateComponentOutput |

### Database Schema

```sql
CREATE TABLE cycles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    parent_cycle_id UUID REFERENCES cycles(id),
    branch_point VARCHAR(50),
    current_step VARCHAR(50) NOT NULL DEFAULT 'issue_raising',
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT cycles_status_valid CHECK (status IN ('active', 'completed', 'archived')),
    CONSTRAINT cycles_current_step_valid CHECK (current_step IN (
        'issue_raising', 'problem_frame', 'objectives', 'alternatives',
        'consequences', 'tradeoffs', 'recommendation', 'decision_quality',
        'notes_next_steps'
    ))
);

CREATE TABLE components (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cycle_id UUID NOT NULL REFERENCES cycles(id) ON DELETE CASCADE,
    component_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'not_started',
    structured_data JSONB NOT NULL DEFAULT '{}',
    -- Version for optimistic locking (component-level, not aggregate-level)
    -- Allows concurrent edits to different components without conflict
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(cycle_id, component_type),
    CONSTRAINT components_status_valid CHECK (status IN (
        'not_started', 'in_progress', 'complete', 'needs_revision'
    )),
    CONSTRAINT components_type_valid CHECK (component_type IN (
        'issue_raising', 'problem_frame', 'objectives', 'alternatives',
        'consequences', 'tradeoffs', 'recommendation', 'decision_quality',
        'notes_next_steps'
    ))
);

-- Indexes
CREATE INDEX idx_cycles_session ON cycles(session_id);
CREATE INDEX idx_cycles_parent ON cycles(parent_cycle_id);
CREATE INDEX idx_cycles_status ON cycles(status);
CREATE INDEX idx_components_cycle ON components(cycle_id);
CREATE INDEX idx_components_type ON components(component_type);

-- Index for cross-module component -> cycle lookup (conversation authorization)
CREATE INDEX idx_components_id ON components(id);
```

### PostgresCycleReader Implementation (Cross-Module Lookups)

```rust
impl CycleReader for PostgresCycleReader {
    // ... existing methods ...

    async fn find_by_component(
        &self,
        component_id: &ComponentId,
    ) -> Result<Option<CycleView>, ReaderError> {
        let row = sqlx::query_as!(
            CycleRow,
            r#"
            SELECT c.id, c.session_id, c.parent_cycle_id, c.branch_point,
                   c.current_step, c.status, c.created_at, c.updated_at
            FROM cycles c
            JOIN components comp ON c.id = comp.cycle_id
            WHERE comp.id = $1
            "#,
            component_id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(cycle_row) => {
                // Load components and build progress
                let components = self.load_components(cycle_row.id).await?;
                let progress = self.calculate_progress(&components);

                Ok(Some(CycleView {
                    id: CycleId::from(cycle_row.id),
                    session_id: SessionId::from(cycle_row.session_id),
                    parent_cycle_id: cycle_row.parent_cycle_id.map(CycleId::from),
                    branch_point: cycle_row.branch_point.map(|s| s.parse().unwrap()),
                    status: cycle_row.status.parse().unwrap(),
                    progress,
                    created_at: cycle_row.created_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_cycle_id_for_component(
        &self,
        component_id: &ComponentId,
    ) -> Result<Option<CycleId>, ReaderError> {
        let row = sqlx::query_scalar!(
            r#"
            SELECT cycle_id
            FROM components
            WHERE id = $1
            "#,
            component_id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(CycleId::from))
    }
}
```

---

## File Structure

```
backend/src/domain/cycle/
├── mod.rs                  # Module exports
├── cycle.rs                # Cycle aggregate
├── cycle_test.rs           # Aggregate tests
├── progress.rs             # CycleProgress value object
├── progress_test.rs
├── events.rs               # CycleEvent enum
└── errors.rs               # Cycle-specific errors

backend/src/ports/
├── cycle_repository.rs     # CycleRepository trait
└── cycle_reader.rs         # CycleReader trait

backend/src/application/
├── commands/
│   ├── create_cycle.rs
│   ├── create_cycle_test.rs
│   ├── branch_cycle.rs
│   ├── branch_cycle_test.rs
│   ├── start_component.rs
│   ├── complete_component.rs
│   ├── update_component_output.rs
│   └── navigate_component.rs
└── queries/
    ├── get_cycle.rs
    ├── get_cycle_tree.rs
    └── get_component.rs

backend/src/adapters/
├── http/cycle/
│   ├── handlers.rs
│   ├── handlers_test.rs
│   ├── dto.rs
│   └── routes.rs
└── postgres/
    ├── cycle_repository.rs
    ├── cycle_repository_test.rs
    ├── cycle_reader.rs
    └── component_mapper.rs     # Maps JSONB to component types

frontend/src/modules/cycle/
├── domain/
│   ├── cycle.ts
│   ├── cycle.test.ts
│   ├── progress.ts
│   └── cycle-tree.ts
├── api/
│   ├── cycle-api.ts
│   ├── use-cycle.ts
│   └── use-cycle-tree.ts
├── components/
│   ├── CycleTree.tsx
│   ├── CycleTree.test.tsx
│   ├── CycleProgress.tsx
│   ├── ComponentNav.tsx
│   ├── ComponentNav.test.tsx
│   └── BranchDialog.tsx
└── index.ts
```

---

## Invariants

| Invariant | Enforcement |
|-----------|-------------|
| Cycle belongs to session | Constructor and reconstitute require session_id |
| All 9 components exist | Created in constructor |
| Components follow order | validate_can_start() check |
| Only one component in_progress | Implicit via status machine |
| Branch point must be started | can_branch_at() check |
| Branch inherits state up to point | branch_at() copies components |
| Completed/archived cycles immutable | ensure_mutable() check |
| Component version monotonically increases | update_component_output() increments |
| Concurrent edits to same component rejected | Version check before update |

---

## Scaling Considerations

### Component-Level Versioning

The Cycle aggregate uses **component-level versioning** to reduce concurrency conflicts when multiple users edit different parts of the same cycle simultaneously.

**Problem Solved:**
```
Without component-level versioning:
T1: Server A loads Cycle (version 5)
T2: Server B loads Cycle (version 5)
T3: Server A updates IssueRaising, saves (version 6) ✅
T4: Server B updates Objectives, saves (version 6) ❌ CONFLICT!
```

**With component-level versioning:**
```
T1: Server A loads Cycle (IssueRaising v3, Objectives v2)
T2: Server B loads Cycle (IssueRaising v3, Objectives v2)
T3: Server A updates IssueRaising (v3→v4) ✅
T4: Server B updates Objectives (v2→v3) ✅ (different component)
```

### Client-Side Integration

```typescript
// frontend/src/lib/cycle/use-component.ts

export function useComponent(cycleId: string, componentType: ComponentType) {
    const [component, setComponent] = useState<ComponentView | null>(null);

    const updateComponent = async (output: any) => {
        if (!component) return;

        try {
            const result = await api.updateComponent(cycleId, componentType, {
                output,
                expected_version: component.version,  // Include version
            });

            setComponent(prev => ({
                ...prev!,
                structured_output: output,
                version: result.new_version,  // Update local version
            }));
        } catch (e) {
            if (e.code === 'CONCURRENCY_CONFLICT') {
                // Refresh and show merge dialog
                const fresh = await api.getComponent(cycleId, componentType);
                showConflictDialog(component, fresh);
            }
            throw e;
        }
    };

    return { component, updateComponent };
}
```

### Database Update Pattern

```sql
-- Update with version check (repository adapter)
UPDATE components
SET structured_data = $1,
    version = version + 1,
    updated_at = NOW()
WHERE cycle_id = $2
  AND component_type = $3
  AND version = $4  -- Optimistic lock check
RETURNING version;

-- If no rows returned, concurrent modification occurred
```

See [SCALING-READINESS.md](../architecture/SCALING-READINESS.md) for full scaling architecture.

---

## Test Categories

### Unit Tests (Domain)

| Category | Example Tests |
|----------|---------------|
| Creation | `new_cycle_has_all_nine_components` |
| Creation | `new_cycle_starts_at_issue_raising` |
| Branching | `branch_copies_components_up_to_point` |
| Branching | `cannot_branch_at_not_started_component` |
| Component | `start_component_changes_to_in_progress` |
| Component | `cannot_start_without_previous_started` |
| Navigation | `navigate_to_started_component_succeeds` |
| Progress | `percent_complete_calculates_correctly` |

### Integration Tests

| Category | Example Tests |
|----------|---------------|
| Repository | `save_persists_cycle_with_components` |
| Repository | `update_modifies_component_output` |
| Tree | `get_cycle_tree_builds_hierarchy` |

---

*Module Version: 1.1.0*
*Based on: SYSTEM-ARCHITECTURE.md v1.3.0*
*Language: Rust*
*Updated: 2026-01-08 (Component-Level Versioning for Scaling)*
