# Ralph Loop - Autonomous Development Controller

Execute a skill repeatedly until completion or max iterations reached.

---

## Usage

```
/ralph-loop "<skill-command>" --max-iterations N --completion-promise "<signal>"
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `skill-command` | Yes | The skill invocation to repeat (e.g., `/dev features/`) |
| `--max-iterations` | No | Maximum loop iterations (default: 100) |
| `--completion-promise` | No | Signal string that indicates completion (default: `DEV_COMPLETE`) |

---

## Examples

```bash
# Process entire roadmap
/ralph-loop "/dev features/" --max-iterations 500 --completion-promise "DEV_COMPLETE: All features done"

# Single module
/ralph-loop "/dev features/membership/" --max-iterations 200 --completion-promise "DEV_COMPLETE: All features done"

# Specific feature
/ralph-loop "/dev features/membership/subscription-state-machine.md" --max-iterations 50 --completion-promise "DEV_COMPLETE: subscription-state-machine"
```

---

## Processing Order (from DEV-ROADMAP.md)

When given a folder path, features are processed in dependency order:

1. `features/membership/subscription-state-machine.md` (23 tasks)
2. `features/membership/stripe-webhook-handling.md` (35 tasks)
3. `features/session/session-events.md` (13 tasks)
4. `features/cycle/component-status-validation.md` (9 tasks)
5. `features/proact-types/component-schemas.md` (7 tasks)
6. `features/analysis/algorithm-specifications.md` (TBD tasks)
7. `features/integrations/ai-provider-integration.md` (30 tasks)
8. `features/integrations/authentication-identity.md` (17 tasks)
9. `features/integrations/membership-access-control.md` (22 tasks)
10. `features/integrations/websocket-dashboard.md` (16 tasks)
11. `features/dashboard/consequences-table-ui.md` (9 tasks)
12. `features/dashboard/frontend-accessibility.md` (16 tasks)
13. `features/integrations/notification-service.md` (25 tasks)
14. `features/integrations/rate-limiting.md` (25 tasks)
15. `features/integrations/observability.md` (25 tasks)
16. `features/integrations/event-versioning.md` (35 tasks)
17. `features/integrations/full-proact-journey.md` (26 tasks)

---

## Workflow

```
┌─────────────────────────────────────────────────────────────┐
│  RALPH LOOP                                                 │
│                                                             │
│  iteration = 0                                              │
│  max_iterations = N                                         │
│                                                             │
│  WHILE iteration < max_iterations:                          │
│    │                                                        │
│    ├──► Execute: skill-command                             │
│    │                                                        │
│    ├──► Parse output for signals:                          │
│    │      │                                                 │
│    │      ├─ "DEV_COMPLETE" → EXIT (success)               │
│    │      ├─ "DEV_BLOCKED"  → EXIT (blocked)               │
│    │      └─ "DEV_CONTINUE" → Continue loop                │
│    │                                                        │
│    └──► iteration++                                         │
│                                                             │
│  IF iteration >= max_iterations:                            │
│    OUTPUT: "RALPH_TIMEOUT: Max iterations reached"         │
└─────────────────────────────────────────────────────────────┘
```

---

## Signal Protocol

The inner skill (`/dev`) emits signals that control loop behavior:

| Signal | Meaning | Ralph Action |
|--------|---------|--------------|
| `DEV_COMPLETE: All features done` | All work finished | Exit loop (success) |
| `DEV_COMPLETE: <feature-name>` | Single feature done | Continue to next |
| `DEV_CONTINUE` | More work remains | Continue loop |
| `DEV_BLOCKED: <reason>` | Cannot proceed | Exit loop (error) |

---

## State Management

Progress is tracked in two places:

1. **Feature file checkboxes** - `[x]` marks completed tasks
2. **Workflow state files** - `.claude/workflow-state/active/` for session state

On resume:
- Ralph loop reads feature files to determine completion status
- Skips already-completed features
- Resumes from first incomplete task

---

## Execution

When invoked, perform the following:

### Step 1: Parse Arguments

```
skill_command = $1
max_iterations = $2 or 100
completion_promise = $3 or "DEV_COMPLETE"
```

### Step 2: Read Feature Order

If `skill_command` targets a folder, read `DEV-ROADMAP.md` to get ordered feature list.

### Step 3: Loop Execution

For each iteration:
1. Invoke the skill command
2. Check output for signals
3. If `DEV_COMPLETE` or `DEV_BLOCKED` - exit
4. If `DEV_CONTINUE` - increment iteration and continue

### Step 4: Status Report

On exit, report:
- Total iterations completed
- Features processed
- PRs created
- Exit reason (complete, blocked, timeout)

---

## Example Output

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔄 RALPH LOOP - Starting autonomous development
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Command: /dev features/
Max iterations: 500
Completion promise: DEV_COMPLETE: All features done

─────────────────────────────────────────────────
📁 Feature Order (from DEV-ROADMAP.md):
─────────────────────────────────────────────────
  1. ⏳ membership/subscription-state-machine.md
  2. ⏳ membership/stripe-webhook-handling.md
  3. ⏳ session/session-events.md
  ... (17 total features)
─────────────────────────────────────────────────

[Iteration 1] Invoking: /dev features/membership/subscription-state-machine.md
  ✅ Task 1/23: pending_can_transition_to_active
  DEV_CONTINUE

[Iteration 2] Invoking: /dev features/membership/subscription-state-machine.md
  ✅ Task 2/23: pending_can_transition_to_expired
  DEV_CONTINUE

...

[Iteration 23] Invoking: /dev features/membership/subscription-state-machine.md
  ✅ Task 23/23: reactivation_after_period_end_fails
  ✅ Feature complete: subscription-state-machine
  🚀 PR #42 created
  DEV_CONTINUE

[Iteration 24] Invoking: /dev features/membership/stripe-webhook-handling.md
  ✅ Task 1/35: verify_webhook_signature
  DEV_CONTINUE

...

[Iteration 350] Invoking: /dev features/integrations/full-proact-journey.md
  ✅ Task 26/26: complete_journey_test
  ✅ Feature complete: full-proact-journey
  🚀 PR #58 created
  DEV_COMPLETE: All features done

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎉 RALPH LOOP - Complete
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Summary:
  Iterations: 350 / 500
  Features: 17 / 17
  Tasks: 328 total
  PRs: 17 created
  Duration: ~14 hours

Exit: DEV_COMPLETE: All features done
```

---

## Error Recovery

### Test Failure

```
[Iteration 47] Invoking: /dev features/session/session-events.md
  ❌ Task 5/13 failed: Tests not passing
  DEV_BLOCKED: Test failure requires manual fix

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚠️ RALPH LOOP - Blocked
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Fix the issue and resume with:
  /ralph-loop "/dev features/" --max-iterations 453
```

### Timeout

```
[Iteration 500] Invoking: /dev features/analysis/algorithm-specifications.md
  ✅ Task 15/40: scoring_algorithm_test
  DEV_CONTINUE

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⏰ RALPH LOOP - Timeout
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Max iterations (500) reached.

Progress saved. Resume with:
  /ralph-loop "/dev features/" --max-iterations 500
```

---

## See Also

- `/dev` - Feature development skill
- `DEV-ROADMAP.md` - Ordered feature list
- `.claude/workflow-state/` - State persistence
