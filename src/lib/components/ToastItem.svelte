<script lang="ts">
	import { fly } from 'svelte/transition';
	import { dismiss, type Toast } from '$lib/toast.svelte';

	let { toast }: { toast: Toast } = $props();

	const DURATION = { success: 4000, info: 4000, error: 6000 } as const;

	let timer: ReturnType<typeof setTimeout> | undefined;

	function start() {
		timer = setTimeout(() => dismiss(toast.id), DURATION[toast.variant]);
	}
	function pause() {
		if (timer) clearTimeout(timer);
		timer = undefined;
	}

	$effect(() => {
		start();
		return pause;
	});
</script>

<div
	class="toast {toast.variant}"
	role={toast.variant === 'error' ? 'alert' : 'status'}
	onmouseenter={pause}
	onmouseleave={start}
	transition:fly={{ x: 24, duration: 200 }}
>
	<span class="msg">{toast.message}</span>
	<button class="close" aria-label="Dismiss" onclick={() => dismiss(toast.id)}>×</button>
</div>

<style>
	.toast {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		background: var(--bg-surface-elevated);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-left-width: 4px;
		border-radius: var(--radius-md);
		box-shadow: var(--shadow-lg);
		color: var(--color-text);
		font-size: calc(0.95rem * var(--font-scale));
	}
	.toast.success {
		border-left-color: var(--color-success);
	}
	.toast.error {
		border-left-color: var(--color-danger);
	}
	.toast.info {
		border-left-color: var(--color-primary);
	}
	.msg {
		flex: 1;
	}
	.close {
		background: none;
		border: none;
		color: var(--color-text-muted);
		font-size: 1.25rem;
		line-height: 1;
		cursor: pointer;
		padding: 0 var(--space-1);
	}
	.close:hover {
		color: var(--color-text);
	}
</style>
