<script lang="ts">
	import type { DocumentTreeNode } from '$lib/types/document';
	import { documentTreeStore, selectedCycleId } from '$lib/stores/document';
	import DocumentTreeNodeComponent from './DocumentTreeNode.svelte';

	/** Optional root nodes (uses store if not provided) */
	export let nodes: DocumentTreeNode[] | null = null;

	/** Currently selected cycle ID (uses store if not provided) */
	export let selectedId: string | null = null;

	/** Whether to show compact mode (fewer details) */
	export let compact = false;

	/** Called when a node is selected */
	export let onSelect: ((cycleId: string) => void) | null = null;

	// Use props or fall back to stores
	$: displayNodes = nodes ?? $documentTreeStore;
	$: currentSelection = selectedId ?? $selectedCycleId;

	/**
	 * Handle node selection - update store and call callback.
	 */
	function handleSelect(cycleId: string) {
		selectedCycleId.set(cycleId);
		if (onSelect) {
			onSelect(cycleId);
		}
	}
</script>

<div class="document-tree" class:compact>
	<div class="tree-header">
		<slot name="header">
			<h3>Document Tree</h3>
		</slot>
	</div>

	{#if displayNodes.length === 0}
		<div class="empty-state">
			<p>No documents yet</p>
			<slot name="empty-action" />
		</div>
	{:else}
		<ul class="tree-root">
			{#each displayNodes as node (node.document_id)}
				<DocumentTreeNodeComponent
					{node}
					selectedId={currentSelection}
					{compact}
					onSelect={handleSelect}
				/>
			{/each}
		</ul>
	{/if}
</div>

<style>
	.document-tree {
		font-size: 0.875rem;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius);
		overflow: hidden;
	}

	.tree-header {
		padding: 0.75rem 1rem;
		border-bottom: 1px solid var(--color-border);
		background: var(--color-background);
	}

	.tree-header h3 {
		margin: 0;
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.empty-state {
		padding: 2rem;
		text-align: center;
		color: var(--color-text-muted);
	}

	.empty-state p {
		margin-bottom: 1rem;
	}

	.tree-root {
		list-style: none;
		padding: 0.5rem;
		margin: 0;
	}

	/* Compact mode adjustments */
	.compact .tree-header {
		padding: 0.5rem 0.75rem;
	}

	.compact .tree-root {
		padding: 0.25rem;
	}
</style>
