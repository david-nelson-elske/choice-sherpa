# Feature: Cycle Domain Events

**Module:** cycle
**Type:** Event Publishing
**Priority:** P0
**Phase:** 3 of Full PrOACT Journey Integration
**Depends On:** features/foundation/event-infrastructure.md

> Cycle module publishes domain events for cycle and component lifecycle changes, enabling real-time dashboard updates and cross-module coordination.

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Required |
| Authorization Model | Inherited ownership: user must own parent session to access cycle |
| Sensitive Data | Component outputs (Confidential); Decision progress (Internal) |
| Rate Limiting | Required: 200 requests/minute per user for cycle/component operations |
| Audit Logging | All cycle lifecycle events and component state changes |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| cycle_id | Internal | Log in audit events |
| session_id | Internal | Used for authorization chain lookup |
| component_id | Internal | Log in audit events |
| component_type | Internal | Safe to log, no sensitive data |
| parent_cycle_id | Internal | Log for branching audit trail |
| branch_point | Internal | Log for branching audit trail |
| inherited_components | Internal | List of component types, no sensitive data |
| change_summary | Confidential | Truncated output data, may contain decision details |
| dq_overall_score | Internal | Numeric score, no PII |
| progress (CycleProgressSnapshot) | Internal | Numeric progress indicators |

### Security Events to Log

- CycleCreated: INFO level with session_id, cycle_id, parent_cycle_id (if branched)
- CycleBranched: INFO level with parent_cycle_id, branch_point, inherited components count
- ComponentStarted: INFO level with cycle_id, component_type, timestamp
- ComponentCompleted: INFO level with cycle_id, component_type, progress snapshot
- ComponentOutputUpdated: DEBUG level with cycle_id, component_type (omit actual output)
- CycleCompleted: INFO level with cycle_id, dq_overall_score
- CycleArchived: INFO level with cycle_id, timestamp
- Authorization failures (WARN level): Unauthorized cycle/component access attempts
- Invalid state transitions (WARN level): Attempts to violate component ordering

### Authorization Rules

1. **Cycle Creation**: User must own the parent session (`session.is_owner(&user_id)`)
2. **Cycle Branching**: User must own the parent session of the source cycle
3. **Component Operations**: User must own the session containing the cycle
4. **Event Routing**: WebSocket events routed only to session owner via session_id

---

## Problem Statement

The cycle module manages the core PrOACT workflow but operates in isolation. Other modules need visibility into:
- When cycles are created or branched
- When components are started, updated, or completed
- When a full cycle is completed

Without events, the dashboard would need to poll constantly, and the conversation module wouldn't know when to initialize for a new component.

### Current State

- Cycle operations complete without notification
- Dashboard must poll for progress updates
- Conversation module doesn't know when components start
- No branching visibility across the system

### Desired State

- Every significant cycle/component change publishes an event
- Dashboard receives real-time progress updates
- Conversation initializes automatically when component starts
- Analysis triggers automatically when relevant components complete

---

## Domain Events

### CycleCreated

Published when a new root cycle is created in a session.

```rust
// backend/src/domain/cycle/events.rs

use serde::{Deserialize, Serialize};

/// Published when a new cycle is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleCreated {
    pub event_id: EventId,
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    pub parent_cycle_id: Option<CycleId>,
    pub created_at: Timestamp,
}

impl DomainEvent for CycleCreated {
    fn event_type(&self) -> &'static str {
        "cycle.created"
    }

    fn aggregate_id(&self) -> String {
        self.cycle_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.created_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

**Subscribers:**
- `SessionCycleTracker` - Add cycle to session's cycle list
- `DashboardUpdateHandler` - Show new cycle in tree view

---

### CycleBranched

Published when a cycle branches from an existing cycle at a specific component.

```rust
/// Published when a cycle is branched from another
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleBranched {
    pub event_id: EventId,
    pub cycle_id: CycleId,
    pub parent_cycle_id: CycleId,
    pub session_id: SessionId,
    pub branch_point: ComponentType,
    pub inherited_components: Vec<ComponentType>,
    pub branched_at: Timestamp,
}

