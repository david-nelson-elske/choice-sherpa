<!--
  CycleProgress - Visual progress indicator for a cycle.

  Shows a progress bar with percentage and current step label.
  Used in cycle headers and tree nodes.
-->

<script lang="ts">
	import { COMPONENT_LABELS, type ComponentType } from '../domain/types';

	interface Props {
		/** Progress percentage (0-100) */
		percent: number;
		/** Current step in the cycle */
		currentStep: ComponentType;
		/** Whether to show the label */
		showLabel?: boolean;
		/** Size variant */
		size?: 'small' | 'medium' | 'large';
	}

	let {
		percent,
		currentStep,
		showLabel = true,
		size = 'medium'
	}: Props = $props();

	const currentLabel = $derived(COMPONENT_LABELS[currentStep]);
</script>

<div class="cycle-progress cycle-progress--{size}">
	<div class="progress-bar">
		<div
			class="progress-fill"
			class:progress-fill--complete={percent === 100}
			style="width: {percent}%"
		></div>
	</div>
	<div class="progress-info">
		<span class="progress-percent">{percent}%</span>
		{#if showLabel}
			<span class="progress-label">{currentLabel}</span>
		{/if}
	</div>
</div>

<style>
	.cycle-progress {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.progress-bar {
		height: 8px;
		background: #e5e7eb;
		border-radius: 4px;
		overflow: hidden;
	}

	.cycle-progress--small .progress-bar {
		height: 4px;
	}

	.cycle-progress--large .progress-bar {
		height: 12px;
	}

	.progress-fill {
		height: 100%;
		background: #4f46e5;
		border-radius: 4px;
		transition: width 0.3s ease;
	}

	.progress-fill--complete {
		background: #10b981;
	}

	.progress-info {
		display: flex;
		justify-content: space-between;
		align-items: center;
		font-size: 0.875rem;
	}

	.cycle-progress--small .progress-info {
		font-size: 0.75rem;
	}

	.progress-percent {
		font-weight: 600;
		color: #1f2937;
	}

	.progress-label {
		color: #6b7280;
	}
</style>
