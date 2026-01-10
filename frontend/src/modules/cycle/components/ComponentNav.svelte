<!--
  ComponentNav - Navigation between PrOACT components.

  Displays clickable steps showing component status and enables
  navigation between components in a cycle.
-->

<script lang="ts">
	import {
		COMPONENT_ORDER,
		COMPONENT_LABELS,
		type ComponentType,
		type ComponentStatusItem
	} from '../domain/types';

	interface Props {
		/** Current step in the cycle */
		currentStep: ComponentType;
		/** Status of all components */
		componentStatuses: ComponentStatusItem[];
		/** Whether navigation is disabled */
		disabled?: boolean;
		/** Called when a component is clicked */
		onNavigate?: (component: ComponentType) => void;
	}

	let {
		currentStep,
		componentStatuses,
		disabled = false,
		onNavigate
	}: Props = $props();

	function getStatus(type: ComponentType): ComponentStatusItem | undefined {
		return componentStatuses.find((s) => s.component_type === type);
	}

	function handleClick(type: ComponentType) {
		if (disabled) return;
		onNavigate?.(type);
	}

	function handleKeyDown(event: KeyboardEvent, type: ComponentType) {
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			handleClick(type);
		}
	}
</script>

<nav class="component-nav" aria-label="PrOACT components">
	<ol class="component-list">
		{#each COMPONENT_ORDER as component, index}
			{@const status = getStatus(component)}
			{@const isCurrent = component === currentStep}
			{@const isComplete = status?.status === 'complete'}
			{@const isInProgress = status?.status === 'in_progress'}
			{@const isClickable = !disabled && (isComplete || isInProgress || isCurrent)}
			<li class="component-item">
				<button
					type="button"
					class="component-button"
					class:component-button--current={isCurrent}
					class:component-button--complete={isComplete}
					class:component-button--in-progress={isInProgress}
					class:component-button--disabled={!isClickable}
					disabled={!isClickable}
					aria-current={isCurrent ? 'step' : undefined}
					onclick={() => handleClick(component)}
					onkeydown={(e) => handleKeyDown(e, component)}
				>
					<span class="component-number">{index + 1}</span>
					<span class="component-label">{COMPONENT_LABELS[component]}</span>
					{#if isComplete}
						<span class="component-check" aria-hidden="true">✓</span>
					{/if}
				</button>
				{#if index < COMPONENT_ORDER.length - 1}
					<div class="component-connector" class:component-connector--complete={isComplete}></div>
				{/if}
			</li>
		{/each}
	</ol>
</nav>

<style>
	.component-nav {
		width: 100%;
		overflow-x: auto;
	}

	.component-list {
		display: flex;
		align-items: flex-start;
		list-style: none;
		margin: 0;
		padding: 0;
		min-width: max-content;
	}

	.component-item {
		display: flex;
		align-items: center;
	}

	.component-button {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
		padding: 0.75rem 1rem;
		background: none;
		border: 2px solid #e5e7eb;
		border-radius: 8px;
		cursor: pointer;
		transition: all 0.2s;
		min-width: 100px;
	}

	.component-button:hover:not(:disabled) {
		border-color: #4f46e5;
	}

	.component-button--current {
		border-color: #4f46e5;
		background: #eef2ff;
	}

	.component-button--complete {
		border-color: #10b981;
		background: #ecfdf5;
	}

	.component-button--in-progress {
		border-color: #f59e0b;
		background: #fffbeb;
	}

	.component-button--disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.component-number {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		background: #f3f4f6;
		border-radius: 50%;
		font-size: 0.75rem;
		font-weight: 600;
		color: #6b7280;
	}

	.component-button--current .component-number {
		background: #4f46e5;
		color: white;
	}

	.component-button--complete .component-number {
		background: #10b981;
		color: white;
	}

	.component-label {
		font-size: 0.75rem;
		color: #374151;
		text-align: center;
		line-height: 1.2;
	}

	.component-check {
		color: #10b981;
		font-size: 1rem;
	}

	.component-connector {
		width: 24px;
		height: 2px;
		background: #e5e7eb;
		margin: 0 4px;
	}

	.component-connector--complete {
		background: #10b981;
	}
</style>
