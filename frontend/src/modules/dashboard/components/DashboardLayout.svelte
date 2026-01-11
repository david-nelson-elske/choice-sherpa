<!--
  DashboardLayout - Main layout container for dashboard.

  Combines the cycle tree sidebar, overview panel, and detail drawer
  into a cohesive dashboard interface.
-->

<script lang="ts">
	import type { DashboardOverview, ComponentDetailView } from '../index';
	import type { ComponentType } from '../../cycle/domain/types';
	import CycleTreeSidebar from './CycleTreeSidebar.svelte';
	import OverviewPanel from './OverviewPanel.svelte';
	import ComponentDetailDrawer from './ComponentDetailDrawer.svelte';

	interface CycleSummary {
		cycle_id: string;
		branch_point: ComponentType | null;
		progress_percent: number;
		is_active: boolean;
	}

	interface Props {
		/** Dashboard overview data */
		overview: DashboardOverview;
		/** List of cycles for the sidebar */
		cycles: CycleSummary[];
		/** Currently selected component detail */
		selectedDetail: ComponentDetailView | null;
		/** Whether detail drawer is open */
		detailDrawerOpen: boolean;
		/** Called when cycle is selected */
		onSelectCycle: (cycleId: string) => void;
		/** Called when decision statement edit is clicked */
		onEditStatement?: () => void;
		/** Called when "View Full" is clicked for a component */
		onViewComponent?: (componentType: string) => void;
		/** Called when detail drawer should close */
		onCloseDetail: () => void;
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
		overview,
		cycles,
		selectedDetail,
		detailDrawerOpen,
		onSelectCycle,
		onEditStatement,
		onViewComponent,
		onCloseDetail,
		onNavigatePrevious,
		onNavigateNext,
		onBranch,
		onRevise
	}: Props = $props();
</script>

<div class="dashboard-layout">
	<!-- Sidebar -->
	<CycleTreeSidebar
		cycles={cycles}
		activeCycleId={overview.active_cycle_id}
		onSelectCycle={onSelectCycle}
	/>

	<!-- Main Content -->
	<main class="dashboard-main">
		<OverviewPanel
			overview={overview}
			onEditStatement={onEditStatement}
			onViewComponent={onViewComponent}
		/>
	</main>

	<!-- Detail Drawer -->
	<ComponentDetailDrawer
		detail={selectedDetail}
		open={detailDrawerOpen}
		onClose={onCloseDetail}
		onNavigatePrevious={onNavigatePrevious}
		onNavigateNext={onNavigateNext}
		onBranch={onBranch}
		onRevise={onRevise}
	/>
</div>

<style>
	.dashboard-layout {
		display: flex;
		width: 100%;
		height: 100vh;
		background: #f9fafb;
	}

	.dashboard-main {
		flex: 1;
		overflow-y: auto;
		padding: 2rem;
	}

	@media (max-width: 768px) {
		.dashboard-layout {
			flex-direction: column;
			height: auto;
		}

		.dashboard-main {
			padding: 1rem;
		}
	}
</style>
