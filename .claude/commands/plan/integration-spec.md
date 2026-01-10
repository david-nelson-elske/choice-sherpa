# Integration Specification

Define cross-module features that span multiple bounded contexts.

ultrathink: Analyze data flows between modules, identify failure modes, design compensation strategies, and ensure the integration respects bounded context boundaries.

## Usage

```
/integration-spec <name> [description]
/integration-spec guest-checkout "Purchase without account"
```

---

## When to Use (vs /feature-brief)

| Indicator | Threshold |
|-----------|-----------|
| Modules modified | 3+ |
| Data flows between modules | Yes |
| Complex rollback needed | Yes |
| New shared interfaces | Yes |
| Spans multiple aggregates | Yes |

---

## Output Location

`features/integrations/<name>.md`

---

## Output Structure

```markdown
# Integration: [Title]

**Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md
**Type:** [User Journey | System Process | Data Flow]
**Priority:** [P0-P3]

> [One-line description]

---

## Overview
[Purpose, why it spans modules, user/system need]

---

## Modules Involved
| Module | Role | Changes |
|--------|------|---------|
| [Module] | [Producer/Consumer/Both] | [Description] |

---

## Data Flow
```
[Module A] ──(data)──> [Module B] ──(data)──> [Module C]
```

### Flow Steps
1. **[Step]** ([Module])
   - Input: [data]
   - Action: [what happens]
   - Output: [data]
   - Failure: [recovery]

---

## Coordination Points

### Synchronous Calls
| From | To | Method | Purpose |
|------|----|--------|---------|

### Asynchronous Events
| Event | Publisher | Subscribers | Purpose |
|-------|-----------|-------------|---------|

### Shared State
| Data | Owner | Readers | Consistency |
|------|-------|---------|-------------|

---

## Failure Modes
| Failure | Impact | Detection | Recovery |
|---------|--------|-----------|----------|

### Compensation
If [Module B] fails after [Module A] succeeds:
1. [Compensation step]
2. [Final state]

---

## Shared Types
[New interfaces needed]

---

## API Contracts

### Endpoints Created
| Method | Path | Module | Purpose |
|--------|------|--------|---------|

### Events Published
| Event | Module | Payload | When |
|-------|--------|---------|------|

---

## Implementation Phases

### Phase 1: [Foundation]
**Goal:** [Outcome]
**Modules:** [Which]
**Deliverables:**
- [ ] [Task]
**Exit Criteria:** [Verification]

---

## Testing Strategy

### Unit Tests (Per Module)
| Module | Focus |
|--------|-------|

### Integration Tests
| Test | Modules | Scenario |
|------|---------|----------|

### E2E Tests
| Journey | Steps | Verification |
|---------|-------|--------------|

---

## Rollout Plan

### Feature Flags
| Flag | Purpose | Default |
|------|---------|---------|

### Migration Steps
1. [Step with verification]
```

---

## Module Roles

| Role | Description |
|------|-------------|
| Producer | Creates/owns primary data |
| Consumer | Reads from other modules |
| Both | Reads and writes |

---

## Failure Categories

| Category | Examples | Recovery |
|----------|----------|----------|
| Network | Timeout, refused | Retry |
| Business | Insufficient funds | Compensate |
| System | DB down | Circuit breaker |
| Data | Stale, concurrent mod | Saga |

---

## Validation Checklist

- [ ] All modules exist in architecture
- [ ] Data flow complete (no dead ends)
- [ ] All failures have recovery
- [ ] Coordination points bidirectional
- [ ] Phases are shippable increments
- [ ] Tests cover integration points
- [ ] Rollout includes feature flags
