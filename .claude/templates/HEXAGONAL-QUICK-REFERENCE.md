# Hexagonal Architecture Quick Reference

> One-page cheat sheet for hexagonal architecture patterns

---

## Layer Responsibilities

```
┌─────────────────────────────────────────────────────────────────┐
│                     DRIVING ADAPTERS                             │
│  (HTTP Handlers, CLI, gRPC, GraphQL, Message Consumers)          │
├─────────────────────────────────────────────────────────────────┤
│                     APPLICATION LAYER                            │
│  (Command Handlers, Query Handlers, Event Handlers)              │
├─────────────────────────────────────────────────────────────────┤
│                        DOMAIN LAYER                              │
│  (Aggregates, Entities, Value Objects, Domain Events)           │
├─────────────────────────────────────────────────────────────────┤
│                          PORTS                                   │
│  (Repository, Reader, Publisher, External Service Interfaces)   │
├─────────────────────────────────────────────────────────────────┤
│                     DRIVEN ADAPTERS                              │
│  (Database, Cache, Message Queue, External APIs)                 │
└─────────────────────────────────────────────────────────────────┘

Dependencies flow INWARD: Adapters → Application → Domain
```

---

## Pattern Templates

### Aggregate

```go
type Order struct {
    id           OrderID              // Value object ID
    items        []*OrderItem         // Child entities
    status       OrderStatus          // Enum
    domainEvents []DomainEvent        // Collected events
}

func NewOrder(customerID string) (*Order, error) {
    // Validation
    return &Order{id: NewOrderID(), status: StatusPending}, nil
}

func (o *Order) AddItem(product Product, qty int) error {
    if o.status != StatusPending { return ErrOrderNotPending }
    o.items = append(o.items, NewOrderItem(product, qty))
    o.addDomainEvent(ItemAdded{OrderID: o.id, ProductID: product.ID()})
    return nil
}

func (o *Order) PullDomainEvents() []DomainEvent {
    events := o.domainEvents
    o.domainEvents = nil
    return events
}
```

### Value Object

```go
type Money struct {
    cents int64
}

func NewMoney(cents int64) (Money, error) {
    if cents < 0 { return Money{}, ErrNegative }
    return Money{cents: cents}, nil
}

func (m Money) Add(other Money) Money {
    return Money{cents: m.cents + other.cents}
}

func (m Money) Equals(other Money) bool {
    return m.cents == other.cents
}
```

### Port Interface

```go
// Write port
type OrderRepository interface {
    Save(ctx context.Context, order *Order) error
    Update(ctx context.Context, order *Order) error
    FindByID(ctx context.Context, id OrderID) (*Order, error)
}

// Read port (CQRS)
type OrderReader interface {
    GetByID(ctx context.Context, id OrderID) (*Order, error)
    List(ctx context.Context, filter OrderFilter) ([]*Order, error)
}
```

### Command Handler

```go
type PlaceOrderCommand struct {
    CustomerID string
    Items      []OrderItemDTO
}

type PlaceOrderHandler struct {
    repo      ports.OrderRepository
    publisher ports.EventPublisher
}

func (h *PlaceOrderHandler) Handle(ctx context.Context, cmd PlaceOrderCommand) (*Result, error) {
    order, err := NewOrder(cmd.CustomerID)
    if err != nil { return nil, err }

    for _, item := range cmd.Items {
        if err := order.AddItem(item.Product, item.Qty); err != nil {
            return nil, err
        }
    }

    if err := h.repo.Save(ctx, order); err != nil { return nil, err }
    h.publisher.Publish(ctx, order.PullDomainEvents()...)

    return &Result{OrderID: order.ID().String()}, nil
}
```

### HTTP Handler

```go
type Handler struct {
    placeOrder *commands.PlaceOrderHandler
}

func (h *Handler) PlaceOrder(w http.ResponseWriter, r *http.Request) {
    var req PlaceOrderRequest
    if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
        respondError(w, 400, "INVALID_JSON", err.Error())
        return
    }

    result, err := h.placeOrder.Handle(r.Context(), req.ToCommand())
    if err != nil {
        h.handleDomainError(w, err)
        return
    }

    respondJSON(w, 201, FromResult(result))
}
```

### Mapper (Database ↔ Domain)

```go
type OrderMapper struct{}

func (m *OrderMapper) ToDomain(row sqlc.OrderRow) (*Order, error) {
    id, _ := ParseOrderID(row.ID.String())
    return &Order{id: id, status: ParseStatus(row.Status)}, nil
}

func (m *OrderMapper) ToCreateParams(o *Order) sqlc.CreateOrderParams {
    return sqlc.CreateOrderParams{
        ID:     uuid.MustParse(o.ID().String()),
        Status: o.Status().String(),
    }
}
```

---

## Directory Structure

```
backend/internal/
├── domain/[module]/       # Business logic (pure, no deps)
│   ├── aggregate.go
│   ├── value_objects.go
│   ├── domain_events.go
│   └── errors.go
├── ports/                 # Interfaces (contracts)
│   ├── [module]_repository.go
│   └── [module]_reader.go
├── application/           # Use cases
│   ├── commands/[module]_*.go
│   └── queries/[module]_*.go
└── adapters/              # Infrastructure
    ├── http/[module]/handlers.go
    ├── postgres/[module]_repository.go
    └── [external]/adapter.go
```

---

## Symmetry Rules

| Rule | Check |
|------|-------|
| Every aggregate has a builder | `*Builder` type exists |
| Every port has an adapter | Implementation satisfies interface |
| Every command has a handler | `*Handler` type with `Handle()` |
| Backend type = Frontend type | Same fields, same names |
| Repository = write, Reader = read | CQRS separation |

---

## Testing Strategy

| Layer | Test Type | Mocking |
|-------|-----------|---------|
| Domain | Unit | None needed (pure functions) |
| Application | Unit | Mock ports |
| Adapters | Integration | Real infrastructure |
| E2E | User journey | Full stack |

---

## Anti-Patterns

| Bad | Good | Why |
|-----|------|-----|
| `orderService.Cancel(order)` | `order.Cancel()` | Logic in aggregate |
| `switch item.(type)` | `item.GetPrice()` | Use polymorphism |
| `type Order { ID sql.NullInt64 }` | `type Order { id OrderID }` | No DB types in domain |
| Domain imports adapter | Adapter imports domain | Dependency inversion |

---

## Quick Commands

```bash
# Run domain tests
go test ./internal/domain/... -v

# Check coverage
go test ./internal/domain/... -cover

# Run specific test
go test -run "TestOrder_AddItem" ./internal/domain/orders/...

# Generate sqlc code
make sqlc-generate

# Frontend tests
npm run test -- --grep "OrderForm"
```

---

*Keep this reference handy during implementation!*
