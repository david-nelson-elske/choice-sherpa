# Hexagonal Architecture Design

> **Purpose**: Transform a high-level project description into a symmetric, well-structured hexagonal architecture design.
> **Output**: Complete architectural specification ready for TDD implementation.
> **Thinking**: Extended (ultrathink enabled)

ultrathink: This skill requires deep architectural reasoning. Analyze the project thoroughly, consider multiple design approaches, evaluate trade-offs, and produce a well-structured hexagonal architecture.

---

## Usage

```
/hexagonal-design <project-description>
```

### Arguments
- `project-description`: Brief description of the system to design (can be provided inline or interactively)

### Examples
```
/hexagonal-design "E-commerce platform for digital products"
/hexagonal-design "Healthcare appointment scheduling system"
/hexagonal-design  # Interactive mode - will prompt for description
```

---

## What This Skill Produces

A complete hexagonal architecture design document containing:

1. **Module Inventory** - Bounded contexts and their responsibilities
2. **Dependency Graph** - Module relationships and build order
3. **Domain Layer Design** - Aggregates, value objects, domain events per module
4. **Ports Specification** - Interfaces for all external dependencies
5. **Application Layer Design** - Command and query handlers
6. **Adapter Mapping** - Implementation strategy for each port
7. **Frontend Module Alignment** - UI modules mirroring backend structure
8. **Cross-Cutting Concerns** - Shared types, error handling, validation

---

## Phase 1: Domain Discovery

### Step 1.1: Identify Core Concepts

Ask these questions about the project:
1. What are the main **nouns** (entities) in the system?
2. What **actions** (verbs) can users perform?
3. What **events** occur that other parts of the system care about?
4. What **rules** must always be true (invariants)?
5. What **external systems** does this integrate with?

### Step 1.2: Group into Bounded Contexts

A bounded context (module) should:
- Have a single, clear responsibility
- Own its own data (no sharing databases)
- Communicate with other contexts via events or explicit interfaces
- Be deployable independently (eventually)

### Step 1.3: Identify Module Dependencies

Map which modules depend on which others:
```
Foundation ← (all modules depend on foundation)
    │
    ├── Module A (independent)
    ├── Module B (independent)
    ├── Module C (depends on A, B)
    │
    └── Module D (depends on C)
```

**Rule**: No circular dependencies. Build order must be deterministic.

---

## Phase 2: Module Template

For each module, define these components:

### 2.1 Domain Layer

```
backend/internal/domain/<module>/
├── <aggregate>.go          # Root entity with business logic
├── <aggregate>_test.go     # Aggregate tests
├── <entity>.go             # Child entities
├── <value_object>.go       # Immutable domain primitives
├── <value_object>_test.go  # Value object tests
├── status.go               # Enums with transitions
├── errors.go               # Domain-specific errors
├── domain_events.go        # Events emitted by aggregates
└── builder.go              # Builder for aggregate (optional)
```

### 2.2 Ports Layer

```
backend/internal/ports/
├── <module>_repository.go  # Write operations interface
├── <module>_reader.go      # Read operations interface (CQRS)
└── <module>_publisher.go   # Event publishing interface (if needed)
```

### 2.3 Application Layer

```
backend/internal/application/
├── commands/
│   ├── create_<aggregate>.go
│   ├── update_<aggregate>.go
│   ├── <action>_<aggregate>.go
│   └── *_test.go
└── queries/
    ├── get_<aggregate>.go
    ├── list_<aggregates>.go
    └── *_test.go
```

### 2.4 Adapter Layer

```
backend/internal/adapters/
├── http/<module>/
│   ├── handlers.go         # HTTP handlers
│   ├── handlers_test.go    # Handler tests
│   ├── dto.go              # Request/response DTOs
│   └── routes.go           # Route registration
├── postgres/
│   ├── <module>_repository.go
│   ├── <module>_reader.go
│   ├── sqlc/queries/<module>.sql
│   └── mappers/<module>_mapper.go
└── <external>/             # External service adapters
    └── <service>_adapter.go
```

### 2.5 Frontend Module

```
frontend/src/modules/<module>/
├── domain/
│   ├── <types>.ts          # Domain types mirroring backend
│   └── <types>.test.ts
├── api/
│   ├── <module>-api.ts     # API client
│   ├── use-<module>.ts     # React hooks
│   └── *.test.ts
├── components/
│   ├── <Component>.tsx
│   └── <Component>.test.tsx
└── index.ts                # Public exports
```

---

## Phase 3: Symmetry Checklist

### 3.1 Vertical Symmetry (Per Module)

Each module should have symmetric layers:

| Layer | Backend | Frontend |
|-------|---------|----------|
| Domain | `domain/<module>/` | `modules/<module>/domain/` |
| Ports | `ports/<module>_*.go` | (N/A - uses API) |
| Application | `application/commands/` | (N/A - backend only) |
| Adapters | `adapters/http/<module>/` | `modules/<module>/api/` |
| Presentation | (N/A) | `modules/<module>/components/` |

