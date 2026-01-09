# Skill State Integration Guide

This guide shows how to integrate workflow state tracking into Claude Code skills.

---

## Quick Start

Add this to the beginning of any stateful skill:

```markdown
## Workflow State Management

Before proceeding, check for existing workflow state:

\`\`\`bash
source .claude/lib/workflow-state.sh

RESUME_STATUS=$(workflow_init "SKILL_NAME" "$CONTEXT_VAR")

if [ "$RESUME_STATUS" = "resume" ]; then
    workflow_display_resume_prompt
    # Wait for user confirmation, then continue from saved state
    CURRENT_PHASE=$(workflow_current_phase)
else
    # Initialize phases for new workflow
    workflow_add_phase "phase1"
    workflow_add_phase "phase2"
    workflow_transition "phase1"
fi
\`\`\`
```

---

## `/dev` Skill Integration

### At Skill Start

```markdown
## Initialize Dev Workflow

\`\`\`bash
source .claude/lib/workflow-state.sh

# Initialize with feature file as context
RESUME=$(workflow_init "dev" "$FEATURE_FILE")

if [ "$RESUME" = "resume" ]; then
    echo "📂 Found existing workflow state"
    workflow_display_status
    echo ""
    PHASE=$(workflow_current_phase)

    case "$PHASE" in
        "init"|"load_feature")
            echo "Resuming from: Load feature file"
            ;;
        "create_branch")
            echo "Resuming from: Create branch"
            ;;
        "task_execution")
            CURRENT=$(workflow_current_task)
            TOTAL=$(jq '.tasks | length' "$WORKFLOW_STATE_FILE")
            echo "Resuming from: Task $CURRENT of $TOTAL"
            # Check TDD phase
            TDD_PHASE=$(jq -r ".tasks[$CURRENT - 1].tdd_phase // \"red\"" "$WORKFLOW_STATE_FILE")
            echo "TDD Phase: $TDD_PHASE"
            ;;
        "verification")
            PENDING=$(workflow_pending_checkpoints)
            echo "Resuming from: Verification ($PENDING pending)"
            ;;
        "pr_creation")
            echo "Resuming from: PR creation"
            ;;
    esac
else
    echo "🆕 Starting new dev workflow"

    # Set up phases
    workflow_add_phase "load_feature" "pending"
    workflow_add_phase "create_branch" "pending"
    workflow_add_phase "task_execution" "pending"
    workflow_add_phase "verification" "pending"
    workflow_add_phase "pr_creation" "pending"

    # Initialize checkpoints
    workflow_checkpoint "lint" "pending"
    workflow_checkpoint "test" "pending"
    workflow_checkpoint "security_review" "pending"
    workflow_checkpoint "code_simplifier" "pending"
    workflow_checkpoint "pr" "pending"
fi
\`\`\`
```

### After Loading Feature File

```bash
# Parse tasks from feature file and add to state
while IFS= read -r task; do
    workflow_add_task "$task"
done < <(grep -E '^\s*-\s*\[\s*\]' "$FEATURE_FILE" | sed 's/.*\[\s*\]\s*//')

workflow_phase_complete "load_feature"
workflow_transition "create_branch"
```

### After Creating Branch

```bash
workflow_set '.workflow.branch' "$BRANCH_NAME"
workflow_phase_complete "create_branch"
workflow_transition "task_execution"
```

### Before Each Task (TDD Cycle)

```bash
TASK_INDEX=$(( $(jq '[.tasks[] | select(.status == "completed")] | length' "$WORKFLOW_STATE_FILE") + 1 ))
workflow_task_start "$TASK_INDEX"

# RED phase
workflow_tdd_phase "$TASK_INDEX" "red"
# ... write failing test ...
workflow_tdd_phase_complete "$TASK_INDEX" "red"

# GREEN phase
workflow_tdd_phase "$TASK_INDEX" "green"
# ... implement ...
workflow_tdd_phase_complete "$TASK_INDEX" "green"

# REFACTOR phase
workflow_tdd_phase "$TASK_INDEX" "refactor"
# ... refactor ...
workflow_tdd_phase_complete "$TASK_INDEX" "refactor"
```

### After Each Task Commit

```bash
COMMIT_SHA=$(git rev-parse HEAD)
workflow_task_complete "$TASK_INDEX" "$COMMIT_SHA"

# Check if all tasks done
if [ "$(workflow_pending_tasks)" -eq 0 ]; then
    workflow_phase_complete "task_execution"
    workflow_transition "verification"
fi
```

### Verification Phase

```bash
# Lint
workflow_checkpoint "lint" "running"
if cargo clippy -- -D warnings; then
    workflow_checkpoint "lint" "passed"
else
    workflow_checkpoint "lint" "failed" "clippy warnings"
fi

# Test
workflow_checkpoint "test" "running"
TEST_COUNT=$(cargo test 2>&1 | grep -oP '\d+(?= passed)')
if cargo test; then
    workflow_checkpoint "test" "passed" "$TEST_COUNT tests"
else
    workflow_checkpoint "test" "failed"
fi

# Security Review
workflow_checkpoint "security_review" "running"
# ... run security review agent ...
workflow_checkpoint "security_review" "passed" "No critical findings"

# Code Simplifier
workflow_checkpoint "code_simplifier" "running"
# ... run simplifier agent ...
workflow_checkpoint "code_simplifier" "passed"

workflow_phase_complete "verification"
workflow_transition "pr_creation"
```

### After PR Created

