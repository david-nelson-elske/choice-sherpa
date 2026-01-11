<script lang="ts">
	import { page } from '$app/stores';
	import { getProactTreeView } from '$modules/cycle/api/cycle-api';
	import PrOACTTreeView from '$modules/cycle/components/PrOACTTreeView.svelte';
	import type { PrOACTTreeNode } from '$modules/cycle/domain/types';
	import { onMount } from 'svelte';

	const sessionId = $page.params.sessionId;
	const session = $page.data.session;

	let tree: PrOACTTreeNode | null = $state(null);
	let loading = $state(true);
	let error: string | null = $state(null);
	let selectedCycleId: string | null = $state(null);

	async function loadTree() {
		loading = true;
		error = null;
		try {
			tree = await getProactTreeView(session, sessionId);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load cycle tree';
			console.error('Failed to load cycle tree:', err);
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		loadTree();
	});

	function handleSelectCycle(cycleId: string) {
		selectedCycleId = cycleId;
		// Could navigate to cycle detail page here if needed
		// goto(`/sessions/${sessionId}/cycles/${cycleId}`);
	}

	function handleRefresh() {
		loadTree();
	}
</script>

<svelte:head>
	<title>Cycle Tree - Choice Sherpa</title>
</svelte:head>

<div class="cycles-page">
	<header class="page-header">
		<div class="header-content">
			<h1>Cycle Tree</h1>
			<p class="subtitle">
				Visualize your decision cycles and branches with the PrOACT framework
			</p>
		</div>
		<div class="header-actions">
			<button type="button" class="btn btn-secondary" onclick={handleRefresh} disabled={loading}>
				{loading ? 'Loading...' : 'Refresh'}
			</button>
		</div>
	</header>

	<div class="legend">
		<h3>PrOACT Framework</h3>
		<div class="legend-items">
			<div class="legend-item">
				<span class="legend-letter">P</span>
				<span class="legend-label">Problem Frame</span>
			</div>
			<div class="legend-item">
				<span class="legend-letter">R</span>
				<span class="legend-label">Objectives (Really matters)</span>
			</div>
			<div class="legend-item">
				<span class="legend-letter">O</span>
				<span class="legend-label">Options/Alternatives</span>
			</div>
			<div class="legend-item">
				<span class="legend-letter">A</span>
				<span class="legend-label">Analysis/Consequences</span>
			</div>
			<div class="legend-item">
				<span class="legend-letter">C</span>
				<span class="legend-label">Clear Tradeoffs</span>
			</div>
			<div class="legend-item">
				<span class="legend-letter">T</span>
				<span class="legend-label">Think Through</span>
			</div>
		</div>

		<div class="status-legend">
			<span class="status-item">
				<span class="status-icon completed">●</span> Completed
			</span>
			<span class="status-item">
				<span class="status-icon in-progress">◉</span> In Progress
			</span>
			<span class="status-item">
				<span class="status-icon not-started">○</span> Not Started
			</span>
		</div>
	</div>

	<main class="page-content">
		{#if loading}
			<div class="loading-state">
				<div class="spinner"></div>
				<p>Loading cycle tree...</p>
			</div>
		{:else if error}
			<div class="error-state">
				<p class="error-message">{error}</p>
				<button type="button" class="btn btn-primary" onclick={handleRefresh}>Try Again</button>
			</div>
		{:else}
			<PrOACTTreeView {tree} {selectedCycleId} onSelect={handleSelectCycle} />
		{/if}
	</main>
</div>

<style>
	.cycles-page {
		max-width: 1200px;
		margin: 0 auto;
		padding: 2rem;
	}

	.page-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 2rem;
		gap: 2rem;
	}

	.header-content h1 {
		margin: 0 0 0.5rem 0;
		font-size: 2rem;
		font-weight: 700;
		color: #1f2937;
	}

	.subtitle {
		margin: 0;
		color: #6b7280;
		font-size: 1rem;
	}

	.header-actions {
		display: flex;
		gap: 0.75rem;
	}

	.legend {
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		padding: 1.5rem;
		margin-bottom: 2rem;
	}

	.legend h3 {
		margin: 0 0 1rem 0;
		font-size: 1.125rem;
		font-weight: 600;
		color: #1f2937;
	}

	.legend-items {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 1rem;
		margin-bottom: 1.5rem;
	}

	.legend-item {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.legend-letter {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 2rem;
		height: 2rem;
		background: #4f46e5;
		color: white;
		border-radius: 6px;
		font-weight: 700;
		font-size: 1rem;
	}

	.legend-label {
		font-size: 0.875rem;
		color: #374151;
	}

	.status-legend {
		display: flex;
		gap: 1.5rem;
		padding-top: 1rem;
		border-top: 1px solid #f3f4f6;
	}

	.status-item {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.875rem;
		color: #6b7280;
	}

	.status-icon {
		font-size: 1.125rem;
	}

	.status-icon.completed {
		color: #047857;
	}

	.status-icon.in-progress {
		color: #d97706;
	}

	.status-icon.not-started {
		color: #9ca3af;
	}

	.page-content {
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		padding: 2rem;
	}

	.loading-state,
	.error-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 4rem 2rem;
		gap: 1rem;
	}

	.spinner {
		width: 3rem;
		height: 3rem;
		border: 3px solid #f3f4f6;
		border-top-color: #4f46e5;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.error-message {
		color: #dc2626;
		font-size: 1rem;
		margin: 0;
	}

	.btn {
		padding: 0.625rem 1.25rem;
		border-radius: 6px;
		font-weight: 500;
		font-size: 0.875rem;
		cursor: pointer;
		border: none;
		transition: all 0.2s;
	}

	.btn-primary {
		background: #4f46e5;
		color: white;
	}

	.btn-primary:hover {
		background: #4338ca;
	}

	.btn-secondary {
		background: white;
		color: #374151;
		border: 1px solid #d1d5db;
	}

	.btn-secondary:hover {
		background: #f9fafb;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
