# Technology Stack Analysis

> **SUPERSEDED** - This document has been superseded by:
> - [`RUST-JUSTIFICATION.md`](./RUST-JUSTIFICATION.md) - Backend language decision
> - [`SVELTEKIT-JUSTIFICATION.md`](./SVELTEKIT-JUSTIFICATION.md) - Frontend framework decision
>
> The analysis below was the initial review. The final decisions select **Rust** for backend and **SvelteKit** for frontend. See the justification documents for rationale.

---

> **Created:** 2026-01-07
> **Status:** ~~Architecture Review~~ SUPERSEDED
> **Purpose:** Deep analysis of language, frameworks, and external dependencies (historical)

---

## Technology Stack Summary

| Layer | Technology | Version/Spec |
|-------|-----------|--------------|
| **Backend Language** | Go | 1.22+ |
| **Frontend Language** | TypeScript | - |
| **Frontend Framework** | React | 18 |
| **Database** | PostgreSQL | 16 |
| **Cache/PubSub** | Redis | - |
| **AI Providers** | OpenAI / Anthropic | Port-abstracted |
| **API Style** | REST + WebSocket | Streaming for conversations |
| **State Management** | React Query | @tanstack/react-query |

---

## Backend: Go 1.22+

### Strengths for This Domain

1. **Hexagonal architecture fit**: Go's explicit interfaces without inheritance make port/adapter patterns natural - interfaces are satisfied implicitly, encouraging clean boundaries

2. **Concurrency model**: goroutines + channels ideal for WebSocket streaming and handling multiple AI provider calls

3. **Compilation speed**: Fast feedback loops during TDD cycles

4. **Deployment simplicity**: Single binary deployment, no runtime dependencies

### Considerations

- Go's type system lacks generics sophistication (improved in 1.18+, but still limited) - the `GetStructuredOutput() interface{}` pattern in proact-types will require type assertions

- No sum types means `ComponentType` enum validation relies on runtime checks, not compile-time guarantees

- Error handling verbosity can clutter business logic

### Implied Dependencies

| Tool | Purpose | Evidence |
|------|---------|----------|
| **sqlc** | Type-safe SQL code generation | `sqlc/queries/*.sql` paths in file structure |
| **testify** | Test assertions | Testing strategy section |
| **mockery** | Mock generation | Testing strategy section |
| **testcontainers** | Integration testing | Testing strategy section |

---

## Frontend: React 18 + TypeScript

### Architecture Alignment

1. **Module-mirrored structure**: Frontend modules (`session/`, `cycle/`, `conversation/`) mirror backend bounded contexts - this is excellent for developer navigation and reduces cognitive load

2. **React Query (@tanstack/react-query)**: Perfect for CQRS read models - handles caching, invalidation, and optimistic updates without a global store

3. **TypeScript**: Type definitions mirror Go structs, enabling compile-time validation across the stack

### Missing Specifications

| Category | Status | Options to Consider |
|----------|--------|---------------------|
| Bundler/Build Tool | ❌ Not specified | Vite (recommended), webpack, esbuild |
| Component Library | ❌ Not specified | shadcn/ui, Radix UI, Headless UI |
| Form Handling | ❌ Not specified | react-hook-form, formik |
| Routing | ❌ Not specified | react-router v6, @tanstack/router |
| CSS/Styling | ❌ Not specified | Tailwind CSS, CSS Modules, styled-components |

---

## Database: PostgreSQL 16

### Excellent Fit For

- **JSONB columns** for `structured_data` in components table
- **Strong ACID guarantees** important for conversation history preservation
- **Native UUID support** (`gen_random_uuid()`)
- **Advanced indexing** on JSONB fields for component queries
- **Row-level security** potential for multi-tenant scenarios

### Schema Design Notes

The schema uses `ON DELETE CASCADE` for:
- `cycles.session_id → sessions.id`
- `components.cycle_id → cycles.id`
- `conversation_messages.conversation_id → conversations.id`

This correctly implements the aggregate ownership model where:
- Deleting a session cascades to cycles
- Deleting a cycle cascades to components
- Deleting a conversation cascades to messages

---

## AI Provider Abstraction

### Port-Based Design

```go
type AIProvider interface {
    Complete(ctx context.Context, req CompletionRequest) (*CompletionResponse, error)
    Stream(ctx context.Context, req CompletionRequest) (<-chan CompletionChunk, error)
}
```

### Strengths

1. **Provider swapping**: Switch between OpenAI and Anthropic without domain changes
2. **Testing isolation**: `MockAIAdapter` enables deterministic tests
3. **Correct placement**: Interface in `ports/`, implementations in `adapters/ai/`

### Missing Considerations

| Concern | Status | Recommendation |
|---------|--------|----------------|
| Retry logic | ❌ Not specified | Implement exponential backoff with jitter |
| Circuit breaker | ❌ Not specified | Consider sony/gobreaker or similar |
| Rate limiting | ❌ Not specified | Client-side rate limiting for API quotas |
| Timeout handling | ❌ Not specified | Context-based timeouts, streaming chunk timeouts |
| Cost tracking | ❌ Not specified | Token counting and budget enforcement |

