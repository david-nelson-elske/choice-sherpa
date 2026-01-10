# Dev - Feature-Driven Development

Execute features using TDD, with commits per task and PR on completion.

## Usage

```
/dev <path>                    # Process feature file or folder
/dev features/user-auth.md     # Single feature
/dev features/                 # All features in folder
/dev                           # Resume current feature
```

## Options

| Option | Description |
|--------|-------------|
| `--dry-run` | Show plan without executing |
| `--skip-pr` | Commits only, no PR |
| `--batch-pr` | One PR for all features |
| `--continue` | Skip confirmation prompts |

---

## Workflow Overview

```
Load Feature → Create Worktree → Execute Tasks → Verify → Create PR
                    ↓
              .worktrees/<module>/
                    ↓
              For each task:
                TDD (red→green→refactor) → lint → test → commit
```

---

## Feature File Format

```markdown
# Feature: <Name>

> Brief description for PR summary.

## Context
- Business rules, constraints

## Tasks
- [ ] First task
- [ ] Second task

## Acceptance Criteria
- [ ] Criterion 1
```

---

## Step-by-Step Process

### 1. Load Feature

Parse title, summary, context, tasks, and acceptance criteria.

### 2. Create Module Worktree

| Input | Module | Worktree | Branch |
|-------|--------|----------|--------|
| `features/session/create.md` | session | `.worktrees/session/` | `feat/session` |
| `features/user-auth.md` | user-auth | `.worktrees/user-auth/` | `feat/user-auth` |

### 3. Execute Each Task

For each `- [ ]` task:
1. Announce task
2. TDD cycle: RED → GREEN → REFACTOR
3. `/lint` and `/test` (must pass)
4. `/commit` with task description
5. Mark `- [x]` in feature file

### 4. Final Verification

- Run full test suite
- Verify acceptance criteria
- Check coverage

### 5. Create PR

```bash
/pr --from-feature <feature-file> --base <base-branch>
```

---

## Folder Processing

When given a folder, all features in that module share ONE worktree and ONE branch:

```
/dev features/auth/

📁 Module: auth
🌿 Branch: feat/auth
📂 Worktree: .worktrees/auth/

Features:
  1. user-registration.md (4 tasks)
  2. user-login.md (3 tasks)
  3. password-reset.md (5 tasks)

→ 12 commits clustered on feat/auth
→ Single PR for entire module
```

Processing order: alphabetical or by numeric prefix (`01-`, `02-`, etc.)


---

## Resumability

Progress tracked via:
1. **Feature checkboxes**: `- [ ]` → `- [x]`
2. **State files**: `.claude/workflow-state/active/dev-*.json`

Resume with `/dev` (no args) or `/dev <same-path>`.

---

## Error Handling

| Error | Action |
|-------|--------|
| Test failure | Fix and retry, skip task, or abort |
| Lint failure | Run `/lint --fix`, fix remaining manually |
| Blocked task | Mark as `[BLOCKED]`, continue with next |

---

## References

- TDD workflow: `/tdd`
- Git conventions: `.claude/lib/examples/shared/git-conventions.md`
