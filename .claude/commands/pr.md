# PR - Create Pull Request

Create a pull request with proper formatting. Can generate from feature file or commits.

## Usage
```
/pr [options]
/pr --from-feature features/user-auth.md
/pr --draft
```

## Options
- `--from-feature <file>`: Generate PR from feature file
- `--draft`: Create as draft PR
- `--no-push`: Don't push, just show PR preview
- `--base <branch>`: Target branch (default: main)

---

## Prerequisites

Before creating PR, verify all checks pass:

### 1. Tests Pass
```bash
/test
```

### 2. Lint Passes
```bash
/lint
```

### 3. Security Review Passes
```bash
/security-review --pr
```

**Required**: No CRITICAL or HIGH severity findings allowed.

### 4. Code Simplification Review
```bash
/code-simplifier
```

Review code for unnecessary complexity and suggest simplifications.

### 5. Branch is Up to Date
```bash
git fetch origin main
git rebase origin/main
```

### 6. Changes are Committed
```bash
git status  # Should be clean
```

### 7. Sync Requirements Checklist
```bash
/checklist-sync <module>
```

Sync the REQUIREMENTS checklist to reflect current filesystem state. This ensures the PR includes accurate progress metrics.

### 8. Update Implementation Summary

Ensure `REQUIREMENTS/summary.md` reflects current progress across all modules. This provides reviewers with overall project status context.

---

## Process

### Step 1: Run Verification

```bash
# Run full check suite
/lint && /test && /security-review --pr && /code-simplifier
```

All four must pass before PR creation:
- **Lint**: Code style and quality
- **Test**: Functionality verification
- **Security**: No CRITICAL/HIGH vulnerabilities
- **Simplify**: No unnecessary complexity

### Step 1.5: Sync Requirements Checklist

```bash
# Determine module from changed files and sync checklist
/checklist-sync <module>
```

This step:
1. Detects which module(s) are affected by the PR
2. Syncs the corresponding REQUIREMENTS checklist(s)
3. Extracts progress metrics for the PR description
4. Stages the updated checklist file(s) for commit

### Step 1.6: Update Implementation Summary

After syncing checklists, update the overall progress summary:

```bash
# Update REQUIREMENTS/summary.md with current progress from all checklists
```

This step:
1. Reads progress metrics from all `CHECKLIST-*.md` files
2. Updates `REQUIREMENTS/summary.md` with current file/test counts
3. Stages the updated summary for commit

The summary document provides a quick overview of implementation status across all modules for the PR reviewer.

### Step 2: Analyze Changes

```
📊 PR Analysis

Branch: feat/user-auth → main
Commits: 5
Files changed: 8
Lines: +324, -12

Changes by type:
  src/         6 files  (+280, -10)
  tests/       2 files  (+44, -2)
```

### Step 3: Generate PR Content

**From Feature File (`--from-feature`):**

Reads the feature file to generate:
- Title from `# Feature:` heading
- Summary from description/blockquote
- Task list from `## Tasks` checkboxes
- Checklist from `## Acceptance Criteria`

**From Commits (default):**

- Title from branch name: `feat/user-auth` → `[user-auth] Feature implementation`
- Body from commit messages

### Step 4: Create PR

```bash
gh pr create \
  --title "[user-auth] User Authentication" \
  --body "$(cat pr-body.md)" \
  --base main
```

---

## PR Title Format

```
[<scope>] <Brief description>
```

Examples:
- `[auth] Add user registration and login`
- `[cart] Implement add-to-cart functionality`
- `[api] Add rate limiting to endpoints`

---

## PR Body Template

```markdown
## Summary
<Brief description of what this PR accomplishes>

## Changes
<!-- Auto-generated from commits or feature file tasks -->
- Added user registration endpoint
- Added login with JWT tokens
- Added password hashing utility
- Added input validation

## Requirements Progress
<!-- Auto-generated from /checklist-sync -->
| Module | Files | Tests | Progress |
|--------|-------|-------|----------|
| session | 12/45 (27%) | 28/85 (33%) | 30% |

<details>
<summary>Files added in this PR</summary>

- ✅ `backend/src/domain/session/session.rs`
- ✅ `backend/src/domain/session/events.rs`
- ✅ `backend/src/domain/session/errors.rs`
</details>

📋 See: [REQUIREMENTS/CHECKLIST-session.md](../REQUIREMENTS/CHECKLIST-session.md)

## Testing
- [x] Unit tests added
- [x] Integration tests added (if applicable)
- [x] All tests passing locally

## Security Review
- [x] Dependency audit passed (`cargo audit`, `npm audit`)
- [x] No CRITICAL/HIGH security findings
- [x] Secrets scan passed
- [x] Access control verified

## Checklist
- [x] Code follows project style guidelines
- [x] Self-reviewed the code
- [x] Tests cover new functionality
- [x] Security review completed
- [x] Code simplification review completed
- [x] Requirements checklist synced
- [x] Implementation summary updated
- [ ] Documentation updated (if needed)

---
🤖 Generated with [Claude Code](https://claude.ai/code)
```

