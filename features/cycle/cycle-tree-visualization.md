# Cycle Tree Visualization

**Module:** cycle / dashboard
**Type:** Feature Enhancement
**Priority:** P1 (Complements Decision Document)
**Status:** Specification
**Version:** 1.0.0
**Created:** 2026-01-09

---

## Executive Summary

The Cycle Tree Browser provides a visual representation of decision exploration using **PrOACT letter nodes** as the primary visual metaphor. Each node displays the 6 letters (P-r-O-A-C-T) with color-coded status, allowing users to instantly understand their decision progress and explore branches.

---

## Visual Design

### PrOACT Node Representation

```
┌─────────────────────────────────────┐
│                                     │
│     P  r  O  A  C  T               │
│     ●  ●  ●  ●  ●  ●               │
│                                     │
│   "Career Decision v1"              │
│   Updated: Jan 9, 2026              │
└─────────────────────────────────────┘
```

### Letter-to-Component Mapping

| Letter | Component | Full Name |
|--------|-----------|-----------|
| **P** | ProblemFrame | Problem Frame |
| **r** | Objectives | Objectives (what Really matters) |
| **O** | Alternatives | Options/Alternatives |
| **A** | Consequences | Analysis/Consequences |
| **C** | Tradeoffs | Clear Tradeoffs |
| **T** | Recommendation + DQ | Think Through / Decide |

**Note:** Issue Raising and Notes/Next Steps are pre/post steps, not part of core PrOACT visualization.

### Status Colors

| Status | Color | Hex | Meaning |
|--------|-------|-----|---------|
| **Completed** | Green | `#22C55E` | Component finished |
| **In Progress** | Orange | `#F97316` | Currently working |
| **Not Started** | Red/Gray | `#EF4444` / `#9CA3AF` | Not yet begun |

### Visual States

```
Initial Cycle (all not started):
    P  r  O  A  C  T
    ○  ○  ○  ○  ○  ○     (all gray/red)

Working on Problem Frame:
    P  r  O  A  C  T
    ◉  ○  ○  ○  ○  ○     (P orange, rest gray)

Completed through Objectives:
    P  r  O  A  C  T
    ●  ●  ○  ○  ○  ○     (P,r green, rest gray)

Fully completed:
    P  r  O  A  C  T
    ●  ●  ●  ●  ●  ●     (all green)
```

---

## Tree Structure

### Branching Visualization

When a user branches at a specific component, the tree shows:

```
                    ┌─────────────────┐
                    │  P r O A C T    │
                    │  ● ● ● ● ● ●    │  ← Original complete cycle
                    │  "Main Path"    │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
     Branch at O      Branch at A      Branch at C
    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │ P r O A C T│    │ P r O A C T│    │ P r O A C T│
    │ ● ● ◉ ○ ○ ○│    │ ● ● ● ◉ ○ ○│    │ ● ● ● ● ◉ ○│
    │ "What if  │    │ "Risk     │    │ "Weight   │
    │  remote?" │    │  Analysis"│    │  Balance" │
    └──────────┘    └──────────┘    └──────────┘
```

### Branch Point Indicator

The branch connection shows WHERE the branch occurred:

```
    ┌──────────────┐
    │  P r O A C T │
    │  ● ● ● ● ● ● │
    └──────┬───────┘
           │
           │ ←── Branch point: "O" (Alternatives)
           │
    ┌──────▼───────┐
    │  P r O A C T │
    │  ● ● ◉ ○ ○ ○ │  ← Inherits P,r from parent
    └──────────────┘      Starts fresh at O
```

---

## Interaction Design

### Node Actions

| Action | Trigger | Result |
|--------|---------|--------|
| **Select Node** | Click | Load that cycle's decision document |
| **Expand/Collapse** | Click chevron | Show/hide child branches |
| **Branch Here** | Right-click letter | Create new branch at that component |
| **View Diff** | Compare icon | Show differences between branches |

### Letter Actions

| Action | Trigger | Result |
|--------|---------|--------|
| **Click Letter** | Click on P/r/O/A/C/T | Navigate to that component in document |
| **Hover Letter** | Mouse over | Show component status tooltip |
| **Branch at Letter** | Right-click | Create branch starting at that point |

### Tooltip Information

```
┌─────────────────────────────────┐
│ O - Alternatives                │
│ Status: Completed ✓             │
│ 4 alternatives defined          │
│ Last updated: 2 hours ago       │
│                                 │
│ [View] [Branch Here]            │
└─────────────────────────────────┘
```