---

## Redis Usage

### Specified Purposes

1. Sessions (unclear if HTTP sessions or application sessions)
2. PubSub (unclear target use case)

### Likely Use Cases

| Use Case | Confidence | Notes |
|----------|------------|-------|
| WebSocket connection state | High | Multi-instance coordination |
| AI response streaming buffer | Medium | Chunk aggregation |
| Rate limiting state | Medium | Per-user API limits |
| Conversation context caching | Low | PostgreSQL may suffice |

### Recommendation

Clarify Redis's role before implementation. If only used for PubSub, consider:
- PostgreSQL LISTEN/NOTIFY for simpler deployments
- In-memory channels for single-instance development

---

## Potential Issues & Gaps

### 1. Event Sourcing Claim vs. Implementation

**Documented claim:**
> State Management: Event Sourced

**Actual implementation pattern:**
- Tables have mutable state (`status`, `structured_data`)
- `PullDomainEvents()` pattern suggests event publishing, not event sourcing
- No event store schema
- No reconstitution from events

**Verdict:** This is **event-driven architecture** with **domain event publishing**, not true event sourcing. The terminology should be corrected to avoid confusion during implementation.

### 2. Missing External Dependencies

| Category | Gap | Recommended Options |
|----------|-----|---------------------|
| **Authentication** | No auth system | Auth0, Clerk, or JWT + bcrypt |
| **API Framework** | No Go HTTP router | chi (recommended), echo, fiber |
| **Migrations** | Tool referenced but not named | golang-migrate, goose |
| **Logging** | No structured logging | zerolog (recommended), zap |
| **Configuration** | No config management | viper, envconfig, koanf |
| **Observability** | No metrics/tracing | OpenTelemetry, Prometheus |
| **Validation** | No input validation library | go-playground/validator |

### 3. Type Mapping Complexity

The 9 component types with polymorphic structured outputs create complexity:

```go
type Component interface {
    GetStructuredOutput() interface{}
    SetStructuredOutput(data interface{}) error
}
```

**Implications:**
- Type switches required in handlers
- Custom JSON marshaling/unmarshaling needed
- JSONB querying in PostgreSQL requires careful indexing
- Frontend type guards needed for TypeScript

**Alternative considered:** Separate tables per component type vs. single `components` table with JSONB. The chosen JSONB approach is simpler but trades off query power for flexibility.

---

## Dependency Graph Validation

### Build Order

```
Phase 1: foundation (no deps)
Phase 2: proact-types, session (parallel, both depend on foundation)
Phase 3: cycle, conversation, analysis (depend on foundation + proact-types)
Phase 4: dashboard (depends on all)
```

### Validation Results

✅ Dependencies form a DAG (no cycles)

⚠️ **Hidden dependency:** `conversation` module imports `CycleRepository` to update component outputs - this creates an implicit dependency on `cycle` module not shown in the dependency diagram.

```
conversation → cycle (implicit, for UpdateComponentOutput)
```

---

## Summary Assessment

| Aspect | Rating | Notes |
|--------|--------|-------|
| **Language choice** | ✅ Strong | Go + TypeScript are solid for this domain |
| **Framework choices** | ⚠️ Incomplete | Core frameworks need explicit specification |
| **Database** | ✅ Strong | PostgreSQL 16 is excellent fit |
| **AI abstraction** | ✅ Strong | Port-based design is correct |
| **Architecture clarity** | ⚠️ Mixed | Event sourcing claim incorrect; some gaps |
| **Frontend stack** | ⚠️ Incomplete | Missing bundler, component library, routing |
| **Observability** | ❌ Missing | No logging, metrics, or tracing specified |
| **Security** | ❌ Missing | No authentication/authorization specified |

---

## Recommendations

### Immediate Actions

1. **Create explicit TECH-STACK.md** with all dependencies and versions
2. **Correct "event sourcing" terminology** to "event-driven" or "domain events"
3. **Select and document** HTTP router, logging, and config libraries
4. **Define authentication strategy** before session module implementation

### Before Phase 1 (foundation)

- [ ] Select Go HTTP router (recommend: chi)
- [ ] Select logging library (recommend: zerolog)
- [ ] Select config library (recommend: viper or envconfig)
- [ ] Select validation library (recommend: go-playground/validator)

### Before Phase 2 (session)

- [ ] Define authentication/authorization strategy
- [ ] Select migration tool (recommend: golang-migrate)
- [ ] Define error handling patterns with codes

### Before Frontend Work

- [ ] Select bundler (recommend: Vite)
- [ ] Select component library (recommend: shadcn/ui + Radix)
- [ ] Select routing library (recommend: react-router v6)
- [ ] Select form library (recommend: react-hook-form + zod)

---

*Analysis Version: 1.0.0*
*Based on: SYSTEM-ARCHITECTURE.md v1.1.0*
