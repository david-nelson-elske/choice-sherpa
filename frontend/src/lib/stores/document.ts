/**
 * Document stores for reactive state management.
 *
 * Uses Svelte 5 runes for state management.
 */

import { writable, derived, type Readable } from 'svelte/store';
import type {
	DocumentState,
	EditorState,
	SaveStatus,
	DocumentTreeNode
} from '$lib/types/document';
import * as api from '$lib/api/document';

// ════════════════════════════════════════════════════════════════════════════════
// Document State Store
// ════════════════════════════════════════════════════════════════════════════════

function createDocumentStore() {
	const { subscribe, set, update } = writable<DocumentState | null>(null);

	return {
		subscribe,

		/**
		 * Load a document for a cycle.
		 */
		async load(cycleId: string): Promise<void> {
			try {
				const response = await api.getDocument(cycleId);
				set({
					cycleId: response.cycle_id,
					sessionId: response.session_id,
					content: response.content,
					version: 1, // Initial version
					isDirty: false
				});
			} catch (error) {
				console.error('Failed to load document:', error);
				throw error;
			}
		},

		/**
		 * Update the content (marks as dirty).
		 */
		setContent(content: string): void {
			update((state) => {
				if (!state) return null;
				return {
					...state,
					content,
					isDirty: true
				};
			});
		},

		/**
		 * Mark document as saved.
		 */
		markSaved(version: number): void {
			update((state) => {
				if (!state) return null;
				return {
					...state,
					version,
					lastSaved: new Date(),
					isDirty: false
				};
			});
		},

		/**
		 * Clear the document state.
		 */
		clear(): void {
			set(null);
		}
	};
}

export const documentStore = createDocumentStore();

// ════════════════════════════════════════════════════════════════════════════════
// Editor State Store
// ════════════════════════════════════════════════════════════════════════════════

function createEditorStore() {
	const { subscribe, update } = writable<EditorState>({
		isEditing: false,
		saveStatus: 'idle'
	});

	let saveTimeout: ReturnType<typeof setTimeout> | null = null;

	return {
		subscribe,

		/**
		 * Enter edit mode.
		 */
		startEditing(): void {
			update((state) => ({ ...state, isEditing: true }));
		},

		/**
		 * Exit edit mode.
		 */
		stopEditing(): void {
			update((state) => ({ ...state, isEditing: false }));
		},

		/**
		 * Set save status.
		 */
		setSaveStatus(status: SaveStatus, error?: string): void {
			update((state) => ({ ...state, saveStatus: status, error }));
		},

		/**
		 * Save the current document with debouncing.
		 *
		 * @param documentId - The document ID to save
		 * @param content - The content to save
		 * @param delayMs - Debounce delay in milliseconds (default: 1000)
		 */
		debouncedSave(documentId: string, content: string, delayMs = 1000): void {
			if (saveTimeout) {
				clearTimeout(saveTimeout);
			}

			saveTimeout = setTimeout(async () => {
				try {
					this.setSaveStatus('saving');
					const result = await api.updateDocument(documentId, content);
					documentStore.markSaved(result.version);
					this.setSaveStatus('saved');

					// Reset to idle after a short delay
					setTimeout(() => this.setSaveStatus('idle'), 2000);
				} catch (error) {
					const message = error instanceof Error ? error.message : 'Save failed';
					this.setSaveStatus('error', message);
				}
			}, delayMs);
		},

		/**
		 * Cancel any pending save.
		 */
		cancelPendingSave(): void {
			if (saveTimeout) {
				clearTimeout(saveTimeout);
				saveTimeout = null;
			}
		}
	};
}

export const editorStore = createEditorStore();

// ════════════════════════════════════════════════════════════════════════════════
// Derived Stores
// ════════════════════════════════════════════════════════════════════════════════

/** Whether there are unsaved changes */
export const hasUnsavedChanges: Readable<boolean> = derived(
	documentStore,
	($doc) => $doc?.isDirty ?? false
);

/** The current document content */
export const documentContent: Readable<string> = derived(
	documentStore,
	($doc) => $doc?.content ?? ''
);

/** Whether the editor is in edit mode */
export const isEditing: Readable<boolean> = derived(
	editorStore,
	($editor) => $editor.isEditing
);

/** Current save status */
export const saveStatus: Readable<SaveStatus> = derived(
	editorStore,
	($editor) => $editor.saveStatus
);

// ════════════════════════════════════════════════════════════════════════════════
// Document Tree Store
// ════════════════════════════════════════════════════════════════════════════════

function createDocumentTreeStore() {
	const { subscribe, set } = writable<DocumentTreeNode[]>([]);

	return {
		subscribe,

		/**
		 * Set the document tree.
		 */
		setTree(nodes: DocumentTreeNode[]): void {
			set(nodes);
		},

		/**
		 * Clear the tree.
		 */
		clear(): void {
			set([]);
		}
	};
}

export const documentTreeStore = createDocumentTreeStore();

// ════════════════════════════════════════════════════════════════════════════════
// Selected Document Store
// ════════════════════════════════════════════════════════════════════════════════

/** Currently selected document/cycle ID in the tree */
export const selectedCycleId = writable<string | null>(null);
