<!--
  ObjectivesList - Display list of objectives with type and measure.

  Shows objectives grouped by type (fundamental vs means) with
  their associated measures.
-->

<script lang="ts">
	import type { ObjectiveSummary } from '../index';

	interface Props {
		/** List of objectives to display */
		objectives: ObjectiveSummary[];
		/** Maximum number to show before "show more" */
		maxVisible?: number;
	}

	let {
		objectives,
		maxVisible = 5
	}: Props = $props();

	let showAll = $state(false);

	const fundamentalObjectives = $derived(
		objectives.filter(obj => obj.type === 'fundamental')
	);

	const meansObjectives = $derived(
		objectives.filter(obj => obj.type === 'means')
	);

	const visibleObjectives = $derived(
		showAll ? objectives : objectives.slice(0, maxVisible)
	);

	const hasMore = $derived(objectives.length > maxVisible);
</script>

<div class="objectives-list">
	<div class="objectives-header">
		<h3 class="objectives-title">Objectives</h3>
		<div class="objectives-count">
			<span class="count-badge count-badge--fundamental">
				{fundamentalObjectives.length} Fundamental
			</span>
			<span class="count-badge count-badge--means">
				{meansObjectives.length} Means
			</span>
		</div>
	</div>

	{#if objectives.length === 0}
		<p class="objectives-empty">
			No objectives defined yet. Add objectives in the Objectives component.
		</p>
	{:else}
		<ul class="objectives-items">
			{#each visibleObjectives as objective}
				<li class="objective-item objective-item--{objective.type}">
					<div class="objective-content">
						<div class="objective-text">{objective.text}</div>
						{#if objective.measure}
							<div class="objective-measure">
								Measure: {objective.measure}
							</div>
						{/if}
					</div>
					<span class="objective-type-badge objective-type-badge--{objective.type}">
						{objective.type === 'fundamental' ? 'F' : 'M'}
					</span>
				</li>
			{/each}
		</ul>

		{#if hasMore}
			<button
				type="button"
				class="show-more-button"
				onclick={() => showAll = !showAll}
			>
				{showAll ? 'Show Less' : `Show ${objectives.length - maxVisible} More`}
			</button>
		{/if}
	{/if}
</div>

<style>
	.objectives-list {
		background: white;
		border: 2px solid #e5e7eb;
		border-radius: 12px;
		padding: 1.5rem;
	}

	.objectives-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.objectives-title {
		font-size: 1.125rem;
		font-weight: 600;
		color: #111827;
		margin: 0;
	}

	.objectives-count {
		display: flex;
		gap: 0.5rem;
	}

	.count-badge {
		padding: 0.25rem 0.5rem;
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 600;
	}

	.count-badge--fundamental {
		background: #dbeafe;
		color: #1e40af;
	}

	.count-badge--means {
		background: #e0e7ff;
		color: #4338ca;
	}

	.objectives-empty {
		margin: 0;
		padding: 2rem 1rem;
		text-align: center;
		font-size: 0.875rem;
		color: #9ca3af;
		font-style: italic;
	}

	.objectives-items {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.objective-item {
		display: flex;
		gap: 0.75rem;
		padding: 0.75rem;
		border-radius: 8px;
		border: 1px solid;
		transition: all 0.2s;
	}

	.objective-item--fundamental {
		background: #f0f9ff;
		border-color: #bfdbfe;
	}

	.objective-item--means {
		background: #f5f3ff;
		border-color: #c7d2fe;
	}

	.objective-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.objective-text {
		font-size: 0.875rem;
		color: #374151;
		line-height: 1.4;
	}

	.objective-measure {
		font-size: 0.75rem;
		color: #6b7280;
		font-style: italic;
	}

	.objective-type-badge {
		flex-shrink: 0;
		width: 24px;
		height: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 50%;
		font-size: 0.75rem;
		font-weight: 700;
	}

	.objective-type-badge--fundamental {
		background: #2563eb;
		color: white;
	}

	.objective-type-badge--means {
		background: #4f46e5;
		color: white;
	}

	.show-more-button {
		width: 100%;
		margin-top: 0.75rem;
		padding: 0.5rem;
		background: white;
		border: 1px solid #d1d5db;
		border-radius: 6px;
		font-size: 0.875rem;
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
