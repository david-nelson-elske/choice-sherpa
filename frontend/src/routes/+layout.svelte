<!--
  Root layout component.

  Provides the main application shell with:
  - Navigation header with auth controls
  - Main content area (slot)
  - Session context for child components
-->

<script lang="ts">
	import { signIn, signOut } from '@auth/sveltekit/client';
	import type { LayoutData } from './$types';

	let { data, children }: { data: LayoutData; children: any } = $props();
</script>

<div class="app">
	<header>
		<nav>
			<a href="/" class="logo">Choice Sherpa</a>

			<div class="nav-links">
				<a href="/dashboard">Dashboard</a>
				<a href="/sessions">Sessions</a>
			</div>

			<div class="auth-controls">
				{#if data.session?.user}
					<span class="user-info">
						{data.session.user.name || data.session.user.email}
					</span>
					<button onclick={() => signOut({ callbackUrl: '/' })} class="btn btn-secondary">
						Sign out
					</button>
				{:else}
					<button onclick={() => signIn('zitadel')} class="btn btn-primary">
						Sign in
					</button>
				{/if}
			</div>
		</nav>
	</header>

	<main>
		{@render children()}
	</main>

	<footer>
		<p>&copy; {new Date().getFullYear()} Choice Sherpa. All rights reserved.</p>
	</footer>
</div>

<style>
	.app {
		display: flex;
		flex-direction: column;
		min-height: 100vh;
	}

	header {
		background: #1a1a2e;
		color: white;
		padding: 1rem 2rem;
	}

	nav {
		display: flex;
		align-items: center;
		gap: 2rem;
		max-width: 1200px;
		margin: 0 auto;
	}

	.logo {
		font-size: 1.5rem;
		font-weight: bold;
		color: white;
		text-decoration: none;
	}

	.nav-links {
		display: flex;
		gap: 1.5rem;
		flex: 1;
	}

	.nav-links a {
		color: rgba(255, 255, 255, 0.8);
		text-decoration: none;
		transition: color 0.2s;
	}

	.nav-links a:hover {
		color: white;
	}

	.auth-controls {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.user-info {
		color: rgba(255, 255, 255, 0.9);
		font-size: 0.9rem;
	}

	.btn {
		padding: 0.5rem 1rem;
		border: none;
		border-radius: 4px;
		cursor: pointer;
		font-size: 0.9rem;
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
		background: rgba(255, 255, 255, 0.1);
		color: white;
		border: 1px solid rgba(255, 255, 255, 0.2);
	}

	.btn-secondary:hover {
		background: rgba(255, 255, 255, 0.2);
	}

	main {
		flex: 1;
		padding: 2rem;
		max-width: 1200px;
		margin: 0 auto;
		width: 100%;
	}

	footer {
		background: #f5f5f5;
		padding: 1rem 2rem;
		text-align: center;
		color: #666;
		font-size: 0.9rem;
	}
</style>
