<!--
  AlternativesPills - Display alternatives as pill badges.

  Shows alternatives as compact pills with status quo indication.
  Useful for dashboard overview and comparisons.
-->

<script lang="ts">
	import type { AlternativeSummary } from '../index';

	interface Props {
		/** List of alternatives to display */
		alternatives: AlternativeSummary[];
		/** Whether to show descriptions on hover */
		showDescriptions?: boolean;
		/** Maximum number to show before "show more" */
		maxVisible?: number;
	}

	let {
		alternatives,
		showDescriptions = true,
		maxVisible = 6
	}: Props = $props();

	let showAll = $state(false);

	const visibleAlternatives = $derived(
		showAll ? alternatives : alternatives.slice(0, maxVisible)
	);

	const hasMore = $derived(alternatives.length > maxVisible);
</script>

<div class="alternatives-pills">
	<div class="alternatives-header">
		<h3 class="alternatives-title">Alternatives</h3>
		<span class="alternatives-count">{alternatives.length}</span>
	</div>

	{#if alternatives.length === 0}
		<p class="alternatives-empty">
			No alternatives defined yet. Add alternatives in the Alternatives component.
		</p>
	{:else}
		<div class="pills-container">
			{#each visibleAlternatives as alternative}
				<div
					class="alternative-pill"
					class:alternative-pill--status-quo={alternative.is_status_quo}
					title={showDescriptions && alternative.description ? alternative.description : alternative.name}
				>
					<span class="pill-name">{alternative.name}</span>
					{#if alternative.is_status_quo}
						<span class="pill-badge">Status Quo</span>
					{/if}
				</div>
			{/each}
		</div>

		{#if hasMore}
			<button
				type="button"
				class="show-more-button"
				onclick={() => showAll = !showAll}
			>
				{showAll ? 'Show Less' : `+${alternatives.length - maxVisible} More`}
			</button>
		{/if}
	{/if}
</div>

<style>
	.alternatives-pills {
		background: white;
		border: 2px solid #e5e7eb;
		border-radius: 12px;
		padding: 1.5rem;
	}

	.alternatives-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.alternatives-title {
		font-size: 1.125rem;
		font-weight: 600;
		color: #111827;
		margin: 0;
	}

	.alternatives-count {
		padding: 0.25rem 0.5rem;
		background: #f3f4f6;
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 600;
		color: #6b7280;
	}

	.alternatives-empty {
		margin: 0;
		padding: 2rem 1rem;
		text-align: center;
		font-size: 0.875rem;
		color: #9ca3af;
		font-style: italic;
	}

	.pills-container {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
	}

	.alternative-pill {
		display: inline-flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 0.75rem;
		background: #f3f4f6;
		border: 1px solid #d1d5db;
		border-radius: 20px;
		font-size: 0.875rem;
		color: #374151;
		transition: all 0.2s;
		cursor: default;
	}

	.alternative-pill:hover {
		background: #e5e7eb;
		transform: translateY(-1px);
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
	}

	.alternative-pill--status-quo {
		background: #fef3c7;
		border-color: #fbbf24;
		color: #92400e;
	}

	.alternative-pill--status-quo:hover {
		background: #fde68a;
	}

	.pill-name {
		font-weight: 500;
	}

	.pill-badge {
		padding: 0.125rem 0.375rem;
		background: #f59e0b;
		border-radius: 10px;
		font-size: 0.625rem;
		font-weight: 700;
		color: white;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.show-more-button {
		margin-top: 0.75rem;
		padding: 0.375rem 0.75rem;
		background: white;
		border: 1px solid #d1d5db;
		border-radius: 20px;
		font-size: 0.75rem;
		font-weight: 500;
		color: #4f46e5;
		cursor: pointer;
		transition: all 0.2s;
	}

	.show-more-button:hover {
		background: #f9fafb;
		border-color: #4f46e5;
	}
</style>