impl DomainEvent for CycleBranched {
    fn event_type(&self) -> &'static str {
        "cycle.branched"
    }

    fn aggregate_id(&self) -> String {
        self.cycle_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.branched_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

**Subscribers:**
- `SessionCycleTracker` - Add branch to session's cycle list
- `DashboardUpdateHandler` - Show branch in cycle tree

---

### ComponentStarted

Published when work begins on a component within a cycle.

```rust
/// Published when a component is started
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStarted {
    pub event_id: EventId,
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    pub component_id: ComponentId,
    pub component_type: ComponentType,
    pub started_at: Timestamp,
}

impl DomainEvent for ComponentStarted {
    fn event_type(&self) -> &'static str {
        "component.started"
    }

    fn aggregate_id(&self) -> String {
        self.cycle_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.started_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

**Subscribers:**
- `ConversationInitHandler` - Initialize conversation for component
- `DashboardUpdateHandler` - Update progress indicator

---

### ComponentOutputUpdated

Published when structured data is extracted/updated for a component.

```rust
/// Published when component output is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentOutputUpdated {
    pub event_id: EventId,
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    pub component_id: ComponentId,
    pub component_type: ComponentType,
    /// Brief summary of what changed (for logging, not full data)
    pub change_summary: String,
    pub updated_at: Timestamp,
}

impl DomainEvent for ComponentOutputUpdated {
    fn event_type(&self) -> &'static str {
        "component.output_updated"
    }

    fn aggregate_id(&self) -> String {
        self.cycle_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.updated_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

**Subscribers:**
- `DashboardUpdateHandler` - Refresh component data in view

---

### ComponentCompleted

Published when a component is marked as complete.

```rust
/// Published when a component is completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentCompleted {
    pub event_id: EventId,
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    pub component_id: ComponentId,
    pub component_type: ComponentType,
    pub completed_at: Timestamp,
    /// Current progress after completion
    pub progress: CycleProgressSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleProgressSnapshot {
    pub completed_count: i32,
    pub total_count: i32,
    pub percent_complete: i32,
    pub current_step: ComponentType,
}

impl DomainEvent for ComponentCompleted {
    fn event_type(&self) -> &'static str {
        "component.completed"
    }

    fn aggregate_id(&self) -> String {
        self.cycle_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.completed_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

**Subscribers:**
- `DashboardUpdateHandler` - Update progress bar
- `AnalysisTriggerHandler` - Trigger analysis for Consequences/DQ components

---

### ComponentMarkedForRevision

Published when a component needs to be revisited.

```rust
/// Published when a component is marked for revision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMarkedForRevision {
    pub event_id: EventId,
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    pub component_id: ComponentId,
    pub component_type: ComponentType,
    pub reason: String,
    pub marked_at: Timestamp,
}

