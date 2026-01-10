<!--
  ComponentDetailDrawer - Slide-out drawer for component details.

  Shows detailed component information including structured output,
  conversation history, and navigation actions.
-->

<script lang="ts">
	import { COMPONENT_LABELS, type ComponentType } from '../../cycle/domain/types';
	import type { ComponentDetailView } from '../index';

	interface Props {
		/** Component detail data */
		detail: ComponentDetailView | null;
		/** Whether the drawer is open */
		open: boolean;
		/** Called when drawer should close */
		onClose: () => void;
		/** Called when navigate to previous component */
		onNavigatePrevious?: () => void;
		/** Called when navigate to next component */
		onNavigateNext?: () => void;
		/** Called when branch is clicked */
		onBranch?: () => void;
		/** Called when revise is clicked */
		onRevise?: () => void;
	}

	let {
		detail,
		open,
		onClose,
		onNavigatePrevious,
		onNavigateNext,
		onBranch,
		onRevise
	}: Props = $props();

	const componentLabel = $derived(
		detail ? COMPONENT_LABELS[detail.component_type] : ''
	);

	const statusLabel = $derived(
		detail?.status === 'complete' ? 'Complete' :
		detail?.status === 'in_progress' ? 'In Progress' : 'Not Started'
	);

	const statusClass = $derived(
		detail?.status === 'complete' ? 'status--complete' :
		detail?.status === 'in_progress' ? 'status--in-progress' : 'status--not-started'
	);

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			onClose();
		}
	}

	function handleBackdropClick(event: MouseEvent) {
		if (event.target === event.currentTarget) {
			onClose();
		}
	}
</script>

{#if open}
	<div
		class="drawer-backdrop"
		onclick={handleBackdropClick}
		onkeydown={handleKeyDown}
		role="presentation"
	>
		<aside
			class="drawer"
			role="dialog"
			aria-modal="true"
			aria-label="Component details"
		>
			<!-- Header -->
			<div class="drawer-header">
				<div class="header-content">
					<h2 class="drawer-title">{componentLabel}</h2>
					<span class="status-badge {statusClass}">{statusLabel}</span>
				</div>
				<button
					type="button"
					class="close-button"
					onclick={onClose}
					aria-label="Close drawer"
				>
					✕
				</button>
			</div>

			<!-- Content -->
			<div class="drawer-content">
				{#if detail}
					<!-- Conversation Info -->
					<section class="detail-section">
						<h3 class="section-title">Conversation</h3>
						<p class="section-text">
							{detail.conversation_message_count} message{detail.conversation_message_count === 1 ? '' : 's'}
							{#if detail.last_message_at}
								· Last: {new Date(detail.last_message_at).toLocaleString()}
							{/if}
						</p>
					</section>

					<!-- Structured Output -->
					{#if detail.structured_output}
						<section class="detail-section">
							<h3 class="section-title">Output</h3>
							<pre class="output-preview">{JSON.stringify(detail.structured_output, null, 2)}</pre>
						</section>
					{/if}

					<!-- Actions -->
					<section class="detail-section">
						<h3 class="section-title">Actions</h3>
						<div class="action-buttons">
							{#if detail.can_branch && onBranch}
								<button type="button" class="action-button action-button--primary" onclick={onBranch}>
									Branch from Here
								</button>
							{/if}
							{#if detail.can_revise && onRevise}
								<button type="button" class="action-button action-button--secondary" onclick={onRevise}>
									Revise Component
								</button>
							{/if}
						</div>
					</section>
				{/if}
			</div>

			<!-- Footer Navigation -->
			<div class="drawer-footer">
				{#if detail?.previous_component && onNavigatePrevious}
					<button type="button" class="nav-button" onclick={onNavigatePrevious}>
						← {COMPONENT_LABELS[detail.previous_component]}
					</button>
				{/if}
				{#if detail?.next_component && onNavigateNext}
					<button type="button" class="nav-button nav-button--next" onclick={onNavigateNext}>
						{COMPONENT_LABELS[detail.next_component]} →
					</button>
				{/if}
			</div>
		</aside>
	</div>
{/if}

<style>
	.drawer-backdrop {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.5);
		z-index: 1000;
		display: flex;
		justify-content: flex-end;
		animation: fadeIn 0.2s ease-out;
	}

	@keyframes fadeIn {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	.drawer {
		width: 100%;
		max-width: 500px;
		height: 100%;
		background: white;
		box-shadow: -4px 0 6px rgba(0, 0, 0, 0.1);
		display: flex;
		flex-direction: column;
		animation: slideIn 0.3s ease-out;
	}

	@keyframes slideIn {
		from {
			transform: translateX(100%);
		}
		to {
			transform: translateX(0);
		}
	}

	.drawer-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		padding: 1.5rem;
		border-bottom: 1px solid #e5e7eb;
	}

	.header-content {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.drawer-title {
		font-size: 1.5rem;
		font-weight: 700;
		color: #111827;
		margin: 0;
	}

	.status-badge {
		display: inline-flex;
		padding: 0.25rem 0.5rem;
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.status--complete {
		background: #d1fae5;
		color: #065f46;
	}

	.status--in-progress {
		background: #fef3c7;
		color: #92400e;
	}

	.status--not-started {
		background: #f3f4f6;
		color: #6b7280;
	}

	.close-button {
		padding: 0.5rem;
		background: none;
		border: none;
		font-size: 1.5rem;
		color: #6b7280;
		cursor: pointer;
		transition: color 0.2s;
	}

	.close-button:hover {
		color: #111827;
	}

	.drawer-content {
		flex: 1;
		overflow-y: auto;
		padding: 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.detail-section {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.section-title {
		font-size: 0.875rem;
		font-weight: 600;
		color: #6b7280;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		margin: 0;
	}

	.section-text {
		margin: 0;
		font-size: 0.875rem;
		color: #374151;
	}

	.output-preview {
		margin: 0;
		padding: 1rem;
		background: #f9fafb;
		border: 1px solid #e5e7eb;
		border-radius: 6px;
		font-size: 0.75rem;
		overflow-x: auto;
		max-height: 300px;
	}

	.action-buttons {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.action-button {
		padding: 0.75rem 1rem;
		border-radius: 6px;
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s;
	}

	.action-button--primary {
		background: #4f46e5;
		border: 1px solid #4f46e5;
		color: white;
	}

	.action-button--primary:hover {
		background: #4338ca;
	}

	.action-button--secondary {
		background: white;
		border: 1px solid #d1d5db;
		color: #374151;
	}

	.action-button--secondary:hover {
		background: #f9fafb;
	}

	.drawer-footer {
		display: flex;
		justify-content: space-between;
		padding: 1.5rem;
		border-top: 1px solid #e5e7eb;
	}

	.nav-button {
		padding: 0.5rem 1rem;
		background: white;
		border: 1px solid #d1d5db;
		border-radius: 6px;
		font-size: 0.875rem;
		font-weight: 500;
		color: #4f46e5;
		cursor: pointer;
		transition: all 0.2s;
	}

	.nav-button:hover {
		background: #f9fafb;
		border-color: #4f46e5;
	}

	.nav-button--next {
		margin-left: auto;
	}
</style>
