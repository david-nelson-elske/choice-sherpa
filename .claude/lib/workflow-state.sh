#!/bin/bash
# Workflow State Management Library for Claude Code Skills
# Source this file in skill prompts to get state management functions
#
# Usage in skill:
#   source .claude/lib/workflow-state.sh
#   workflow_init "dev" "features/auth/login.md"
#   workflow_transition "task_execution"
#   workflow_checkpoint "lint" "passed"
#   workflow_complete

set -euo pipefail

# =============================================================================
# Configuration
# =============================================================================

WORKFLOW_STATE_DIR=".claude/workflow-state"
WORKFLOW_ACTIVE_DIR="$WORKFLOW_STATE_DIR/active"
WORKFLOW_COMPLETED_DIR="$WORKFLOW_STATE_DIR/completed"
WORKFLOW_INDEX="$WORKFLOW_STATE_DIR/index.json"

# =============================================================================
# Directory Setup
# =============================================================================

workflow_ensure_dirs() {
    mkdir -p "$WORKFLOW_ACTIVE_DIR"
    mkdir -p "$WORKFLOW_COMPLETED_DIR"

    if [ ! -f "$WORKFLOW_INDEX" ]; then
        cat > "$WORKFLOW_INDEX" << 'EOF'
{
  "schema_version": "1.0",
  "updated_at": null,
  "active_workflows": []
}
EOF
    fi
}

# =============================================================================
# State File Path
# =============================================================================

workflow_state_path() {
    local skill="$1"
    local context="$2"
    local hash=$(echo "$context" | md5sum | cut -c1-8)
    echo "$WORKFLOW_ACTIVE_DIR/${skill}-${hash}.json"
}

# =============================================================================
# Initialize or Resume Workflow
# =============================================================================

# Returns: "new" or "resume"
workflow_init() {
    local skill="$1"
    local context="$2"  # Feature file path or task description
    local branch="${3:-$(git branch --show-current 2>/dev/null || echo 'main')}"

    workflow_ensure_dirs

    local state_file=$(workflow_state_path "$skill" "$context")
    export WORKFLOW_STATE_FILE="$state_file"
    export WORKFLOW_SKILL="$skill"

    if [ -f "$state_file" ]; then
        # Validate JSON
        if ! jq empty "$state_file" 2>/dev/null; then
            echo "warning: Corrupted state file, starting fresh" >&2
            rm -f "$state_file"
        else
            # Check branch match
            local state_branch=$(jq -r '.workflow.branch // "unknown"' "$state_file")
            if [ "$state_branch" != "$branch" ] && [ "$state_branch" != "unknown" ]; then
                echo "warning: State is for branch '$state_branch', current is '$branch'" >&2
            fi
            echo "resume"
            return 0
        fi
    fi

    # Create new state
    local workflow_id="${skill}-$(echo "$context" | md5sum | cut -c1-8)"
    local now=$(date -Iseconds)

    cat > "$state_file" << EOF
{
  "_schema": {
    "version": "1.0",
    "skill": "$skill"
  },
  "workflow": {
    "id": "$workflow_id",
    "status": "in_progress",
    "created_at": "$now",
    "updated_at": "$now",
    "branch": "$branch"
  },
  "context": {
    "source": "$context"
  },
  "state_machine": {
    "current_phase": "init",
    "phases": []
  },
  "tasks": [],
  "checkpoints": {},
  "history": [
    {"at": "$now", "event": "workflow_started", "context": "$context"}
  ]
}
EOF

    # Update index
    jq --arg id "$workflow_id" --arg skill "$skill" --arg ctx "$context" --arg time "$now" '
      .updated_at = $time |
      .active_workflows += [{
        "id": $id,
        "skill": $skill,
        "context": $ctx,
        "started_at": $time,
        "last_activity": $time
      }]
    ' "$WORKFLOW_INDEX" > "$WORKFLOW_INDEX.tmp" && mv "$WORKFLOW_INDEX.tmp" "$WORKFLOW_INDEX"

    echo "new"
}

# =============================================================================
# State Queries
# =============================================================================

workflow_current_phase() {
    jq -r '.state_machine.current_phase' "$WORKFLOW_STATE_FILE"
}

workflow_get() {
    local path="$1"
    jq -r "$path" "$WORKFLOW_STATE_FILE"
}