### 3.2 Horizontal Symmetry (Across Modules)

All modules should follow identical patterns:

| Pattern | Applies To |
|---------|------------|
| Aggregate + Builder | All aggregates |
| Repository + Reader ports | All modules with persistence |
| Command + Query handlers | All modules with business operations |
| HTTP handlers + DTOs | All modules with API |
| Domain types + API hooks | All frontend modules |

### 3.3 Naming Symmetry

Consistent naming across the system:

| Backend | Frontend | Database |
|---------|----------|----------|
| `Event` | `Event` | `events` |
| `EventID` | `eventId: string` | `id (UUID)` |
| `EventStatus` | `EventStatus` | `status` |
| `CreateEvent` | `createEvent()` | `INSERT INTO events` |
| `ErrEventNotFound` | `EventNotFoundError` | (N/A) |

---

## Phase 4: Design Patterns Reference

### 4.1 Aggregate Pattern

```go
// Aggregate root with invariant protection
type Event struct {
    id            EventID           // Value object ID
    title         string            // Required field
    status        EventStatus       // Enum with transitions
    registrations []*Registration   // Child entities
    domainEvents  []DomainEvent     // Collected events
}

// Business method enforces invariants
func (e *Event) Register(userID string) (*Registration, error) {
    if e.status != StatusPublished { return nil, ErrNotPublished }
    if e.IsFull() { return nil, ErrEventFull }

    reg := NewRegistration(e.id, userID)
    e.registrations = append(e.registrations, reg)
    e.addDomainEvent(RegistrationCreated{...})
    return reg, nil
}
```

### 4.2 Value Object Pattern

```go
// Immutable, self-validating value object
type Money struct {
    cents int64  // Private, immutable
}

func NewMoney(cents int64) (Money, error) {
    if cents < 0 { return Money{}, ErrNegativeMoney }
    return Money{cents: cents}, nil
}

func (m Money) Add(other Money) Money {
    return Money{cents: m.cents + other.cents}  // Returns NEW instance
}
```

### 4.3 Port Interface Pattern

```go
// Repository port (write operations)
type EventRepository interface {
    Save(ctx context.Context, event *Event) error
    Update(ctx context.Context, event *Event) error
    Delete(ctx context.Context, id EventID) error
    FindByID(ctx context.Context, id EventID) (*Event, error)
}

// Reader port (CQRS - read operations)
type EventReader interface {
    GetByID(ctx context.Context, id EventID) (*Event, error)
    List(ctx context.Context, filter EventFilter) ([]*Event, error)
}
```

### 4.4 Command Handler Pattern

```go
type RegisterForEventCommand struct {
    EventID  string
    UserID   string
    IsMember bool
}

type RegisterForEventHandler struct {
    repo      ports.EventRepository
    publisher ports.DomainEventPublisher
}

func (h *RegisterForEventHandler) Handle(ctx context.Context, cmd RegisterForEventCommand) (*Result, error) {
    // 1. Load aggregate
    event, err := h.repo.FindByID(ctx, cmd.EventID)
    if err != nil { return nil, err }

    // 2. Execute domain logic
    registration, err := event.Register(cmd.UserID, cmd.IsMember)
    if err != nil { return nil, err }

    // 3. Persist changes
    err = h.repo.Update(ctx, event)
    if err != nil { return nil, err }

    // 4. Publish domain events
    h.publisher.Publish(ctx, event.PullDomainEvents()...)

    return &Result{ID: registration.ID()}, nil
}
```

### 4.5 HTTP Handler Pattern

```go
type Handler struct {
    registerForEvent *commands.RegisterForEventHandler
    listEvents       *queries.ListEventsHandler
}

func (h *Handler) RegisterForEvent(w http.ResponseWriter, r *http.Request) {
    var req RegisterRequest
    if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
        respondError(w, http.StatusBadRequest, "INVALID_JSON", err.Error())
        return
    }

    result, err := h.registerForEvent.Handle(r.Context(), req.ToCommand())
    if err != nil {
        h.handleDomainError(w, err)  // Maps domain errors to HTTP status
        return
    }

    respondJSON(w, http.StatusCreated, FromResult(result))
}
```

### 4.6 Mapper Pattern

```go
// Isolates database types from domain types
type EventMapper struct{}

func (m *EventMapper) ToDomain(row sqlc.EventRow) (*events.Event, error) {
    builder := events.NewEventBuilder().
        WithID(events.EventID(row.ID.String())).
        WithTitle(row.Title)

    // Handle nullable fields
    if row.Description.Valid {
        builder.WithDescription(row.Description.String)
    }

    // Restore status via state transitions
    event, _ := builder.Build()
    m.restoreStatus(event, row.Status)

    return event, nil
}

func (m *EventMapper) ToCreateParams(e *events.Event) sqlc.CreateEventParams {
    return sqlc.CreateEventParams{
        ID:     uuid.MustParse(e.ID().String()),
        Title:  e.Title(),
        Status: e.Status().String(),
    }
}
```

---

## Phase 5: Shared Foundation

