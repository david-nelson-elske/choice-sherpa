<!--
  ConsequencesMatrix - Display the consequences table.

  Shows a compact matrix of alternatives vs objectives with
  Pugh ratings (-2 to +2) color-coded for easy comparison.
-->

<script lang="ts">
	import type { CompactConsequencesTable } from '../index';

	interface Props {
		/** The consequences table data */
		table: CompactConsequencesTable | null;
		/** Called when "View Full Table" is clicked */
		onViewFull?: () => void;
	}

	let {
		table,
		onViewFull
	}: Props = $props();

	const hasTable = $derived(
		table !== null &&
		table.alternatives.length > 0 &&
		table.objectives.length > 0
	);

	function getRatingClass(rating: number | null): string {
		if (rating === null) return 'rating-none';
		if (rating <= -2) return 'rating-very-negative';
		if (rating === -1) return 'rating-negative';
		if (rating === 0) return 'rating-neutral';
		if (rating === 1) return 'rating-positive';
		return 'rating-very-positive';
	}

	function getRatingSymbol(rating: number | null): string {
		if (rating === null) return '—';
		if (rating > 0) return `+${rating}`;
		return rating.toString();
	}

	function getCell(objectiveIndex: number, alternativeIndex: number) {
		if (!table) return null;
		return table.cells.find(
			c => c.objective_index === objectiveIndex && c.alternative_index === alternativeIndex
		);
	}
</script>

<div class="consequences-matrix" class:consequences-matrix--empty={!hasTable}>
	<div class="matrix-header">
		<h3 class="matrix-title">Consequences</h3>
		{#if hasTable && onViewFull}
			<button
				type="button"
				class="view-full-button"
				onclick={onViewFull}
				aria-label="View full consequences table"
			>
				View Full
			</button>
		{/if}
	</div>

	{#if hasTable}
		<div class="table-container">
			<table class="consequences-table">
				<thead>
					<tr>
						<th class="corner-cell"></th>
						{#each table.alternatives as alternative}
							<th class="alternative-header">{alternative}</th>
						{/each}
					</tr>
				</thead>
				<tbody>
					{#each table.objectives as objective, objIndex}
						<tr>
							<th class="objective-header">{objective}</th>
							{#each table.alternatives as _, altIndex}
								{@const cell = getCell(objIndex, altIndex)}
								{@const rating = cell?.rating ?? null}
								<td class="rating-cell {getRatingClass(rating)}">
									<span class="rating-value">{getRatingSymbol(rating)}</span>
								</td>
							{/each}
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<div class="legend">
			<span class="legend-title">Pugh Rating:</span>
			<div class="legend-items">
				<span class="legend-item rating-very-negative">-2</span>
				<span class="legend-item rating-negative">-1</span>
				<span class="legend-item rating-neutral">0</span>
				<span class="legend-item rating-positive">+1</span>
				<span class="legend-item rating-very-positive">+2</span>
			</div>
		</div>
	{:else}
		<p class="empty-message">
			No consequences table yet. Complete the Consequences component to evaluate alternatives.
		</p>
	{/if}
</div>

<style>
	.consequences-matrix {
		background: white;
		border: 2px solid #e5e7eb;
		border-radius: 12px;
		padding: 1.5rem;
	}

	.consequences-matrix--empty {
		border-style: dashed;
		background: #f9fafb;
	}

	.matrix-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.matrix-title {
		font-size: 1.125rem;
		font-weight: 600;
		color: #111827;
		margin: 0;
	}

	.view-full-button {
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

	.view-full-button:hover {
		background: #f9fafb;
		border-color: #4f46e5;
	}

	.table-container {
		overflow-x: auto;
		margin-bottom: 1rem;
	}

	.consequences-table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.875rem;
	}

	.corner-cell {
		background: #f9fafb;
		border: 1px solid #e5e7eb;
	}

	.alternative-header {
		padding: 0.5rem;
		background: #f3f4f6;
		border: 1px solid #e5e7eb;
		font-weight: 600;
		color: #374151;
		text-align: center;
		min-width: 80px;
	}

	.objective-header {
		padding: 0.5rem;
		background: #f3f4f6;
		border: 1px solid #e5e7eb;
		font-weight: 600;
		color: #374151;
		text-align: left;
		max-width: 200px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.rating-cell {
		padding: 0.5rem;
		border: 1px solid #e5e7eb;
		text-align: center;
		font-weight: 600;
		transition: all 0.2s;
	}

	.rating-cell:hover {
		transform: scale(1.1);
		z-index: 1;
	}

	.rating-value {
		display: inline-block;
	}

	.rating-none {
		background: #f9fafb;
		color: #9ca3af;
	}

	.rating-very-negative {
		background: #fef2f2;
		color: #dc2626;
	}

	.rating-negative {
		background: #fef3c7;
		color: #f59e0b;
	}

	.rating-neutral {
		background: #f3f4f6;
		color: #6b7280;
	}

	.rating-positive {
		background: #d1fae5;
		color: #059669;
	}

	.rating-very-positive {
		background: #a7f3d0;
		color: #047857;
	}

	.legend {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding-top: 1rem;
		border-top: 1px solid #e5e7eb;
		font-size: 0.75rem;
	}

	.legend-title {
		font-weight: 600;
		color: #6b7280;
	}

	.legend-items {
		display: flex;
		gap: 0.5rem;
	}

	.legend-item {
		padding: 0.25rem 0.5rem;
		border-radius: 4px;
		font-weight: 600;
		font-size: 0.75rem;
	}

	.empty-message {
		margin: 0;
		padding: 2rem 1rem;
		text-align: center;
		font-size: 0.875rem;
		color: #9ca3af;
		font-style: italic;
	}
</style>
