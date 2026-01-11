<!--
  DecisionStatement - Display for the main decision statement.

  Shows the decision statement with optional edit functionality.
  If no statement is set, displays a placeholder prompt.
-->

<script lang="ts">
	interface Props {
		/** The decision statement text */
		statement: string | null;
		/** Whether editing is allowed */
		editable?: boolean;
		/** Called when edit button is clicked */
		onEdit?: () => void;
	}

	let {
		statement,
		editable = false,
		onEdit
	}: Props = $props();

	const hasStatement = $derived(statement !== null && statement.trim().length > 0);
</script>

<div class="decision-statement" class:decision-statement--empty={!hasStatement}>
	<div class="statement-header">
		<h2 class="statement-title">Decision Statement</h2>
		{#if editable && onEdit}
			<button
				type="button"
				class="edit-button"
				onclick={onEdit}
				aria-label="Edit decision statement"
			>
				Edit
			</button>
		{/if}
	</div>
	<div class="statement-content">
		{#if hasStatement}
			<p class="statement-text">{statement}</p>
		{:else}
			<p class="statement-placeholder">
				No decision statement yet. Define your decision in the Problem Frame component.
			</p>
		{/if}
	</div>
</div>

<style>
	.decision-statement {
		background: white;
		border: 2px solid #e5e7eb;
		border-radius: 12px;
		padding: 1.5rem;
		transition: border-color 0.2s;
	}

	.decision-statement--empty {
		border-style: dashed;
		background: #f9fafb;
	}

	.statement-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.statement-title {
		font-size: 1.125rem;
		font-weight: 600;
		color: #111827;
		margin: 0;
	}

	.edit-button {
		padding: 0.375rem 0.75rem;
		background: white;
		border: 1px solid #d1d5db;
		border-radius: 6px;
		font-size: 0.875rem;
		font-weight: 500;
		color: #4f46e5;
		cursor: pointer;
		transition: all 0.2s;
	}

	.edit-button:hover {
		background: #f9fafb;
		border-color: #4f46e5;
	}

	.edit-button:active {
		transform: scale(0.98);
	}

	.statement-content {
		line-height: 1.6;
	}

	.statement-text {
		margin: 0;
		font-size: 1rem;
		color: #374151;
	}

	.statement-placeholder {
		margin: 0;
		font-size: 0.875rem;
		color: #9ca3af;
		font-style: italic;
	}
</style>
