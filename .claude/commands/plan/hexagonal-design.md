# Hexagonal Architecture Design

Transform a project description into a hexagonal architecture specification.

ultrathink: Analyze the project thoroughly, consider multiple design approaches, evaluate trade-offs, and produce a well-structured hexagonal architecture.

## Usage

```
/hexagonal-design <project-description>
/hexagonal-design "E-commerce platform for digital products"
/hexagonal-design                   # Interactive mode
```

---

## Output Structure

| Section | Content |
|---------|---------|
| Module Inventory | Bounded contexts with responsibilities |
| Dependency Graph | Module relationships and build order |
| Domain Layer | Aggregates, value objects, events per module |
| Ports | Interface specifications |
| Application Layer | Commands and queries |
| Adapters | Implementation mapping |
| Frontend Alignment | UI modules mirroring backend |

---

## Phase 1: Domain Discovery

### Questions to Ask

| Question | Discovers |
|----------|-----------|
| Main nouns (entities)? | Aggregates and entities |
| User actions (verbs)? | Commands and use cases |
| State change events? | Domain events |
| Invariants (always-true rules)? | Business rules |
| External systems? | Adapter requirements |

### Bounded Context Criteria

- Single, clear responsibility
- Owns its data (no shared databases)
- Communicates via events or interfaces
- Eventually independently deployable

---

## Phase 2: Module Structure

### Backend Per-Module

| Layer | Files |
|-------|-------|
| Domain | `aggregate.rs`, `value_objects.rs`, `events.rs`, `errors.rs` |
| Ports | `repository.rs`, `reader.rs` (CQRS) |
| Application | `commands/*.rs`, `queries/*.rs` |
| Adapters | `http/handlers.rs`, `postgres/repository.rs` |

### Frontend Per-Module

| Layer | Files |
|-------|-------|
| Domain | `types.ts`, `types.test.ts` |
| API | `api.ts`, `hooks.ts` |
| Components | `Component.svelte`, `Component.test.ts` |

---

## Phase 3: Symmetry Rules

### Vertical (Per Module)

| Layer | Backend | Frontend |
|-------|---------|----------|
| Domain | `domain/<module>/` | `modules/<module>/domain/` |
| Ports | `ports/<module>_*.rs` | N/A |
| Application | `application/` | N/A |
| Adapters | `adapters/http/` | `modules/<module>/api/` |
| Presentation | N/A | `modules/<module>/components/` |

### Horizontal (Across Modules)

| Pattern | Apply To |
|---------|----------|
| Aggregate + Builder | All aggregates |
| Repository + Reader | All persistent modules |
| Command + Query | All modules with operations |

---

## Phase 4: Output Template

```markdown
# [Project] Architecture

## Module Inventory
| Module | Responsibility | Dependencies |
|--------|---------------|--------------|
| foundation | Shared types | None |
| [module] | [description] | foundation |

## Build Order
1. Foundation (Phase 1)
2. [Independent modules] (Phase 2)
3. [Dependent modules] (Phase 3)

## Module: [name]

### Domain
- **Aggregate**: [Name] - [description]
- **Value Objects**: [list]
- **Events**: [list]
- **Invariants**: [rules]

### Ports
- `[Module]Repository` - [methods]
- `[Module]Reader` - [methods]

### Application
- **Commands**: [list]
- **Queries**: [list]

### Adapters
- **HTTP**: [endpoints]
- **Database**: [tables]
```

---

## Design Principles

| Principle | Guideline |
|-----------|-----------|
| Domain First | Understand business before infrastructure |
| Dependency Inversion | High-level depends on abstractions |
| Single Responsibility | One reason to change per component |
| Explicit Dependencies | Constructor injection |
| Test Boundaries | Domain=unit, Application=mock ports, Adapters=integration |

---

## Anti-Patterns

| Anti-Pattern | Symptom | Fix |
|--------------|---------|-----|
| Anemic Domain | Logic in services | Move to aggregate methods |
| Type Switches | Breaking polymorphism | Use interface methods |
| DB in Domain | `sql.NullInt64` in domain | Use value objects |
| Circular Deps | A→B→A | Extract shared interface |

---

## Reference

- Patterns: `.claude/lib/examples/rust/common-patterns.md`
- Error Handling: `.claude/lib/examples/rust/error-handling.md`