impl DomainEvent for ComponentMarkedForRevision {
    fn event_type(&self) -> &'static str {
        "component.marked_for_revision"
    }

    fn aggregate_id(&self) -> String {
        self.cycle_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.marked_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

---

### CycleCompleted

Published when all components in a cycle are complete.

```rust
/// Published when an entire cycle is completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleCompleted {
    pub event_id: EventId,
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    pub dq_overall_score: Option<Percentage>,
    pub completed_at: Timestamp,
}

impl DomainEvent for CycleCompleted {
    fn event_type(&self) -> &'static str {
        "cycle.completed"
    }

    fn aggregate_id(&self) -> String {
        self.cycle_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.completed_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

**Subscribers:**
- `DashboardUpdateHandler` - Mark cycle complete in view
- `NotificationHandler` (future) - Notify user of completion

---

### CycleArchived

Published when a cycle is archived.

```rust
/// Published when a cycle is archived
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleArchived {
    pub event_id: EventId,
    pub cycle_id: CycleId,
    pub session_id: SessionId,
    pub archived_at: Timestamp,
}

impl DomainEvent for CycleArchived {
    fn event_type(&self) -> &'static str {
        "cycle.archived"
    }

    fn aggregate_id(&self) -> String {
        self.cycle_id.to_string()
    }

    fn occurred_at(&self) -> Timestamp {
        self.archived_at
    }

    fn event_id(&self) -> EventId {
        self.event_id.clone()
    }
}
```

---

## Acceptance Criteria

### AC1: CreateCycle Publishes Event

**Given** a user creates a new cycle in a session
**When** the cycle is successfully persisted
**Then** a `CycleCreated` event is published with session_id and cycle_id

### AC2: BranchCycle Publishes Event with Inheritance Info

**Given** a user branches from an existing cycle at Objectives
**When** the branch is created
**Then** a `CycleBranched` event includes:
- Parent cycle ID
- Branch point (Objectives)
- List of inherited components (IssueRaising, ProblemFrame, Objectives)

### AC3: StartComponent Publishes Event

**Given** a cycle with NotStarted component
**When** the component is started
**Then** a `ComponentStarted` event is published with component type

### AC4: CompleteComponent Publishes Event with Progress

**Given** a component in InProgress status
**When** the component is completed
**Then** a `ComponentCompleted` event includes current cycle progress snapshot

### AC5: UpdateComponentOutput Publishes Summary

**Given** a started component
**When** structured output is updated
**Then** a `ComponentOutputUpdated` event includes a brief change summary (not full data)

### AC6: Analysis Triggers on Specific Components

**Given** Consequences component is completed
**When** `ComponentCompleted` event is published
**Then** `AnalysisTriggerHandler` calculates and publishes Pugh scores

**Given** DecisionQuality component is completed
**When** `ComponentCompleted` event is published
**Then** `AnalysisTriggerHandler` calculates and publishes DQ scores

### AC7: Events Include Session ID for Routing

**Given** any cycle/component event is published
**When** the event envelope is created
**Then** it includes `session_id` in payload for WebSocket room routing

---

## Technical Design

### Command Handler Changes

```rust
// backend/src/application/commands/create_cycle.rs

pub struct CreateCycleHandler {
    cycle_repo: Arc<dyn CycleRepository>,
    session_repo: Arc<dyn SessionRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl CreateCycleHandler {
    pub async fn handle(
        &self,
        cmd: CreateCycleCommand,
        metadata: CommandMetadata,
    ) -> Result<CycleId, DomainError> {
        // Verify session exists and user has access
        let session = self.session_repo
            .find_by_id(cmd.session_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::SessionNotFound, "Session not found"))?;

        if !session.is_owner(&cmd.user_id) {
            return Err(DomainError::new(ErrorCode::SessionUnauthorized, "Not authorized"));
        }

        // Create cycle
        let cycle = Cycle::new(cmd.session_id)?;

        // Persist
        self.cycle_repo.save(&cycle).await?;

        // Publish event
        let event = CycleCreated {
            event_id: EventId::new(),
            cycle_id: cycle.id(),
            session_id: cmd.session_id,
            parent_cycle_id: None,
            created_at: Timestamp::now(),
        };

        let envelope = EventEnvelope::from_event(&event, "Cycle")
            .with_correlation_id(metadata.correlation_id);

        self.event_publisher.publish(envelope).await?;

        Ok(cycle.id())
    }
}
```

```rust
// backend/src/application/commands/branch_cycle.rs

