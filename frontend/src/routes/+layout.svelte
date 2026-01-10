<script lang="ts">
	import '../app.css';
</script>

<div class="app-container">
	<slot />
</div>

<style>
	.app-container {
		min-height: 100vh;
		display: flex;
		flex-direction: column;
	}
</style>
