<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { EditorState } from '@codemirror/state';
	import { EditorView, keymap, lineNumbers, highlightActiveLine } from '@codemirror/view';
	import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
	import { markdown } from '@codemirror/lang-markdown';
	import { syntaxHighlighting, defaultHighlightStyle } from '@codemirror/language';
	import { documentStore, editorStore, saveStatus } from '$lib/stores/document';

	/** The cycle/document ID for saving */
	export let documentId: string;

	/** Initial content (optional, will load from store if not provided) */
	export let initialContent = '';

	/** Whether the editor is read-only */
	export let readOnly = false;

	/** Debounce delay for auto-save in ms */
	export let autoSaveDelay = 1000;

	let editorContainer: HTMLDivElement;
	let view: EditorView | null = null;

	// Track the last known content to avoid circular updates
	let lastContent = '';

	/**
	 * Create the CodeMirror editor with markdown support.
	 */
	function createEditor(content: string): EditorView {
		const startState = EditorState.create({
			doc: content,
			extensions: [
				lineNumbers(),
				highlightActiveLine(),
				history(),
				markdown(),
				syntaxHighlighting(defaultHighlightStyle),
				keymap.of([...defaultKeymap, ...historyKeymap]),
				EditorState.readOnly.of(readOnly),
				EditorView.updateListener.of((update) => {
					if (update.docChanged) {
						const newContent = update.state.doc.toString();
						lastContent = newContent;

						// Update the store
						documentStore.setContent(newContent);

						// Trigger debounced auto-save if not read-only
						if (!readOnly && documentId) {
							editorStore.debouncedSave(documentId, newContent, autoSaveDelay);
						}
					}
				}),
				// Custom theme for the editor
				EditorView.theme({
					'&': {
						height: '100%',
						fontSize: '14px'
					},
					'.cm-scroller': {
						overflow: 'auto',
						fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace'
					},
					'.cm-content': {
						padding: '16px',
						minHeight: '400px'
					},
					'.cm-gutters': {
						backgroundColor: '#f8f9fa',
						borderRight: '1px solid #e9ecef'
					},
					'.cm-activeLineGutter': {
						backgroundColor: '#e9ecef'
					},
					'&.cm-focused': {
						outline: 'none'
					}
				})
			]
		});

		return new EditorView({
			state: startState,
			parent: editorContainer
		});
	}

	/**
	 * Update editor content from external source.
	 */
	function setContent(content: string): void {
		if (!view || content === lastContent) return;

		const transaction = view.state.update({
			changes: {
				from: 0,
				to: view.state.doc.length,
				insert: content
			}
		});
		view.dispatch(transaction);
		lastContent = content;
	}

	/**
	 * Get current editor content.
	 */
	export function getContent(): string {
		return view?.state.doc.toString() ?? '';
	}

	/**
	 * Focus the editor.
	 */
	export function focus(): void {
		view?.focus();
	}

	onMount(() => {
		const content = initialContent || '';
		lastContent = content;
		view = createEditor(content);

		// Enter edit mode
		if (!readOnly) {
			editorStore.startEditing();
		}
	});

	onDestroy(() => {
		// Clean up
		if (!readOnly) {
			editorStore.cancelPendingSave();
			editorStore.stopEditing();
		}
		view?.destroy();
	});

	// Reactive: update content when store changes externally
	$: if (view && $documentStore?.content && $documentStore.content !== lastContent) {
		setContent($documentStore.content);
	}
</script>

<div class="editor-wrapper">
	<div class="editor-header">
		<div class="editor-title">
			<slot name="title">Decision Document</slot>
		</div>
		<div class="editor-status">
			{#if $saveStatus === 'saving'}
				<span class="status-saving">Saving...</span>
			{:else if $saveStatus === 'saved'}
				<span class="status-saved">Saved</span>
			{:else if $saveStatus === 'error'}
				<span class="status-error">Save failed</span>
			{/if}
		</div>
	</div>

	<div class="editor-container" bind:this={editorContainer}></div>
</div>

<style>
	.editor-wrapper {
		display: flex;
		flex-direction: column;
		height: 100%;
		border: 1px solid #e9ecef;
		border-radius: 8px;
		overflow: hidden;
		background: white;
	}

	.editor-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 12px 16px;
		border-bottom: 1px solid #e9ecef;
		background: #f8f9fa;
	}

	.editor-title {
		font-weight: 600;
		color: #212529;
	}

	.editor-status {
		font-size: 12px;
	}

	.status-saving {
		color: #6c757d;
	}

	.status-saved {
		color: #28a745;
	}

	.status-error {
		color: #dc3545;
	}

	.editor-container {
		flex: 1;
		overflow: hidden;
	}

	/* Ensure CodeMirror fills the container */
	.editor-container :global(.cm-editor) {
		height: 100%;
	}
</style>
