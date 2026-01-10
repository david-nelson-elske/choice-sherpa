<!--
  CycleTree - Hierarchical display of cycle branches.

  Shows the primary cycle and all its branches in a tree structure.
  Each node displays progress and allows navigation to that cycle.
-->

<script lang="ts">
	import type { CycleTreeNode, ComponentType } from '../domain/types';
	import { COMPONENT_LABELS } from '../domain/types';
	import CycleProgress from './CycleProgress.svelte';
	import Self from './CycleTree.svelte';

	interface Props {
		/** The root node of the tree */
		tree: CycleTreeNode | null;
		/** Currently selected cycle ID */
		selectedCycleId?: string | null;
		/** Called when a cycle is selected */
		onSelect?: (cycleId: string) => void;
	}

	let { tree, selectedCycleId = null, onSelect }: Props = $props();

	function getBranchLabel(branchPoint: ComponentType | null): string {
		if (!branchPoint) return 'Primary Cycle';
		return `Branch from ${COMPONENT_LABELS[branchPoint]}`;
	}

	function handleSelect(cycleId: string) {
		onSelect?.(cycleId);
	}

	function handleKeyDown(event: KeyboardEvent, cycleId: string) {
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			handleSelect(cycleId);
		}
	}
</script>

{#if tree}
	<div class="cycle-tree" role="tree" aria-label="Cycle branches">
		<div class="tree-node" role="treeitem" aria-selected={tree.cycle.id === selectedCycleId} aria-expanded={tree.children.length > 0}>
			<button
				type="button"
				class="node-button"
				class:node-button--selected={tree.cycle.id === selectedCycleId}
				onclick={() => handleSelect(tree.cycle.id)}
				onkeydown={(e) => handleKeyDown(e, tree.cycle.id)}
			>
				<div class="node-header">
					<span class="node-label">{getBranchLabel(tree.cycle.branch_point)}</span>
					<span class="node-status node-status--{tree.cycle.status}">{tree.cycle.status}</span>
				</div>
				<CycleProgress
					percent={tree.cycle.progress_percent}
					currentStep={tree.cycle.current_step}
					size="small"
				/>
			</button>

			{#if tree.children.length > 0}
				<ul class="tree-children" role="group">
					{#each tree.children as child}
						<li>
							<Self
								tree={child}
								{selectedCycleId}
								{onSelect}
							/>
						</li>
					{/each}
				</ul>
			{/if}
		</div>
	</div>
{:else}
	<div class="tree-empty">
		<p>No cycles yet</p>
	</div>
{/if}

<style>
	.cycle-tree {
		font-size: 0.875rem;
	}

	.tree-node {
		position: relative;
	}

	.node-button {
		display: block;
		width: 100%;
		padding: 0.75rem;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		text-align: left;
		cursor: pointer;
		transition: all 0.2s;
	}

	.node-button:hover {
		border-color: #4f46e5;
	}

	.node-button--selected {
		border-color: #4f46e5;
		box-shadow: 0 0 0 2px rgba(79, 70, 229, 0.2);
	}

	.node-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.5rem;
	}

	.node-label {
		font-weight: 500;
		color: #1f2937;
	}

	.node-status {
		padding: 0.125rem 0.5rem;
		border-radius: 9999px;
		font-size: 0.75rem;
		font-weight: 500;
	}

	.node-status--active {
		background: #dbeafe;
		color: #1d4ed8;
	}

	.node-status--completed {
		background: #d1fae5;
		color: #047857;
	}

	.node-status--archived {
		background: #f3f4f6;
		color: #6b7280;
	}

	.tree-children {
		list-style: none;
		margin: 0;
		padding: 0;
		margin-left: 1.5rem;
		margin-top: 0.5rem;
		border-left: 2px solid #e5e7eb;
		padding-left: 1rem;
	}

	.tree-children li {
		margin-top: 0.5rem;
		position: relative;
	}

	.tree-children li::before {
		content: '';
		position: absolute;
		left: -1rem;
		top: 1rem;
		width: 0.75rem;
		height: 2px;
		background: #e5e7eb;
	}

	.tree-empty {
		padding: 2rem;
		text-align: center;
		background: #f9fafb;
		border-radius: 8px;
		color: #6b7280;
	}
</style>