---

## Layout Options

### Vertical Tree (Default)

```
        [Root]
           │
     ┌─────┼─────┐
     │     │     │
   [v1]  [v2]  [v3]
     │
   [v1.1]
```

### Horizontal Tree (Wide screens)

```
[Root] ─┬─ [v1] ─── [v1.1]
        ├─ [v2]
        └─ [v3]
```

### Timeline View (Chronological)

```
Jan 5     Jan 7     Jan 8     Jan 9
  │         │         │         │
  ●─────────●─────────●─────────●
 Root     Branch    Branch    Current
          at O      at C
```

---

## Document Integration

### Each Node = One Decision Document

Every cycle node has its own decision document:

```
Cycle Tree                    Document View
───────────                   ─────────────
┌──────────┐                 ┌────────────────────────────┐
│ P r O A C T│   ──────────►  │ # Career Decision          │
│ ● ● ● ● ● ●│   Click        │                            │
│ "Main"    │                 │ ## Problem Frame           │
└──────────┘                 │ Decision: Should I...      │
                              │                            │
                              │ ## Objectives              │
                              │ ...                        │
                              └────────────────────────────┘
```

### Branch Creates New Document

When branching:
1. **Copy document** up to branch point
2. **Mark remaining sections** as "needs work"
3. **Link documents** for diff/comparison
4. **Track lineage** (parent document reference)

```
Parent Document (completed)     Child Document (branched at O)
───────────────────────────     ────────────────────────────
# Career Decision               # Career Decision (Branch: Remote Option)
                                > Branched from: [Main] at Alternatives

## Problem Frame ✓              ## Problem Frame ✓
[inherited content]             [inherited content - read-only]

## Objectives ✓                 ## Objectives ✓
[inherited content]             [inherited content - read-only]

## Alternatives ✓               ## Alternatives ⚠️ IN PROGRESS
- Option A: Accept              - Option A: Accept
- Option B: Stay                - Option B: Stay
- Option C: Counter             - Option C: Remote counter  ← NEW

## Consequences ✓               ## Consequences ○ NOT STARTED
[completed analysis]            [needs analysis for new option]
```

---

## Data Model Updates

### Cycle Entity Additions

```rust
pub struct Cycle {
    // ... existing fields ...

    /// Branch metadata for visualization
    branch_metadata: BranchMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchMetadata {
    /// Component where branch occurred (None for root)
    pub branch_point: Option<ComponentType>,

    /// User-provided branch label
    pub branch_label: Option<String>,

    /// Visual position hint (for tree layout)
    pub position_hint: Option<PositionHint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionHint {
    pub x: f32,
    pub y: f32,
}
```

### Cycle Tree View Model

```rust
/// View model for cycle tree visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleTreeNode {
    pub cycle_id: CycleId,
    pub label: String,
    pub branch_point: Option<PrOACTLetter>,
    pub letter_statuses: PrOACTStatus,
    pub children: Vec<CycleTreeNode>,
    pub document_id: DecisionDocumentId,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrOACTStatus {
    pub p: LetterStatus,  // Problem Frame
    pub r: LetterStatus,  // Objectives (Really matters)
    pub o: LetterStatus,  // Options/Alternatives
    pub a: LetterStatus,  // Analysis/Consequences
    pub c: LetterStatus,  // Clear Tradeoffs
    pub t: LetterStatus,  // Think Through (Recommendation + DQ)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LetterStatus {
    NotStarted,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrOACTLetter {
    P, R, O, A, C, T
}

impl PrOACTLetter {
    pub fn to_component_types(&self) -> Vec<ComponentType> {
        match self {
            PrOACTLetter::P => vec![ComponentType::ProblemFrame],
            PrOACTLetter::R => vec![ComponentType::Objectives],
            PrOACTLetter::O => vec![ComponentType::Alternatives],
            PrOACTLetter::A => vec![ComponentType::Consequences],
            PrOACTLetter::C => vec![ComponentType::Tradeoffs],
            PrOACTLetter::T => vec![
                ComponentType::Recommendation,
                ComponentType::DecisionQuality,
            ],
        }
    }
}
```

---

## Frontend Components

### CycleTree.svelte

