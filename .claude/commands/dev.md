# Dev - Feature-Driven Development

Execute features using TDD, with commits per task and PR on completion.

## Usage
```
/dev <path>
/dev features/user-auth.md       # Single feature file
/dev features/                   # All features in folder
/dev                             # Resume current feature
```

## Ralph Loop Integration

Use `/dev` as the prompt for autonomous development:

```bash
/ralph-loop "/dev features/" --max-iterations 100 --completion-promise "DEV_COMPLETE: All features done"
```

### Completion Signals

The skill emits these signals for Ralph loop detection:

| Signal | Meaning |
|--------|---------|
| `DEV_COMPLETE: All features done` | All features in folder processed |
| `DEV_COMPLETE: <feature-name>` | Single feature file completed |
| `DEV_BLOCKED: <reason>` | Cannot proceed, needs intervention |
| `DEV_CONTINUE` | More work remains, continue loop |

### Example: Autonomous Feature Development

```bash
# Process entire features folder unattended
/ralph-loop "/dev features/" --max-iterations 200 --completion-promise "DEV_COMPLETE: All features done"

# Process single feature
/ralph-loop "/dev features/user-auth.md" --max-iterations 50 --completion-promise "DEV_COMPLETE: user-auth"

# Process subfolder
/ralph-loop "/dev features/auth/" --max-iterations 100 --completion-promise "DEV_COMPLETE: All features done"
```

### How It Works

1. Ralph loop calls `/dev features/`
2. `/dev` processes one task (TDD cycle + commit)
3. `/dev` outputs `DEV_CONTINUE` (more tasks remain)
4. Ralph loop continues to next iteration
5. Repeat until `DEV_COMPLETE` or max iterations

### Recommended Max Iterations

| Scope | Suggested Max |
|-------|---------------|
| Single task | 10-20 |
| Single feature (5 tasks) | 50 |
| Feature folder (3 features) | 150 |
| Large folder (10+ features) | 500 |

---

## Arguments
- `path`: Path to feature file (.md) OR folder containing feature files
  - **File**: Process single feature
  - **Folder**: Process all .md files in folder sequentially

---

## Path Handling

### Single File
```
/dev features/user-auth.md
```
Processes one feature file through completion.

### Folder (Multiple Features)
```
/dev features/
/dev features/auth/
```
Processes ALL `.md` files in the folder:
1. Lists all feature files found
2. Shows completion status of each
3. Processes first incomplete feature
4. After completion, moves to next
5. Creates PR after each feature (or batch at end)

### No Argument (Resume)
```
/dev
```
Resumes the most recently active feature based on:
1. Current branch name → matching feature file
2. Last modified feature file with incomplete tasks

---

## Folder Processing

When given a folder:

```
/dev features/

📁 Found 4 feature files in features/

  ✅ features/user-model.md (5/5 tasks)
  🔄 features/user-auth.md (2/5 tasks) ← Current
  ⏳ features/user-profile.md (0/3 tasks)
  ⏳ features/user-settings.md (0/4 tasks)

Continue with: features/user-auth.md? (Y/n)
```

### Processing Order

Features are processed in order:
1. **Alphabetical** by filename (default)
2. **Or** by prefix number if present:
   ```
   features/
   ├── 01-user-model.md
   ├── 02-user-auth.md
   ├── 03-user-profile.md
   └── 04-user-settings.md
   ```

### Completion Flow

```
┌─────────────────────────────────────────────────────────────┐
│  /dev features/                                              │
│                                                              │
│  For each .md file in folder:                               │
│    │                                                         │
│    ├─→ Skip if all tasks [x] complete                       │
│    │                                                         │
│    └─→ Process feature:                                      │
│          For each task [ ]:                                  │
│            → TDD cycle (red → green → refactor)             │
│            → lint && test                                    │
│            → commit                                          │
│            → mark [x]                                        │
│                                                              │
│          When feature complete:                              │
│            → /pr --from-feature <file>                      │
│            → Move to next feature                            │
│                                                              │
│  When all features complete:                                 │
│    → Summary of all PRs created                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Feature File Format

```markdown
# Feature: <Name>

> Brief description for PR summary.

## Context
<!-- Business rules, patterns, constraints -->
- Rule 1
- Rule 2

## Tasks
- [ ] First task description
- [ ] Second task description
- [ ] Third task description

## Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2
```

---

## Single Feature Process

### Step 1: Load Feature File

Parse the feature file:

1. **Title**: Extract from `# Feature: <title>`
2. **Summary**: First paragraph/blockquote after title
3. **Context**: Content under `## Context` heading
4. **Tasks**: Checkbox items under `## Tasks`
5. **Acceptance Criteria**: Items under `## Acceptance Criteria`

```
📄 Feature: User Authentication
   2 of 5 tasks completed

   Remaining:
   - Add login method to AuthService
   - Create POST /auth/register endpoint
   - Create POST /auth/login endpoint
```

### Step 2: Create/Verify Branch

