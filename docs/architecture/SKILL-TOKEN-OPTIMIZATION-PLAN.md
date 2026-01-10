# Skill Token Optimization Plan

> **Goal:** Reduce skill token consumption by 80-85% without sacrificing output quality
> **Current:** ~8,664 lines / ~222KB / ~40,000+ tokens
> **Target:** ~1,500 lines / ~40KB / ~7,000 tokens

## Design Principle

**Skills = Instructions only. Zero inline code.**

All code examples live in `.claude/lib/examples/<language>/<concept>.md` and are read on-demand only when Claude needs clarification.

---

## Executive Summary

The current skill architecture prioritizes comprehensiveness over efficiency. Each skill contains:
- **138 inline code blocks** across all skills
- Multi-language examples (Go, Python, Rust, TypeScript) when project uses only Rust + TS
- Extensive educational content loaded on every invocation
- Redundant process descriptions across related skills
- ASCII diagrams duplicated in prose form
- "See Also" and troubleshooting sections rarely needed

**Proposed Solution:** Code-free skills that reference an external example library.

---

## Example Library Architecture

### Directory Structure

```
.claude/lib/examples/
├── rust/
│   ├── testing.md          # Test patterns, naming, AAA, assertions
│   ├── error-handling.md   # Result, ?, unwrap, error types
│   ├── security.md         # unsafe, injection prevention, validation
│   └── common-patterns.md  # Structs, enums, traits, macros
│
├── typescript/
│   ├── testing.md          # Vitest patterns, mocking, assertions
│   ├── error-handling.md   # Try/catch, Result pattern, validation
│   ├── security.md         # XSS, input validation, auth patterns
│   └── common-patterns.md  # Types, interfaces, async patterns
│
└── shared/
    ├── git-conventions.md  # Commit format, branch naming, PR template
    ├── tdd-cycle.md        # RED/GREEN/REFACTOR principles (no code)
    └── aaa-pattern.md      # Arrange-Act-Assert explanation
```

### Why Language → Concept (not Skill → Language)

| Structure | Pros | Cons |
|-----------|------|------|
| `<skill>/<language>` | Direct mapping | Many small files, cross-skill duplication |
| **`<language>/<concept>`** | Grouped patterns, fewer files, DRY | Slightly indirect reference |

**Winner: `<language>/<concept>`** because:
1. One `testing.md` covers tdd-red, tdd-green, tdd-refactor, and test skills
2. Backend work reads only `rust/*`, frontend reads only `typescript/*`
3. Related patterns stay together (test naming + AAA + assertions)
4. ~6 files vs ~40 files

### Example File Content

**`.claude/lib/examples/rust/testing.md` (~150 lines total)**

```markdown
# Rust Testing Patterns

## Test Naming Convention
\`\`\`rust
#[test]
fn test_<subject>_<scenario>_<expected>() { }

// Examples:
fn test_user_validate_with_invalid_email_returns_error() { }
fn test_money_add_with_positive_values_returns_sum() { }
\`\`\`

## AAA Pattern
\`\`\`rust
#[test]
fn test_example() {
    // Arrange
    let user = User::new("test@example.com");

    // Act
    let result = user.validate();

    // Assert
    assert!(result.is_ok());
}
\`\`\`

## Common Assertions
\`\`\`rust
assert_eq!(actual, expected);
assert!(condition);
assert!(result.is_ok());
assert!(result.is_err());
assert_matches!(value, Pattern::Variant { .. });
\`\`\`

## Mocking (with mockall)
\`\`\`rust
#[automock]
trait Repository {
    fn find(&self, id: Uuid) -> Option<Entity>;
}

let mut mock = MockRepository::new();
mock.expect_find()
    .with(eq(id))
    .returning(|_| Some(entity));
\`\`\`

## Testing Errors
\`\`\`rust
#[test]
fn test_returns_error_on_invalid_input() {
    let result = validate("");
    assert!(matches!(result, Err(ValidationError::Empty)));
}
\`\`\`
```

### How Skills Reference Examples

Skills contain zero code, just references:

