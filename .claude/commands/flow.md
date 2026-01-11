# Flow - Streamlined TDD Development

Execute spec → implement → PR in one continuous flow with progressive context loading.

## Usage

```
/flow <spec-path>
```

---

## Process

**1. VALIDATE SPEC**
- Verify spec file exists (required argument)
- If missing: error with helpful message
- Extract **lightweight summary**: title, task list (IDs only), context

**2. SETUP WORKTREE** (with isolation)
- Module = parent folder of spec (or filename without extension)
- Branch = `feat/<module>`
- Worktree = `.worktrees/<module>/` (absolute path from repo root)
- Run: `git worktree add .worktrees/<module> -b feat/<module>` (skip if exists)
- Verify correct branch: `git branch --show-current` must equal `feat/<module>`
- Spec path is now `$WORKTREE_PATH/features/<module>/<spec>.md`

**3. FOR EACH TASK** (progressive loading)
- Find next unchecked `- [ ]` in spec (in worktree)
- Extract **task context**:
  - The task line itself
  - The matching requirement row from Requirements table (by ID)
  - If task has `sec` tag → also load Security section
  - If task has `perf` tag → also load Performance notes if present
- 3.1 **RED** — Write failing test for the requirement's Given/When/Then
- 3.2 **GREEN** — Minimal implementation, test passes
- 3.3 **REFACTOR** — Clean up, tests still pass
- 3.4 **COMMIT** — `feat(<module>): <task description>`
- Mark `- [x]` in spec, repeat for next task

**4. LINT**
- `cargo clippy -- -D warnings`
- `cd frontend && npm run lint`
- Must pass (fix if needed)

**5. TEST**
- `cargo test`
- `cd frontend && npm test`
- Must pass (fix if needed)

**6. ARCHIVE SPEC**
- `mv <spec> docs/features/<module>/`

**7. FINAL COMMIT**
- `chore(<module>): archive <spec-name>`

**8. CREATE PR**
- Push branch
- `gh pr create --title "[<module>] <title>"`
- Return PR URL

---

## Error Recovery

| Phase | On Failure |
|-------|------------|
| RED | Test must fail. If passes → wrong test |
| GREEN | Fix implementation until pass |
| REFACTOR | Undo if tests fail |
| LINT | Run `--fix`, then fix remaining |
| TEST | Fix failing tests before continue |

---

## Context Loading Strategy

**Goal:** Minimize tokens while preserving spec fidelity.

| Phase | What's Loaded | ~Tokens |
|-------|---------------|---------|
| Planning | Header + Task IDs + Context | ~200 |
| Per task | Task line + Requirement row | ~100 |
| Security task | + Security section | +150 |
| Verification | Full Requirements table | ~300 |

**Example:** For task `- [ ] S1: Reject rename by non-owner \`sec\``

Load:
```markdown
## Context
- Extend Session aggregate with `rename()` method

## Task
- [ ] S1: Reject rename by non-owner `sec`

## Requirement
| S1 | Owner only | Non-owner user | Rename | Reject | `ForbiddenError` |

## Security
| session.owner_id | Internal | Compare against authenticated user_id |
```

**NOT loaded:** Other requirement rows, other tasks, header prose.

---

## Spec Format

```markdown
# Feature: <Title>

> One-line summary.

## Requirements
| ID | Rule | Given | When | Then | Error |
|----|------|-------|------|------|-------|
| R1 | ... | ... | ... | ... | ... |

## Tasks
- [ ] R1: First task
- [ ] R2: Second task

## Context
- Key constraint
```

---

## Worktree Isolation

**Problem:** Multiple terminals may work on different modules simultaneously. Each must be isolated to its own worktree/branch.

**Rules:**

| Rule | Implementation |
|------|----------------|
| **Absolute paths** | Always use `$WORKTREE_PATH/...` for file operations |
| **Verify before write** | Check `git branch --show-current` before any commit |
| **No relative cd** | Never `cd ..` back to main repo during task execution |
| **Scoped tools** | All Read/Write/Edit operations use absolute worktree paths |

**Multi-terminal safety:**
```bash
# Terminal 1: Working on session module
cd /home/user/project/.worktrees/session/
git branch --show-current  # → feat/session

# Terminal 2: Working on cycle module
cd /home/user/project/.worktrees/cycle/
git branch --show-current  # → feat/cycle
```

Each terminal's git operations are **automatically scoped** to its worktree's branch because:
1. `.git` in worktree points to main repo's git dir
2. But `HEAD` is independent per worktree
3. Commits go to the worktree's checked-out branch

**Agent spawning:** When spawning Task agents, include the absolute worktree path in the prompt context so agents operate in the correct directory.

---

## Resumability

Progress tracked via **spec checkboxes** and **git commits**.

**Resume:** Run `/flow <same-spec-path>` — detects existing worktree, switches to it, skips tasks already marked `[x]`.

**State sources:**
- Worktree: if `.worktrees/<module>/` exists, switch to it (don't recreate)
- Spec file: `- [x]` = done, `- [ ]` = pending
- Git log: `git log --oneline feat/<module>` shows completed work
