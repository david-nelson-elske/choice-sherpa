# [Project Name] - Hexagonal Architecture Specification

> **Generated**: [DATE]
> **Status**: Draft / In Review / Approved
> **Version**: 1.0.0

---

## Executive Summary

[2-3 sentence description of the system and its primary purpose]

---

## Module Inventory

### Bounded Contexts

| Module | Responsibility | Core Entity | External Dependencies |
|--------|---------------|-------------|----------------------|
| foundation | Shared value objects, utilities, error types | N/A | None |
| [module1] | [Primary responsibility] | [Aggregate name] | [External services] |
| [module2] | [Primary responsibility] | [Aggregate name] | [External services] |
| [module3] | [Primary responsibility] | [Aggregate name] | [External services] |

### Dependency Graph

```
foundation (Phase 1)
    │
    ├── [module1] (Phase 2) ─── [independent]
    ├── [module2] (Phase 2) ─── [independent]
    │
    ├── [module3] (Phase 3) ─── depends on: [module1, module2]
    │
    └── [module4] (Phase 4) ─── depends on: [module3]
```

### Build Order

| Phase | Modules | Rationale |
|-------|---------|-----------|
| 1 | foundation | Shared types required by all modules |
| 2 | [independent modules] | No inter-module dependencies |
| 3 | [dependent modules] | Requires Phase 2 modules |
| 4 | [integration modules] | Integrates multiple modules |

---

## Foundation Module

### Shared Value Objects

| Value Object | Description | Validation Rules |
|--------------|-------------|------------------|
| Money | Monetary amounts in cents | Non-negative, integer cents |
| Email | Email addresses | RFC 5322 compliant |
| [Custom VO] | [Description] | [Rules] |

### Shared Interfaces

```go
// Purchasable - implemented by all items that can be added to cart
type Purchasable interface {
    ID() string
    Name() string
    GetUnitPrice() Money
    HasServiceFee() bool
    IsGSTApplicable() bool
}
```

### Business Constants

| Constant | Value | Description |
|----------|-------|-------------|
| GSTRateBasisPoints | 500 | 5% GST |
| ServiceFeeRateBasisPoints | 250 | 2.5% service fee |
| [Custom constant] | [Value] | [Description] |

### Error Types

| Error | Description | HTTP Status |
|-------|-------------|-------------|
| ErrNotFound | Resource not found | 404 |
| ErrValidation | Validation failed | 400 |
| ErrConflict | Business rule conflict | 409 |
| [Custom error] | [Description] | [Status] |

---

## Module: [Module Name]

### Overview

[1-2 paragraph description of what this module does and why it exists]

### Domain Layer

#### Aggregate: [AggregateName]

**Invariants (Business Rules)**:
1. [Invariant 1 - e.g., "Capacity cannot be negative"]
2. [Invariant 2 - e.g., "Cannot register when event is full"]
3. [Invariant 3 - e.g., "Status transitions must follow allowed paths"]

**State Transitions**:
```
[Initial] ──▶ [State1] ──▶ [State2] ──▶ [Terminal]
                  │
                  └──▶ [AltTerminal]
```

**Structure**:
```go
type [AggregateName] struct {
    id             [ID Type]           // Value object for identity
    [field1]       [type]              // Description
    [field2]       [type]              // Description
    status         [StatusType]        // Enum with transitions
    [children]     []*[ChildEntity]    // Child entities
    domainEvents   []DomainEvent       // Collected events
}
```

**Business Methods**:
| Method | Description | Domain Events Emitted |
|--------|-------------|----------------------|
| `Create(...)` | Factory method with validation | `[Aggregate]Created` |
| `[Action](...)` | [Description] | `[Aggregate][Action]` |
| `Cancel(reason)` | Transition to cancelled state | `[Aggregate]Cancelled` |

#### Value Objects

| Name | Type | Validation | Operations |
|------|------|------------|------------|
| [ID Type] | string | UUID with prefix | Parse, String |
| [Status] | enum | Valid transitions | CanTransitionTo |
| [Custom VO] | [type] | [rules] | [methods] |

#### Domain Events

| Event | Trigger | Payload |
|-------|---------|---------|
| `[Aggregate]Created` | New aggregate created | ID, timestamp |
| `[Aggregate][Action]` | [Action] performed | ID, [relevant fields] |
| `[Aggregate]Cancelled` | Aggregate cancelled | ID, reason, timestamp |

### Ports (Interfaces)

#### [Module]Repository (Write Operations)

