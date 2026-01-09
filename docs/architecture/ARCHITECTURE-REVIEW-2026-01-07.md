# Architecture Specification Review

> **Date:** 2026-01-07
> **Reviewer:** Claude (Architecture Consistency Check)
> **Status:** ~~CRITICAL INCONSISTENCIES FOUND~~ **RESOLVED**

---

## Executive Summary

A comprehensive review of all specifications against the hexagonal architecture design revealed **critical inconsistencies** that must be resolved before implementation begins. The documents are internally inconsistent regarding:

1. **Backend Language** - Go vs Rust
2. **Frontend Framework** - React vs SvelteKit
3. **File Path Conventions** - `internal/` vs `src/`
4. **Component Count** - 8 vs 9 PrOACT types

---

## Documents Reviewed

| Document | Location | Purpose |
|----------|----------|---------|
| SYSTEM-ARCHITECTURE.md | `docs/architecture/` | Master hex arch design |
| RUST-JUSTIFICATION.md | `docs/architecture/` | Backend language decision |
| SVELTEKIT-JUSTIFICATION.md | `docs/architecture/` | Frontend framework decision |
| TECH-STACK-ANALYSIS.md | `docs/architecture/` | Technology analysis |
| functional-spec-20260107.md | `docs/architecture/` | Functional requirements |
| foundation.md | `docs/modules/` | Module specification |
| proact-types.md | `docs/modules/` | Module specification |
| session.md | `docs/modules/` | Module specification |
| cycle.md | `docs/modules/` | Module specification |
| conversation.md | `docs/modules/` | Module specification |
| analysis.md | `docs/modules/` | Module specification |
| dashboard.md | `docs/modules/` | Module specification |
| ai-engine.md | `docs/modules/` | Module specification |
| CHECKLIST-*.md (7 files) | `REQUIREMENTS/` | Implementation checklists |

---

## Critical Inconsistency #1: Backend Language

### The Conflict

| Document | Says |
|----------|------|
| **SYSTEM-ARCHITECTURE.md** | Go (all code examples in Go) |
| **TECH-STACK-ANALYSIS.md** | Go 1.22+ (analyzes Go ecosystem) |
| **RUST-JUSTIFICATION.md** | Rust (explicitly selected) |
| **All module specs** | Rust (all code examples in Rust) |
| **All checklists** | Rust (references cargo, Rust tests) |

### Evidence

**SYSTEM-ARCHITECTURE.md lines 152-203:**
```go
// ComponentType - The 9 PrOACT phases
type ComponentType string
const (
    ComponentIssueRaising ComponentType = "issue_raising"
    ...
)
```

**RUST-JUSTIFICATION.md:**
> "After careful analysis, **Rust** has been selected as the backend language"

**foundation.md line 14:**
> "| **Language** | Rust |"

### Resolution Required

- [ ] Update SYSTEM-ARCHITECTURE.md to use Rust code examples
- [ ] Update TECH-STACK-ANALYSIS.md or mark as superseded
- [ ] Ensure all documents reference the same language

---

## Critical Inconsistency #2: Frontend Framework

### The Conflict

| Document | Says |
|----------|------|
| **SYSTEM-ARCHITECTURE.md line 23** | "React + Module-aligned" |
| **TECH-STACK-ANALYSIS.md** | React 18 + TypeScript |
| **SVELTEKIT-JUSTIFICATION.md** | SvelteKit + TypeScript (explicitly selected) |

### Evidence

**SYSTEM-ARCHITECTURE.md line 23:**
> "| Frontend | React + Module-aligned | UI modules mirror backend bounded contexts |"

**SVELTEKIT-JUSTIFICATION.md:**
> "After thorough analysis, **SvelteKit** with TypeScript has been selected"

### Resolution Required

- [ ] Update SYSTEM-ARCHITECTURE.md to reference SvelteKit
- [ ] Update TECH-STACK-ANALYSIS.md or mark as superseded
- [ ] Add SvelteKit-specific file structure to architecture doc

---

## Critical Inconsistency #3: File Path Conventions

### The Conflict

| Document | Backend Path |
|----------|--------------|
| **SYSTEM-ARCHITECTURE.md** | `backend/internal/domain/` (Go convention) |
| **All module specs** | `backend/src/domain/` (Rust convention) |
| **All checklists** | `backend/src/domain/` (Rust convention) |

### Evidence

**SYSTEM-ARCHITECTURE.md lines 219-233:**
```
backend/internal/domain/foundation/
├── ids.go
├── timestamp.go
...
```

**CHECKLIST-foundation.md lines 22-31:**
```
| `backend/src/domain/foundation/mod.rs` | Module exports |
| `backend/src/domain/foundation/ids.rs` | SessionId, CycleId... |
```

### Resolution Required

- [ ] Update SYSTEM-ARCHITECTURE.md file paths to `backend/src/domain/`
- [ ] Update file extensions from `.go` to `.rs`

---

## Minor Inconsistency #4: Component Count

