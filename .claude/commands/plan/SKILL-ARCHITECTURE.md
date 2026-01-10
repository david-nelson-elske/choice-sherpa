# Claude Code Skill Architecture

Layered skill architecture for TDD development with integrated workflows.

---

## Four-Tier System

```
┌─────────────────────────────────────────────────┐
│              TIER 1: ORCHESTRATION               │
│  /dev, /tdd, /pr, /commit                        │
├─────────────────────────────────────────────────┤
│              TIER 2: PATTERNS                    │
│  /lint, /test, /security-review                  │
├─────────────────────────────────────────────────┤
│              TIER 3: DEFINITION                  │
│  /feature-brief, /module-spec, /integration-spec │
│  /module-checklist, /module-refine               │
│  /architecture-validate, /checklist-sync         │
├─────────────────────────────────────────────────┤
│              TIER 4: REFERENCE                   │
│  .claude/lib/examples/                           │
│  Rust, TypeScript, and shared patterns           │
└─────────────────────────────────────────────────┘
```

---

## Tier Responsibilities

### Tier 1: Orchestration

| Skill | Responsibility |
|-------|----------------|
| `/dev` | Feature-driven development entry point |
| `/tdd` | Execute RED→GREEN→REFACTOR cycle |
| `/commit` | Create atomic commits |
| `/pr` | Create pull requests with verification |

### Tier 2: Patterns

| Skill | Responsibility |
|-------|----------------|
| `/lint` | Run code quality checks |
| `/test` | Run tests with coverage |
| `/security-review` | OWASP analysis, dependency audit |

### Tier 3: Definition

| Skill | Responsibility |
|-------|----------------|
| `/hexagonal-design` | Design system architecture |
| `/feature-brief` | Quick feature capture |
| `/module-spec` | Full module specification |
| `/module-checklist` | Generate tracking checklist |
| `/module-refine` | Validate specifications |
| `/integration-spec` | Cross-module features |
| `/architecture-validate` | Validate against architecture |
| `/checklist-sync` | Sync checklist with filesystem |

### Tier 4: Reference Library

| Path | Content |
|------|---------|
| `.claude/lib/examples/rust/` | Rust patterns and examples |
| `.claude/lib/examples/typescript/` | TypeScript patterns |
| `.claude/lib/examples/shared/` | Git, TDD, testing patterns |

---

## Document Hierarchy

```
docs/architecture/SYSTEM-ARCHITECTURE.md    ← Level 0 (System)
     ↓
docs/modules/<module>.md                    ← Level 1 (Module Spec)
     ↓
features/<module>/<feature>.md              ← Level 2 (Feature)
features/integrations/<integration>.md
     ↓
REQUIREMENTS/CHECKLIST-<module>.md          ← Level 3 (Tracking)
```

---

## Workflow: New Project

```
/hexagonal-design → docs/architecture/SYSTEM-ARCHITECTURE.md
     ↓
/architecture-validate --all (baseline)
```

## Workflow: Single Feature

```
/feature-brief --module events waitlist
     ↓
/architecture-validate features/events/waitlist.md
     ↓
/dev features/events/waitlist.md
```

## Workflow: Module Development

```
/module-spec events → docs/modules/events.md
     ↓
/module-refine docs/modules/events.md
     ↓
/module-checklist events → REQUIREMENTS/CHECKLIST-events.md
     ↓
/dev features/events/
```

---

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Separate tiers | Clear separation of concerns |
| Reference library | On-demand loading reduces tokens |
| Code-free skills | Instructions only, no inline examples |
| Table compression | Scan faster than prose |

---

## Success Metrics

- Developer can `/dev features/X` and be guided through implementation
- Skills reference library instead of inline code
- PRs follow conventions automatically
- Progress trackable via `/checklist-sync`
- Each skill has single responsibility