```svelte
<script lang="ts">
  import type { CycleTreeNode } from '$lib/cycle/types';
  import PrOACTNode from './PrOACTNode.svelte';

  export let tree: CycleTreeNode;
  export let selectedCycleId: string | null = null;
  export let onSelectCycle: (id: string) => void;
  export let onBranchAt: (cycleId: string, letter: string) => void;
</script>

<div class="cycle-tree">
  <PrOACTNode
    node={tree}
    depth={0}
    {selectedCycleId}
    {onSelectCycle}
    {onBranchAt}
  />
</div>
```

### PrOACTNode.svelte

```svelte
<script lang="ts">
  import type { CycleTreeNode, LetterStatus } from '$lib/cycle/types';

  export let node: CycleTreeNode;
  export let depth: number;
  export let selectedCycleId: string | null;
  export let onSelectCycle: (id: string) => void;
  export let onBranchAt: (cycleId: string, letter: string) => void;

  const letters = ['P', 'r', 'O', 'A', 'C', 'T'] as const;

  function getStatusColor(status: LetterStatus): string {
    switch (status) {
      case 'completed': return 'bg-green-500';
      case 'in_progress': return 'bg-orange-500';
      case 'not_started': return 'bg-gray-400';
    }
  }

  function handleLetterClick(letter: string) {
    // Navigate to that section in document
  }

  function handleLetterRightClick(letter: string, e: MouseEvent) {
    e.preventDefault();
    onBranchAt(node.cycle_id, letter);
  }
</script>

<div
  class="proact-node"
  class:selected={selectedCycleId === node.cycle_id}
  on:click={() => onSelectCycle(node.cycle_id)}
>
  <div class="letters">
    {#each letters as letter, i}
      <button
        class="letter {getStatusColor(node.letter_statuses[letter.toLowerCase()])}"
        on:click|stopPropagation={() => handleLetterClick(letter)}
        on:contextmenu={(e) => handleLetterRightClick(letter, e)}
        title="{letter}: {node.letter_statuses[letter.toLowerCase()]}"
      >
        {letter}
      </button>
    {/each}
  </div>

  <div class="label">{node.label}</div>
  <div class="updated">Updated: {formatDate(node.updated_at)}</div>
</div>

{#if node.children.length > 0}
  <div class="children" style="margin-left: {depth * 24}px">
    {#each node.children as child}
      <div class="branch-line"></div>
      <svelte:self
        node={child}
        depth={depth + 1}
        {selectedCycleId}
        {onSelectCycle}
        {onBranchAt}
      />
    {/each}
  </div>
{/if}

<style>
  .proact-node {
    @apply p-4 rounded-lg border-2 cursor-pointer transition-all;
    @apply hover:border-blue-400;
  }

  .proact-node.selected {
    @apply border-blue-500 bg-blue-50;
  }

  .letters {
    @apply flex gap-1 justify-center mb-2;
  }

  .letter {
    @apply w-8 h-8 rounded-full text-white font-bold;
    @apply flex items-center justify-center;
    @apply hover:ring-2 ring-offset-1;
  }

  .label {
    @apply text-center font-medium text-gray-800;
  }

  .updated {
    @apply text-center text-xs text-gray-500;
  }

  .children {
    @apply mt-4 pl-4 border-l-2 border-gray-200;
  }
</style>
```

---

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/sessions/:id/cycle-tree` | Get tree structure for session |
| `POST` | `/api/cycles/:id/branch` | Create branch at specified letter |

### Response: Get Cycle Tree

```json
{
  "root": {
    "cycle_id": "uuid-1",
    "label": "Career Decision",
    "branch_point": null,
    "letter_statuses": {
      "p": "completed",
      "r": "completed",
      "o": "completed",
      "a": "completed",
      "c": "completed",
      "t": "completed"
    },
    "document_id": "doc-uuid-1",
    "updated_at": "2026-01-09T10:00:00Z",
    "children": [
      {
        "cycle_id": "uuid-2",
        "label": "Remote Option",
        "branch_point": "O",
        "letter_statuses": {
          "p": "completed",
          "r": "completed",
          "o": "in_progress",
          "a": "not_started",
          "c": "not_started",
          "t": "not_started"
        },
        "document_id": "doc-uuid-2",
        "updated_at": "2026-01-09T14:30:00Z",
        "children": []
      }
    ]
  }
}
```

---

## Related Documents

- [Decision Document Specification](./decision-document.md)
- [Cycle Module](../../docs/modules/cycle.md)
- [Dashboard Module](../../docs/modules/dashboard.md)

---

*Version: 1.0.0*
*Created: 2026-01-09*
