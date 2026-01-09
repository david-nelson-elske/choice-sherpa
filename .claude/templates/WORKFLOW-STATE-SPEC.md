# Workflow State Tracking Specification

**Version:** 1.0.0
**Status:** Draft
**Purpose:** Persistent state tracking for resumable Claude Code skill workflows

---

## Problem Statement

Long-running skills like `/dev` and `/tdd` are vulnerable to context loss during:
- Session compaction (context limit reached)
- User session interruption
- Network disconnection
- Application crashes

When context is lost, the skill loses track of:
1. Current workflow phase (e.g., RED/GREEN/REFACTOR in TDD)
2. Task completion status
3. Pending verification steps (lint, test, security review, PR)
4. Accumulated decisions and context

This leads to incomplete workflows, skipped steps, and user frustration.

---

## Design Goals

| Goal | Description |
|------|-------------|
| **Resumable** | Enable exact workflow resumption from any point |
| **Skill-agnostic** | Generic system usable by any workflow skill |
| **Minimal** | Track only essential state, not full context |
| **Human-readable** | JSON format inspectable for debugging |
| **Conflict-safe** | Handle concurrent sessions gracefully |
| **Self-cleaning** | Automatic cleanup of stale/completed state |

---

## Architecture

### State Machine Model

All workflow skills are modeled as finite state machines:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Workflow State Machine                        │
│                                                                  │
│   ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐     │
│   │  INIT   │───▶│ PHASE_1 │───▶│ PHASE_2 │───▶│  DONE   │     │
│   └─────────┘    └────┬────┘    └────┬────┘    └─────────┘     │
│                       │              │                          │
│                       ▼              ▼                          │
│                  ┌─────────┐    ┌─────────┐                     │
│                  │ BLOCKED │    │ BLOCKED │                     │
│                  └─────────┘    └─────────┘                     │
└─────────────────────────────────────────────────────────────────┘
```

### File Structure

```
.claude/
└── workflow-state/
    ├── active/                     # Currently in-progress workflows
    │   ├── dev-{hash}.json         # /dev workflow state
    │   └── tdd-{hash}.json         # /tdd workflow state
    ├── completed/                  # Recently completed (24h retention)
    │   └── dev-{hash}.json
    └── index.json                  # Quick lookup of active workflows
