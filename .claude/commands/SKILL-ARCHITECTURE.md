# Claude Code Skill Architecture

> **Purpose**: Define the layered skill architecture for TDD development with integrated commit/PR workflows.
> **Status**: COMPLETE - All skills implemented
> **Integration**: Reads from COMMIT-PR-STRATEGY.md as single source of truth

---

## Architecture: Four-Tier Skill System

The architecture uses **four-tier separation of concerns**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     TIER 0: COORDINATION SKILLS                          │
│         (Multi-agent coordination, locks, messaging, heartbeats)         │
├─────────────────────────────────────────────────────────────────────────┤
│  /agent register       - Register agent session                         │
│  /agent status         - View all agents, locks, messages               │
│  /agent lock <module>  - Acquire exclusive lock on module/layer         │
│  /agent unlock         - Release lock                                   │
│  /agent broadcast      - Send message to all agents                     │
│  /agent cleanup        - Remove stale locks and messages                │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    │        INFORMS/GATES          │
                    ▼                               ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         TIER 1: STEERING SKILLS                          │
│              (Orchestrate workflow, track progress, enforce strategy)    │
├─────────────────────────────────────────────────────────────────────────┤
│  /dev                  - Smart entry: "What should I work on next?"     │
│  /dev-module <module>  - Full module lifecycle with checkpoints         │
│  /dev-layer <layer>    - Layer-level work with commit guidance          │
│  /dev-checkpoint       - Progress audit: "Where am I? What's next?"     │
│  /dev-pr               - PR preparation following COMMIT-PR-STRATEGY    │
│  /dev lock/unlock      - Lock aliases (convenience)                     │
│  /dev agents           - Status alias (convenience)                     │
└─────────────────────────────────────────────────────────────────────────┘
                                     │
                     ┌───────────────┴───────────────┐
                     │         ORCHESTRATES          │
                     ▼                               ▼
┌────────────────────────────────────┐  ┌────────────────────────────────┐
│   TIER 2: TDD PATTERN SKILLS       │  │   TIER 2: GIT/PR SKILLS        │
│   (Testing methodology)            │  │   (Version control)            │
├────────────────────────────────────┤  ├────────────────────────────────┤
│ Layer Patterns:                    │  │ /commit (plugin)               │
│   /tdd-aggregate                   │  │ /commit-push-pr (plugin)       │
│   /tdd-value-object                │  │ /pr                            │
│   /tdd-command                     │  │                                │
│   /tdd-query                       │  ├────────────────────────────────┤
│   /tdd-repository                  │  │   TIER 2: SECURITY SKILLS      │
│   /tdd-http                        │  │   (Application security)       │
│   /tdd-component                   │  ├────────────────────────────────┤
│   /tdd-hook                        │  │ /security-review               │
├────────────────────────────────────┤  │   - Dependency audit           │
│ Phase Skills:                      │  │   - OWASP Top 10 checks        │
│   /tdd-red                         │  │   - Access control review      │
│   /tdd-green                       │  │   - Secrets detection          │
│   /tdd-refactor                    │  │   - Integrated with /pr        │
│   /tdd-verify                      │  └────────────────────────────────┘
├────────────────────────────────────┤
│ Layer Orchestrators:               │
│   /tdd-domain                      │
│   /tdd-application                 │
│   /tdd-adapter                     │
│   /tdd-frontend                    │
├────────────────────────────────────┤
│ Cross-Module:                      │
│   /tdd-journey                     │
└────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      TIER 3: MODULE CONTEXT SKILLS                       │
│              (Business rules, file inventories, test cases)              │
├─────────────────────────────────────────────────────────────────────────┤
│  /tdd-events      │ /tdd-cart         │ /tdd-facilities                 │
│  /tdd-memberships │ /tdd-programs     │ /tdd-donations                  │
│  /tdd-volunteering│ /tdd-content      │ /tdd-email                      │
│  /tdd-foundation  │ /tdd-pricing      │                                 │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Tier Responsibilities

### Tier 0: Coordination Skills (NEW)

**Purpose**: Enable multiple Claude Code agents to work on the same codebase without conflicts.

| Skill | Responsibility |
|-------|----------------|
| `/agent register` | Register agent session with unique ID |
| `/agent status` | Display all agents, locks, and messages |
| `/agent lock` | Acquire exclusive lock on module/layer |
| `/agent unlock` | Release held locks |
| `/agent broadcast` | Send message to all agents |
| `/agent cleanup` | Remove stale locks and expired messages |

**Key Behaviors**:
1. File-based coordination (no external services)
2. Atomic lock acquisition using filesystem
3. Auto-expiring locks with TTL
4. Heartbeat-based stale detection
5. Message queue for inter-agent communication

**File Locations**:
- `.claude/agents/registry.yaml` - Agent registry and config
- `.claude/agents/locks/*.lock` - One file per lock
- `.claude/agents/queue/*.yaml` - Message queue