```go
type [Module]Repository interface {
    Save(ctx context.Context, aggregate *[Aggregate]) error
    Update(ctx context.Context, aggregate *[Aggregate]) error
    Delete(ctx context.Context, id [IDType]) error
    FindByID(ctx context.Context, id [IDType]) (*[Aggregate], error)
}
```

#### [Module]Reader (Read Operations)

```go
type [Module]Reader interface {
    GetByID(ctx context.Context, id [IDType]) (*[Aggregate], error)
    List(ctx context.Context, filter [Filter]) (*[ListResult], error)
    Count(ctx context.Context, filter [Filter]) (int, error)
}
```

### Application Layer

#### Commands

| Command | Handler | Description |
|---------|---------|-------------|
| `Create[Aggregate]Command` | `Create[Aggregate]Handler` | Creates new aggregate |
| `[Action][Aggregate]Command` | `[Action][Aggregate]Handler` | Performs [action] |
| `Cancel[Aggregate]Command` | `Cancel[Aggregate]Handler` | Cancels aggregate |

#### Queries

| Query | Handler | Description |
|-------|---------|-------------|
| `Get[Aggregate]Query` | `Get[Aggregate]Handler` | Retrieves by ID |
| `List[Aggregates]Query` | `List[Aggregates]Handler` | Lists with filters |

### Adapter Layer

#### HTTP Endpoints

| Method | Path | Handler | Auth |
|--------|------|---------|------|
| POST | `/api/[module]` | Create[Aggregate] | Required |
| GET | `/api/[module]` | List[Aggregates] | Optional |
| GET | `/api/[module]/{id}` | Get[Aggregate] | Optional |
| POST | `/api/[module]/{id}/[action]` | [Action][Aggregate] | Required |
| DELETE | `/api/[module]/{id}` | Cancel[Aggregate] | Required |

#### Database Tables

| Table | Columns | Indexes |
|-------|---------|---------|
| `[module_plural]` | id, [fields], status, created_at, updated_at | idx_[module]_status |
| `[child_plural]` | id, [parent]_id, [fields] | idx_[child]_[parent]_id |

#### External Service Adapters

| Service | Adapter | Purpose |
|---------|---------|---------|
| [Service Name] | `[Service]Adapter` | [Purpose] |

### Frontend Module

#### Domain Types

```typescript
// [module]/domain/[types].ts
export interface [Aggregate] {
    id: string;
    [field1]: [type];
    [field2]: [type];
    status: [Status]Type;
}

export const [Status] = {
    [VALUE1]: '[value1]',
    [VALUE2]: '[value2]',
} as const;
```

#### API Hooks

| Hook | Purpose | Parameters |
|------|---------|------------|
| `use[Aggregates]()` | List aggregates | filter, pagination |
| `use[Aggregate](id)` | Get single aggregate | id |
| `useCreate[Aggregate]()` | Create mutation | form data |
| `use[Action][Aggregate]()` | [Action] mutation | id, data |

#### Components

| Component | Purpose | Props |
|-----------|---------|-------|
| `[Aggregate]Card` | Display in list | aggregate |
| `[Aggregate]Form` | Create/edit form | onSubmit, initial? |
| `[Aggregate]Detail` | Full detail view | id |
| `[Aggregate]List` | Paginated list | filter? |

---

## Cross-Cutting Concerns

### Authentication & Authorization

| Operation | Required Role | Notes |
|-----------|--------------|-------|
| Read (public) | None | Public data |
| Read (private) | Authenticated | User's own data |
| Write | Authenticated | [Specific rules] |
| Admin | [Role] | Administrative operations |

### Validation Strategy

| Layer | Validation Type | Examples |
|-------|----------------|----------|
| Domain | Invariants | Business rules, state transitions |
| Application | Command validation | Required fields, formats |
| HTTP | Request validation | JSON schema, auth |
| Frontend | Form validation | Client-side UX |

### Error Handling

| Domain Error | HTTP Status | Error Code |
|--------------|-------------|------------|
| `Err[X]NotFound` | 404 | `[X]_NOT_FOUND` |
| `Err[X]Full` | 409 | `[X]_FULL` |
| `ErrInvalid[Y]` | 400 | `INVALID_[Y]` |
| `ErrNotAuthorized` | 403 | `NOT_AUTHORIZED` |

### Event Publishing

| Source Module | Event | Subscribers |
|---------------|-------|-------------|
| [module1] | [Event1] | [module2], [module3] |
| [module2] | [Event2] | [module1] |

---

## File Structure

### Backend