workflow_status() {
    if [ ! -f "$WORKFLOW_STATE_FILE" ]; then
        echo "none"
        return
    fi
    jq -r '.workflow.status' "$WORKFLOW_STATE_FILE"
}

# =============================================================================
# State Mutations
# =============================================================================

workflow_set() {
    local path="$1"
    local value="$2"
    local now=$(date -Iseconds)

    jq --arg val "$value" --arg time "$now" "
      $path = \$val |
      .workflow.updated_at = \$time
    " "$WORKFLOW_STATE_FILE" > "$WORKFLOW_STATE_FILE.tmp.$$" \
        && mv "$WORKFLOW_STATE_FILE.tmp.$$" "$WORKFLOW_STATE_FILE"
}

workflow_transition() {
    local new_phase="$1"
    local now=$(date -Iseconds)

    jq --arg phase "$new_phase" --arg time "$now" '
      .state_machine.current_phase = $phase |
      .workflow.updated_at = $time |
      .history += [{"at": $time, "event": "phase_transition", "to": $phase}]
    ' "$WORKFLOW_STATE_FILE" > "$WORKFLOW_STATE_FILE.tmp.$$" \
        && mv "$WORKFLOW_STATE_FILE.tmp.$$" "$WORKFLOW_STATE_FILE"
}

workflow_add_phase() {
    local name="$1"
    local status="${2:-pending}"
    local now=$(date -Iseconds)

    jq --arg name "$name" --arg status "$status" --arg time "$now" '
      .state_machine.phases += [{"name": $name, "status": $status, "added_at": $time}] |
      .workflow.updated_at = $time
    ' "$WORKFLOW_STATE_FILE" > "$WORKFLOW_STATE_FILE.tmp.$$" \
        && mv "$WORKFLOW_STATE_FILE.tmp.$$" "$WORKFLOW_STATE_FILE"
}

workflow_phase_complete() {
    local name="$1"
    local now=$(date -Iseconds)

    jq --arg name "$name" --arg time "$now" '
      .state_machine.phases = [
        .state_machine.phases[] |
        if .name == $name then .status = "completed" | .completed_at = $time else . end
      ] |
      .workflow.updated_at = $time |
      .history += [{"at": $time, "event": "phase_completed", "phase": $name}]
    ' "$WORKFLOW_STATE_FILE" > "$WORKFLOW_STATE_FILE.tmp.$$" \
        && mv "$WORKFLOW_STATE_FILE.tmp.$$" "$WORKFLOW_STATE_FILE"
}

# =============================================================================
# Task Management
# =============================================================================

workflow_add_task() {
    local description="$1"
    local now=$(date -Iseconds)

    jq --arg desc "$description" --arg time "$now" '
      .tasks += [{
        "index": (.tasks | length + 1),
        "description": $desc,
        "status": "pending",
        "added_at": $time
      }] |
      .workflow.updated_at = $time
    ' "$WORKFLOW_STATE_FILE" > "$WORKFLOW_STATE_FILE.tmp.$$" \
        && mv "$WORKFLOW_STATE_FILE.tmp.$$" "$WORKFLOW_STATE_FILE"
}

workflow_task_start() {
    local index="$1"
    local now=$(date -Iseconds)

    jq --argjson idx "$index" --arg time "$now" '
      .tasks[$idx - 1].status = "in_progress" |
      .tasks[$idx - 1].started_at = $time |
      .workflow.updated_at = $time |
      .history += [{"at": $time, "event": "task_started", "task": $idx}]
    ' "$WORKFLOW_STATE_FILE" > "$WORKFLOW_STATE_FILE.tmp.$$" \
        && mv "$WORKFLOW_STATE_FILE.tmp.$$" "$WORKFLOW_STATE_FILE"
}

workflow_task_complete() {
    local index="$1"
    local commit_sha="${2:-}"
    local now=$(date -Iseconds)

    jq --argjson idx "$index" --arg sha "$commit_sha" --arg time "$now" '
      .tasks[$idx - 1].status = "completed" |
      .tasks[$idx - 1].completed_at = $time |
      .tasks[$idx - 1].commit_sha = $sha |
      .workflow.updated_at = $time |
      .history += [{"at": $time, "event": "task_completed", "task": $idx, "commit": $sha}]
    ' "$WORKFLOW_STATE_FILE" > "$WORKFLOW_STATE_FILE.tmp.$$" \
        && mv "$WORKFLOW_STATE_FILE.tmp.$$" "$WORKFLOW_STATE_FILE"
}