### Tier 1: Steering Skills

**Purpose**: Orchestrate the development workflow by combining TDD methodology with COMMIT-PR-STRATEGY.md rules.

| Skill | Responsibility |
|-------|----------------|
| `/dev` | Entry point - analyzes context, suggests next action |
| `/dev-module <module>` | Guides full module development through all layers |
| `/dev-layer <layer>` | Guides layer development with commit checkpoints |
| `/dev-checkpoint` | Audits progress, shows what's done/remaining |
| `/dev-pr` | Prepares PR following COMMIT-PR-STRATEGY template |

**Key Behaviors**:
1. Reads COMMIT-PR-STRATEGY.md for workflow rules
2. Tracks progress through module/layer lifecycle
3. Enforces branch naming: `feat/<module>-<layer>`
4. Prompts for commits at appropriate points
5. Emits exit signals: `MODULE DOMAIN COMPLETE: events`

### Tier 2: Pattern Skills (TDD + Git)

**Purpose**: Provide focused, reusable patterns for specific tasks.

**TDD Pattern Skills** - HOW to write tests and implement:
- `/tdd-aggregate` - Domain aggregate with invariants
- `/tdd-command` - Command handler pattern
- `/tdd-http` - HTTP handler pattern
- etc.

**Git/PR Skills** - HOW to version and share:
- `/commit` - Create atomic commits (existing plugin)
- `/commit-push-pr` - Full PR workflow (existing plugin)

### Tier 3: Module Context Skills (EXISTING - to be slimmed)

**Purpose**: Provide domain-specific business rules and context.

These become reference documents rather than workflow orchestrators:
- Business rules specific to the module
- File inventory (what to create)
- Test case inventory (what to test)
- Exit criteria (when done)

---

## Workflow Example: Developing Events Module

```
Developer: /dev-module events

┌─────────────────────────────────────────────────────────────────┐
│ DEV-MODULE ORCHESTRATION                                        │
├─────────────────────────────────────────────────────────────────┤
│ 1. Creates branch: feat/events-domain                           │
│ 2. Loads context: docs/modules/events.md                        │
│ 3. Shows phase: "Starting DOMAIN layer (1 of 4)"               │
│                                                                 │
│ 4. For each component in domain layer:                          │
│    ├─ Calls /tdd-aggregate Event                                │
│    │    └─ Applies aggregate pattern with events business rules │
│    ├─ After completion: suggests /commit                        │
│    │    └─ "feat(events): add Event aggregate with capacity"   │
│    ├─ Calls /tdd-aggregate EventTicket                          │
│    └─ ... continues through domain components                   │
│                                                                 │
│ 5. At layer complete:                                           │
│    ├─ Runs /tdd-verify domain                                   │
│    ├─ Emits: "MODULE DOMAIN COMPLETE: events"                  │
│    ├─ Suggests: /dev-pr for domain layer                        │
│    └─ Creates new branch: feat/events-application               │
│                                                                 │
│ 6. Continues through application, adapter, frontend layers      │
│                                                                 │
│ 7. At module complete:                                          │
│    ├─ Runs full verification                                    │
│    ├─ Suggests release tag: v0.2.0-events                       │
│    └─ Updates IMPLEMENTATION-STATUS.yaml                        │
└─────────────────────────────────────────────────────────────────┘
```

---

## Integration with COMMIT-PR-STRATEGY.md

The steering layer **reads and enforces** COMMIT-PR-STRATEGY.md:

| Strategy Rule | Steering Skill Enforcement |
|---------------|---------------------------|
| Branch naming: `feat/<module>-<layer>` | `/dev-module` creates correct branch |
| Atomic TDD commits | `/dev-layer` prompts commits after each TDD cycle |
| Commit message format | `/commit` integration uses correct prefix |
| PR size < 500 lines | `/dev-pr` warns if PR too large |
| Exit signals | `/dev-checkpoint` emits at layer completion |
| Phase tags | `/dev-module` suggests tags at phase completion |
| Dependency order | `/dev` enforces Foundation before other modules |

---

## File Structure