```
backend/
├── cmd/server/
│   └── main.go
├── internal/
│   ├── domain/
│   │   ├── shared/              # Foundation module
│   │   │   ├── money.go
│   │   │   └── [value_objects].go
│   │   ├── errors/              # Shared error types
│   │   │   └── domain_errors.go
│   │   ├── [module1]/           # Module 1 domain
│   │   │   ├── [aggregate].go
│   │   │   ├── [aggregate]_test.go
│   │   │   ├── [value_objects].go
│   │   │   ├── status.go
│   │   │   ├── errors.go
│   │   │   └── domain_events.go
│   │   └── [module2]/           # Module 2 domain
│   ├── ports/                   # All port interfaces
│   │   ├── [module1]_repository.go
│   │   ├── [module1]_reader.go
│   │   └── [module2]_repository.go
│   ├── application/
│   │   ├── commands/
│   │   │   ├── [module1]_commands.go
│   │   │   └── [module2]_commands.go
│   │   └── queries/
│   │       ├── [module1]_queries.go
│   │       └── [module2]_queries.go
│   └── adapters/
│       ├── http/
│       │   ├── [module1]/
│       │   │   ├── handlers.go
│       │   │   ├── dto.go
│       │   │   └── routes.go
│       │   └── [module2]/
│       ├── postgres/
│       │   ├── [module1]_repository.go
│       │   ├── [module1]_reader.go
│       │   ├── mappers/
│       │   │   └── [module1]_mapper.go
│       │   └── sqlc/
│       │       ├── queries/
│       │       │   └── [module1].sql
│       │       └── generated/
│       └── [external]/
│           └── [service]_adapter.go
└── migrations/
    └── [timestamp]_[description].sql
```

### Frontend

```
frontend/
├── src/
│   ├── app/                     # Next.js App Router
│   │   ├── (public)/
│   │   │   └── [module1]/       # Public pages
│   │   ├── (portal)/
│   │   │   └── [module1]/       # Authenticated pages
│   │   └── api/                 # API routes (if needed)
│   ├── modules/
│   │   ├── [module1]/
│   │   │   ├── domain/
│   │   │   │   ├── [types].ts
│   │   │   │   └── [types].test.ts
│   │   │   ├── api/
│   │   │   │   ├── [module]-api.ts
│   │   │   │   ├── use-[module].ts
│   │   │   │   └── *.test.ts
│   │   │   ├── components/
│   │   │   │   ├── [Component].tsx
│   │   │   │   └── [Component].test.tsx
│   │   │   └── index.ts
│   │   └── [module2]/
│   └── shared/
│       ├── domain/
│       │   ├── money.ts
│       │   └── money.test.ts
│       ├── components/
│       │   └── ui/              # Shared UI components
│       └── hooks/
└── tests/
    └── e2e/                     # Playwright tests
```

---

## Implementation Phases

### Phase 1: Foundation

**Duration**: [estimate]

**Deliverables**:
- [ ] Shared value objects (Money, Email, etc.)
- [ ] Domain error types
- [ ] Shared interfaces (Purchasable if applicable)
- [ ] Database migrations for shared types
- [ ] Frontend shared types

**Exit Criteria**:
- All shared types have 100% test coverage
- No lint errors
- Frontend types mirror backend

### Phase 2: [Independent Modules]

**Duration**: [estimate]

**Deliverables**:
- [ ] [Module1] domain layer
- [ ] [Module1] ports and adapters
- [ ] [Module1] frontend module
- [ ] [Module2] domain layer
- [ ] [Module2] ports and adapters
- [ ] [Module2] frontend module

**Exit Criteria**:
- Domain coverage >= 95%
- All API endpoints functional
- Frontend integration complete

### Phase 3: [Dependent Modules]

**Duration**: [estimate]

**Deliverables**:
- [ ] [Module3] with integration to [Module1, Module2]
- [ ] Cross-module event handlers

**Exit Criteria**:
- Integration tests passing
- Event flow verified

### Phase 4: [Integration Modules]

**Duration**: [estimate]

**Deliverables**:
- [ ] [Module4] integrating all modules
- [ ] E2E user journeys

**Exit Criteria**:
- Full E2E flow working
- Performance acceptable

---

## Appendix

### A: API Contract Summary

[OpenAPI specification reference or inline summary]

### B: Database Schema

[ERD or schema reference]

### C: Event Catalog

[Complete list of domain events with schemas]

### D: Decision Log

| Decision | Date | Rationale |
|----------|------|-----------|
| [Decision 1] | [Date] | [Why] |
| [Decision 2] | [Date] | [Why] |

---

*Document generated by `/hexagonal-design` skill*
