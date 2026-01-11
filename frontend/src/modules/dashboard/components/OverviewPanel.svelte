<!--
  OverviewPanel - Main dashboard overview panel.

  Composes all dashboard components into a cohesive overview
  displaying the key decision elements and current state.
-->

<script lang="ts">
	import type { DashboardOverview } from '../index';
	import DecisionStatement from './DecisionStatement.svelte';
	import ObjectivesList from './ObjectivesList.svelte';
	import AlternativesPills from './AlternativesPills.svelte';
	import ConsequencesMatrix from './ConsequencesMatrix.svelte';
	import RecommendationCard from './RecommendationCard.svelte';
	import DQScoreBadge from './DQScoreBadge.svelte';

	interface Props {
		/** The dashboard overview data */
		overview: DashboardOverview;
		/** Called when decision statement edit is clicked */
		onEditStatement?: () => void;
		/** Called when "View Full" is clicked for any component */
		onViewComponent?: (componentType: string) => void;
	}

	let {
		overview,
		onEditStatement,
		onViewComponent
	}: Props = $props();

	const lastUpdatedDate = $derived(new Date(overview.last_updated).toLocaleString());
</script>

<div class="overview-panel">
	<!-- Header Section -->
	<div class="overview-header">
		<div class="header-title">
			<h1 class="session-title">{overview.session_title}</h1>
			<p class="last-updated">Last updated: {lastUpdatedDate}</p>
		</div>
		<DQScoreBadge score={overview.dq_score} size="large" />
	</div>

	<!-- Decision Statement -->
	<section class="section">
		<DecisionStatement
			statement={overview.decision_statement}
			editable={true}
			onEdit={onEditStatement}
		/>
	</section>

	<!-- Two Column Layout: Objectives and Alternatives -->
	<section class="section section--two-column">
		<ObjectivesList objectives={overview.objectives} />
		<AlternativesPills alternatives={overview.alternatives} />
	</section>

	<!-- Consequences Table -->
	<section class="section">
		<ConsequencesMatrix
			table={overview.consequences_table}
			onViewFull={() => onViewComponent?.('consequences')}
		/>
	</section>

	<!-- Recommendation -->
	<section class="section">
		<RecommendationCard
			recommendation={overview.recommendation}
			onViewFull={() => onViewComponent?.('recommendation')}
		/>
	</section>

	<!-- Cycle Info Footer -->
	{#if overview.cycle_count > 1}
		<div class="cycle-info">
			<span class="info-icon">🌳</span>
			<span class="info-text">
				This session has {overview.cycle_count} {overview.cycle_count === 1 ? 'cycle' : 'cycles'}
				{#if overview.active_cycle_id}
					(viewing: {overview.active_cycle_id})
				{/if}
			</span>
		</div>
	{/if}
</div>

<style>
	.overview-panel {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		max-width: 1200px;
		margin: 0 auto;
	}

	.overview-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: 1.5rem;
		padding: 1.5rem;
		background: white;
		border: 2px solid #e5e7eb;
		border-radius: 12px;
	}

	.header-title {
		flex: 1;
	}

	.session-title {
		font-size: 2rem;
		font-weight: 700;
		color: #111827;
		margin: 0 0 0.5rem 0;
		line-height: 1.2;
	}

	.last-updated {
		font-size: 0.875rem;
		color: #6b7280;
		margin: 0;
	}

	.section {
		width: 100%;
	}

	.section--two-column {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1.5rem;
	}

	@media (max-width: 768px) {
		.section--two-column {
			grid-template-columns: 1fr;
		}

		.overview-header {
			flex-direction: column;
			align-items: stretch;
		}

		.session-title {
			font-size: 1.5rem;
		}
	}

	.cycle-info {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 1rem 1.5rem;
		background: #fffbeb;
		border: 1px solid #fbbf24;
		border-radius: 8px;
		font-size: 0.875rem;
		color: #92400e;
	}

	.info-icon {
		font-size: 1.25rem;
	}

	.info-text {
		flex: 1;
	}
</style>