```markdown
## Test Writing

Follow the AAA pattern (Arrange → Act → Assert).

**Naming:** `test_<subject>_<scenario>_<expected>()`

For syntax examples, see:
- Rust: `.claude/lib/examples/rust/testing.md`
- TypeScript: `.claude/lib/examples/typescript/testing.md`
```

Claude reads the example file **only if needed** (unfamiliar pattern, edge case, etc.).

---

## Current State Analysis

### Token Distribution by Skill Category

| Category | Skills | Lines | Est. Tokens | % of Total |
|----------|--------|-------|-------------|------------|
| **Core TDD** | dev, tdd, tdd-red, tdd-green, tdd-refactor | 1,882 | ~8,500 | 22% |
| **Quality Gates** | lint, test, security-review | 1,119 | ~5,000 | 13% |
| **Git Operations** | commit, pr, clean-worktrees | 967 | ~4,400 | 11% |
| **Architecture** | hexagonal-design, module-spec, etc. | 4,696 | ~21,000 | 54% |
| **Total** | 21 skills | 8,664 | ~40,000 | 100% |

### Waste Analysis

| Waste Type | Instances | Est. Tokens | Savings Potential |
|------------|-----------|-------------|-------------------|
| Non-project language examples (Go, Python) | 47 code blocks | ~4,000 | 100% |
| Redundant TDD phase content | 3 overlapping skills | ~2,500 | 70% |
| Verbose prose → could be tables | 50+ paragraphs | ~3,000 | 60% |
| "See Also" / "Troubleshooting" sections | 35 sections | ~2,000 | 80% |
| Duplicate workflow diagrams | 8 diagrams | ~800 | 50% |
| Inline examples → could reference files | 138 code blocks | ~6,000 | 40% |

**Total Recoverable:** ~15,000-18,000 tokens (37-45%)

---

## Optimization Strategies

### Strategy 1: Hierarchical Skill Architecture

Replace monolithic skills with a two-tier structure:

```
┌─────────────────────────────────────────────────────────┐
│  Tier 1: Core Instructions (~200-400 tokens each)       │
│  - Essential workflow steps                             │
│  - Critical constraints                                 │
│  - When to load Tier 2                                  │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼ (loaded on-demand)
┌─────────────────────────────────────────────────────────┐
│  Tier 2: Reference Library (read via file access)       │
│  - Language-specific examples                           │
│  - Troubleshooting guides                               │
│  - Edge case handling                                   │
└─────────────────────────────────────────────────────────┘
```

**Example - Current `/tdd-red` (271 lines, ~1,200 tokens):**
```markdown
# TDD RED Phase - Write Failing Test
[... 271 lines of content ...]
```

**Optimized `/tdd-red` (~60 lines, ~270 tokens):**
```markdown
# TDD RED Phase

Write a failing test BEFORE implementing.

## Process
1. Identify test location (see `.claude/lib/test-locations.md` if unsure)
2. Write test using AAA pattern: Arrange → Act → Assert
3. Run test - must FAIL for the right reason
4. Verify failure is "missing implementation" not syntax error

## Test Naming
`Test<Subject>_<Scenario>_<Expected>` (Rust/Go)
`should <expected> when <condition>` (TypeScript)

## Output
🔴 RED: Test written and failing
File: <path>
Failure: <reason>

Ready for GREEN: /tdd-green

## Reference
For examples: `.claude/lib/examples/tdd-red-examples.md`
```

**Savings: 78% reduction per skill**

---

### Strategy 2: Consolidate TDD Skills

Current: 4 separate skills with overlapping content
- `/tdd` (326 lines)
- `/tdd-red` (271 lines)
- `/tdd-green` (276 lines)
- `/tdd-refactor` (373 lines)

**Total: 1,246 lines → ~5,600 tokens**

**Proposed: 1 parameterized skill**

```markdown
# TDD - Test-Driven Development

## Usage
/tdd <task>              # Full cycle
/tdd --phase red <task>  # RED only
/tdd --phase green       # GREEN only (requires prior RED)
/tdd --phase refactor    # REFACTOR only (requires prior GREEN)

## Current Phase: ${PHASE}

${PHASE_INSTRUCTIONS}
```

Where `${PHASE_INSTRUCTIONS}` is a compact section (~50-80 lines per phase).

