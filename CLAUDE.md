# Choice Sherpa - Project Guide

## Project Overview

**Choice Sherpa** is an interactive decision support application that guides users through the PrOACT framework via conversational AI. The system is a living dashboard that captures, organizes, and presents decision-relevant information through structured conversations.

### Core Concepts

| Concept | Description |
|---------|-------------|
| **Session** | Top-level container for a decision context |
| **Cycle** | A complete/partial path through PrOACT (supports branching) |
| **Component** | One of 8 PrOACT steps with structured outputs |
| **Conversation** | AI-guided dialogue within each component |

### PrOACT Components (8 Steps)

1. **Issue Raising** - Categorize initial thoughts into decisions, objectives, uncertainties
2. **Problem Frame** - Define decision architecture, constraints, stakeholders
3. **Objectives** - Identify fundamental vs means objectives with measures
4. **Alternatives** - Capture options, strategy tables, status quo baseline
5. **Consequences** - Build consequence table with Pugh ratings (-2 to +2)
6. **Tradeoffs** - Surface dominated alternatives, tensions, irrelevant objectives
7. **Recommendation** - Synthesize analysis (does NOT decide for user)
8. **Decision Quality** - Rate 7 elements (0-100%), overall = minimum score

---

## Architecture

Hexagonal architecture with TDD development workflow.

### Module Classification

| Type | Purpose | Examples |
|------|---------|----------|
| **Shared Domain** | Type definitions only (no ports/adapters) | foundation, proact-types |
| **Full Module** | Complete bounded context with ports/adapters | session, membership, cycle, conversation, dashboard |
| **Domain Services** | Stateless business logic (pure functions) | analysis |

### Module Inventory

| Module | Type | Responsibility | Dependencies |
|--------|------|---------------|--------------|
| `foundation` | Shared Domain | Value objects, IDs, enums, errors | None |
| `proact-types` | Shared Domain | Component interface, 9 PrOACT types | foundation |
| `membership` | Full Module | Subscriptions, access control, payments | foundation |
| `session` | Full Module | Session lifecycle, user ownership | foundation, membership (access check) |
| `cycle` | Full Module | Cycles, components (owned), branching | foundation, proact-types, session |
| `conversation` | Full Module | AI agent behavior, message handling | foundation, proact-types |
| `analysis` | Domain Services | Pugh matrix, DQ scoring (pure functions) | foundation, proact-types |
| `dashboard` | Full Module | Read models for Overview/Detail views | all modules |

### Build Order

```
Phase 1: foundation
Phase 2: membership, proact-types (parallel)
Phase 3: session (depends on membership AccessChecker)
Phase 4: cycle, conversation, analysis (parallel)
Phase 5: dashboard
```

### Dependency Graph

```
                         ┌─────────────┐
                         │  dashboard  │ (Phase 5)
                         └──────┬──────┘
            ┌───────────────────┼───────────────────┐
     ┌──────▼──────┐     ┌──────▼──────┐     ┌──────▼──────┐
     │conversation │     │  analysis   │     │    cycle    │ (Phase 4)
     └──────┬──────┘     └──────┬──────┘     └──────┬──────┘
            └───────────────────┼───────────────────┘
                         ┌──────▼──────┐
                         │ proact-types│ (Phase 2)
                         └──────┬──────┘
            ┌───────────────────┼───────────────────┐
     ┌──────▼──────┐     ┌──────▼──────┐            │
     │   session   │────►│ membership  │            │
     └─────────────┘     └──────┬──────┘            │
       (Phase 3)           (Phase 2)                │
            └───────────────────┴───────────────────┘
                         ┌──────▼──────┐
                         │ foundation  │ (Phase 1)
                         └─────────────┘
```

**Note:** Session depends on Membership via the `AccessChecker` port for gating session creation.

### Aggregate Boundaries

```
Session (refs CycleIDs) ──► Cycle (OWNS Components) ◄── Conversation (refs ComponentID)
```

- **Cycle is the aggregate root** for components
- Components are child entities embedded in Cycle
- Conversations reference components by ID

### Document Hierarchy

```
docs/architecture/SYSTEM-ARCHITECTURE.md    ← System design (Level 0)
     ↓
docs/modules/<module>.md                    ← Module specifications (Level 1)
     ↓
features/<module>/<feature>.md              ← Feature specifications (Level 2)
features/integrations/<integration>.md      ← Cross-module features
     ↓
REQUIREMENTS/CHECKLIST-<module>.md          ← Implementation tracking
```

---

## Technology Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust (axum, sqlx, tokio, serde) |
| Frontend | SvelteKit + TypeScript |
| Database | PostgreSQL 16 |
| Cache/PubSub | Redis |
| Auth | Zitadel (self-hosted OIDC) |
| Email | Resend |
| AI Provider | OpenAI / Anthropic (port-based) |
| API | REST + WebSocket (streaming) |

### File Structure

```
backend/
├── src/
│   ├── domain/{foundation,membership,session,cycle,proact,conversation,analysis,dashboard}/
│   │   └── membership/value_objects/  # Money (cents!), Tier, Status, etc.
│   ├── ports/           # Repository and service interfaces
│   │   ├── access_checker.rs    # Cross-module access control
│   │   └── payment_provider.rs  # External payment (Stripe)
│   ├── application/     # Commands and queries
│   └── adapters/        # HTTP, Postgres, Redis, AI, Stripe implementations
├── migrations/
└── Cargo.toml

frontend/
├── src/
│   ├── lib/             # Shared types, components, hooks
│   ├── routes/          # SvelteKit routes
│   │   ├── membership/  # Subscription management
│   │   ├── pricing/     # Plans and pricing
│   │   └── account/     # User account
│   └── modules/         # Module-aligned (session, cycle, proact, etc.)
└── package.json
```