```bash
PR_URL="https://github.com/owner/repo/pull/42"
workflow_checkpoint "pr" "passed" "$PR_URL"
workflow_phase_complete "pr_creation"
workflow_complete "$PR_URL"

echo "DEV_COMPLETE: $(basename $FEATURE_FILE .md)"
```

---

## `/tdd` Skill Integration

### At Skill Start

```bash
source .claude/lib/workflow-state.sh

# Initialize with task description as context
RESUME=$(workflow_init "tdd" "$TASK_DESCRIPTION")

if [ "$RESUME" = "resume" ]; then
    PHASE=$(workflow_current_phase)
    echo "📂 Resuming TDD workflow from: $PHASE phase"

    case "$PHASE" in
        "red")
            echo "Continue writing failing test..."
            ;;
        "green")
            echo "Continue implementing to make test pass..."
            # Show last test error
            LAST_ERROR=$(workflow_get '.test_state.last_error // "unknown"')
            echo "Last error: $LAST_ERROR"
            ;;
        "refactor")
            echo "Continue refactoring..."
            ;;
        "commit")
            echo "Ready to commit..."
            ;;
    esac
else
    echo "🆕 Starting new TDD cycle"

    # Set up TDD phases
    workflow_add_phase "red" "pending"
    workflow_add_phase "green" "pending"
    workflow_add_phase "refactor" "pending"
    workflow_add_phase "commit" "pending"

    # Add single task
    workflow_add_task "$TASK_DESCRIPTION"
    workflow_task_start 1
    workflow_transition "red"
fi
```

### RED Phase

```bash
# Starting RED
workflow_tdd_phase 1 "red"

# After writing test
workflow_set '.test_state.test_file' "$TEST_FILE"
workflow_set '.test_state.test_name' "$TEST_NAME"

# After confirming failure
TEST_OUTPUT=$(cargo test "$TEST_NAME" 2>&1)
FAILURE_REASON=$(echo "$TEST_OUTPUT" | grep -A2 "FAILED" | tail -1)
workflow_set '.test_state.failure_confirmed' 'true'
workflow_set '.test_state.failure_reason' "$FAILURE_REASON"

workflow_tdd_phase_complete 1 "red"
workflow_phase_complete "red"
workflow_transition "green"
```

### GREEN Phase

```bash
# Starting GREEN
workflow_tdd_phase 1 "green"

# Track implementation file
workflow_set '.implementation_file' "$IMPL_FILE"

# After each test run
if cargo test "$TEST_NAME" 2>&1; then
    workflow_set '.test_state.tests_passing' 'true'
    workflow_tdd_phase_complete 1 "green"
    workflow_phase_complete "green"
    workflow_transition "refactor"
else
    LAST_ERROR=$(cargo test "$TEST_NAME" 2>&1 | grep "error\[" | head -1)
    workflow_set '.test_state.last_error' "$LAST_ERROR"
fi
```

### REFACTOR Phase

```bash
# Starting REFACTOR
workflow_tdd_phase 1 "refactor"

# After each refactor iteration, verify tests still pass
if cargo test "$TEST_NAME" 2>&1; then
    echo "Tests still passing after refactor"
else
    echo "WARNING: Tests failing after refactor!"
fi

# When done refactoring
workflow_tdd_phase_complete 1 "refactor"
workflow_phase_complete "refactor"
workflow_transition "commit"
```

### Commit Phase

```bash
git add -A
git commit -m "feat: $TASK_DESCRIPTION"
COMMIT_SHA=$(git rev-parse HEAD)

workflow_task_complete 1 "$COMMIT_SHA"
workflow_phase_complete "commit"
workflow_complete
```

---

## Handling Context Compaction

When a skill is resumed after context compaction, the summary should include:

```markdown
## Context Compaction Recovery

If the conversation was compacted, workflow state should be checked:

1. The state file persists across compaction
2. On resume, the skill reads the state file
3. The skill continues from the saved phase
4. A note is added to history: `workflow_mark_compaction`

The resume flow:
1. User invokes `/dev` or `/tdd`
2. Skill checks for existing state
3. If found, displays resume prompt
4. User confirms (or starts fresh)
5. Skill continues from saved checkpoint
```

---

## State File Locations

| Skill | State File Pattern |
|-------|-------------------|
| `/dev` | `.claude/workflow-state/active/dev-{hash}.json` |
| `/tdd` | `.claude/workflow-state/active/tdd-{hash}.json` |
| `/test` | Stateless (no state file) |
| `/lint` | Stateless (no state file) |
| `/pr` | Stateless (final step of other workflows) |

---

## Debugging

### View Current State

```bash
cat .claude/workflow-state/active/dev-*.json | jq .
```

### View Workflow History

```bash
jq '.history' .claude/workflow-state/active/dev-*.json
```

### Force Clear State

```bash
rm -rf .claude/workflow-state/active/*
```

### View All Active Workflows

```bash
cat .claude/workflow-state/index.json | jq '.active_workflows'
```

---

## Best Practices

1. **Update state frequently**: After each significant action, not just phase transitions
2. **Include enough context**: Store file paths, test names, error messages
3. **Use checkpoints**: For verification steps that take time
4. **Record history**: Makes debugging easier
5. **Handle errors**: Don't crash if state file is corrupted
6. **Atomic writes**: Use temp file + mv pattern

---

## Migration

For existing skills without state tracking:

1. Add state initialization at skill start
2. Add state updates at each transition point
3. Add resume logic for each phase
4. Test resumption from each phase
5. Update skill documentation