**Target: 350 lines → ~1,600 tokens (71% reduction)**

---

### Strategy 3: Project-Specific Language Filtering

Current skills include examples for: Go, Python, Rust, TypeScript, JavaScript

Project uses: **Rust (backend) + TypeScript (frontend) only**

**Action:** Remove all Go and Python examples from:
- tdd.md, tdd-red.md, tdd-green.md, tdd-refactor.md
- lint.md, test.md
- security-review.md

**47 code blocks × ~50 tokens avg = ~2,350 tokens saved**

---

### Strategy 4: Extract Shared Patterns Library

Create `.claude/lib/patterns/` with reusable content:

```
.claude/lib/patterns/
├── test-naming.md       # Test naming conventions (Rust, TS)
├── commit-format.md     # Conventional commits spec
├── aaa-pattern.md       # Arrange-Act-Assert template
├── error-handling.md    # Common error patterns
└── security-checks.md   # OWASP quick reference
```

Skills reference these instead of duplicating:

```markdown
## Test Naming
See: `.claude/lib/patterns/test-naming.md`
```

Claude reads the file only when needed.

---

### Strategy 5: Compress Prose to Tables

**Before (verbose prose):**
```markdown
### Types

When creating a commit, you should use one of the following types
to categorize your change. The `feat` type is used for new features
that add functionality. The `fix` type is used when you're fixing
a bug. The `refactor` type is for code restructuring that doesn't
change behavior. The `test` type is for adding or updating tests.
The `docs` type is for documentation changes...
```

**After (table):**
```markdown
### Types
| Type | Use For |
|------|---------|
| feat | New functionality |
| fix | Bug fixes |
| refactor | Restructure without behavior change |
| test | Add/update tests |
| docs | Documentation |
```

**Savings: ~60% per section**

---

### Strategy 6: Remove Low-Value Sections

Eliminate or minimize:

| Section | Action | Rationale |
|---------|--------|-----------|
| "See Also" | Remove | Claude knows the skills |
| "Troubleshooting" | Move to lib | Rarely needed inline |
| "CI Alignment" | Remove | Project-specific config |
| Multiple ASCII diagrams | Keep 1 | Redundant visualizations |
| "What NOT to Do" | Condense | 2 bullets max |

---

### Strategy 7: Smart Skill Composition

For `/dev`, instead of loading everything upfront:

**Current flow:**
```
/dev invoked → Load 640 lines immediately
```

**Optimized flow:**
```
/dev invoked → Load core dispatcher (100 lines)
  │
  ├─ Parsing feature file? → Read feature-parser section
  ├─ TDD phase? → Load tdd-core (minimal)
  ├─ Quality check? → Load lint-core or test-core
  └─ Creating PR? → Load pr-core
```

**Implementation:** Use conditional sections marked with HTML comments:

```markdown
# Dev - Feature-Driven Development

## Core Process
[always loaded - ~100 lines]

<!-- SECTION: worktree-management -->
## Worktree Management
[loaded when creating worktree]
<!-- /SECTION -->

<!-- SECTION: tdd-integration -->
## TDD Cycle
[loaded during task execution]
<!-- /SECTION -->
```

---

## Implementation Plan

### Phase 1: Quick Wins (Day 1) - Save ~8,000 tokens

| Task | Est. Savings |
|------|--------------|
| Remove Go/Python examples from all skills | 2,500 |
| Convert verbose prose to tables (5 skills) | 1,500 |
| Remove "See Also" sections (20 skills) | 1,000 |
| Remove redundant diagrams | 500 |
| Condense "What NOT to Do" sections | 500 |
| Remove "CI Alignment" sections | 500 |
| Trim troubleshooting to 3 bullets max | 1,500 |

### Phase 2: Consolidation (Day 2) - Save ~5,000 tokens

| Task | Est. Savings |
|------|--------------|
| Merge tdd-red/green/refactor into tdd | 3,500 |
| Create `.claude/lib/patterns/` library | 1,000 |
| Update skills to reference patterns | 500 |

### Phase 3: Architecture (Day 3) - Save ~5,000 tokens

