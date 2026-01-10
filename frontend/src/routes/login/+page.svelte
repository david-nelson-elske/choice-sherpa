<!--
  Custom sign-in page.

  Provides a branded sign-in experience while using Auth.js
  for the actual authentication flow.
-->

<script lang="ts">
	import { signIn } from '@auth/sveltekit/client';
	import { page } from '$app/stores';

	// Get callback URL from query params
	const callbackUrl = $derived($page.url.searchParams.get('callbackUrl') || '/');
</script>

<svelte:head>
	<title>Sign In - Choice Sherpa</title>
</svelte:head>

<div class="signin-page">
	<div class="signin-card">
		<h1>Welcome to Choice Sherpa</h1>
		<p class="subtitle">Sign in to continue your decision journey.</p>

		<div class="providers">
			<button onclick={() => signIn('zitadel', { callbackUrl })} class="btn btn-provider">
				<svg class="icon" viewBox="0 0 24 24" fill="currentColor">
					<path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 3c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm0 14.2c-2.5 0-4.71-1.28-6-3.22.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08-1.29 1.94-3.5 3.22-6 3.22z"/>
				</svg>
				Sign in with Zitadel
			</button>
		</div>

		<p class="terms">
			By signing in, you agree to our
			<a href="/terms">Terms of Service</a> and
			<a href="/privacy">Privacy Policy</a>.
		</p>
	</div>
</div>

<style>
	.signin-page {
		min-height: 80vh;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.signin-card {
		background: white;
		padding: 3rem;
		border-radius: 12px;
		box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
		max-width: 400px;
		width: 100%;
		text-align: center;
	}

	h1 {
		font-size: 1.75rem;
		color: #1a1a2e;
		margin-bottom: 0.5rem;
	}

	.subtitle {
		color: #666;
		margin-bottom: 2rem;
	}

	.providers {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.75rem;
		padding: 0.875rem 1.5rem;
		border: none;
		border-radius: 8px;
		font-size: 1rem;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.2s;
	}

	.btn-provider {
		background: #1a1a2e;
		color: white;
	}

	.btn-provider:hover {
		background: #2d2d4a;
	}

	.icon {
		width: 20px;
		height: 20px;
	}

	.terms {
		margin-top: 2rem;
		font-size: 0.85rem;
		color: #888;
	}

	.terms a {
		color: #4f46e5;
		text-decoration: none;
	}

	.terms a:hover {
		text-decoration: underline;
	}
</style>