```

### Index File (`index.json`)

Fast lookup without scanning directory:

```json
{
  "schema_version": "1.0",
  "updated_at": "2025-01-08T22:30:00Z",
  "active_workflows": [
    {
      "id": "dev-a1b2c3",
      "skill": "dev",
      "feature": "features/foundation/event-infrastructure.md",
      "started_at": "2025-01-08T20:00:00Z",
      "last_activity": "2025-01-08T22:30:00Z"
    }
  ]
}
```

---

## State File Schema

### Common Fields (All Skills)

```json
{
  "_schema": {
    "version": "1.0",
    "skill": "dev|tdd|other",
    "spec_url": ".claude/templates/WORKFLOW-STATE-SPEC.md"
  },

  "workflow": {
    "id": "dev-a1b2c3d4",
    "status": "in_progress|completed|blocked|abandoned",
    "created_at": "2025-01-08T20:00:00Z",
    "updated_at": "2025-01-08T22:30:00Z",
    "branch": "feat/event-infrastructure"
  },

  "context": {
    // Skill-specific context needed for resumption
  },

  "state_machine": {
    "current_phase": "task_execution",
    "current_step": "lint",
    "phases": [
      {"name": "init", "status": "completed"},
      {"name": "task_execution", "status": "in_progress", "progress": "5/7"},
      {"name": "verification", "status": "pending"},
      {"name": "pr_creation", "status": "pending"}
    ]
  },

  "checkpoints": {
    // Named checkpoints for verification steps
  },

  "history": [
    // Append-only log of transitions for debugging
  ]
}
```

---

## Skill-Specific Schemas

### `/dev` Workflow State

```json
{
  "_schema": {
    "version": "1.0",
    "skill": "dev"
  },

  "workflow": {
    "id": "dev-evt-infra-a1b2",
    "status": "in_progress",
    "created_at": "2025-01-08T20:00:00Z",
    "updated_at": "2025-01-08T22:30:00Z",
    "branch": "feat/event-infrastructure"
  },

  "context": {
    "feature_file": "features/foundation/event-infrastructure.md",
    "feature_title": "Event Infrastructure",
    "folder_mode": false,
    "folder_path": null,
    "folder_features": []
  },

  "state_machine": {
    "current_phase": "task_execution",
    "phases": [
      {
        "name": "load_feature",
        "status": "completed",
        "completed_at": "2025-01-08T20:01:00Z"
      },
      {
        "name": "create_branch",
        "status": "completed",
        "completed_at": "2025-01-08T20:02:00Z"
      },
      {
        "name": "task_execution",
        "status": "in_progress",
        "started_at": "2025-01-08T20:02:30Z"
      },
      {
        "name": "verification",
        "status": "pending",
        "steps": ["lint", "test", "security_review", "code_simplifier"]
      },
      {
        "name": "pr_creation",
        "status": "pending"
      }
    ]
  },

  "tasks": [
    {
      "index": 1,
      "description": "Implement EventId value object",
      "status": "completed",
      "commit_sha": "172c0b0",
      "tdd_phases": {
        "red": "completed",
        "green": "completed",
        "refactor": "completed"
      }
    },
    {
      "index": 2,
      "description": "Implement OutboxPublisher",
      "status": "in_progress",
      "commit_sha": null,
      "tdd_phases": {
        "red": "completed",
        "green": "in_progress",
        "refactor": "pending"
      }
    },
    {
      "index": 3,
      "description": "Add integration tests",
      "status": "pending"
    }
  ],

  "checkpoints": {
    "lint": {"status": "pending", "last_run": null, "passed": null},
    "test": {"status": "pending", "last_run": null, "passed": null, "count": null},
    "security_review": {"status": "pending", "last_run": null, "findings": null},
    "code_simplifier": {"status": "pending", "last_run": null, "issues": null},
    "pr_created": {"status": "pending", "pr_number": null, "pr_url": null}
  },

  "history": [
    {"at": "2025-01-08T20:00:00Z", "event": "workflow_started"},
    {"at": "2025-01-08T20:01:00Z", "event": "phase_completed", "phase": "load_feature"},
    {"at": "2025-01-08T20:02:00Z", "event": "phase_completed", "phase": "create_branch"},
    {"at": "2025-01-08T20:10:00Z", "event": "task_completed", "task": 1, "commit": "172c0b0"},
    {"at": "2025-01-08T22:30:00Z", "event": "context_compacted", "note": "session resumed"}
  ]
}
```

### `/tdd` Workflow State

```json
{
  "_schema": {
    "version": "1.0",
    "skill": "tdd"
  },

  "workflow": {
    "id": "tdd-auth-login-b2c3",
    "status": "in_progress",
    "created_at": "2025-01-08T21:00:00Z",
    "updated_at": "2025-01-08T21:15:00Z",
    "branch": "feat/user-auth"
  },

  "context": {
    "task_description": "Add login method to AuthService",
    "feature_file": "features/auth/user-login.md",
    "task_index": 2
  },

  "state_machine": {
    "current_phase": "green",
    "cycle_count": 1,
    "phases": [
      {
        "name": "red",
        "status": "completed",
        "test_file": "tests/auth_service_test.rs",
        "test_name": "test_login_with_valid_credentials",
        "failure_confirmed": true,
        "failure_reason": "AuthService::login not implemented"
      },
      {
        "name": "green",
        "status": "in_progress",
        "implementation_file": "src/services/auth_service.rs",
        "tests_passing": false,
        "last_error": "no method named `login` found"
      },
      {
        "name": "refactor",
        "status": "pending"
      },
      {
        "name": "commit",
        "status": "pending"
      }
    ]
  },

  "test_state": {
    "test_command": "cargo test login",
    "last_run": "2025-01-08T21:14:00Z",
    "last_result": "failed",
    "failing_tests": ["test_login_with_valid_credentials"],
    "passing_tests": []
  },

  "files_modified": [
    {
      "path": "tests/auth_service_test.rs",
      "phase": "red",
      "lines_added": 25
    },
    {
      "path": "src/services/auth_service.rs",
      "phase": "green",
      "lines_added": 12
    }
  ],

  "history": [
    {"at": "2025-01-08T21:00:00Z", "event": "workflow_started", "task": "Add login method"},
    {"at": "2025-01-08T21:05:00Z", "event": "phase_started", "phase": "red"},
    {"at": "2025-01-08T21:10:00Z", "event": "test_written", "file": "tests/auth_service_test.rs"},
    {"at": "2025-01-08T21:11:00Z", "event": "failure_confirmed", "reason": "not implemented"},
    {"at": "2025-01-08T21:12:00Z", "event": "phase_completed", "phase": "red"},
    {"at": "2025-01-08T21:12:00Z", "event": "phase_started", "phase": "green"}
  ]
}
```

---

## State Operations API

Skills interact with state via shell commands (since skills are markdown prompts):

### Initialize/Resume Workflow

```bash
# Check for existing workflow state
STATE_FILE=".claude/workflow-state/active/dev-$(echo "$FEATURE_FILE" | md5sum | cut -c1-8).json"