workflow_current_task() {
    jq -r '.tasks | map(select(.status == "in_progress")) | .[0].index // empty' "$WORKFLOW_STATE_FILE"
}

workflow_pending_tasks() {
    jq -r '.tasks | map(select(.status == "pending")) | length' "$WORKFLOW_STATE_FILE"
}

# =============================================================================
# TDD Phase Tracking
# =============================================================================

workflow_tdd_phase() {
    local task_index="$1"
    local phase="$2"  # red, green, refactor
    local now=$(date -Iseconds)

    jq --argjson idx "$task_index" --arg phase "$phase" --arg time "$now" '
      .tasks[$idx - 1].tdd_phase = $phase |
      .tasks[$idx - 1].tdd_phases[$phase] = "in_progress" |
      .workflow.updated_at = $time |
      .history += [{"at": $time, "event": "tdd_phase", "task": $idx, "phase": $phase}]
    ' "$WORKFLOW_STATE_FILE" > "$WORKFLOW_STATE_FILE.tmp.$$" \
        && mv "$WORKFLOW_STATE_FILE.tmp.$$" "$WORKFLOW_STATE_FILE"
}

workflow_tdd_phase_complete() {
    local task_index="$1"
    local phase="$2"
    local now=$(date -Iseconds)

    jq --argjson idx "$task_index" --arg phase "$phase" --arg time "$now" '
      .tasks[$idx - 1].tdd_phases[$phase] = "completed" |
      .workflow.updated_at = $time
    ' "$WORKFLOW_STATE_FILE" > "$WORKFLOW_STATE_FILE.tmp.$$" \
        && mv "$WORKFLOW_STATE_FILE.tmp.$$" "$WORKFLOW_STATE_FILE"
}

# =============================================================================
# Checkpoints (Verification Steps)
# =============================================================================

workflow_checkpoint() {
    local name="$1"      # lint, test, security_review, code_simplifier, pr
    local status="$2"    # pending, running, passed, failed
    local details="${3:-}"
    local now=$(date -Iseconds)

    jq --arg name "$name" --arg status "$status" --arg details "$details" --arg time "$now" '
      .checkpoints[$name] = {
        "status": $status,
        "last_run": $time,
        "details": $details
      } |
      .workflow.updated_at = $time |
      .history += [{"at": $time, "event": "checkpoint", "name": $name, "status": $status}]
    ' "$WORKFLOW_STATE_FILE" > "$WORKFLOW_STATE_FILE.tmp.$$" \
        && mv "$WORKFLOW_STATE_FILE.tmp.$$" "$WORKFLOW_STATE_FILE"
}

workflow_pending_checkpoints() {
    jq -r '
      .checkpoints | to_entries |
      map(select(.value.status == "pending" or .value.status == null)) |
      .[].key
    ' "$WORKFLOW_STATE_FILE" 2>/dev/null || echo ""
}

# =============================================================================
# Workflow Completion
# =============================================================================

workflow_complete() {
    local pr_url="${1:-}"
    local now=$(date -Iseconds)

    jq --arg time "$now" --arg pr "$pr_url" '
      .workflow.status = "completed" |
      .workflow.completed_at = $time |
      .workflow.pr_url = $pr |
      .history += [{"at": $time, "event": "workflow_completed", "pr": $pr}]
    ' "$WORKFLOW_STATE_FILE" > "$WORKFLOW_STATE_FILE.tmp.$$" \
        && mv "$WORKFLOW_STATE_FILE.tmp.$$" "$WORKFLOW_STATE_FILE"

    # Move to completed
    mv "$WORKFLOW_STATE_FILE" "$WORKFLOW_COMPLETED_DIR/"

    # Update index
    local workflow_id=$(workflow_get '.workflow.id')
    jq --arg id "$workflow_id" --arg time "$now" '
      .updated_at = $time |
      .active_workflows = [.active_workflows[] | select(.id != $id)]
    ' "$WORKFLOW_INDEX" > "$WORKFLOW_INDEX.tmp" && mv "$WORKFLOW_INDEX.tmp" "$WORKFLOW_INDEX"
}