| Task | Est. Savings |
|------|--------------|
| Implement core/extended split for /dev | 2,000 |
| Implement core/extended split for /pr | 1,000 |
| Implement core/extended split for /security-review | 1,500 |
| Add conditional section loading | 500 |

### Phase 4: Validation (Day 4)

| Task |
|------|
| Test all skills with minimal workflow |
| Verify output quality unchanged |
| Measure actual token reduction |
| Document new architecture |

---

## Expected Results

### Token Reduction Summary

| Category | Current | Code-Free | Reduction |
|----------|---------|-----------|-----------|
| Core TDD (5 skills → 1) | ~8,500 | ~1,200 | 86% |
| Quality Gates | ~5,000 | ~900 | 82% |
| Git Operations | ~4,400 | ~800 | 82% |
| Architecture skills | ~21,000 | ~4,000 | 81% |
| **Total** | **~40,000** | **~7,000** | **82%** |

### Single TDD Task Token Impact

| Metric | Current | Code-Free | Reduction |
|--------|---------|-----------|-----------|
| Skill prompts per task | ~11,400 | ~2,100 | 82% |
| Full feature (5 tasks + PR) | ~32,400 | ~5,500 | 83% |
| % of context window | ~16% | ~2.7% | 83% |

### Example Library (On-Demand)

| File | Lines | Tokens | When Loaded |
|------|-------|--------|-------------|
| `rust/testing.md` | ~150 | ~700 | First Rust test written |
| `rust/error-handling.md` | ~100 | ~450 | Error handling needed |
| `typescript/testing.md` | ~150 | ~700 | First TS test written |
| `shared/git-conventions.md` | ~80 | ~360 | Commit/PR creation |

**Note:** Example files are loaded once per session when needed, not on every skill invocation.

---

## New Skill File Structure

```
.claude/
├── commands/           # Core skills (minimal tokens)
│   ├── dev.md          # ~150 lines (was 640)
│   ├── tdd.md          # ~200 lines (consolidated, was 1,246 across 4 files)
│   ├── lint.md         # ~80 lines (was 315)
│   ├── test.md         # ~80 lines (was 292)
│   ├── commit.md       # ~100 lines (was 364)
│   ├── pr.md           # ~150 lines (was 460)
│   └── security-review.md  # ~150 lines (was 512)
│
├── lib/
│   ├── workflow-state.sh   # (existing)
│   └── patterns/           # (new) Shared reference content
│       ├── test-naming.md
│       ├── commit-format.md
│       ├── aaa-pattern.md
│       └── security-checks.md
│
└── examples/           # (new) Extended examples for edge cases
    ├── tdd-rust.md
    ├── tdd-typescript.md
    └── troubleshooting/
```

---

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Lost context affects output quality | Test each skill before/after, maintain critical instructions |
| File reference overhead | Reference files are small, read is fast |
| Maintenance complexity | Clear documentation, single source of truth |
| Skill behavior regression | Create test cases for each skill |

---

## Success Metrics

1. **Token reduction:** ≥60% overall, ≥70% for TDD workflow
2. **Skill output quality:** Identical to current (spot check 10 scenarios)
3. **Load time:** No perceptible increase
4. **Maintainability:** Easier to update (one place per pattern)

---

## Appendix: Detailed Skill Targets

### Core TDD Skills (Consolidated)

**New `/tdd` skill target: 200 lines**

```
Section                    Lines
─────────────────────────────────
Usage & Arguments            15
Core TDD Cycle Diagram       10
RED Phase (condensed)        40
GREEN Phase (condensed)      40
REFACTOR Phase (condensed)   40
Quality Checks               15
Output Format                20
Reference Links              10
─────────────────────────────────
Total                       190
```

### Dev Skill

**New `/dev` skill target: 150 lines**

```
Section                    Lines
─────────────────────────────────
Usage & Arguments            15
Core Workflow Steps          30
Worktree Quick Reference     20
Task Execution Loop          25
Feature File Format          15
Error Handling               15
Ralph Loop Signals           15
Reference Links               5
─────────────────────────────────
Total                       140
```

### Quality Gate Skills

**New `/lint` target: 80 lines**
**New `/test` target: 80 lines**