if [ -f "$STATE_FILE" ]; then
    echo "📂 Resuming workflow from saved state..."
    CURRENT_PHASE=$(jq -r '.state_machine.current_phase' "$STATE_FILE")
    CURRENT_TASK=$(jq -r '.tasks | map(select(.status == "in_progress")) | .[0].index' "$STATE_FILE")
else
    echo "🆕 Starting new workflow..."
    # Initialize state file
    mkdir -p .claude/workflow-state/active
    cat > "$STATE_FILE" << 'INIT_STATE'
    { ... initial state ... }
INIT_STATE
fi
```

### Update State (Phase Transition)

```bash
# Transition to next phase
jq --arg phase "green" --arg time "$(date -Iseconds)" '
  .state_machine.current_phase = $phase |
  .workflow.updated_at = $time |
  .history += [{"at": $time, "event": "phase_started", "phase": $phase}]
' "$STATE_FILE" > "$STATE_FILE.tmp" && mv "$STATE_FILE.tmp" "$STATE_FILE"
```

### Complete Task

```bash
# Mark task completed with commit
jq --arg idx "$TASK_INDEX" --arg sha "$COMMIT_SHA" --arg time "$(date -Iseconds)" '
  .tasks[$idx | tonumber - 1].status = "completed" |
  .tasks[$idx | tonumber - 1].commit_sha = $sha |
  .workflow.updated_at = $time |
  .history += [{"at": $time, "event": "task_completed", "task": ($idx | tonumber), "commit": $sha}]
' "$STATE_FILE" > "$STATE_FILE.tmp" && mv "$STATE_FILE.tmp" "$STATE_FILE"
```

### Record Checkpoint

```bash
# Record lint checkpoint
jq --arg time "$(date -Iseconds)" --arg passed "$LINT_PASSED" '
  .checkpoints.lint.status = "completed" |
  .checkpoints.lint.last_run = $time |
  .checkpoints.lint.passed = ($passed == "true")
' "$STATE_FILE" > "$STATE_FILE.tmp" && mv "$STATE_FILE.tmp" "$STATE_FILE"
```

### Complete Workflow

```bash
# Mark workflow complete and move to completed/
jq --arg time "$(date -Iseconds)" '
  .workflow.status = "completed" |
  .workflow.completed_at = $time
' "$STATE_FILE" > "$STATE_FILE.tmp"

mv "$STATE_FILE.tmp" ".claude/workflow-state/completed/$(basename $STATE_FILE)"

# Update index
jq --arg id "$WORKFLOW_ID" '
  .active_workflows = [.active_workflows[] | select(.id != $id)]
' .claude/workflow-state/index.json > .claude/workflow-state/index.json.tmp \
  && mv .claude/workflow-state/index.json.tmp .claude/workflow-state/index.json
```

---

## Skill Integration

### Skill Startup Sequence

Every stateful skill should start with:

```markdown
## Workflow State Check

Before proceeding, check for existing workflow state:

1. **Calculate state file path:**
   ```bash
   WORKFLOW_HASH=$(echo "${FEATURE_FILE:-$TASK_DESCRIPTION}" | md5sum | cut -c1-8)
   STATE_FILE=".claude/workflow-state/active/${SKILL_NAME}-${WORKFLOW_HASH}.json"
   ```

2. **If state file exists:**
   - Read current phase: `jq -r '.state_machine.current_phase' "$STATE_FILE"`
   - Read pending steps: `jq -r '.checkpoints | to_entries | map(select(.value.status == "pending")) | .[].key' "$STATE_FILE"`
   - Display resume summary to user
   - Continue from saved state

3. **If no state file:**
   - Initialize new state file
   - Begin workflow from start
```

### Skill Completion Sequence

Every stateful skill should end with:

```markdown
## Workflow Completion

When all phases complete:

1. **Move state to completed:**
   ```bash
   mv "$STATE_FILE" ".claude/workflow-state/completed/"
   ```

2. **Update index file**

3. **Output completion signal:**
   ```
   DEV_COMPLETE: <feature-name>
   ```