### 5.1 Shared Value Objects

Define once, use everywhere:

```
backend/internal/domain/shared/
├── money.go              # Money value object
├── money_test.go
├── email.go              # Email value object
├── email_test.go
├── address.go            # Address value object
├── purchasable.go        # Polymorphic interface
└── purchasable_test.go
```

```
frontend/src/shared/domain/
├── money.ts              # Mirror of backend Money
├── money.test.ts
└── purchasable.ts        # Mirror of backend interface
```

### 5.2 Shared Error Handling

```
backend/internal/domain/errors/
├── domain_errors.go      # Base domain error types
├── validation_errors.go  # Validation error types
└── not_found_errors.go   # Not found error types
```

### 5.3 Business Constants

Define business rules as constants:

```go
// backend/internal/domain/shared/constants.go
const (
    GSTRateBasisPoints        = 500   // 5%
    ServiceFeeRateBasisPoints = 250   // 2.5%
    MinBookingLeadTimeHours   = 24
    MaxAdvanceBookingDays     = 90
)
```

---

## Phase 6: Output Template

Generate this document structure:

```markdown
# [Project Name] Architecture

## Module Inventory

| Module | Responsibility | Dependencies |
|--------|---------------|--------------|
| foundation | Shared types, utilities | None |
| [module1] | [description] | foundation |
| ... | ... | ... |

## Build Order

1. Foundation (Phase 1)
2. [Independent modules] (Phase 2)
3. [Dependent modules] (Phase 3)
4. [Integration modules] (Phase 4)

## Module: [module-name]

### Domain Layer
- **Aggregate**: [Name] - [description]
- **Value Objects**: [list]
- **Domain Events**: [list]
- **Invariants**: [business rules]

### Ports
- `[Module]Repository` - [methods]
- `[Module]Reader` - [methods]

### Application Layer
- **Commands**: [list]
- **Queries**: [list]

### Adapters
- **HTTP**: [endpoints]
- **Database**: [tables]
- **External**: [services]

### Frontend
- **Domain Types**: [types]
- **API Hooks**: [hooks]
- **Components**: [components]

---

[Repeat for each module]
```

---

## Design Principles

### 1. Domain First
Always start with the domain layer. Understand the business problem before writing infrastructure code.

### 2. Dependency Inversion
High-level modules (domain, application) should not depend on low-level modules (adapters). Both should depend on abstractions (ports).

### 3. Single Responsibility
Each component has one reason to change:
- Aggregate: Business rules change
- Port: Interface contract changes
- Adapter: Infrastructure changes
- Handler: API contract changes

### 4. Explicit Dependencies
All dependencies are injected through constructors:
```go
func NewHandler(repo ports.Repository) *Handler {
    return &Handler{repo: repo}
}
```

### 5. Test Boundaries
- Domain: Unit tests (no mocks needed)
- Application: Unit tests (mock ports)
- Adapters: Integration tests (real infrastructure)
- E2E: User journey tests

---

## Anti-Patterns to Avoid

### 1. Anemic Domain Model
```go
// BAD: Logic outside aggregate
orderService.Cancel(order, reason)

// GOOD: Logic in aggregate
order.Cancel(reason)
```

### 2. Type Switches in Application Layer
```go
// BAD: Breaks polymorphism
switch item := p.(type) {
    case *Event: ...
    case *Membership: ...
}

// GOOD: Use interface methods
total := item.GetPrice()  // Polymorphic call
```

### 3. Database Types in Domain
```go
// BAD: Domain depends on database
type Event struct {
    ID sql.NullInt64  // Database type leaked
}

// GOOD: Clean domain types
type Event struct {
    id EventID  // Domain value object
}
```

### 4. Circular Dependencies
```go
// BAD: Cart imports Events, Events imports Cart
// GOOD: Both depend on shared Purchasable interface
```

---

## Execution Workflow

When `/hexagonal-design` is invoked:

1. **Gather Requirements**
   - Parse project description
   - Ask clarifying questions if needed
   - Identify scope boundaries

2. **Discover Modules**
   - Extract nouns (entities)
   - Group into bounded contexts
   - Map dependencies

3. **Design Each Module**
   - Define aggregates and value objects
   - Identify ports needed
   - Plan commands and queries
   - Map adapters

4. **Verify Symmetry**
   - Check vertical alignment
   - Check horizontal patterns
   - Ensure naming consistency

5. **Generate Output**
   - Produce architecture document
   - Include file structure
   - Document business rules

6. **Validate Design**
   - No circular dependencies
   - All modules have clear boundaries
   - Shared types in foundation

---

## See Also

- `/tdd-domain` - Domain layer implementation
- `/tdd-application` - Application layer implementation
- `/tdd-adapter` - Adapter layer implementation
- `/tdd-frontend` - Frontend layer implementation
- `/dev-module` - Full module development workflow
- `STANDARDS.md` - Coding standards reference
- `COMMIT-PR-STRATEGY.md` - Version control workflow

---

*Version: 1.0.0*
*Created: 2026-01-07*
