<!--
  BranchDialog - Modal dialog for creating cycle branches.

  Allows users to select a branch point and create a new "what-if" cycle
  that inherits all completed work up to the branch point.
-->

<script lang="ts">
	import {
		COMPONENT_ORDER,
		COMPONENT_LABELS,
		type ComponentType,
		type ComponentStatusItem
	} from '../domain/types';

	interface Props {
		/** Whether the dialog is open */
		open: boolean;
		/** Current cycle ID */
		cycleId: string;
		/** Status of all components (determines valid branch points) */
		componentStatuses: ComponentStatusItem[];
		/** Called when branch is confirmed */
		onBranch?: (branchPoint: ComponentType) => void;
		/** Called when dialog is closed */
		onClose?: () => void;
	}

	let {
		open,
		cycleId,
		componentStatuses,
		onBranch,
		onClose
	}: Props = $props();

	let selectedBranchPoint: ComponentType | null = $state(null);

	// Only allow branching from started components (in_progress or complete)
	const validBranchPoints = $derived(
		COMPONENT_ORDER.filter((type) => {
			const status = componentStatuses.find((s) => s.component_type === type);
			return status?.status === 'in_progress' || status?.status === 'complete';
		})
	);

	function handleSubmit() {
		if (selectedBranchPoint) {
			onBranch?.(selectedBranchPoint);
			selectedBranchPoint = null;
		}
	}

	function handleClose() {
		selectedBranchPoint = null;
		onClose?.();
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			handleClose();
		}
	}

	function handleBackdropClick(event: MouseEvent) {
		if (event.target === event.currentTarget) {
			handleClose();
		}
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div
		class="dialog-backdrop"
		role="dialog"
		aria-modal="true"
		aria-labelledby="branch-dialog-title"
		onclick={handleBackdropClick}
		onkeydown={handleKeyDown}
	>
		<div class="dialog-content">
			<header class="dialog-header">
				<h2 id="branch-dialog-title">Create Branch</h2>
				<button
					type="button"
					class="close-button"
					aria-label="Close"
					onclick={handleClose}
				>
					×
				</button>
			</header>

			<div class="dialog-body">
				<p class="dialog-description">
					Select a branch point to create a "what-if" exploration.
					The new branch will inherit all completed work up to this point.
				</p>

				{#if validBranchPoints.length === 0}
					<div class="empty-state">
						<p>No valid branch points available.</p>
						<p class="hint">Start at least one component to enable branching.</p>
					</div>
				{:else}
					<fieldset class="branch-point-options">
						<legend class="sr-only">Select branch point</legend>
						{#each validBranchPoints as type}
							{@const status = componentStatuses.find((s) => s.component_type === type)}
							<label class="branch-option" class:branch-option--selected={selectedBranchPoint === type}>
								<input
									type="radio"
									name="branch-point"
									value={type}
									bind:group={selectedBranchPoint}
								/>
								<span class="option-content">
									<span class="option-label">{COMPONENT_LABELS[type]}</span>
									<span class="option-status">{status?.status === 'complete' ? 'Complete' : 'In Progress'}</span>
								</span>
							</label>
						{/each}
					</fieldset>
				{/if}
			</div>

			<footer class="dialog-footer">
				<button type="button" class="btn btn-secondary" onclick={handleClose}>
					Cancel
				</button>
				<button
					type="button"
					class="btn btn-primary"
					disabled={!selectedBranchPoint}
					onclick={handleSubmit}
				>
					Create Branch
				</button>
			</footer>
		</div>
	</div>
{/if}

<style>
	.dialog-backdrop {
		position: fixed;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		background: rgba(0, 0, 0, 0.5);
		z-index: 50;
	}

	.dialog-content {
		background: white;
		border-radius: 12px;
		box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
		width: 100%;
		max-width: 480px;
		max-height: 90vh;
		overflow: auto;
	}

	.dialog-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1rem 1.5rem;
		border-bottom: 1px solid #e5e7eb;
	}

	.dialog-header h2 {
		margin: 0;
		font-size: 1.25rem;
		color: #1f2937;
	}

	.close-button {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		background: none;
		border: none;
		font-size: 1.5rem;
		color: #6b7280;
		cursor: pointer;
		border-radius: 4px;
	}

	.close-button:hover {
		background: #f3f4f6;
	}

	.dialog-body {
		padding: 1.5rem;
	}

	.dialog-description {
		margin: 0 0 1rem;
		color: #6b7280;
		line-height: 1.5;
	}

	.branch-point-options {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		border: none;
		margin: 0;
		padding: 0;
	}

	.branch-option {
		display: flex;
		align-items: center;
		padding: 0.75rem 1rem;
		background: #f9fafb;
		border: 2px solid transparent;
		border-radius: 8px;
		cursor: pointer;
		transition: all 0.2s;
	}

	.branch-option:hover {
		background: #f3f4f6;
	}

	.branch-option--selected {
		background: #eef2ff;
		border-color: #4f46e5;
	}

	.branch-option input {
		margin-right: 0.75rem;
	}

	.option-content {
		display: flex;
		flex-direction: column;
	}

	.option-label {
		font-weight: 500;
		color: #1f2937;
	}

	.option-status {
		font-size: 0.75rem;
		color: #6b7280;
	}

	.empty-state {
		padding: 1.5rem;
		text-align: center;
		background: #f9fafb;
		border-radius: 8px;
	}

	.empty-state p {
		margin: 0;
		color: #6b7280;
	}

	.hint {
		margin-top: 0.5rem !important;
		font-size: 0.875rem;
	}

	.dialog-footer {
		display: flex;
		justify-content: flex-end;
		gap: 0.75rem;
		padding: 1rem 1.5rem;
		border-top: 1px solid #e5e7eb;
	}

	.btn {
		padding: 0.625rem 1.25rem;
		border-radius: 6px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s;
	}

	.btn-secondary {
		background: white;
		border: 1px solid #d1d5db;
		color: #374151;
	}

	.btn-secondary:hover {
		background: #f9fafb;
	}

	.btn-primary {
		background: #4f46e5;
		border: 1px solid #4f46e5;
		color: white;
	}

	.btn-primary:hover:not(:disabled) {
		background: #4338ca;
	}

	.btn-primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		border: 0;
	}
</style>