pub struct BranchCycleHandler {
    cycle_repo: Arc<dyn CycleRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl BranchCycleHandler {
    pub async fn handle(
        &self,
        cmd: BranchCycleCommand,
        metadata: CommandMetadata,
    ) -> Result<CycleId, DomainError> {
        // Load parent cycle
        let parent = self.cycle_repo
            .find_by_id(cmd.parent_cycle_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::CycleNotFound, "Parent cycle not found"))?;

        // Validate branch point
        if !parent.can_branch_at(cmd.branch_point) {
            return Err(DomainError::new(
                ErrorCode::InvalidNavigation,
                "Cannot branch at this component",
            ));
        }

        // Create branch
        let branch = Cycle::branch_at(&parent, cmd.branch_point)?;

        // Determine inherited components
        let inherited = ComponentType::all()
            .iter()
            .take_while(|&&ct| ct <= cmd.branch_point)
            .cloned()
            .collect::<Vec<_>>();

        // Persist
        self.cycle_repo.save(&branch).await?;

        // Publish event
        let event = CycleBranched {
            event_id: EventId::new(),
            cycle_id: branch.id(),
            parent_cycle_id: cmd.parent_cycle_id,
            session_id: parent.session_id(),
            branch_point: cmd.branch_point,
            inherited_components: inherited,
            branched_at: Timestamp::now(),
        };

        let envelope = EventEnvelope::from_event(&event, "Cycle")
            .with_correlation_id(metadata.correlation_id);

        self.event_publisher.publish(envelope).await?;

        Ok(branch.id())
    }
}
```

```rust
// backend/src/application/commands/start_component.rs

pub struct StartComponentHandler {
    cycle_repo: Arc<dyn CycleRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl StartComponentHandler {
    pub async fn handle(
        &self,
        cmd: StartComponentCommand,
        metadata: CommandMetadata,
    ) -> Result<ComponentId, DomainError> {
        // Load cycle
        let mut cycle = self.cycle_repo
            .find_by_id(cmd.cycle_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::CycleNotFound, "Cycle not found"))?;

        // Start component
        let component_id = cycle.start_component(cmd.component_type)?;

        // Persist
        self.cycle_repo.update(&cycle).await?;

        // Publish event
        let event = ComponentStarted {
            event_id: EventId::new(),
            cycle_id: cmd.cycle_id,
            session_id: cycle.session_id(),
            component_id,
            component_type: cmd.component_type,
            started_at: Timestamp::now(),
        };

        let envelope = EventEnvelope::from_event(&event, "Cycle")
            .with_correlation_id(metadata.correlation_id);

        self.event_publisher.publish(envelope).await?;

        Ok(component_id)
    }
}
```

```rust
// backend/src/application/commands/complete_component.rs

pub struct CompleteComponentHandler {
    cycle_repo: Arc<dyn CycleRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl CompleteComponentHandler {
    pub async fn handle(
        &self,
        cmd: CompleteComponentCommand,
        metadata: CommandMetadata,
    ) -> Result<(), DomainError> {
        // Load cycle
        let mut cycle = self.cycle_repo
            .find_by_id(cmd.cycle_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::CycleNotFound, "Cycle not found"))?;

        // Get component ID before completion
        let component = cycle.get_component(cmd.component_type)?;
        let component_id = component.id();

        // Complete component
        cycle.complete_component(cmd.component_type)?;

        // Get progress after completion
        let progress = cycle.get_progress();

        // Persist
        self.cycle_repo.update(&cycle).await?;

        // Publish event
        let event = ComponentCompleted {
            event_id: EventId::new(),
            cycle_id: cmd.cycle_id,
            session_id: cycle.session_id(),
            component_id,
            component_type: cmd.component_type,
            completed_at: Timestamp::now(),
            progress: CycleProgressSnapshot {
                completed_count: progress.completed_steps,
                total_count: progress.total_steps,
                percent_complete: progress.percent_complete(),
                current_step: progress.current_step,
            },
        };

        let envelope = EventEnvelope::from_event(&event, "Cycle")
            .with_correlation_id(metadata.correlation_id);

        self.event_publisher.publish(envelope).await?;

        // Check if cycle is now complete
        if progress.completed_steps == progress.total_steps {
            let dq_score = cycle.get_component(ComponentType::DecisionQuality)
                .ok()
                .and_then(|c| c.output_as_value().get("overall_score").cloned())
                .and_then(|v| v.as_i64())
                .map(|v| Percentage::new(v as i32).unwrap_or_default());

            let complete_event = CycleCompleted {
                event_id: EventId::new(),
                cycle_id: cmd.cycle_id,
                session_id: cycle.session_id(),
                dq_overall_score: dq_score,
                completed_at: Timestamp::now(),
            };

            let envelope = EventEnvelope::from_event(&complete_event, "Cycle")
                .with_correlation_id(metadata.correlation_id.clone())
                .with_causation_id(event.event_id.as_str());

            self.event_publisher.publish(envelope).await?;
        }

        Ok(())
    }
}
```

```rust
// backend/src/application/commands/update_component_output.rs

