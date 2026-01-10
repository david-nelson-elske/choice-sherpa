# Git Conventions

## Commit Message Format

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

### Types

| Type | Use For |
|------|---------|
| feat | New functionality |
| fix | Bug fixes |
| refactor | Restructure without behavior change |
| test | Add/update tests |
| docs | Documentation only |
| style | Formatting, no code change |
| perf | Performance improvement |
| chore | Build, tooling, dependencies |

### Scope

Module or area affected: `session`, `cycle`, `api`, `auth`, etc.

### Subject

- Imperative mood: "add feature" not "added feature"
- Lowercase
- No period at end
- Max 50 characters

### Examples

```
feat(cycle): add component branching support

fix(session): prevent duplicate session creation

refactor(foundation): extract Money value object

test(cycle): add coverage for edge cases

docs(api): update OpenAPI spec for new endpoints

chore(deps): update sqlx to 0.7
```

### Body (Optional)

- Wrap at 72 characters
- Explain *what* and *why*, not *how*
- Separate from subject with blank line

```
fix(session): prevent race condition in session creation

The session creation endpoint could create duplicate sessions when
called rapidly. Added a unique constraint check before insert and
wrapped in a transaction.

Closes #123
```

## Branch Naming

```
<type>/<module>[-description]
```

| Type | Use For |
|------|---------|
| feat/ | New features |
| fix/ | Bug fixes |
| refactor/ | Refactoring |
| chore/ | Maintenance |

### Examples

```
feat/session
feat/cycle-branching
fix/auth-token-refresh
refactor/foundation-money
chore/update-dependencies
```

## PR Title Format

Same as commit message subject:
```
feat(cycle): add component branching support
```

## PR Body Template

```markdown
## Summary
- Brief description of changes
- Key decisions made

## Test Plan
- [ ] Unit tests added/updated
- [ ] Integration tests if applicable
- [ ] Manual testing steps

## Related
- Closes #123
- Related to #456
```

## Worktree Conventions

```bash
# Create worktree for module
git worktree add .worktrees/<module> feat/<module>

# List worktrees
git worktree list

# Remove after merge
git worktree remove .worktrees/<module>
```

## Merge Strategy

- Squash merge for feature branches
- Rebase for small fixes
- Never force-push to main

## Co-Author Attribution

```
Co-Authored-By: Claude <noreply@anthropic.com>
```
