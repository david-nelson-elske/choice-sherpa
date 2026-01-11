<!--
  RecommendationCard - Display the recommendation summary.

  Shows the recommended alternative and a preview of the rationale.
  Links to the full recommendation component for details.
-->

<script lang="ts">
	import type { RecommendationSummary } from '../index';

	interface Props {
		/** The recommendation summary data */
		recommendation: RecommendationSummary | null;
		/** Called when "View Full Recommendation" is clicked */
		onViewFull?: () => void;
	}

	let {
		recommendation,
		onViewFull
	}: Props = $props();

	const hasRecommendation = $derived(
		recommendation !== null &&
		recommendation.recommended_alternative !== null
	);
</script>

<div class="recommendation-card" class:recommendation-card--empty={!hasRecommendation}>
	<div class="card-header">
		<h3 class="card-title">Recommendation</h3>
		{#if hasRecommendation && onViewFull}
			<button
				type="button"
				class="view-full-button"
				onclick={onViewFull}
				aria-label="View full recommendation"
			>
				View Full
			</button>
		{/if}
	</div>

	<div class="card-content">
		{#if hasRecommendation}
			<div class="recommended-alternative">
				<span class="alternative-label">Recommended:</span>
				<span class="alternative-name">{recommendation.recommended_alternative}</span>
			</div>

			{#if recommendation.rationale_preview}
				<div class="rationale-preview">
					<p class="rationale-text">{recommendation.rationale_preview}</p>
					{#if onViewFull}
						<button
							type="button"
							class="read-more-link"
							onclick={onViewFull}
						>
							Read more →
						</button>
					{/if}
				</div>
			{/if}
		{:else}
			<p class="empty-message">
				No recommendation yet. Complete the Recommendation component to synthesize your analysis.
			</p>
		{/if}
	</div>
</div>

<style>
	.recommendation-card {
		background: white;
		border: 2px solid #10b981;
		border-radius: 12px;
		padding: 1.5rem;
		transition: all 0.2s;
	}

	.recommendation-card--empty {
		border-color: #e5e7eb;
		border-style: dashed;
		background: #f9fafb;
	}

	.card-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.card-title {
		font-size: 1.125rem;
		font-weight: 600;
		color: #111827;
		margin: 0;
	}

	.view-full-button {
		padding: 0.375rem 0.75rem;
		background: white;
		border: 1px solid #10b981;
		border-radius: 6px;
		font-size: 0.875rem;
		font-weight: 500;
		color: #10b981;
		cursor: pointer;
		transition: all 0.2s;
	}

	.view-full-button:hover {
		background: #ecfdf5;
		border-color: #059669;
		color: #059669;
	}

	.view-full-button:active {
		transform: scale(0.98);
	}

	.card-content {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.recommended-alternative {
		display: flex;
		align-items: baseline;
		gap: 0.5rem;
		padding: 1rem;
		background: #ecfdf5;
		border: 1px solid #6ee7b7;
		border-radius: 8px;
	}

	.alternative-label {
		font-size: 0.875rem;
		font-weight: 600;
		color: #047857;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.alternative-name {
		font-size: 1.125rem;
		font-weight: 700;
		color: #065f46;
	}

	.rationale-preview {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.rationale-text {
		margin: 0;
		font-size: 0.875rem;
		color: #374151;
		line-height: 1.6;
	}

	.read-more-link {
		align-self: flex-start;
		padding: 0;
		background: none;
		border: none;
		font-size: 0.875rem;
		font-weight: 500;
		color: #10b981;
		cursor: pointer;
		transition: color 0.2s;
	}

	.read-more-link:hover {
		color: #059669;
		text-decoration: underline;
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