pub struct UpdateComponentOutputHandler {
    cycle_repo: Arc<dyn CycleRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl UpdateComponentOutputHandler {
    pub async fn handle(
        &self,
        cmd: UpdateComponentOutputCommand,
        metadata: CommandMetadata,
    ) -> Result<(), DomainError> {
        // Load cycle
        let mut cycle = self.cycle_repo
            .find_by_id(cmd.cycle_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::CycleNotFound, "Cycle not found"))?;

        // Get component for ID
        let component = cycle.get_component(cmd.component_type)?;
        let component_id = component.id();

        // Update output
        cycle.update_component_output(cmd.component_type, cmd.output.clone())?;

        // Create change summary (first 100 chars of stringified output)
        let change_summary = serde_json::to_string(&cmd.output)
            .map(|s| s.chars().take(100).collect::<String>())
            .unwrap_or_else(|_| "output updated".to_string());

        // Persist
        self.cycle_repo.update(&cycle).await?;

        // Publish event
        let event = ComponentOutputUpdated {
            event_id: EventId::new(),
            cycle_id: cmd.cycle_id,
            session_id: cycle.session_id(),
            component_id,
            component_type: cmd.component_type,
            change_summary,
            updated_at: Timestamp::now(),
        };

        let envelope = EventEnvelope::from_event(&event, "Cycle")
            .with_correlation_id(metadata.correlation_id);

        self.event_publisher.publish(envelope).await?;

        Ok(())
    }
}
```

---

## File Structure

```
backend/src/domain/cycle/
├── mod.rs                    # Add events export
├── cycle.rs                  # Existing aggregate
├── progress.rs               # Existing progress value object
├── events.rs                 # NEW: All cycle domain events
└── events_test.rs            # NEW: Event unit tests

backend/src/application/commands/
├── create_cycle.rs           # MODIFY: Add EventPublisher
├── create_cycle_test.rs      # MODIFY: Test event publishing
├── branch_cycle.rs           # MODIFY: Add EventPublisher
├── branch_cycle_test.rs      # MODIFY: Test event publishing
├── start_component.rs        # MODIFY: Add EventPublisher
├── start_component_test.rs   # MODIFY: Test event publishing
├── complete_component.rs     # MODIFY: Add EventPublisher
├── complete_component_test.rs # MODIFY: Test event publishing
├── update_component_output.rs # MODIFY: Add EventPublisher
└── update_component_output_test.rs # MODIFY: Test event publishing
```

---

## Test Specifications

### Unit Tests: Event Types

```rust
#[test]
fn cycle_created_event_type() {
    let event = CycleCreated {
        event_id: EventId::new(),
        cycle_id: CycleId::new(),
        session_id: SessionId::new(),
        parent_cycle_id: None,
        created_at: Timestamp::now(),
    };

    assert_eq!(event.event_type(), "cycle.created");
}

