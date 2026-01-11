<!--
  Dashboard Page - Main dashboard view for a session.

  This page demonstrates how to use the dashboard components.
  In a real implementation, this would:
  1. Load dashboard data from the API
  2. Handle state management for cycle selection
  3. Handle component detail drawer state
  4. Integrate with navigation and actions
-->

<script lang="ts">
	import { page } from '$app/stores';
	import DashboardLayout from '$lib/modules/dashboard/components/DashboardLayout.svelte';
	import type { DashboardOverview, ComponentDetailView } from '$lib/modules/dashboard';
	import type { ComponentType } from '$lib/modules/cycle/domain/types';

	// In a real implementation, these would come from:
	// - API calls using getDashboardOverview()
	// - Svelte stores for state management
	// - Route parameters and navigation

	const sessionId = $page.params.sessionId;

	// Example: Mock dashboard data
	// Replace with: const overview = await getDashboardOverview(session, sessionId);
	const mockOverview: DashboardOverview = {
		session_id: sessionId,
		session_title: 'Example Decision',
		decision_statement: 'What technology stack should we use for our new product?',
		objectives: [
			{ text: 'Maximize developer productivity', type: 'fundamental', measure: 'Story points per sprint' },
			{ text: 'Minimize long-term costs', type: 'fundamental', measure: 'Total cost of ownership' },
			{ text: 'Use modern frameworks', type: 'means', measure: null }
		],
		alternatives: [
			{ name: 'React + Node.js', description: 'Popular JavaScript stack', is_status_quo: true },
			{ name: 'Svelte + Rust', description: 'Modern performant stack', is_status_quo: false },
			{ name: 'Vue + Python', description: 'Flexible proven stack', is_status_quo: false }
		],
		consequences_table: {
			objectives: ['Developer Productivity', 'Long-term Costs'],
			alternatives: ['React + Node.js', 'Svelte + Rust', 'Vue + Python'],
			cells: [
				{ objective_index: 0, alternative_index: 0, rating: 0, note: 'Baseline' },
				{ objective_index: 0, alternative_index: 1, rating: 1, note: 'Simpler code' },
				{ objective_index: 0, alternative_index: 2, rating: -1, note: 'Smaller ecosystem' },
				{ objective_index: 1, alternative_index: 0, rating: 0, note: 'Baseline' },
				{ objective_index: 1, alternative_index: 1, rating: 2, note: 'Lower hosting costs' },
				{ objective_index: 1, alternative_index: 2, rating: 1, note: 'Moderate costs' }
			]
		},
		recommendation: {
			recommended_alternative: 'Svelte + Rust',
			rationale_preview: 'While React is the status quo, Svelte + Rust offers superior performance and lower long-term costs...'
		},
		dq_score: 78,
		active_cycle_id: 'cycle-1',
		cycle_count: 2,
		last_updated: new Date().toISOString()
	};

	// Example: Mock cycles
	const mockCycles = [
		{
			cycle_id: 'cycle-1',
			branch_point: null,
			progress_percent: 85,
			is_active: true
		},
		{
			cycle_id: 'cycle-2',
			branch_point: 'alternatives' as ComponentType,
			progress_percent: 45,
			is_active: false
		}
	];

	// State for detail drawer
	let selectedDetail: ComponentDetailView | null = $state(null);
	let detailDrawerOpen = $state(false);

	// Handlers
	function handleSelectCycle(cycleId: string) {
		console.log('Select cycle:', cycleId);
		// In real implementation:
		// - Update active cycle
		// - Reload dashboard data
		// navigate(`/sessions/${sessionId}/dashboard?cycle=${cycleId}`);
	}

	function handleEditStatement() {
		console.log('Edit statement');
		// In real implementation:
		// - Navigate to problem frame component
		// navigate(`/sessions/${sessionId}/cycles/${cycleId}/components/problem_frame`);
	}

	function handleViewComponent(componentType: string) {
		console.log('View component:', componentType);
		// In real implementation:
		// - Fetch component detail
		// - Open detail drawer
		// const detail = await getComponentDetail(session, cycleId, componentType);
		// selectedDetail = detail;
		// detailDrawerOpen = true;
	}

	function handleCloseDetail() {
		detailDrawerOpen = false;
		selectedDetail = null;
	}

	function handleNavigatePrevious() {
		console.log('Navigate previous');
		// In real implementation: load previous component detail
	}

	function handleNavigateNext() {
		console.log('Navigate next');
		// In real implementation: load next component detail
	}

	function handleBranch() {
		console.log('Branch from component');
		// In real implementation: create branch cycle
	}

	function handleRevise() {
		console.log('Revise component');
		// In real implementation: navigate to component for editing
	}
</script>

<svelte:head>
	<title>Dashboard - {mockOverview.session_title}</title>
</svelte:head>

<DashboardLayout
	overview={mockOverview}
	cycles={mockCycles}
	selectedDetail={selectedDetail}
	detailDrawerOpen={detailDrawerOpen}
	onSelectCycle={handleSelectCycle}
	onEditStatement={handleEditStatement}
	onViewComponent={handleViewComponent}
	onCloseDetail={handleCloseDetail}
	onNavigatePrevious={handleNavigatePrevious}
	onNavigateNext={handleNavigateNext}
	onBranch={handleBranch}
	onRevise={handleRevise}
/>
