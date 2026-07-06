<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		selected = false,
		onclick,
		class: className = '',
		children,
		...rest
	}: {
		selected?: boolean;
		onclick?: ((e: MouseEvent) => void) | undefined;
		class?: string;
		children?: Snippet;
		[key: string]: unknown;
	} = $props();
</script>

<!-- onclick present → real <button> (native keyboard/focus/semantics); else a div -->
<svelte:element
	this={onclick ? 'button' : 'div'}
	class={['card', className, { selected, interactive: !!onclick }]}
	type={onclick ? 'button' : undefined}
	aria-pressed={onclick ? selected : undefined}
	{onclick}
	{...rest}
>
	{@render children?.()}
</svelte:element>

<style>
	.card {
		appearance: none;
		background: var(--bg-surface);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-md);
		box-shadow: var(--shadow-md);
		padding: var(--space-6);
		color: var(--color-text);
		font-family: var(--font-body);
		/* buttons center+shrink by default; cards fill their cell and left-align */
		text-align: left;
		width: 100%;
		/* no-op outside a stretch context; lets cards fill a grid row */
		height: 100%;
		transition:
			border-color var(--duration-fast) var(--easing),
			box-shadow var(--duration-fast) var(--easing),
			transform var(--duration-fast) var(--easing);
	}
	.card.interactive {
		cursor: pointer;
	}
	.card.interactive:hover {
		border-color: var(--color-primary);
		box-shadow: var(--shadow-lg);
		transform: translateY(-2px);
	}
	.card.selected {
		border-color: var(--color-primary);
		box-shadow: var(--focus-ring), var(--shadow-md);
	}
	.card.interactive:focus-visible {
		outline: none;
		box-shadow: var(--focus-ring), var(--shadow-md);
	}
</style>