#[test]
fn cycle_branched_includes_inheritance() {
    let event = CycleBranched {
        event_id: EventId::new(),
        cycle_id: CycleId::new(),
        parent_cycle_id: CycleId::new(),
        session_id: SessionId::new(),
        branch_point: ComponentType::Objectives,
        inherited_components: vec![
            ComponentType::IssueRaising,
            ComponentType::ProblemFrame,
            ComponentType::Objectives,
        ],
        branched_at: Timestamp::now(),
    };

    assert_eq!(event.inherited_components.len(), 3);
    assert_eq!(event.branch_point, ComponentType::Objectives);
}

#[test]
fn component_completed_includes_progress() {
    let event = ComponentCompleted {
        event_id: EventId::new(),
        cycle_id: CycleId::new(),
        session_id: SessionId::new(),
        component_id: ComponentId::new(),
        component_type: ComponentType::Objectives,
        completed_at: Timestamp::now(),
        progress: CycleProgressSnapshot {
            completed_count: 3,
            total_count: 9,
            percent_complete: 33,
            current_step: ComponentType::Alternatives,
        },
    };

    assert_eq!(event.progress.completed_count, 3);
    assert_eq!(event.progress.percent_complete, 33);
}
```

### Unit Tests: Command Handlers

```rust
#[tokio::test]
async fn create_cycle_publishes_event() {
    let cycle_repo = Arc::new(InMemoryCycleRepository::new());
    let session_repo = Arc::new(InMemorySessionRepository::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    // Create session first
    let session = Session::new(UserId::new("user-1"), "Test".to_string()).unwrap();
    let session_id = session.id();
    session_repo.save(&session).await.unwrap();

    let handler = CreateCycleHandler::new(cycle_repo, session_repo, event_bus.clone());

    let cmd = CreateCycleCommand {
        session_id,
        user_id: UserId::new("user-1"),
    };

    // Act
    let cycle_id = handler.handle(cmd, CommandMetadata::default()).await.unwrap();

    // Assert
    let events = event_bus.events_of_type("cycle.created");
    assert_eq!(events.len(), 1);

    let payload: CycleCreated = events[0].payload_as().unwrap();
    assert_eq!(payload.cycle_id, cycle_id);
    assert_eq!(payload.session_id, session_id);
    assert!(payload.parent_cycle_id.is_none());
}

#[tokio::test]
async fn branch_cycle_publishes_event_with_inheritance() {
    let cycle_repo = Arc::new(InMemoryCycleRepository::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    // Create parent cycle with completed components
    let session_id = SessionId::new();
    let mut parent = Cycle::new(session_id).unwrap();
    parent.start_component(ComponentType::IssueRaising).unwrap();
    parent.complete_component(ComponentType::IssueRaising).unwrap();
    parent.start_component(ComponentType::ProblemFrame).unwrap();
    parent.complete_component(ComponentType::ProblemFrame).unwrap();
    parent.start_component(ComponentType::Objectives).unwrap();
    parent.complete_component(ComponentType::Objectives).unwrap();
    let parent_id = parent.id();
    cycle_repo.save(&parent).await.unwrap();

    let handler = BranchCycleHandler::new(cycle_repo, event_bus.clone());

    let cmd = BranchCycleCommand {
        parent_cycle_id: parent_id,
        branch_point: ComponentType::Objectives,
    };

    // Act
    let branch_id = handler.handle(cmd, CommandMetadata::default()).await.unwrap();

    // Assert
    let events = event_bus.events_of_type("cycle.branched");
    assert_eq!(events.len(), 1);

    let payload: CycleBranched = events[0].payload_as().unwrap();
    assert_eq!(payload.cycle_id, branch_id);
    assert_eq!(payload.parent_cycle_id, parent_id);
    assert_eq!(payload.branch_point, ComponentType::Objectives);
    assert_eq!(payload.inherited_components.len(), 3);
}

#[tokio::test]
async fn start_component_publishes_event() {
    let cycle_repo = Arc::new(InMemoryCycleRepository::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    let cycle = Cycle::new(SessionId::new()).unwrap();
    let cycle_id = cycle.id();
    cycle_repo.save(&cycle).await.unwrap();

    let handler = StartComponentHandler::new(cycle_repo, event_bus.clone());

    let cmd = StartComponentCommand {
        cycle_id,
        component_type: ComponentType::IssueRaising,
    };

    // Act
    handler.handle(cmd, CommandMetadata::default()).await.unwrap();

    // Assert
    let events = event_bus.events_of_type("component.started");
    assert_eq!(events.len(), 1);

    let payload: ComponentStarted = events[0].payload_as().unwrap();
    assert_eq!(payload.cycle_id, cycle_id);
    assert_eq!(payload.component_type, ComponentType::IssueRaising);
}

#[tokio::test]
async fn complete_component_publishes_event_with_progress() {
    let cycle_repo = Arc::new(InMemoryCycleRepository::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    let mut cycle = Cycle::new(SessionId::new()).unwrap();
    cycle.start_component(ComponentType::IssueRaising).unwrap();
    let cycle_id = cycle.id();
    cycle_repo.save(&cycle).await.unwrap();

    let handler = CompleteComponentHandler::new(cycle_repo, event_bus.clone());

    let cmd = CompleteComponentCommand {
        cycle_id,
        component_type: ComponentType::IssueRaising,
    };

    // Act
    handler.handle(cmd, CommandMetadata::default()).await.unwrap();

    // Assert
    let events = event_bus.events_of_type("component.completed");
    assert_eq!(events.len(), 1);

    let payload: ComponentCompleted = events[0].payload_as().unwrap();
    assert_eq!(payload.component_type, ComponentType::IssueRaising);
    assert_eq!(payload.progress.completed_count, 1);
    assert_eq!(payload.progress.total_count, 9);
}

#[tokio::test]
async fn completing_last_component_publishes_cycle_completed() {
    let cycle_repo = Arc::new(InMemoryCycleRepository::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    // Create cycle with 8 completed components
    let mut cycle = Cycle::new(SessionId::new()).unwrap();
    for comp_type in ComponentType::all().iter().take(8) {
        cycle.start_component(*comp_type).unwrap();
        cycle.complete_component(*comp_type).unwrap();
    }
    cycle.start_component(ComponentType::NotesNextSteps).unwrap();
    let cycle_id = cycle.id();
    cycle_repo.save(&cycle).await.unwrap();

    let handler = CompleteComponentHandler::new(cycle_repo, event_bus.clone());

    let cmd = CompleteComponentCommand {
        cycle_id,
        component_type: ComponentType::NotesNextSteps,
    };

    // Act
    handler.handle(cmd, CommandMetadata::default()).await.unwrap();

    // Assert - both events published
    assert!(event_bus.has_event("component.completed"));
    assert!(event_bus.has_event("cycle.completed"));

    let cycle_events = event_bus.events_of_type("cycle.completed");
    let payload: CycleCompleted = cycle_events[0].payload_as().unwrap();
    assert_eq!(payload.cycle_id, cycle_id);
}
```

---

## Dependencies

### Module Dependencies

- `foundation::events` - EventId, EventEnvelope, DomainEvent
- `foundation::ids` - CycleId, SessionId, ComponentId
- `foundation::component_type` - ComponentType enum
- `foundation::timestamp` - Timestamp
- `ports::event_publisher` - EventPublisher trait

---

## Related Documents

- **Integration Spec:** features/integrations/full-proact-journey.md
- **Phase 1:** features/foundation/event-infrastructure.md
- **Phase 2:** features/session/session-events.md
- **Checklist:** REQUIREMENTS/CHECKLIST-events.md (Phase 3)
- **Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md

---

*Version: 1.0.0*
*Created: 2026-01-07*
*Phase: 3 of 8*
