<!--
  Home page component.

  Shows a welcome message and getting started information.
  Displays different content based on authentication state.
-->

<script lang="ts">
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<svelte:head>
	<title>Choice Sherpa - Decision Support</title>
	<meta name="description" content="Interactive decision support using the PrOACT framework" />
</svelte:head>

<div class="home">
	<section class="hero">
		<h1>Welcome to Choice Sherpa</h1>
		<p class="tagline">Navigate complex decisions with confidence using the PrOACT framework.</p>

		{#if data.session?.user}
			<div class="cta-authenticated">
				<p>Welcome back, <strong>{data.session.user.name || 'Decision Maker'}</strong>!</p>
				<a href="/sessions" class="btn btn-primary">Go to Sessions</a>
			</div>
		{:else}
			<div class="cta-guest">
				<p>Sign in to start making better decisions.</p>
				<a href="/login" class="btn btn-primary">Get Started</a>
			</div>
		{/if}
	</section>

	<section class="features">
		<h2>The PrOACT Framework</h2>
		<div class="feature-grid">
			<div class="feature">
				<h3>Problem</h3>
				<p>Define the decision context, constraints, and stakeholders.</p>
			</div>
			<div class="feature">
				<h3>Objectives</h3>
				<p>Identify what matters most - fundamental vs means objectives.</p>
			</div>
			<div class="feature">
				<h3>Alternatives</h3>
				<p>Generate and explore creative options beyond the obvious.</p>
			</div>
			<div class="feature">
				<h3>Consequences</h3>
				<p>Evaluate how each alternative performs against objectives.</p>
			</div>
			<div class="feature">
				<h3>Tradeoffs</h3>
				<p>Navigate tensions and find the best balance.</p>
			</div>
		</div>
	</section>
</div>

<style>
	.home {
		max-width: 900px;
		margin: 0 auto;
	}

	.hero {
		text-align: center;
		padding: 3rem 0;
	}

	h1 {
		font-size: 2.5rem;
		color: #1a1a2e;
		margin-bottom: 0.5rem;
	}

	.tagline {
		font-size: 1.25rem;
		color: #666;
		margin-bottom: 2rem;
	}

	.cta-authenticated,
	.cta-guest {
		margin-top: 2rem;
	}

	.cta-authenticated p,
	.cta-guest p {
		margin-bottom: 1rem;
		color: #444;
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

	.features {
		padding: 3rem 0;
	}

	.features h2 {
		text-align: center;
		font-size: 1.75rem;
		color: #1a1a2e;
		margin-bottom: 2rem;
	}

	.feature-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		gap: 1.5rem;
	}

	.feature {
		background: #f8f9fa;
		padding: 1.5rem;
		border-radius: 8px;
		border-left: 4px solid #4f46e5;
	}

	.feature h3 {
		color: #1a1a2e;
		margin-bottom: 0.5rem;
	}

	.feature p {
		color: #666;
		font-size: 0.95rem;
		line-height: 1.5;
	}
</style>
