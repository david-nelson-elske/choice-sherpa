<!--
  Authentication error page.

  Displays when there's an error during the authentication flow.
-->

<script lang="ts">
	import { page } from '$app/stores';

	// Get error from query params
	const error = $derived($page.url.searchParams.get('error') || 'Unknown error');
</script>

<svelte:head>
	<title>Authentication Error - Choice Sherpa</title>
</svelte:head>

<div class="error-page">
	<div class="error-card">
		<div class="error-icon">!</div>
		<h1>Authentication Error</h1>
		<p class="error-message">{error}</p>

		<div class="actions">
			<a href="/auth/signin" class="btn btn-primary">Try Again</a>
			<a href="/" class="btn btn-secondary">Go Home</a>
		</div>
	</div>
</div>

<style>
	.error-page {
		min-height: 80vh;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.error-card {
		background: white;
		padding: 3rem;
		border-radius: 12px;
		box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
		max-width: 400px;
		width: 100%;
		text-align: center;
	}

	.error-icon {
		width: 60px;
		height: 60px;
		background: #fee2e2;
		color: #dc2626;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 2rem;
		font-weight: bold;
		margin: 0 auto 1.5rem;
	}

	h1 {
		font-size: 1.5rem;
		color: #1a1a2e;
		margin-bottom: 0.5rem;
	}

	.error-message {
		color: #666;
		margin-bottom: 2rem;
	}

	.actions {
		display: flex;
		gap: 1rem;
		justify-content: center;
	}

	.btn {
		padding: 0.75rem 1.5rem;
		border-radius: 6px;
		font-weight: 500;
		text-decoration: none;
		transition: background 0.2s;
	}

	.btn-primary {
		background: #4f46e5;
		color: white;
	}

	.btn-primary:hover {
		background: #4338ca;
	}

	.btn-secondary {
		background: #f3f4f6;
		color: #374151;
	}

	.btn-secondary:hover {
		background: #e5e7eb;
	}
</style>
