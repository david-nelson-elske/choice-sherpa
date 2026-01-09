# Agent-Native Enrichments for the PrOACT Cycle

> Analysis based on [Agent-Native Architecture](https://every.to/guides/agent-native) principles applied to Choice Sherpa's PrOACT decision framework.

## Overview

This document proposes 5 concrete enhancements to Choice Sherpa's PrOACT cycle based on agent-native design principles:

1. **Parity** - Agents should have identical capabilities to users
2. **Granularity** - Tools should be atomic primitives, not pre-built workflows
3. **Composability** - New features emerge via prompts, not code changes
4. **Files as Interface** - Human-readable, inspectable outputs build trust
5. **Emergent Capability** - Agents accomplishing unanticipated tasks

---

## 1. Decision Document as Live Artifact

### Principle Applied
**Files as universal interface** - Agents naturally understand filesystem operations, and user-inspectable outputs build trust.

### Current State
Component outputs are JSON blobs persisted in PostgreSQL, visible only through the application UI.

### Agent-Native Enhancement
Generate and maintain a continuously-updated **`decision.md`** document that both user and agent operate on.

```markdown
# Career Decision: Should I take the VP role at StartupCo?

## Problem Frame
- **Decision Maker**: Me (with spouse input)
- **Deadline**: January 30th offer expiration
- **Constraints**: Kids in school, spouse's career...

## Objectives
| Objective | Type | Measure |
|-----------|------|---------|
| Maximize compensation | Fundamental | Total comp ($/yr) |
| Maintain work-life balance | Fundamental | Hours/week + travel % |
...

## Consequences Matrix
| Alternative | Compensation | Work-Life | Growth |
|-------------|--------------|-----------|--------|
| Accept VP | +2 (400k) | -1 (55hrs) | +2 |
| Stay current | 0 (baseline) | 0 | 0 |
...

---
*Last updated by AI: 2026-01-09 14:32*
*Analysis Quality: 78% (weakest: Clear Tradeoffs)*
```

### Benefits
- Users can **edit the document directly** (parity)
- **Export/share** for spouse consultation, advisor review
- **Version control** via git - see how thinking evolved
- Agent actions become **transparent and auditable**

### Implementation Notes
- Generate markdown from component outputs on every update
- Parse markdown edits back into structured data
- Store both formats (JSON for queries, markdown for humans)
- Consider WebDAV or similar for real-time sync

---

## 2. Atomic Decision Tools with Emergent Composition

### Principle Applied
**Granularity** - Tools should be atomic primitives. Features emerge as agents compose them, not as pre-built workflows.

### Current State
Agent behavior is defined by component-specific system prompts. The agent operates within a single component at a time.

### Agent-Native Enhancement
Define atomic **decision tools** the agent can invoke at any point:

```rust
// Atomic decision primitives
add_objective(name, measure, is_fundamental) -> ObjectiveId
add_alternative(name, description) -> AlternativeId
rate_consequence(alt_id, obj_id, rating, reasoning)
mark_dominated(alt_id, dominated_by_id, reason)
flag_uncertainty(description, resolvable: bool)
suggest_revisit(component, reason: String) // Cross-component navigation
branch_cycle(hypothesis: String) // What-if exploration
request_user_confirmation(summary: String)
```

### Emergent Behavior Examples

| Trigger | Agent Response | Tool Invocation |
|---------|----------------|-----------------|
| User mentions "cost" repeatedly | "You've mentioned cost 3 times. Should I add 'Minimize cost' as an objective?" | `add_objective()` |
| User skips consequences for one alternative | "Alternative B hasn't been evaluated yet" | `suggest_revisit(Consequences, ...)` |
| Agent discovers dominated alternative | Auto-marks and explains | `mark_dominated()` |
| User expresses uncertainty | "Should we flag this for further research?" | `flag_uncertainty()` |

### Key Insight
The agent can discover patterns you didn't anticipate, because it composes tools based on conversation signals rather than following a script.

### Implementation Notes
- Define tools as AIProvider function calls (OpenAI tools / Anthropic tool_use)
- Each tool maps to domain commands (existing CQRS pattern)
- Agent receives tool results and continues conversation
- Log all tool invocations for audit trail

---

## 3. Non-Linear Flow via Intelligent Navigation

### Principle Applied
**Emergent capability** - Rigid step order prevents agents from accomplishing unanticipated tasks. The agent should navigate based on what the decision actually needs.

### Current State
Components follow a linear order (Issue Raising → Problem Frame → ... → Decision Quality). The `validate_can_start()` method enforces sequential progression.

### Agent-Native Enhancement
Give the agent **navigation agency** to jump between components based on conversation flow:

```
User: "Actually, I just realized there's another option I hadn't considered..."

Agent: "Great catch! Let me add that to your alternatives."
       [Invokes: navigate_to(Alternatives), add_alternative(...)]

       "Now, how would this new option perform against your objectives?"
       [Invokes: navigate_to(Consequences), starts rating new row]
```

### Navigation Intelligence

| Pattern | Agent Behavior |
|---------|----------------|
| **Forward jumps** | "You seem ready to evaluate consequences - want to skip ahead?" |
| **Backward revisits** | "This tradeoff reveals we're missing an objective. Let's add it." |
| **Cross-pollination** | "This uncertainty should probably appear in Problem Frame too." |
| **Parallel work** | "Let's rate alternatives as we define them, rather than waiting." |

### Implementation Notes
- Add `suggest_navigation(target_component, reason)` tool
- Track navigation suggestions in `AgentState.pending_navigations`
- Relax `validate_can_start()` to allow agent-initiated jumps
- Preserve "visited" state separate from "completed" state
- Consider navigation history for backtracking

---

## 4. Multi-Resolution Analysis (Progressive Depth)

### Principle Applied
**Progressive disclosure** - Simple requests work immediately, power users discover endless depth (like Excel → financial models).

### Current State
Every decision gets the same full PrOACT treatment regardless of stakes or complexity.

### Agent-Native Enhancement
Adaptive analysis depth based on decision stakes and user engagement:

| Level | Name | Duration | What It Includes |
|-------|------|----------|------------------|
| 1 | **Quick Check** | 5-10 min | Issue Raising → Quick objectives brainstorm → "Here are your top 3 considerations" |
| 2 | **Standard** | 30-60 min | Full 8-step PrOACT walkthrough |
| 3 | **Deep Analysis** | 2-4 hours | Full PrOACT + sensitivity analysis + scenario planning |
| 4 | **Facilitated** | Multi-session | Multi-stakeholder, weighted objectives, probabilistic consequences |

### Agent Determines Level Via

```
Stakes Assessment:
- "How reversible is this decision?"
- "What's the financial/life impact?"
- "Who else is affected?"

Engagement Signals:
- Short answers → stay at Level 1
- Follow-up questions → escalate depth
- "Tell me more" → increase detail

Explicit Requests:
- "I want to really think this through" → Level 3+
- "Quick gut check" → Level 1

Time Constraints:
- "I need to decide by tomorrow" → compress appropriately
```

### Key Insight
Start simple, let users discover depth. Most decisions need Level 1-2. The framework adapts rather than overwhelming.

### Implementation Notes
- Add `analysis_depth: AnalysisLevel` to Session or Cycle
- Agent prompts can reference depth to adjust questioning style
- Level 1 might skip components or combine them
- Level 4 introduces additional components (stakeholder interviews, scenario branches)

---

## 5. Cross-Decision Intelligence (Context Persistence)

### Principle Applied
**Context pattern** (`context.md`) - Maintain portable session memory that persists user preferences, available resources, and guidelines.

### Current State
Each session/cycle is isolated. The agent starts fresh with no knowledge of the user's decision-making patterns.

### Agent-Native Enhancement
Maintain a persistent **Decision Profile** that evolves across sessions:

```markdown
# Decision Profile: david@example.com

## Values & Tendencies
- Consistently weights "work-life balance" in top 3 objectives
- Tends to undervalue financial risk in initial framing
- Prefers quantitative measures over qualitative when available
- Often needs prompting to consider long-term consequences

## Decision History
| Date | Decision | DQ Score | Key Objectives | Outcome |
|------|----------|----------|----------------|---------|
| 2025-03 | Career: Accept Tech Lead role | 85% | Growth, Compensation | Accepted, satisfied |
| 2025-08 | Purchase: New car vs used | 72% | Cost, Reliability | Chose used, minor regrets |

## Learned Preferences
- Prefers "rate consequences as you go" over "batch rating"
- Responds well to devil's advocate challenges
- Dislikes lengthy preambles - prefers direct questions
```

### How the Agent Uses This

| Context | Agent Behavior |
|---------|----------------|
| Past objectives | "Last time you made a career decision, you weighted 'time with family' highest. Should we start there?" |
| Known blind spots | "Your history shows you tend to undervalue financial risk. Let me push back on this consequence rating." |
| Pattern recognition | "Based on your past decisions, you might regret choosing based primarily on cost. What's your gut say?" |
| Style preferences | Skips lengthy preambles, asks direct questions |

### Privacy-First Implementation
- Store locally (user device or encrypted cloud)
- User-controlled - can view, edit, delete
- Exportable for portability
- Agent reads at session start
- No persistence without explicit permission
- GDPR/privacy compliant by design

### Implementation Notes
- New `UserProfile` entity linked to `UserId`
- Profile updated after each completed cycle (with permission)
- Agent receives profile summary in system prompt
- Consider ML-based pattern extraction vs explicit user input

---

## Summary Matrix

| Suggestion | Agent-Native Principle | Complexity | Impact | Priority |
|------------|----------------------|------------|--------|----------|
| Decision Document | Files as interface | Medium | High trust, exportability | P1 |
| Atomic Tools | Granularity + Emergent | Medium-High | Unlock unanticipated behaviors | P1 |
| Non-Linear Flow | Composability | Low-Medium | Better UX, fewer rigid walls | P2 |
| Progressive Depth | Progressive disclosure | Medium | Reach broader audience | P2 |
| Context Persistence | Context pattern | High | Long-term value, differentiation | P3 |

---

## Recommended Implementation Order

### Phase 1: Foundation (Suggestions 1 + 2)
**Decision Document + Atomic Tools**

These two work together: atomic tools modify the decision state, and the document makes those modifications visible. Start here because:
- Decision Document provides immediate user value (export, share, trust)
- Atomic Tools enable all future agent intelligence

### Phase 2: Intelligence (Suggestion 3)
**Non-Linear Navigation**

Once atomic tools exist, add navigation tools. This:
- Unlocks the "emergent capability" test from the article
- Reduces friction in natural conversation flow
- Low risk - users can still follow linear path if preferred

### Phase 3: Adaptation (Suggestion 4)
**Progressive Depth**

With navigation working, add depth adaptation. This:
- Broadens the addressable market (quick decisions → deep analysis)
- Requires understanding of which components to skip/combine at each level
- Builds on navigation infrastructure

### Phase 4: Personalization (Suggestion 5)
**Context Persistence**

Implement last because:
- Requires significant data over time to be valuable
- Privacy implications need careful design
- Highest implementation complexity
- But provides strongest long-term differentiation

---

## The Ultimate Test

From the article:

> Can an agent accomplish a task within your domain that you *didn't explicitly design*? If yes, you've built something genuinely agent-native. If no, your architecture remains too constrained.

**With these enhancements**, the Choice Sherpa agent could:
- Spontaneously create a "quick sensitivity analysis" by branching the cycle and adjusting consequence ratings
- Recognize a repeating pattern and suggest creating a "decision template" for similar future decisions
- Notice stakeholder conflicts and suggest a facilitated multi-party session
- Adapt its questioning style based on learned user preferences
- Generate a "decision retrospective" by comparing predicted vs actual outcomes

None of these would need to be explicitly designed - they would **emerge** from the composition of atomic tools guided by conversation context.

---

*Document Version: 1.0.0*
*Created: 2026-01-09*
*Based on: Agent-Native Architecture (every.to/guides/agent-native)*