```
.claude/
├── agents/                      # Multi-agent coordination
│   ├── registry.yaml            # Agent registry and config
│   ├── locks/                   # Lock files (one per lock)
│   │   └── *.lock
│   └── queue/                   # Message queue
│       └── *.yaml
│
└── commands/
    ├── # COORDINATION SKILLS (TIER 0)
    │   ├── agent.md                 # Main agent coordination skill
    │   └── agent-utils.md           # Shared utilities reference
    │
    ├── # STEERING SKILLS (TIER 1)
    │   ├── dev.md                    # Smart entry point (lock-aware)
    │   ├── dev-module.md             # Module lifecycle orchestrator
    │   ├── dev-layer.md              # Layer development with commits
    │   ├── dev-checkpoint.md         # Progress audit
│   └── dev-pr.md                 # PR preparation
│
├── # MODULE DEFINITION SKILLS
│   ├── feature-brief.md          # Architecture-aware feature capture
│   ├── module-spec.md            # Full specification generator
│   ├── module-checklist.md       # Tracking checklist generator
│   ├── module-refine.md          # Specification validator
│   ├── integration-spec.md       # Cross-module feature specs
│   └── architecture-validate.md  # Validate specs against architecture
│
├── # LAYER PATTERN SKILLS
│   ├── tdd-aggregate.md          # Domain aggregate pattern
│   ├── tdd-value-object.md       # Value object pattern
│   ├── tdd-command.md            # Command handler pattern
│   ├── tdd-query.md              # Query handler pattern
│   ├── tdd-repository.md         # Repository adapter pattern
│   ├── tdd-http.md               # HTTP handler pattern
│   ├── tdd-component.md          # React component pattern
│   └── tdd-hook.md               # React hook pattern
│
├── # LAYER ORCHESTRATOR SKILLS (ENHANCE EXISTING)
│   ├── tdd-domain.md             # Enhance to use pattern skills
│   ├── tdd-application.md        # NEW
│   ├── tdd-adapter.md            # NEW
│   └── tdd-frontend.md           # NEW
│
├── # JOURNEY SKILLS (NEW)
│   └── tdd-journey.md            # Cross-module user journeys
│
├── # SECURITY SKILLS
│   └── security-review.md       # Security analysis for PRs
│
├── # PHASE SKILLS (KEEP AS-IS)
│   ├── tdd.md
│   ├── tdd-red.md
│   ├── tdd-green.md
│   ├── tdd-refactor.md
│   ├── tdd-verify.md
│   ├── tdd-fixture.md
│   └── test.md
│
└── # MODULE CONTEXT SKILLS (SLIM DOWN)
    ├── tdd-events.md             # Slim: business rules + references
    ├── tdd-cart.md
    ├── tdd-facilities.md
    ├── tdd-memberships.md
    ├── tdd-programs.md
    ├── tdd-donations.md
    ├── tdd-volunteering.md
    ├── tdd-content.md
    ├── tdd-email.md
    ├── tdd-foundation.md
    └── tdd-pricing.md
```

---

## Document Hierarchy & Skill Flow

The skills produce and consume documents in a layered hierarchy:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        LEVEL 0: SYSTEM ARCHITECTURE                          │
│                   docs/architecture/SYSTEM-ARCHITECTURE.md                   │
│                                                                              │
│  Created by: /hexagonal-design                                              │
│  Contains: Module inventory, dependency graph, shared types, phases         │
│  Validated by: /architecture-validate --all                                 │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                     ┌───────────────┼───────────────┐
                     ▼               ▼               ▼
┌──────────────────────────┐ ┌──────────────────────────┐ ┌────────────────────┐
│   MODULE SPECIFICATIONS   │ │   SINGLE-MODULE FEATURES │ │ INTEGRATION SPECS  │
│   docs/modules/<m>.md     │ │   features/<m>/<f>.md    │ │ features/          │
│                          │ │                          │ │ integrations/<i>.md│
│ Created: /module-spec    │ │ Created: /feature-brief  │ │ Created:           │
│ Refined: /module-refine  │ │ Validated:               │ │   /integration-spec│
│ Tracked: /module-checklist│ │   /architecture-validate │ │ Validated:         │
└──────────────────────────┘ └──────────────────────────┘ │  /architecture-    │
          │                            │                   │     validate       │
          ▼                            ▼                   └────────────────────┘
┌──────────────────────────────────────────────────────────────────────────────┐
│                            IMPLEMENTATION                                     │
│                                                                              │
│  Executed by: /dev, /dev-module, /dev-layer                                 │
│  Using: /tdd-domain, /tdd-application, /tdd-adapter, /tdd-frontend          │
│  Patterns: /tdd-aggregate, /tdd-command, /tdd-http, etc.                    │
│  Cross-module: /tdd-journey                                                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Typical Workflows

**New Project Setup:**
```
/hexagonal-design "project description"
     ↓
docs/architecture/SYSTEM-ARCHITECTURE.md
     ↓
/architecture-validate --all (baseline check)
```

**Single-Module Feature:**
```
/feature-brief --module events waitlist
     ↓
/architecture-validate features/events/waitlist.md
     ↓
/module-spec features/events/waitlist.md (if complex)
     ↓
/dev features/events/waitlist.md
```

**Cross-Module Feature:**
```
/integration-spec guest-checkout
     ↓
/architecture-validate features/integrations/guest-checkout.md
     ↓
/dev-module (for each affected module, in dependency order)
     ↓
/tdd-journey (for integration testing)
```

