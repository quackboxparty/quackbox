<script lang="ts">
	import { toasts } from '$lib/toast.svelte';
	import ToastItem from './ToastItem.svelte';
</script>

<div class="toaster">
	{#each toasts as t (t.id)}
		<ToastItem toast={t} />
	{/each}
</div>

<style>
	.toaster {
		position: fixed;
		bottom: var(--space-4);
		right: var(--space-4);
		z-index: 100;
		display: flex;
		flex-direction: column-reverse;
		gap: var(--space-2);
		pointer-events: none;
		max-width: min(90vw, 24rem);
	}
	.toaster :global(> *) {
		pointer-events: auto;
	}
</style>
