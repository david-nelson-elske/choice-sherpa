/**
 * Document-related types matching backend DTOs.
 */

/** Component types in PrOACT framework */
export type ComponentType =
	| 'issue_raising'
	| 'problem_frame'
	| 'objectives'
	| 'alternatives'
	| 'consequences'
	| 'tradeoffs'
	| 'recommendation'
	| 'decision_quality';

/** Component status */
export type ComponentStatus = 'not_started' | 'in_progress' | 'completed';

/** Cycle status */
export type CycleStatus = 'active' | 'completed' | 'abandoned';

/** Export format options */
export type ExportFormat = 'markdown' | 'pdf' | 'html';

/** Document format options for generation */
export type DocumentFormat = 'full' | 'summary' | 'export';

// ════════════════════════════════════════════════════════════════════════════════
// API Response Types
// ════════════════════════════════════════════════════════════════════════════════

/** Response from GET /api/cycles/:id/document */
export interface DocumentResponse {
	content: string;
	cycle_id: string;
	session_id: string;
	format: string;
}

/** Response from POST /api/cycles/:id/document/regenerate */
export interface RegenerateDocumentResponse {
	document_id: string;
	cycle_id: string;
	session_id: string;
	version: number;
	format: string;
	is_new: boolean;
	content: string;
}

/** Response from PUT /api/documents/:id */
export interface UpdateDocumentResponse {
	document_id: string;
	cycle_id: string;
	version: number;
	components_updated: number;
	parse_summary: ParseSummaryResponse;
}

/** Parse result summary */
export interface ParseSummaryResponse {
	sections_parsed: number;
	warnings: number;
	errors: number;
}

/** Response from POST /api/cycles/:id/branch */
export interface BranchCycleResponse {
	cycle_id: string;
	parent_cycle_id: string;
	document_id: string;
	branch_point: ComponentType;
	branch_label: string;
	content: string;
}

/** Standard error response */
export interface ErrorResponse {
	code: string;
	message: string;
	details?: Record<string, unknown>;
}

// ════════════════════════════════════════════════════════════════════════════════
// Document Tree Types
// ════════════════════════════════════════════════════════════════════════════════

/** PrOACT status for visualization */
export interface PrOACTStatus {
	issue_raising: ComponentStatus;
	problem_frame: ComponentStatus;
	objectives: ComponentStatus;
	alternatives: ComponentStatus;
	consequences: ComponentStatus;
	tradeoffs: ComponentStatus;
	recommendation: ComponentStatus;
	decision_quality: ComponentStatus;
}

/** A node in the document tree */
export interface DocumentTreeNode {
	document_id: string;
	cycle_id: string;
	label: string;
	proact_status: PrOACTStatus;
	branch_point?: ComponentType;
	children: DocumentTreeNode[];
}

/** Complete document tree for a session */
export interface DocumentTree {
	session_id: string;
	documents: DocumentTreeNode[];
}

// ════════════════════════════════════════════════════════════════════════════════
// Editor State Types
// ════════════════════════════════════════════════════════════════════════════════

/** Current document state for editor */
export interface DocumentState {
	cycleId: string;
	sessionId: string;
	content: string;
	version: number;
	lastSaved?: Date;
	isDirty: boolean;
}

/** Save status */
export type SaveStatus = 'idle' | 'saving' | 'saved' | 'error';

/** Editor state */
export interface EditorState {
	isEditing: boolean;
	saveStatus: SaveStatus;
	error?: string;
}
