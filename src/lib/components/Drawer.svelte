<script lang="ts">
	import type { Snippet } from 'svelte';
	import { fly, fade } from 'svelte/transition';
	import { m } from '$lib/paraglide/messages';

	let {
		open = $bindable(false),
		title,
		header,
		children
	}: {
		/** Controlled open state — toggle from a navbar button, bind:open both ways. */
		open?: boolean;
		/** Accessible dialog title. Rendered visibly unless `header` overrides it. */
		title: string;
		/** Optional visible header (tabs, custom title). When given, `title` is sr-only. */
		header?: Snippet;
		children: Snippet;
	} = $props();
</script>

<svelte:window onkeydown={(e: KeyboardEvent) => open && e.key === 'Escape' && (open = false)} />

{#if open}
	<button
		class="scrim"
		aria-label={'Close ' + title}
		transition:fade={{ duration: 200 }}
		onclick={() => (open = false)}
	></button>
	<div
		class="drawer"
		role="dialog"
		aria-modal="true"
		aria-label={title}
		transition:fly={{ x: 320, duration: 200 }}
	>
		<div class="drawer-head">
			{#if header}
				{@render header()}
				<h2 class="sr-only">{title}</h2>
			{:else}
				<h2 class="drawer-title">{title}</h2>
			{/if}
			<button class="drawer-close" aria-label={m.close()} onclick={() => (open = false)}>✕</button>
		</div>
		{@render children()}
	</div>
{/if}

<style>
	.scrim {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		border: none;
		z-index: 40;
		cursor: default;
	}
	.drawer {
		position: fixed;
		top: 0;
		right: 0;
		bottom: 0;
		width: min(20rem, 85vw);
		background: var(--bg-surface-elevated);
		border-left: var(--border-width) var(--border-style) var(--border-color);
		box-shadow: var(--shadow-lg);
		padding: var(--space-6);
		z-index: 50;
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}
	.drawer-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
	}
	.drawer-title {
		font-family: var(--font-heading);
		font-size: calc(1.25rem * var(--font-scale));
		margin: 0;
	}
	.drawer-close {
		background: none;
		border: none;
		font-size: 1.25rem;
		color: var(--color-text-muted);
		cursor: pointer;
		flex-shrink: 0;
	}
	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border: 0;
	}
</style>