workflow_abandon() {
    local reason="${1:-abandoned}"
    local now=$(date -Iseconds)

    jq --arg time "$now" --arg reason "$reason" '
      .workflow.status = "abandoned" |
      .workflow.abandoned_at = $time |
      .workflow.abandon_reason = $reason |
      .history += [{"at": $time, "event": "workflow_abandoned", "reason": $reason}]
    ' "$WORKFLOW_STATE_FILE" > "$WORKFLOW_STATE_FILE.tmp.$$" \
        && mv "$WORKFLOW_STATE_FILE.tmp.$$" "$WORKFLOW_STATE_FILE"

    # Move to completed (for history)
    mv "$WORKFLOW_STATE_FILE" "$WORKFLOW_COMPLETED_DIR/"

    # Update index
    local workflow_id=$(workflow_get '.workflow.id')
    jq --arg id "$workflow_id" '
      .active_workflows = [.active_workflows[] | select(.id != $id)]
    ' "$WORKFLOW_INDEX" > "$WORKFLOW_INDEX.tmp" && mv "$WORKFLOW_INDEX.tmp" "$WORKFLOW_INDEX"
}

# =============================================================================
# Display Helpers
# =============================================================================

workflow_display_status() {
    if [ ! -f "$WORKFLOW_STATE_FILE" ]; then
        echo "No active workflow"
        return
    fi

    local skill=$(workflow_get '._schema.skill')
    local status=$(workflow_get '.workflow.status')
    local phase=$(workflow_current_phase)
    local created=$(workflow_get '.workflow.created_at')
    local updated=$(workflow_get '.workflow.updated_at')

    echo "Workflow: $skill"
    echo "Status: $status"
    echo "Current Phase: $phase"
    echo "Started: $created"
    echo "Last Activity: $updated"
    echo ""

    echo "Tasks:"
    jq -r '.tasks[] | "  \(if .status == "completed" then "✅" elif .status == "in_progress" then "🔄" else "⏳" end) \(.index). \(.description)"' "$WORKFLOW_STATE_FILE"

    echo ""
    echo "Checkpoints:"
    jq -r '.checkpoints | to_entries[] | "  \(if .value.status == "passed" then "✅" elif .value.status == "failed" then "❌" elif .value.status == "running" then "🔄" else "⏳" end) \(.key)"' "$WORKFLOW_STATE_FILE" 2>/dev/null || echo "  (none)"
}

workflow_display_resume_prompt() {
    if [ ! -f "$WORKFLOW_STATE_FILE" ]; then
        return 1
    fi

    local context=$(workflow_get '.context.source')
    local phase=$(workflow_current_phase)
    local tasks_done=$(jq '[.tasks[] | select(.status == "completed")] | length' "$WORKFLOW_STATE_FILE")
    local tasks_total=$(jq '.tasks | length' "$WORKFLOW_STATE_FILE")
    local updated=$(workflow_get '.workflow.updated_at')

    echo "┌────────────────────────────────────────────────────────────┐"
    echo "│  Found existing workflow state                              │"
    echo "├────────────────────────────────────────────────────────────┤"
    echo "│  Context: $context"
    echo "│  Phase: $phase"
    echo "│  Progress: $tasks_done/$tasks_total tasks"
    echo "│  Last activity: $updated"
    echo "└────────────────────────────────────────────────────────────┘"
    echo ""
    echo "Resume this workflow? [Y/n]"
}

# =============================================================================
# Cleanup
# =============================================================================

workflow_cleanup_old() {
    # Remove completed workflows older than 24 hours
    find "$WORKFLOW_COMPLETED_DIR" -type f -mtime +1 -delete 2>/dev/null || true

    # Remove stale active workflows older than 7 days
    find "$WORKFLOW_ACTIVE_DIR" -type f -mtime +7 -delete 2>/dev/null || true
}

# =============================================================================
# Context Compaction Detection
# =============================================================================

workflow_mark_compaction() {
    local now=$(date -Iseconds)

    if [ -f "$WORKFLOW_STATE_FILE" ]; then
        jq --arg time "$now" '
          .history += [{"at": $time, "event": "context_compacted", "note": "session resumed after compaction"}]
        ' "$WORKFLOW_STATE_FILE" > "$WORKFLOW_STATE_FILE.tmp.$$" \
            && mv "$WORKFLOW_STATE_FILE.tmp.$$" "$WORKFLOW_STATE_FILE"
    fi
}

# Auto-cleanup on source
workflow_cleanup_old 2>/dev/null || true
