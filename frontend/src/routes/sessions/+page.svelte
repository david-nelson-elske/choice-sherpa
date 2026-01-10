<!--
  Sessions page - protected route.

  Displays the user's decision sessions.
  Requires authentication (enforced in +page.server.ts).
-->

<script lang="ts">
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<svelte:head>
	<title>My Sessions - Choice Sherpa</title>
</svelte:head>

<div class="sessions-page">
	<header class="page-header">
		<h1>My Sessions</h1>
		<p class="subtitle">
			Welcome, <strong>{data.user?.name || data.user?.email}</strong>
		</p>
	</header>

	<div class="actions">
		<a href="/sessions/new" class="btn btn-primary">
			+ New Session
		</a>
	</div>

	{#if data.sessions.length === 0}
		<div class="empty-state">
			<h2>No sessions yet</h2>
			<p>Create your first decision session to get started with the PrOACT framework.</p>
			<a href="/sessions/new" class="btn btn-primary">Create Session</a>
		</div>
	{:else}
		<div class="sessions-list">
			{#each data.sessions as session}
				<div class="session-card">
					<!-- Session card content would go here -->
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.sessions-page {
		max-width: 900px;
		margin: 0 auto;
	}

	.page-header {
		margin-bottom: 2rem;
	}

	h1 {
		font-size: 2rem;
		color: #1a1a2e;
		margin-bottom: 0.5rem;
	}

	.subtitle {
		color: #666;
	}

	.actions {
		margin-bottom: 2rem;
	}

	.btn {
		display: inline-block;
		padding: 0.75rem 1.5rem;
		border-radius: 4px;
		text-decoration: none;
		font-weight: 500;
		transition: background 0.2s;
	}

	.btn-primary {
		background: #4f46e5;
		color: white;
	}

	.btn-primary:hover {
		background: #4338ca;
	}

	.empty-state {
		text-align: center;
		padding: 4rem 2rem;
		background: #f8f9fa;
		border-radius: 8px;
	}

	.empty-state h2 {
		color: #1a1a2e;
		margin-bottom: 0.5rem;
	}

	.empty-state p {
		color: #666;
		margin-bottom: 1.5rem;
	}

	.sessions-list {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.session-card {
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		padding: 1.5rem;
	}
</style>
