/**
 * Cycle module exports.
 *
 * This module provides everything needed to work with cycles
 * in the frontend application.
 */

// Domain types
export type {
	ComponentType,
	CycleStatus,
	ComponentStatus,
	ComponentStatusItem,
	CycleView,
	CycleSummary,
	CycleProgressView,
	CycleTreeNode,
	ComponentOutputView
} from './domain/types';

export {
	COMPONENT_ORDER,
	COMPONENT_LABELS,
	getComponentIndex,
	canStartComponent,
	getComponentLabel,
	calculateProgress
} from './domain/types';

// API client
export {
	createCycle,
	branchCycle,
	startComponent,
	completeComponent,
	updateComponentOutput,
	navigateToComponent,
	completeCycle,
	archiveCycle,
	getCycle,
	listCycles,
	getCycleTree,
	getComponentOutput,
	getCycleLineage,
	ApiError
} from './api/cycle-api';

export type {
	CreateCycleRequest,
	CreateCycleResponse,
	BranchCycleRequest,
	BranchCycleResponse,
	StartComponentRequest,
	CompleteComponentRequest,
	UpdateComponentOutputRequest,
	NavigateToComponentRequest
} from './api/cycle-api';

// Reactive stores
export {
	cycleStore,
	cycleTreeStore,
	componentOutputStore,
	isLoading,
	progressPercent,
	currentStep,
	isComplete
} from './api/stores';

// Components are imported directly in .svelte files
// Example: import CycleProgress from '$modules/cycle/components/CycleProgress.svelte';
