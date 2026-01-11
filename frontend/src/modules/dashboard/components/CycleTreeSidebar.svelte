<!--
  CycleTreeSidebar - Sidebar showing cycle hierarchy.

  Displays the cycle tree with branch visualization,
  allowing users to switch between cycles.
-->

<script lang="ts">
	import { getComponentLabel, type ComponentType } from '../../cycle/domain/types';

	interface CycleSummary {
		cycle_id: string;
		branch_point: ComponentType | null;
		progress_percent: number;
		is_active: boolean;
	}

	interface Props {
		/** List of cycles to display */
		cycles: CycleSummary[];
		/** Currently active cycle ID */
		activeCycleId: string | null;
		/** Called when a cycle is selected */
		onSelectCycle: (cycleId: string) => void;
	}

	let {
		cycles,
		activeCycleId,
		onSelectCycle
	}: Props = $props();

	const rootCycles = $derived(
		cycles.filter(c => c.branch_point === null)
	);

	const branchCycles = $derived(
		cycles.filter(c => c.branch_point !== null)
	);
</script>

<aside class="cycle-tree-sidebar">
	<div class="sidebar-header">
		<h2 class="sidebar-title">Cycles</h2>
		<span class="cycle-count">{cycles.length}</span>
	</div>

	<div class="sidebar-content">
		{#if cycles.length === 0}
			<p class="empty-message">No cycles yet</p>
		{:else}
			<!-- Root Cycles -->
			<div class="cycle-list">
				{#each rootCycles as cycle}
					<button
						type="button"
						class="cycle-item"
						class:cycle-item--active={cycle.cycle_id === activeCycleId}
						onclick={() => onSelectCycle(cycle.cycle_id)}
					>
						<div class="cycle-icon">
							<span class="icon-text">📊</span>
						</div>
						<div class="cycle-details">
							<span class="cycle-label">Main Cycle</span>
							<div class="progress-bar">
								<div class="progress-fill" style="width: {cycle.progress_percent}%"></div>
							</div>
							<span class="progress-text">{cycle.progress_percent}%</span>
						</div>
					</button>
				{/each}
			</div>

			<!-- Branch Cycles -->
			{#if branchCycles.length > 0}
				<div class="branches-section">
					<h3 class="section-title">Branches</h3>
					<div class="cycle-list">
						{#each branchCycles as cycle}
							<button
								type="button"
								class="cycle-item cycle-item--branch"
								class:cycle-item--active={cycle.cycle_id === activeCycleId}
								onclick={() => onSelectCycle(cycle.cycle_id)}
							>
								<div class="cycle-icon">
									<span class="icon-text">🌿</span>
								</div>
								<div class="cycle-details">
									<span class="cycle-label">
										Branch from {cycle.branch_point ? getComponentLabel(cycle.branch_point) : 'Unknown'}
									</span>
									<div class="progress-bar">
										<div class="progress-fill" style="width: {cycle.progress_percent}%"></div>
									</div>
									<span class="progress-text">{cycle.progress_percent}%</span>
								</div>
							</button>
						{/each}
					</div>
				</div>
			{/if}
		{/if}
	</div>
</aside>

<style>
	.cycle-tree-sidebar {
		width: 280px;
		height: 100%;
		background: white;
		border-right: 1px solid #e5e7eb;
		display: flex;
		flex-direction: column;
	}

	.sidebar-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1.5rem;
		border-bottom: 1px solid #e5e7eb;
	}

	.sidebar-title {
		font-size: 1.125rem;
		font-weight: 600;
		color: #111827;
		margin: 0;
	}

	.cycle-count {
		padding: 0.25rem 0.5rem;
		background: #f3f4f6;
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 600;
		color: #6b7280;
	}

	.sidebar-content {
		flex: 1;
		overflow-y: auto;
		padding: 1rem;
	}

	.empty-message {
		padding: 2rem 1rem;
		text-align: center;
		font-size: 0.875rem;
		color: #9ca3af;
		font-style: italic;
	}

	.cycle-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.cycle-item {
		width: 100%;
		display: flex;
		gap: 0.75rem;
		padding: 0.75rem;
		background: white;
		border: 2px solid #e5e7eb;
		border-radius: 8px;
		text-align: left;
		cursor: pointer;
		transition: all 0.2s;
	}

	.cycle-item:hover {
		border-color: #4f46e5;
		background: #f9fafb;
	}

	.cycle-item--active {
		border-color: #4f46e5;
		background: #eef2ff;
	}

	.cycle-item--branch {
		border-left-width: 4px;
		border-left-color: #10b981;
	}

	.cycle-icon {
		flex-shrink: 0;
		font-size: 1.5rem;
	}

	.icon-text {
		display: block;
	}

	.cycle-details {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.cycle-label {
		font-size: 0.875rem;
		font-weight: 500;
		color: #374151;
	}

	.progress-bar {
		height: 4px;
		background: #e5e7eb;
		border-radius: 2px;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		background: #4f46e5;
		border-radius: 2px;
		transition: width 0.3s ease;
	}

	.cycle-item--active .progress-fill {
		background: #4338ca;
	}

	.progress-text {
		font-size: 0.75rem;
		color: #6b7280;
	}

	.branches-section {
		margin-top: 1.5rem;
		padding-top: 1.5rem;
		border-top: 1px solid #e5e7eb;
	}

	.section-title {
		font-size: 0.75rem;
		font-weight: 600;
		color: #6b7280;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		margin: 0 0 0.75rem 0;
	}

	@media (max-width: 768px) {
		.cycle-tree-sidebar {
			width: 100%;
			height: auto;
			border-right: none;
			border-bottom: 1px solid #e5e7eb;
		}
	}
</style>
