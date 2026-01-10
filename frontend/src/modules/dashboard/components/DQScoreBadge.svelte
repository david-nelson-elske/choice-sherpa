<!--
  DQScoreBadge - Decision Quality score display badge.

  Shows the DQ score as a percentage with color-coding based on
  the score level (low/medium/high).
-->

<script lang="ts">
	import { getDQCategory, formatDQScore } from '../index';

	interface Props {
		/** DQ score (0-100) or null if not yet assessed */
		score: number | null;
		/** Size variant */
		size?: 'small' | 'medium' | 'large';
		/** Whether to show the label */
		showLabel?: boolean;
	}

	let {
		score,
		size = 'medium',
		showLabel = true
	}: Props = $props();

	const category = $derived(getDQCategory(score));
	const formattedScore = $derived(formatDQScore(score));
</script>

<div class="dq-badge dq-badge--{size} dq-badge--{category}" role="status" aria-label="Decision Quality Score">
	<div class="dq-score">{formattedScore}</div>
	{#if showLabel}
		<div class="dq-label">
			{#if category === 'none'}
				Not Assessed
			{:else if category === 'low'}
				Needs Work
			{:else if category === 'medium'}
				Good Progress
			{:else}
				Excellent
			{/if}
		</div>
	{/if}
</div>

<style>
	.dq-badge {
		display: inline-flex;
		flex-direction: column;
		align-items: center;
		gap: 0.25rem;
		padding: 0.75rem 1rem;
		border-radius: 8px;
		border: 2px solid;
		transition: all 0.2s;
	}

	.dq-badge--small {
		padding: 0.5rem 0.75rem;
	}

	.dq-badge--large {
		padding: 1rem 1.5rem;
	}

	.dq-badge--none {
		background: #f9fafb;
		border-color: #e5e7eb;
		color: #6b7280;
	}

	.dq-badge--low {
		background: #fef2f2;
		border-color: #fca5a5;
		color: #dc2626;
	}

	.dq-badge--medium {
		background: #fffbeb;
		border-color: #fcd34d;
		color: #d97706;
	}

	.dq-badge--high {
		background: #ecfdf5;
		border-color: #6ee7b7;
		color: #059669;
	}

	.dq-score {
		font-size: 1.5rem;
		font-weight: 700;
		line-height: 1;
	}

	.dq-badge--small .dq-score {
		font-size: 1.125rem;
	}

	.dq-badge--large .dq-score {
		font-size: 2rem;
	}

	.dq-label {
		font-size: 0.75rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		opacity: 0.8;
	}

	.dq-badge--small .dq-label {
		font-size: 0.625rem;
	}

	.dq-badge--large .dq-label {
		font-size: 0.875rem;
	}
</style>
