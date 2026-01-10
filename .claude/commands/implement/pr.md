# PR - Create Pull Request

Create a pull request with verification and proper formatting.

## Usage

```
/pr [options]
/pr --from-feature features/user-auth.md
/pr --draft
/pr --base <branch>
```

| Option | Description |
|--------|-------------|
| `--from-feature <file>` | Generate PR from feature file |
| `--draft` | Create as draft PR |
| `--no-push` | Preview only, don't create |
| `--base <branch>` | Target branch (default: from workflow state or main) |

---

## Prerequisites (All Required)

1. `/lint` - Code quality passes
2. `/test` - All tests pass
3. `/security-review --pr` - No CRITICAL/HIGH findings
4. `/code-simplifier` - No unnecessary complexity
5. `/checklist-sync <module>` - Requirements synced
6. Branch up to date with base
7. All changes committed

---

## Base Branch Resolution

| Priority | Source |
|----------|--------|
| 1 | Explicit `--base` argument |
| 2 | Workflow state (from `/dev`) |
| 3 | Default: `main` |

---

## PR Title Format

```
[<scope>] <Brief description>
```

Examples: `[auth] Add user registration`, `[cart] Implement checkout`

---

## PR Body Template

```markdown
## Summary
<Description from feature file or commits>

## Changes
- Task 1
- Task 2

## Requirements Progress
| Module | Files | Tests | Progress |
|--------|-------|-------|----------|
| session | 12/45 | 28/85 | 30% |

## Checklist
- [x] Tests passing
- [x] Lint passing
- [x] Security review passed
- [x] Code simplification reviewed
- [x] Requirements synced
```

---

## Size Guidelines

| Size | Lines | Recommendation |
|------|-------|----------------|
| XS-M | < 500 | Good |
| L | 500-1000 | Consider splitting |
| XL | > 1000 | Must split |

---

## Output

```
🚀 Creating Pull Request

Pre-flight checks:
  ✅ Tests passing
  ✅ Lint passing
  ✅ Security review passed
  ✅ Code simplification passed
  ✅ Requirements synced

Branch: feat/user-auth → main
Title: [user-auth] User Authentication

✅ PR #42 created: https://github.com/user/repo/pull/42
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `gh: not found` | Install: `brew install gh` then `gh auth login` |
| PR creation fails | Check `gh auth status` and push branch first |
