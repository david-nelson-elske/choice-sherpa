<script lang="ts">
	// Home page - in production would list sessions/cycles
	// For now, serves as a placeholder
</script>

<div class="home-page">
	<header class="hero">
		<div class="container">
			<h1>Choice Sherpa</h1>
			<p class="tagline">Navigate complex decisions with the PrOACT framework</p>
		</div>
	</header>

	<main class="container">
		<section class="features">
			<h2>Decision Document Editor</h2>
			<p>
				The Decision Document is a living artifact that captures your decision analysis as you work
				through the PrOACT framework. Edit directly, export to multiple formats, and branch to
				explore alternatives.
			</p>

			<div class="feature-list">
				<div class="feature-card">
					<h3>Live Editing</h3>
					<p>Edit your decision document directly with auto-save and markdown preview.</p>
				</div>

				<div class="feature-card">
					<h3>Multi-Format Export</h3>
					<p>Export your document as Markdown, HTML, or PDF for sharing and archival.</p>
				</div>

				<div class="feature-card">
					<h3>Branching</h3>
					<p>Create alternative paths to explore "what if" scenarios without losing progress.</p>
				</div>

				<div class="feature-card">
					<h3>Version Tracking</h3>
					<p>Every edit is tracked, so you can always see how your analysis evolved.</p>
				</div>
			</div>
		</section>

		<section class="cta">
			<p class="text-muted">
				To view a document, navigate to <code>/cycles/[cycleId]/document</code>
			</p>
		</section>
	</main>
</div>

<style>
	.home-page {
		min-height: 100vh;
	}

	.hero {
		background: linear-gradient(135deg, var(--color-primary), var(--color-primary-dark));
		color: white;
		padding: 4rem 0;
		text-align: center;
	}

	.hero h1 {
		font-size: 3rem;
		margin-bottom: 0.5rem;
	}

	.tagline {
		font-size: 1.25rem;
		opacity: 0.9;
	}

	main {
		padding: 3rem 1rem;
	}

	.features {
		max-width: 800px;
		margin: 0 auto;
	}

	.features h2 {
		font-size: 2rem;
		margin-bottom: 1rem;
	}

	.features > p {
		color: var(--color-text-muted);
		font-size: 1.125rem;
		margin-bottom: 2rem;
	}

	.feature-list {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		gap: 1.5rem;
	}

	.feature-card {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius);
		padding: 1.5rem;
	}

	.feature-card h3 {
		font-size: 1.125rem;
		margin-bottom: 0.5rem;
		color: var(--color-primary);
	}

	.feature-card p {
		color: var(--color-text-muted);
		font-size: 0.875rem;
	}

	.cta {
		text-align: center;
		margin-top: 3rem;
		padding-top: 2rem;
		border-top: 1px solid var(--color-border);
	}

	code {
		background: var(--color-background);
		padding: 0.25rem 0.5rem;
		border-radius: 4px;
		font-family: var(--font-mono);
		font-size: 0.875rem;
	}
</style>