**Module Development:**
```
/module-spec events
     ↓
/module-refine docs/modules/events.md
     ↓
/module-checklist events
     ↓
/dev-module events
```

---

## Key Design Decisions

### 1. Why Separate Steering Layer?

**Problem**: TDD skills focus on testing; COMMIT-PR-STRATEGY focuses on workflow. Mixing them creates bloated skills that do too much.

**Solution**: Steering skills orchestrate both without duplicating either.

### 2. Why Not Integrate Commits into TDD Skills?

**Problem**: Each TDD skill would need commit logic, creating duplication.

**Solution**: Steering skills know WHEN to commit; `/commit` knows HOW. Clean separation.

### 3. Why Keep Module Skills?

**Problem**: Module skills contain domain-specific business rules that can't be generalized.

**Solution**: Slim them down to context providers, not workflow orchestrators.

---

## Migration Path

1. **Phase 1**: Create layer pattern skills (building blocks)
2. **Phase 2**: Create steering skills (orchestration)
3. **Phase 3**: Slim down module skills (reference pattern skills)
4. **Phase 4**: Create remaining layer orchestrators (tdd-application, etc.)
5. **Phase 5**: Create journey skills (cross-module)

---

## Success Metrics

After implementation:
- Developer can `/dev-module events` and be guided through entire module
- Commits follow COMMIT-PR-STRATEGY conventions automatically
- PRs are correctly sized and formatted
- Progress is trackable via `/dev-checkpoint`
- No duplication between skills
- Each skill has single responsibility

---

## Implementation Status

| Skill | Status | File |
|-------|--------|------|
| **Coordination Skills (Tier 0)** | | |
| `/agent` | Implemented | `agent.md` |
| `/agent register` | Implemented | `agent.md` |
| `/agent status` | Implemented | `agent.md` |
| `/agent lock` | Implemented | `agent.md` |
| `/agent unlock` | Implemented | `agent.md` |
| `/agent broadcast` | Implemented | `agent.md` |
| `/agent cleanup` | Implemented | `agent.md` |
| Agent Utilities | Implemented | `agent-utils.md` |
| **Steering Skills (Tier 1)** | | |
| `/dev` | ✅ Implemented | `dev.md` |
| `/dev-module` | ✅ Implemented | `dev-module.md` |
| `/dev-layer` | ✅ Implemented | `dev-layer.md` |
| `/dev-checkpoint` | ✅ Implemented | `dev-checkpoint.md` |
| `/dev-pr` | ✅ Implemented | `dev-pr.md` |
| **Layer Pattern Skills** | | |
| `/tdd-aggregate` | ✅ Implemented | `tdd-aggregate.md` |
| `/tdd-value-object` | ✅ Implemented | `tdd-value-object.md` |
| `/tdd-command` | ✅ Implemented | `tdd-command.md` |
| `/tdd-query` | ✅ Implemented | `tdd-query.md` |
| `/tdd-repository` | ✅ Implemented | `tdd-repository.md` |
| `/tdd-http` | ✅ Implemented | `tdd-http.md` |
| `/tdd-component` | ✅ Implemented | `tdd-component.md` |
| `/tdd-hook` | ✅ Implemented | `tdd-hook.md` |
| **Layer Orchestrators** | | |
| `/tdd-domain` | ✅ Enhanced | `tdd-domain.md` |
| `/tdd-application` | ✅ Implemented | `tdd-application.md` |
| `/tdd-adapter` | ✅ Implemented | `tdd-adapter.md` |
| `/tdd-frontend` | ✅ Implemented | `tdd-frontend.md` |
| **Journey Skills** | | |
| `/tdd-journey` | ✅ Implemented | `tdd-journey.md` |
| **Module Skills (Slimmed)** | | |
| `/tdd-events` | ✅ Refactored | `tdd-events.md` |
| **Security Skills** | | |
| `/security-review` | ✅ Implemented | `security-review.md` |
| **Architecture Design Skills** | | |
| `/hexagonal-design` | ✅ Implemented | `hexagonal-design.md` |
| **Module Definition Skills** | | |
| `/feature-brief` | ✅ Enhanced (v2) | `feature-brief.md` |
| `/module-spec` | ✅ Implemented | `module-spec.md` |
| `/module-checklist` | ✅ Implemented | `module-checklist.md` |
| `/module-refine` | ✅ Implemented | `module-refine.md` |
| `/integration-spec` | ✅ Implemented | `integration-spec.md` |
| `/architecture-validate` | ✅ Implemented | `architecture-validate.md` |

---

## Templates

| Template | Purpose | Location |
|----------|---------|----------|
| `ARCHITECTURE-TEMPLATE.md` | Skeleton for new project architectures | `.claude/templates/` |

---

*Document Version: 2.3.0*
*Created: 2026-01-03*
*Updated: 2026-01-07*