---

## Commands

```yaml
## Test Commands
test_all: cargo test
test_coverage: cargo tarpaulin --out Html
test_frontend: cd frontend && npm test

## Lint Commands
lint: cargo clippy -- -D warnings
lint_frontend: cd frontend && npm run lint
format: cargo fmt --all

## Build Commands
build: cargo build --release
build_frontend: cd frontend && npm run build

## Database
migrate_up: sqlx migrate run
migrate_down: sqlx migrate revert

## Development
dev: cargo watch -x run
dev_frontend: cd frontend && npm run dev
```

---

## Key Domain Rules

### Cycle Branching
- Cycles can branch at any started component
- Child cycle inherits all completed components up to branch point
- Enables "what if" exploration without losing work

### Component Progression
- Components should be started in order (can skip, not go back)
- Only one component can be "in progress" at a time
- Conversation history preserved for each component

### Decision Quality
- 7 elements rated 0-100%
- Overall DQ = minimum of all element scores
- 100% = "good decision at time made, regardless of outcome"

### AI Agent Behavior
- Acts as "thoughtful decision professional"
- Asks probing questions, surfaces assumptions
- Does NOT make decisions for users
- Synthesizes conversations into structured outputs

---

## Available Skills

### Architecture & Definition

| Skill | Description |
|-------|-------------|
| `/hexagonal-design` | Design system architecture from project description |
| `/feature-brief` | Quick feature capture (architecture-aware) |
| `/module-spec` | Full module specification |
| `/module-checklist` | Generate implementation checklist |
| `/module-refine` | Validate and improve specifications |
| `/integration-spec` | Cross-module feature specification |
| `/architecture-validate` | Validate specs against architecture |

### TDD & Execution

| Skill | Description |
|-------|-------------|
| `/dev <path>` | Process feature file or folder (uses worktrees) |
| `/tdd <task>` | Single TDD cycle |
| `/tdd-red` | RED phase - write failing test |
| `/tdd-green` | GREEN phase - minimal implementation |
| `/tdd-refactor` | REFACTOR phase - improve quality |
| `/test` | Run tests |
| `/lint` | Run linters |
| `/commit` | Create git commit |
| `/pr` | Create pull request |
| `/checklist-sync` | Sync REQUIREMENTS checklist with filesystem state |
| `/clean-worktrees` | Remove worktrees for merged PRs |

### Security & Code Quality

| Skill | Description |
|-------|-------------|
| `/security-review` | Analyze code for security vulnerabilities (OWASP, dependencies) |
| `/code-simplifier` | Review code for unnecessary complexity and suggest simplifications |

**Note:** Both `/security-review` and `/code-simplifier` are automatically invoked by `/pr`. PRs are blocked if CRITICAL or HIGH severity security issues are found.

---

## Worktree-Based Development

Development uses **git worktrees** for module isolation. Each module gets its own worktree directory, enabling:

- **Parallel development** across multiple terminals without branch conflicts
- **Clustered commits** - all work on a module stays on one branch
- **Single PR per module** - cleaner review process
- **No branch switching** - work in `.worktrees/<module>/` directly

### Worktree Locations

```
choice-sherpa/
├── .worktrees/              # Module worktrees (gitignored)
│   ├── session/             # → feat/session branch
│   ├── membership/          # → feat/membership branch
│   └── cycle/               # → feat/cycle branch
├── backend/                 # Main repo (unchanged)
├── frontend/
└── ...
```

### Development Flow

```bash
# 1. Start work on a module (creates worktree automatically)
/dev features/session/

# 2. Work happens in .worktrees/session/
#    All commits go to feat/session branch
#    Multiple features in session/ share the same worktree

# 3. When module complete, PR is created
#    Worktree stays until PR merged

# 4. After merge, clean up
/clean-worktrees
```

### Manual Worktree Commands

```bash
# List active worktrees
git worktree list

# Create worktree manually
git worktree add .worktrees/mymodule feat/mymodule

# Remove worktree
git worktree remove .worktrees/mymodule
```

---

## Workflow

### 1. Define Features
```bash
# Single-module feature
/feature-brief --module session create-session

# Cross-module feature
/integration-spec cycle-branching

# Validate against architecture
/architecture-validate features/session/create-session.md
```

### 2. Elaborate (If Needed)
```bash
# Complex features need full specs
/module-spec features/session/create-session.md
/module-refine docs/modules/session.md
/module-checklist session
```

### 3. Implement (TDD)
```bash
# Execute feature with TDD
/dev features/session/create-session.md

# Or process entire module
/dev features/session/
```

---

## Deferred Concerns

The following areas are explicitly **not addressed** in the current scope:

| Area | Status | Notes |
|------|--------|-------|
| **Multi-tenancy** | Not started | Single-user focus for MVP |
| **Analytics/Telemetry** | Not started | No usage tracking or business metrics |

These will require dedicated planning phases before implementation.

**Note:** Monetization/membership is now addressed via the `membership` module (see `docs/modules/membership.md`).

---

## Quick Reference

- **System Architecture**: `docs/architecture/SYSTEM-ARCHITECTURE.md`
- **Security Standard**: `docs/architecture/APPLICATION-SECURITY-STANDARD.md`
- **Functional Spec**: `docs/architecture/functional-spec-20260107.md`
- **Hexagonal Cheat Sheet**: `.claude/templates/HEXAGONAL-QUICK-REFERENCE.md`
