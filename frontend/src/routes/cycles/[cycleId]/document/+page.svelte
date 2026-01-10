<script lang="ts">
	import { onMount } from 'svelte';
	import { DocumentEditor } from '$lib/components';
	import { documentStore, hasUnsavedChanges } from '$lib/stores/document';
	import { downloadDocument, regenerateDocument } from '$lib/api/document';
	import type { PageData } from './$types';
	import type { ExportFormat } from '$lib/types/document';

	export let data: PageData;

	let isExporting = false;
	let isRegenerating = false;
	let exportError: string | null = null;

	// Initialize the document store with loaded data
	onMount(() => {
		documentStore.load(data.cycleId);
	});

	/**
	 * Handle document export.
	 */
	async function handleExport(format: ExportFormat) {
		if (isExporting) return;

		isExporting = true;
		exportError = null;

		try {
			await downloadDocument(data.cycleId, format);
		} catch (err) {
			exportError = err instanceof Error ? err.message : 'Export failed';
		} finally {
			isExporting = false;
		}
	}

	/**
	 * Handle document regeneration.
	 */
	async function handleRegenerate() {
		if (isRegenerating) return;

		// Warn if there are unsaved changes
		if ($hasUnsavedChanges) {
			const confirmed = confirm(
				'You have unsaved changes. Regenerating will overwrite your edits. Continue?'
			);
			if (!confirmed) return;
		}

		isRegenerating = true;

		try {
			const result = await regenerateDocument(data.cycleId);
			// Update the store with the regenerated content
			documentStore.markSaved(result.version);
			// Reload to get fresh content
			await documentStore.load(data.cycleId);
		} catch (err) {
			alert(err instanceof Error ? err.message : 'Regeneration failed');
		} finally {
			isRegenerating = false;
		}
	}

	// Warn before leaving with unsaved changes
	function handleBeforeUnload(event: BeforeUnloadEvent) {
		if ($hasUnsavedChanges) {
			event.preventDefault();
			event.returnValue = '';
		}
	}
</script>

<svelte:window on:beforeunload={handleBeforeUnload} />

<div class="document-page">
	<header class="page-header">
		<div class="header-content container">
			<div class="header-left">
				<a href="/cycles/{data.cycleId}" class="back-link">← Back to Cycle</a>
				<h1>Decision Document</h1>
			</div>

			<div class="header-actions">
				<div class="export-buttons">
					<button
						class="btn btn-secondary"
						on:click={() => handleExport('markdown')}
						disabled={isExporting}
					>
						Export MD
					</button>
					<button
						class="btn btn-secondary"
						on:click={() => handleExport('html')}
						disabled={isExporting}
					>
						Export HTML
					</button>
					<button
						class="btn btn-secondary"
						on:click={() => handleExport('pdf')}
						disabled={isExporting}
					>
						Export PDF
					</button>
				</div>

				<button
					class="btn btn-primary"
					on:click={handleRegenerate}
					disabled={isRegenerating}
				>
					{isRegenerating ? 'Regenerating...' : 'Regenerate'}
				</button>
			</div>
		</div>
	</header>

	{#if exportError}
		<div class="error-banner container">
			<span>{exportError}</span>
			<button class="dismiss-btn" on:click={() => (exportError = null)}>×</button>
		</div>
	{/if}

	<main class="editor-main container">
		<DocumentEditor documentId={data.document.cycle_id} initialContent={data.document.content}>
			<span slot="title">
				Decision Document - Cycle {data.cycleId.slice(0, 8)}...
			</span>
		</DocumentEditor>
	</main>
</div>

<style>
	.document-page {
		display: flex;
		flex-direction: column;
		min-height: 100vh;
	}

	.page-header {
		background: var(--color-surface);
		border-bottom: 1px solid var(--color-border);
		padding: 1rem 0;
		position: sticky;
		top: 0;
		z-index: 10;
	}

	.header-content {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 1rem;
	}

	.header-left {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.back-link {
		font-size: 0.875rem;
		color: var(--color-text-muted);
	}

	.back-link:hover {
		color: var(--color-primary);
	}

	h1 {
		font-size: 1.5rem;
		margin: 0;
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.export-buttons {
		display: flex;
		gap: 0.5rem;
	}

	.error-banner {
		display: flex;
		justify-content: space-between;
		align-items: center;
		background: #fef2f2;
		color: var(--color-error);
		padding: 0.75rem 1rem;
		margin-top: 1rem;
		border-radius: var(--border-radius);
		border: 1px solid #fecaca;
	}

	.dismiss-btn {
		background: none;
		border: none;
		font-size: 1.25rem;
		color: inherit;
		padding: 0;
		line-height: 1;
	}

	.editor-main {
		flex: 1;
		padding: 1.5rem 1rem;
	}

	/* Make the editor fill available space */
	.editor-main :global(.editor-wrapper) {
		height: calc(100vh - 200px);
		min-height: 500px;
	}

	@media (max-width: 768px) {
		.header-content {
			flex-direction: column;
			align-items: flex-start;
		}

		.header-actions {
			flex-direction: column;
			width: 100%;
		}

		.export-buttons {
			width: 100%;
		}

		.export-buttons button {
			flex: 1;
		}
	}
</style>