---

## PR from Feature File

When using `--from-feature`, the PR body is generated from the feature file:

**Feature file:**
```markdown
# Feature: User Authentication

> Allow users to register and login securely.

## Tasks
- [x] Create password hashing utility
- [x] Add register method to AuthService
- [x] Add login method to AuthService

## Acceptance Criteria
- [x] Passwords are hashed with bcrypt
- [x] JWT tokens expire in 24 hours
```

**Generated PR:**
```markdown
## Summary
Allow users to register and login securely.

## Completed Tasks
- [x] Create password hashing utility
- [x] Add register method to AuthService
- [x] Add login method to AuthService

## Acceptance Criteria
- [x] Passwords are hashed with bcrypt
- [x] JWT tokens expire in 24 hours

## Testing
- [x] All tests passing locally

---
🤖 Generated with [Claude Code](https://claude.ai/code)
```

---

## PR Size Guidelines

| Size | Lines Changed | Recommendation |
|------|---------------|----------------|
| XS | < 50 | ✅ Ideal |
| S | 50-200 | ✅ Good |
| M | 200-500 | ✅ Acceptable |
| L | 500-1000 | ⚠️ Consider splitting |
| XL | > 1000 | ❌ Split required |

If PR is too large:
```
⚠️ PR Size Warning

This PR has 1,234 lines changed (XL).
Consider splitting into smaller PRs:

Option 1: Split by layer
  - PR 1: Backend changes (600 lines)
  - PR 2: Frontend changes (634 lines)

Option 2: Split by feature
  - PR 1: Registration (500 lines)
  - PR 2: Login (734 lines)
```

---

## Draft PRs

Use `--draft` for:
- Work in progress
- Early feedback requests
- CI validation before ready

```bash
/pr --draft --from-feature features/user-auth.md
```

---

## Output

```
🚀 Creating Pull Request

Pre-flight checks:
  ✅ Tests passing (24 tests)
  ✅ Lint passing
  ✅ Security review passed (0 critical, 0 high)
  ✅ Code simplification review passed
  ✅ Branch up to date with main
  ✅ All changes committed
  ✅ Requirements checklist synced (session: 12/45 files)
  ✅ Implementation summary updated

PR Preview:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Title: [user-auth] User Authentication

## Summary
Allow users to register and login securely.

## Completed Tasks
- [x] Create password hashing utility
- [x] Add register method to AuthService
- [x] Add login method to AuthService

## Requirements Progress
| Module | Files | Tests | Progress |
|--------|-------|-------|----------|
| session | 12/45 (27%) | 28/85 (33%) | 30% |
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Creating PR...

✅ PR #42 created: https://github.com/user/repo/pull/42

Files updated:
  - REQUIREMENTS/CHECKLIST-session.md
  - REQUIREMENTS/summary.md

Next steps:
1. PR will run CI checks
2. Request review when ready
3. Address feedback
4. Merge when approved
```

---

## Troubleshooting

### "gh: command not found"

```bash
# Install GitHub CLI
# macOS
brew install gh

# Linux
sudo apt install gh

# Then authenticate
gh auth login
```

### PR Creation Fails

```bash
# Check authentication
gh auth status

# Check remote
git remote -v

# Push branch first if needed
git push -u origin $(git branch --show-current)
```

### CI Fails After PR

1. Check CI logs in GitHub
2. Fix issues locally
3. Push fixes: `git push`
4. CI will re-run automatically

---

## See Also

- `/dev` - Feature-driven development
- `/test` - Run tests
- `/lint` - Code quality checks
- `/security-review` - Security analysis
- `/code-simplifier` - Code simplification review
- `/checklist-sync` - Sync REQUIREMENTS checklists with filesystem
- `/commit` - Create commits (use commit-commands plugin)
- `docs/architecture/APPLICATION-SECURITY-STANDARD.md` - Security requirements
- `REQUIREMENTS/CHECKLIST-*.md` - Module implementation checklists
- `REQUIREMENTS/summary.md` - Overall implementation progress summary