Focus on:
- Usage (5 lines)
- Auto-detection table (10 lines)
- Rust commands (15 lines)
- TypeScript commands (15 lines)
- Output format (15 lines)
- Quick troubleshooting (10 lines)

---

## Complete Example: Code-Free `/tdd` Skill

This shows the full optimized skill - consolidating 4 skills (1,246 lines) into one code-free skill (~120 lines).

```markdown
# TDD - Test-Driven Development

Execute the Red → Green → Refactor cycle.

## Usage

/tdd <task>                    # Full cycle
/tdd --phase red <task>        # RED only
/tdd --phase green             # GREEN only
/tdd --phase refactor          # REFACTOR only

## The Cycle

```
RED ──────► GREEN ──────► REFACTOR
Write       Make it       Improve
failing     pass          quality
test        (minimal)     (tests stay green)
```

---

## RED Phase

Write a failing test BEFORE implementation.

### Process
1. Identify test file location (co-located with source)
2. Write test using AAA pattern: Arrange → Act → Assert
3. Name test: `test_<subject>_<scenario>_<expected>`
4. Run test - **must fail**
5. Verify failure reason is "missing implementation" not syntax error

### Output
🔴 RED: Test failing
   File: <path>
   Test: <name>
   Reason: <expected failure>

---

## GREEN Phase

Write MINIMAL code to pass the test.

### Rules
- Only what the test requires
- No extra features
- No optimization
- Hardcoding is OK (next test will force generalization)

### Process
1. Implement smallest change to pass
2. Run test - **must pass**
3. Run all related tests - no regressions

### Output
🟢 GREEN: Test passing
   File: <path>
   Implementation: <brief description>

---

## REFACTOR Phase

Improve code quality. Tests must stay green.

### Targets
| Smell | Fix |
|-------|-----|
| Duplicate code | Extract function |
| Magic numbers | Extract constant |
| Long function | Extract method |
| Poor naming | Rename |

### Rules
- Run tests after EACH small change
- No new functionality
- Undo if tests fail

### DRY Check
- [ ] No repeated code (3+ lines)
- [ ] Using project macros where applicable
- [ ] Single source of truth for business rules

### Output
🔵 REFACTOR: Code improved
   Changes: <list>
   Tests: ✅ All passing

---

## Verification

After cycle complete:
1. Run full test suite: `/test`
2. Run linter: `/lint`
3. Check coverage meets targets

---

## References

For language-specific syntax:
- Rust: `.claude/lib/examples/rust/testing.md`
- TypeScript: `.claude/lib/examples/typescript/testing.md`
- Git conventions: `.claude/lib/examples/shared/git-conventions.md`
```

**Line count: ~120 lines (~540 tokens)**
**Reduction: 90% from original 1,246 lines**

---

## Migration Checklist

### Skills to Consolidate
- [ ] Merge tdd, tdd-red, tdd-green, tdd-refactor → tdd

### Skills to Strip (remove all code blocks)
- [ ] dev.md
- [ ] lint.md
- [ ] test.md
- [ ] commit.md
- [ ] pr.md
- [ ] security-review.md
- [ ] hexagonal-design.md
- [ ] module-spec.md
- [ ] module-checklist.md
- [ ] integration-spec.md
- [ ] architecture-validate.md
- [ ] feature-brief.md
- [ ] module-refine.md

### Example Library to Create
- [ ] `.claude/lib/examples/rust/testing.md`
- [ ] `.claude/lib/examples/rust/error-handling.md`
- [ ] `.claude/lib/examples/rust/security.md`
- [ ] `.claude/lib/examples/rust/common-patterns.md`
- [ ] `.claude/lib/examples/typescript/testing.md`
- [ ] `.claude/lib/examples/typescript/error-handling.md`
- [ ] `.claude/lib/examples/typescript/security.md`
- [ ] `.claude/lib/examples/typescript/common-patterns.md`
- [ ] `.claude/lib/examples/shared/git-conventions.md`
- [ ] `.claude/lib/examples/shared/tdd-cycle.md`
- [ ] `.claude/lib/examples/shared/aaa-pattern.md`

---

*Plan created: 2026-01-09*
*Estimated effort: 3 days*
*Expected savings: 82% token reduction (40,000 → 7,000 tokens)*