### The Conflict

| Location | Count |
|----------|-------|
| **SYSTEM-ARCHITECTURE.md line 44** | "8 PrOACT types" |
| **SYSTEM-ARCHITECTURE.md lines 153-165** | Lists 9 components |
| **foundation.md line 48** | "ComponentType (9 variants)" |
| **All other documents** | 9 components |

### Evidence

**SYSTEM-ARCHITECTURE.md line 44:**
> "| `proact-types` | Shared Domain | Component interface, **8 PrOACT types** | foundation |"

But the same document lists 9 components:
1. IssueRaising
2. ProblemFrame
3. Objectives
4. Alternatives
5. Consequences
6. Tradeoffs
7. Recommendation
8. DecisionQuality
9. NotesNextSteps

### Resolution Required

- [ ] Change "8 PrOACT types" to "9 PrOACT types" on line 44

---

## Document Status Assessment

| Document | Status | Action Needed |
|----------|--------|---------------|
| SYSTEM-ARCHITECTURE.md | **NEEDS UPDATE** | Convert Go→Rust, React→SvelteKit, fix paths |
| RUST-JUSTIFICATION.md | Current | None |
| SVELTEKIT-JUSTIFICATION.md | Current | None |
| TECH-STACK-ANALYSIS.md | **SUPERSEDED** | Mark as historical or delete |
| functional-spec-20260107.md | Current | Minor review for consistency |
| foundation.md | Current | None |
| proact-types.md | Current | None |
| session.md | Current | None |
| cycle.md | Current | None |
| conversation.md | Current | None |
| analysis.md | Current | None |
| dashboard.md | Current | None |
| ai-engine.md | Current | None |
| All CHECKLIST-*.md | Current | None |

---

## Alignment Matrix

### Module Specs vs SYSTEM-ARCHITECTURE.md

| Attribute | Module Specs | SYSTEM-ARCHITECTURE.md | Aligned? |
|-----------|--------------|------------------------|----------|
| Backend Language | Rust | Go | ❌ NO |
| Frontend Framework | (implied SvelteKit) | React | ❌ NO |
| Backend File Paths | `backend/src/domain/` | `backend/internal/domain/` | ❌ NO |
| Component Count | 9 | 8 (with 9 listed) | ❌ NO |
| Hexagonal Pattern | Yes | Yes | ✅ YES |
| CQRS (Repository/Reader) | Yes | Yes | ✅ YES |
| Domain Events | Yes | Yes | ✅ YES |
| Aggregate Boundaries | Cycle owns Components | Cycle owns Components | ✅ YES |
| Module Classification | 3 types | 3 types | ✅ YES |
| Build Order | Phase 1-4 | Phase 1-4 | ✅ YES |

### Good News

The core architectural patterns ARE consistent:
- Hexagonal architecture with ports/adapters
- CQRS with separate Repository (write) and Reader (query) ports
- Domain events with `pull_domain_events()` pattern
- Cycle as aggregate root owning components
- Module classification (Shared Domain, Full Module, Domain Services)
- Dependency graph and build order

---

## Recommended Resolution

### Option A: Update SYSTEM-ARCHITECTURE.md (RECOMMENDED)

Since the justification documents and all module specs align on Rust + SvelteKit:

1. **Update SYSTEM-ARCHITECTURE.md**:
   - Convert all Go code examples to Rust
   - Change "React + Module-aligned" to "SvelteKit + Module-aligned"
   - Update file paths from `internal/` to `src/`
   - Fix "8 PrOACT types" to "9 PrOACT types"

2. **Handle TECH-STACK-ANALYSIS.md**:
   - Add header marking it as "SUPERSEDED by RUST-JUSTIFICATION.md and SVELTEKIT-JUSTIFICATION.md"
   - Or delete entirely

### Option B: Revert to Go + React

If the team prefers Go + React, would need to:
- Update all 8 module specs
- Update all 7 checklists
- Delete RUST-JUSTIFICATION.md and SVELTEKIT-JUSTIFICATION.md

**This is not recommended** as significant work has been done on Rust specs.

---

## Checklist for Resolution

- [x] Update SYSTEM-ARCHITECTURE.md line 23: React → SvelteKit
- [x] Update SYSTEM-ARCHITECTURE.md line 44: "8 PrOACT types" → "9 PrOACT types"
- [x] Update SYSTEM-ARCHITECTURE.md lines 152-350+: Convert Go → Rust code examples
- [x] Update SYSTEM-ARCHITECTURE.md file paths: `internal/` → `src/`, `.go` → `.rs`
- [x] Add deprecation notice to TECH-STACK-ANALYSIS.md
- [x] Verify frontend file paths align with SvelteKit conventions
- [x] Update testing strategy table (Go → Rust tools)
- [x] Update deployment architecture (Go → Rust binary)
- [x] Update frontend architecture section (React Query → SvelteKit patterns)

---

*Review completed: 2026-01-07*
*Resolution completed: 2026-01-07*
