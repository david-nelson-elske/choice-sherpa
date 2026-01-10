# Commit - Create Git Commit

Create an atomic commit with proper formatting.

## Usage

```
/commit [message]
/commit "add user validation"     # With message
/commit                           # Auto-generate from changes
/commit --amend                   # Amend last commit (if not pushed)
```

---

## Process

1. Check for changes (`git status`)
2. Review staged/unstaged changes
3. Stage changes if needed
4. Generate or validate message
5. Create commit

---

## Message Format

```
<type>(<scope>): <description>

[optional body]

Co-Authored-By: Claude <noreply@anthropic.com>
```

### Types

| Type | Use For |
|------|---------|
| `feat` | New functionality |
| `fix` | Bug fixes |
| `refactor` | Code restructuring |
| `test` | Tests only |
| `docs` | Documentation |
| `chore` | Maintenance |

### Description Rules

- Imperative mood: "add" not "added"
- Lowercase, no period
- Max 50 characters

---

## Auto-Generation

| Changes | Generated Message |
|---------|-------------------|
| Single file `auth.ts` | `feat(auth): update auth service` |
| Test file added | `test(auth): add auth service tests` |
| Multiple related files | `feat(user): add user model with tests` |

---

## Safety Checks

| Check | Action |
|-------|--------|
| 25+ files | Warn about large commit |
| `.env`, secrets | Block staging |
| Untracked files | Prompt before including |

---

## Pre-Commit Sequence

```
/lint → /test → /commit
```

---

## Output

```
📝 Creating commit...

feat(auth): add password hashing utility

Files:
  A  src/utils/password.ts
  A  src/utils/password.test.ts

✅ Committed: abc1234
Branch: feat/user-auth
```

---

## Reference

Git conventions: `.claude/lib/examples/shared/git-conventions.md`
