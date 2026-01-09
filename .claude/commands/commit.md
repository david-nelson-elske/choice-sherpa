# Commit - Create Git Commit

Create an atomic git commit with proper formatting and message conventions.

## Usage
```
/commit [message]
/commit "add user validation"
/commit                        # Auto-generate message from changes
```

## Arguments
- `message`: Optional commit message. If omitted, generates from staged changes.

---

## Process

### Step 1: Check for Changes

```bash
git status
```

If no changes:
```
⚠️ No changes to commit.

Working tree is clean.
```

### Step 2: Review Changes

Display summary of what will be committed:

```
📊 Changes to commit:

Staged:
  M  src/services/auth.ts
  A  src/services/auth.test.ts

Unstaged:
  M  src/utils/helpers.ts

Untracked:
  ?  src/temp.ts
```

### Step 3: Stage Changes

If there are unstaged changes, prompt:

```
Stage all changes? (Y/n)
```

Or stage specific files:
```bash
git add src/services/auth.ts src/services/auth.test.ts
```

### Step 4: Generate/Validate Message

**If message provided:**
- Validate format
- Ensure it's descriptive

**If no message:**
- Analyze staged changes
- Generate message from file names and diff

### Step 5: Create Commit

```bash
git commit -m "<type>(<scope>): <description>

🤖 Generated with Claude Code

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Commit Message Format

```
<type>(<scope>): <short description>

[optional body]

[optional footer]
```

### Types

| Type | Description | Example |
|------|-------------|---------|
| `feat` | New feature | `feat(auth): add login endpoint` |
| `fix` | Bug fix | `fix(cart): correct price calculation` |
| `refactor` | Code restructuring | `refactor(user): extract validation` |
| `test` | Add/update tests | `test(auth): add login tests` |
| `docs` | Documentation | `docs(readme): update setup instructions` |
| `style` | Formatting | `style: fix indentation` |
| `chore` | Maintenance | `chore(deps): update dependencies` |

### Scope

Derived from:
1. Feature file name (if in `/dev` workflow)
2. Primary directory changed
3. Module/component name

### Description

- Imperative mood ("add" not "added")
- No period at end
- Max 50 characters
- Lowercase

**Good:**
```
feat(auth): add password reset endpoint
fix(cart): handle empty cart edge case
refactor(user): extract email validation
```

**Bad:**
```
feat(auth): Added password reset endpoint.  # Past tense, period
fix: fixed bug                               # Vague
updated stuff                                # No type, vague
```

---

## Auto-Generated Messages

When no message provided, analyze changes:

### Single File Change
```
M  src/services/auth.ts

→ feat(auth): update auth service
```

### Test File Added
```
A  src/services/auth.test.ts

→ test(auth): add auth service tests
```

### Multiple Related Files
```
M  src/models/user.ts
A  src/models/user.test.ts

→ feat(user): add user model with tests
```

### Refactoring Pattern
```
M  src/utils/validation.ts
D  src/helpers/validate.ts

→ refactor(utils): consolidate validation logic
```

---

## TDD Commit Patterns

During TDD workflow, commits follow phases:

### After RED Phase (optional)
```bash
git commit -m "WIP: add failing test for user validation"
```

### After GREEN Phase
```bash
git commit -m "feat(user): add email validation

Implements basic email format checking."
```

### After REFACTOR Phase
```bash
git commit -m "refactor(user): extract validation constants"
```

### Complete TDD Cycle
```bash
git commit -m "feat(user): add email validation with tests

- Add User.validateEmail() method
- Add comprehensive test coverage
- Extract email regex to constants"
```

---

## Commit Checklist

Before committing, verify:

- [ ] Tests pass (`/test`)
- [ ] Lint passes (`/lint`)
- [ ] No debugging code left
- [ ] No sensitive data (passwords, keys)
- [ ] Message is clear and descriptive

---

## Safety Features

### Prevent Large Commits
```
⚠️ Large commit warning

This commit includes 25 files and 1,500+ lines.
Consider splitting into smaller, atomic commits.

Proceed anyway? (y/N)
```

### Prevent Sensitive Files
```
🚨 Security warning

These files may contain sensitive data:
  - .env
  - config/secrets.json

Remove from staging? (Y/n)
```

### Prevent Untracked Files
```
⚠️ Untracked files detected

  ? src/temp.ts
  ? debug.log

Include in commit? (y/N)
```

---

## Integration with /dev

When running inside `/dev` workflow:

1. Scope is auto-set from feature filename
2. Message generated from current task
3. Commit happens automatically after passing tests

```
# Inside /dev features/user-auth.md
# After completing task "Add password hashing utility"

📝 Committing: feat(user-auth): add password hashing utility
```

---

## Examples

### Basic Commit
```
> /commit "add user registration"

📝 Creating commit...

Staged files:
  A  src/services/auth.ts
  A  src/services/auth.test.ts

Message: feat(auth): add user registration

✅ Committed: abc1234
```

### Auto-Generated Message
```
> /commit

📝 Analyzing changes...

Staged files:
  M  src/models/cart.ts
  A  src/models/cart.test.ts

Generated message: feat(cart): update cart model with tests

Accept this message? (Y/n)

✅ Committed: def5678
```

### With Body
```
> /commit

Message: feat(payment): add Stripe integration

Add body? (y/N) y

Body:
- Add StripeService for payment processing
- Add webhook handler for payment events
- Add idempotency key support

✅ Committed: ghi9012
```

---

## Amend Previous Commit

```
/commit --amend
```

**Safety checks:**
- Only amend if commit is not pushed
- Only amend commits made in current session
- Warn if commit message will change

---

## Output

```
📝 Creating commit...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
feat(auth): add password hashing utility

🤖 Generated with Claude Code

Co-Authored-By: Claude <noreply@anthropic.com>
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Files:
  A  src/utils/password.ts
  A  src/utils/password.test.ts

✅ Committed: abc1234

Branch: feat/user-auth
Ahead of origin by 3 commits
```

---

## See Also

- `/dev` - Feature-driven development (auto-commits)
- `/pr` - Create pull request
- `/lint` - Pre-commit quality check
- `/test` - Pre-commit test verification
