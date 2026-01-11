<!--
  PrOACTTreeView - Hierarchical display of cycle branches with PrOACT letter status.

  Shows the primary cycle and all its branches in a tree structure.
  Each node displays the PrOACT letter statuses (P-r-O-A-C-T) for visualization.
-->

<script lang="ts">
	import type { PrOACTTreeNode, PrOACTLetter, LetterStatus } from '../domain/types';

	interface Props {
		/** The root node of the tree */
		tree: PrOACTTreeNode | null;
		/** Currently selected cycle ID */
		selectedCycleId?: string | null;
		/** Called when a cycle is selected */
		onSelect?: (cycleId: string) => void;
	}

	let { tree, selectedCycleId = null, onSelect }: Props = $props();

	const letters: PrOACTLetter[] = ['P', 'R', 'O', 'A', 'C', 'T'];

	const letterNames: Record<PrOACTLetter, string> = {
		P: 'Problem Frame',
		R: 'Objectives (Really matters)',
		O: 'Options/Alternatives',
		A: 'Analysis/Consequences',
		C: 'Clear Tradeoffs',
		T: 'Think Through (Recommendation + DQ)'
	};

	function getStatusIcon(status: LetterStatus): string {
		switch (status) {
			case 'completed':
				return '●';
			case 'in_progress':
				return '◉';
			case 'not_started':
				return '○';
		}
	}

	function getStatusClass(status: LetterStatus): string {
		return `status-${status}`;
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

	function getBranchLabel(node: PrOACTTreeNode): string {
		if (node.label) return node.label;
		if (!node.branch_point) return 'Primary Cycle';
		return `Branch at ${node.branch_point}`;
	}
</script>

{#if tree}
	<div class="proact-tree" role="tree" aria-label="PrOACT cycle tree">
		<div class="tree-node" role="treeitem" aria-expanded={tree.children.length > 0}>
			<button
				type="button"
				class="node-button"
				class:node-button--selected={tree.cycle_id === selectedCycleId}
				onclick={() => handleSelect(tree.cycle_id)}
				onkeydown={(e) => handleKeyDown(e, tree.cycle_id)}
			>
				<div class="node-header">
					<span class="node-label">{getBranchLabel(tree)}</span>
					{#if tree.branch_point}
						<span class="branch-badge">Branch at {tree.branch_point}</span>
					{/if}
				</div>

				<div class="letter-status">
					{#each letters as letter}
						{@const status = tree.letter_statuses[letter.toLowerCase()]}
						<div
							class="letter {getStatusClass(status)}"
							title="{letterNames[letter]}: {status.replace('_', ' ')}"
						>
							<span class="letter-icon">{getStatusIcon(status)}</span>
							<span class="letter-label">{letter}</span>
						</div>
					{/each}
				</div>

				<div class="node-footer">
					<span class="updated-at">{new Date(tree.updated_at).toLocaleDateString()}</span>
				</div>
			</button>

			{#if tree.children.length > 0}
				<ul class="tree-children" role="group">
					{#each tree.children as child}
						<li>
							<svelte:self tree={child} {selectedCycleId} {onSelect} />
						</li>
					{/each}
				</ul>
			{/if}
		</div>
	</div>
{:else}
	<div class="tree-empty">
		<p>No cycles yet</p>
		<p class="empty-subtitle">Create your first decision cycle to get started</p>
	</div>
{/if}

<style>
	.proact-tree {
		font-size: 0.875rem;
	}

	.tree-node {
		position: relative;
	}

	.node-button {
		display: block;
		width: 100%;
		padding: 1rem;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		text-align: left;
		cursor: pointer;
		transition: all 0.2s;
	}

	.node-button:hover {
		border-color: #4f46e5;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
	}

	.node-button--selected {
		border-color: #4f46e5;
		box-shadow: 0 0 0 3px rgba(79, 70, 229, 0.2);
		background: #f5f5ff;
	}

	.node-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.75rem;
		gap: 0.5rem;
	}

	.node-label {
		font-weight: 600;
		color: #1f2937;
		font-size: 1rem;
	}

	.branch-badge {
		padding: 0.25rem 0.5rem;
		border-radius: 6px;
		font-size: 0.75rem;
		font-weight: 500;
		background: #fef3c7;
		color: #92400e;
	}

	.letter-status {
		display: flex;
		gap: 0.5rem;
		align-items: center;
		padding: 0.5rem 0;
	}

	.letter {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.125rem;
		flex: 1;
	}

	.letter-icon {
		font-size: 1.25rem;
		line-height: 1;
	}

	.letter-label {
		font-size: 0.75rem;
		font-weight: 600;
		text-transform: uppercase;
	}

	.status-completed {
		color: #047857;
	}

	.status-in_progress {
		color: #d97706;
	}

	.status-not_started {
		color: #9ca3af;
	}

	.node-footer {
		margin-top: 0.5rem;
		padding-top: 0.5rem;
		border-top: 1px solid #f3f4f6;
		display: flex;
		justify-content: flex-end;
	}

	.updated-at {
		font-size: 0.75rem;
		color: #6b7280;
	}

	.tree-children {
		list-style: none;
		margin: 0;
		padding: 0;
		margin-left: 2rem;
		margin-top: 0.75rem;
		border-left: 2px solid #e5e7eb;
		padding-left: 1.25rem;
	}

	.tree-children li {
		margin-top: 0.75rem;
		position: relative;
	}

	.tree-children li::before {
		content: '';
		position: absolute;
		left: -1.25rem;
		top: 1.5rem;
		width: 1rem;
		height: 2px;
		background: #e5e7eb;
	}

	.tree-empty {
		padding: 3rem 2rem;
		text-align: center;
		background: #f9fafb;
		border-radius: 8px;
		color: #6b7280;
	}

	.tree-empty p {
		margin: 0;
		font-size: 1rem;
		font-weight: 500;
		color: #374151;
	}

	.empty-subtitle {
		margin-top: 0.5rem;
		font-size: 0.875rem;
		color: #9ca3af;
	}
</style>
