/**
 * Document API client.
 *
 * Provides functions for interacting with the decision document endpoints.
 */

import type {
	DocumentResponse,
	RegenerateDocumentResponse,
	UpdateDocumentResponse,
	BranchCycleResponse,
	DocumentFormat,
	ExportFormat,
	ComponentType,
	ErrorResponse,
	DocumentTree
} from '$lib/types/document';

/** Base API error class */
export class ApiError extends Error {
	constructor(
		public code: string,
		message: string,
		public status: number,
		public details?: Record<string, unknown>
	) {
		super(message);
		this.name = 'ApiError';
	}

	static async fromResponse(response: Response): Promise<ApiError> {
		try {
			const data = (await response.json()) as ErrorResponse;
			return new ApiError(data.code, data.message, response.status, data.details);
		} catch {
			return new ApiError('UNKNOWN_ERROR', response.statusText, response.status);
		}
	}
}

// ════════════════════════════════════════════════════════════════════════════════
// Document Operations
// ════════════════════════════════════════════════════════════════════════════════

/**
 * Get a decision document for a cycle.
 *
 * @param cycleId - The cycle ID
 * @param format - Optional format (full, summary, export)
 * @returns The document content and metadata
 */
export async function getDocument(
	cycleId: string,
	format: DocumentFormat = 'full'
): Promise<DocumentResponse> {
	const response = await fetch(`/api/cycles/${cycleId}/document?format=${format}`);

	if (!response.ok) {
		throw await ApiError.fromResponse(response);
	}

	return response.json();
}

/**
 * Regenerate and persist a decision document.
 *
 * @param cycleId - The cycle ID
 * @returns The regenerated document with version info
 */
export async function regenerateDocument(cycleId: string): Promise<RegenerateDocumentResponse> {
	const response = await fetch(`/api/cycles/${cycleId}/document/regenerate`, {
		method: 'POST'
	});

	if (!response.ok) {
		throw await ApiError.fromResponse(response);
	}

	return response.json();
}

/**
 * Update a decision document from user edits.
 *
 * @param documentId - The document ID
 * @param content - The edited markdown content
 * @param syncToComponents - Whether to update component outputs (default: true)
 * @returns The update result with parse summary
 */
export async function updateDocument(
	documentId: string,
	content: string,
	syncToComponents = true
): Promise<UpdateDocumentResponse> {
	const response = await fetch(`/api/documents/${documentId}`, {
		method: 'PUT',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({
			content,
			sync_to_components: syncToComponents
		})
	});

	if (!response.ok) {
		throw await ApiError.fromResponse(response);
	}

	return response.json();
}

/**
 * Export a document in various formats.
 *
 * @param cycleId - The cycle ID
 * @param format - Export format (markdown, pdf, html)
 * @returns A Blob containing the exported content
 */
export async function exportDocument(cycleId: string, format: ExportFormat): Promise<Blob> {
	const response = await fetch(`/api/cycles/${cycleId}/document/export?format=${format}`);

	if (!response.ok) {
		throw await ApiError.fromResponse(response);
	}

	return response.blob();
}

/**
 * Download an exported document.
 *
 * Creates a temporary link and triggers a download.
 *
 * @param cycleId - The cycle ID
 * @param format - Export format
 */
export async function downloadDocument(cycleId: string, format: ExportFormat): Promise<void> {
	const blob = await exportDocument(cycleId, format);

	const extension = format === 'markdown' ? 'md' : format;
	const filename = `decision-${cycleId}.${extension}`;

	const url = URL.createObjectURL(blob);
	const link = document.createElement('a');
	link.href = url;
	link.download = filename;
	document.body.appendChild(link);
	link.click();
	document.body.removeChild(link);
	URL.revokeObjectURL(url);
}

// ════════════════════════════════════════════════════════════════════════════════
// Document Tree Operations
// ════════════════════════════════════════════════════════════════════════════════

/**
 * Get the document tree for a session.
 *
 * Returns a hierarchical view of all documents/cycles in the session,
 * including branching relationships and PrOACT status.
 *
 * @param sessionId - The session ID
 * @returns The document tree structure
 */
export async function getDocumentTree(sessionId: string): Promise<DocumentTree> {
	const response = await fetch(`/api/sessions/${sessionId}/document-tree`);

	if (!response.ok) {
		throw await ApiError.fromResponse(response);
	}

	return response.json();
}

// ════════════════════════════════════════════════════════════════════════════════
// Branching Operations
// ════════════════════════════════════════════════════════════════════════════════

/**
 * Branch a cycle at a specific component.
 *
 * Creates a new cycle branch and its corresponding document.
 *
 * @param cycleId - The parent cycle ID
 * @param branchPoint - The component to branch at
 * @returns The new branch with its document
 */
export async function branchCycle(
	cycleId: string,
	branchPoint: ComponentType
): Promise<BranchCycleResponse> {
	const response = await fetch(`/api/cycles/${cycleId}/branch`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ branch_point: branchPoint })
	});

	if (!response.ok) {
		throw await ApiError.fromResponse(response);
	}

	return response.json();
}

// ════════════════════════════════════════════════════════════════════════════════
// Utility Functions
// ════════════════════════════════════════════════════════════════════════════════

/**
 * Check if an error is an API error.
 */
export function isApiError(error: unknown): error is ApiError {
	return error instanceof ApiError;
}

/**
 * Check if an error is a version conflict.
 */
export function isVersionConflict(error: unknown): boolean {
	return isApiError(error) && error.code === 'VERSION_CONFLICT';
}

/**
 * Check if an error is a not found error.
 */
export function isNotFound(error: unknown): boolean {
	return isApiError(error) && error.code === 'NOT_FOUND';
}