```

---

## Resume Scenarios

### Scenario 1: Context Compaction Mid-Task

**Before compaction:**
- Phase: `task_execution`
- Current task: 5 of 7
- TDD phase: `green`

**After resume:**
1. Read state file
2. Display: "Resuming task 5/7, currently in GREEN phase"
3. Continue implementation

### Scenario 2: Context Compaction Before Verification

**Before compaction:**
- Phase: `task_execution` (all tasks complete)
- Checkpoints: all pending

**After resume:**
1. Read state file
2. Display: "All tasks complete. Pending: lint, test, security_review, pr"
3. Run verification sequence

### Scenario 3: Context Compaction Mid-PR

**Before compaction:**
- Phase: `pr_creation`
- PR pushed but not created

**After resume:**
1. Read state file
2. Check git status
3. Create PR if branch pushed

---

## Cleanup Policy

### Automatic Cleanup

```bash
# Run periodically (e.g., on skill startup)
find .claude/workflow-state/completed -mtime +1 -delete  # Delete >24h old
find .claude/workflow-state/active -mtime +7 -delete     # Delete >7d stale
```

### Manual Cleanup

```bash
# User command to clear all state
rm -rf .claude/workflow-state/active/*
rm -rf .claude/workflow-state/completed/*
```

---

## Error Handling

### Corrupted State File

```bash
if ! jq empty "$STATE_FILE" 2>/dev/null; then
    echo "⚠️ Corrupted state file detected. Starting fresh."
    rm "$STATE_FILE"
    # Initialize new state
fi
```

### Stale State (Branch Mismatch)

```bash
STATE_BRANCH=$(jq -r '.workflow.branch' "$STATE_FILE")
CURRENT_BRANCH=$(git branch --show-current)

if [ "$STATE_BRANCH" != "$CURRENT_BRANCH" ]; then
    echo "⚠️ State is for branch '$STATE_BRANCH' but current branch is '$CURRENT_BRANCH'"
    echo "Options: 1) Switch to $STATE_BRANCH  2) Abandon state and start fresh"
fi
```

### Concurrent Modification

Use atomic writes:
```bash
# Write to temp file first, then atomic move
jq '...' "$STATE_FILE" > "$STATE_FILE.tmp.$$" && mv "$STATE_FILE.tmp.$$" "$STATE_FILE"
```

---

## Implementation Checklist

### Phase 1: Core Infrastructure
- [ ] Create `.claude/workflow-state/` directory structure
- [ ] Implement state file schema validation
- [ ] Add cleanup script to skill startup

### Phase 2: `/dev` Integration
- [ ] Add state initialization on `/dev` start
- [ ] Add state updates after each task commit
- [ ] Add checkpoint recording for verification steps
- [ ] Add resume detection and handling
- [ ] Add state cleanup on completion

### Phase 3: `/tdd` Integration
- [ ] Add state initialization on `/tdd` start
- [ ] Add RED phase state tracking
- [ ] Add GREEN phase state tracking
- [ ] Add REFACTOR phase state tracking
- [ ] Add resume from any TDD phase

### Phase 4: Testing
- [ ] Test resume after context compaction
- [ ] Test resume after session interruption
- [ ] Test concurrent session handling
- [ ] Test corrupted state recovery

---

## Example: Resuming After Compaction

**User invokes `/dev features/auth/` after compaction:**

```
📂 Checking for existing workflow state...

Found active workflow:
  ID: dev-auth-a1b2c3
  Feature: features/auth/user-login.md
  Started: 2 hours ago
  Last activity: 45 minutes ago

  Progress:
    ✅ Load feature (3/5 tasks complete)
    ✅ Create branch (feat/user-auth)
    🔄 Task execution:
       ✅ Task 1: Add User model (committed: abc123)
       ✅ Task 2: Add password hashing (committed: def456)
       ✅ Task 3: Add AuthService (committed: ghi789)
       🔄 Task 4: Add login method (GREEN phase - tests failing)
       ⏳ Task 5: Add logout method
    ⏳ Verification (lint, test, security, simplify)
    ⏳ PR creation

Resume from Task 4 (GREEN phase)? [Y/n]
```

**User presses Enter:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔄 Resuming: Task 4/5 - Add login method
   Phase: GREEN (make tests pass)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Last test run (45 min ago):
  ❌ test_login_with_valid_credentials

Continuing implementation...
```

---

## See Also

- `/dev` skill documentation
- `/tdd` skill documentation
- `.claude/templates/SKILL-TEMPLATE.md`