```bash
# Derive branch name from feature filename
# features/user-auth.md → feat/user-auth
git checkout -b feat/<feature-name>
```

### Step 3: Execute Each Task

For each task marked `- [ ]`:

#### a) Announce Task
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎯 Task 3/5: Add login method to AuthService
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

#### b) TDD Cycle

**RED Phase:**
- Identify test location based on task
- Write failing test
- Confirm failure for right reason

**GREEN Phase:**
- Write minimal implementation
- Confirm test passes

**REFACTOR Phase:**
- Improve code quality
- Keep tests green

#### c) Quality Checks
```bash
/lint   # Must pass
/test   # Must pass
```

#### d) Commit
```bash
/commit "feat(<scope>): <task description>"
```

#### e) Update Feature File
```markdown
- [x] Add login method to AuthService
```

### Step 4: Final Verification

When all tasks complete:
1. Run full test suite
2. Verify acceptance criteria
3. Check coverage

### Step 5: Create PR
```
/pr --from-feature <feature-file>
```

---

## Multi-Feature Session

When processing a folder:

```
> /dev features/auth/

📁 Processing folder: features/auth/

Found 3 feature files:
  1. user-registration.md (0/4 tasks)
  2. user-login.md (0/3 tasks)
  3. password-reset.md (0/5 tasks)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📄 Feature 1/3: User Registration
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[... processes all tasks ...]

✅ Feature complete: User Registration
🚀 PR #42 created

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📄 Feature 2/3: User Login
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[... processes all tasks ...]

✅ Feature complete: User Login
🚀 PR #43 created

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📄 Feature 3/3: Password Reset
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[... processes all tasks ...]

✅ Feature complete: Password Reset
🚀 PR #44 created

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎉 All features complete!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Summary:
  ✅ User Registration → PR #42
  ✅ User Login → PR #43
  ✅ Password Reset → PR #44

Total: 12 tasks, 12 commits, 3 PRs

DEV_COMPLETE: All features done
```

### Signal Output (for Ralph Loop)

After each iteration, `/dev` outputs a signal:

**After completing a task (more remain):**
```
✅ Task complete: Add login method
DEV_CONTINUE
```

**After completing a feature (more features remain):**
```
✅ Feature complete: user-auth
🚀 PR #42 created
DEV_CONTINUE
```

**After all features complete:**
```
🎉 All features complete!
DEV_COMPLETE: All features done
```

**When blocked:**
```
❌ Cannot proceed: Test failure in auth.test.ts
DEV_BLOCKED: Test failure requires manual fix
```

---

## Resumability

Progress is tracked via checkboxes in feature files:
- `- [ ]` = Not started
- `- [x]` = Completed

If interrupted:
```bash
/dev features/user-auth.md  # Resumes from first unchecked task
/dev features/              # Resumes folder from first incomplete file
/dev                        # Auto-detects and resumes
```

---

## Configuration

Read from `CLAUDE.md`:

```markdown
## Dev Workflow
- pr_per_feature: true         # Create PR after each feature (default)
- pr_batch: false              # Or batch all features into one PR
- auto_push: true              # Push after each commit
- require_tests: true          # Block commit if tests fail
- require_lint: true           # Block commit if lint fails

## Test Commands
- test_all: `npm test`
- test_coverage: `npm test -- --coverage`

## Lint Commands
- lint: `npm run lint`
```

---

## Options

```
/dev <path> [options]

Options:
  --dry-run       Show what would be done without executing
  --skip-pr       Don't create PRs (commits only)
  --batch-pr      One PR for all features in folder
  --continue      Skip confirmation prompts
```

---

## Example: Dry Run

```
> /dev features/ --dry-run

📁 Dry run for: features/

Would process 3 feature files:

1. features/user-auth.md
   Branch: feat/user-auth
   Tasks: 5
   Commits: ~5
   PR: #1

2. features/user-profile.md
   Branch: feat/user-profile
   Tasks: 3
   Commits: ~3
   PR: #2

3. features/user-settings.md
   Branch: feat/user-settings
   Tasks: 4
   Commits: ~4
   PR: #3

Estimated: 12 tasks, 12 commits, 3 PRs
```

---

## Error Handling

### Test Failure
```
❌ Tests failed

Options:
1. Fix and retry
2. Skip task (mark as [BLOCKED])
3. Abort feature

Choice:
```

### Lint Failure
```
❌ Lint errors

Run /lint --fix to auto-fix.
Remaining errors must be fixed manually.
```

### Blocked Task
If a task cannot be completed:
```markdown
- [BLOCKED] Task description - reason for blocking
```
Blocked tasks are skipped; feature can still complete.

---

## See Also

- `/tdd` - Single task TDD workflow
- `/tdd-red` - RED phase details
- `/tdd-green` - GREEN phase details
- `/tdd-refactor` - REFACTOR phase details
- `/commit` - Create commits
- `/lint` - Code quality checks
- `/test` - Test runner
- `/pr` - Pull request creation
