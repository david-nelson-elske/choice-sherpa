<script lang="ts">
	import type { DocumentTreeNode, ComponentStatus, ComponentType } from '$lib/types/document';

	/** The node to render */
	export let node: DocumentTreeNode;

	/** Currently selected cycle ID */
	export let selectedId: string | null = null;

	/** Whether to show compact mode */
	export let compact = false;

	/** Called when this or a child node is selected */
	export let onSelect: (cycleId: string) => void;

	/** Nesting depth for indentation */
	export let depth = 0;

	/** PrOACT component display order */
	const proactComponents: { key: ComponentType; label: string; abbrev: string }[] = [
		{ key: 'issue_raising', label: 'Issue Raising', abbrev: 'IR' },
		{ key: 'problem_frame', label: 'Problem Frame', abbrev: 'PF' },
		{ key: 'objectives', label: 'Objectives', abbrev: 'O' },
		{ key: 'alternatives', label: 'Alternatives', abbrev: 'A' },
		{ key: 'consequences', label: 'Consequences', abbrev: 'C' },
		{ key: 'tradeoffs', label: 'Tradeoffs', abbrev: 'T' },
		{ key: 'recommendation', label: 'Recommendation', abbrev: 'R' },
		{ key: 'decision_quality', label: 'Decision Quality', abbrev: 'DQ' }
	];

	$: isSelected = selectedId === node.cycle_id;

	function getStatusClass(status: ComponentStatus): string {
		switch (status) {
			case 'completed':
				return 'status-completed';
			case 'in_progress':
				return 'status-in-progress';
			default:
				return 'status-not-started';
		}
	}

	function getBranchLabel(branchPoint: ComponentType | undefined): string {
		if (!branchPoint) return '';
		const component = proactComponents.find((c) => c.key === branchPoint);
		return component ? `from ${component.abbrev}` : '';
	}
</script>

<li class="tree-node" class:selected={isSelected} class:has-children={node.children.length > 0}>
	<button
		class="node-button"
		on:click={() => onSelect(node.cycle_id)}
		aria-pressed={isSelected}
		style="--depth: {depth}"
	>
		<div class="node-header">
			<span class="node-label">{node.label}</span>
			{#if node.branch_point}
				<span class="branch-badge">{getBranchLabel(node.branch_point)}</span>
			{/if}
		</div>

		{#if !compact}
			<div class="proact-status">
				{#each proactComponents as comp (comp.key)}
					<div
						class="status-dot {getStatusClass(node.proact_status[comp.key])}"
						title="{comp.label}: {node.proact_status[comp.key]}"
					>
						<span class="status-abbrev">{comp.abbrev}</span>
					</div>
				{/each}
			</div>
		{:else}
			<div class="compact-status">
				{#each proactComponents as comp (comp.key)}
					<span
						class="status-pip {getStatusClass(node.proact_status[comp.key])}"
						title="{comp.label}: {node.proact_status[comp.key]}"
					></span>
				{/each}
			</div>
		{/if}
	</button>

	{#if node.children.length > 0}
		<ul class="tree-children">
			{#each node.children as child (child.document_id)}
				<svelte:self {onSelect} node={child} selectedId={selectedId} {compact} depth={depth + 1} />
			{/each}
		</ul>
	{/if}
</li>

<style>
	.tree-node {
		position: relative;
		list-style: none;
	}

	.tree-children {
		list-style: none;
		padding: 0;
		margin: 0;
		margin-left: 1.5rem;
		border-left: 2px solid var(--color-border);
	}

	/* Child nodes have connector lines - using :global for recursive children */
	.tree-children :global(.tree-node) {
		padding-left: 1rem;
	}

	.tree-children :global(.tree-node::before) {
		content: '';
		position: absolute;
		left: -2px;
		top: 1.25rem;
		width: 1rem;
		height: 2px;
		background: var(--color-border);
	}

	.node-button {
		display: block;
		width: 100%;
		padding: 0.75rem;
		margin: 0.25rem 0;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius);
		text-align: left;
		cursor: pointer;
		transition:
			border-color 0.15s,
			box-shadow 0.15s;
	}

	.node-button:hover {
		border-color: var(--color-primary);
	}

	.selected > .node-button {
		border-color: var(--color-primary);
		box-shadow: 0 0 0 3px rgb(37 99 235 / 0.1);
	}

	.node-header {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-bottom: 0.5rem;
	}

	.node-label {
		font-weight: 500;
		color: var(--color-text);
	}

	.branch-badge {
		font-size: 0.75rem;
		padding: 0.125rem 0.5rem;
		background: var(--color-background);
		border-radius: 9999px;
		color: var(--color-text-muted);
	}

	/* PrOACT Status Display */
	.proact-status {
		display: flex;
		gap: 0.25rem;
	}

	.status-dot {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.75rem;
		height: 1.5rem;
		border-radius: 4px;
		font-size: 0.625rem;
		font-weight: 600;
	}

	.status-abbrev {
		opacity: 0.8;
	}

	.status-completed {
		background: #dcfce7;
		color: #166534;
	}

	.status-in-progress {
		background: #fef3c7;
		color: #92400e;
	}

	.status-not-started {
		background: var(--color-background);
		color: var(--color-text-muted);
	}

	/* Compact Mode */
	.compact-status {
		display: flex;
		gap: 0.25rem;
	}

	.status-pip {
		width: 0.5rem;
		height: 0.5rem;
		border-radius: 50%;
	}
</style>
